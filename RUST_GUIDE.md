# Rust Implementation Guide

A detailed guide for understanding and extending the Blockchain Document Storage platform written in Rust.

## Table of Contents

- [Overview](#overview)
- [Module Structure](#module-structure)
- [Key Patterns](#key-patterns)
- [Handlers & Extractors](#handlers--extractors)
- [State & Dependencies](#state--dependencies)
- [Error Handling](#error-handling)
- [Testing](#testing)
- [Extending the Codebase](#extending-the-codebase)
- [Conventions](#conventions)

---

## Overview

This project uses **Actix-web** for HTTP, **Tokio** for async runtime, **SQLx** for MySQL, and **serde** for serialization. The architecture follows a layered design: handlers → services → external systems (DB, S3, blockchain).

---

## Module Structure

```
src/
├── main.rs          # App entry, routes, AppState
├── handlers/        # HTTP handlers (thin layer)
├── middleware/      # Auth extractor, future middleware
├── models/          # Data structs (DB, request/response DTOs)
├── services/        # Business logic (blockchain, storage, auth)
└── utils/           # Pure helpers (hash, errors)
```

| Module | Responsibility |
|--------|----------------|
| `handlers` | Parse request, call services, return HTTP response |
| `middleware` | JWT validation, custom extractors |
| `models` | `User`, `Document`, `Claims`, DTOs for API |
| `services` | Database, S3, blockchain operations |
| `utils` | Hashing, error types, reusable helpers |

---

## Key Patterns

### Async Handlers

All handlers are `async fn` returning `impl Responder`:

```rust
pub async fn upload_document(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    payload: web::Json<UploadDocumentRequest>,
) -> impl Responder {
    match build_upload_response(&user, &payload.into_inner()) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}
```

Actix-web resolves extractors (`AuthenticatedUser`, `web::Data`, `web::Json`) before calling the handler. If any extractor fails (e.g. invalid JWT), Actix returns an error response and never invokes the handler.

### Extractor-Based Auth

`AuthenticatedUser` is an Actix `FromRequest` extractor that:

1. Reads `Authorization: Bearer <token>`
2. Validates the JWT with `AppState.jwt_secret`
3. Returns `Claims` on success, or `401 Unauthorized` on failure

Protected handlers add it as a parameter:

```rust
pub async fn list_documents(
    user: AuthenticatedUser,  // Must be present; 401 if missing/invalid
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let user_id = user.user_id();
    // ...
}
```

### Request/Response DTOs

Models use serde for JSON:

```rust
#[derive(Debug, Deserialize)]
pub struct UploadDocumentRequest {
    pub file_name: String,
    pub file_content: String,  // base64
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: i32,
    pub file_name: String,
    pub document_hash: String,
    // ...
}
```

---

## Handlers & Extractors

### Common Extractors

| Extractor | Purpose |
|-----------|---------|
| `web::Data<AppState>` | Shared app state (DB pool, services, JWT secret) |
| `web::Json<T>` | Parse JSON body into `T` |
| `web::Path<T>` | Path parameters (e.g. `/{id}`) |
| `web::Query<T>` | Query string parameters |
| `AuthenticatedUser` | Validated JWT claims (custom extractor) |

### Helper Functions

Keep handlers thin. Move logic into pure helpers for easier testing:

```rust
fn build_upload_response(
    user: &AuthenticatedUser,
    request: &UploadDocumentRequest,
) -> Result<DocumentResponse, String> {
    let document_hash = hash_base64(&request.file_content)
        .map_err(|_| "invalid base64 file_content".to_string())?;
    Ok(DocumentResponse { /* ... */ })
}
```

---

## State & Dependencies

`AppState` holds shared resources:

```rust
pub struct AppState {
    pub db: sqlx::MySqlPool,
    pub blockchain: BlockchainService,
    pub storage: StorageService,
    pub jwt_secret: String,
}
```

Inject it with `web::Data::new(app_state)` and access in handlers via `web::Data<AppState>`. Actix clones the `Arc`; the pool and services are shared across requests.

---

## Error Handling

- **Extractor errors** (e.g. invalid JWT): Actix returns `401` or `400` before the handler runs.
- **Handler logic**: Return `Result` from helpers and map to HTTP responses:

```rust
match build_upload_response(&user, &req) {
    Ok(response) => HttpResponse::Ok().json(response),
    Err(msg) => HttpResponse::BadRequest().body(msg),
}
```

For richer error types, consider `thiserror` + a custom `Responder` impl to map domain errors to HTTP status codes.

---

## Testing

### Unit Tests

Place tests in the same module with `#[cfg(test)]`:

```rust
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
        let req = UploadDocumentRequest {
            file_name: "hello.txt".to_string(),
            file_content: "aGVsbG8gd29ybGQ=".to_string(),
            mime_type: "text/plain".to_string(),
        };
        let result = build_upload_response(&user, &req).expect("expected success");
        assert_eq!(result.file_name, "hello.txt");
        assert_eq!(result.status, "pending");
    }
}
```

### Integration Tests

Use `actix_web::test` for HTTP-level tests:

```rust
use actix_web::{test, web, App};

#[actix_web::test]
async fn test_health() {
    let app = test::init_service(
        App::new().route("/health", web::get().to(health::health_check))
    ).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

---

## Extending the Codebase

### Adding a New Endpoint

1. Define request/response DTOs in `models/` if needed.
2. Add handler in `handlers/`.
3. Register route in `main.rs`:

```rust
.service(
    web::scope("/api/documents")
        .route("/new-action", web::post().to(documents::new_action)),
)
```

### Adding a New Extractor

Implement `FromRequest`:

```rust
impl FromRequest for MyExtractor {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Parse from req, return Ok(Self(...)) or Err(...)
        ready(Ok(MyExtractor(/* ... */)))
    }
}
```

### Adding a Utility

Add to `utils/` and re-export in `utils/mod.rs`:

```rust
// utils/hash.rs
pub fn hash_bytes(data: &[u8]) -> String { /* ... */ }
```

---

## Conventions

- **Imports**: Group by `crate`, `std`, then external crates; sort alphabetically within groups.
- **Async**: Use `async fn` for handlers; prefer `Result`-returning sync helpers for pure logic.
- **Cloning**: Clone only when necessary (e.g. moving into async blocks); prefer references in sync code.
- **Logging**: Use `log::info!`, `log::error!` etc.; configure with `RUST_LOG`.
- **Formatting**: Run `cargo fmt` before committing; use `cargo clippy` for linting.

---

## Resources

- [Actix-web Extractors](https://actix.rs/docs/extractors/)
- [Tokio Async Runtime](https://tokio.rs/)
- [SQLx Query Documentation](https://github.com/launchbadge/sqlx)
- [Serde Serialization](https://serde.rs/)
