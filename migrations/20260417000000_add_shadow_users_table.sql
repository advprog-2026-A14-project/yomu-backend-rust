-- Migration for user_sync shadow_users table
-- This table stores shadow user records synced from Java backend

CREATE TABLE IF NOT EXISTS shadow_users (
    user_id UUID PRIMARY KEY,
    total_score INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_shadow_users_user_id ON shadow_users(user_id);