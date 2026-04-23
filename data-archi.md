# channel
- id (int)
- name (string)
- description (string)
- private (bool)
- categoryId (int)
- serverId (int)

# user
- id (int)
- name (string)
- photo (?)
- status (enum)
- color (string)

# category
- id (int)
- serverId (int)
- name (string)
- private (bool)

# message
- id (int)
- userId (int)
- channelId (int)
- serverId (int)
- content (string)
- deleted (bool)
- createdAt (string)
- editedAt (string)
- deletedAt (string)

# server
- id (int)
- name (string)
- description (string)
- ownerId (int)

# user-server
- userId (int)
- serverId (int)
- admin (bool)
- createdAt (string)

# invitation
- id (int)
- senderId (int)
- sentId (int)
- serverId (int)
- status (enum : Pending, Accepted, Refused, Expired)
- message (string)
- createdAt (string)
- updatedAt (string)
- expiresAt (string)