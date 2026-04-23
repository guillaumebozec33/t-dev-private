use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Ban {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username:String,
    pub server_id: Uuid,
    pub banned_by: Uuid,
    pub banned_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Ban {
    pub fn new(user_id: Uuid, server_id: Uuid, banned_by: Uuid, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: String::new(),
            user_id,
            server_id,
            banned_by,
            banned_at: Utc::now(),
            expires_at,
        }
    }

    pub fn is_active(&self) -> bool {
        Utc::now() < self.expires_at
    }

    pub fn is_permanent(&self) -> bool {
        // Consider bans > 100 years as permanent
        let duration = self.expires_at - self.banned_at;
        duration.num_days() > 36500 // ~100 years
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use uuid::Uuid;

    fn make_ban(expires_in: Duration) -> Ban {
        Ban::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() + expires_in,
        )
    }

    #[test]
    fn test_new_ban() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let banned_by = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let ban = Ban::new(user_id, server_id, banned_by, expires_at);

        assert_eq!(ban.user_id, user_id);
        assert_eq!(ban.server_id, server_id);
        assert_eq!(ban.banned_by, banned_by);
        assert_eq!(ban.expires_at, expires_at);
    }

    #[test]
    fn test_is_active_non_expire() {
        let ban = make_ban(Duration::hours(1));
        assert!(ban.is_active());
    }

    #[test]
    fn test_is_active_expire() {
        let ban = make_ban(Duration::hours(-1));
        assert!(!ban.is_active());
    }

    #[test]
    fn test_is_permanent_vrai() {
        let ban = make_ban(Duration::days(40000));
        assert!(ban.is_permanent());
    }

    #[test]
    fn test_is_permanent_faux() {
        let ban = make_ban(Duration::days(7));
        assert!(!ban.is_permanent());
    }
}
