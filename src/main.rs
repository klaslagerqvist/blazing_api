use actix_web::{error::ErrorBadRequest, get, post, web, App, HttpResponse, HttpServer, Responder};

use alloy::{
    network::{EthereumWallet, TransactionBuilder}, primitives::{
        utils::{format_ether, parse_ether},
        Address,
        U256
    },
    providers::{ext::AnvilApi, Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
    transports::http::{reqwest, Client, Http}
};

mod models;
use models::{AssetType, CollectRequest, DisperseRequest, OneTransactionHashResponse, MultipleTransactionHashResponse};

use eyre::Result;
use std::env;
use dotenv::dotenv;

// Mock ERC20 token contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    MockToken,
    "artifacts/MockToken.json"
);

// Disperse & collect contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Disperse,
    "artifacts/Disperse.json"
);

const ANVIL_PORT: u16 = 8545;
const ANVIL_HOST: &str = "http://127.0.0.1";
const ANVIL_FORK_URL: &str = "https://eth.merkle.io";

// Shared app state (shared between endpoints).
struct State {
    provider: Box<dyn Provider<Http<Client>>>,
    rpc_url: reqwest::Url,
    mock_token_address: Address,
    disperse_address: Address,
    // List of signers (generated random wallets).
    signers: Vec<PrivateKeySigner>,
}

fn get_signer<'a>(signers: &'a Vec<PrivateKeySigner>, index: usize) -> Result<&'a PrivateKeySigner, actix_web::Error> {
    signers.get(index).ok_or_else(|| ErrorBadRequest("Invalid wallet index"))
}

fn parse_amount(amount_str: &str) -> Result<U256, actix_web::Error> {
    amount_str.parse::<U256>()
        .map_err(|_| ErrorBadRequest("Invalid amount format: amount must be a string representing a number in WEI units"))
}

async fn disperse_assets(state: web::Data<State>, info: web::Json<DisperseRequest>) -> Result<HttpResponse, actix_web::Error> {
    let is_percentage = info.is_percentage;

    // Get the signer.
    let signer = get_signer(&state.signers, info.from as usize)?;
    let from = signer.address();

    let wallet = EthereumWallet::from(signer.clone());
    // Create a provider with the wallet.
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(state.rpc_url.clone());

    let mut recipients = Vec::new();
    // Validate the recipients.
    for index in &info.to {
        let signer = match state.signers.get(*index as usize) {
            Some(signer) => signer,
            None => return Err(ErrorBadRequest("Invalid recipient wallet index")),
        };

        recipients.push(signer.address());
    }

    let parsed_amount: U256 = parse_amount(&info.amount)?;
    let mut amount_to_disperse = parsed_amount;

    // Initialize the disperse contract.
    let disperse_contract = Disperse::new(state.disperse_address, &provider);

    let transaction_hash;

    match info.asset_type {
        AssetType::Eth => {
            // Get balance of the sender.
            let balance = provider
                .get_balance(from)
                .await
                .map_err(|e| ErrorBadRequest(format!("Failed to get ETH balance: {}", e)))?;
    
            if is_percentage {
                println!("Dispersing {}% of total ETH balance", info.amount);
                amount_to_disperse = balance * parsed_amount / U256::from(100);
            }
 
            if amount_to_disperse > balance {
                return Err(ErrorBadRequest("Insufficient ETH balance in source wallet"));
            }

            println!("Dispersing {} ETH", format_ether(amount_to_disperse));

            // Call disperseEth method on the contract.
            let pending_transaction = disperse_contract
                .disperseEth(recipients)
                .value(amount_to_disperse)
                .send()
                .await
                .map_err(|e| ErrorBadRequest(format!("Failed to disperse ETH: {}", e)))?;

            // Get the transaction hash.
            transaction_hash = pending_transaction.inner().tx_hash().to_string();
        }
        AssetType::Token => {
            // Set up the token contract.
            let token_contract = MockToken::new(state.mock_token_address, &provider);
            
            // Get balance of the sender.
            let balance = token_contract
                .balanceOf(from)
                .call()
                .await
                .map_err(|e| ErrorBadRequest(format!("Failed to get token balance: {}", e)))?
                ._0;

            if is_percentage {
                println!("Dispersing {}% of total token balance", info.amount);
                amount_to_disperse = balance * parsed_amount / U256::from(100);
            }

            // Check if the amount to disperse is greater than the balance.
            if amount_to_disperse > balance {
                return Err(ErrorBadRequest("Insufficient token balance"));
            }

            println!("Dispersing {} tokens", amount_to_disperse);

            // Call disperseTokens method on the contract.
            let pending_transaction = disperse_contract
                .disperseTokens(state.mock_token_address, recipients, amount_to_disperse)
                .send()
                .await
                .map_err(|e| ErrorBadRequest(format!("Failed to disperse tokens: {}", e)))?;

            // Get the transaction hash.
            transaction_hash = pending_transaction.inner().tx_hash().to_string();
        }
    }

    Ok(HttpResponse::Ok().json(OneTransactionHashResponse {
        transaction_hash
    }))
}

