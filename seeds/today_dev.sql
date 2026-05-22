-- Dev seed: missions for CURRENT_DATE + demo user in engine_users.
-- Safe to run multiple times (ON CONFLICT DO NOTHING).
--
-- Usage:
--   docker exec -i yomu-postgres psql -U yomu -d yomu_engine < seeds/today_dev.sql
-- Or from host with psql + DATABASE_URL.

BEGIN;

INSERT INTO engine_users (user_id, total_score) VALUES
    ('b3333333-3333-3333-3333-333333333333', 800)
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) VALUES
    ('d9999999-9999-9999-9999-999999999991', 'Baca 2 Artikel Hari Ini', 2, CURRENT_DATE, 20, 'ReadArticle'),
    ('d9999999-9999-9999-9999-999999999992', 'Kerjakan 1 Kuis Hari Ini', 1, CURRENT_DATE, 15, 'Quiz')
ON CONFLICT (id) DO NOTHING;

COMMIT;
