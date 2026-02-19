use actix_web::{web, HttpResponse, Responder};

use crate::middleware::AuthenticatedUser;
use crate::models::{
    Document, DocumentResponse, TransferOwnershipRequest, UploadDocumentRequest,
    VerificationResponse, VerifyDocumentRequest,
};
use crate::AppState;

pub async fn upload_document(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    payload: web::Json<UploadDocumentRequest>,
) -> impl Responder {
    // TODO: implement S3 upload, hashing, DB insert, and blockchain record creation
    let _user_id = user.user_id();
    let _jwt_claims = user.claims();
    let _app_state = state.into_inner();
    let _request = payload.into_inner();

    HttpResponse::NotImplemented().body("upload_document not yet implemented")
}

pub async fn list_documents(
    user: AuthenticatedUser,
    _state: web::Data<AppState>,
    _query: web::Query<serde_json::Value>,
) -> impl Responder {
    let _user_id = user.user_id();

    let empty: Vec<DocumentResponse> = Vec::new();
    HttpResponse::Ok().json(empty)
}

pub async fn get_document(
    user: AuthenticatedUser,
    _state: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let _user_id = user.user_id();
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
    _state: web::Data<AppState>,
    payload: web::Json<VerifyDocumentRequest>,
) -> impl Responder {
    let request = payload.into_inner();

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
    _state: web::Data<AppState>,
    path: web::Path<i32>,
    payload: web::Json<TransferOwnershipRequest>,
) -> impl Responder {
    let _user_id = user.user_id();
    let _document_id = path.into_inner();
    let _request = payload.into_inner();

    HttpResponse::NotImplemented().body("transfer_ownership not yet implemented")
}

