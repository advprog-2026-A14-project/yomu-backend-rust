-- ============================================================================
-- SEED DATA FOR YOMU RUST BACKEND (GAMIFICATION ENGINE)
-- ============================================================================
-- This migration seeds the gamification database with demo data for
-- development and testing. All INSERTs use ON CONFLICT DO NOTHING to
-- ensure idempotency — safe to run multiple times.
--
-- Seeding order (parents first, children after):
--   1. engine_users     (no dependencies)
--   2. shadow_users     (no dependencies, but conceptually depends on users)
--   3. achievements     (no dependencies)
--   4. daily_missions   (no dependencies)
--   5. clans            (depends on engine_users for leader_id)
--   6. clan_members     (depends on clans + engine_users)
--   7. clan_buffs       (depends on clans)
--   8. user_achievements(depends on engine_users + achievements)
--   9. user_missions    (depends on engine_users + daily_missions)
--  10. quiz_history     (depends on engine_users)
-- ============================================================================

BEGIN;

-- ============================================================================
-- FIXED UUID REFERENCE
-- ============================================================================
-- Users:
--   a1111111-1111-1111-1111-111111111111  alice_admin  ADMIN
--   a2222222-2222-2222-2222-222222222222  bob_admin    ADMIN
--   b3333333-3333-3333-3333-333333333333  charlie      PELAJAR
--   b4444444-4444-4444-4444-444444444444  diana        PELAJAR
--   b5555555-5555-5555-5555-555555555555  eric         PELAJAR
--   b6666666-6666-6666-6666-666666666666  fiona        PELAJAR
--   b7777777-7777-7777-7777-777777777777  george       PELAJAR
--   b8888888-8888-8888-8888-888888888888  hannah       PELAJAR
--
-- Achievements:
--   c1111111-1111-1111-1111-111111111111  ach-001
--   c2222222-2222-2222-2222-222222222222  ach-002
--   c3333333-3333-3333-3333-333333333333  ach-003
--   c4444444-4444-4444-4444-444444444444  ach-004
--   c5555555-5555-5555-5555-555555555555  ach-005
--   c6666666-6666-6666-6666-666666666666  ach-006
--   c7777777-7777-7777-7777-777777777777  ach-007
--   c8888888-8888-8888-8888-888888888888  ach-008
--
-- Daily Missions:
--   d1111111-1111-1111-1111-111111111111  miss-001
--   d2222222-2222-2222-2222-222222222222  miss-002
--   d3333333-3333-3333-3333-333333333333  miss-003
--   d4444444-4444-4444-4444-444444444444  miss-004
--   d5555555-5555-5555-5555-555555555555  miss-005
--   d6666666-6666-6666-6666-666666666666  miss-006
--
-- Clans:
--   e1111111-1111-1111-1111-111111111111  clan-001
--   e2222222-2222-2222-2222-222222222222  clan-002
--   e3333333-3333-3333-3333-333333333333  clan-003
--
-- Clan Buffs:
--   90111111-1111-1111-1111-111111111111  buff-001
--   90222222-2222-2222-2222-222222222222  buff-002
--   90333333-3333-3333-3333-333333333333  buff-003
--   90444444-4444-4444-4444-444444444444  buff-004
--
-- Quiz History:
--   f1111111-1111-1111-1111-111111111111  hist-001
--   f2222222-2222-2222-2222-222222222222  hist-002
--   f3333333-3333-3333-3333-333333333333  hist-003
--   f4444444-4444-4444-4444-444444444444  hist-004
--   f5555555-5555-5555-5555-555555555555  hist-005
--   f6666666-6666-6666-6666-666666666666  hist-006
--   f7777777-7777-7777-7777-777777777777  hist-007
--   f8888888-8888-8888-8888-888888888888  hist-008
--   f9999999-9999-9999-9999-999999999999  hist-009
--   faaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa  hist-010
--   fbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb  hist-011
--   fccccccc-cccc-cccc-cccc-cccccccccccc  hist-012
-- ============================================================================


