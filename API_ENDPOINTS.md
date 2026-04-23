# API Endpoints Documentation

Base URL: `http://localhost:3001/api`

## 🔐 Authentication

### Public Endpoints

#### POST /auth/signup
Créer un nouveau compte utilisateur.

**Body:**
```json
{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

**Response:** `200 OK`
```json
{
  "token": "jwt_token",
  "user": {
    "id": "uuid",
    "username": "string",
    "email": "string",
    "avatar_url": "string | null",
    "status": "online"
  }
}
```

#### POST /auth/login
Authentifier un utilisateur.

**Body:**
```json
{
  "email": "string",
  "password": "string"
}
```

**Response:** `200 OK`
```json
{
  "token": "jwt_token",
  "user": {
    "id": "uuid",
    "username": "string",
    "email": "string",
    "avatar_url": "string | null",
    "status": "online"
  }
}
```

### Protected Endpoints

#### POST /auth/logout
Déconnecter l'utilisateur (invalide le token).

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`

#### GET /me
Récupérer les informations de l'utilisateur connecté.

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`
```json
{
  "id": "uuid",
  "username": "string",
  "email": "string",
  "avatar_url": "string | null",
  "status": "online | away | dnd | offline"
}
```

---

## 🏠 Servers (Communities/Guilds)

### POST /servers
Créer un nouveau serveur.

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "name": "string",
  "description": "string | null"
}
```

**Response:** `201 Created`
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string | null",
  "owner_id": "uuid",
  "icon_url": "string | null",
  "created_at": "datetime"
}
```

### GET /servers
Lister tous les serveurs de l'utilisateur.

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "name": "string",
    "description": "string | null",
    "owner_id": "uuid",
    "icon_url": "string | null",
    "created_at": "datetime"
  }
]
```

### GET /servers/:server_id
Récupérer les détails d'un serveur.

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string | null",
  "owner_id": "uuid",
  "icon_url": "string | null",
  "created_at": "datetime"
}
```

### PUT /servers/:server_id
Mettre à jour un serveur (owner uniquement).

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "name": "string",
  "description": "string | null"
}
```

**Response:** `200 OK`

### DELETE /servers/:server_id
Supprimer un serveur (owner uniquement).

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`

### POST /servers/join
Rejoindre un serveur via code d'invitation.

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "invite_code": "string"
}
```

**Response:** `200 OK`

### POST /servers/:server_id/join
Rejoindre un serveur directement (si public).

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`

### DELETE /servers/:server_id/leave
Quitter un serveur.

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`

### GET /servers/:server_id/members
Lister les membres d'un serveur.

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "user_id": "uuid",
    "username": "string",
    "server_id": "uuid",
    "role": "owner | admin | member",
    "status": "online | away | dnd | offline",
    "joined_at": "datetime"
  }
]
```

### DELETE /servers/:server_id/members/:member_id/kick
Expulser un membre du serveur (owner/admin uniquement).

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`

**WebSocket:** Émet `member_kicked` à tous les membres du serveur.

### POST /servers/:server_id/members/:member_id/ban
Bannir un membre du serveur.

**Headers:** `Authorization: Bearer {token}`

**Permissions:**
- Owner : Peut bannir tout le monde (sauf lui-même)
- Admin : Peut bannir uniquement les membres

**Body:**
```json
{
  "duration_hours": 24  // optionnel, undefined = permanent (1000 ans)
}
```

**Durées disponibles:**
- 1 heure : `1`
- 1 jour : `24`
- 2 jours : `48`
- 1 semaine : `168`
- 1 mois : `720`
- Permanent : `undefined`

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

**WebSocket:** Émet `member_banned` à tous les membres du serveur.

**Comportement:**
- Le membre est automatiquement expulsé (kick)
- Ses anciens messages restent visibles
- Il ne peut plus rejoindre le serveur via invitation

### DELETE /servers/:server_id/bans/:user_id
Débannir un utilisateur (owner uniquement).

**Headers:** `Authorization: Bearer {token}`

**Response:** `204 No Content`

### GET /servers/:server_id/bans
Lister les utilisateurs bannis du serveur (owner/admin uniquement).

**Headers:** `Authorization: Bearer {token}`

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

### PUT /servers/:server_id/members/:user_id
Changer le rôle d'un membre (owner/admin uniquement).

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "role": "admin | member"
}
```

**Response:** `200 OK`

**WebSocket:** Émet `member_role_changed` à tous les membres du serveur.

### PUT /servers/:server_id/owner
Transférer la propriété du serveur (owner uniquement).

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "new_owner_id": "uuid"
}
```

**Response:** `200 OK`

