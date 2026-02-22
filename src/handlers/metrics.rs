use actix_web::{web, HttpResponse, Responder};

use crate::AppState;

/// GET /api/metrics - returns processing metrics as JSON.
pub async fn get_metrics(state: web::Data<AppState>) -> impl Responder {
    let snapshot = state.metrics.snapshot();
    HttpResponse::Ok().json(snapshot)
}
