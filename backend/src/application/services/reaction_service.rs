use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{ToggleReactionRequest, ReactionResponse};
use crate::domain::entities::Reaction;
use crate::domain::errors::DomainError;
use crate::domain::repositories::{ReactionRepository, UserRepository};

pub struct ReactionService {
    reaction_repo: Arc<dyn ReactionRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl ReactionService {
    pub fn new(
        reaction_repo: Arc<dyn ReactionRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            reaction_repo,
            user_repo,
        }
    }

    /// Toggle a reaction: if the user already reacted with the same emoji, remove it.
    /// If the user reacted with a different emoji, update it. Otherwise, create it.
    /// Returns the updated list of reactions for the message.
    pub async fn toggle_reaction(
        &self,
        user_id: Uuid,
        message_id: Uuid,
        req: ToggleReactionRequest,
    ) -> Result<Vec<ReactionResponse>, DomainError> {
        let existing = self.reaction_repo.find_by_user_and_message(user_id, message_id).await?;

        match existing {
            Some(reaction) if reaction.emoji == req.emoji => {
                // Same emoji: remove the reaction
                self.reaction_repo.delete(reaction.id).await?;
            }
            Some(reaction) => {
                // Different emoji: update
                self.reaction_repo.update_emoji(reaction.id, &req.emoji).await?;
            }
            None => {
                // No existing reaction: create
                let reaction = Reaction::new(message_id, user_id, req.emoji);
                self.reaction_repo.create(&reaction).await?;
            }
        }

        self.get_reactions(message_id).await
    }

    pub async fn get_reactions(&self, message_id: Uuid) -> Result<Vec<ReactionResponse>, DomainError> {
        let reactions = self.reaction_repo.find_by_message_id(message_id).await?;
        let mut responses: Vec<ReactionResponse> = Vec::new();

        for reaction in reactions {
            let mut response = ReactionResponse::from(reaction.clone());
            if let Ok(Some(user)) = self.user_repo.find_by_id(reaction.user_id).await {
                response.username = Some(user.username);
            }
            responses.push(response);
        }

        Ok(responses)
    }

    pub async fn get_reactions_for_messages(&self, message_ids: &[Uuid]) -> Result<Vec<ReactionResponse>, DomainError> {
        let reactions = self.reaction_repo.find_by_message_ids(message_ids).await?;
        let mut responses: Vec<ReactionResponse> = Vec::new();

        for reaction in reactions {
            let mut response = ReactionResponse::from(reaction.clone());
            if let Ok(Some(user)) = self.user_repo.find_by_id(reaction.user_id).await {
                response.username = Some(user.username);
            }
            responses.push(response);
        }

        Ok(responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::Utc;
    use crate::domain::entities::{Reaction, User};
    use crate::domain::enums::UserStatus;
    use crate::domain::repositories::reaction_repository::MockReactionRepository;
    use crate::domain::repositories::user_repository::MockUserRepository;

    fn make_reaction(message_id: Uuid, user_id: Uuid, emoji: &str) -> Reaction {
        Reaction {
            id: Uuid::new_v4(),
            message_id,
            user_id,
            emoji: emoji.to_string(),
            created_at: Utc::now(),
        }
    }

    fn make_user(id: Uuid) -> User {
        User {
            id,
            username: "alice".to_string(),
            email: "alice@test.com".to_string(),
            password_hash: "hash".to_string(),
            avatar_url: None,
            status: UserStatus::Online,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ── toggle_reaction ────────────────────────────────────────

    // Cas 1 : aucune réaction existante → crée
    #[tokio::test]
    async fn test_toggle_reaction_creer() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        let mut mock_reaction_repo = MockReactionRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_reaction_repo
            .expect_find_by_user_and_message()
            .returning(|_, _| Ok(None));

        mock_reaction_repo
            .expect_create()
            .returning(move |r| Ok(make_reaction(message_id, user_id, &r.emoji)));

        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "👍")]));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let service = ReactionService::new(Arc::new(mock_reaction_repo), Arc::new(mock_user_repo));
        let req = ToggleReactionRequest { emoji: "👍".to_string() };
        let result = service.toggle_reaction(user_id, message_id, req).await;

