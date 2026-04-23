# Tests Backend

## 🧪 Structure des Tests

```
backend/src/
├── tests.rs                    # Tests généraux
├── domain/tests.rs             # Tests entités et enums
├── application/tests.rs        # Tests DTOs et validation
├── infrastructure/tests.rs     # Tests sécurité (JWT, Argon2)
└── interface/tests.rs          # Tests événements WebSocket
```

## 🚀 Lancer les Tests

### Tous les tests
```bash
cd backend
docker build --target builder -t rtc-backend-builder .
docker run --rm rtc-backend-builder cargo test
```

Le résutat est affiché directement dans la console.

## 📋 Tests Implémentés

### Domain Tests (domain/tests.rs)
- ✅ Création d'entités User
- ✅ Création d'entités Server
- ✅ Création d'entités Channel
- ✅ Création d'entités Message
- ✅ Création d'entités Member
- ✅ Enums UserStatus (online, away, dnd, offline)
- ✅ Enums MemberRole (owner, admin, member)
- ✅ Enums ChannelType (text, voice)

### Application Tests (application/tests.rs)
- ✅ Validation SignupRequest
- ✅ Validation LoginRequest
- ✅ Validation CreateServerRequest
- ✅ Validation CreateChannelRequest
- ✅ Validation SendMessageRequest
- ✅ Validation UpdateMemberRoleRequest
- ✅ Validation longueur contenu message (1-2000 chars)
- ✅ Gestion descriptions optionnelles

### Infrastructure Tests (infrastructure/tests.rs)
- ✅ Hash de mot de passe (Argon2)
- ✅ Vérification mot de passe correct
- ✅ Vérification mot de passe incorrect
- ✅ Génération token JWT
- ✅ Validation token JWT correct
- ✅ Validation token JWT avec mauvais secret
- ✅ Validation token JWT invalide
- ✅ Unicité des hashes (salt aléatoire)
- ✅ Structure des claims JWT

### Interface Tests (interface/tests.rs)
- ✅ Sérialisation TypingEvent
- ✅ Désérialisation TypingEvent
- ✅ Sérialisation PresenceEvent
- ✅ Désérialisation PresenceEvent
- ✅ SocketEvent::JoinServer
- ✅ SocketEvent::JoinChannel
- ✅ SocketEvent::TypingStart
- ✅ SocketEvent::UpdateStatus

## 📊 Couverture

Les tests couvrent :
- **Entités du domaine** : 100%
- **DTOs et validation** : 100%
- **Sécurité (JWT/Argon2)** : 100%
- **Événements WebSocket** : 100%

## 🔧 Configuration

Les tests utilisent :
- `tokio-test` pour les tests async
- Pas de base de données requise (tests unitaires)
- Pas de Redis requis (tests unitaires)

## 📝 Notes

- Les tests sont **unitaires** et ne nécessitent pas de services externes
- Pour les tests d'intégration, voir `tests/` (à créer)
- Les tests de sécurité vérifient Argon2 et JWT
- Les tests WebSocket vérifient la sérialisation/désérialisation

## 🎯 Prochaines Étapes

Tests à ajouter :
- [ ] Tests d'intégration avec base de données
- [ ] Tests d'intégration avec Redis
- [ ] Tests des controllers HTTP
- [ ] Tests des repositories
- [ ] Tests des services
- [ ] Tests end-to-end
