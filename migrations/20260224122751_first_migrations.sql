CREATE TABLE IF NOT EXISTS engine_users (
    user_id UUID PRIMARY KEY,
    total_score INT NOT NULL DEFAULT 0
);


CREATE TABLE IF NOT EXISTS clans (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    leader_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE RESTRICT,
    tier VARCHAR(50) NOT NULL,
    total_score INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


CREATE TABLE IF NOT EXISTS clan_members (
    clan_id UUID NOT NULL REFERENCES clans(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (clan_id, user_id)
);

CREATE TABLE IF NOT EXISTS clan_buffs (
    id UUID PRIMARY KEY,
    clan_id UUID NOT NULL REFERENCES clans(id) ON DELETE CASCADE,
    buff_name VARCHAR(255) NOT NULL,
    multiplier DECIMAL(5,2) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMPTZ NOT NULL
);


CREATE TABLE IF NOT EXISTS achievements (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    milestone_target INT NOT NULL,
    achievement_type VARCHAR(100) NOT NULL
);


CREATE TABLE IF NOT EXISTS user_achievements (
    user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
    achievement_id UUID NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    current_progress INT NOT NULL DEFAULT 0,
    is_completed BOOLEAN NOT NULL DEFAULT false,
    is_shown_on_profile BOOLEAN NOT NULL DEFAULT false,
    completed_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, achievement_id)
);


CREATE TABLE IF NOT EXISTS daily_missions (
    id UUID PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    target_count INT NOT NULL,
    date DATE NOT NULL
);


CREATE TABLE IF NOT EXISTS user_missions (
    user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
    mission_id UUID NOT NULL REFERENCES daily_missions(id) ON DELETE CASCADE,
    current_progress INT NOT NULL DEFAULT 0,
    is_claimed BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (user_id, mission_id)
);

CREATE TABLE IF NOT EXISTS quiz_history (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
    article_id UUID NOT NULL,
    score INT NOT NULL DEFAULT 0,
    accuracy DECIMAL(5,2) NOT NULL DEFAULT 0.00,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);