async fn collect_assets(state: web::Data<State>, info: web::Json<CollectRequest>) -> Result<HttpResponse, actix_web::Error> {
    // Get the recipient
    let recipient_signer = get_signer(&state.signers, info.to as usize)?;
    let recipient = recipient_signer.address();

    let mut transaction_hashes = Vec::new();
    let parsed_amount = parse_amount(&info.amount)?;

    for &sender_index in &info.from {
        // Get the sender
        let signer = get_signer(&state.signers, sender_index as usize)?;
        let from = signer.address();

        // Set up the provider with the wallet
        let wallet = EthereumWallet::from(signer.clone());
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(state.rpc_url.clone());

        let mut amount_to_collect = parsed_amount;

        match info.asset_type {
            AssetType::Eth => {
                let balance = provider
                    .get_balance(from)
                    .await
                    .map_err(|e| ErrorBadRequest(format!("Failed to get ETH balance: {}", e)))?;

                // Handle percentage amount
                if info.is_percentage {
                    amount_to_collect = balance * parsed_amount / U256::from(100);
                }

                // Check if the amount to collect is greater than the balance
                if amount_to_collect > balance {
                    return Err(ErrorBadRequest("Insufficient ETH balance"));
                }

                println!("Collecting {} ETH from {} to {}", format_ether(amount_to_collect), from, recipient);

                // Transfer eth to the recipient
                let tx = TransactionRequest::default().with_to(recipient).with_value(amount_to_collect);
                let pending_transaction = provider
                    .send_transaction(tx)
                    .await
                    .map_err(|e| ErrorBadRequest(format!("Failed to send ETH: {}", e)))?;

                transaction_hashes.push(pending_transaction.inner().tx_hash().to_string());
            }
            AssetType::Token => {
                let token_contract = MockToken::new(state.mock_token_address, provider.clone());
                let balance = token_contract
                    .balanceOf(from)
                    .call()
                    .await
                    .map_err(|e| ErrorBadRequest(format!("Failed to get token balance: {}", e)))?
                    ._0;

                // Handle percentage amount
                if info.is_percentage {
                    println!("Collecting {}% of total token balance", info.amount);
                    amount_to_collect = balance * parsed_amount / U256::from(100);
                }

                if amount_to_collect > balance {
                    return Err(ErrorBadRequest("Insufficient token balance"));
                }

                println!("Collecting {} tokens from {} to {}", format_ether(amount_to_collect), from, recipient);

                // Transfer tokens to the recipient
                let pending_transaction = token_contract
                    .transfer(recipient, amount_to_collect)
                    .send()
                    .await
                    .map_err(|e| ErrorBadRequest(format!("Failed to transfer tokens: {}", e)))?;

                transaction_hashes.push(pending_transaction.inner().tx_hash().to_string());
            }
        }
    }

    Ok(HttpResponse::Ok().json(MultipleTransactionHashResponse {
        transaction_hashes,
    }))
}

