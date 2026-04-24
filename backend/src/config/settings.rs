use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub server_host: String,
    pub server_port: u16,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")?,
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiration: std::env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()?,
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_from_env_success() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("DATABASE_URL", "postgres://test");
        std::env::set_var("REDIS_URL", "redis://test");
        std::env::set_var("JWT_SECRET", "supersecret");
        std::env::set_var("JWT_EXPIRATION", "3600");
        std::env::set_var("SERVER_HOST", "0.0.0.0");
        std::env::set_var("SERVER_PORT", "8080");

        let settings = Settings::from_env().unwrap();
        assert_eq!(settings.database_url, "postgres://test");
        assert_eq!(settings.jwt_secret, "supersecret");
        assert_eq!(settings.jwt_expiration, 3600);
        assert_eq!(settings.server_port, 8080);
    }

    #[test]
    fn test_from_env_defaults() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("DATABASE_URL", "postgres://test2");
        std::env::set_var("REDIS_URL", "redis://test2");
        std::env::set_var("JWT_SECRET", "secret2");
        std::env::remove_var("JWT_EXPIRATION");
        std::env::remove_var("SERVER_HOST");
        std::env::remove_var("SERVER_PORT");

        let settings = Settings::from_env().unwrap();
        assert_eq!(settings.jwt_expiration, 86400);
        assert_eq!(settings.server_host, "0.0.0.0");
        assert_eq!(settings.server_port, 3001);
    }
}
