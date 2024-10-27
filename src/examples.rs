let signer = PrivateKeySigner::random();
signers.push(signer.clone());
let address = signer.address();

provider.anvil_set_balance(address, balance).await?;
let wallet = EthereumWallet::from(signer);
let provider_with_wallet = ProviderBuilder::new().with_recommended_fillers().wallet(wallet).on_provider(&provider);

// Send 10 eth to bob
let bob_address = bob.address();

let tx =
TransactionRequest::default().with_to(bob_address).with_value(U256::from(1000000000));

// // Send the transaction and listen for the transaction to be included.
let tx_hash = provider_with_wallet.send_transaction(tx).await?.watch().await?;

// Set allowance for the disperse contract.
let max_allowance = U256::MAX;
let _ = mock_token.approve(disperse_address, max_allowance).send().await?;



// async fn test(state: &State) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = &state.provider;

//     let accounts = provider.get_accounts().await?;
//     let alice = accounts[0];
//     let bob = accounts[1];

//     let alice_balance = provider.get_balance(alice).await?;
//     println!("Alice balance: {}", format_ether(alice_balance));

//     let bob_balance = provider.get_balance(bob).await?;
//     println!("Bob balance: {}", format_ether(bob_balance));

//     Ok(())
// }

// async fn test_transfers(state: &State) -> Result<(), Box<dyn std::error::Error>> {
//     let State { provider, mock_token } = state;

//     let accounts = provider.get_accounts().await?;
//     let alice = accounts[0];
//     let bob = accounts[1];


//     let alice_balance = mock_token.balanceOf(alice).call().await?._0;
//     println!("Alice balance before: {}", alice_balance);

//     let pu = parse_ether("1000")?;
//     let num: U256 = pu.into();

//     // Get alice eth balance
//     let alice_balance = provider.get_balance(alice).await?;

//     // Print in ether
//     println!("Alice ether balance: {}", format_ether(alice_balance));

//     // Mint 1000 tokens to Alice.
//     mock_token.mint(alice, num).send().await?;


//     // Alice should have 1000 tokens.
//     let alice_balance = mock_token.balanceOf(alice).call().await?._0;
//     println!("Alice balance after: {}", alice_balance);

//     let transfer_amount = parse_ether("100")?;

//     // Transfer 100 tokens from Alice to Bob.
//     mock_token.transfer(bob, transfer_amount).send().await?;

//     // Alice should have 900 tokens.
//     let alice_balance = mock_token.balanceOf(alice).call().await?._0;
//     println!("Alice balance after transfer: {}", format_ether(alice_balance));

//     // println!("Alice: {}", alice); 
//     // println!("Bob: {}", bob);

//     Ok(())
// }



    // Mint 1000 tokens to first account.
    let num: U256 = parse_ether("1000")?;
    let accounts = provider.get_accounts().await?;
    let alice = accounts[0];
    // mock_token.mint(alice, num).send().await?;


    // Set allowance for the disperse contract.
    // let allowance = parse_ether("1000000")?;

    // Transfer tokens from account 2 to disperse contract.
    // let accounts = provider.get_accounts().await?;
    // let alice = accounts[0];
    // let bob = accounts[1];

    // mock_token.transfer(disperse_contract.address(), allowance).send().await?;