-- ############################################################################
-- 1. engine_users
-- ############################################################################
-- Core user records in the gamification engine. These UUIDs match the
-- Java backend's auth users table for cross-backend consistency.
-- total_score reflects cumulative points earned through missions,
-- achievements, and clan activities.
-- ############################################################################

INSERT INTO engine_users (user_id, total_score) VALUES
    ('a1111111-1111-1111-1111-111111111111', 1200),  -- alice_admin
    ('a2222222-2222-2222-2222-222222222222', 950),   -- bob_admin
    ('b3333333-3333-3333-3333-333333333333', 800),   -- charlie
    ('b4444444-4444-4444-4444-444444444444', 1500),  -- diana
    ('b5555555-5555-5555-5555-555555555555', 600),   -- eric
    ('b6666666-6666-6666-6666-666666666666', 1100),  -- fiona
    ('b7777777-7777-7777-7777-777777777777', 400),   -- george
    ('b8888888-8888-8888-8888-888888888888', 1300)   -- hannah
ON CONFLICT (user_id) DO NOTHING;


-- ############################################################################
-- 2. shadow_users
-- ############################################################################
-- Shadow copies of users synced from the Java backend via the outbox
-- pattern. Used for user lookup and validation within the Rust backend.
-- total_score mirrors engine_users for consistency; created_at reflects
-- when each user first joined the platform.
-- ############################################################################

INSERT INTO shadow_users (user_id, total_score, created_at) VALUES
    ('a1111111-1111-1111-1111-111111111111', 1200, '2025-08-15 10:00:00+00'),  -- alice_admin
    ('a2222222-2222-2222-2222-222222222222', 950,  '2025-08-20 14:30:00+00'),  -- bob_admin
    ('b3333333-3333-3333-3333-333333333333', 800,  '2025-09-01 08:00:00+00'),  -- charlie
    ('b4444444-4444-4444-4444-444444444444', 1500, '2025-09-05 16:45:00+00'),  -- diana
    ('b5555555-5555-5555-5555-555555555555', 600,  '2025-10-12 09:15:00+00'),  -- eric
    ('b6666666-6666-6666-6666-666666666666', 1100, '2025-10-20 11:00:00+00'),  -- fiona
    ('b7777777-7777-7777-7777-777777777777', 400,  '2025-11-05 13:30:00+00'),  -- george
    ('b8888888-8888-8888-8888-888888888888', 1300, '2025-11-15 07:45:00+00')   -- hannah
ON CONFLICT (user_id) DO NOTHING;


-- ############################################################################
-- 3. achievements
-- ############################################################################
-- Achievement definitions. Users earn these by meeting milestone targets.
-- achievement_type must be one of: Common, Rare, Epic, Legendary.
-- reward_points are granted to the user upon completion.
-- ############################################################################

INSERT INTO achievements (id, name, milestone_target, achievement_type, reward_points) VALUES
    ('c1111111-1111-1111-1111-111111111111', 'First Steps',        1,   'Common',    10),   -- ach-001
    ('c2222222-2222-2222-2222-222222222222', 'Daily Reader',       7,   'Common',    15),   -- ach-002
    ('c3333333-3333-3333-3333-333333333333', 'Quiz Master',        10,  'Rare',      25),   -- ach-003
    ('c4444444-4444-4444-4444-444444444444', 'Commentator',        5,   'Rare',      20),   -- ach-004
    ('c5555555-5555-5555-5555-555555555555', 'Clan Founder',       1,   'Rare',      30),   -- ach-005
    ('c6666666-6666-6666-6666-666666666666', 'Polyglot Explorer',  50,  'Epic',      50),   -- ach-006
    ('c7777777-7777-7777-7777-777777777777', 'Streak Keeper',      30,  'Epic',      60),   -- ach-007
    ('c8888888-8888-8888-8888-888888888888', 'Legendary Scholar',  100, 'Legendary', 100)   -- ach-008
ON CONFLICT (id) DO NOTHING;


