-- Adds trigger_type to achievements so each achievement declares which user event
-- increments its progress. Existing rows default to 'QuizComplete'.
ALTER TABLE achievements
    ADD COLUMN trigger_type VARCHAR(50) NOT NULL DEFAULT 'QuizComplete';
