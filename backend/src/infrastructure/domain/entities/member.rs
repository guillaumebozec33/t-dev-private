use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::enums::Role;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Member {
    pub id: Uuid,
    pub user_id: Uuid,
    pub server_id: Uuid,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
}

impl Member {
    pub fn new(user_id: Uuid, server_id: Uuid, role: Role) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            server_id,
            role,
            joined_at: Utc::now(),
        }
    }

    pub fn is_owner(&self) -> bool {
        matches!(self.role, Role::Owner)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, Role::Owner | Role::Admin)
    }

    pub fn can_manage_channels(&self) -> bool {
        self.is_admin()
    }

    pub fn can_manage_members(&self) -> bool {
        self.is_owner()
    }

    pub fn can_delete_message(&self, author_id: Uuid) -> bool {
        self.user_id == author_id || self.is_admin()
    }

    pub fn can_create_invitation(&self) -> bool {
        true // Tous les membres peuvent créer des invitations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_owner() {
        let member = Member::new(Uuid::new_v4(), Uuid::new_v4(), Role::Owner);
        assert!(member.is_owner());
        assert!(member.can_manage_channels());
        assert!(member.can_manage_members());
    }

    #[test]
    fn test_member_admin() {
        let member = Member::new(Uuid::new_v4(), Uuid::new_v4(), Role::Admin);
        assert!(!member.is_owner());
        assert!(member.is_admin());
        assert!(member.can_manage_channels());
        assert!(!member.can_manage_members());
    }

    #[test]
    fn test_member_basic() {
        let member = Member::new(Uuid::new_v4(), Uuid::new_v4(), Role::Member);
        assert!(!member.is_owner());
        assert!(!member.is_admin());
        assert!(member.can_create_invitation());
    }

    #[test]
    fn test_can_delete_message() {
        let user_id = Uuid::new_v4();
        let member = Member::new(user_id, Uuid::new_v4(), Role::Member);
        assert!(member.can_delete_message(user_id));
        assert!(!member.can_delete_message(Uuid::new_v4()));
    }
}
