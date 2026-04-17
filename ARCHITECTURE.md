# Yomu Backend Rust Architecture

**Version:** 0.1.0
**Last Updated:** 2026-04-16
**Framework:** Axum 0.8.8 + Tower middleware
**Edition:** Rust 2024, MSRV 1.85
**Ports:** PostgreSQL 5432, Redis 6379, App 8080

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [System Architecture Diagram](#2-system-architecture-diagram)
3. [Module Architecture](#3-module-architecture)
4. [Clean Architecture Layers](#4-clean-architecture-layers)
5. [Layer Dependencies Diagram](#5-layer-dependencies-diagram)
6. [Hexagonal Architecture](#6-hexagonal-architecture)
7. [API Architecture](#7-api-architecture)
8. [Database Architecture](#8-database-architecture)
9. [Entity Relationship Diagrams](#9-entity-relationship-diagrams)
10. [Data Flow Diagrams](#10-data-flow-diagrams)
11. [Error Handling Architecture](#11-error-handling-architecture)
12. [CI/CD Architecture](#12-cicd-architecture)
13. [Deployment Architecture](#13-deployment-architecture)
14. [Technology Decisions](#14-technology-decisions)
15. [Design Patterns](#15-design-patterns)
16. [Code Organization](#16-code-organization)
17. [Key Interfaces](#17-key-interfaces)
18. [Configuration Management](#18-configuration-management)
19. [Security Considerations](#19-security-considerations)
20. [Performance Considerations](#20-performance-considerations)
21. [Testing Strategy](#21-testing-strategy)
22. [Future Considerations](#22-future-considerations)

---

## 1. Architecture Overview

The Yomu Backend Rust is a gamification engine built with Axum that provides clan management, leaderboards, achievements, missions, and user synchronization services. It operates as a standalone Rust microservice that receives events from the Java backend (via REST outbox pattern) and serves the Next.js frontend.

The architecture follows Clean Architecture principles with clear separation between domain logic, application services, infrastructure implementations, and presentation layers. Each bounded context (league, gamification, user_sync) maintains its own clean architecture structure while sharing common infrastructure components.

The system is designed for high performance with PostgreSQL as the source of truth for all data and Redis for leaderboard caching. The application layer uses async/await with tokio for concurrent request handling and implements the repository pattern for data access abstraction.

Key architectural characteristics include:

- **Reactive Data Flow**: All handlers are async, enabling high concurrency
- **Repository Pattern**: Data access is abstracted behind traits for testability
- **Domain-Driven Design**: Each module has clearly bounded contexts
- **Dependency Inversion**: Infrastructure depends on domain interfaces, not vice versa
- **Error Domain Mapping**: Domain-specific errors with proper HTTP status code translation

The service integrates with an external Java microservice (Java Core) for user data synchronization and quiz history, maintaining a "shadow user" concept where users from the Java system are represented in the Rust engine for gamification purposes.

---

## 2. System Architecture Diagram

```mermaid
flowchart TB
    subgraph External["External Systems"]
        JavaBackend["Java Backend<br/>(Port 8080)"]
        NextJSFrontend["Next.js Frontend<br/>(Port 3000)"]
        JavaCore["Java Core Service"]
    end

    subgraph YomuRust["Yomu Engine Rust"]
        subgraph Presentation["Presentation Layer"]
            HealthEndpoint["/health"]
            ClanController["Clan Controller"]
            ScoreController["Score Controller"]
            UserSyncController["User Sync Controller"]
            SwaggerUI["Swagger UI<br/>/swagger-ui"]
        end

        subgraph Application["Application Layer"]
            CreateClanUC["CreateClanUseCase"]
            JoinClanUC["JoinClanUseCase"]
            GetClanDetailUC["GetClanDetailUseCase"]
            GetLeaderboardUC["GetLeaderboardUseCase"]
            GetUserTierUC["GetUserTierUseCase"]
            SyncQuizGamUC["SyncQuizGamificationUseCase"]
            ClaimMissionUC["ClaimMissionRewardUseCase"]
            SyncNewUserUC["SyncNewUserUseCase"]
        end

        subgraph Domain["Domain Layer"]
            subgraph LeagueDomain["League Module"]
                Clan["Clan Entity"]
                ClanMember["ClanMember Entity"]
                Score["Score Entity"]
                ClanTier["ClanTier Enum"]
                MemberRole["MemberRole Enum"]
                LeagueError["LeagueError"]
            end

            subgraph GamificationDomain["Gamification Module"]
                Achievement["Achievement Entity"]
                DailyMission["DailyMission Entity"]
                UserAchievement["UserAchievement Entity"]
                UserMission["UserMission Entity"]
                AchievementType["AchievementType Enum"]
            end

            subgraph UserSyncDomain["User Sync Module"]
                ShadowUser["ShadowUser Entity"]
                UserSyncError["UserSyncError"]
            end

            subgraph SharedDomain["Shared Domain"]
                AppError["AppError"]
            end
        end

        subgraph Infrastructure["Infrastructure Layer"]
            subgraph Database["Database Repositories"]
                ClanPostgresRepo["ClanPostgresRepo"]
                LeaderboardRedisRepo["LeaderboardRedisRepo"]
                AchievementPostgresRepo["AchievementPostgresRepo"]
                MissionPostgresRepo["MissionPostgresRepo"]
                UserPostgresRepo["UserPostgresRepo"]
            end

            subgraph HttpClients["HTTP Clients"]
                HttpClient["HttpClient"]
                JavaCoreClient["JavaCoreClient"]
            end
        end
    end

    subgraph DataStores["Data Stores"]
        PostgreSQL[("PostgreSQL<br/>Port 5432")]
        Redis[("Redis<br/>Port 6379")]
    end

    NextJSFrontend -->|HTTP/REST| Presentation
    JavaBackend -->|Outbox Sync POST| UserSyncController
    JavaCore -->|HTTP API Calls| HttpClient

    Presentation --> Application
    Application --> Domain
    Application --> Infrastructure
    Infrastructure --> PostgreSQL
    Infrastructure --> Redis

    ClanController --> CreateClanUC
    ClanController --> JoinClanUC
    ClanController --> GetClanDetailUC
    ScoreController --> GetLeaderboardUC
    UserSyncController --> SyncNewUserUC

    CreateClanUC --> Clan
    JoinClanUC --> ClanMember
    GetLeaderboardUC --> LeaderboardRedisRepo
    SyncQuizGamUC --> Achievement
    SyncQuizGamUC --> DailyMission
```

---

## 3. Module Architecture

The system comprises three primary modules, each representing a distinct bounded context with its own domain model, application services, infrastructure adapters, and presentation components.

### 3.1 League Module

The League module handles all clan-related functionality including creation, membership management, scoring, and leaderboards.

**Bounded Context:**
- Clan lifecycle (creation, deletion concept, tier management)
- Clan membership and roles
- Score calculation with modifiers
- Tiered leaderboards (Bronze, Silver, Gold, Diamond)

**Core Entities:**
- `Clan`: Represents a clan with name, leader, tier, and total score
- `ClanMember`: Join table with user_id, clan_id, role, and joined_at
- `Score`: Value object for score calculations with buff/debuff multipliers
- `ClanTier`: Enum (Bronze, Silver, Gold, Diamond)
- `MemberRole`: Enum (Leader, Member)

**Key Use Cases:**
- CreateClanUseCase: Creates a new clan with the leader as first member
- JoinClanUseCase: Adds a user to an existing clan
- GetClanDetailUseCase: Retrieves full clan details with member list
- GetLeaderboardUseCase: Fetches top clans from Redis cache
- GetUserTierUseCase: Determines user's current tier information
- UpdateScoreUseCase: Updates clan score with potential modifiers

**Ports (Repository Traits):**
- `ClanRepository`: CRUD operations for clans and members
- `LeaderboardCache`: Redis-based leaderboard operations

**Adapters:**
- `ClanPostgresRepo`: PostgreSQL implementation of ClanRepository
- `LeaderboardRedisRepo`: Redis implementation of LeaderboardCache

### 3.2 Gamification Module

The Gamification module manages achievements, missions, and rewards to drive user engagement.

**Bounded Context:**
- Achievement definitions and user progress tracking
- Daily mission management and completion tracking
- Reward point system
- Automatic reward granting on achievement completion

**Core Entities:**
- `Achievement`: Milestone-based achievements with types (Common, Rare, Epic, Legendary)
- `DailyMission`: Date-specific missions with targets and rewards
- `UserAchievement`: User's progress toward an achievement
- `UserMission`: User's progress on a daily mission

**Key Use Cases:**
- SyncQuizGamificationUseCase: Processes quiz completions to update missions and achievements
- ClaimMissionRewardUseCase: Allows users to claim completed mission rewards

**Ports (Repository Traits):**
- `AchievementRepository`: Achievement and UserAchievement persistence
- `MissionRepository`: DailyMission and UserMission persistence

**Adapters:**
- `AchievementPostgresRepo`: PostgreSQL implementation
- `MissionPostgresRepo`: PostgreSQL implementation

### 3.3 User Sync Module

The User Sync module handles synchronization of user data and quiz history from the Java backend, creating shadow users for gamification purposes.

**Bounded Context:**
- Shadow user creation from Java outbox events
- Quiz history recording and score aggregation
- User existence validation
- Idempotent sync operations (returning existing user instead of error)

**Core Entities:**
- `ShadowUser`: Minimal user representation (user_id, total_score)
- `QuizHistory`: Quiz attempt record (id, user_id, article_id, score, accuracy, completed_at)

**Key Use Cases:**
- `SyncNewUserUseCase`: Creates a new shadow user from sync request (idempotent - returns existing user if already exists)
- `SyncQuizHistoryUseCase`: Records quiz history and updates user's total_score with validation (score >= 0, 0.0 <= accuracy <= 100.0)

**Ports (Repository Traits):**
- `UserRepository`: ShadowUser persistence operations (get_shadow_user, update_total_score)
- `QuizHistoryRepository`: Quiz history persistence operations (insert_quiz_history, get_quiz_histories_by_user)

**Adapters:**
- `UserPostgresRepo`: PostgreSQL implementation with total_score update
- `QuizHistoryPostgresRepo`: PostgreSQL implementation with NUMERIC→FLOAT8 casting for accuracy

---

## 4. Clean Architecture Layers

Each module follows Clean Architecture with four distinct layers that enforce separation of concerns and create testable, maintainable code structures.

### 4.1 Domain Layer (`domain/`)

The innermost layer containing pure business logic with no external dependencies. This layer declares interfaces (traits) that are implemented by outer layers.

**Contents:**
- Entities: Business objects with validation logic
- Value Objects: Immutable types representing domain concepts
- Enums: Domain-specific type definitions
- Error Types: Domain-specific errors implementing std::error::Error
- Repository Traits: Interface definitions for data access (ports)

**Example Entity (Clan):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Clan {
    id: Uuid,
    name: String,
    leader_id: Uuid,
    tier: ClanTier,
    total_score: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Clan {
    pub fn new(name: String, leader_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            leader_id,
            tier: ClanTier::default(),
            total_score: 0,
            created_at: chrono::Utc::now(),
        }
    }
    // Getters and domain methods
}
```

**Example Error Type:**
```rust
#[derive(Error, Debug)]
pub enum LeagueError {
    #[error("Clan not found: {0}")]
    ClanNotFound(String),
    #[error("Clan is full: {0}")]
    ClanIsFull(String),
    #[error("User already in a clan: {0}")]
    UserAlreadyInClan(String),
}
```

### 4.2 Application Layer (`application/`)

The orchestration layer containing use cases that coordinate domain objects and repository interfaces. This layer implements the application-specific business rules.

**Contents:**
- DTOs: Data Transfer Objects for input/output
- Use Cases: Service classes implementing business workflows
- Dependency Injection: Constructor injection of repository implementations

**Example Use Case:**
```rust
pub struct CreateClanUseCase<R: ClanRepository> {
    repo: R,
}

impl<R: ClanRepository> CreateClanUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, dto: CreateClanDto) -> Result<Clan, AppError> {
        // Validate user not already in clan
        if self.repo.is_user_in_any_clan(dto.leader_id).await? {
            return Err(AppError::BadRequest(
                "User is already in a clan".to_string(),
            ));
        }
        // Create clan entity
        let clan = Clan::new(dto.name, dto.leader_id);
        self.repo.create_clan(&clan).await?;
        // Add leader as first member
        let member = ClanMember::new(clan.id(), dto.leader_id, MemberRole::Leader);
        self.repo.add_member(&member).await?;
        Ok(clan)
    }
}
```

### 4.3 Infrastructure Layer (`infrastructure/`)

The outer layer containing implementations of domain interfaces. This layer depends on the domain layer and contains concrete technology-specific code.

**Contents:**
- Database Repositories: PostgreSQL implementations of repository traits
- Cache Repositories: Redis implementations of cache traits
- HTTP Clients: External service client implementations
- Mappers: Translation between domain entities and database rows

**Example Repository Implementation:**
```rust
pub struct ClanPostgresRepo {
    pool: PgPool,
}

impl ClanPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClanRepository for ClanPostgresRepo {
    async fn create_clan(&self, clan: &Clan) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan.id())
        .bind(clan.name())
        .bind(clan.leader_id())
        .bind(clan.tier().to_string())
        .bind(clan.total_score())
        .bind(clan.created_at())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }
    // Other repository methods...
}
```

### 4.4 Presentation Layer (`presentation/`)

The outermost layer handling HTTP requests and responses. Controllers extract data from requests, delegate to use cases, and format responses.

**Contents:**
- Controllers: Request handlers with Axum extractors
- Routes: URL path configuration and HTTP method binding
- Response DTOs: API response structures

**Example Controller:**
```rust
pub async fn create_clan_handler(
    State(state): State<AppState>,
    Json(dto): Json<CreateClanDto>,
) -> Result<(StatusCode, Json<ApiResponse<Clan>>), AppError> {
    let repo = ClanPostgresRepo::new(state.db);
    let use_case = CreateClanUseCase::new(repo);
    let clan = use_case.execute(dto).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Clan created successfully", clan)),
    ))
}
```

---

## 5. Layer Dependencies Diagram

```mermaid
graph TB
    subgraph Presentation["Presentation Layer<br/>(presentation/)"]
        Controllers["Controllers"]
        Routes["Routes"]
    end

    subgraph Application["Application Layer<br/>(application/)"]
        UseCases["Use Cases"]
        DTOs["DTOs"]
    end

    subgraph Domain["Domain Layer<br/>(domain/)"]
        Entities["Entities"]
        ValueObjects["Value Objects"]
        Enums["Enums"]
        ErrorTypes["Error Types"]
        RepositoryTraits["Repository Traits"]
    end

    subgraph Infrastructure["Infrastructure Layer<br/>(infrastructure/)"]
        PostgresRepos["PostgreSQL Repositories"]
        RedisRepos["Redis Repositories"]
        HttpClients["HTTP Clients"]
        Mappers["Mappers"]
    end

    Presentation --> Application
    Application --> Domain
    Infrastructure --> Domain
    Application --> Infrastructure

    Controllers --> UseCases
    Controllers --> DTOs
    UseCases --> RepositoryTraits
    UseCases --> Entities
    UseCases --> DTOs
    DTOs --> Entities
    PostgresRepos --> RepositoryTraits
    PostgresRepos --> Entities
    PostgresRepos --> Mappers
    RedisRepos --> RepositoryTraits
    HttpClients --> Entities
```

---

## 6. Hexagonal Architecture

Hexagonal Architecture (also known as Ports and Adapters) structures the application with the domain at its core, isolated from external concerns through well-defined ports.

### 6.1 Ports

Ports are interfaces defined by the domain layer that specify how the application can interact with the outside world.

**Driving Ports (Primary):**
- Use case interfaces invoked by presentation layer
- Defined in `application/` as service structs

**Driven Ports (Secondary):**
- Repository interfaces for data persistence
- Cache interfaces for caching operations
- HTTP client interfaces for external services
- Defined in `domain/repositories/`

### 6.2 Adapters

Adapters are implementations of the driven ports that connect to external systems.

**Primary Adapters:**
- Axum controllers and route handlers
- Located in `presentation/`

**Secondary Adapters:**
- `ClanPostgresRepo`: PostgreSQL persistence for clans
- `LeaderboardRedisRepo`: Redis caching for leaderboards
- `AchievementPostgresRepo`: PostgreSQL persistence for achievements
- `MissionPostgresRepo`: PostgreSQL persistence for missions
- `UserPostgresRepo`: PostgreSQL persistence for shadow users
- `HttpClient`: HTTP client for Java Core service

### 6.3 Hexagonal Architecture Diagram

```mermaid
graph LR
    subgraph External["External World"]
        Frontend["Next.js Frontend"]
        JavaBackend["Java Backend"]
        PostgreSQL["PostgreSQL"]
        Redis["Redis"]
    end

    subgraph PrimaryAdapters["Primary Adapters (Driving)"]
        ClanController["Clan Controller"]
        ScoreController["Score Controller"]
        UserSyncController["User Sync Controller"]
    end

    subgraph ApplicationLayer["Application Layer"]
        CreateClanUC["CreateClanUseCase"]
        JoinClanUC["JoinClanUseCase"]
        GetLeaderboardUC["GetLeaderboardUseCase"]
        SyncQuizGamUC["SyncQuizGamificationUseCase"]
        SyncNewUserUC["SyncNewUserUseCase"]
    end

    subgraph DomainCore["Domain Core"]
        Clan["Clan Entity"]
        ClanMember["ClanMember Entity"]
        Achievement["Achievement Entity"]
        ShadowUser["ShadowUser Entity"]
        ClanRepository["ClanRepository Trait"]
        LeaderboardCache["LeaderboardCache Trait"]
        AchievementRepository["AchievementRepository Trait"]
        MissionRepository["MissionRepository Trait"]
        UserRepository["UserRepository Trait"]
    end

    subgraph SecondaryAdapters["Secondary Adapters (Driven)"]
        ClanPostgresRepo["ClanPostgresRepo"]
        LeaderboardRedisRepo["LeaderboardRedisRepo"]
        AchievementPostgresRepo["AchievementPostgresRepo"]
        MissionPostgresRepo["MissionPostgresRepo"]
        UserPostgresRepo["UserPostgresRepo"]
        HttpClient["HttpClient"]
    end

    Frontend --> PrimaryAdapters
    JavaBackend --> PrimaryAdapters

    PrimaryAdapters --> ApplicationLayer
    ApplicationLayer --> DomainCore

    DomainCore --> SecondaryAdapters
    SecondaryAdapters --> PostgreSQL
    SecondaryAdapters --> Redis

    ClanPostgresRepo -.->|implements| ClanRepository
    LeaderboardRedisRepo -.->|implements| LeaderboardCache
    AchievementPostgresRepo -.->|implements| AchievementRepository
    MissionPostgresRepo -.->|implements| MissionRepository
    UserPostgresRepo -.->|implements| UserRepository
```

---

## 7. API Architecture

### 7.1 Route Structure

The API is organized into two main route groups: public API v1 and internal API.

```
/health                            GET  - Health check
/error                             GET  - Error simulation (dev)
/swagger-ui                        GET  - Swagger documentation
/api-docs/openapi.json             GET  - OpenAPI schema

/api/v1/clans                      POST - Create a new clan
/api/v1/clans/{id}                 GET  - Get clan details
/api/v1/clans/{id}/join            POST - Join a clan
/api/v1/leaderboards               GET  - Get leaderboard (query: tier)
/api/v1/users/{user_id}/tier       GET  - Get user's tier

/api/internal/users/sync           POST - Sync new user from Java (idempotent)
/api/internal/quiz-history/sync     POST - Sync quiz history and update score
```

### 7.2 Controller Overview

**ClanController** (`clan_controller.rs`):
- `create_clan_handler`: POST /api/v1/clans
- `get_clan_detail_handler`: GET /api/v1/clans/{id}
- `join_clan_handler`: POST /api/v1/clans/{id}/join
- `get_user_tier_handler`: GET /api/v1/users/{user_id}/tier

**ScoreController** (`score_controller.rs`):
- `get_leaderboard_handler`: GET /api/v1/leaderboards

**InternalUserController** (`internal_user_controller.rs`):
- `sync_user_handler`: POST /api/internal/users/sync

**QuizHistoryController** (`quiz_history_controller.rs`):
- `sync_quiz_history_handler`: POST /api/internal/quiz-history/sync

### 7.3 API Response Format

All responses follow a consistent JSON structure:

```json
{
  "success": true,
  "message": "Operation description",
  "data": { ... }
}
```

**Success Response Example:**
```json
{
  "success": true,
  "message": "Clan created successfully",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Dragon Slayers",
    "leader_id": "550e8400-e29b-41d4-a716-446655440001",
    "tier": "Bronze",
    "total_score": 0,
    "created_at": "2026-04-16T10:30:00Z"
  }
}
```

**Error Response Example:**
```json
{
  "success": false,
  "message": "Clan not found: 550e8400-e29b-41d4-a716-446655440000",
  "data": null
}
```

### 7.4 Route Registration

Routes are registered in `main.rs` using Axum's routing:

```rust
let api_v1_router = Router::new()
    .merge(modules::league::presentation::routes::league_routes());

let internal_api_router = Router::new()
    .nest("/users", modules::user_sync::presentation::routes::user_sync_routes());

let app = Router::new()
    .route("/health", get(health_check))
    .route("/error", get(simulate_error))
    .merge(swagger)
    .nest("/api/v1", api_v1_router)
    .nest("/api/internal", internal_api_router)
    .with_state(state)
    .layer(middleware_stack);
```

---

## 8. Database Architecture

### 8.1 PostgreSQL Schema

The system uses PostgreSQL 18 as the primary data store. All tables are created via SQL migrations in the `migrations/` directory.

**Core Tables:**

```sql
-- Users table (shadow users from Java)
CREATE TABLE engine_users (
    user_id UUID PRIMARY KEY,
    total_score INT NOT NULL DEFAULT 0
);

-- Clans table
CREATE TABLE clans (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    leader_id UUID NOT NULL REFERENCES engine_users(user_id),
    tier VARCHAR(50) NOT NULL,
    total_score INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Clan members join table
CREATE TABLE clan_members (
    clan_id UUID NOT NULL REFERENCES clans(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (clan_id, user_id)
);

-- Clan buffs/debuffs
CREATE TABLE clan_buffs (
    id UUID PRIMARY KEY,
    clan_id UUID NOT NULL REFERENCES clans(id) ON DELETE CASCADE,
    buff_name VARCHAR(255) NOT NULL,
    multiplier DECIMAL(5,2) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMPTZ NOT NULL
);

-- Achievements
CREATE TABLE achievements (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    milestone_target INT NOT NULL,
    achievement_type VARCHAR(100) NOT NULL,
    reward_points INT NOT NULL DEFAULT 0
);

-- User achievements (progress tracking)
CREATE TABLE user_achievements (
    user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
    achievement_id UUID NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    current_progress INT NOT NULL DEFAULT 0,
    is_completed BOOLEAN NOT NULL DEFAULT false,
    is_shown_on_profile BOOLEAN NOT NULL DEFAULT false,
    completed_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, achievement_id)
);

-- Daily missions
CREATE TABLE daily_missions (
    id UUID PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    target_count INT NOT NULL,
    date DATE NOT NULL,
    reward_points INT NOT NULL DEFAULT 0
);

-- User missions (progress tracking)
CREATE TABLE user_missions (
    user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
    mission_id UUID NOT NULL REFERENCES daily_missions(id) ON DELETE CASCADE,
    current_progress INT NOT NULL DEFAULT 0,
    is_claimed BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (user_id, mission_id)
);

-- Quiz history from Java
CREATE TABLE quiz_history (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
    article_id UUID NOT NULL,
    score INT NOT NULL DEFAULT 0,
    accuracy DECIMAL(5,2) NOT NULL DEFAULT 0.00,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 8.2 Redis Schema

Redis is used exclusively for leaderboard caching to provide fast read access to ranked clan data.

**Key Patterns:**

| Key Pattern | Type | Description |
|-------------|------|-------------|
| `leaderboard:global` | Sorted Set | All clans ranked by score |
| `leaderboard:Bronze` | Sorted Set | Bronze tier clans |
| `leaderboard:Silver` | Sorted Set | Silver tier clans |
| `leaderboard:Gold` | Sorted Set | Gold tier clans |
| `leaderboard:Diamond` | Sorted Set | Diamond tier clans |

**Operations:**
- `ZINCRBY leaderboard:global <score> <clan_id>`: Increment clan score
- `ZREVRANGE leaderboard:Bronze 0 9 WITHSCORES`: Get top 10 Bronze clans

### 8.3 Connection Pooling

**PostgreSQL Pool Configuration:**
```rust
PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(5))
    .test_before_acquire(true)
    .connect(database_url)
    .await?;
```

**Redis Connection:**
- Uses multiplexed connection for concurrent command support
- Single connection with async multiplexing via `get_multiplexed_async_connection()`

---

## 9. Entity Relationship Diagrams

### 9.1 League Module ER Diagram

```mermaid
erDiagram
    engine_users ||--o{ clans : "leads"
    engine_users ||--o{ clan_members : "belongs_to"
    clans ||--o{ clan_members : "has"
    clans ||--o{ clan_buffs : "has"

    engine_users {
        uuid user_id PK
        int total_score
    }

    clans {
        uuid id PK
        string name
        uuid leader_id FK
        string tier
        int total_score
        timestamptz created_at
    }

    clan_members {
        uuid clan_id FK
        uuid user_id FK
        timestamptz joined_at
    }

    clan_buffs {
        uuid id PK
        uuid clan_id FK
        string buff_name
        decimal multiplier
        boolean is_active
        timestamptz expires_at
    }
```

### 9.2 Gamification Module ER Diagram

```mermaid
erDiagram
    engine_users ||--o{ user_achievements : "progresses"
    achievements ||--o{ user_achievements : "tracks"
    engine_users ||--o{ user_missions : "progresses"
    daily_missions ||--o{ user_missions : "tracks"
    engine_users ||--o{ quiz_history : "completes"

    engine_users {
        uuid user_id PK
        int total_score
    }

    achievements {
        uuid id PK
        string name
        int milestone_target
        string achievement_type
        int reward_points
    }

    user_achievements {
        uuid user_id FK
        uuid achievement_id FK
        int current_progress
        boolean is_completed
        boolean is_shown_on_profile
        timestamptz completed_at
    }

    daily_missions {
        uuid id PK
        string description
        int target_count
        date date
        int reward_points
    }

    user_missions {
        uuid user_id FK
        uuid mission_id FK
        int current_progress
        boolean is_claimed
    }

    quiz_history {
        uuid id PK
        uuid user_id FK
        uuid article_id
        int score
        decimal accuracy
        timestamptz completed_at
    }
```

### 9.3 User Sync Module ER Diagram

```mermaid
erDiagram
    shadow_users ||--o{ quiz_history : "has"
    shadow_users {
        uuid user_id PK
        int total_score
        timestamptz created_at
    }
    quiz_history {
        uuid id PK
        uuid user_id FK
        uuid article_id
        int score
        decimal accuracy
        timestamptz completed_at
    }
```

Note: The User Sync module uses the `shadow_users` table for shadow users (synced from Java) and the `quiz_history` table for recording quiz attempts. The `shadow_users` table stores user_id, total_score (accumulated from quiz scores), and created_at.

---

## 10. Data Flow Diagrams

### 10.1 Create Clan Flow

```mermaid
sequenceDiagram
    participant Frontend
    participant Controller
    participant UseCase
    participant Repository
    participant Database

    Frontend->>Controller: POST /api/v1/clans<br/>CreateClanDto {name, leader_id}
    Controller->>UseCase: execute(CreateClanDto)
    UseCase->>Repository: is_user_in_any_clan(leader_id)
    Repository->>Database: SELECT COUNT(*) FROM clan_members<br/>WHERE user_id = $1
    Database-->>Repository: count
    Repository-->>UseCase: bool (is in clan)

    alt User already in clan
        UseCase-->>Controller: Err(AppError::BadRequest)
        Controller-->>Frontend: 400 Bad Request
    else User not in clan
        UseCase->>Repository: create_clan(Clan)
        Repository->>Database: INSERT INTO clans VALUES (...)
        Database-->>Repository: OK
        Repository-->>UseCase: Ok(())

        UseCase->>Repository: add_member(ClanMember)
        Repository->>Database: INSERT INTO clan_members VALUES (...)
        Database-->>Repository: OK
        Repository-->>UseCase: Ok(())

        UseCase-->>Controller: Ok(Clan)
        Controller-->>Frontend: 201 Created<br/>ApiResponse<Clan>
    end
```

### 10.2 Join Clan Flow

```mermaid
sequenceDiagram
    participant Frontend
    participant Controller
    participant UseCase
    participant Repository
    participant Database

    Frontend->>Controller: POST /api/v1/clans/{id}/join<br/>JoinClanDto {user_id}
    Controller->>UseCase: execute(JoinClanDto)
    UseCase->>Repository: get_clan_by_id(clan_id)
    Repository->>Database: SELECT * FROM clans WHERE id = $1
    Database-->>Repository: Option<Clan>
    Repository-->>UseCase: Option<Clan>

    alt Clan not found
        UseCase-->>Controller: Err(AppError::NotFound)
        Controller-->>Frontend: 404 Not Found
    end

    UseCase->>Repository: is_user_in_any_clan(user_id)
    Repository->>Database: SELECT COUNT(*) FROM clan_members<br/>WHERE user_id = $1
    Database-->>Repository: count
    Repository-->>UseCase: bool

    alt User already in clan
        UseCase-->>Controller: Err(AppError::BadRequest)
        Controller-->>Frontend: 400 Bad Request
    else User can join
        UseCase->>Repository: add_member(ClanMember)
        Repository->>Database: INSERT INTO clan_members VALUES (...)
        Database-->>Repository: OK
        Repository-->>UseCase: Ok(())

        UseCase-->>Controller: Ok(ClanMember)
        Controller-->>Frontend: 200 OK<br/>ApiResponse<ClanMember>
    end
```

### 10.3 Get Leaderboard Flow

```mermaid
sequenceDiagram
    participant Frontend
    participant Controller
    participant UseCase
    participant RedisRepo
    participant Redis

    Frontend->>Controller: GET /api/v1/leaderboards?tier=Bronze
    Controller->>UseCase: execute("Bronze")
    UseCase->>RedisRepo: get_top_clans("Bronze", 10)
    RedisRepo->>Redis: ZREVRANGE leaderboard:Bronze 0 9 WITHSCORES
    Redis-->>RedisRepo: Vec<(clan_id, score)>
    RedisRepo-->>UseCase: Vec<LeaderboardEntry>
    UseCase-->>Controller: Ok(LeaderboardDto)
    Controller-->>Frontend: 200 OK<br/>ApiResponse<LeaderboardDto>
```

### 10.4 Sync User Flow (Idempotent)

```mermaid
sequenceDiagram
    participant JavaBackend
    participant Controller
    participant UseCase
    participant Repository
    participant Database

    JavaBackend->>Controller: POST /api/internal/users/sync<br/>SyncUserRequestDto {user_id}
    Controller->>UseCase: execute(SyncUserRequestDto)
    UseCase->>Repository: exists_shadow_user(user_id)
    Repository->>Database: SELECT COUNT(*) FROM shadow_users<br/>WHERE user_id = $1
    Database-->>Repository: count
    Repository-->>UseCase: bool

    alt User already exists
        UseCase->>Repository: get_shadow_user(user_id)
        Repository->>Database: SELECT * FROM shadow_users<br/>WHERE user_id = $1
        Database-->>Repository: ShadowUser
        Repository-->>UseCase: Some(ShadowUser)
        UseCase-->>Controller: Ok(ShadowUser)
        Controller-->>JavaBackend: 201 Created<br/>ApiResponse<SyncUserResponseDto>
    else New user
        UseCase->>Repository: insert_shadow_user(ShadowUser)
        Repository->>Database: INSERT INTO shadow_users VALUES (...)
        Database-->>Repository: OK
        Repository-->>UseCase: Ok(())
        UseCase-->>Controller: Ok(ShadowUser)
        Controller-->>JavaBackend: 201 Created<br/>ApiResponse<SyncUserResponseDto>
    end
```

### 10.5 Sync Quiz History Flow

```mermaid
sequenceDiagram
    participant JavaBackend
    participant Controller
    participant UseCase
    participant UserRepo
    participant QuizRepo
    participant Database

    JavaBackend->>Controller: POST /api/internal/quiz-history/sync<br/>QuizHistoryRequestDto {user_id, article_id, score, accuracy}
    Controller->>UseCase: execute(QuizHistoryRequestDto)
    UseCase->>UserRepo: exists_shadow_user(user_id)
    UserRepo->>Database: SELECT COUNT(*) FROM shadow_users WHERE user_id = $1
    Database-->>UserRepo: bool
    UserRepo-->>UseCase: bool

    alt User not found
        UseCase-->>Controller: Err(UserSyncError::UserNotFound)
        Controller-->>JavaBackend: 404 Not Found
    else Validation failed
        alt score < 0 or accuracy not in [0.0, 100.0]
            UseCase-->>Controller: Err(UserSyncError::InvalidQuizData)
            Controller-->>JavaBackend: 400 Bad Request
        end
    else Valid quiz data and user exists
        UseCase->>QuizRepo: insert_quiz_history(QuizHistory)
        QuizRepo->>Database: INSERT INTO quiz_history VALUES (...)
        Database-->>QuizRepo: OK
        QuizRepo-->>UseCase: Ok(())

        UseCase->>UserRepo: update_total_score(user_id, score)
        UserRepo->>Database: UPDATE shadow_users SET total_score = total_score + $1 WHERE user_id = $2
        Database-->>UserRepo: OK
        UserRepo-->>UseCase: Ok(())

        UseCase-->>Controller: Ok(QuizHistoryResponseDto)
        Controller-->>JavaBackend: 201 Created<br/>ApiResponse<QuizHistoryApiResponse>
    end
```

---

## 11. Error Handling Architecture

### 11.1 Error Type Hierarchy

```mermaid
graph TB
    subgraph SharedErrors["Shared Error Types"]
        AppError["AppError<br/>(thiserror enum)"]
    end

    subgraph LeagueErrors["League-Specific Errors"]
        LeagueError["LeagueError<br/>(thiserror enum)"]
    end

    subgraph GamificationErrors["Gamification-Specific Errors"]
        GamificationError["GamificationError"]
    end

    subgraph UserSyncErrors["User Sync Errors"]
        UserSyncError["UserSyncError<br/>(thiserror enum)"]
    end

    subgraph HttpErrors["HTTP-Specific Errors"]
        HttpError["HttpError"]
    end

    AppError --> LeagueError
    AppError --> UserSyncError
```

### 11.2 AppError Enum

The central error type shared across all modules:

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    InternalServer(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
```

### 11.3 LeagueError Enum

Domain-specific errors for the league module:

```rust
#[derive(Error, Debug)]
pub enum LeagueError {
    #[error("Clan not found: {0}")]
    ClanNotFound(String),

    #[error("Clan is full: {0}")]
    ClanIsFull(String),

    #[error("User already in a clan: {0}")]
    UserAlreadyInClan(String),

    #[error("User not in any clan: {0}")]
    UserNotInAnyClan(String),

    #[error("Max clans reached: {0}")]
    MaxClansReached(String),
}
```

### 11.4 HTTP Status Code Mapping

| Error Type | HTTP Status | Condition |
|------------|-------------|-----------|
| `AppError::InternalServer` | 500 | Database failures, unexpected errors |
| `AppError::BadRequest` | 400 | Invalid input, validation failures |
| `AppError::NotFound` | 404 | Resource not found |
| `LeagueError::ClanNotFound` | 404 | Clan does not exist |
| `LeagueError::ClanIsFull` | 400 | Clan at capacity |
| `LeagueError::UserAlreadyInClan` | 400 | User already a member |
| `LeagueError::UserNotInAnyClan` | 400 | User has no clan |
| `LeagueError::MaxClansReached` | 400 | System clan limit |
| `UserSyncError::UserNotFound` | 404 | User not in Engine DB |
| `UserSyncError::InvalidQuizData` | 400 | Invalid score or accuracy |
| `UserSyncError::ValidationError` | 400 | Request validation failed |
| `UserSyncError::DatabaseError` | 500 | Database error |

### 11.5 UserSyncError Enum

Domain-specific errors for the user sync module:

```rust
#[derive(Error, Debug)]
pub enum UserSyncError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid quiz data: {0}")]
    InvalidQuizData(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}
```

### 11.6 Error Propagation

```mermaid
flowchart TB
    subgraph Infrastructure["Infrastructure Layer"]
        SQLError["sqlx::Error"]
        RedisError["redis::RedisError"]
    end

    subgraph Domain["Domain Layer"]
        AppError["AppError"]
    end

    subgraph UseCase["Use Case Layer"]
        BusinessRule["Business Rule Violation"]
    end

    subgraph Presentation["Presentation Layer"]
        Handler["Controller Handler"]
    end

    SQLError -->|map_err| AppError::InternalServer
    RedisError -->|map_err| AppError::InternalServer
    BusinessRule -->|Err| AppError
    AppError -->|IntoResponse| Handler
```

---

## 12. CI/CD Architecture

### 12.1 GitHub Actions Workflow

The CI pipeline runs on every push to any branch and consists of multiple parallel jobs.

```mermaid
flowchart TB
    subgraph Trigger["On Push to Any Branch"]
    end

    Trigger --> FmtCheck["Format Check<br/>cargo fmt --all -- --check"]
    Trigger --> ClippyCheck["Clippy Lint<br/>cargo clippy"]
    Trigger --> DocCheck["Documentation<br/>cargo doc"]
    Trigger --> TestSuite["Test Suite"]

    Trigger --> DockerBuild["Docker Build<br/>(after all pass)"]

    TestSuite --> UnitTests["Unit Tests<br/>cargo test --lib"]
    TestSuite --> BinaryTests["Binary Tests<br/>cargo test --bin"]
    TestSuite --> IntegrationTests["Integration Tests"]
    TestSuite --> OfflineCheck["sqlx prepare check"]

    IntegrationTests --> PostgresService["PostgreSQL Service"]
    IntegrationTests --> RedisService["Redis Service"]

    FmtCheck -.-> DockerBuild
    ClippyCheck -.-> DockerBuild
    DocCheck -.-> DockerBuild
    TestSuite -.-> DockerBuild

    DockerBuild --> BuildImage["Build Docker Image<br/>docker build"]
```

### 12.2 CI Pipeline Steps

**1. Format Check (fmt):**
```yaml
- name: Check formatting
  run: cargo fmt --all -- --check
```

**2. Clippy Lint (clippy):**
```yaml
- name: Run Clippy
  run: cargo clippy --all --all-targets -- -W warnings
```

**3. Documentation (doc):**
```yaml
- name: Build documentation
  run: cargo doc --all --no-deps
  env:
    RUSTDOCFLAGS: -D warnings
```

**4. Test Suite:**
```yaml
# PostgreSQL and Redis services started
- name: Run library unit tests
  run: cargo test --lib
- name: Run binary unit tests
  run: cargo test --bin yomu-backend-rust
- name: Run infrastructure tests
  run: cargo test --test infrastructure_test
- name: Run presentation tests
  run: cargo test --test presentation_test
- name: Run use case tests
  run: cargo test --test usecase_test
- name: Check sqlx prepare
  run: cargo sqlx prepare --workspace --check
```

**5. Docker Build:**
```yaml
- name: Build Docker image
  uses: docker/build-push-action@v5
  with:
    context: .
    push: false
    tags: yomu-engine:latest
    cache-from: type=gha
    cache-to: type=gha,mode=max
```

---

## 13. Deployment Architecture

### 13.1 Docker Compose Services

```mermaid
graph TB
    subgraph DockerNetwork["yomu-network"]
        subgraph PostgresContainer["postgres"]
            PostgresImg["postgres:18-alpine"]
        end

        subgraph RedisContainer["redis"]
            RedisImg["redis:8-alpine"]
        end

        subgraph AppContainer["app"]
            AppImg["yomu-engine:latest"]
        end

        subgraph DevContainer["dev"]
            DevImg["yomu-engine-dev:latest"]
        end
    end

    PostgresImg <-->|port 5432| AppContainer
    RedisImg <-->|port 6379| AppContainer
    RedisImg <-->|port 6379| DevContainer
    AppContainer <-->|port 8080| External["External"]
    DevContainer <-->|port 8080| External
```

### 13.2 Container Configuration

**PostgreSQL Container:**
```yaml
postgres:
  image: postgres:18-alpine
  ports:
    - "5432:5432"
  volumes:
    - postgres_data:/var/lib/postgresql/data
  healthcheck:
    test: ["CMD-SHELL", "pg_isready -U yomu"]
    interval: 10s
    timeout: 5s
    retries: 10
```

**Redis Container:**
```yaml
redis:
  image: redis:8-alpine
  ports:
    - "6379:6379"
  volumes:
    - redis_data:/data
  command: redis-server --appendonly yes
```

**Application Container:**
```yaml
app:
  build:
    context: .
    dockerfile: Dockerfile
  ports:
    - "8080:8080"
  environment:
    - DATABASE_URL=postgresql://yomu:yomu_password@postgres:5432/yomu_engine
    - REDIS_URL=redis://redis:6379
    - RUST_LOG=info
  depends_on:
    postgres:
      condition: service_healthy
    redis:
      condition: service_healthy
```

### 13.3 Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `APP_HOST` | Bind address | `0.0.0.0` |
| `APP_PORT` | Bind port | `8080` |
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `REDIS_URL` | Redis connection string | Required |
| `JAVA_CORE_URL` | Java Core service URL | Required |
| `JAVA_CORE_API_KEY` | API key for Java Core | Required |
| `RUST_LOG` | Tracing log level | `info` |
| `RUST_BACKTRACE` | Enable backtrace | `1` |

---

## 14. Technology Decisions

### 14.1 Why Axum?

**Rationale:**
- **Async First**: Built on tokio and tower for true async request handling
- **Type Safety**: Leverages Rust's type system with compile-time guarantees
- **Minimal Overhead**: Low-level control without unnecessary abstractions
- **Tower Integration**: Seamless middleware composition via tower layer pattern
- **Active Ecosystem**: Well-maintained with strong community support

**Alternative Considered:** Actix-web
- Actix is more mature but has steeper learning curve
- Axum's design aligns better with Tower's middleware philosophy
- Axum's extractor pattern is more ergonomic for our use cases

### 14.2 Why SQLx?

**Rationale:**
- **Compile-Time Safety**: SQL queries checked against database schema at compile time
- **No ORM Overhead**: Direct SQL control without object-relational mapping complexity
- **Async Native**: Full async support with connection pooling
- **Offline Mode**: Can verify queries without database connection via `cargo sqlx prepare`
- **Type Mapping**: Automatic conversion between PostgreSQL types and Rust types

**Alternative Considered:** Diesel
- Diesel requires more boilerplate for similar functionality
- SQLx's async support is more mature
- Diesel compile times are significantly longer

### 14.3 Why Redis?

**Rationale:**
- **Leaderboard Operations**: Native sorted set operations (ZINCRBY, ZREVRANGE) perfect for rankings
- **In-Memory Speed**: Sub-millisecond read latency for frequent leaderboard queries
- **Atomic Operations**: Redis commands are atomic, preventing race conditions
- **Persistence**: Optional AOF persistence for durability
- **Mature Client**: redis-rs provides excellent async support

**Alternative Considered:** PostgreSQL-only
- Window functions can implement leaderboards but are slower at scale
- Redis offloads read-heavy operations from PostgreSQL
- Separation of concerns: PostgreSQL for consistency, Redis for speed

### 14.4 Why Rust 2024 Edition?

**Rationale:**
- **Async Traits**: Better support for async fn in traits
- **Precise Capture**: More control over captures in closures
- **RPITIT**: Return position impl Trait in traits
- **MSRV 1.85**: Allows use of latest Rust features while maintaining compatibility

### 14.5 Why async-trait?

**Rationale:**
- Object-safe async trait support required for dynamic dispatch
- Repository traits need to be behind `dyn Trait` for dependency injection
- Enables mockall for testing

---

## 15. Design Patterns

### 15.1 Repository Pattern

The repository pattern abstracts data access behind traits, enabling testability and loose coupling.

**Trait Definition:**
```rust
#[async_trait]
pub trait ClanRepository: Send + Sync {
    async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>;
    async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError>;
    async fn add_member(&self, member: &ClanMember) -> Result<(), AppError>;
    async fn get_members_by_clan_id(&self, clan_id: Uuid) -> Result<Vec<ClanMember>, AppError>;
    async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError>;
    async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;
    async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
}
```

**Implementation:**
```rust
pub struct ClanPostgresRepo {
    pool: PgPool,
}

#[async_trait]
impl ClanRepository for ClanPostgresRepo {
    async fn create_clan(&self, clan: &Clan) -> Result<(), AppError> {
        // SQL implementation...
    }
}
```

### 15.2 Use Case Pattern

Use cases encapsulate application-specific business logic and orchestrate domain objects.

```rust
pub struct CreateClanUseCase<R: ClanRepository> {
    repo: R,
}

impl<R: ClanRepository> CreateClanUseCase<R> {
    pub async fn execute(&self, dto: CreateClanDto) -> Result<Clan, AppError> {
        // Business logic...
    }
}
```

### 15.3 Result Type Pattern

All functions return `Result<T, E>` for explicit error handling.

```rust
// Domain returns specific error
pub async fn execute(&self, dto: CreateClanDto) -> Result<Clan, AppError>

// Repository returns domain error
async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>

// Infrastructure maps technical errors
.map_err(|e| AppError::InternalServer(e.to_string()))
```

### 15.4 Dependency Injection

Dependencies are injected via constructors for testability.

```rust
// In controller
pub async fn create_clan_handler(
    State(state): State<AppState>,
    Json(dto): Json<CreateClanDto>,
) -> Result<(StatusCode, Json<ApiResponse<Clan>>), AppError> {
    let repo = ClanPostgresRepo::new(state.db);
    let use_case = CreateClanUseCase::new(repo);
    // Use case instantiated per request
}
```

### 15.5 Factory Pattern

Entity creation is encapsulated in factory methods.

```rust
impl Clan {
    pub fn new(name: String, leader_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            leader_id,
            tier: ClanTier::default(),
            total_score: 0,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn with_id(/* all fields */) -> Self { /* for database reconstruction */ }
}
```

---

## 16. Code Organization

### 16.1 Directory Structure

```
yomu-backend-rust/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── lib.rs                     # Library root, exports modules
│   │
│   ├── config/
│   │   ├── mod.rs                 # Configuration structs
│   │   └── database.rs            # Database connection initialization
│   │
│   ├── modules/
│   │   ├── mod.rs                 # Module exports
│   │   │
│   │   ├── league/                # League Module (Clans, Leaderboards)
│   │   │   ├── mod.rs
│   │   │   │
│   │   │   ├── domain/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── entities/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── clan.rs
│   │   │   │   │   ├── clan_member.rs
│   │   │   │   │   └── score.rs
│   │   │   │   ├── errors/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── league_error.rs
│   │   │   │   └── repositories/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── clan_repository.rs
│   │   │   │       └── leaderboard_cache.rs
│   │   │   │
│   │   │   ├── application/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── dto/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── create_clan_dto.rs
│   │   │   │   │   ├── join_clan_dto.rs
│   │   │   │   │   ├── leaderboard_dto.rs
│   │   │   │   │   ├── clan_detail_dto.rs
│   │   │   │   │   ├── user_tier_dto.rs
│   │   │   │   │   └── update_score_dto.rs
│   │   │   │   └── use_cases/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── clan/
│   │   │   │       │   ├── mod.rs
│   │   │   │       │   ├── create_clan_usecase.rs
│   │   │   │       │   ├── join_clan_usecase.rs
│   │   │   │       │   └── get_clan_detail_usecase.rs
│   │   │   │       ├── score/
│   │   │   │       │   ├── mod.rs
│   │   │   │       │   ├── get_leaderboard_usecase.rs
│   │   │   │       │   └── calculate_score_usecase.rs
│   │   │   │       └── user/
│   │   │   │           ├── mod.rs
│   │   │   │           └── get_user_tier_usecase.rs
│   │   │   │
│   │   │   ├── infrastructure/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── database/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── postgres/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── clan_postgres_repo.rs
│   │   │   │   │   │   └── mappers/
│   │   │   │   │   │       ├── mod.rs
│   │   │   │   │   │       └── clan_mapper.rs
│   │   │   │   │   └── redis/
│   │   │   │   │       ├── mod.rs
│   │   │   │   │       └── leaderboard_redis_repo.rs
│   │   │   │   └── http/
│   │   │   │       ├── mod.rs
│   │   │   │       └── java_core_client.rs
│   │   │   │
│   │   │   └── presentation/
│   │   │       ├── mod.rs
│   │   │       ├── routes.rs
│   │   │       └── controllers/
│   │   │           ├── mod.rs
│   │   │           ├── clan_controller.rs
│   │   │           └── score_controller.rs
│   │   │
│   │   ├── gamification/          # Gamification Module (Achievements, Missions)
│   │   │   ├── mod.rs
│   │   │   ├── domain/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── entities/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── achievement.rs
│   │   │   │   │   ├── daily_mission.rs
│   │   │   │   │   ├── user_achievement.rs
│   │   │   │   │   └── user_mission.rs
│   │   │   │   ├── errors/
│   │   │   │   │   └── mod.rs
│   │   │   │   └── repositories/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── achievement_repository.rs
│   │   │   │       └── mission_repository.rs
│   │   │   ├── application/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── dto/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── quiz_sync.rs
│   │   │   │   └── use_cases/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── sync_quiz_gamification.rs
│   │   │   │       └── claim_mission_reward.rs
│   │   │   ├── infrastructure/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── database/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── postgres/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── achievement_repository.rs
│   │   │   │   │   │   ├── mission_repository.rs
│   │   │   │   │   │   └── mappers/
│   │   │   │   │   │       └── mod.rs
│   │   │   │   │   └── redis/
│   │   │   │   │       └── mod.rs
│   │   │   │   └── http/
│   │   │   │       └── mod.rs
│   │   │   └── presentation/
│   │   │       ├── mod.rs
│   │   │       ├── controllers/
│   │   │       │   └── mod.rs
│   │   │       └── routes.rs
│   │   │
│   │   └── user_sync/             # User Sync Module (Shadow Users)
│   │       ├── mod.rs
│   │       ├── domain/
│   │       │   ├── mod.rs
│   │       │   ├── entities/
│   │       │   │   ├── mod.rs
│   │       │   │   └── shadow_user.rs
│   │       │   ├── errors/
│   │       │   │   ├── mod.rs
│   │       │   │   └── user_sync_error.rs
│   │       │   └── repositories/
│   │       │       ├── mod.rs
│   │       │       └── user_repository.rs
│   │       ├── application/
│   │       │   ├── mod.rs
│   │       │   ├── dto/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── sync_user_dto.rs
│   │       │   │   ├── quiz_history_dto.rs
│   │       │   │   └── sync_user_response_dto.rs
│   │       │   └── use_cases/
│   │       │       ├── mod.rs
│   │       │       └── sync_new_user_usecase.rs
│   │       ├── infrastructure/
│   │       │   ├── mod.rs
│   │       │   ├── database/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── postgres/
│   │       │   │   │   ├── mod.rs
│   │       │   │   │   ├── user_postgres_repo.rs
│   │       │   │   │   └── mappers/
│   │       │   │   │       ├── mod.rs
│   │       │   │   │       └── user_mapper.rs
│   │       │   │   └── postgres.rs
│   │       │   └── database/postgres.rs
│   │       └── presentation/
│   │           ├── mod.rs
│   │           ├── routes.rs
│   │           └── controllers/
│   │               ├── mod.rs
│   │               └── internal_user_controller.rs
│   │
│   └── shared/
│       ├── mod.rs
│       ├── domain/
│       │   ├── mod.rs
│       │   └── base_error.rs
│       ├── utils/
│       │   ├── mod.rs
│       │   └── response.rs
│       └── infrastructure/
│           ├── mod.rs
│           ├── http/
│           │   ├── mod.rs
│           │   └── client.rs
│           └── http.rs
│
├── tests/
│   └── modules/
│       ├── league/
│       │   ├── domain_test.rs
│       │   ├── usecase_test.rs
│       │   ├── infrastructure_test.rs
│       │   └── presentation_test.rs
│       ├── gamification/
│       │   ├── domain_test.rs
│       │   ├── usecase_test.rs
│       │   └── infrastructure_test.rs
│       └── user_sync/
│           ├── domain_test.rs
│           ├── usecase_test.rs
│           ├── infrastructure_test.rs
│           └── presentation_test.rs
│
├── migrations/
│   ├── 20260224122751_first_migrations.sql
│   └── 20260416022505_add_reward_points.sql
│
├── docker-compose.yml
├── Dockerfile
├── Dockerfile.dev
├── Cargo.toml
├── rustfmt.toml
├── .clippy.toml
└── .github/
    └── workflows/
        ├── ci.yml
        └── sonar.yml
```

### 16.2 Module Organization Rationale

**Layer Separation:**
- Each module has its own `domain/`, `application/`, `infrastructure/`, `presentation/` structure
- This keeps related code together and enforces architectural boundaries

**Entity Colocation:**
- All entity-related code (definitions, tests) lives in `domain/entities/`
- DTOs live in `application/dto/` (input/output concerns)
- Mappers live in `infrastructure/database/*/mappers/` (persistence concerns)

**Test Organization:**
- Tests mirror source structure under `tests/modules/`
- Integration tests require Docker services
- Unit tests run with mocked dependencies

---

## 17. Key Interfaces

### 17.1 ClanRepository Trait

```rust
use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ClanRepository: Send + Sync {
    async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>;
    async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError>;
    async fn add_member(&self, member: &ClanMember) -> Result<(), AppError>;
    async fn get_members_by_clan_id(&self, clan_id: Uuid) -> Result<Vec<ClanMember>, AppError>;
    async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError>;
    async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;
    async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
}
```

### 17.2 LeaderboardCache Trait

```rust
use crate::modules::league::application::dto::LeaderboardEntry;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait LeaderboardCache: Send + Sync {
    async fn update_clan_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
    async fn get_top_clans(
        &self,
        tier: &str,
        limit: usize,
    ) -> Result<Vec<LeaderboardEntry>, AppError>;
}
```

### 17.3 AchievementRepository Trait

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::gamification::domain::entities::achievement::Achievement;
use crate::modules::gamification::domain::entities::user_achievement::UserAchievement;

#[async_trait]
pub trait AchievementRepository: Send + Sync {
    async fn get_achievement_by_id(&self, id: Uuid) -> Result<Option<Achievement>, String>;
    async fn get_user_achievements(&self, user_id: Uuid) -> Result<Vec<UserAchievement>, String>;
    async fn save_user_achievement(&self, user_achievement: &UserAchievement)
        -> Result<(), String>;
    async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String>;
}
```

### 17.4 MissionRepository Trait

```rust
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::modules::gamification::domain::entities::daily_mission::DailyMission;
use crate::modules::gamification::domain::entities::user_mission::UserMission;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MissionRepository: Send + Sync {
    async fn get_active_missions_by_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<DailyMission>, String>;
    async fn get_user_mission(
        &self,
        user_id: Uuid,
        mission_id: Uuid,
    ) -> Result<Option<UserMission>, String>;
    async fn save_user_mission(&self, user_mission: &UserMission) -> Result<(), String>;
    async fn get_daily_mission_by_id(&self, id: Uuid) -> Result<Option<DailyMission>, String>;
    async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String>;
}
```

### 17.5 UserRepository Trait

```rust
use super::super::entities::shadow_user::ShadowUser;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_shadow_user(&self, user: &ShadowUser) -> Result<(), AppError>;
    async fn exists_shadow_user(&self, user_id: Uuid) -> Result<bool, AppError>;
    async fn check_exists(&self, user_id: Uuid) -> bool;
}
```

---

## 18. Configuration Management

### 18.1 Configuration Structure

```rust
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub java_core_url: String,
    pub java_core_api_key: String,
}
```

### 18.2 Environment Variable Loading

```rust
impl AppConfig {
    pub fn load() -> Self {
        let _ = dotenvy::dotenv(); // Load .env file if present

        Self {
            host: get_env("APP_HOST", "0.0.0.0"),
            port: get_env("APP_PORT", "8080")
                .parse()
                .unwrap_or_else(|_| panic!("APP_PORT must be a number")),
            database_url: get_env_strict("DATABASE_URL"),
            redis_url: get_env_strict("REDIS_URL"),
            java_core_url: get_env_strict("JAVA_CORE_URL"),
            java_core_api_key: get_env_strict("JAVA_CORE_API_KEY"),
        }
    }
}

fn get_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_env_strict(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Missing required environment variable: {}", key))
}
```

### 18.3 Startup Initialization

```rust
#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()...)
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let app_config = config::AppConfig::load();

    // Initialize PostgreSQL pool
    let db_pool = match config::database::init_postgres_pool(&app_config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed connecting to database: {}", e);
            std::process::exit(1);
        }
    };

    // Run migrations
    if let Err(e) = sqlx::migrate!().run(&db_pool).await {
        tracing::error!("Error while doing database migrations: {}", e);
        std::process::exit(1);
    }

    // Initialize Redis pool
    let redis_pool = match config::database::init_redis_pool(&app_config.redis_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed connecting to Redis: {}", e);
            std::process::exit(1);
        }
    };

    // Create application state
    let state = AppState {
        db: db_pool,
        redis: redis_pool,
    };

    // Build and run router...
}
```

### 18.4 Environment Variable Summary

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `APP_HOST` | No | `0.0.0.0` | Server bind address |
| `APP_PORT` | No | `8080` | Server bind port |
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `REDIS_URL` | Yes | - | Redis connection string |
| `JAVA_CORE_URL` | Yes | - | Java Core service URL |
| `JAVA_CORE_API_KEY` | Yes | - | API key for Java Core |

---

## 19. Security Considerations

### 19.1 CORS Configuration

The application uses tower-http CORS middleware with permissive settings for development:

```rust
.layer(
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any),
);
```

**Production Note:** In production, `allow_origin` should be restricted to known frontend domains.

### 19.2 Input Validation

**DTO Validation:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateClanDto {
    pub name: String,
    pub leader_id: Uuid,
}
```

**Validator crate** is available for declarative validation:
```rust
use validator::Validate;

#[derive(Validate)]
pub struct CreateClanDto {
    #[validate(length(min = 3, max = 50))]
    pub name: String,
    pub leader_id: Uuid,
}
```

**Entity Validation:**
```rust
impl Achievement {
    pub fn update_details(
        &mut self,
        new_name: String,
        new_target: i32,
        new_reward: i32,
    ) -> Result<(), &'static str> {
        if new_name.trim().is_empty() {
            return Err("Nama achievement tidak boleh kosong.");
        }
        if new_target <= 0 {
            return Err("Target milestone harus lebih dari 0.");
        }
        if new_reward < 0 {
            return Err("Poin reward tidak boleh bernilai negatif.");
        }
        // ...
    }
}
```

### 19.3 API Key Protection

Internal endpoints are protected by API key validation via custom middleware:

```rust
// HttpClient sends API key header
.header("x-api-key", &self.api_key)
```

### 19.4 SQL Injection Prevention

SQLx uses parameterized queries automatically:
```rust
sqlx::query("SELECT * FROM clans WHERE id = $1")
    .bind(clan_id)
```

### 19.5 Rate Limiting (Future)

Rate limiting is not currently implemented but is recommended for production:
- Token bucket algorithm per IP
- Endpoint-specific limits for expensive operations

---

## 20. Performance Considerations

### 20.1 Connection Pooling

**PostgreSQL Pool:**
- Max connections: 20
- Min connections: 5
- Acquire timeout: 5 seconds
- Test before acquire: true

**Redis:**
- Multiplexed connection for concurrent commands
- Single connection with async dispatch

### 20.2 Caching Strategy

**Leaderboard Caching:**
- Redis sorted sets for O(log N) insertions and O(log N + M) range queries
- Write-through caching: PostgreSQL updates trigger Redis updates
- Cache miss: Query PostgreSQL, populate Redis

### 20.3 Async Runtime

**Tokio Runtime Configuration:**
```toml
tokio = { version = "1.49.0", features = ["full"] }
```

This enables all tokio features including:
- Multi-threaded runtime for parallel task execution
- Async file I/O
- Time utilities

### 20.4 Query Optimization

**Efficient Queries:**
```rust
// Batch member retrieval
let rows = sqlx::query(
    "SELECT clan_id, user_id, joined_at FROM clan_members WHERE clan_id = $1"
)
.bind(clan_id)
.fetch_all(&self.pool)
.await?;

// Single UPDATE with atomic increment
sqlx::query("UPDATE clans SET total_score = total_score + $1 WHERE id = $2")
    .bind(score)
    .bind(clan_id)
    .execute(&self.pool)
    .await?;
```

### 20.5 Serialization

**JSON Performance:**
- serde with derive macros for zero-cost serialization
- `#[serde(skip_serializing_if = "Vec::is_empty")]` to reduce payload size

### 20.6 Middleware Stack

```rust
let middleware_stack = ServiceBuilder::new()
    .layer(TraceLayer::new_for_http())        // Request tracing
    .layer(TimeoutLayer::with_status_code(    // 10s timeout
        StatusCode::REQUEST_TIMEOUT,
        Duration::from_secs(10),
    ))
    .layer(CorsLayer::new())                  // CORS handling
    .service(Router::new())
    .with_state(state);
```

---

## 21. Testing Strategy

### 21.1 Test Organization

```
tests/
├── modules/
│   ├── league/
│   │   ├── domain_test.rs       // Entity unit tests
│   │   ├── usecase_test.rs      // Use case logic tests
│   │   ├── infrastructure_test.rs // Repository tests
│   │   └── presentation_test.rs // Controller tests
│   ├── gamification/
│   │   └── ...
│   └── user_sync/
│       └── ...
```

### 21.2 Test Types

**Domain Tests:**
- Entity creation and validation
- Value object behavior
- Business rule enforcement
- Error condition handling

**Use Case Tests:**
- Mock repository implementations
- Happy path and error paths
- Business logic verification
- Mockall for trait mocking

**Infrastructure Tests:**
- Database repository integration
- Redis cache operations
- SQLx query verification

**Presentation Tests:**
- HTTP endpoint behavior
- Request/response serialization
- Route configuration

### 21.3 Mock Repository Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::user_sync::application::dto::SyncUserRequestDto;
    use crate::modules::user_sync::domain::entities::ShadowUser;
    use crate::shared::domain::base_error::AppError;
    use async_trait::async_trait;
    use uuid::Uuid;

    struct MockUserRepository {
        existing_users: std::sync::Mutex<std::collections::HashSet<Uuid>>,
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn insert_shadow_user(&self, user: &ShadowUser) -> Result<(), AppError> {
            Ok(())
        }

        async fn exists_shadow_user(&self, user_id: Uuid) -> Result<bool, AppError> {
            Ok(self.existing_users.lock().unwrap().contains(&user_id))
        }

        async fn check_exists(&self, user_id: Uuid) -> bool {
            self.existing_users.lock().unwrap().contains(&user_id)
        }
    }

    #[tokio::test]
    async fn sync_user_success() {
        let user_id = Uuid::new_v4();
        let repo = MockUserRepository::new();
        let use_case = SyncNewUserUseCase::new(repo);

        let dto = SyncUserRequestDto { user_id };
        let result = use_case.execute(dto).await;

        assert!(result.is_ok());
    }
}
```

### 21.4 Test Commands

```bash
# Run all tests
cargo test --all

# Run library tests only
cargo test --lib

# Run specific test file
cargo test --test usecase_test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test sync_user_success
```

### 21.5 SQLx Offline Mode

Verify queries compile without database:
```bash
cargo sqlx prepare --workspace --check
```

---

## 22. Future Considerations

### 22.1 Scalability

**Horizontal Scaling:**
- Deploy multiple instances behind load balancer
- Stateless design allows easy replication
- PostgreSQL connection pooling per instance

**Read Replicas:**
- PostgreSQL streaming replication for read-heavy workloads
- Separate Redis clusters for leaderboard sharding by tier

**Caching Improvements:**
- Redis cluster mode for high availability
- Cache invalidation strategies for leaderboards
- Session caching for user data

### 22.2 Feature Roadmap

**Clan Features:**
- Clan chat/announcements
- Clan wars/tournaments
- Tier promotion/demotion system
- Clan customization (emblem, description)

**Gamification Features:**
- Achievement categories and badges
- Seasonal missions and events
- Social sharing of achievements
- Leaderboard timeframes (daily, weekly, monthly)

**Social Features:**
- Friend system
- User profiles with stats
- Activity feeds

### 22.3 Architecture Improvements

**Event Sourcing:**
- Store all state changes as events
- Enables audit trail and replay
- CQRS for read/write separation

**Message Queue:**
- Replace polling with message queue (Kafka, RabbitMQ)
- Eventual consistency for cross-service communication
- Better handling of burst traffic

**GraphQL API:**
- Flexible querying for frontend
- Reduced over-fetching
- Real-time subscriptions

### 22.4 Operational Improvements

**Observability:**
- Distributed tracing (Jaeger, Zipkin)
- Structured logging with correlation IDs
- Custom metrics (Prometheus)

**Reliability:**
- Circuit breaker for external services
- Retry policies with exponential backoff
- Health check endpoint enhancement

**Security:**
- JWT authentication
- Rate limiting per user/IP
- Input sanitization
- SQL injection detection

### 22.5 Performance Optimization

**Database:**
- Index optimization for frequent queries
- Partitioning for large tables (quiz_history)
- Query result caching

**Application:**
- Response compression (gzip)
- HTTP/2 support
- Connection keep-alive

---

## Appendix A: API Reference Summary

### Public API Endpoints

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| GET | /health | Health check | None |
| POST | /api/v1/clans | Create clan | User ID |
| GET | /api/v1/clans/{id} | Get clan details | User ID |
| POST | /api/v1/clans/{id}/join | Join clan | User ID |
| GET | /api/v1/leaderboards | Get leaderboard | None |
| GET | /api/v1/users/{user_id}/tier | Get user tier | None |

### Internal API Endpoints

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| POST | /api/internal/users/sync | Sync shadow user | API Key |

---

## Appendix B: Database Schema Reference

### Indexes

```sql
-- Performance indexes
CREATE INDEX idx_clan_members_user_id ON clan_members(user_id);
CREATE INDEX idx_clan_members_clan_id ON clan_members(clan_id);
CREATE INDEX idx_engine_users_total_score ON engine_users(total_score DESC);
CREATE INDEX idx_achievements_type ON achievements(achievement_type);
CREATE INDEX idx_daily_missions_date ON daily_missions(date);
CREATE INDEX idx_quiz_history_user_id ON quiz_history(user_id);
CREATE INDEX idx_quiz_history_completed_at ON quiz_history(completed_at);
```

---

## Appendix C: Glossary

| Term | Definition |
|------|------------|
| Bounded Context | A domain-driven design concept defining clear boundaries within which domain models exist |
| Clean Architecture | An architectural pattern that separates code into layers with strict dependency rules |
| Driven Port | An interface defined by the domain that is implemented by infrastructure (secondary adapter) |
| Driving Port | An interface defined by the domain that is called by presentation (primary adapter) |
| Hexagonal Architecture | An architectural pattern using ports and adapters to isolate domain logic from external concerns |
| Shadow User | A minimal user representation in Rust synced from the Java backend |
| Source of Truth | The authoritative data store for a given entity |
| Use Case | An application service that orchestrates domain objects to fulfill a business request |

---

*This document is the definitive architecture reference for the Yomu Backend Rust project. For implementation details, refer to module-specific documentation and source code comments.*