-- ############################################################################
-- 4. daily_missions
-- ############################################################################
-- Time-limited missions that users can complete for reward points.
-- mission_type must be one of: ReadArticle, Quiz, DailyLogin.
-- Each mission is valid for a specific date.
-- ############################################################################

INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) VALUES
    ('d1111111-1111-1111-1111-111111111111', 'Baca 2 Artikel',    2, '2026-05-01', 20, 'ReadArticle'),  -- miss-001
    ('d2222222-2222-2222-2222-222222222222', 'Kerjakan 3 Kuis',   3, '2026-05-01', 30, 'Quiz'),         -- miss-002
    ('d3333333-3333-3333-3333-333333333333', 'Login Harian',      1, '2026-05-02', 10, 'DailyLogin'),   -- miss-003
    ('d4444444-4444-4444-4444-444444444444', 'Baca 5 Artikel',    5, '2026-05-02', 40, 'ReadArticle'),  -- miss-004
    ('d5555555-5555-5555-5555-555555555555', 'Kerjakan 1 Kuis',   1, '2026-05-03', 15, 'Quiz'),         -- miss-005
    ('d6666666-6666-6666-6666-666666666666', 'Login 3 Hari',      3, '2026-05-03', 25, 'DailyLogin')    -- miss-006
ON CONFLICT (id) DO NOTHING;


-- ############################################################################
-- 5. clans
-- ############################################################################
-- User groups that compete on the leaderboard.
-- tier must be one of: Bronze, Silver, Gold, Diamond.
-- leader_id must reference an existing engine_users row.
-- total_score is the clan's aggregate score.
-- ############################################################################

INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES
    ('e1111111-1111-1111-1111-111111111111', 'Polyglot Pioneers',
        'b3333333-3333-3333-3333-333333333333', 'Bronze', 2400, '2025-12-01 10:00:00+00'),  -- clan-001, leader: charlie

    ('e2222222-2222-2222-2222-222222222222', 'Linguistic Legends',
        'b4444444-4444-4444-4444-444444444444', 'Silver', 3800, '2025-12-10 14:00:00+00'),  -- clan-002, leader: diana

    ('e3333333-3333-3333-3333-333333333333', 'Word Warriors',
        'b5555555-5555-5555-5555-555555555555', 'Gold',   5100, '2025-12-20 09:00:00+00')   -- clan-003, leader: eric
ON CONFLICT (id) DO NOTHING;


-- ############################################################################
-- 6. clan_members
-- ############################################################################
-- Many-to-many relationship linking users to clans.
-- Every clan has a leader (already referenced in clans.leader_id).
-- Composite primary key: (clan_id, user_id).
-- Both FKs cascade on delete (member removed if user or clan deleted).
-- ############################################################################

INSERT INTO clan_members (clan_id, user_id, joined_at) VALUES
    -- clan-001 "Polyglot Pioneers": charlie (leader), fiona, george
    ('e1111111-1111-1111-1111-111111111111', 'b3333333-3333-3333-3333-333333333333', '2025-12-01 10:00:00+00'),  -- charlie
    ('e1111111-1111-1111-1111-111111111111', 'b6666666-6666-6666-6666-666666666666', '2025-12-02 11:30:00+00'),  -- fiona
    ('e1111111-1111-1111-1111-111111111111', 'b7777777-7777-7777-7777-777777777777', '2025-12-03 09:15:00+00'),  -- george

    -- clan-002 "Linguistic Legends": diana (leader), alice, bob
    ('e2222222-2222-2222-2222-222222222222', 'b4444444-4444-4444-4444-444444444444', '2025-12-10 14:00:00+00'),  -- diana
    ('e2222222-2222-2222-2222-222222222222', 'a1111111-1111-1111-1111-111111111111', '2025-12-11 08:00:00+00'),  -- alice
    ('e2222222-2222-2222-2222-222222222222', 'a2222222-2222-2222-2222-222222222222', '2025-12-12 10:45:00+00'),  -- bob

    -- clan-003 "Word Warriors": eric (leader), hannah
    ('e3333333-3333-3333-3333-333333333333', 'b5555555-5555-5555-5555-555555555555', '2025-12-20 09:00:00+00'),  -- eric
    ('e3333333-3333-3333-3333-333333333333', 'b8888888-8888-8888-8888-888888888888', '2025-12-21 11:00:00+00')   -- hannah