        assert!(result.is_ok());
        let reactions = result.unwrap();
        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions[0].emoji, "👍");
        assert_eq!(reactions[0].username, Some("alice".to_string()));
    }

    // Cas 2 : même emoji déjà posé → supprime
    #[tokio::test]
    async fn test_toggle_reaction_supprimer_meme_emoji() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let existing = make_reaction(message_id, user_id, "👍");

        let mut mock_reaction_repo = MockReactionRepository::new();

        mock_reaction_repo
            .expect_find_by_user_and_message()
            .returning(move |_, _| Ok(Some(existing.clone())));

        mock_reaction_repo
            .expect_delete()
            .returning(|_| Ok(()));

        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(|_| Ok(vec![]));

        let service = ReactionService::new(Arc::new(mock_reaction_repo), Arc::new(MockUserRepository::new()));
        let req = ToggleReactionRequest { emoji: "👍".to_string() };
        let result = service.toggle_reaction(user_id, message_id, req).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    // Cas 3 : emoji différent → met à jour
    #[tokio::test]
    async fn test_toggle_reaction_changer_emoji() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let existing = make_reaction(message_id, user_id, "👍");

        let mut mock_reaction_repo = MockReactionRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_reaction_repo
            .expect_find_by_user_and_message()
            .returning(move |_, _| Ok(Some(existing.clone())));

        mock_reaction_repo
            .expect_update_emoji()
            .returning(move |_, _| Ok(make_reaction(message_id, user_id, "❤️")));

        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "❤️")]));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let service = ReactionService::new(Arc::new(mock_reaction_repo), Arc::new(mock_user_repo));
        let req = ToggleReactionRequest { emoji: "❤️".to_string() };
        let result = service.toggle_reaction(user_id, message_id, req).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0].emoji, "❤️");
    }

    // ── get_reactions ──────────────────────────────────────────

    #[tokio::test]
    async fn test_get_reactions_success() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        let mut mock_reaction_repo = MockReactionRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "🔥")]));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let service = ReactionService::new(Arc::new(mock_reaction_repo), Arc::new(mock_user_repo));
        let result = service.get_reactions(message_id).await;

        assert!(result.is_ok());
        let reactions = result.unwrap();
        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions[0].username, Some("alice".to_string()));
    }

    #[tokio::test]
    async fn test_get_reactions_vide() {
        let message_id = Uuid::new_v4();

        let mut mock_reaction_repo = MockReactionRepository::new();
        mock_reaction_repo
            .expect_find_by_message_id()
            .returning(|_| Ok(vec![]));

        let service = ReactionService::new(Arc::new(mock_reaction_repo), Arc::new(MockUserRepository::new()));
        let result = service.get_reactions(message_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    // ── get_reactions_for_messages ─────────────────────────────

    #[tokio::test]
    async fn test_get_reactions_for_messages_success() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        let mut mock_reaction_repo = MockReactionRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_reaction_repo
            .expect_find_by_message_ids()
            .returning(move |_| Ok(vec![make_reaction(message_id, user_id, "👍")]));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let service = ReactionService::new(Arc::new(mock_reaction_repo), Arc::new(mock_user_repo));
        let result = service.get_reactions_for_messages(&[message_id]).await;

        assert!(result.is_ok());
        let reactions = result.unwrap();
        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions[0].username, Some("alice".to_string()));
    }

    #[tokio::test]
    async fn test_get_reactions_for_messages_vide() {
        let message_id = Uuid::new_v4();

        let mut mock_reaction_repo = MockReactionRepository::new();
        mock_reaction_repo
            .expect_find_by_message_ids()
            .returning(|_| Ok(vec![]));

        let service = ReactionService::new(Arc::new(mock_reaction_repo), Arc::new(MockUserRepository::new()));
        let result = service.get_reactions_for_messages(&[message_id]).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
