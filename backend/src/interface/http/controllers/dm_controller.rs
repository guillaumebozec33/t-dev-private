use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::application::dto::*;
use crate::domain::errors::DomainError;
use crate::interface::http::middleware::AuthUser;
use crate::shared::app_state::AppState;

pub async fn open_conversation(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<OpenConversationRequest>,
) -> Result<(StatusCode, Json<ConversationResponse>), DomainError> {
    let conversation = state.dm_service.open_conversation(auth_user.id, req).await?;
    Ok((StatusCode::OK, Json(conversation)))
}

pub async fn list_conversations(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<ConversationResponse>>, DomainError> {
    let conversations = state.dm_service.list_conversations(auth_user.id).await?;
    Ok(Json(conversations))
}

pub async fn get_dm_messages(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<GetDmMessagesQuery>,
) -> Result<Json<Vec<DmMessageResponse>>, DomainError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let before = query.before.and_then(|s| Uuid::parse_str(&s).ok());

    let messages = state.dm_service.get_messages(auth_user.id, conversation_id, limit, before).await?;
    Ok(Json(messages))
}

pub async fn send_dm(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<Uuid>,
    Json(req): Json<SendDmRequest>,
) -> Result<(StatusCode, Json<DmMessageResponse>), DomainError> {
    req.validate()
        .map_err(|e| DomainError::ValidationError(e.to_string()))?;

    let other_user_id = state.dm_service.get_other_user_id(auth_user.id, conversation_id).await?;
    let message = state.dm_service.send_message(auth_user.id, conversation_id, req).await?;

    if let Some(io) = &state.socket_io {
        let _ = io.to(format!("dm:{}", other_user_id)).emit("dm_message_received", &message);
        let _ = io.to(format!("dm:{}", auth_user.id)).emit("dm_message_received", &message);
    }

    Ok((StatusCode::CREATED, Json(message)))
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

    use crate::domain::entities::{Conversation, DirectMessage, User};
    use crate::domain::enums::UserStatus;
    use crate::domain::repositories::dm_repository::MockDmRepository;
    use crate::domain::repositories::reaction_repository::MockReactionRepository;
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

    fn make_conversation(user1_id: Uuid, user2_id: Uuid) -> Conversation {
        Conversation {
            id: Uuid::new_v4(),
            user1_id,
            user2_id,
            created_at: Utc::now(),
        }
    }

    fn make_dm(conversation_id: Uuid, sender_id: Uuid) -> DirectMessage {
        DirectMessage {
            id: Uuid::new_v4(),
            conversation_id,
            sender_id,
            content: "Hello !".to_string(),
            created_at: Utc::now(),
        }
    }

    fn make_token(user_id: Uuid, settings: &Settings) -> String {
        create_token(user_id, &settings.jwt_secret, settings.jwt_expiration).unwrap()
    }

    fn make_router(
        mock_user_repo: MockUserRepository,
        mock_dm_repo: MockDmRepository,
    ) -> Router {
        let state = AppState::new_for_test_full(
            Arc::new(mock_user_repo),
            Arc::new(MockServerRepository::new()),
            Arc::new(MockChannelRepository::new()),
            Arc::new(MockMessageRepository::new()),
            Arc::new(mock_dm_repo),
            Arc::new(MockReactionRepository::new()),
            make_settings(),
        );

        Router::new()
            .route("/dm/conversations", axum::routing::post(open_conversation))
            .route("/dm/conversations", axum::routing::get(list_conversations))
            .route("/dm/conversations/:id/messages", axum::routing::get(get_dm_messages))
            .route("/dm/conversations/:id/messages", axum::routing::post(send_dm))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    // ── open_conversation ──────────────────────────────────────

    #[tokio::test]
    async fn test_open_conversation_success() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_dm_repo = MockDmRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(other_id))));

        mock_dm_repo
            .expect_find_or_create_conversation()
            .returning(move |u1, u2| Ok(make_conversation(u1, u2)));

        let app = make_router(mock_user_repo, mock_dm_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/dm/conversations")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "user_id": other_id }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── list_conversations ─────────────────────────────────────

    #[tokio::test]
    async fn test_list_conversations_success() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_dm_repo = MockDmRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(other_id))));

        mock_dm_repo
            .expect_find_conversations_by_user()
            .returning(move |_| Ok(vec![make_conversation(user_id, other_id)]));

        let app = make_router(mock_user_repo, mock_dm_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dm/conversations")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── send_dm ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_send_dm_success() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_dm_repo = MockDmRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        // get_other_user_id call + send_message call both use find_conversation_by_id
        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user_id, other_id))));

        mock_dm_repo
            .expect_create_dm_message()
            .returning(move |_| Ok(make_dm(conv_id, user_id)));

        let app = make_router(mock_user_repo, mock_dm_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/dm/conversations/{}/messages", conv_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "content": "Hello !" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_send_dm_validation_error() {
        let user_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let app = make_router(MockUserRepository::new(), MockDmRepository::new());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/dm/conversations/{}/messages", conv_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "content": "" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::CREATED);
    }

    // ── get_dm_messages ────────────────────────────────────────

    #[tokio::test]
    async fn test_get_dm_messages_success() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_dm_repo = MockDmRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user_id, other_id))));

        mock_dm_repo
            .expect_find_messages_by_conversation()
            .returning(move |_, _, _| Ok(vec![make_dm(conv_id, user_id)]));

        let app = make_router(mock_user_repo, mock_dm_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/dm/conversations/{}/messages", conv_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_dm_messages_with_limit() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();
        let before_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_dm_repo = MockDmRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user_id, other_id))));

        mock_dm_repo
            .expect_find_messages_by_conversation()
            .returning(move |_, _, _| Ok(vec![]));

        let app = make_router(mock_user_repo, mock_dm_repo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/dm/conversations/{}/messages?limit=10&before={}", conv_id, before_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── socket.io send_dm ──────────────────────────────────────

    #[tokio::test]
    async fn test_send_dm_with_socket_io() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_dm_repo = MockDmRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user_id, other_id))));

        mock_dm_repo
            .expect_create_dm_message()
            .returning(move |_| Ok(make_dm(conv_id, user_id)));

        let base_state = AppState::new_for_test_full(
            Arc::new(mock_user_repo),
            Arc::new(MockServerRepository::new()),
            Arc::new(MockChannelRepository::new()),
            Arc::new(MockMessageRepository::new()),
            Arc::new(mock_dm_repo),
            Arc::new(MockReactionRepository::new()),
            make_settings(),
        );
        let (_, io) = socketioxide::SocketIo::new_layer();
        io.ns("/", |_: socketioxide::extract::SocketRef| {});
        let state = base_state.with_socket_io(io);

        let app = Router::new()
            .route("/dm/conversations/:id/messages", axum::routing::post(send_dm))
            .layer(axum::middleware::from_fn_with_state(state.clone(), crate::interface::http::middleware::auth_middleware))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/dm/conversations/{}/messages", conv_id))
                    .header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::json!({ "content": "Hello !" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // ── auth ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_sans_token_retourne_401() {
        let app = make_router(MockUserRepository::new(), MockDmRepository::new());

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dm/conversations")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
