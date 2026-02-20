# Yomu Backend Rust

## Project Structure

```
yomu-engine-rust/
├── Cargo.toml
├── .env.example
├── .gitignore
├── README.md
├── Dockerfile
├── Dockerfile.dev
├── docker-compose.yml
├── rustfmt.toml
├── .clippy.toml
│
└── src/
    ├── main.rs                    # Entry point (Axum, DB, Dependencies init)
    ├── config/                    # Environment & connection settings
    │
    ├── shared/                    # Shared Kernel (cross-module code)
    │   ├── domain/                # Base types/structs for Domain
    │   └── utils/                 # Common helpers
    │
    └── modules/                   # BOUNDED CONTEXTS (Feature modules)
        ├── league/                # Clan & League module
        ├── gamification/          # Achievements & Missions module
        └── user_sync/             # User Synchronization module
```

## Quick Start with Docker

### Prerequisites
- Docker
- Docker Compose

### Option 1: Production Mode
```bash
# Start PostgreSQL, Redis, and Application
docker-compose up -d

# Check if services are running
docker-compose ps

# View logs
docker-compose logs -f app
```

### Option 2: Development Mode (Hot Reload)
```bash
# Start all services with hot reload
docker-compose up -d dev

# View dev logs
docker-compose logs -f dev
```

### Access the Application
- **Health Check**: http://localhost:8080/health
- **PostgreSQL**: localhost:5432 (user: yomu, password: yomu_password)
- **Redis**: localhost:6379

## Local Development (Without Docker)

### Prerequisites
- Rust 1.75+
- PostgreSQL 14+
- Redis 7+

### Setup
```bash
# 1. Copy environment file
cp .env.example .env

# 2. Update .env with your database credentials
# DATABASE_URL=postgres://yomu:yomu_password@localhost:5432/yomu_engine

# 3. Run the server
cargo run

# Or with hot reload
cargo watch -x run
```

### Linting
```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all -- -D warnings
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check endpoint |

## Architecture

This project uses a **Module-Based (Feature-Based) Structure**. Instead of grouping files by their technical roles, we group them by their **Bounded Contexts** (business features).

Inside each module, we apply the Hexagonal Architecture layers:

| Layer | Description |
|-------|-------------|
| `domain/` | Pure Rust, zero external dependencies. Entities, Value Objects, Ports (Traits) |
| `application/` | Use Cases (Application Logic), DTOs |
| `infrastructure/` | Adapters: PostgreSQL (SQLx), Redis, HTTP clients |
| `presentation/` | Delivery: Axum Controllers & Routes |

## Modules

### 1. League Module (`src/modules/league/`)
- Clan management (create, join, leave)
- Score calculation & Leaderboard
- Ports: `ClanRepository`, `LeaderboardCache`
- Adapters: PostgreSQL (SQLx), Redis

### 2. Gamification Module (`src/modules/gamification/`)
- Achievements & Missions system
- Reward claiming logic

### 3. User Sync Module (`src/modules/user_sync/`)
- Internal API for syncing users from Java Core
- Shadow user entity representation

## Dependencies

- **Web**: Axum, Tower, Tower-HTTP
- **Database**: SQLx (PostgreSQL), Redis
- **Async**: Tokio
- **Serialization**: Serde
- **Error Handling**: Thiserror, Anyhow
- **Logging**: Tracing

## Docker Commands Reference

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (reset database)
docker-compose down -v

# Rebuild containers
docker-compose build

# Rebuild and start
docker-compose up -d --build

# View all container logs
docker-compose logs

# Run specific service
docker-compose up -d postgres
```

## CI/CD

GitHub Actions workflow is configured in `.github/workflows/ci.yml`:
- Format check (rustfmt)
- Lint (clippy)
- Documentation build
- Tests with PostgreSQL & Redis
- Docker image build
