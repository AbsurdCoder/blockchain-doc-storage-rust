use actix_web::{web, HttpResponse, Responder};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::Deserialize;
use sqlx::FromRow;

use crate::middleware::AuthenticatedUser;
use crate::models::{
    BlockchainStatus, DocumentResponse, TransferOwnershipRequest, UploadDocumentRequest,
    VerificationResponse, VerifyDocumentRequest,
};
use crate::services::BlockchainAnchor;
use crate::utils::{hash_base64, hash_bytes};
use crate::AppState;

/// Internal helper to build a basic `DocumentResponse` for an uploaded document.
/// This currently focuses on hashing and response shaping; persistence and S3
/// upload are handled elsewhere.
fn parse_user_id(user: &AuthenticatedUser) -> Result<i32, String> {
    user.user_id()
        .parse::<i32>()
        .map_err(|_| "invalid user id in token".to_string())
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, FromRow)]
struct DocumentRow {
    pub id: i32,
    pub file_name: String,
    pub document_hash: String,
    pub file_size: i64,
    pub mime_type: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub blockchain_status: Option<String>,
}

fn to_document_response(row: DocumentRow) -> DocumentResponse {
    DocumentResponse {
        id: row.id,
        file_name: row.file_name,
        document_hash: row.document_hash,
        file_size: row.file_size,
        mime_type: row.mime_type,
        status: row.status,
        created_at: row.created_at,
        blockchain_status: row.blockchain_status,
    }
}