### POST /servers/:server_id/invitations
Générer un code d'invitation pour le serveur.

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "max_uses": "number"
}
```

**Response:** `201 Created`
```json
{
  "invite_code": "string",
  "expires_at": "datetime"
}
```

---

## 📺 Channels

### POST /servers/:server_id/channels
Créer un nouveau channel dans un serveur.

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "name": "string",
  "description": "string | null",
  "channel_type": "text | voice"
}
```

**Response:** `201 Created`
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string | null",
  "server_id": "uuid",
  "channel_type": "text | voice",
  "position": "number",
  "created_at": "datetime"
}
```

**WebSocket:** Émet `channel_created` à tous les membres du serveur.

### GET /servers/:server_id/channels
Lister tous les channels d'un serveur.

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "name": "string",
    "description": "string | null",
    "server_id": "uuid",
    "channel_type": "text | voice",
    "position": "number",
    "created_at": "datetime"
  }
]
```

### GET /channels/:channel_id
Récupérer les détails d'un channel.

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`

### PUT /channels/:channel_id
Mettre à jour un channel (admin/owner uniquement).

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "name": "string",
  "description": "string | null"
}
```

**Response:** `200 OK`

**WebSocket:** Émet `channel_updated` à tous les membres du serveur.

### DELETE /channels/:channel_id
Supprimer un channel (admin/owner uniquement).

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`

**WebSocket:** Émet `channel_deleted` à tous les membres du serveur.

---

## 💬 Messages

### POST /channels/:channel_id/messages
Envoyer un message dans un channel.

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "content": "string (1-2000 chars)"
}
```

**Response:** `201 Created`
```json
{
  "id": "uuid",
  "author_id": "uuid",
  "channel_id": "uuid",
  "content": "string",
  "author_username": "string",
  "author_avatar_url": "string | null",
  "edited": false,
  "created_at": "datetime",
  "updated_at": "datetime | null"
}
```

**WebSocket:** Émet `new_message` à tous les membres du channel.

### GET /channels/:channel_id/messages
Récupérer l'historique des messages d'un channel.

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "author_id": "uuid",
    "channel_id": "uuid",
    "content": "string",
    "author_username": "string",
    "author_avatar_url": "string | null",
    "edited": false,
    "created_at": "datetime",
    "updated_at": "datetime | null"
  }
]
```

### DELETE /messages/:message_id
Supprimer un message (auteur ou admin/owner uniquement).

**Headers:** `Authorization: Bearer {token}`

**Response:** `200 OK`

**WebSocket:** Émet `message_deleted` à tous les membres du channel.

### PATCH /messages/:message_id
Éditer un message (auteur uniquement).

**Headers:** `Authorization: Bearer {token}`

**Body:**
```json
{
  "content": "string (1-2000 chars)"
}
```

**Response:** `200 OK`

**WebSocket:** Émet `message_edited` à tous les membres du channel.

---

## 📝 Notes

### Authentification
- Tous les endpoints protégés nécessitent un header `Authorization: Bearer {token}`
- Le token JWT est obtenu via `/auth/login` ou `/auth/signup`
- Le token expire après 24h (configurable)

### Permissions
- **Owner** : Créateur du serveur, tous les droits
- **Admin** : Peut gérer channels, messages, membres (sauf owner)
- **Member** : Peut envoyer messages, voir channels

### WebSocket
- Les événements WebSocket sont émis automatiquement lors de certaines actions REST
- Les clients doivent rejoindre les rooms appropriées (`join_server`, `join_channel`)
- Voir `WEBSOCKET.md` pour la documentation complète des événements temps réel

### Codes d'erreur
- `400` : Bad Request (données invalides)
- `401` : Unauthorized (token manquant/invalide)
- `403` : Forbidden (permissions insuffisantes)
- `404` : Not Found (ressource inexistante)
- `500` : Internal Server Error


---

# WebSocket Documentation

URL de connexion: `ws://localhost:3001`

## 🔌 Connexion WebSocket

### Configuration Client
```typescript
import { io } from 'socket.io-client';

const socket = io('http://localhost:3001', {
  transports: ['websocket', 'polling'],
  auth: { token: 'jwt_token' },
  reconnection: true,
  reconnectionDelay: 1000,
  reconnectionAttempts: 5
});
```

### Événements de Connexion
- `connect` : Connexion établie
- `disconnect` : Déconnexion
- `connect_error` : Erreur de connexion

---

## 📤 Événements Client → Serveur

### join_server
Rejoindre la room d'un serveur.
```json
{ "server_id": "uuid" }
```

### leave_server
Quitter la room d'un serveur.
```json
{ "server_id": "uuid" }
```

### join_channel
Rejoindre la room d'un channel.
```json
{ "channel_id": "uuid" }
```

### leave_channel
Quitter la room d'un channel.
```json
{ "channel_id": "uuid" }
```

