# 🚫 Système de Bannissement

## Vue d'ensemble

Le système de bannissement permet aux propriétaires et administrateurs de serveurs de bannir temporairement ou définitivement des membres.

## Fonctionnalités

### Permissions

- **Owner (Propriétaire)** : Peut bannir n'importe quel membre (sauf lui-même)
- **Admin (Administrateur)** : Peut bannir uniquement les membres (pas les autres admins ni l'owner)
- **Member (Membre)** : Ne peut pas bannir

### Durées de bannissement

- **1 heure**
- **1 jour**
- **2 jours**
- **1 semaine**
- **1 mois**
- **Permanent** (implémenté comme 1000 ans)

### Comportement

1. **Lors du bannissement** :
   - Le membre est automatiquement expulsé du serveur
   - Ses anciens messages restent visibles
   - Il ne peut plus rejoindre le serveur via invitation
   - Un événement WebSocket `member_banned` est émis

2. **Tentative de rejoindre** :
   - Message d'erreur : "Vous êtes banni du serveur jusqu'au [date]"
   - Ou : "Vous êtes banni de ce serveur définitivement"

3. **Expiration** :
   - Vérification à la volée lors de tentative de rejoin
   - Nettoyage automatique des bans expirés en base de données

### Gestion des bans

- **Liste des bans** : Accessible via un bouton 🚫 dans la liste des membres (Owner/Admin uniquement)
- **Débannir** : Seul l'owner peut débannir un utilisateur avant l'expiration
- **Informations affichées** :
  - ID de l'utilisateur banni
  - Date du bannissement
  - Date d'expiration
  - Temps restant
  - Type (temporaire/permanent)

## API Endpoints

### POST `/servers/:server_id/members/:member_id/ban`
Bannir un membre.

**Body:**
```json
{
  "duration_hours": 24  // optionnel, undefined = permanent
}
```

**Response:** `201 Created`
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "server_id": "uuid",
  "banned_by": "uuid",
  "banned_at": "datetime",
  "expires_at": "datetime",
  "is_permanent": false
}
```

### DELETE `/servers/:server_id/bans/:user_id`
Débannir un utilisateur (Owner uniquement).

**Response:** `204 No Content`

### GET `/servers/:server_id/bans`
Lister les utilisateurs bannis (Owner/Admin).

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "user_id": "uuid",
    "server_id": "uuid",
    "banned_by": "uuid",
    "banned_at": "datetime",
    "expires_at": "datetime",
    "is_permanent": false
  }
]
```

## WebSocket

### Événement `member_banned`
Émis dans la room `server:{server_id}` quand un membre est banni.

```json
{
  "server_id": "uuid",
  "member_id": "uuid"
}
```

## Base de données

### Table `bans`

```sql
CREATE TABLE bans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    banned_by UUID NOT NULL REFERENCES users(id),
    banned_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE(user_id, server_id)
);
```

**Index** :
- `idx_bans_user_server` : Performance pour vérifier si un utilisateur est banni
- `idx_bans_expires_at` : Performance pour le nettoyage des bans expirés
- `idx_bans_server_id` : Performance pour lister les bans d'un serveur

## UI/UX

### Menu contextuel (clic droit sur membre)

**Owner** :
- Transférer propriété
- Promouvoir Admin
- Rétrograder Membre
- Kick
- **Bannir** (rouge)

**Admin** :
- **Bannir** (rouge) - uniquement pour les membres

### Modale de sélection de durée

- Design responsive
- Sélection par boutons radio stylisés
- Couleurs cohérentes avec le site (bordeaux)
- Confirmation avec bouton rouge

### Liste des bans

- Accessible via bouton 🚫 dans l'en-tête de la liste des membres
- Affichage des informations de ban
- Bouton "Débannir" (vert) pour l'owner
- Scroll si beaucoup de bans
- Design responsive

## Codes d'erreur

- `403 FORBIDDEN` : Permissions insuffisantes
- `403 USER_BANNED` : Utilisateur banni (avec date d'expiration)
- `403 USER_BANNED_PERMANENTLY` : Utilisateur banni définitivement
- `404 MEMBER_NOT_FOUND` : Membre introuvable
- `400 VALIDATION_ERROR` : Données invalides (ex: se bannir soi-même)

## Migration

Pour appliquer la migration :

```bash
cd backend
sqlx migrate run
```

Ou manuellement :
```bash
psql -U rtc_user -d rtc_db -f migrations/20240102000001_add_bans.sql
```

## Tests

### Scénarios à tester

1. ✅ Owner peut bannir un admin
2. ✅ Owner peut bannir un membre
3. ✅ Owner ne peut pas se bannir lui-même
4. ✅ Admin peut bannir un membre
5. ✅ Admin ne peut pas bannir un autre admin
6. ✅ Admin ne peut pas bannir l'owner
7. ✅ Membre ne peut pas bannir
8. ✅ Utilisateur banni ne peut pas rejoindre via invitation
9. ✅ Ban temporaire expire correctement
10. ✅ Owner peut débannir
11. ✅ Admin ne peut pas débannir
12. ✅ WebSocket notifie correctement les membres
13. ✅ Liste des bans affiche les bonnes informations

## Notes techniques

- Les bans permanents sont implémentés comme des bans de 1000 ans pour simplifier la logique
- La vérification se fait à la volée lors de `join_server`
- Un job de nettoyage peut être ajouté pour supprimer les bans expirés (optionnel)
- Les messages de l'utilisateur banni restent visibles (historique)
- Le ban est au niveau serveur uniquement (pas global)
