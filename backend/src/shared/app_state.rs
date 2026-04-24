use std::sync::Arc;
use socketioxide::SocketIo;
use sqlx::PgPool;

use crate::application::services::{AuthService, ServerService, ChannelService, MessageService, DmService, ReactionService};
use crate::config::Settings;
use crate::domain::repositories::{UserRepository, ServerRepository, ChannelRepository, MessageRepository, DmRepository, ReactionRepository};
use crate::infrastructure::persistence::postgres::{
    PgUserRepository, PgServerRepository, PgChannelRepository, PgMessageRepository, PgDmRepository, PgReactionRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub user_repo: Arc<dyn UserRepository>,
    pub server_repo: Arc<dyn ServerRepository>,
    pub channel_repo: Arc<dyn ChannelRepository>,
    pub message_repo: Arc<dyn MessageRepository>,
    pub dm_repo: Arc<dyn DmRepository>,
    pub reaction_repo: Arc<dyn ReactionRepository>,
    pub auth_service: Arc<AuthService>,
    pub server_service: Arc<ServerService>,
    pub channel_service: Arc<ChannelService>,
    pub message_service: Arc<MessageService>,
    pub dm_service: Arc<DmService>,
    pub reaction_service: Arc<ReactionService>,
    pub socket_io: Option<SocketIo>,
    pub redis_client: redis::Client,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_client: redis::Client, settings: Settings) -> Self {
        let user_repo: Arc<dyn UserRepository> = Arc::new(PgUserRepository::new(db_pool.clone()));
        let server_repo: Arc<dyn ServerRepository> = Arc::new(PgServerRepository::new(db_pool.clone()));
        let channel_repo: Arc<dyn ChannelRepository> = Arc::new(PgChannelRepository::new(db_pool.clone()));
        let message_repo: Arc<dyn MessageRepository> = Arc::new(PgMessageRepository::new(db_pool.clone()));
        let dm_repo: Arc<dyn DmRepository> = Arc::new(PgDmRepository::new(db_pool.clone()));
        let reaction_repo: Arc<dyn ReactionRepository> = Arc::new(PgReactionRepository::new(db_pool.clone()));

        let auth_service = Arc::new(AuthService::new(user_repo.clone(), settings.clone()));
        let server_service = Arc::new(ServerService::new(server_repo.clone(), channel_repo.clone()));
        let channel_service = Arc::new(ChannelService::new(server_repo.clone(), channel_repo.clone()));
        let message_service = Arc::new(MessageService::new(
            server_repo.clone(),
            channel_repo.clone(),
            message_repo.clone(),
            user_repo.clone(),
        ));
        let dm_service = Arc::new(DmService::new(dm_repo.clone(), user_repo.clone()));
        let reaction_service = Arc::new(ReactionService::new(reaction_repo.clone(), user_repo.clone()));

        Self {
            settings,
            user_repo,
            server_repo,
            channel_repo,
            message_repo,
            dm_repo,
            reaction_repo,
            auth_service,
            server_service,
            channel_service,
            message_service,
            dm_service,
            reaction_service,
            socket_io: None,
            redis_client,
        }
    }

    pub fn with_socket_io(mut self, io: SocketIo) -> Self {
        self.socket_io = Some(io);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_settings() -> Settings {
        Settings {
            jwt_secret: "secret_test".to_string(),
            jwt_expiration: 3600,
            database_url: "postgres://fake".to_string(),
            redis_url: "redis://fake".to_string(),
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
        }
    }

    #[tokio::test]
    async fn test_app_state_new() {
        let pool = sqlx::PgPool::connect_lazy(
            "postgres://rtc_user:rtc_password@localhost:5433/rtc_db"
        ).unwrap();
        let redis = redis::Client::open("redis://127.0.0.1/").unwrap();
        let state = AppState::new(pool, redis, make_settings());
        assert!(state.socket_io.is_none());
    }

    #[tokio::test]
    async fn test_app_state_with_socket_io() {
        let pool = sqlx::PgPool::connect_lazy(
            "postgres://rtc_user:rtc_password@localhost:5433/rtc_db"
        ).unwrap();
        let redis = redis::Client::open("redis://127.0.0.1/").unwrap();
        let state = AppState::new(pool, redis, make_settings());

        let (_, io) = socketioxide::SocketIo::new_layer();
        io.ns("/", |_: socketioxide::extract::SocketRef| {});
        let state = state.with_socket_io(io);
        assert!(state.socket_io.is_some());
    }
}

