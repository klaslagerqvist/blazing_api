use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub enum AssetType {
    Eth,
    Token,
}

#[derive(Deserialize)]
pub struct DisperseRequest {
    pub from: u32, // From wallet index.
    pub amount: String,
    pub is_percentage: bool, // If true, use percentage, else use amount.
    pub to: Vec<u32>, // To wallet indexes.
    pub asset_type: AssetType,
}

#[derive(Deserialize)]
pub struct CollectRequest {
    pub from: Vec<u32>, // List of sender wallet indexes.
    pub amount: String,
    pub is_percentage: bool, // If true, use percentage, else use amount.
    pub to: u32, // Recipient wallet index.
    pub asset_type: AssetType,
}

#[derive(Serialize)]
pub struct OneTransactionHashResponse {
    pub transaction_hash: String,
}

#[derive(Serialize)]
pub struct MultipleTransactionHashResponse {
    pub transaction_hashes: Vec<String>,
}
