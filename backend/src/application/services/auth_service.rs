use std::sync::Arc;

use crate::application::dto::{SignupRequest, LoginRequest, AuthResponse, UserResponse};
use crate::domain::entities::User;
use crate::domain::errors::DomainError;
use crate::domain::repositories::UserRepository;
use crate::infrastructure::security::{hash_password, verify_password, create_token};
use crate::config::Settings;

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    settings: Settings,
}

impl AuthService {
    pub fn new(user_repo: Arc<dyn UserRepository>, settings: Settings) -> Self {
        Self { user_repo, settings }
    }

    pub async fn signup(&self, req: SignupRequest) -> Result<AuthResponse, DomainError> {
        if self.user_repo.find_by_email(&req.email).await?.is_some() {
            return Err(DomainError::EmailAlreadyExists);
        }

        if self.user_repo.find_by_username(&req.username).await?.is_some() {
            return Err(DomainError::UsernameAlreadyExists);
        }

        let password_hash = hash_password(&req.password)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let user = User::new(req.username, req.email, password_hash);
        let created_user = self.user_repo.create(&user).await?;

        let token = create_token(created_user.id, &self.settings.jwt_secret, self.settings.jwt_expiration)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(AuthResponse {
            token,
            user: UserResponse::from(created_user),
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, DomainError> {
        let user = self.user_repo
            .find_by_email(&req.email)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        if !verify_password(&req.password, &user.password_hash)
            .map_err(|e| DomainError::InternalError(e.to_string()))? 
        {
            return Err(DomainError::InvalidCredentials);
        }

        let token = create_token(user.id, &self.settings.jwt_secret, self.settings.jwt_expiration)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(AuthResponse {
            token,
            user: UserResponse::from(user),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use crate::domain::repositories::user_repository::MockUserRepository;
    use crate::config::Settings;
    use chrono::Utc;
    use uuid::Uuid;

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
            password_hash: "$argon2...".to_string(),
            avatar_url: None,
            status: crate::domain::enums::UserStatus::Online,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }


    #[tokio::test]
    async fn test_signup_success() {
        let mut mock_repo = MockUserRepository::new();

        // Email pas encore utilisé
        mock_repo
            .expect_find_by_email()
            .returning(|_| Ok(None));

        // Username pas encore utilisé
        mock_repo
            .expect_find_by_username()
            .returning(|_| Ok(None));

        // Création de l'utilisateur
        mock_repo
            .expect_create()
            .returning(|_| Ok(make_user()));

        let service = AuthService::new(Arc::new(mock_repo), make_settings());
        let req = SignupRequest {
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            password: "motdepasse123".to_string(),
        };

        let result = service.signup(req).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().token.is_empty());
    }

    #[tokio::test]
    async fn test_signup_email_deja_pris() {
        let mut mock_repo = MockUserRepository::new();

        // Email déjà utilisé → retourne un user existant
        mock_repo
            .expect_find_by_email()
            .returning(|_| Ok(Some(make_user())));

        let service = AuthService::new(Arc::new(mock_repo), make_settings());
        let req = SignupRequest {
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            password: "motdepasse123".to_string(),
        };

        let result = service.signup(req).await;
        assert!(matches!(result, Err(DomainError::EmailAlreadyExists)));
    }

    #[tokio::test]
    async fn test_signup_username_deja_pris() {
        let mut mock_repo = MockUserRepository::new();

        mock_repo
            .expect_find_by_email()
            .returning(|_| Ok(None));

        // Username déjà pris
        mock_repo
            .expect_find_by_username()
            .returning(|_| Ok(Some(make_user())));

        let service = AuthService::new(Arc::new(mock_repo), make_settings());
        let req = SignupRequest {
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            password: "motdepasse123".to_string(),
        };

        let result = service.signup(req).await;
        assert!(matches!(result, Err(DomainError::UsernameAlreadyExists)));
    }


    #[tokio::test]
    async fn test_login_success() {
        let mut mock_repo = MockUserRepository::new();
        let mut user = make_user();

        // Hash un vrai mot de passe pour que verify_password fonctionne
        user.password_hash = hash_password("motdepasse123").unwrap();

        mock_repo
            .expect_find_by_email()
            .returning(move |_| Ok(Some(user.clone())));

        let service = AuthService::new(Arc::new(mock_repo), make_settings());
        let req = LoginRequest {
            email: "guillaume@test.com".to_string(),
            password: "motdepasse123".to_string(),
        };

        let result = service.login(req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_login_user_inexistant() {
        let mut mock_repo = MockUserRepository::new();

        mock_repo
            .expect_find_by_email()
            .returning(|_| Ok(None)); // utilisateur introuvable

        let service = AuthService::new(Arc::new(mock_repo), make_settings());
        let req = LoginRequest {
            email: "inexistant@test.com".to_string(),
            password: "motdepasse123".to_string(),
        };

        let result = service.login(req).await;
        assert!(matches!(result, Err(DomainError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_login_mauvais_mot_de_passe() {
        let mut mock_repo = MockUserRepository::new();
        let mut user = make_user();
        user.password_hash = hash_password("bon_mot_de_passe").unwrap();

        mock_repo
            .expect_find_by_email()
            .returning(move |_| Ok(Some(user.clone())));

        let service = AuthService::new(Arc::new(mock_repo), make_settings());
        let req = LoginRequest {
            email: "guillaume@test.com".to_string(),
            password: "mauvais_mot_de_passe".to_string(), // ← mauvais
        };

        let result = service.login(req).await;
        assert!(matches!(result, Err(DomainError::InvalidCredentials)));
    }
}
