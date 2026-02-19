use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: i32,
    pub user_id: i32,
    pub file_name: String,
    pub document_hash: String,
    pub file_size: i64,
    pub mime_type: String,
    pub s3_url: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UploadDocumentRequest {
    pub file_name: String,
    pub file_content: String, // base64 encoded
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyDocumentRequest {
    pub document_hash: Option<String>,
    pub file_content: Option<String>, // base64 encoded for hash calculation
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: i32,
    pub file_name: String,
    pub document_hash: String,
    pub file_size: i64,
    pub mime_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub blockchain_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    pub exists: bool,
    pub document_hash: String,
    pub owner: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub blockchain_confirmed: bool,
}

#[derive(Debug, Deserialize)]
pub struct TransferOwnershipRequest {
    pub new_owner_email: String,
}
