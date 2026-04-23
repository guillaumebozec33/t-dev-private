use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invitation {
    pub id: Uuid,
    pub server_id: Uuid,
    pub code: String,
    pub created_by: Uuid,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Invitation {
    pub fn new(server_id: Uuid, created_by: Uuid, max_uses: Option<i32>, expires_at: Option<DateTime<Utc>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            server_id,
            code: Self::generate_code(),
            created_by,
            max_uses,
            uses: 0,
            expires_at,
            created_at: Utc::now(),
        }
    }

    fn generate_code() -> String {
        use std::iter;
        use rand::Rng;
        
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        iter::repeat_with(|| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .take(8)
            .collect()
    }

    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            if expires_at < Utc::now() {
                return false;
            }
        }
        
        if let Some(max_uses) = self.max_uses {
            if self.uses >= max_uses {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_invitation() {
        let server_id  = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let expires_at   = Utc::now() + chrono::Duration::days(1);
        let invitation = Invitation::new(server_id, created_by, Some(10), Some(expires_at));
        assert_eq!(invitation.server_id, server_id);
        assert_eq!(invitation.created_by, created_by);
        assert_eq!(invitation.max_uses, Some(10));
        assert_eq!(invitation.expires_at, Some(expires_at));
        assert!(invitation.code.len() == 8);
        assert!(invitation.id != Uuid::nil());
        assert!(invitation.uses == 0);
        assert!(invitation.created_at != DateTime::<Utc>::MIN_UTC);
        assert!(invitation.is_valid());
    }

    #[test]
    fn test_invitation_invalid() {
        let server_id  = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let expires_at   = Utc::now() - chrono::Duration::days(1);
        let invitation = Invitation::new(server_id, created_by, Some(0), Some(expires_at));
        assert_eq!(invitation.server_id, server_id);
        assert_eq!(invitation.created_by, created_by);
        assert_eq!(invitation.max_uses, Some(0));
        assert_eq!(invitation.expires_at, Some(expires_at));
        assert!(!invitation.is_valid());
    }

    #[test]
    fn test_invitation_uses(){
        let server_id  = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let expires_at   = Utc::now() + chrono::Duration::days(1);
        let mut invitation = Invitation::new(server_id, created_by, Some(2), Some(expires_at));
        invitation.uses = 2;
        assert!(!invitation.is_valid());

        invitation.uses = 1;
        assert!(invitation.is_valid());

        invitation.max_uses = None;
        assert!(invitation.is_valid());
        }

    #[test]
    fn test_generate_code(){
        let server_id  = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let expires_at   = Utc::now() - chrono::Duration::days(1);
        let invitation = Invitation::new(server_id, created_by, Some(0), Some(expires_at));
        assert!(invitation.code.len() == 8);
        let newInvitation = Invitation::new(server_id, created_by, Some(0), Some(expires_at));
        assert!(invitation.code != newInvitation.code);
        assert!(newInvitation.code.len() == 8);
    }


    #[test]
    fn test_invitation_no_expiry_no_max_uses() {
        let server_id  = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let invitation = Invitation::new(server_id, created_by, None, None);
        assert!(invitation.is_valid());
    }

    #[test]
    fn test_invitation_valid_expiry_no_max_uses() {
        let server_id  = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::days(1);
        let invitation = Invitation::new(server_id, created_by, None, Some(expires_at));
        assert!(invitation.is_valid());
    }
}

