use anyhow::Result;
use tracing::info;

mod config;
mod application;
mod infrastructure;
mod interface;
mod shared;

use crate::config::Settings;
use crate::shared::app_state::AppState;
use crate::interface::http::routes::create_router;
use crate::interface::websocket::handler::setup_socket_io;
use crate::infrastructure::domain;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let settings = Settings::from_env()?;
    
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&settings.database_url)
        .await?;
    
    let redis_client = redis::Client::open(settings.redis_url.clone())?;
    
    let user_repo: std::sync::Arc<dyn crate::domain::repositories::UserRepository> = 
        std::sync::Arc::new(crate::infrastructure::persistence::postgres::PgUserRepository::new(db_pool.clone()));
    
    let (socket_layer, socket_io) = setup_socket_io(
        db_pool.clone(),
        user_repo.clone(),
        settings.jwt_secret.clone()
    ).await;
    
    let app_state = AppState::new(db_pool, redis_client, settings.clone())
        .with_socket_io(socket_io.clone());
    
    let app = create_router(app_state, socket_io).layer(socket_layer);
    
    let addr = format!("{}:{}", settings.server_host, settings.server_port);
    info!("🚀 Server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
