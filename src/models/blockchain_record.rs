use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlockchainRecord {
    pub id: i32,
    pub document_id: i32,
    pub transaction_hash: String,
    pub block_number: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BlockchainStatus {
    pub is_healthy: bool,
    pub total_documents: i64,
    pub last_block: i64,
    pub network_status: String,
}
