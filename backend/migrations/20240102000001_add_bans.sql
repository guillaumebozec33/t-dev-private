-- Bans table
CREATE TABLE bans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    banned_by UUID NOT NULL REFERENCES users(id),
    banned_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE(user_id, server_id)
);

-- Index for performance
CREATE INDEX idx_bans_user_server ON bans(user_id, server_id);
CREATE INDEX idx_bans_expires_at ON bans(expires_at);
CREATE INDEX idx_bans_server_id ON bans(server_id);