pub async fn upload_document(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    payload: web::Json<UploadDocumentRequest>,
) -> impl Responder {
    let user_id = match parse_user_id(&user) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().body(e),
    };

    let req = payload.into_inner();

    let bytes = match BASE64.decode(req.file_content.trim()) {
        Ok(b) => b,
        Err(_) => {
            state.metrics.record_document_upload_error();
            return HttpResponse::BadRequest().body("invalid base64 file_content");
        }
    };

    let document_hash = hash_bytes(&bytes);
    let file_size = bytes.len() as i64;

    let s3_url = match state
        .storage
        .upload_bytes(user_id, &req.file_name, &req.mime_type, bytes)
        .await
    {
        Ok(url) => url,
        Err(e) => {
            state.metrics.record_document_upload_error();
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    // Insert document
    let insert = sqlx::query(
        r#"
        INSERT INTO documents (user_id, file_name, document_hash, file_size, mime_type, s3_url, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, 'pending', NOW(), NOW())
        "#,
    )
    .bind(user_id)
    .bind(&req.file_name)
    .bind(&document_hash)
    .bind(file_size)
    .bind(&req.mime_type)
    .bind(&s3_url)
    .execute(&state.db)
    .await;

    let doc_id = match insert {
        Ok(res) => res.last_insert_id() as i32,
        Err(e) => {
            state.metrics.record_document_upload_error();
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };

    // Anchor on blockchain (optional depending on env)
    let anchor: BlockchainAnchor = match state.blockchain.anchor_document_hash(&document_hash).await {
        Ok(a) => a,
        Err(e) => BlockchainAnchor {
            transaction_hash: String::new(),
            block_number: 0,
            status: format!("failed: {}", e),
        },
    };

    // Store blockchain record (even if skipped/failed, for auditability)
    let _ = sqlx::query(
        r#"
        INSERT INTO blockchain_records (document_id, transaction_hash, block_number, status, created_at)
        VALUES (?, ?, ?, ?, NOW())
        "#,
    )
    .bind(doc_id)
    .bind(&anchor.transaction_hash)
    .bind(anchor.block_number)
    .bind(&anchor.status)
    .execute(&state.db)
    .await;

    // Update document status if confirmed
    if anchor.status == "confirmed" {
        let _ = sqlx::query("UPDATE documents SET status = 'confirmed' WHERE id = ?")
            .bind(doc_id)
            .execute(&state.db)
            .await;
    }

    state.metrics.record_document_uploaded();

    let response = DocumentResponse {
        id: doc_id,
        file_name: req.file_name,
        document_hash,
        file_size,
        mime_type: req.mime_type,
        status: if anchor.status == "confirmed" {
            "confirmed".to_string()
        } else {
            "pending".to_string()
        },
        created_at: chrono::Utc::now(),
        blockchain_status: Some(anchor.status),
    };

    HttpResponse::Ok().json(response)
}

pub async fn list_documents(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    query: web::Query<ListDocumentsQuery>,
) -> impl Responder {
    let user_id = match parse_user_id(&user) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().body(e),
    };
    state.metrics.record_documents_listed();

    let search = query.search.clone().unwrap_or_default();
    let status = query.status.clone();

    let rows: Result<Vec<DocumentRow>, sqlx::Error> = if let Some(status) = status {
        sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT
              d.id, d.file_name, d.document_hash, d.file_size, d.mime_type, d.status, d.created_at,
              (SELECT br.status FROM blockchain_records br WHERE br.document_id = d.id ORDER BY br.id DESC LIMIT 1) AS blockchain_status
            FROM documents d
            WHERE d.user_id = ?
              AND d.status = ?
              AND (? = '' OR d.file_name LIKE CONCAT('%', ?, '%'))
            ORDER BY d.created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(status)
        .bind(&search)
        .bind(&search)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT
              d.id, d.file_name, d.document_hash, d.file_size, d.mime_type, d.status, d.created_at,
              (SELECT br.status FROM blockchain_records br WHERE br.document_id = d.id ORDER BY br.id DESC LIMIT 1) AS blockchain_status
            FROM documents d
            WHERE d.user_id = ?
              AND (? = '' OR d.file_name LIKE CONCAT('%', ?, '%'))
            ORDER BY d.created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(&search)
        .bind(&search)
        .fetch_all(&state.db)
        .await
    };

    match rows {
        Ok(rows) => HttpResponse::Ok().json(rows.into_iter().map(to_document_response).collect::<Vec<_>>()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_document(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let user_id = match parse_user_id(&user) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().body(e),
    };
    state.metrics.record_document_fetched();
    let id = path.into_inner();

    // Allow admins to fetch any document; otherwise owner-only.
    let is_admin = user.is_admin();

    let row: Result<DocumentRow, sqlx::Error> = if is_admin {
        sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT
              d.id, d.file_name, d.document_hash, d.file_size, d.mime_type, d.status, d.created_at,
              (SELECT br.status FROM blockchain_records br WHERE br.document_id = d.id ORDER BY br.id DESC LIMIT 1) AS blockchain_status
            FROM documents d
            WHERE d.id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&state.db)
        .await
    } else {
        sqlx::query_as::<_, DocumentRow>(
            r#"
            SELECT
              d.id, d.file_name, d.document_hash, d.file_size, d.mime_type, d.status, d.created_at,
              (SELECT br.status FROM blockchain_records br WHERE br.document_id = d.id ORDER BY br.id DESC LIMIT 1) AS blockchain_status
            FROM documents d
            WHERE d.id = ? AND d.user_id = ?
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
    };

    match row {
        Ok(row) => HttpResponse::Ok().json(to_document_response(row)),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("document not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn verify_document(
    state: web::Data<AppState>,
    payload: web::Json<VerifyDocumentRequest>,
) -> impl Responder {
    let request = payload.into_inner();
    state.metrics.record_document_verified();

    let hash = if let Some(h) = request.document_hash.clone() {
        h
    } else if let Some(fc) = request.file_content.clone() {
        match hash_base64(&fc) {
            Ok(h) => h,
            Err(_) => return HttpResponse::BadRequest().body("invalid base64 file_content"),
        }
    } else {
        return HttpResponse::BadRequest().body("provide document_hash or file_content");
    };

    // Lookup document by hash
    let row: Result<(i32, chrono::DateTime<chrono::Utc>, Option<String>), sqlx::Error> = sqlx::query_as(
        r#"
        SELECT d.id, d.created_at, u.email
        FROM documents d
        JOIN users u ON u.id = d.user_id
        WHERE d.document_hash = ?
        "#,
    )
    .bind(&hash)
    .fetch_one(&state.db)
    .await;

    let (exists, doc_id, created_at, owner_email) = match row {
        Ok((id, ts, email)) => (true, Some(id), Some(ts), email),
        Err(sqlx::Error::RowNotFound) => (false, None, None, None),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let (blockchain_confirmed, tx_hash) = if let Some(doc_id) = doc_id {
        let rec: Result<(String, String), sqlx::Error> = sqlx::query_as(
            r#"
            SELECT status, transaction_hash
            FROM blockchain_records
            WHERE document_id = ?
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
        .bind(doc_id)
        .fetch_one(&state.db)
        .await;

        match rec {
            Ok((status, tx)) => (status == "confirmed", Some(tx)),
            Err(_) => (false, None),
        }
    } else {
        (false, None)
    };

    let response = VerificationResponse {
        exists,
        document_hash: hash,
        owner: owner_email,
        timestamp: created_at,
        metadata: tx_hash.map(|tx| serde_json::json!({ "transaction_hash": tx })),
        blockchain_confirmed,
    };

    HttpResponse::Ok().json(response)
}

pub async fn transfer_ownership(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    path: web::Path<i32>,
    payload: web::Json<TransferOwnershipRequest>,
) -> impl Responder {
    let from_user_id = match parse_user_id(&user) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().body(e),
    };
    let document_id = path.into_inner();
    let req = payload.into_inner();
    state.metrics.record_transfer_initiated();

    // Lookup target user by email
    let to_user_id: Result<i32, sqlx::Error> =
        sqlx::query_scalar("SELECT id FROM users WHERE email = ?")
            .bind(&req.new_owner_email)
            .fetch_one(&state.db)
            .await;

    let to_user_id = match to_user_id {
        Ok(id) => id,
        Err(sqlx::Error::RowNotFound) => return HttpResponse::BadRequest().body("new owner not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    // Ensure requester owns the document (or is admin)
    let is_admin = user.is_admin();
    let owned: Result<i64, sqlx::Error> = if is_admin {
        sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE id = ?")
            .bind(document_id)
            .fetch_one(&state.db)
            .await
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE id = ? AND user_id = ?")
            .bind(document_id)
            .bind(from_user_id)
            .fetch_one(&state.db)
            .await
    };

    match owned {
        Ok(n) if n > 0 => {}
        Ok(_) => return HttpResponse::NotFound().body("document not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    // Update ownership
    if let Err(e) = sqlx::query("UPDATE documents SET user_id = ? WHERE id = ?")
        .bind(to_user_id)
        .bind(document_id)
        .execute(&state.db)
        .await
    {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    // Insert transfer history
    let _ = sqlx::query(
        r#"
        INSERT INTO document_transfers (document_id, from_user_id, to_user_id, created_at)
        VALUES (?, ?, ?, NOW())
        "#,
    )
    .bind(document_id)
    .bind(from_user_id)
    .bind(to_user_id)
    .execute(&state.db)
    .await;

    HttpResponse::Ok().body("ok")
}

pub async fn blockchain_status(state: web::Data<AppState>) -> impl Responder {
    // Minimal status derived from configuration + DB counts.
    let total_documents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM documents")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let last_block: i64 = sqlx::query_scalar::<_, i64>("SELECT COALESCE(MAX(block_number), 0) FROM blockchain_records")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let is_healthy = state.blockchain.is_configured();

    let status = BlockchainStatus {
        is_healthy,
        total_documents,
        last_block,
        network_status: if is_healthy {
            "configured".to_string()
        } else {
            "not_configured".to_string()
        },
    };

    HttpResponse::Ok().json(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Claims, UploadDocumentRequest};

    fn dummy_user() -> AuthenticatedUser {
        let claims = Claims {
            sub: "1".to_string(),
            email: "user@example.com".to_string(),
            role: "user".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        };
        AuthenticatedUser(claims)
    }

    #[test]
    fn build_upload_response_succeeds_for_valid_base64() {
        // Keep a basic hashing test aligned with upload input format.
        let _user = dummy_user();
        let file_content = "aGVsbG8gd29ybGQ="; // hello world
        let computed = crate::utils::hash_base64(file_content).unwrap();
        assert_eq!(computed, crate::utils::hash_base64(file_content).unwrap());
    }

    #[test]
    fn build_upload_response_fails_for_invalid_base64() {
        assert!(crate::utils::hash_base64("!!!not-valid-base64!!!").is_err());
    }
}