async fn setup_anvil() -> Result<State, Box<dyn std::error::Error>> {
    // Spin up a forked Anvil node.
    // Ensure `anvil` is available in $PATH.
    let anvil_rpc_url: reqwest::Url = format!("{ANVIL_HOST}:{ANVIL_PORT}").parse().unwrap();
    
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_anvil_with_wallet_and_config(|anvil| 
            anvil.fork(ANVIL_FORK_URL)
                .port(ANVIL_PORT)
    );


    let balance = parse_ether("1000")?;

    let mock_token =  MockToken::deploy(&provider, "Mock Token".into(),"MOCK".into()).await?;
    let disperse_contract = Disperse::deploy(&provider).await?;
    let mock_token_address = mock_token.address().clone();
    let disperse_address = disperse_contract.address().clone();

    let mut signers: Vec<PrivateKeySigner> = Vec::new();

    // Create 10 random wallets with 1000 ETH each, and 1000 tokens. Also set allowance for the disperse contract.
    for _ in 0..10 {
        let signer = PrivateKeySigner::random();
        signers.push(signer.clone());
        let address = signer.address();

        provider.anvil_set_balance(address, balance).await?;
        let wallet = EthereumWallet::from(signer);

        let provider_with_wallet = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_provider(&provider);

        let token_instance = MockToken::new(mock_token_address, &provider_with_wallet);

        // Set max allowance for the wallet to the disperse contract. 
        let _ = token_instance.approve(disperse_address, U256::MAX).send().await?;

        // Mint 1000 tokens to the wallet.
        let _ = mock_token.mint(address, parse_ether("1000")?).send().await?;
    }

    // Print account index, address and their balances
    for (index, account) in signers.iter().enumerate() {
        let address = account.address();
        let balance = provider.get_balance(address).await?;
        println!("Account {}: {} - {}", index, address, format_ether(balance));
    }

    // Setup app state.
    let state = State {
        // It seems that when the provider goes out of scope, the anvil node is killed.
        // So we need to keep the anvil provider alive for the lifetime of the application.
        provider: Box::new(provider),
        rpc_url: anvil_rpc_url,
        mock_token_address,
        disperse_address,
        signers,
    };

    Ok(state)
}

#[get("/balances")]
async fn balances(state: web::Data<State>) -> impl Responder {
    let provider = ProviderBuilder::new().on_http(state.rpc_url.clone());
    let signers = &state.signers;
    let mut balances = Vec::new();

    let token_contract = MockToken::new(state.mock_token_address, &provider);

    for signer in signers {
        let balance = provider.get_balance(signer.address()).await.unwrap();
        let token_balance = token_contract.balanceOf(signer.address()).call().await.unwrap()._0;
        balances.push(format!("Address {}, ETH {} Tokens: {}", signer.address(), format_ether(balance), format_ether(token_balance)));
    }

    HttpResponse::Ok().body(balances.join("\n"))
}

#[post("/disperse_assets")]
async fn disperse_assets_endpoint(state: web::Data<State>, info: web::Json<DisperseRequest>) -> Result<HttpResponse, actix_web::Error> {
    disperse_assets(state, info).await
}

#[post("/collect_assets")]
async fn collect_assets_endpoint(state: web::Data<State>, info: web::Json<CollectRequest>) -> Result<HttpResponse, actix_web::Error> {
    collect_assets(state, info).await
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // Start anvil and deploy contracts.
    let state = setup_anvil().await.unwrap();

    // Create app state.
    let app_state = web::Data::new(state);

    let port = env::var("PORT").unwrap_or("8080".to_string());
    println!("Starting server at http://127.0.0.1:{}", port);

    HttpServer::new(move || {
        App::new().service(
            web::scope("/api")
            .app_data(app_state.clone())
            .service(disperse_assets_endpoint)
            .service(collect_assets_endpoint)
            .service(balances)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}