ON CONFLICT (clan_id, user_id) DO NOTHING;


-- ############################################################################
-- 7. clan_buffs
-- ############################################################################
-- Temporary or permanent multipliers applied to clan members' activities.
-- multiplier is a DECIMAL(5,2) representing the score multiplier (e.g., 1.50 = +50%).
-- is_active controls whether the buff is currently in effect.
-- expires_at marks when an active buff becomes inactive.
-- ############################################################################

INSERT INTO clan_buffs (id, clan_id, buff_name, multiplier, is_active, expires_at) VALUES
    ('71111111-1111-1111-1111-111111111111', 'e1111111-1111-1111-1111-111111111111',
        'XP Boost',          1.50, true,  '2026-06-01 00:00:00+00'),  -- buff-001: clan-001 active

    ('72222222-2222-2222-2222-222222222222', 'e1111111-1111-1111-1111-111111111111',
        'Score Multiplier',  2.00, true,  '2026-06-15 00:00:00+00'),  -- buff-002: clan-001 active

    ('73333333-3333-3333-3333-333333333333', 'e2222222-2222-2222-2222-222222222222',
        'Reading Buff',      1.25, false, '2026-04-01 00:00:00+00'),  -- buff-003: clan-002 expired

    ('74444444-4444-4444-4444-444444444444', 'e3333333-3333-3333-3333-333333333333',
        'Quiz Master Buff',  1.75, true,  '2026-07-01 00:00:00+00')   -- buff-004: clan-003 active
ON CONFLICT (id) DO NOTHING;


-- ############################################################################
-- 8. user_achievements
-- ############################################################################
-- Tracks each user's progress toward achievements.
-- current_progress: how many milestone units the user has completed.
-- is_completed: true when current_progress >= achievement.milestone_target.
-- is_shown_on_profile: whether the user displays this achievement publicly.
-- completed_at: timestamp when the achievement was first completed.
--
-- Composite primary key: (user_id, achievement_id).
-- ############################################################################

INSERT INTO user_achievements (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile, completed_at) VALUES
    -- charlie: First Steps (done), Quiz Master (in progress)
    ('b3333333-3333-3333-3333-333333333333', 'c1111111-1111-1111-1111-111111111111', 1,  true,  true,  '2026-02-15 10:00:00+00'),
    ('b3333333-3333-3333-3333-333333333333', 'c3333333-3333-3333-3333-333333333333', 5,  false, false, NULL),

    -- diana: First Steps (done), Daily Reader (done), Polyglot Explorer (in progress)
    ('b4444444-4444-4444-4444-444444444444', 'c1111111-1111-1111-1111-111111111111', 1,  true,  true,  '2026-01-10 08:30:00+00'),
    ('b4444444-4444-4444-4444-444444444444', 'c2222222-2222-2222-2222-222222222222', 7,  true,  true,  '2026-03-01 12:00:00+00'),
    ('b4444444-4444-4444-4444-444444444444', 'c6666666-6666-6666-6666-666666666666', 25, false, false, NULL),

    -- eric: First Steps (done), Commentator (in progress)
    ('b5555555-5555-5555-5555-555555555555', 'c1111111-1111-1111-1111-111111111111', 1,  true,  true,  '2026-02-20 09:00:00+00'),
    ('b5555555-5555-5555-5555-555555555555', 'c4444444-4444-4444-4444-444444444444', 3,  false, false, NULL),

    -- fiona: First Steps (done), Clan Founder (done)
    ('b6666666-6666-6666-6666-666666666666', 'c1111111-1111-1111-1111-111111111111', 1,  true,  true,  '2026-01-25 11:00:00+00'),
    ('b6666666-6666-6666-6666-666666666666', 'c5555555-5555-5555-5555-555555555555', 1,  true,  true,  '2026-04-10 15:30:00+00'),

    -- george: First Steps (done), Daily Reader (in progress)
    ('b7777777-7777-7777-7777-777777777777', 'c1111111-1111-1111-1111-111111111111', 1,  true,  false, '2026-03-05 14:00:00+00'),
    ('b7777777-7777-7777-7777-777777777777', 'c2222222-2222-2222-2222-222222222222', 4,  false, false, NULL),

    -- hannah: First Steps (done), Streak Keeper (in progress)
    ('b8888888-8888-8888-8888-888888888888', 'c1111111-1111-1111-1111-111111111111', 1,  true,  true,  '2026-01-30 16:00:00+00'),
    ('b8888888-8888-8888-8888-888888888888', 'c7777777-7777-7777-7777-777777777777', 15, false, false, NULL),

    -- alice: First Steps (done)
    ('a1111111-1111-1111-1111-111111111111', 'c1111111-1111-1111-1111-111111111111', 1,  true,  true,  '2026-01-05 07:00:00+00'),

    -- bob: First Steps (done)
    ('a2222222-2222-2222-2222-222222222222', 'c1111111-1111-1111-1111-111111111111', 1,  true,  true,  '2026-01-08 08:00:00+00')
