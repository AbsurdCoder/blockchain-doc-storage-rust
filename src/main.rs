use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use handlers::{auth, documents, health, metrics};
use services::{blockchain::BlockchainService, storage::StorageService};
use std::sync::Arc;
use utils::Metrics;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::MySqlPool,
    pub blockchain: BlockchainService,
    pub storage: StorageService,
    pub jwt_secret: String,
    pub metrics: Arc<Metrics>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting Blockchain Document Storage Server");

    // Get configuration from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    // Create database connection pool
    log::info!("Connecting to database...");
    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    log::info!("Database connected successfully");

    // Initialize services
    let blockchain_service = BlockchainService::new();
    let storage_service = StorageService::new().await;

    // Create application state
    let app_state = AppState {
        db: db_pool,
        blockchain: blockchain_service,
        storage: storage_service,
        jwt_secret,
        metrics: Arc::new(Metrics::default()),
    };

    let server_address = format!("{}:{}", server_host, server_port);
    log::info!("Starting server at http://{}", server_address);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::permissive(); // Configure CORS as needed

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            // Health check
            .route("/health", web::get().to(health::health_check))
            // Metrics
            .route("/api/metrics", web::get().to(metrics::get_metrics))
            // Auth routes
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    .route("/me", web::get().to(auth::get_current_user))
                    .route("/logout", web::post().to(auth::logout)),
            )
            // Document routes
            .service(
                web::scope("/api/documents")
                    .route("", web::post().to(documents::upload_document))
                    .route("", web::get().to(documents::list_documents))
                    .route("/{id}", web::get().to(documents::get_document))
                    .route("/verify", web::post().to(documents::verify_document))
                    .route("/{id}/transfer", web::post().to(documents::transfer_ownership)),
            )
            // Blockchain routes
            .service(
                web::scope("/api/blockchain")
                    .route("/status", web::get().to(documents::blockchain_status)),
            )
    })
    .bind(&server_address)?
    .run()
    .await
}
