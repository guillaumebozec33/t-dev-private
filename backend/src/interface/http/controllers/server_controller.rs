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
use crate::interface::websocket::handler::{emit_member_joined, emit_member_left, emit_member_role_changed,emit_member_kicked, emit_member_banned, emit_server_updated};
use crate::shared::app_state::AppState;

#[utoipa::path(
    post,
    path = "/servers",
    request_body = CreateServerRequest,
    responses(
        (status = 201, body = ServerResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_server(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateServerRequest>,
) -> Result<(StatusCode, Json<ServerResponse>), DomainError> {
    req.validate()
        .map_err(|e| DomainError::ValidationError(e.to_string()))?;
    let response = state.server_service.create_server(auth_user.id, req).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/servers",
    responses(
        (status = 200, body = Vec<ServerResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_servers(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<ServerResponse>>, DomainError> {
    let servers = state.server_service.get_user_servers(auth_user.id).await?;
    Ok(Json(servers))
}

#[utoipa::path(
    get,
    path = "/servers/{server_id}",
    params(("server_id" = String, Path, description = "Server ID")),
    responses(
        (status = 200, body = ServerResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_server(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<ServerResponse>, DomainError> {
    let server = state.server_service.get_server(auth_user.id, server_id).await?;
    Ok(Json(server))
}

#[utoipa::path(
    put,
    path = "/servers/{server_id}",
    request_body = UpdateServerRequest,
    params(("server_id" = String, Path, description = "Server ID")),
    responses(
        (status = 200, body = ServerResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Server not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_server(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
    Json(req): Json<UpdateServerRequest>,
) -> Result<Json<ServerResponse>, DomainError> {
    let server = state.server_service.update_server(auth_user.id, server_id, req).await?;
    if let Some(io) = &state.socket_io {
        emit_server_updated(io, &serde_json::to_value(&server).unwrap());
    }
    Ok(Json(server))
}

#[utoipa::path(
    delete,
    path = "/servers/{server_id}",
    params(("server_id" = String, Path, description = "Server ID")),
    responses(
        (status = 204, description = "Server deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Server not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_server(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, DomainError> {
    state.server_service.delete_server(auth_user.id, server_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/servers/{server_id}/join",
    request_body = JoinServerRequest,
    params(("server_id" = String, Path, description = "Server ID")),
    responses(
        (status = 200, body = ServerResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn join_server(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(_server_id): Path<Uuid>,
    Json(req): Json<JoinServerRequest>,
) -> Result<Json<ServerResponse>, DomainError> {
    let server = state.server_service.join_server(auth_user.id, &req.invite_code).await?;
    
    if let Some(io) = &state.socket_io {
        let server_uuid = Uuid::parse_str(&server.id)
            .map_err(|_| DomainError::InternalError("Invalid server ID".to_string()))?;
        
        let members = state.server_service.get_members(auth_user.id, server_uuid).await?;
        let member = members
            .into_iter()
            .find(|m| m.user_id == auth_user.id.to_string())
            .ok_or(DomainError::InternalError("Member not found".to_string()))?;
        
        emit_member_joined(io, &server.id, &serde_json::to_value(&member).unwrap());
    }
    
    Ok(Json(server))
}

#[utoipa::path(
    post,
    path = "/servers/join",
    request_body = JoinServerRequest,
    responses(
        (status = 200, body = ServerResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn join_server_by_code(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<JoinServerRequest>,
) -> Result<Json<ServerResponse>, DomainError> {
    let server = state.server_service.join_server(auth_user.id, &req.invite_code).await?;
    
    if let Some(io) = &state.socket_io {
        let server_uuid = Uuid::parse_str(&server.id)
            .map_err(|_| DomainError::InternalError("Invalid server ID".to_string()))?;
        
        let members = state.server_service.get_members(auth_user.id, server_uuid).await?;
        let member = members
            .into_iter()
            .find(|m| m.user_id == auth_user.id.to_string())
            .ok_or(DomainError::InternalError("Member not found".to_string()))?;
        
        emit_member_joined(io, &server.id, &serde_json::to_value(&member).unwrap());
    }
    
    Ok(Json(server))
}

#[utoipa::path(
    delete,
    path = "/servers/{server_id}/leave",
    params(("server_id" = Uuid, Path, description = "Server ID")),
    responses(
        (status = 204, description = "Left server"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn leave_server(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
) -> Result<StatusCode, DomainError> {
    state.server_service.leave_server(auth_user.id, server_id).await?;
    
    if let Some(io) = &state.socket_io {
        emit_member_left(io, &server_id.to_string(), &auth_user.id.to_string());
    }
    
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/servers/{server_id}/members/{member_id}/kick",
    params(
        ("server_id" = Uuid, Path, description = "Server ID"),
        ("member_id" = Uuid, Path, description = "Member ID to kick"),
    ),
    responses(
        (status = 204, description = "Member kicked"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn kick_member(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((server_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, DomainError> {
    // Vérifier que l'utilisateur est owner et kick le membre
    state.server_service.kick_member(auth_user.id, server_id, member_id).await?;
    
    if let Some(io) = &state.socket_io {
        // Notifier que le membre a été kicked
        emit_member_kicked(io, &server_id.to_string(), &member_id.to_string());
    }
    
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/servers/{server_id}/members",
    params(("server_id" = Uuid, Path, description = "Server ID")),
    responses(
        (status = 200, body = Vec<MemberResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_members(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<MemberResponse>>, DomainError> {
    let members = state.server_service.get_members(auth_user.id, server_id).await?;
    Ok(Json(members))
}

#[utoipa::path(
    put,
    path = "/servers/{server_id}/members/{user_id}",
    request_body = UpdateMemberRoleRequest,
    params(
        ("server_id" = Uuid, Path, description = "Server ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, body = MemberResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_member_role(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<MemberResponse>, DomainError> {
    let member = state.server_service.update_member_role(auth_user.id, server_id, user_id, req.role.clone()).await?;
    
    if let Some(io) = &state.socket_io {
        emit_member_role_changed(io, &server_id.to_string(), &user_id.to_string(), &req.role.to_string());
    }
    
    Ok(Json(member))
}

#[utoipa::path(
    post,
    path = "/servers/{server_id}/invitations",
    request_body = CreateInvitationRequest,
    params(("server_id" = Uuid, Path, description = "Server ID")),
    responses(
        (status = 201, body = InvitationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_invitation(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
    Json(req): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<InvitationResponse>), DomainError> {
    let invitation = state.server_service.create_invitation(auth_user.id, server_id, req).await?;
    Ok((StatusCode::CREATED, Json(invitation)))
}

#[utoipa::path(
    put,
    path = "/servers/{server_id}/owner",
    request_body = TransferOwnershipRequest,
    params(("server_id" = Uuid, Path, description = "Server ID")),
    responses(
        (status = 200, description = "Ownership transferred"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Server not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn transfer_ownership(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<StatusCode, DomainError> {
    let new_owner_id = Uuid::parse_str(&req.new_owner_id)
        .map_err(|_| DomainError::ValidationError("Invalid user ID".to_string()))?;
    
    state.server_service.transfer_ownership(auth_user.id, server_id, new_owner_id).await?;
    
    if let Some(io) = &state.socket_io {
        emit_member_role_changed(io, &server_id.to_string(), &new_owner_id.to_string(), "owner");
        emit_member_role_changed(io, &server_id.to_string(), &auth_user.id.to_string(), "member");
    }
    
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/servers/{server_id}/members/{member_id}/ban",
    request_body = BanMemberRequest,
    params(
        ("server_id" = Uuid, Path, description = "Server ID"),
        ("member_id" = Uuid, Path, description = "Member ID to ban"),
    ),
    responses(
        (status = 201, body = BanResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn ban_member(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((server_id, member_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<BanMemberRequest>,
) -> Result<(StatusCode, Json<BanResponse>), DomainError> {
    let ban = state.server_service.ban_member(auth_user.id, server_id, member_id, req).await?;

    if let Some(io) = &state.socket_io {
        emit_member_banned(io, &server_id.to_string(), &member_id.to_string());
    }

    Ok((StatusCode::CREATED, Json(ban)))
}

#[utoipa::path(
    delete,
    path = "/servers/{server_id}/members/{user_id}/ban",
    params(
        ("server_id" = Uuid, Path, description = "Server ID"),
        ("user_id" = Uuid, Path, description = "User ID to unban"),
    ),
    responses(
        (status = 204, description = "Member unbanned"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Ban not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn unban_member(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, DomainError> {
    state.server_service.unban_member(auth_user.id, server_id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/servers/{server_id}/bans",
    params(("server_id" = Uuid, Path, description = "Server ID")),
    responses(
        (status = 200, body = Vec<BanResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_bans(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<BanResponse>>, DomainError> {
    let bans = state.server_service.get_bans(auth_user.id, server_id).await?;
    Ok(Json(bans))
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

    use crate::domain::entities::{Server, Member, Invitation, Channel};
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

    fn make_server(owner_id: Uuid) -> Server {
        Server {
            id: Uuid::new_v4(),
            name: "Mon serveur".to_string(),
            description: None,
            icon_url: None,
            owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_member(user_id: Uuid, server_id: Uuid, role: Role) -> Member {
        Member::new(user_id, server_id, role)
    }

    fn make_invitation(server_id: Uuid, inviter_id: Uuid) -> Invitation {
        Invitation::new(server_id, inviter_id, Some(10), Some(Utc::now() + chrono::Duration::days(1)))
    }

    fn make_token(user_id: Uuid, settings: &Settings) -> String {
        create_token(user_id, &settings.jwt_secret, settings.jwt_expiration).unwrap()
    }

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
            .route("/servers", axum::routing::post(create_server))
            .route("/servers", axum::routing::get(get_servers))
            .route("/servers/:server_id", axum::routing::get(get_server))
            .route("/servers/:server_id", axum::routing::put(update_server))
            .route("/servers/:server_id", axum::routing::delete(delete_server))
            .route("/servers/:server_id/join", axum::routing::post(join_server))
            .route("/servers/join", axum::routing::post(join_server_by_code))
            .route("/servers/:server_id/leave", axum::routing::post(leave_server))
            .route("/servers/:server_id/members", axum::routing::get(get_members))
            .route("/servers/:server_id/members/:member_id/kick", axum::routing::post(kick_member))
            .route("/servers/:server_id/members/:user_id/role", axum::routing::put(update_member_role))
            .route("/servers/:server_id/members/:user_id/ban", axum::routing::post(ban_member).delete(unban_member))
            .route("/servers/:server_id/bans", axum::routing::get(get_bans))
            .route("/servers/:server_id/invitations", axum::routing::post(create_invitation))
            .route("/servers/:server_id/transfer", axum::routing::post(transfer_ownership))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    // ── create_server ──────────────────────────────────────────

    #[tokio::test]
    async fn test_create_server_success() {
        let user_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_create()
            .returning(move |_| Ok(make_server(user_id)));

        mock_server_repo
            .expect_add_member()
            .returning(move |_| Ok(make_member(user_id, Uuid::new_v4(), Role::Owner)));

        mock_channel_repo
            .expect_create()
            .returning(move |_| Ok(Channel {
                id: Uuid::new_v4(),
                server_id: Uuid::new_v4(),
                name: "general".to_string(),
                description: None,
                channel_type: "text".to_string(),
                position: 0,
                is_private: false,
                icon: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }));

        let app = make_router(mock_server_repo, mock_channel_repo);

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/servers")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "name": "Mon serveur" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_server_validation_error() {
        let user_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let app = make_router(MockServerRepository::new(), MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/servers")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "name": "" }).to_string())) // nom vide → invalide
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::CREATED);
    }

    // ── get_servers ────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_servers_success() {
        let user_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_by_user_id()
            .returning(move |_| Ok(vec![make_server(user_id)]));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/servers")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── get_server ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_server_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/servers/{}", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_server_not_member() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(|_, _| Ok(None));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/servers/{}", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::OK);
    }

    // ── delete_server ──────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_server_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_delete()
            .returning(|_| Ok(()));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/servers/{}", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_server_not_owner() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/servers/{}", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::NO_CONTENT);
    }

    // ── join_server ────────────────────────────────────────────

    #[tokio::test]
    async fn test_join_server_success() {
        let user_id = Uuid::new_v4();
        let inviter_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, inviter_id))));

        mock_server_repo
            .expect_find_member()
            .returning(|_, _| Ok(None));

        mock_server_repo
            .expect_add_member()
            .returning(move |_| Ok(make_member(user_id, server_id, Role::Member)));

        mock_server_repo
            .expect_increment_invitation_uses()
            .returning(|_| Ok(()));

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/join", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "invite_code": "CODE1234" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_join_server_deja_membre() {
        let user_id = Uuid::new_v4();
        let inviter_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, inviter_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/join", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "invite_code": "CODE1234" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::OK);
    }

    // ── leave_server ───────────────────────────────────────────

    #[tokio::test]
    async fn test_leave_server_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/leave", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_leave_server_owner_cannot_leave() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Owner))));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/leave", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::NO_CONTENT);
    }

    // ── kick_member ────────────────────────────────────────────

    #[tokio::test]
    async fn test_kick_member_success() {
        let owner_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(owner_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        let mut server = make_server(owner_id);
        server.id = server_id;

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(server.clone())));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(member_id, server_id, Role::Member))));

        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/members/{}/kick", server_id, member_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    // ── create_invitation ──────────────────────────────────────

    #[tokio::test]
    async fn test_create_invitation_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_server_repo
            .expect_create_invitation()
            .returning(move |_| Ok(make_invitation(server_id, user_id)));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/invitations", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({
                        "max_uses": 10,
                        "expires_in_hours": 24
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // ── transfer_ownership ─────────────────────────────────────

    #[tokio::test]
    async fn test_transfer_ownership_success() {
        let user_id = Uuid::new_v4();
        let new_owner_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        // ensure_member pour vérifier que user_id est owner
        mock_server_repo
            .expect_find_member()
            .returning(move |uid, _| {
                if uid == user_id {
                    Ok(Some(make_member(user_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(new_owner_id, server_id, Role::Member)))
                }
            });

        mock_server_repo
            .expect_update_member_role()
            .returning(move |_, _| Ok(make_member(new_owner_id, server_id, Role::Owner)));

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));

        mock_server_repo
            .expect_update()
            .returning(move |_| Ok(make_server(new_owner_id)));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/transfer", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({
                        "new_owner_id": new_owner_id.to_string()
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_transfer_ownership_invalid_uuid() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let app = make_router(MockServerRepository::new(), MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/transfer", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({
                        "new_owner_id": "pas-un-uuid"
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::OK);
    }

    // ── update_server ──────────────────────────────────────────

    #[tokio::test]
    async fn test_update_server_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));

        mock_server_repo
            .expect_update()
            .returning(move |_| Ok(make_server(user_id)));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/servers/{}", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "name": "Nouveau nom" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── get_members ────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_members_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_server_repo
            .expect_find_members_with_users()
            .returning(move |_| Ok(vec![]));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/servers/{}/members", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── join_server_by_code ────────────────────────────────────

    #[tokio::test]
    async fn test_join_server_by_code_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, creator_id))));
        mock_server_repo
            .expect_find_member()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_add_member()
            .returning(move |_| Ok(make_member(user_id, server_id, Role::Member)));
        mock_server_repo
            .expect_increment_invitation_uses()
            .returning(|_| Ok(()));
        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/servers/join")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "invite_code": "CODE5678" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── update_member_role ─────────────────────────────────────

    #[tokio::test]
    async fn test_update_member_role_success() {
        let owner_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(owner_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        let fixed_member_id = Uuid::new_v4();
        mock_server_repo
            .expect_find_member()
            .times(2)
            .returning(move |uid, _| {
                if uid == owner_id {
                    Ok(Some(make_member(owner_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(target_id, server_id, Role::Member)))
                }
            });

        let fixed = fixed_member_id;
        mock_server_repo
            .expect_update_member_role()
            .returning(move |_, _| {
                let mut m = make_member(target_id, server_id, Role::Admin);
                m.id = fixed;
                Ok(m)
            });

        mock_server_repo
            .expect_find_members_with_users()
            .returning(move |_| Ok(vec![
                crate::domain::repositories::server_repository::MemberWithUser {
                    id: fixed_member_id,
                    user_id: target_id,
                    server_id,
                    role: Role::Admin,
                    joined_at: Utc::now(),
                    username: "bob".to_string(),
                    avatar_url: None,
                    status: "online".to_string(),
                }
            ]));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/servers/{}/members/{}/role", server_id, target_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "role": "admin" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── ban_member ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_ban_member_success() {
        let owner_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(owner_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |uid, _| {
                if uid == owner_id {
                    Ok(Some(make_member(owner_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(member_id, server_id, Role::Member)))
                }
            });

        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));

        mock_server_repo
            .expect_create_ban()
            .returning(move |ban| Ok(ban.clone()));

        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/members/{}/ban", server_id, member_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "duration_hours": 24 }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // ── unban_member ───────────────────────────────────────────

    #[tokio::test]
    async fn test_unban_member_success() {
        let owner_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(owner_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(owner_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_remove_ban()
            .returning(|_, _| Ok(()));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/servers/{}/members/{}/ban", server_id, user_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    // ── get_bans ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_bans_success() {
        let owner_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(owner_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(owner_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_find_bans_by_server()
            .returning(|_| Ok(vec![]));

        let app = make_router(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/servers/{}/bans", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── socket.io variant tests ────────────────────────────────

    fn make_router_with_io(
        mock_server_repo: MockServerRepository,
        mock_channel_repo: MockChannelRepository,
    ) -> Router {
        let base_state = AppState::new_for_test(
            Arc::new(MockUserRepository::new()),
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(MockMessageRepository::new()),
            make_settings(),
        );
        let (_, io) = socketioxide::SocketIo::new_layer();
        io.ns("/", |_: socketioxide::extract::SocketRef| {});
        let state = base_state.with_socket_io(io);

        Router::new()
            .route("/servers/:server_id/leave", axum::routing::post(leave_server))
            .route("/servers/:server_id/members/:member_id/kick", axum::routing::post(kick_member))
            .route("/servers/:server_id/members/:user_id/role", axum::routing::put(update_member_role))
            .route("/servers/:server_id/members/:user_id/ban", axum::routing::post(ban_member).delete(unban_member))
            .route("/servers/:server_id/transfer", axum::routing::post(transfer_ownership))
            .route("/servers/:server_id/join", axum::routing::post(join_server))
            .route("/servers/join", axum::routing::post(join_server_by_code))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_join_server_with_socket_io() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut server = make_server(user_id);
        server.id = server_id;

        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, creator_id))));

        // First call: not yet a member; second call (from get_members): is member
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| {
                let n = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if n == 0 { Ok(None) } else { Ok(Some(make_member(user_id, server_id, Role::Member))) }
            });

        mock_server_repo
            .expect_add_member()
            .returning(move |_| Ok(make_member(user_id, server_id, Role::Member)));
        mock_server_repo
            .expect_increment_invitation_uses()
            .returning(|_| Ok(()));
        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(server.clone())));
        mock_server_repo
            .expect_find_members_with_users()
            .returning(move |_| Ok(vec![
                crate::domain::repositories::server_repository::MemberWithUser {
                    id: Uuid::new_v4(),
                    user_id,
                    server_id,
                    role: Role::Member,
                    joined_at: Utc::now(),
                    username: "alice".to_string(),
                    avatar_url: None,
                    status: "online".to_string(),
                }
            ]));

        let app = make_router_with_io(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/join", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "invite_code": "CODE9999" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_join_server_by_code_with_socket_io() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut server = make_server(user_id);
        server.id = server_id;

        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, creator_id))));

        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| {
                let n = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if n == 0 { Ok(None) } else { Ok(Some(make_member(user_id, server_id, Role::Member))) }
            });

        mock_server_repo
            .expect_add_member()
            .returning(move |_| Ok(make_member(user_id, server_id, Role::Member)));
        mock_server_repo
            .expect_increment_invitation_uses()
            .returning(|_| Ok(()));
        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(server.clone())));
        mock_server_repo
            .expect_find_members_with_users()
            .returning(move |_| Ok(vec![
                crate::domain::repositories::server_repository::MemberWithUser {
                    id: Uuid::new_v4(),
                    user_id,
                    server_id,
                    role: Role::Member,
                    joined_at: Utc::now(),
                    username: "alice".to_string(),
                    avatar_url: None,
                    status: "online".to_string(),
                }
            ]));

        let app = make_router_with_io(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/servers/join")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "invite_code": "CODE8888" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_leave_server_with_socket_io() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));
        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let app = make_router_with_io(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/leave", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_kick_member_with_socket_io() {
        let owner_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(owner_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let mut server = make_server(owner_id);
        server.id = server_id;

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(server.clone())));
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(member_id, server_id, Role::Member))));
        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let app = make_router_with_io(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/members/{}/kick", server_id, member_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_update_member_role_with_socket_io() {
        let owner_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(owner_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        let fixed_member_id = Uuid::new_v4();
        let fixed = fixed_member_id;

        mock_server_repo
            .expect_find_member()
            .returning(move |uid, _| {
                if uid == owner_id {
                    Ok(Some(make_member(owner_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(target_id, server_id, Role::Member)))
                }
            });

        mock_server_repo
            .expect_update_member_role()
            .returning(move |_, _| {
                let mut m = make_member(target_id, server_id, Role::Admin);
                m.id = fixed;
                Ok(m)
            });

        mock_server_repo
            .expect_find_members_with_users()
            .returning(move |_| Ok(vec![
                crate::domain::repositories::server_repository::MemberWithUser {
                    id: fixed_member_id,
                    user_id: target_id,
                    server_id,
                    role: Role::Admin,
                    joined_at: Utc::now(),
                    username: "bob".to_string(),
                    avatar_url: None,
                    status: "online".to_string(),
                }
            ]));

        let app = make_router_with_io(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/servers/{}/members/{}/role", server_id, target_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "role": "admin" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ban_member_with_socket_io() {
        let owner_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(owner_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        mock_server_repo
            .expect_find_member()
            .returning(move |uid, _| {
                if uid == owner_id {
                    Ok(Some(make_member(owner_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(member_id, server_id, Role::Member)))
                }
            });
        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_create_ban()
            .returning(move |ban| Ok(ban.clone()));
        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let app = make_router_with_io(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/members/{}/ban", server_id, member_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({ "duration_hours": 24 }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_transfer_ownership_with_socket_io() {
        let user_id = Uuid::new_v4();
        let new_owner_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let settings = make_settings();
        let token = make_token(user_id, &settings);

        let mut mock_server_repo = MockServerRepository::new();
        mock_server_repo
            .expect_find_member()
            .returning(move |uid, _| {
                if uid == user_id {
                    Ok(Some(make_member(user_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(new_owner_id, server_id, Role::Member)))
                }
            });
        mock_server_repo
            .expect_update_member_role()
            .returning(move |_, _| Ok(make_member(new_owner_id, server_id, Role::Owner)));
        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));
        mock_server_repo
            .expect_update()
            .returning(move |_| Ok(make_server(new_owner_id)));

        let app = make_router_with_io(mock_server_repo, MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/servers/{}/transfer", server_id))
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({
                        "new_owner_id": new_owner_id.to_string()
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ── sans token ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_sans_token_retourne_401() {
        let app = make_router(MockServerRepository::new(), MockChannelRepository::new());

        let response: axum::response::Response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/servers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}