ON CONFLICT (user_id, achievement_id) DO NOTHING;


-- ############################################################################
-- 9. user_missions
-- ############################################################################
-- Tracks each user's progress toward daily missions.
-- current_progress: how many units the user has completed toward target_count.
-- is_claimed: whether the user has claimed the reward points.
--
-- Composite primary key: (user_id, mission_id).
-- ############################################################################

INSERT INTO user_missions (user_id, mission_id, current_progress, is_claimed) VALUES
    -- charlie: miss-001 done & claimed, miss-003 done & claimed
    ('b3333333-3333-3333-3333-333333333333', 'd1111111-1111-1111-1111-111111111111', 2, true),
    ('b3333333-3333-3333-3333-333333333333', 'd3333333-3333-3333-3333-333333333333', 1, true),

    -- diana: miss-001 done & claimed, miss-004 in progress (not claimed)
    ('b4444444-4444-4444-4444-444444444444', 'd1111111-1111-1111-1111-111111111111', 2, true),
    ('b4444444-4444-4444-4444-444444444444', 'd4444444-4444-4444-4444-444444444444', 3, false),

    -- eric: miss-002 done & claimed, miss-005 done & claimed
    ('b5555555-5555-5555-5555-555555555555', 'd2222222-2222-2222-2222-222222222222', 3, true),
    ('b5555555-5555-5555-5555-555555555555', 'd5555555-5555-5555-5555-555555555555', 1, true),

    -- fiona: miss-003 done & claimed, miss-006 in progress (not claimed)
    ('b6666666-6666-6666-6666-666666666666', 'd3333333-3333-3333-3333-333333333333', 1, true),
    ('b6666666-6666-6666-6666-666666666666', 'd6666666-6666-6666-6666-666666666666', 2, false),

    -- george: miss-001 in progress (not claimed)
    ('b7777777-7777-7777-7777-777777777777', 'd1111111-1111-1111-1111-111111111111', 1, false),

    -- hannah: miss-002 in progress (not claimed)
    ('b8888888-8888-8888-8888-888888888888', 'd2222222-2222-2222-2222-222222222222', 2, false)
ON CONFLICT (user_id, mission_id) DO NOTHING;


-- ############################################################################
-- 10. quiz_history
-- ############################################################################
-- Historical record of quiz attempts by users on various articles.
-- article_id references an article from the Java backend (no FK constraint
-- because articles live in a separate database).
-- score: absolute score (0-100).
-- accuracy: decimal percentage (0.00 to 1.00).
-- completed_at: when the quiz was completed.
-- ############################################################################

