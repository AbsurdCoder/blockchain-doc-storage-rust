use actix_web::{web, HttpResponse, Responder};

use crate::middleware::AuthenticatedUser;
use crate::models::{LoginRequest, RegisterRequest, UserResponse};
use crate::services::AuthService;
use crate::AppState;

pub async fn register(
    state: web::Data<AppState>,
    payload: web::Json<RegisterRequest>,
) -> impl Responder {
    match AuthService::register(&state.db, &state.jwt_secret, payload.into_inner()).await {
        Ok(resp) => {
            state.metrics.record_auth_registration();
            HttpResponse::Ok().json(resp)
        }
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

pub async fn login(state: web::Data<AppState>, payload: web::Json<LoginRequest>) -> impl Responder {
    match AuthService::login(&state.db, &state.jwt_secret, payload.into_inner()).await {
        Ok(resp) => {
            state.metrics.record_auth_login();
            HttpResponse::Ok().json(resp)
        }
        Err(e) => HttpResponse::Unauthorized().body(e.to_string()),
    }
}

pub async fn get_current_user(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
) -> impl Responder {
    let user_id: i32 = match user.user_id().parse() {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("invalid user id in token"),
    };

    match AuthService::get_user_by_id(&state.db, user_id).await {
        Ok(u) => HttpResponse::Ok().json(UserResponse::from(u)),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

pub async fn logout(_user: AuthenticatedUser) -> impl Responder {
    // JWT logout is typically handled client-side (delete token).
    HttpResponse::Ok().body("ok")
}

