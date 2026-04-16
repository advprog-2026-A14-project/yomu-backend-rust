ALTER TABLE achievements ADD COLUMN reward_points INT NOT NULL DEFAULT 0;
ALTER TABLE daily_missions ADD COLUMN reward_points INT NOT NULL DEFAULT 0;