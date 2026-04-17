# Yomu Backend Rust

## Version 0.1.0

High-performance gamification engine API built with Rust, Axum, PostgreSQL, and Redis. Part of the Yomu polyglot learning platform.

---

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Technology Stack](#technology-stack)
4. [Architecture Diagram](#architecture-diagram)
5. [Prerequisites](#prerequisites)
6. [Installation](#installation)
7. [Configuration](#configuration)
8. [Database Setup](#database-setup)
9. [Running the Application](#running-the-application)
10. [Docker Compose](#docker-compose)
11. [API Documentation](#api-documentation)
12. [API Response Format](#api-response-format)
13. [Testing](#testing)
14. [Development Workflow](#development-workflow)
15. [Code Quality](#code-quality)
16. [Database Migrations](#database-migrations)
17. [Troubleshooting](#troubleshooting)
18. [Deployment](#deployment)
19. [Monitoring](#monitoring)
20. [Security](#security)
21. [CI/CD](#cicd)
22. [Contributing](#contributing)
23. [License](#license)
24. [Links](#links)

---

## Overview

Yomu Backend Rust is the gamification microservice of the Yomu learning platform. It handles clan management, leaderboards, achievements, missions, and user synchronization with the Java-based core service.

The service exposes REST APIs consumed by the Next.js frontend through a BFF (Backend for Frontend) proxy pattern. User data flows from the Java backend via an outbox synchronization pattern.

### Key Responsibilities

- **Clan Management**: Create clans, join clans, view clan details
- **Leaderboards**: Real-time clan rankings by tier (Bronze, Silver, Gold, Diamond)
- **Gamification**: Achievements, daily missions, reward points
- **User Sync**: Shadow users synced from Java backend for gamification tracking

---

## Features

### League Module

- Create new clans with a leader
- Join existing clans
- View clan details (members, active buffs/debuffs, tier)
- Real-time leaderboards by tier
- User tier information retrieval

### Gamification Module

- Achievement tracking with milestone targets
- Daily missions with progress tracking
- Reward points system
- Quiz history synchronization

### User Sync Module

- Shadow user creation from Java backend
- Quiz history synchronization
- Idempotent sync operations

### Infrastructure Features

- Health check endpoint with PostgreSQL and Redis status
- Swagger UI API documentation at `/swagger-ui`
- Graceful shutdown handling
- Request timeout middleware (10 seconds)
- CORS support for all origins
- Structured JSON logging with tracing

---

## Technology Stack

| Category | Dependency | Version | Purpose |
|----------|------------|---------|---------|
| **Web Framework** | axum | 0.8.8 | HTTP server with type-safe routing |
| **Middleware** | tower | 0.5.2 | Middleware composability |
| **Middleware** | tower-http | 0.6.2 | CORS, tracing, timeout layers |
| **Async Runtime** | tokio | 1.49.0 | Multi-threaded runtime |
| **Serialization** | serde | 1.0.228 | JSON serialization/deserialization |
| **Database ORM** | sqlx | 0.8.6 | Async PostgreSQL with compile-time checks |
| **Redis Client** | redis | 1.0.4 | Redis connection for leaderboard caching |
| **UUID** | uuid | 1.21.0 | UUID generation and serialization |
| **Date/Time** | chrono | 0.4.44 | DateTime handling with timezone support |
| **Environment** | dotenvy | 0.15.7 | Environment variable loading |
| **Error Handling** | thiserror | 2.0.18 | Custom error types with derive macros |
| **Error Handling** | anyhow | 1.0.95 | Flexible error context for applications |
| **Logging** | tracing | 0.1.41 | Structured logging facade |
| **Logging** | tracing-subscriber | 0.3.22 | Logging subscriber with env filter |
| **Validation** | validator | 0.20.0 | Request validation with derive macros |
| **HTTP Client** | reqwest | 0.13.2 | HTTP client for internal service calls |
| **Async Traits** | async-trait | 0.1 | Async trait methods |
| **API Documentation** | utoipa | 5.4.0 | OpenAPI generation |
| **Swagger UI** | utoipa-swagger-ui | 9.0.2 | Swagger UI integration |
| **Testing** | mockall | 0.14.0 | Mocking for trait tests |

### Rust Edition and MSRV

- **Edition**: 2024
- **MSRV (Minimum Supported Rust Version)**: 1.85

---

## Architecture Diagram

### System Overview

```mermaid
flowchart LR
    subgraph Frontend["Frontend (Next.js)"]
        UI["React UI"]
    end

    subgraph RustBackend["Yomu Engine (Rust)"]
        API["BFF API Proxy"]
        subgraph Modules["Bounded Contexts"]
            League["League Module\n- Clans\n- Leaderboards"]
            Gamification["Gamification Module\n- Achievements\n- Missions\n- Rewards"]
            UserSync["User Sync Module\n- Shadow Users\n- Quiz History"]
        end
    end

    subgraph JavaBackend["Java Backend"]
        Outbox["Outbox Pattern\n(Sync Events)"]
    end

    subgraph Data["Data Layer"]
        PG[("PostgreSQL")]
        Redis[("Redis")]
    end

    UI --> API
    API --> League
    API --> Gamification
    Outbox -->|"REST sync"| UserSync
    League --> PG
    League --> Redis
    Gamification --> PG
    UserSync --> PG
```

### Module Architecture (Hexagonal/Ports & Adapters)

```mermaid
flowchart TD
    subgraph Presentation["Presentation Layer"]
        Controllers["Controllers\n(Axum Handlers)"]
        Routes["Routes"]
    end

    subgraph Application["Application Layer"]
        UseCases["Use Cases\n(Business Logic)"]
        DTOs["DTOs\n(Data Transfer)"]
    end

    subgraph Domain["Domain Layer"]
        Entities["Entities"]
        Traits["Repository Traits\n(Ports)"]
        Errors["Error Types"]
    end

    subgraph Infrastructure["Infrastructure Layer"]
        Postgres["PostgreSQL Adapter\n(SQLx)"]
        Redis["Redis Adapter"]
        HTTP["HTTP Client"]
    end

    Routes --> Controllers
    Controllers --> UseCases
    UseCases --> DTOs
    UseCases --> Traits
    UseCases --> Entities
    Entities --> Errors
    Traits <--> Postgres
    Traits <--> Redis
    Traits <--> HTTP
```

### API Flow Diagram

```mermaid
sequenceDiagram
    participant Client
    participant API as API Route
    participant Controller
    participant UseCase as Use Case
    participant Repo as Repository
    participant DB as Database

    Client->>API: POST /clans {name, leader_id}
    API->>Controller: Json<CreateClanDto>
    Controller->>UseCase: execute(dto)
    UseCase->>Repo: is_user_in_any_clan(leader_id)
    Repo->>DB: SELECT COUNT(*) FROM clan_members
    DB-->>Repo: count
    Repo-->>UseCase: bool
    alt user already in clan
        UseCase-->>Controller: Err(AppError::BadRequest)
        Controller-->>Client: 400 Bad Request
    else user not in clan
        UseCase->>Repo: create_clan(clan)
        Repo->>DB: INSERT INTO clans
        UseCase->>Repo: add_member(member)
        Repo->>DB: INSERT INTO clan_members
        Repo-->>UseCase: Ok(())
        UseCase-->>Controller: Ok(Clan)
        Controller-->>Client: 201 Created
    end
```

### Directory Structure

```mermaid
graph TD
    root["yomu-backend-rust/"]
    root --> src["src/"]
    root --> migrations["migrations/"]
    root --> tests["tests/"]
    root --> github[".github/"]
    root --> docs["docs/"]

    src --> main["main.rs"]
    src --> lib["lib.rs"]
    src --> config["config/"]
    src --> shared["shared/"]
    src --> modules["modules/"]

    config --> config_mod["mod.rs"]
    config --> database["database.rs"]

    shared --> domain["domain/"]
    shared --> utils["utils/"]
    domain --> base_error["base_error.rs"]
    utils --> response["response.rs"]

    modules --> league["league/"]
    modules --> gamification["gamification/"]
    modules --> user_sync["user_sync/"]

    league --> league_domain["domain/"]
    league --> league_app["application/"]
    league --> league_infra["infrastructure/"]
    league --> league_present["presentation/"]

    league_domain --> entities["entities/"]
    league_domain --> repos["repositories/"]
    league_domain --> errors["errors/"]

    gamification --> gama_domain["domain/"]
    gama_domain --> entities_g["entities/"]
    gama_domain --> repos_g["repositories/"]

    user_sync --> user_domain["domain/"]
    user_domain --> entities_u["entities/"]
    user_domain --> repos_u["repositories/"]
    user_sync --> user_app["application/"]
    user_sync --> user_infra["infrastructure/"]
    user_sync --> user_present["presentation/"]
```

---

## Prerequisites

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| Rust | 1.85+ | Compiler toolchain |
| Cargo | Latest | Package manager |
| Docker | 24.0+ | Container runtime |
| Docker Compose | 2.20+ | Multi-container orchestration |
| PostgreSQL | 18+ | Primary database |
| Redis | 8.0+ | Leaderboard cache |

### Optional Software

| Software | Purpose |
|----------|---------|
| sqlx-cli | Database migration management |
| cargo-watch | Hot reload during development |
| cargo-tarpaulin | Code coverage reporting |

---

## Installation

### Clone the Repository

```bash
git clone <repository-url>
cd yomu-backend-rust
```

### Install Rust Dependencies

```bash
cargo fetch
```

### Copy Environment Configuration

```bash
cp .env.example .env
```

Edit `.env` with your configuration values. See [Configuration](#configuration) for details.

### Verify SQLx Offline Mode

```bash
cargo sqlx prepare --workspace --check
```

This verifies that all SQL queries compile correctly against the database schema without requiring a live database connection.

---

## Configuration

All configuration is managed through environment variables. Copy `.env.example` to `.env` and adjust values as needed.

### Environment Variables

| Variable | Default | Required | Description |
|----------|---------|----------|-------------|
| `APP_HOST` | `0.0.0.0` | No | Server bind address |
| `APP_PORT` | `8080` | No | Server port |
| `DATABASE_URL` | - | **Yes** | PostgreSQL connection string |
| `REDIS_URL` | `redis://localhost:6379` | No | Redis connection string |
| `JAVA_CORE_URL` | - | **Yes** | Java backend URL for internal API calls |
| `JAVA_CORE_API_KEY` | - | **Yes** | API key for Java backend authentication |
| `RUST_LOG` | `info` | No | Logging level (trace, debug, info, warn, error) |
| `RUST_BACKTRACE` | `1` | No | Enable backtraces on panics |

### Example .env File

```env
# Server Configuration
APP_HOST=0.0.0.0
APP_PORT=8080

# Database Configuration
DATABASE_URL=postgres://yomu:yomu_password@localhost:5432/yomu_engine

# Redis Configuration
REDIS_URL=redis://localhost:6379

# Java Core Service (Internal API)
JAVA_CORE_URL=http://localhost:8081
JAVA_CORE_API_KEY=your_api_key_here

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
```

### Connection String Formats

**PostgreSQL**:
```
postgresql://username:password@host:port/database_name
```

**Redis**:
```
redis://host:port
```

---

## Database Setup

### Schema Overview

The database schema consists of these main entity groups:

```mermaid
erDiagram
    ENGINE_USER ||--o{ CLAN_MEMBER : has
    CLAN ||--o{ CLAN_MEMBER : has
    CLAN ||--o{ CLAN_BUFF : has
    ENGINE_USER ||--o{ USER_ACHIEVEMENT : earned
    ACHIEVEMENT ||--o{ USER_ACHIEVEMENT : awarded_to
    ENGINE_USER ||--o{ USER_MISSION : assigned
    DAILY_MISSION ||--o{ USER_MISSION : assigned_to
    ENGINE_USER ||--o{ QUIZ_HISTORY : completed
```

### Tables

#### engine_users
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| user_id | UUID | PRIMARY KEY | Unique user identifier |
| total_score | INT | NOT NULL, DEFAULT 0 | User's total gamification score |

#### clans
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PRIMARY KEY | Unique clan identifier |
| name | VARCHAR(255) | NOT NULL | Clan display name |
| leader_id | UUID | FOREIGN KEY | Clan founder/leader |
| tier | VARCHAR(50) | NOT NULL | Clan tier (Bronze/Silver/Gold/Diamond) |
| total_score | INT | NOT NULL, DEFAULT 0 | Combined clan score |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Creation timestamp |

#### clan_members
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| clan_id | UUID | PRIMARY KEY, FOREIGN KEY | Reference to clans table |
| user_id | UUID | PRIMARY KEY, FOREIGN KEY | Reference to engine_users |
| joined_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Join timestamp |

#### clan_buffs
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PRIMARY KEY | Unique buff identifier |
| clan_id | UUID | FOREIGN KEY | Reference to clans table |
| buff_name | VARCHAR(255) | NOT NULL | Name of the buff |
| multiplier | DECIMAL(5,2) | NOT NULL | Score multiplier value |
| is_active | BOOLEAN | NOT NULL, DEFAULT true | Whether buff is currently active |
| expires_at | TIMESTAMPTZ | NOT NULL | When the buff expires |

#### achievements
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PRIMARY KEY | Unique achievement identifier |
| name | VARCHAR(255) | NOT NULL | Achievement name |
| milestone_target | INT | NOT NULL | Score target to unlock |
| achievement_type | VARCHAR(100) | NOT NULL | Category of achievement |
| reward_points | INT | NOT NULL, DEFAULT 0 | Points awarded on completion |

#### user_achievements
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| user_id | UUID | PRIMARY KEY, FOREIGN KEY | Reference to engine_users |
| achievement_id | UUID | PRIMARY KEY, FOREIGN KEY | Reference to achievements |
| current_progress | INT | NOT NULL, DEFAULT 0 | Progress toward milestone |
| is_completed | BOOLEAN | NOT NULL, DEFAULT false | Completion status |
| is_shown_on_profile | BOOLEAN | NOT NULL, DEFAULT false | Visibility flag |
| completed_at | TIMESTAMPTZ | NULL | Completion timestamp |

#### daily_missions
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PRIMARY KEY | Unique mission identifier |
| description | VARCHAR(255) | NOT NULL | Mission description |
| target_count | INT | NOT NULL | Number of actions required |
| date | DATE | NOT NULL | Date mission is active |
| reward_points | INT | NOT NULL, DEFAULT 0 | Points awarded on claim |

#### user_missions
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| user_id | UUID | PRIMARY KEY, FOREIGN KEY | Reference to engine_users |
| mission_id | UUID | PRIMARY KEY, FOREIGN KEY | Reference to daily_missions |
| current_progress | INT | NOT NULL, DEFAULT 0 | Progress toward completion |
| is_claimed | BOOLEAN | NOT NULL, DEFAULT false | Reward claim status |

#### quiz_history
| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PRIMARY KEY | Unique quiz attempt identifier |
| user_id | UUID | FOREIGN KEY | Reference to engine_users |
| article_id | UUID | NOT NULL | Reference to article taken |
| score | INT | NOT NULL, DEFAULT 0 | Quiz score achieved |
| accuracy | DECIMAL(5,2) | NOT NULL, DEFAULT 0.00 | Accuracy percentage |
| completed_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Completion timestamp |

---

## Running the Application

### Method 1: Docker Compose (Recommended)

Start all services with a single command:

```bash
docker compose up -d
```

This starts PostgreSQL, Redis, and the application container.

### Method 2: Hybrid (Docker for Infra, Native for App)

Start only infrastructure services:

```bash
docker compose up -d postgres redis
```

Run the application natively:

```bash
cargo run
```

### Method 3: Native (Full Local Development)

Requires PostgreSQL and Redis running locally.

```bash
# Start PostgreSQL on port 5432
# Start Redis on port 6379

cargo run
```

### Method 4: Development with Hot Reload

```bash
docker compose up -d dev
```

The dev service uses `cargo-watch` to automatically rebuild and restart on source changes.

---

## Docker Compose

### Services Overview

```mermaid
graph LR
    subgraph Network["yomu-network"]
        subgraph Services
            App["app\n:8080"]
            Dev["dev\n:8080, :2222"]
            PG["postgres\n:5432"]
            Redis["redis\n:6379"]
        end
    end
```

### Service Definitions

#### PostgreSQL

```yaml
postgres:
  image: postgres:18-alpine
  container_name: yomu-postgres
  restart: unless-stopped
  environment:
    POSTGRES_USER: yomu
    POSTGRES_PASSWORD: yomu_password
    POSTGRES_DB: yomu_engine
  ports:
    - "5432:5432"
  volumes:
    - postgres_data:/var/lib/postgresql/data
  healthcheck:
    test: ["CMD-SHELL", "pg_isready -U yomu"]
    interval: 10s
    timeout: 5s
    retries: 10
    start_period: 30s
```

#### Redis

```yaml
redis:
  image: redis:8-alpine
  container_name: yomu-redis
  restart: unless-stopped
  ports:
    - "6379:6379"
  volumes:
    - redis_data:/data
  healthcheck:
    test: ["CMD", "redis-cli", "ping"]
    interval: 10s
    timeout: 5s
    retries: 5
    start_period: 10s
  command: redis-server --appendonly yes
```

#### Application (Production)

```yaml
app:
  build:
    context: .
    dockerfile: Dockerfile
  container_name: yomu-engine
  restart: unless-stopped
  ports:
    - "8080:8080"
  environment:
    - DATABASE_URL=postgresql://yomu:yomu_password@postgres:5432/yomu_engine
    - REDIS_URL=redis://redis:6379
    - RUST_LOG=info
    - RUST_BACKTRACE=1
  depends_on:
    postgres:
      condition: service_healthy
    redis:
      condition: service_healthy
```

#### Development (Hot Reload)

```yaml
dev:
  build:
    context: .
    dockerfile: Dockerfile.dev
  container_name: yomu-engine-dev
  restart: unless-stopped
  ports:
    - "8080:8080"
    - "2222:2222"
  environment:
    - DATABASE_URL=postgresql://yomu:yomu_password@postgres:5432/yomu_engine
    - REDIS_URL=redis://redis:6379
    - RUST_LOG=debug
    - RUST_BACKTRACE=1
  volumes:
    - .:/app
    - cargo_cache:/app/target
```

### Volumes

| Volume | Purpose |
|--------|---------|
| `postgres_data` | Persistent PostgreSQL data |
| `redis_data` | Persistent Redis data (AOF) |
| `cargo_cache` | Cached Rust compilation artifacts |

### Networking

All services connect through the `yomu-network` bridge network. The application container can reach PostgreSQL at `postgres:5432` and Redis at `redis:6379`.

---

## API Documentation

### API Routes Overview

```mermaid
mindmap
    root((Yomu API))
        /health
            GET /health
        /swagger-ui
            SwaggerUI
        /api/v1
            /clans
                POST /clans
                GET /clans/:id
                POST /clans/:id/join
            /leaderboards
                GET /leaderboards?tier=X
            /users/:user_id/tier
                GET /users/:user_id/tier
        /api/internal
            /users/sync
                POST /users/sync
```

### Endpoint Reference

#### Health Check

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Returns PostgreSQL and Redis connection status |

**Response Example**:
```json
{
  "success": true,
  "message": "Server is running well",
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "postgres": "connected",
    "redis": "connected"
  }
}
```

#### Public API (`/api/v1`)

##### Create Clan

| Property | Value |
|----------|-------|
| Method | `POST` |
| Endpoint | `/api/v1/clans` |
| Tag | clans |

**Request Body**:
```json
{
  "name": "Dragon Slayers",
  "leader_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response** (201 Created):
```json
{
  "success": true,
  "message": "Clan created successfully",
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Dragon Slayers",
    "leader_id": "550e8400-e29b-41d4-a716-446655440000",
    "tier": "Bronze",
    "total_score": 0,
    "created_at": "2026-04-16T10:30:00Z"
  }
}
```

##### Get Clan Details

| Property | Value |
|----------|-------|
| Method | `GET` |
| Endpoint | `/api/v1/clans/{id}` |
| Tag | League |

**Path Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| id | UUID | Clan identifier |

**Response Example**:
```json
{
  "success": true,
  "message": "Clan detail retrieved",
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Dragon Slayers",
    "leader_id": "550e8400-e29b-41d4-a716-446655440000",
    "tier": "Bronze",
    "total_score": 1500,
    "created_at": "2026-04-16T10:30:00Z",
    "members": [
      {
        "user_id": "550e8400-e29b-41d4-a716-446655440000",
        "role": "Leader",
        "joined_at": "2026-04-16T10:30:00Z"
      }
    ],
    "active_buffs": [],
    "active_debuffs": []
  }
}
```

##### Join Clan

| Property | Value |
|----------|-------|
| Method | `POST` |
| Endpoint | `/api/v1/clans/{id}/join` |
| Tag | clans |

**Request Body**:
```json
{
  "user_id": "660e9500-f30c-52e5-b827-557766551111"
}
```

**Response Example**:
```json
{
  "success": true,
  "message": "Joined clan successfully",
  "data": {
    "clan_id": "123e4567-e89b-12d3-a456-426614174000",
    "user_id": "660e9500-f30c-52e5-b827-557766551111",
    "joined_at": "2026-04-16T11:00:00Z"
  }
}
```

##### Get Leaderboard

| Property | Value |
|----------|-------|
| Method | `GET` |
| Endpoint | `/api/v1/leaderboards` |
| Tag | leaderboard |

**Query Parameters**:
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| tier | String | Bronze | Leaderboard tier (Bronze, Silver, Gold, Diamond) |

**Response Example**:
```json
{
  "success": true,
  "message": "Leaderboard fetched successfully",
  "data": {
    "tier": "Diamond",
    "entries": [
      {
        "clan_id": "123e4567-e89b-12d3-a456-426614174000",
        "clan_name": "Elite Squad",
        "total_score": 50000,
        "tier": "Diamond",
        "rank": 1
      },
      {
        "clan_id": "789e0123-a45b-67c8-d901-234567890123",
        "clan_name": "Warriors",
        "total_score": 45000,
        "tier": "Diamond",
        "rank": 2
      }
    ]
  }
}
```

##### Get User Tier

| Property | Value |
|----------|-------|
| Method | `GET` |
| Endpoint | `/api/v1/users/{user_id}/tier` |
| Tag | League |

**Path Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| user_id | UUID | User identifier |

**Response Example**:
```json
{
  "success": true,
  "message": "Data liga pengguna berhasil diambil",
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "tier": "Gold",
    "total_score": 2500
  }
}
```

#### Internal API (`/api/internal`)

##### Sync User

| Property | Value |
|----------|-------|
| Method | `POST` |
| Endpoint | `/api/internal/users/sync` |
| Tag | User Sync |

**Request Body**:
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com"
}
```

**Response Example**:
```json
{
  "success": true,
  "message": "Shadow user berhasil disinkronisasi",
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "message": "Shadow user berhasil disinkronisasi"
  }
}
```

### Swagger UI

Interactive API documentation is available at:

```
http://localhost:8080/swagger-ui
```

The OpenAPI specification JSON is available at:

```
http://localhost:8080/api-docs/openapi.json
```

---

## API Response Format

All API responses follow a consistent JSON structure.

### Success Response

```json
{
  "success": true,
  "message": "Operation completed successfully",
  "data": { ... }
}
```

### Success Response Without Data

```json
{
  "success": true,
  "message": "Operation completed successfully",
  "data": null
}
```

### Error Response

```json
{
  "success": false,
  "message": "Error description",
  "data": null
}
```

### HTTP Status Code Mapping

| Status Code | Meaning | When Used |
|-------------|---------|-----------|
| 200 | OK | Successful GET, POST operations |
| 201 | Created | Successful resource creation |
| 400 | Bad Request | Invalid input, validation errors |
| 404 | Not Found | Resource does not exist |
| 409 | Conflict | Duplicate resource, user already in clan |
| 500 | Internal Server Error | Database errors, unexpected failures |
| 503 | Service Unavailable | Database or Redis connection failure |

---

## Testing

### Test Structure

Tests are organized by module and layer:

```
tests/
├── modules/
│   ├── league/
│   │   ├── usecase_test.rs      # Use case logic tests
│   │   ├── infrastructure_test.rs # Repository implementation tests
│   │   └── presentation_test.rs  # Controller/route tests
│   ├── gamification/
│   │   ├── domain_test.rs
│   │   ├── usecase_test.rs
│   │   └── infrastructure_test.rs
│   └── user_sync/
│       ├── domain_test.rs
│       ├── usecase_test.rs
│       └── infrastructure_test.rs
```

### Running Tests

#### All Tests

```bash
cargo test
```

#### Library Unit Tests Only

```bash
cargo test --lib
```

#### Binary Unit Tests Only

```bash
cargo test --bin yomu-backend-rust
```

#### Integration Tests by Category

```bash
# Use case tests
cargo test --test usecase_test

# Infrastructure tests
cargo test --test infrastructure_test

# Presentation tests
cargo test --test presentation_test
```

#### Specific Test Module

```bash
cargo test league
cargo test gamification
cargo test user_sync
```

### Testing with Docker

Start test database services:

```bash
docker compose up -d postgres redis
```

Run migrations on test database:

```bash
export DATABASE_URL=postgresql://yomu:yomu_password@localhost:5432/yomu_engine
cargo sqlx migrate run
```

Run tests:

```bash
cargo test
```

### Code Coverage

Generate code coverage report:

```bash
cargo tarpaulin --out Xml --output-dir ./coverage
```

The report is generated at `coverage/cobertura.xml` and can be uploaded to SonarCloud.

---

## Development Workflow

### Daily Development Cycle

1. **Start infrastructure**:
   ```bash
   docker compose up -d postgres redis
   ```

2. **Run in development mode with hot reload**:
   ```bash
   docker compose up -d dev
   ```

3. **Make code changes** - the dev container auto-rebuilds

4. **Run tests locally**:
   ```bash
   cargo test
   cargo clippy --all -- -W warnings
   ```

### Coding Standards

1. **Format code before committing**:
   ```bash
   cargo fmt --all
   ```

2. **Run clippy lints**:
   ```bash
   cargo clippy --all -- -W warnings
   ```

3. **Verify documentation builds**:
   ```bash
   cargo doc --all --no-deps
   ```

4. **Check SQLx offline mode**:
   ```bash
   cargo sqlx prepare --workspace --check
   ```

### Project Conventions

| Convention | Description |
|-------------|-------------|
| **Architecture** | Hexagonal (Ports & Adapters) per module |
| **Error Handling** | Domain errors via `thiserror`, application errors via `anyhow` |
| **Async Traits** | Use `async-trait` for object-safe repository traits |
| **Response Pattern** | Controllers return `Result<Json<ApiResponse<T>>, AppError>` |
| **Entity Design** | Owned data (no `Arc` unless necessary) |
| **Code Organization** | `domain/application/infrastructure/presentation` per module |

### Anti-Patterns to Avoid

- **No domain logic in controllers** - delegate to use cases
- **No DB queries in use cases** - use repository abstractions
- **No `unwrap()` or `expect()` in production code** - use `?` operator or `anyhow::Context`
- **No sync code in async functions** - all handlers must be async

---

## Code Quality

### Formatting

Format all code:

```bash
cargo fmt --all
```

Check formatting without applying:

```bash
cargo fmt --all -- --check
```

### Linting with Clippy

Run all clippy checks:

```bash
cargo clippy --all --all-targets -- -W warnings
```

### Documentation

Build documentation:

```bash
cargo doc --all --no-deps
```

Build documentation with warnings as errors:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps
```

### SQLx Offline Mode

SQLx performs compile-time verification of SQL queries. To prepare for offline mode:

```bash
cargo sqlx prepare --workspace --check
```

This verifies all queries without requiring a database connection.

---

## Database Migrations

### Migration Files Location

Migrations are stored in the `migrations/` directory with naming convention:

```
YYYYMMDDHHMMSS_<migration_name>.sql
```

### Existing Migrations

| Migration | Description |
|-----------|-------------|
| `20260224122751_first_migrations` | Creates all core tables (engine_users, clans, achievements, etc.) |
| `20260416022505_add_reward_points` | Adds reward_points column to achievements and daily_missions |

### Creating a New Migration

```bash
cargo sqlx migrate add <migration_name>
```

This creates a new file in the migrations directory:

```
migrations/
├── 20260224122751_first_migrations.sql
├── 20260416022505_add_reward_points.sql
└── YYYYMMDDHHMMSS_<migration_name>.sql
```

### Running Migrations

With local database:

```bash
export DATABASE_URL=postgresql://yomu:yomu_password@localhost:5432/yomu_engine
cargo sqlx migrate run
```

With Docker:

```bash
docker compose up -d postgres
docker compose exec postgres psql -U yomu -d yomu_engine -c "CREATE DATABASE yomu_engine;"
cargo sqlx migrate run
```

### Reverting Migrations

```bash
cargo sqlx migrate revert
```

### Migration Guidelines

1. Each migration should be idempotent when possible
2. Use `CREATE TABLE IF NOT EXISTS` for tables
3. Use `ALTER TABLE` with `IF EXISTS`/`IF NOT EXISTS` where supported
4. Always include a rollback strategy in comments

---

## Troubleshooting

### Common Issues and Solutions

#### Database Connection Failed

**Error**:
```
Failed connecting to database: connection refused
```

**Solution**:
1. Verify PostgreSQL is running:
   ```bash
   docker compose ps postgres
   ```
2. Check DATABASE_URL is correct
3. Ensure port 5432 is not blocked by firewall

#### Redis Connection Failed

**Error**:
```
Failed connecting to Redis: error connecting to redis
```

**Solution**:
1. Verify Redis is running:
   ```bash
   docker compose ps redis
   ```
2. Check REDIS_URL is correct
3. Ensure port 6379 is accessible

#### SQLx Query Not Found

**Error**:
```
error[sqlx]: unknown query
```

**Solution**:
1. Ensure `SQLX_OFFLINE=true` is not set during development
2. Run `cargo sqlx prepare --workspace --check` to verify queries
3. Rebuild with `cargo build`

#### Migration Failed

**Error**:
```
Error while doing database migrations: migrate error
```

**Solution**:
1. Check migration file syntax
2. Verify database exists and user has permissions
3. Check for conflicting migrations (duplicate versions)

#### Port Already in Use

**Error**:
```
Failed to bind TCP listener on 0.0.0.0:8080: address already in use
```

**Solution**:
1. Find and stop the process using port 8080:
   ```bash
   lsof -i :8080
   # or
   fuser -k 8080/tcp
   ```
2. Or change APP_PORT in .env

#### Docker Build Failed

**Error**:
```
error: cannot find container image
```

**Solution**:
1. Ensure Docker is running
2. Clear build cache:
   ```bash
   docker builder prune
   ```

### Health Check Failures

The `/health` endpoint checks both PostgreSQL and Redis connections.

**Response when unhealthy**:
```json
{
  "success": true,
  "message": "Server is running well",
  "data": {
    "status": "unhealthy",
    "version": "0.1.0",
    "postgres": "error: connection refused",
    "redis": "connected"
  }
}
```

Returns HTTP 503 when unhealthy.

---

## Deployment

### Production Build

The production Dockerfile uses a multi-stage build with musl target for static linking:

```bash
docker build -f Dockerfile -t yomu-engine:latest .
```

### Railway Deployment

The Dockerfile is optimized for Railway deployment:

1. Uses `rust:1.93-bookworm` for building
2. Targets `x86_64-unknown-linux-musl` for static binary
3. Runs as non-root user `yomuuser`
4. Minimal runtime image based on `debian:bookworm-slim`

### Environment Variables for Production

At minimum, set these environment variables:

```env
DATABASE_URL=postgresql://user:password@host:port/database
REDIS_URL=redis://host:port
JAVA_CORE_URL=https://java-backend.example.com
JAVA_CORE_API_KEY=your_secure_api_key
RUST_LOG=warn
```

### Health Check Configuration

Configure load balancer to check:

```
GET http://host:8080/health
```

Expect HTTP 200 with JSON body containing `"status": "healthy"`.

### Graceful Shutdown

The application handles SIGTERM for graceful shutdown. The server stops accepting new connections and waits for in-flight requests to complete (up to timeout).

---

## Monitoring

### Health Endpoint

The `/health` endpoint provides real-time status:

```
GET /health
```

Response includes:
- Overall status (healthy/unhealthy)
- Version from Cargo.toml
- PostgreSQL connection status
- Redis connection status

### Structured Logging

All logs use the `tracing` crate with JSON formatting:

```
RUST_LOG=debug   # Most verbose
RUST_LOG=info    # Default production
RUST_LOG=warn    # Warnings only
RUST_LOG=error   # Errors only
```

### Request Tracing

Tower middleware provides:
- Request/response logging
- Timeout enforcement (10 seconds)
- CORS headers

### Metrics (Future)

Planned integration:
- Prometheus metrics endpoint
- Request duration histograms
- Database query timing
- Cache hit/miss ratios

---

## Security

### CORS Configuration

The application currently allows all origins for development:

```rust
CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any)
```

**Production Note**: Restrict CORS to specific origins before deployment.

### Authentication

The public API does not include authentication (handled by the frontend BFF). The internal API (`/api/internal`) should be protected via network isolation or API key verification.

### API Key for Internal Services

The `JAVA_CORE_API_KEY` environment variable is intended for authenticating requests from the Java backend to this service.

### Input Validation

Request validation uses the `validator` crate:

```rust
#[derive(Validate)]
struct CreateClanDto {
    #[validate(length(min = 1, max = 255))]
    name: String,
    #[validate(uuid)]
    leader_id: Uuid,
}
```

### Rate Limiting (Future)

Not currently implemented. Consider adding tower Governor or similar middleware for production.

---

## CI/CD

### GitHub Actions Workflows

```mermaid
flowchart LR
    subgraph CI["CI Pipeline"]
        direction TB
        fmt["fmt\nCheck formatting"]
        clippy["clippy\nLint checks"]
        doc["doc\nBuild docs"]
        test["test\nTest suite"]
        docker["docker\nBuild image"]

        fmt --> clippy --> doc --> test --> docker
    end

    subgraph Sonar["SonarCloud"]
        sonar["sonar\nCoverage scan"]
    end

    test --> sonar
```

### CI Workflow (ci.yml)

Runs on every push to any branch:

| Job | Steps |
|-----|-------|
| **fmt** | Check code formatting with rustfmt |
| **clippy** | Run clippy lints with warnings as errors |
| **doc** | Build documentation |
| **test** | Run all tests with PostgreSQL and Redis services |
| **docker** | Build Docker image (after all jobs pass) |

### SonarCloud Workflow (sonar.yml)

Runs on push and pull requests:

| Step | Description |
|------|-------------|
| Checkout | Fetch full history for coverage |
| Setup Rust | Install Rust toolchain |
| Install tarpaulin | Code coverage tool |
| Install sqlx-cli | Migration tool |
| Run migrations | Set up test database |
| Generate coverage | Run tarpaulin with Cobertura output |
| Sonar scan | Upload to SonarCloud |

### Environment Variables for CI

The CI pipeline uses:
- `DATABASE_URL`: Test database connection
- `REDIS_URL`: Redis connection
- `SONAR_TOKEN`: SonarCloud authentication (repository secret)

### Badge Status

Configure badges in your repository settings:
- CI: `.github/workflows/ci.yml`
- SonarCloud: `.github/workflows/sonar.yml`

---

## Contributing

### Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/yomu-backend-rust.git
   cd yomu-backend-rust
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/original-repo/yomu-backend-rust.git
   ```

### Branch Strategy

| Branch | Purpose |
|--------|---------|
| `main` | Production-ready code |
| `develop` | Integration branch |
| `feature/*` | New feature development |
| `fix/*` | Bug fixes |
| `refactor/*` | Code refactoring |

### Commit Messages

Follow conventional commits:

```
feat(league): add clan tier promotion system
fix(gamification): correct mission progress calculation
docs(api): update endpoint documentation
refactor(user_sync): simplify sync logic
test(league): add leaderboard query tests
```

### Pull Request Process

1. Create a feature branch from `develop`
2. Make changes with passing tests
3. Run code quality checks:
   ```bash
   cargo fmt --all
   cargo clippy --all -- -W warnings
   cargo test
   ```
4. Open a pull request with description
5. Request review from maintainers
6. Address feedback
7. Squash and merge when approved

### Code Review Checklist

- [ ] Code follows project conventions
- [ ] Tests pass and cover new functionality
- [ ] Documentation updated if needed
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] SQLx queries verified offline

---

## License

This project is proprietary software. All rights reserved.

---

## Links

| Resource | URL |
|----------|-----|
| Repository | `<repository-url>` |
| API Documentation | `http://localhost:8080/swagger-ui` |
| OpenAPI Spec | `http://localhost:8080/api-docs/openapi.json` |
| Health Check | `http://localhost:8080/health` |

---

## Appendix

### A. Clan Tier System

| Tier | Description | Typical Score Range |
|------|-------------|---------------------|
| Bronze | Entry level | 0 - 4,999 |
| Silver | Intermediate | 5,000 - 19,999 |
| Gold | Advanced | 20,000 - 49,999 |
| Diamond | Elite | 50,000+ |

### B. Error Codes Reference

| Error Type | HTTP Status | Description |
|------------|-------------|-------------|
| `AppError::BadRequest` | 400 | Invalid request data |
| `AppError::NotFound` | 404 | Resource not found |
| `AppError::InternalServer` | 500 | Server-side error |

### C. Key Dependencies Graph

```mermaid
graph TB
    Axum --> Tower
    Axum --> Tokio
    Sqlx --> Tokio
    Sqlx --> PostgreSQL
    Redis --> Tokio
    Utoipa --> Swagger
    Tower-http --> Tower
```
