use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User not found")]
    UserNotFound,
    
    #[error("Server not found")]
    ServerNotFound,
    
    #[error("Channel not found")]
    ChannelNotFound,
    
    #[error("Message not found")]
    MessageNotFound,

    #[error("Conversation not found")]
    ConversationNotFound,
    
    #[error("Invitation not found or expired")]
    InvitationNotFound,
    
    #[error("Member not found")]
    MemberNotFound,
    
    #[error("User is banned from this server until {0}")]
    UserBanned(String),
    
    #[error("User is permanently banned from this server")]
    UserBannedPermanently,
    
    #[error("Email already exists")]
    EmailAlreadyExists,
    
    #[error("Username already exists")]
    UsernameAlreadyExists,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Already a member of this server")]
    AlreadyMember,
    
    #[error("Owner cannot leave the server")]
    OwnerCannotLeave,
    
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("You cannot use your own invitation code")]
    UseOwnInvitation,
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            DomainError::UserNotFound => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),
            DomainError::ServerNotFound => (StatusCode::NOT_FOUND, "SERVER_NOT_FOUND"),
            DomainError::ChannelNotFound => (StatusCode::NOT_FOUND, "CHANNEL_NOT_FOUND"),
            DomainError::MessageNotFound => (StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND"),
            DomainError::ConversationNotFound => (StatusCode::NOT_FOUND, "CONVERSATION_NOT_FOUND"),
            DomainError::InvitationNotFound => (StatusCode::NOT_FOUND, "INVITATION_NOT_FOUND"),
            DomainError::MemberNotFound => (StatusCode::NOT_FOUND, "MEMBER_NOT_FOUND"),

            DomainError::EmailAlreadyExists => (StatusCode::CONFLICT, "EMAIL_ALREADY_EXISTS"),
            DomainError::UsernameAlreadyExists => (StatusCode::CONFLICT, "USERNAME_ALREADY_EXISTS"),
            DomainError::AlreadyMember => (StatusCode::CONFLICT, "ALREADY_MEMBER"),

            DomainError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS"),
            DomainError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),

            DomainError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            DomainError::OwnerCannotLeave => (StatusCode::FORBIDDEN, "OWNER_CANNOT_LEAVE"),
            DomainError::UserBanned(_) => (StatusCode::FORBIDDEN, "USER_BANNED"),
            DomainError::UserBannedPermanently => (StatusCode::FORBIDDEN, "USER_BANNED_PERMANENTLY"),

            DomainError::ValidationError(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),

            DomainError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            DomainError::UseOwnInvitation => (StatusCode::BAD_REQUEST, "USE_OWN_INVITATION"),
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": self.to_string(),
            }
        }));

        (status, body).into_response()
    }
}