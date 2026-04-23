ALTER TABLE channels 
ADD COLUMN is_private BOOLEAN DEFAULT FALSE NOT NULL;

CREATE INDEX idx_channels_is_private ON channels(is_private);
