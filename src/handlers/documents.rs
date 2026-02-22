use actix_web::{web, HttpResponse, Responder};

use crate::middleware::AuthenticatedUser;
use crate::models::{
    Document, DocumentResponse, TransferOwnershipRequest, UploadDocumentRequest,
    VerificationResponse, VerifyDocumentRequest,
};
use crate::utils::hash_base64;
use crate::AppState;

/// Internal helper to build a basic `DocumentResponse` for an uploaded document.
/// This currently focuses on hashing and response shaping; persistence and S3
/// upload are handled elsewhere.
fn build_upload_response(
    user: &AuthenticatedUser,
    request: &UploadDocumentRequest,
) -> Result<DocumentResponse, String> {
    // Compute SHA-256 hash from the base64-encoded content
    let document_hash =
        hash_base64(&request.file_content).map_err(|_| "invalid base64 file_content".to_string())?;

    // For now we don't persist to DB or S3 here; just return a basic response.
    // File size and blockchain status can be filled in by later enhancements.
    let response = DocumentResponse {
        id: 0,
        file_name: request.file_name.clone(),
        document_hash,
        file_size: 0,
        mime_type: request.mime_type.clone(),
        status: "pending".to_string(),
        created_at: chrono::Utc::now(),
        blockchain_status: None,
    };

    // `user` is currently unused but kept to make it easy to add
    // ownership/authorization logic here later.
    let _ = user;

    Ok(response)
}

pub async fn upload_document(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    payload: web::Json<UploadDocumentRequest>,
) -> impl Responder {
    match build_upload_response(&user, &payload.into_inner()) {
        Ok(response) => {
            state.metrics.record_document_uploaded();
            HttpResponse::Ok().json(response)
        }
        Err(msg) => {
            state.metrics.record_document_upload_error();
            HttpResponse::BadRequest().body(msg)
        }
    }
}

pub async fn list_documents(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    _query: web::Query<serde_json::Value>,
) -> impl Responder {
    let _user_id = user.user_id();
    state.metrics.record_documents_listed();

    let empty: Vec<DocumentResponse> = Vec::new();
    HttpResponse::Ok().json(empty)
}

pub async fn get_document(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let _user_id = user.user_id();
    state.metrics.record_document_fetched();
    let id = path.into_inner();

    let doc = Document {
        id,
        user_id: 0,
        file_name: String::new(),
        document_hash: String::new(),
        file_size: 0,
        mime_type: String::new(),
        s3_url: String::new(),
        status: String::from("pending"),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let response = DocumentResponse {
        id: doc.id,
        file_name: doc.file_name,
        document_hash: doc.document_hash,
        file_size: doc.file_size,
        mime_type: doc.mime_type,
        status: doc.status,
        created_at: doc.created_at,
        blockchain_status: None,
    };

    HttpResponse::Ok().json(response)
}

pub async fn verify_document(
    state: web::Data<AppState>,
    payload: web::Json<VerifyDocumentRequest>,
) -> impl Responder {
    let request = payload.into_inner();
    state.metrics.record_document_verified();

    let response = VerificationResponse {
        exists: request.document_hash.is_some() || request.file_content.is_some(),
        document_hash: request
            .document_hash
            .unwrap_or_else(|| "computed_hash_placeholder".to_string()),
        owner: None,
        timestamp: None,
        metadata: None,
        blockchain_confirmed: false,
    };

    HttpResponse::Ok().json(response)
}

pub async fn transfer_ownership(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    path: web::Path<i32>,
    payload: web::Json<TransferOwnershipRequest>,
) -> impl Responder {
    let _user_id = user.user_id();
    let _document_id = path.into_inner();
    let _request = payload.into_inner();
    state.metrics.record_transfer_initiated();

    HttpResponse::NotImplemented().body("transfer_ownership not yet implemented")
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
        let user = dummy_user();

        // "hello world" in base64
        let file_content = "aGVsbG8gd29ybGQ=";

        let req = UploadDocumentRequest {
            file_name: "hello.txt".to_string(),
            file_content: file_content.to_string(),
            mime_type: "text/plain".to_string(),
        };

        let result = build_upload_response(&user, &req).expect("expected success");

        assert_eq!(result.file_name, "hello.txt");
        assert_eq!(result.mime_type, "text/plain");
        assert_eq!(
            result.document_hash,
            crate::utils::hash_base64(file_content).unwrap()
        );
        assert_eq!(result.status, "pending");
    }

    #[test]
    fn build_upload_response_fails_for_invalid_base64() {
        let user = dummy_user();

        let req = UploadDocumentRequest {
            file_name: "bad.txt".to_string(),
            file_content: "!!!not-valid-base64!!!".to_string(),
            mime_type: "text/plain".to_string(),
        };

        let result = build_upload_response(&user, &req);
        assert!(result.is_err());
    }
}