### typing_start
Indiquer qu'on écrit (debounce 300ms recommandé).
```json
{ "channel_id": "uuid", "user_id": "uuid" }
```

### typing_stop
Indiquer qu'on a arrêté d'écrire.
```json
{ "channel_id": "uuid", "user_id": "uuid" }
```

### update_status
Changer son statut.
```json
{ "status": "online | away | dnd | offline", "user_id": "uuid" }
```

---

## 📥 Événements Serveur → Client

### new_message
Nouveau message dans un channel (room: `channel:{id}`).
```json
{
  "id": "uuid",
  "author_id": "uuid",
  "channel_id": "uuid",
  "content": "string",
  "author_username": "string",
  "author_avatar_url": "string | null",
  "edited": false,
  "created_at": "datetime",
  "updated_at": "datetime | null"
}
```

### message_deleted
Message supprimé (room: `channel:{id}`).
```json
{ "message_id": "uuid", "channel_id": "uuid" }
```

### message_edited
Message édité (room: `channel:{id}`).
```json
{
  "id": "uuid",
  "channel_id": "uuid",
  "content": "string",
  "edited": true,
  "updated_at": "datetime"
}
```

### user_typing
Utilisateur en train d'écrire (room: `channel:{id}`).
```json
{ "channel_id": "uuid", "user_id": "uuid", "username": "string" }
```

### user_stop_typing
Utilisateur a arrêté d'écrire (room: `channel:{id}`).
```json
{ "channel_id": "uuid", "user_id": "uuid" }
```

### user_status_changed
Statut utilisateur changé (room: `server:{id}`).
```json
{ "server_id": "uuid", "user_id": "uuid", "status": "online | away | dnd | offline" }
```

### member_joined
Nouveau membre (room: `server:{id}`).
```json
{
  "server_id": "uuid",
  "user": {
    "id": "uuid",
    "user_id": "uuid",
    "username": "string",
    "role": "member",
    "status": "online",
    "joined_at": "datetime"
  }
}
```

### member_left
Membre parti (room: `server:{id}`).
```json
{ "server_id": "uuid", "user_id": "uuid" }
```

### member_kicked
Membre expulsé (room: `server:{id}`).
```json
{ "server_id": "uuid", "user_id": "uuid", "kicked_by": "uuid" }
```

### member_banned
Membre banni (room: `server:{id}`).
```json
{ "server_id": "uuid", "member_id": "uuid" }
```

### member_role_changed
Rôle modifié (room: `server:{id}`).
```json
{ "server_id": "uuid", "user_id": "uuid", "role": "owner | admin | member" }
```

### channel_created
Channel créé (room: `server:{id}`).
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string | null",
  "server_id": "uuid",
  "channel_type": "text | voice",
  "position": "number",
  "created_at": "datetime"
}
```

### channel_updated
Channel modifié (room: `server:{id}`).
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string | null",
  "server_id": "uuid",
  "channel_type": "text | voice",
  "position": "number",
  "created_at": "datetime"
}
```

### channel_deleted
Channel supprimé (room: `server:{id}`).
```json
{ "id": "uuid", "server_id": "uuid" }
```

### error
Erreur.
```json
{ "code": "string", "message": "string", "details": {} }
```

---

## 🏗️ Architecture des Rooms

### Channel Rooms
**Format:** `channel:{channel_id}`  
**Événements:** `new_message`, `message_deleted`, `user_typing`, `user_stop_typing`

### Server Rooms
**Format:** `server:{server_id}`  
**Événements:** `member_joined`, `member_left`, `member_role_changed`, `user_status_changed`, `channel_created`, `channel_updated`, `channel_deleted`

---

## 🎯 Hooks Frontend

| Hook | Description |
|------|-------------|
| `useMessageSync(channelId)` | Sync messages temps réel |
| `useChannelSync(serverId)` | Sync channels |
| `useMemberSync(serverId)` | Sync membres |
| `useMemberRoleSync(serverId)` | Sync changements rôles |
| `useTypingIndicator(channelId)` | Indicateur frappe |
| `useUserStatus(serverId)` | Gestion statuts |

---

## 📊 Résumé

| Action REST | Événement WebSocket | Room |
|-------------|---------------------|------|
| POST message | `new_message` | `channel:{id}` |
| PATCH message | `message_edited` | `channel:{id}` |
| DELETE message | `message_deleted` | `channel:{id}` |
| POST channel | `channel_created` | `server:{id}` |
| PUT channel | `channel_updated` | `server:{id}` |
| DELETE channel | `channel_deleted` | `server:{id}` |
| PUT member role | `member_role_changed` | `server:{id}` |
| POST join server | `member_joined` | `server:{id}` |
| DELETE leave server | `member_left` | `server:{id}` |
| DELETE kick member | `member_kicked` | `server:{id}` |
