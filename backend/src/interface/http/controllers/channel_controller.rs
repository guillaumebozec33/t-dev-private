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
use crate::interface::websocket::handler::{emit_channel_created, emit_channel_updated, emit_channel_deleted};
use crate::shared::app_state::AppState;

#[utoipa::path(
    post,
    path = "/servers/{server_id}/channels",
    request_body = CreateChannelRequest,
    params(("server_id" = String, Path, description = "Server ID")),
    responses(
        (status = 201, body = ChannelResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<ChannelResponse>), DomainError> {
    req.validate()
        .map_err(|e| DomainError::ValidationError(e.to_string()))?;

    let channel = state.channel_service.create_channel(auth_user.id, server_id, req).await?;

    if let Some(io) = &state.socket_io {
        emit_channel_created(io, &server_id.to_string(), &serde_json::to_value(&channel).unwrap());
    }

    Ok((StatusCode::CREATED, Json(channel)))
}

#[utoipa::path(
    get,
    path = "/servers/{server_id}/channels",
    params(("server_id" = String, Path, description = "Server ID")),
    responses(
        (status = 200, body = Vec<ChannelResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_channels(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<ChannelResponse>>, DomainError> {
    let channels = state.channel_service.get_channels(auth_user.id, server_id).await?;
    Ok(Json(channels))
}

#[utoipa::path(
    get,
    path = "/channels/{channel_id}",
    params(("channel_id" = String, Path, description = "Channel ID")),
    responses(
        (status = 200, body = ChannelResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<ChannelResponse>, DomainError> {
    let channel = state.channel_service.get_channel(auth_user.id, channel_id).await?;
    Ok(Json(channel))
}

#[utoipa::path(
    put,
    path = "/channels/{channel_id}",
    request_body = UpdateChannelRequest,
    params(("channel_id" = String, Path, description = "Channel ID")),
    responses(
        (status = 200, body = ChannelResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<UpdateChannelRequest>,
) -> Result<Json<ChannelResponse>, DomainError> {
    let channel = state.channel_service.update_channel(auth_user.id, channel_id, req).await?;

    if let Some(io) = &state.socket_io {
        let server_id = channel.server_id.to_string();
        emit_channel_updated(io, &server_id, &serde_json::to_value(&channel).unwrap());
    }

    Ok(Json(channel))
}

#[utoipa::path(
    delete,
    path = "/channels/{channel_id}",
    params(("channel_id" = String, Path, description = "Channel ID")),
    responses(
        (status = 204, description = "Channel deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(channel_id): Path<Uuid>,
) -> Result<StatusCode, DomainError> {
    let channel = state.channel_service.get_channel(auth_user.id, channel_id).await?;
    let server_id = channel.server_id.to_string();

    state.channel_service.delete_channel(auth_user.id, channel_id).await?;

    if let Some(io) = &state.socket_io {
        emit_channel_deleted(io, &server_id, &channel_id.to_string());
    }

    Ok(StatusCode::NO_CONTENT)
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

    use crate::domain::entities::{Channel, Member};
    use crate::domain::enums::Role;
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
            icon: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_member(user_id: Uuid, server_id: Uuid) -> Member {
        Member::new(user_id, server_id, Role::Admin)
    }

    // Génère un JWT valide pour les tests
    // Le middleware auth vérifie ce token et injecte AuthUser dans l'Extension
    fn make_token(user_id: Uuid, settings: &Settings) -> String {
        create_token(user_id, &settings.jwt_secret, settings.jwt_expiration).unwrap()
    }

    // Crée le routeur avec le middleware d'auth branché
    // Sans le middleware, Extension(auth_user) ne serait pas injectée
    // et toutes les routes retourneraient 500
    fn make_router(
        mock_server_repo: MockServerRepository,
        mock_channel_repo: MockChannelRepository,
    ) -> Router {
        let state = AppState::new_for_test(
            Arc::new(MockUserRepository::new()),
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(MockMessageRepository::new()),
            make_settings(),
        );

        Router::new()
            .route("/servers/:server_id/channels", axum::routing::post(create_channel))
            .route("/servers/:server_id/channels", axum::routing::get(get_channels))
            .route("/channels/:channel_id", axum::routing::get(get_channel))
            .route("/channels/:channel_id", axum::routing::put(update_channel))
            .route("/channels/:channel_id", axum::routing::delete(delete_channel))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    // ── create_channel ─────────────────────────────────────────

    #[tokio::test]
    async fn test_create_channel_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id))));

        mock_channel_repo
            .expect_create()
            .returning(move |_| Ok(make_channel(server_id)));

        let app = make_router(mock_server_repo, mock_channel_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/channels", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({
                        "name": "general",
                        "description": null,
                        "channel_type": null
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_channel_not_admin() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        // Membre simple → can_manage_channels() = false → Forbidden
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(Member::new(user_id, server_id, Role::Member))));

        let app = make_router(mock_server_repo, mock_channel_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/channels", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({
                        "name": "general",
                        "description": null,
                        "channel_type": null
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::CREATED);
    }

    // ── get_channels ───────────────────────────────────────────

    #[tokio::test]
    async fn test_get_channels_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id))));

        mock_channel_repo
            .expect_find_by_server_id()
            .returning(move |_| Ok(vec![make_channel(server_id)]));

        let app = make_router(mock_server_repo, mock_channel_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/servers/{}/channels", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── get_channel ────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_channel_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id))));

        let app = make_router(mock_server_repo, mock_channel_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/channels/{}", channel_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_channel_not_found() {
        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let app = make_router(mock_server_repo, mock_channel_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/channels/{}", channel_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::OK);
    }

    // ── delete_channel ─────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_channel_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        // get_channel + delete_channel appellent tous les deux find_by_id
        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id))));

        mock_channel_repo
            .expect_delete()
            .returning(|_| Ok(()));

        let app = make_router(mock_server_repo, mock_channel_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/channels/{}", channel_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    // ── sans token ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_sans_token_retourne_401() {
        let server_id = Uuid::new_v4();
        let app = make_router(MockServerRepository::new(), MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/servers/{}/channels", server_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}