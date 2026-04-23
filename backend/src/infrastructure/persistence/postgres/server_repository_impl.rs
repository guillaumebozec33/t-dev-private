use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::{Server, Member, Invitation, Ban};
use crate::domain::enums::Role;
use crate::domain::errors::DomainError;
use crate::domain::repositories::{ServerRepository, MemberWithUser};

pub struct PgServerRepository {
    pool: PgPool,
}

impl PgServerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServerRepository for PgServerRepository {
    async fn create(&self, server: &Server) -> Result<Server, DomainError> {
        let created = sqlx::query_as::<_, Server>(
            r#"
            INSERT INTO servers (id, name, description, icon_url, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(server.id)
        .bind(&server.name)
        .bind(&server.description)
        .bind(&server.icon_url)
        .bind(server.owner_id)
        .bind(server.created_at)
        .bind(server.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Server>, DomainError> {
        let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(server)
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Server>, DomainError> {
        let servers = sqlx::query_as::<_, Server>(
            r#"
            SELECT s.* FROM servers s
            INNER JOIN members m ON s.id = m.server_id
            WHERE m.user_id = $1
            ORDER BY s.created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(servers)
    }

    async fn update(&self, server: &Server) -> Result<Server, DomainError> {
        let updated = sqlx::query_as::<_, Server>(
            r#"
            UPDATE servers 
            SET name = $2, description = $3, icon_url = $4, owner_id = $5, updated_at = $6
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(server.id)
        .bind(&server.name)
        .bind(&server.description)
        .bind(&server.icon_url)
        .bind(server.owner_id)
        .bind(server.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM servers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn add_member(&self, member: &Member) -> Result<Member, DomainError> {
        let created = sqlx::query_as::<_, Member>(
            r#"
            INSERT INTO members (id, user_id, server_id, role, joined_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(member.id)
        .bind(member.user_id)
        .bind(member.server_id)
        .bind(&member.role)
        .bind(member.joined_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(created)
    }

    async fn find_member(&self, user_id: Uuid, server_id: Uuid) -> Result<Option<Member>, DomainError> {
        let member = sqlx::query_as::<_, Member>(
            "SELECT * FROM members WHERE user_id = $1 AND server_id = $2"
        )
        .bind(user_id)
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(member)
    }

    async fn find_members(&self, server_id: Uuid) -> Result<Vec<Member>, DomainError> {
        let members = sqlx::query_as::<_, Member>(
            "SELECT * FROM members WHERE server_id = $1 ORDER BY joined_at ASC"
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(members)
    }

    async fn find_members_with_users(&self, server_id: Uuid) -> Result<Vec<MemberWithUser>, DomainError> {
        let members = sqlx::query_as::<_, (Uuid, Uuid, Uuid, Role, chrono::DateTime<chrono::Utc>, String, Option<String>, String)>(
            r#"
            SELECT m.id, m.user_id, m.server_id, m.role, m.joined_at, u.username, u.avatar_url, u.status
            FROM members m
            INNER JOIN users u ON m.user_id = u.id
            WHERE m.server_id = $1
            ORDER BY m.joined_at ASC
            "#
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(members
            .into_iter()
            .map(|(id, user_id, server_id, role, joined_at, username, avatar_url, status)| {
                let normalized_status = status.to_lowercase();
                MemberWithUser {
                    id,
                    user_id,
                    server_id,
                    role,
                    joined_at,
                    username,
                    avatar_url,
                    status: normalized_status,
                }
            })
            .collect())
    }

    async fn update_member_role(&self, member_id: Uuid, role: Role) -> Result<Member, DomainError> {
        let updated = sqlx::query_as::<_, Member>(
            "UPDATE members SET role = $2 WHERE id = $1 RETURNING *"
        )
        .bind(member_id)
        .bind(&role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(updated)
    }

    async fn remove_member(&self, user_id: Uuid, server_id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM members WHERE user_id = $1 AND server_id = $2")
            .bind(user_id)
            .bind(server_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn create_invitation(&self, invitation: &Invitation) -> Result<Invitation, DomainError> {
        let created = sqlx::query_as::<_, Invitation>(
            r#"
            INSERT INTO invitations (id, server_id, code, created_by, max_uses, uses, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(invitation.id)
        .bind(invitation.server_id)
        .bind(&invitation.code)
        .bind(invitation.created_by)
        .bind(invitation.max_uses)
        .bind(invitation.uses)
        .bind(invitation.expires_at)
        .bind(invitation.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(created)
    }

    async fn find_invitation_by_code(&self, code: &str) -> Result<Option<Invitation>, DomainError> {
        let invitation = sqlx::query_as::<_, Invitation>(
            "SELECT * FROM invitations WHERE code = $1"
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(invitation)
    }

    async fn increment_invitation_uses(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("UPDATE invitations SET uses = uses + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn create_ban(&self, ban: &Ban) -> Result<Ban, DomainError> {
        let created = sqlx::query_as::<_, Ban>(
                r#"
                INSERT INTO bans (id, user_id, server_id, banned_by, banned_at, expires_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *,
                    (SELECT username FROM users WHERE id = user_id) as username
                "#
            )
        .bind(ban.id)
        .bind(ban.user_id)
        .bind(ban.server_id)
        .bind(ban.banned_by)
        .bind(ban.banned_at)
        .bind(ban.expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(created)
    }

    async fn find_active_ban(&self, user_id: Uuid, server_id: Uuid) -> Result<Option<Ban>, DomainError> {
        let ban = sqlx::query_as::<_, Ban>(
                r#"
                SELECT b.*, u.username
                FROM bans b
                JOIN users u ON u.id = b.user_id
                WHERE b.user_id = $1 AND b.server_id = $2 AND b.expires_at > NOW()
                "#
            )
        .bind(user_id)
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(ban)
    }

    async fn find_bans_by_server(&self, server_id: Uuid) -> Result<Vec<Ban>, DomainError> {
        let bans = sqlx::query_as::<_, Ban>(
            r#"
            SELECT b.*, u.username
            FROM bans b
            JOIN users u ON u.id = b.user_id
            WHERE b.server_id = $1 AND b.expires_at > NOW()
            ORDER BY b.banned_at DESC
            "#
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(bans)
    }

    async fn remove_ban(&self, user_id: Uuid, server_id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM bans WHERE user_id = $1 AND server_id = $2")
            .bind(user_id)
            .bind(server_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired_bans(&self) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM bans WHERE expires_at <= NOW()")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }
}
