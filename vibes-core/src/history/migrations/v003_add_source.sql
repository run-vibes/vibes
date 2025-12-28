-- Add source column to track input origin (cli, web_ui, unknown)
ALTER TABLE messages ADD COLUMN source TEXT NOT NULL DEFAULT 'unknown';

-- Index for filtering by source (useful for analytics)
CREATE INDEX idx_messages_source ON messages(source);
