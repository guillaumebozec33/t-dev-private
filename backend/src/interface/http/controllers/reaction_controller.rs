use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::application::dto::*;
use crate::domain::errors::DomainError;
use crate::interface::http::middleware::AuthUser;
use crate::shared::app_state::AppState;

/// Toggle a reaction on a channel message
pub async fn toggle_reaction(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(message_id): Path<Uuid>,
    Json(req): Json<ToggleReactionRequest>,
) -> Result<Json<Vec<ReactionResponse>>, DomainError> {
    req.validate()
        .map_err(|e| DomainError::ValidationError(e.to_string()))?;

    let reactions = state.reaction_service.toggle_reaction(auth_user.id, message_id, req).await?;

    // Emit socket event to the channel room
    if let Some(io) = &state.socket_io {
        // Try to find the channel_id from the messages table
        if let Ok(Some(message)) = state.message_repo.find_by_id(message_id).await {
            let room = format!("channel:{}", message.channel_id);
            let _ = io.to(room).emit("reaction_updated", &serde_json::json!({
                "message_id": message_id.to_string(),
                "reactions": &reactions,
            }));
        }
    }

    Ok(Json(reactions))
}

/// Toggle a reaction on a DM message
pub async fn toggle_dm_reaction(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(message_id): Path<Uuid>,
    Json(req): Json<ToggleReactionRequest>,
) -> Result<Json<Vec<ReactionResponse>>, DomainError> {
    req.validate()
        .map_err(|e| DomainError::ValidationError(e.to_string()))?;

    let reactions = state.reaction_service.toggle_reaction(auth_user.id, message_id, req).await?;

    // Emit socket event to both DM rooms
    if let Some(io) = &state.socket_io {
        if let Ok(Some(dm)) = state.dm_repo.find_message_by_id(message_id).await {
            if let Ok(Some(conv)) = state.dm_repo.find_conversation_by_id(dm.conversation_id).await {
                let payload = serde_json::json!({
                    "message_id": message_id.to_string(),
                    "reactions": &reactions,
                });
                let _ = io.to(format!("dm:{}", conv.user1_id)).emit("reaction_updated", &payload);
                let _ = io.to(format!("dm:{}", conv.user2_id)).emit("reaction_updated", &payload);
            }
        }
    }

    Ok(Json(reactions))
}

