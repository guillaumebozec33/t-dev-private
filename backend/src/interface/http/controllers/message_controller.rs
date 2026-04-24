use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::application::dto::*;
use crate::application::dto::UpdateMessageRequest;
use crate::domain::errors::DomainError;
use crate::interface::http::middleware::AuthUser;
use crate::shared::app_state::AppState;

#[utoipa::path(
    post,
    path = "/channels/{channel_id}/messages",
    request_body = CreateMessageRequest,
    params(("channel_id" = String, Path, description = "Channel ID")),
    responses(
        (status = 201, body = MessageResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn send_message(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), DomainError> {
    req.validate()
        .map_err(|e| DomainError::ValidationError(e.to_string()))?;

    let message = state.message_service.send_message(auth_user.id, channel_id, req).await?;
    
    if let Some(io) = &state.socket_io {
        let room = format!("channel:{}", channel_id);
        let _ = io.to(room).emit("new_message", &message);
    }
    
    Ok((StatusCode::CREATED, Json(message)))
}

#[utoipa::path(
    get,
    path = "/channels/{channel_id}/messages",
    params(
        ("channel_id" = String, Path, description = "Channel ID"),
        ("limit" = Option<i64>, Query, description = "Max messages to return (default 50, max 100)"),
        ("before" = Option<String>, Query, description = "Cursor: message ID to paginate before"),
    ),
    responses(
        (status = 200, body = Vec<MessageResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_messages(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<GetMessagesQuery>,
) -> Result<Json<Vec<MessageResponse>>, DomainError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let before = query.before.and_then(|s| Uuid::parse_str(&s).ok());
    
    let messages = state.message_service.get_messages(auth_user.id, channel_id, limit, before).await?;
    Ok(Json(messages))
}

#[utoipa::path(
    delete,
    path = "/messages/{message_id}",
    params(("message_id" = String, Path, description = "Message ID")),
    responses(
        (status = 204, description = "Message deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Message not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_message(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, DomainError> {
    let channel_id = state.message_service.delete_message(auth_user.id, message_id).await?;
    
    if let Some(io) = &state.socket_io {
        let room = format!("channel:{}", channel_id);
        let _ = io.to(room).emit("message_deleted", &serde_json::json!({
            "message_id": message_id.to_string(),
            "channel_id": channel_id.to_string()
        }));
    }
    
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/messages/{message_id}",
    request_body = UpdateMessageRequest,
    params(("message_id" = String, Path, description = "Message ID")),
    responses(
        (status = 200, body = MessageResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Message not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn edit_message(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(message_id): Path<Uuid>,
    Json(req): Json<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, DomainError> {
    req.validate()
        .map_err(|e| DomainError::ValidationError(e.to_string()))?;
    let message = state.message_service.edit_message(auth_user.id, message_id, req).await?;
    if let Some(io) = &state.socket_io {
        let room = format!("channel:{}", message.channel_id);
        let _ = io.to(room).emit("message_edited", &message);
    }
    
    Ok(Json(message))
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

    use crate::domain::entities::{Channel, Message, Member, User};
    use crate::domain::enums::{Role, UserStatus};
    use crate::domain::repositories::server_repository::MockServerRepository;
    use crate::domain::repositories::channel_repository::MockChannelRepository;
    use crate::domain::repositories::message_repository::MockMessageRepository;
    use crate::domain::repositories::user_repository::MockUserRepository;
    use crate::infrastructure::security::create_token;
    use crate::interface::http::middleware::auth_middleware;
    use crate::config::Settings;
    use crate::shared::app_state::AppState;

    // ── Helpers ────────────────────────────────────────────────

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

    fn make_channel(server_id: Uuid) -> Channel {
        Channel {
            id: Uuid::new_v4(),
            server_id,
            name: "general".to_string(),
            description: None,
            channel_type: "text".to_string(),
            position: 0,
            is_private: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_message(channel_id: Uuid, author_id: Uuid) -> Message {
        Message {
            id: Uuid::new_v4(),
            channel_id,
            author_id,
            content: "Hello !".to_string(),
            edited: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_user(user_id: Uuid) -> User {
        User {
            id: user_id,
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            password_hash: "hash".to_string(),
            avatar_url: None,
            status: UserStatus::Online,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_token(user_id: Uuid, settings: &Settings) -> String {
        create_token(user_id, &settings.jwt_secret, settings.jwt_expiration).unwrap()
    }

    fn make_router(
        mock_server_repo: MockServerRepository,
        mock_channel_repo: MockChannelRepository,
        mock_message_repo: MockMessageRepository,
        mock_user_repo: MockUserRepository,
    ) -> Router {
        let state = AppState::new_for_test(
            Arc::new(mock_user_repo),
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            make_settings(),
        );

        Router::new()
            .route("/channels/:channel_id/messages", axum::routing::post(send_message))
            .route("/channels/:channel_id/messages", axum::routing::get(get_messages))
            .route("/messages/:message_id", axum::routing::delete(delete_message))
            .route("/messages/:message_id", axum::routing::put(edit_message))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    // ── send_message ───────────────────────────────────────────

    #[tokio::test]
    async fn test_send_message_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_create()
            .returning(move |_| Ok(make_message(channel_id, user_id)));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let app = make_router(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/channels/{}/messages", channel_id))
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
    async fn test_send_message_validation_error() {
        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        // Pas de mock nécessaire — la validation échoue avant d'appeler le service
        let app = make_router(
            MockServerRepository::new(),
            MockChannelRepository::new(),
            MockMessageRepository::new(),
            MockUserRepository::new(),
        );

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/channels/{}/messages", channel_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "content": "" }).to_string())) // contenu vide
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_send_message_channel_not_found() {
        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let app = make_router(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/channels/{}/messages", channel_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "content": "Hello !" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::CREATED);
    }

    // ── get_messages ───────────────────────────────────────────

    #[tokio::test]
    async fn test_get_messages_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_find_by_channel_id()
            .returning(move |_, _, _| Ok(vec![make_message(channel_id, user_id)]));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let app = make_router(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/channels/{}/messages", channel_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_messages_with_limit() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_find_by_channel_id()
            .returning(move |_, _, _| Ok(vec![]));

        let app = make_router(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        // Teste le query param limit
        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/channels/{}/messages?limit=10", channel_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── delete_message ─────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_message_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, user_id))));

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_delete()
            .returning(|_| Ok(()));

        let app = make_router(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/messages/{}", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_message_not_found() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let app = make_router(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/messages/{}", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::NO_CONTENT);
    }

    // ── edit_message ───────────────────────────────────────────

    #[tokio::test]
    async fn test_edit_message_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, user_id))));

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_update()
            .returning(move |_| Ok(make_message(channel_id, user_id)));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let app = make_router(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/messages/{}", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "content": "Message édité" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_edit_message_not_author() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        // Le message appartient à other_user_id → Forbidden
        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, other_user_id))));

        let app = make_router(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/messages/{}", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "content": "Tentative" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::OK);
    }

    // ── socket.io variant tests ────────────────────────────────

    fn make_router_with_io(
        mock_server_repo: MockServerRepository,
        mock_channel_repo: MockChannelRepository,
        mock_message_repo: MockMessageRepository,
        mock_user_repo: MockUserRepository,
    ) -> Router {
        let base_state = AppState::new_for_test(
            Arc::new(mock_user_repo),
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            make_settings(),
        );
        let (_, io) = socketioxide::SocketIo::new_layer();
        io.ns("/", |_: socketioxide::extract::SocketRef| {});
        let state = base_state.with_socket_io(io);

        Router::new()
            .route("/channels/:channel_id/messages", axum::routing::post(send_message))
            .route("/channels/:channel_id/messages", axum::routing::get(get_messages))
            .route("/messages/:message_id", axum::routing::delete(delete_message))
            .route("/messages/:message_id", axum::routing::put(edit_message))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_send_message_with_socket_io() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_create()
            .returning(move |_| Ok(make_message(channel_id, user_id)));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let app = make_router_with_io(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/channels/{}/messages", channel_id))
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
    async fn test_delete_message_with_socket_io() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, user_id))));

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_delete()
            .returning(|_| Ok(()));

        let app = make_router_with_io(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/messages/{}", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_edit_message_with_socket_io() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, user_id))));

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_update()
            .returning(move |_| Ok(make_message(channel_id, user_id)));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let app = make_router_with_io(mock_server_repo, mock_channel_repo, mock_message_repo, mock_user_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/messages/{}", message_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "content": "Message édité" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── sans token ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_sans_token_retourne_401() {
        let channel_id = Uuid::new_v4();
        let app = make_router(
            MockServerRepository::new(),
            MockChannelRepository::new(),
            MockMessageRepository::new(),
            MockUserRepository::new(),
        );

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/channels/{}/messages", channel_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}