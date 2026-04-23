use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct SignupRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub status: String,
}

impl From<crate::domain::entities::User> for UserResponse {
    fn from(user: crate::domain::entities::User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            status: format!("{:?}", user.status).to_lowercase(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;
    use crate::domain::entities::User;
    use crate::domain::enums::UserStatus;
    use uuid::Uuid;
    use chrono::Utc;


    #[test]
    fn test_signup_valid() {
        let req = SignupRequest {
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            password: "motdepasse123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_signup_username_trop_court() {
        let req = SignupRequest {
            username: "ab".to_string(),
            email: "guillaume@test.com".to_string(),
            password: "motdepasse123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_signup_username_trop_long() {
        let req = SignupRequest {
            username: "a".repeat(33),
            email: "guillaume@test.com".to_string(),
            password: "motdepasse123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_signup_email_invalide() {
        let req = SignupRequest {
            username: "guillaume".to_string(),
            email: "pasunemail".to_string(),
            password: "motdepasse123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_signup_password_trop_court() {
        let req = SignupRequest {
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            password: "court".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_login_valid() {
        let req = LoginRequest {
            email: "guillaume@test.com".to_string(),
            password: "nimportequoi".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_login_email_invalide() {
        let req = LoginRequest {
            email: "pasunemail".to_string(),
            password: "nimportequoi".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_user_response_from_user() {
        let user = User {
            id: Uuid::new_v4(),
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            avatar_url: None,
            status: UserStatus::Online,
            password_hash: "hash_fake".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let response = UserResponse::from(user.clone());

        assert_eq!(response.id, user.id.to_string());
        assert_eq!(response.username, "guillaume");
        assert_eq!(response.email, "guillaume@test.com");
        assert_eq!(response.avatar_url, None);
        assert_eq!(response.status, "online");
    }

    #[test]
    fn test_user_response_with_avatar() {
        let user = User {
            id: Uuid::new_v4(),
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            status: UserStatus::Online,
            password_hash: "hash_fake".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = UserResponse::from(user);

        assert_eq!(response.avatar_url, Some("https://example.com/avatar.png".to_string()));
        assert_eq!(response.status, "online");
    }
}