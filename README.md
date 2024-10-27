# Installation

Install forge and rust.

Then, make sure anvil is available in your PATH as the server spins up an anvil node.

# Running the server

Run with `cargo run`.

The server will spin up an anvil node and deploy the Disperse contract and a mock ERC20 token contract to it. You don't need to deploy the contracts manually.

Then it creates 10 new random wallets funded with 1000 ETH and 1000 mock ERC20 tokens each.

You can disperse and collect with these wallets using the endpoints described below.

# Use the API

You can make requests to the API with a tool like curl or Postman. Here are curl examples that you can copy and paste.

```bash


Disperse assets:

curl --location 'http://127.0.0.1:8080/api/disperse_assets' \
--header 'Content-Type: application/json' \
--data '{
    "amount": "25",
    "is_percentage": true,
    "from": 0,
    "to": [
        9,
        2
   ],
   "asset_type": "Token"
}'

Collect assets:

curl --location 'http://127.0.0.1:8080/api/collect_assets' \
--header 'Content-Type: application/json' \
--data '{
    "amount": "50",
    "is_percentage": true,
    "from": [0, 1, 2, 3],
    "to": 9,
    "asset_type": "Token"
}'

Check balances for debugging:

curl --location 'http://127.0.0.1:8080/api/balances'

```

# Endpoints

All endpoints use wallet indexes to identify the sender and recipient wallets. Make sure the indexes provided in the request correspond to valid wallet addresses (there are 10 test wallets available, indexed from 0 to 9). 

The asset type is specified as `Eth` for ETH and `Token` for the mock token deployed on the anvil node.

## 127.0.0.1:8080/api/disperse_assets

This endpoint is used to distribute ETH or ERC20 tokens from a specified wallet to multiple recipient wallets. The amount to be dispersed can be defined as a specific value or as a percentage of the sender's balance.

**Request Method**: `POST`

**JSON Body Structure**:

```json
{
    "amount": "25", // Disperse 25% of the sender's balance
    "is_percentage": true,
    "from": 9, // Index of the sender wallet
    "to": [1, 2], // List of recipient wallet indexes
    "asset_type": "Eth"
}
```

- `amount`: The amount to disperse. If `is_percentage` is `true`, this represents the percentage of the sender's balance or token balance to be dispersed. Otherwise, it represents the fixed amount in WEI.
- `is_percentage`: Boolean indicating if the amount is a percentage.
- `from`: Index of the sender wallet.
- `to`: List of recipient wallet indexes.
- `asset_type`: The type of asset to disperse. Can be either `Eth` or `Token`.


## 127.0.0.1:8080/api/collect_assets

This endpoint is used to collect ETH or ERC20 tokens from multiple sender wallets and send them to a single recipient wallet. The amount to be collected can be defined as a specific value or as a percentage of each sender's balance.

**Request Method**: `POST`

**JSON Body Structure**:

```json
{
    "amount": "50",
    "is_percentage": true,
    "from": [1, 2, 3],
    "to": 1
}
```

- `amount`: The amount to collect. If `is_percentage` is `true`, this represents the percentage of each sender's balance or token balance to be collected. Otherwise, it represents the fixed amount in WEI.
- `is_percentage`: Boolean indicating if the amount is a percentage.
- `from`: List of sender wallet indexes.
- `to`: Index of the recipient wallet.
- `asset_type`: The type of asset to collect. Can be either `Eth` or `Token`.

## 127.0.0.1:8080/api/balances

**Request Method**: `GET`

Call this method to fetch the eth and token balances of all wallets, useful for debugging.