#[cfg(test)]
impl AppState {
    pub fn new_for_test_full(
        user_repo: Arc<dyn UserRepository>,
        server_repo: Arc<dyn ServerRepository>,
        channel_repo: Arc<dyn ChannelRepository>,
        message_repo: Arc<dyn MessageRepository>,
        dm_repo: Arc<dyn DmRepository>,
        reaction_repo: Arc<dyn ReactionRepository>,
        settings: Settings,
    ) -> Self {
        let auth_service = Arc::new(AuthService::new(user_repo.clone(), settings.clone()));
        let server_service = Arc::new(ServerService::new(server_repo.clone(), channel_repo.clone()));
        let channel_service = Arc::new(ChannelService::new(server_repo.clone(), channel_repo.clone()));
        let message_service = Arc::new(MessageService::new(
            server_repo.clone(),
            channel_repo.clone(),
            message_repo.clone(),
            user_repo.clone(),
        ));
        let dm_service = Arc::new(DmService::new(dm_repo.clone(), user_repo.clone()));
        let reaction_service = Arc::new(ReactionService::new(reaction_repo.clone(), user_repo.clone()));

        Self {
            settings,
            user_repo,
            server_repo,
            channel_repo,
            message_repo,
            dm_repo,
            reaction_repo,
            auth_service,
            server_service,
            channel_service,
            message_service,
            dm_service,
            reaction_service,
            socket_io: None,
            redis_client: redis::Client::open("redis://127.0.0.1/").unwrap(),
        }
    }

    pub fn new_for_test(
        user_repo: Arc<dyn UserRepository>,
        server_repo: Arc<dyn ServerRepository>,
        channel_repo: Arc<dyn ChannelRepository>,
        message_repo: Arc<dyn MessageRepository>,
        settings: Settings,
    ) -> Self {
        let pool = sqlx::PgPool::connect_lazy("postgres://rtc_user:rtc_password@localhost:5433/rtc_db").unwrap();
        let dm_repo: Arc<dyn DmRepository> = Arc::new(crate::infrastructure::persistence::postgres::PgDmRepository::new(pool.clone()));
        let reaction_repo: Arc<dyn ReactionRepository> = Arc::new(crate::infrastructure::persistence::postgres::PgReactionRepository::new(pool));
        let auth_service = Arc::new(AuthService::new(user_repo.clone(), settings.clone()));
        let server_service = Arc::new(ServerService::new(server_repo.clone(), channel_repo.clone()));
        let channel_service = Arc::new(ChannelService::new(server_repo.clone(), channel_repo.clone()));
        let message_service = Arc::new(MessageService::new(
            server_repo.clone(),
            channel_repo.clone(),
            message_repo.clone(),
            user_repo.clone(),
        ));
        let dm_service = Arc::new(DmService::new(dm_repo.clone(), user_repo.clone()));
        let reaction_service = Arc::new(ReactionService::new(reaction_repo.clone(), user_repo.clone()));

        Self {
            settings,
            user_repo,
            server_repo,
            channel_repo,
            message_repo,
            dm_repo,
            reaction_repo,
            auth_service,
            server_service,
            channel_service,
            message_service,
            dm_service,
            reaction_service,
            socket_io: None,
            redis_client: redis::Client::open("redis://127.0.0.1/").unwrap(),
        }
    }
}