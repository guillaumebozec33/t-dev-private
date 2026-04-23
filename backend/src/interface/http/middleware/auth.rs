use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::infrastructure::security::verify_token;
use crate::shared::app_state::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, DomainError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(DomainError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(DomainError::Unauthorized)?;

    let claims = verify_token(token, &state.settings.jwt_secret)
        .map_err(|_| DomainError::Unauthorized)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| DomainError::Unauthorized)?;

    request.extensions_mut().insert(AuthUser { id: user_id });

    Ok(next.run(request).await)
}