-- Article UUIDs (not stored in this DB, but referenced by article_id):
--   a0101010-1010-1010-1010-101010101010  article-01
--   a0202020-2020-2020-2020-202020202020  article-02
--   a0303030-3030-3030-3030-303030303030  article-03
--   a0404040-4040-4040-4040-404040404040  article-04
--   a0505050-5050-5050-5050-505050505050  article-05
--   a0606060-6060-6060-6060-606060606060  article-06
--   a0707070-7070-7070-7070-707070707070  article-07
--   a0808080-8080-8080-8080-808080808080  article-08
--   a0909090-9090-9090-9090-909090909090  article-09
--   a0aaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa  article-10
--   a0bbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb  article-11
--   a0cccccc-cccc-cccc-cccc-cccccccccccc  article-12

INSERT INTO quiz_history (id, user_id, article_id, score, accuracy, completed_at) VALUES
    ('f1111111-1111-1111-1111-111111111111', 'b3333333-3333-3333-3333-333333333333',
        'a0101010-1010-1010-1010-101010101010', 90, 0.90, '2026-05-01 09:00:00+00'),  -- hist-001: charlie

    ('f2222222-2222-2222-2222-222222222222', 'b4444444-4444-4444-4444-444444444444',
        'a0202020-2020-2020-2020-202020202020', 100, 1.00, '2026-05-01 10:30:00+00'),  -- hist-002: diana

    ('f3333333-3333-3333-3333-333333333333', 'b5555555-5555-5555-5555-555555555555',
        'a0303030-3030-3030-3030-303030303030', 80, 0.80, '2026-05-02 11:00:00+00'),  -- hist-003: eric

    ('f4444444-4444-4444-4444-444444444444', 'b6666666-6666-6666-6666-666666666666',
        'a0404040-4040-4040-4040-404040404040', 95, 0.95, '2026-05-02 14:00:00+00'),  -- hist-004: fiona

    ('f5555555-5555-5555-5555-555555555555', 'b7777777-7777-7777-7777-777777777777',
        'a0505050-5050-5050-5050-505050505050', 60, 0.60, '2026-05-03 08:15:00+00'),  -- hist-005: george

    ('f6666666-6666-6666-6666-666666666666', 'b8888888-8888-8888-8888-888888888888',
        'a0606060-6060-6060-6060-606060606060', 85, 0.85, '2026-05-03 09:45:00+00'),  -- hist-006: hannah

    ('f7777777-7777-7777-7777-777777777777', 'a1111111-1111-1111-1111-111111111111',
        'a0707070-7070-7070-7070-707070707070', 70, 0.70, '2026-05-04 10:00:00+00'),  -- hist-007: alice

    ('f8888888-8888-8888-8888-888888888888', 'a2222222-2222-2222-2222-222222222222',
        'a0808080-8080-8080-8080-808080808080', 75, 0.75, '2026-05-04 11:00:00+00'),  -- hist-008: bob

    ('f9999999-9999-9999-9999-999999999999', 'b3333333-3333-3333-3333-333333333333',
        'a0909090-9090-9090-9090-909090909090', 100, 1.00, '2026-05-04 14:30:00+00'),  -- hist-009: charlie

    ('faaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'b4444444-4444-4444-4444-444444444444',
        'a0aaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 55, 0.55, '2026-05-05 09:00:00+00'),  -- hist-010: diana

    ('fbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'b5555555-5555-5555-5555-555555555555',
        'a0bbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 90, 0.90, '2026-05-05 10:30:00+00'),  -- hist-011: eric

    ('fccccccc-cccc-cccc-cccc-cccccccccccc', 'b6666666-6666-6666-6666-666666666666',
        'a0cccccc-cccc-cccc-cccc-cccccccccccc', 65, 0.65, '2026-05-05 13:00:00+00')   -- hist-012: fiona
ON CONFLICT (id) DO NOTHING;


-- ============================================================================
-- SEED COMPLETE
-- ============================================================================
-- Summary of seeded rows:
--   engine_users:         8 rows
--   shadow_users:         8 rows
--   achievements:         8 rows
--   daily_missions:       6 rows
--   clans:                3 rows
--   clan_members:         8 rows
--   clan_buffs:           4 rows
--   user_achievements:   15 rows
--   user_missions:       10 rows
--   quiz_history:        12 rows
-- ============================================================================

COMMIT;