/// Get reactions for a message
pub async fn get_reactions(
    State(state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<ReactionResponse>>, DomainError> {
    let reactions = state.reaction_service.get_reactions(message_id).await?;
    Ok(Json(reactions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
        middleware,
        Router,
    };
    use tower::util::ServiceExt;
    use serde_json::json;
    use uuid::Uuid;
    use chrono::Utc;

    use crate::domain::entities::{Reaction, User};
    use crate::domain::enums::UserStatus;
    use crate::domain::repositories::reaction_repository::MockReactionRepository;
    use crate::domain::repositories::dm_repository::MockDmRepository;
    use crate::domain::repositories::server_repository::MockServerRepository;
    use crate::domain::repositories::channel_repository::MockChannelRepository;
    use crate::domain::repositories::message_repository::MockMessageRepository;
    use crate::domain::repositories::user_repository::MockUserRepository;
    use crate::infrastructure::security::create_token;
    use crate::interface::http::middleware::auth_middleware;
    use crate::config::Settings;
    use crate::shared::app_state::AppState;

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

    fn make_user(id: Uuid) -> User {
        User {
            id,
            username: "alice".to_string(),
            email: "alice@test.com".to_string(),
            password_hash: "hash".to_string(),
            avatar_url: None,
            status: UserStatus::Online,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_reaction(message_id: Uuid, user_id: Uuid, emoji: &str) -> Reaction {
        Reaction {
            id: Uuid::new_v4(),
            message_id,
            user_id,
            emoji: emoji.to_string(),
            created_at: Utc::now(),
        }
    }

    fn make_token(user_id: Uuid, settings: &Settings) -> String {
        create_token(user_id, &settings.jwt_secret, settings.jwt_expiration).unwrap()
    }

    fn make_router(
        mock_user_repo: MockUserRepository,
        mock_reaction_repo: MockReactionRepository,
    ) -> Router {
        let state = AppState::new_for_test_full(
            Arc::new(mock_user_repo),
            Arc::new(MockServerRepository::new()),
            Arc::new(MockChannelRepository::new()),
            Arc::new(MockMessageRepository::new()),
            Arc::new(MockDmRepository::new()),
            Arc::new(mock_reaction_repo),
            make_settings(),
        );

        Router::new()
            .route("/messages/:message_id/reactions", axum::routing::put(toggle_reaction))
            .route("/messages/:message_id/reactions", axum::routing::get(get_reactions))
            .route("/dm/messages/:message_id/reactions", axum::routing::put(toggle_dm_reaction))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    fn make_router_with_io(
        mock_user_repo: MockUserRepository,
        mock_reaction_repo: MockReactionRepository,
        mock_message_repo: MockMessageRepository,
        mock_dm_repo: MockDmRepository,
    ) -> Router {
        let state = AppState::new_for_test_full(
            Arc::new(mock_user_repo),
            Arc::new(MockServerRepository::new()),
            Arc::new(MockChannelRepository::new()),
            Arc::new(mock_message_repo),
            Arc::new(mock_dm_repo),
            Arc::new(mock_reaction_repo),
            make_settings(),
        );
        let (_, io) = socketioxide::SocketIo::new_layer();
        io.ns("/", |_: socketioxide::extract::SocketRef| {});
        let state = state.with_socket_io(io);

        Router::new()
            .route("/messages/:message_id/reactions", axum::routing::put(toggle_reaction))
            .route("/messages/:message_id/reactions", axum::routing::get(get_reactions))
            .route("/dm/messages/:message_id/reactions", axum::routing::put(toggle_dm_reaction))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    // ── toggle_reaction ────────────────────────────────────────

    #[tokio::test]
    async fn test_toggle_reaction_success() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_reaction_repo = MockReactionRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        mock_reaction_repo
            .expect_find_by_user_and_message()
            .returning(|_, _| Ok(None));

        mock_reaction_repo
            .expect_create()
            .returning(move |r| Ok(make_reaction(message_id, user_id, &r.emoji)));

        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "👍")]));

        let app = make_router(mock_user_repo, mock_reaction_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/messages/{}/reactions", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "emoji": "👍" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_toggle_reaction_validation_error() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let app = make_router(MockUserRepository::new(), MockReactionRepository::new());

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/messages/{}/reactions", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "emoji": "" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::OK);
    }

    // ── get_reactions ──────────────────────────────────────────

    #[tokio::test]
    async fn test_get_reactions_success() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_reaction_repo = MockReactionRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "🔥")]));

        let app = make_router(mock_user_repo, mock_reaction_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/messages/{}/reactions", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── toggle_dm_reaction ─────────────────────────────────────

    #[tokio::test]
    async fn test_toggle_dm_reaction_success() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_reaction_repo = MockReactionRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        mock_reaction_repo
            .expect_find_by_user_and_message()
            .returning(|_, _| Ok(None));

        mock_reaction_repo
            .expect_create()
            .returning(move |r| Ok(make_reaction(message_id, user_id, &r.emoji)));

        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "👍")]));

        let app = make_router(mock_user_repo, mock_reaction_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/dm/messages/{}/reactions", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "emoji": "👍" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── socket.io variant ──────────────────────────────────────

    #[tokio::test]
    async fn test_toggle_reaction_with_socket_io() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_reaction_repo = MockReactionRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        mock_reaction_repo
            .expect_find_by_user_and_message()
            .returning(|_, _| Ok(None));

        mock_reaction_repo
            .expect_create()
            .returning(move |r| Ok(make_reaction(message_id, user_id, &r.emoji)));

        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "👍")]));

        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| {
                use crate::domain::entities::Message;
                Ok(Some(Message {
                    id: message_id,
                    channel_id,
                    author_id: user_id,
                    content: "hi".to_string(),
                    edited: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }))
            });

        let app = make_router_with_io(mock_user_repo, mock_reaction_repo, mock_message_repo, MockDmRepository::new());

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/messages/{}/reactions", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "emoji": "👍" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── toggle_dm_reaction with socket.io ──────────────────────

    #[tokio::test]
    async fn test_toggle_dm_reaction_with_socket_io() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();
        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_reaction_repo = MockReactionRepository::new();
        let mut mock_dm_repo = MockDmRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        mock_reaction_repo
            .expect_find_by_user_and_message()
            .returning(|_, _| Ok(None));
        mock_reaction_repo
            .expect_create()
            .returning(move |r| Ok(make_reaction(message_id, user_id, &r.emoji)));
        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "👍")]));

        mock_dm_repo
            .expect_find_message_by_id()
            .returning(move |_| {
                use crate::domain::entities::DirectMessage;
                Ok(Some(DirectMessage {
                    id: message_id,
                    conversation_id: conv_id,
                    sender_id: user_id,
                    content: "hi".to_string(),
                    created_at: Utc::now(),
                }))
            });

        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| {
                use crate::domain::entities::Conversation;
                Ok(Some(Conversation {
                    id: conv_id,
                    user1_id,
                    user2_id,
                    created_at: Utc::now(),
                }))
            });

        let app = make_router_with_io(mock_user_repo, mock_reaction_repo, MockMessageRepository::new(), mock_dm_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/dm/messages/{}/reactions", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "emoji": "👍" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── auth ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_sans_token_retourne_401() {
        let message_id = Uuid::new_v4();
        let app = make_router(MockUserRepository::new(), MockReactionRepository::new());

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/messages/{}/reactions", message_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
