use axum::{extract::State, http::StatusCode, Extension, Json};
use validator::Validate;

use crate::application::dto::{SignupRequest, LoginRequest, AuthResponse, UserResponse, UpdateUserRequest};
use crate::domain::errors::DomainError;
use crate::interface::http::middleware::AuthUser;
use crate::shared::app_state::AppState;
use chrono::Utc;

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, body = AuthResponse),
        (status = 400, description = "Validation error")
    )
)]
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), DomainError> {
    if let Err(validation_err) = req.validate() {
        return Err(DomainError::ValidationError(validation_err.to_string()));
    }

    let auth_response = state.auth_service.signup(req).await?;
    
    Ok((StatusCode::CREATED, Json(auth_response)))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = AuthResponse),
        (status = 400, description = "Validation error")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, DomainError> {
    if let Err(validation_err) = req.validate() {
        return Err(DomainError::ValidationError(validation_err.to_string()));
    }

    let auth_response = state.auth_service.login(req).await?;
    Ok(Json(auth_response))
}

// pub async fn logout() -> StatusCode {
//     StatusCode::OK
// }


#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, body = UserResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<UserResponse>, DomainError> {
    let user = state.user_repo
        .find_by_id(auth_user.id)
        .await?
        .ok_or(DomainError::UserNotFound)?;

    Ok(Json(UserResponse::from(user)))
}

#[utoipa::path(
    patch,
    path = "/auth/me",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, DomainError> {
    req.validate()
        .map_err(|e| DomainError::ValidationError(e.to_string()))?;

    let mut user = state.user_repo
        .find_by_id(auth_user.id)
        .await?
        .ok_or(DomainError::UserNotFound)?;

    let mut profile_changed = false;
    let mut status_changed = false;
    let mut new_status_str = String::new();

    if let Some(status) = req.status {
        new_status_str = status.clone();
        user.status = match status.as_str() {
            "online" => crate::domain::enums::UserStatus::Online,
            "dnd" => crate::domain::enums::UserStatus::DoNotDisturb,
            "away" => crate::domain::enums::UserStatus::Away,
            "invisible" => crate::domain::enums::UserStatus::Invisible,
            _ => return Err(DomainError::ValidationError("Invalid status. Use: online, dnd, away, or invisible".to_string())),
        };
        profile_changed = true;
        status_changed = true;
    }
    if let Some(username) = req.username {
        user.username = username;
        profile_changed = true;
    }
    if let Some(avatar_url) = req.avatar_url {
        user.avatar_url = avatar_url; // None = suppression, Some(url) = mise à jour
        profile_changed = true;
    }
    user.updated_at = Utc::now();

    let updated = state.user_repo.update(&user).await?;

    if profile_changed {
        if let Some(io) = &state.socket_io {
            if let Ok(servers) = state.server_repo.find_by_user_id(auth_user.id).await {
                for server in servers {
                    if status_changed {
                        crate::interface::websocket::handler::emit_user_status_changed(
                            io,
                            &server.id.to_string(),
                            &auth_user.id.to_string(),
                            &new_status_str
                        );
                    }

                    crate::interface::websocket::handler::emit_user_profile_updated(
                        io,
                        &server.id.to_string(),
                        &auth_user.id.to_string(),
                        &updated.username,
                        updated.avatar_url.as_deref(),
                        &format!("{:?}", updated.status).to_lowercase(),
                    );
                }
            }

            let profile_status = format!("{:?}", updated.status).to_lowercase();
            crate::interface::websocket::handler::emit_user_profile_updated_to_user_room(
                io,
                &auth_user.id.to_string(),
                &auth_user.id.to_string(),
                &updated.username,
                updated.avatar_url.as_deref(),
                &profile_status,
            );

            if let Ok(conversations) = state.dm_repo.find_conversations_by_user(auth_user.id).await {
                for conversation in conversations {
                    let other_user_id = conversation.other_user(auth_user.id).to_string();
                    crate::interface::websocket::handler::emit_user_profile_updated_to_user_room(
                        io,
                        &other_user_id,
                        &auth_user.id.to_string(),
                        &updated.username,
                        updated.avatar_url.as_deref(),
                        &profile_status,
                    );
                }
            }
        }
    }

    Ok(Json(UserResponse::from(updated)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::util::ServiceExt;
    use serde_json::json;
    use uuid::Uuid;
    use chrono::Utc;

    use crate::domain::entities::User;
    use crate::domain::enums::UserStatus;
    use crate::domain::repositories::server_repository::MockServerRepository;
    use crate::domain::repositories::channel_repository::MockChannelRepository;
    use crate::domain::repositories::message_repository::MockMessageRepository;
    use crate::domain::repositories::user_repository::MockUserRepository;
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

    fn make_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            password_hash: "hash".to_string(),
            avatar_url: None,
            status: UserStatus::Online,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // Crée un routeur de test avec les mocks injectés
    fn make_router(user_repo: MockUserRepository) -> Router {
        let state = AppState::new_for_test(
            Arc::new(user_repo),
            Arc::new(MockServerRepository::new()),
            Arc::new(MockChannelRepository::new()),
            Arc::new(MockMessageRepository::new()),
            make_settings(),
        );

        // Monte uniquement les routes auth pour ces tests
        Router::new()
            .route("/auth/signup", axum::routing::post(signup))
            .route("/auth/login", axum::routing::post(login))
//             .route("/auth/logout", axum::routing::post(logout))
            .with_state(state)
    }

    // ── signup ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_signup_success() {
        let mut mock_user_repo = MockUserRepository::new();
        mock_user_repo.expect_find_by_email().returning(|_| Ok(None));
        mock_user_repo.expect_find_by_username().returning(|_| Ok(None));
        mock_user_repo.expect_create().returning(|_| Ok(make_user()));

        let app = make_router(mock_user_repo);

        let response: axum::response::Response  = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/signup")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "username": "guillaume",
                        "email": "guillaume@test.com",
                        "password": "motdepasse123"
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_signup_validation_error() {
        // Pas besoin de mock — la validation échoue avant d'appeler le repo
        let app = make_router(MockUserRepository::new());

        let response: axum::response::Response  = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/signup")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "username": "ab",        // trop court < 3
                        "email": "pasunemail",   // email invalide
                        "password": "court"      // trop court < 8
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // ValidationError → ton DomainError doit retourner 400 ou 422
        assert_ne!(response.status(), StatusCode::CREATED);
    }

    // ── login ──────────────────────────────────────────────────

    #[tokio::test]
    async fn test_login_success() {
        let mut mock_user_repo = MockUserRepository::new();
        let mut user = make_user();

        use crate::infrastructure::security::hash_password;
        user.password_hash = hash_password("motdepasse123").unwrap();

        mock_user_repo
            .expect_find_by_email()
            .returning(move |_| Ok(Some(user.clone())));

        let app = make_router(mock_user_repo);

        let response: axum::response::Response  = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "email": "guillaume@test.com",
                        "password": "motdepasse123"
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_mauvais_mot_de_passe() {
        let mut mock_user_repo = MockUserRepository::new();
        let mut user = make_user();

        use crate::infrastructure::security::hash_password;
        user.password_hash = hash_password("bon_mot_de_passe").unwrap();

        mock_user_repo
            .expect_find_by_email()
            .returning(move |_| Ok(Some(user.clone())));

        let app = make_router(mock_user_repo);

        let response: axum::response::Response  = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "email": "guillaume@test.com",
                        "password": "mauvais_mot_de_passe"
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // InvalidCredentials → doit retourner 401
        assert_ne!(response.status(), StatusCode::OK);
    }
}