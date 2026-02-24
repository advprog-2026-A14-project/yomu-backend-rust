# Yomu Backend Rust

A high-performance, asynchronous REST API backend built with Rust, designed to power the Yomu backend services platform. This backend handles core features including clan management, gamification systems, and user synchronization with the Java Core platform.

## Table of Contents

- [Project Overview](#project-overview)
- [Technology Stack](#technology-stack)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [How to Run](#how-to-run)
- [Development Workflow](#development-workflow)
- [Useful Commands \& Troubleshooting](#useful-commands--troubleshooting)
- [API Endpoints](#api-endpoints)
- [Architecture](#architecture)
- [Modules Summary](#modules-summary)
- [Database Schema](#database-schema)
- [Configuration](#configuration)
- [Testing](#testing)
- [Dependencies](#dependencies)
- [CI/CD](#cicd)
- [Security Considerations](#security-considerations)
- [Performance Optimization](#performance-optimization)
- [Contributing](#contributing)
- [License](#license)

---

## Project Overview
lorem ipsum
---

## Technology Stack

### Core Technologies

| Category | Technology | Version | Purpose |
|----------|------------|---------|---------|
| **Language** | Rust | 1.75+ | Primary programming language |
| **Runtime** | Tokio | 1.x | Asynchronous runtime |
| **Web Framework** | Axum | 0.7.x | REST API framework |
| **Database** | PostgreSQL | 14+ | Primary data store |
| **Cache** | Redis | 7+ | Caching and pub/sub |
| **ORM** | SQLx | 0.7.x | Type-safe database access |

### Supporting Libraries

| Category | Library | Purpose |
|----------|---------|---------|
| **Serialization** | Serde, serde_json | JSON serialization/deserialization |
| **Validation** | Validator | Request validation |
| **Error Handling** | thiserror, anyhow | Error propagation |
| **Logging** | tracing, tracing-subscriber | Structured logging |
| **Configuration** | config, dotenv | Environment configuration |
| **HTTP Client** | reqwest | External HTTP requests |
| **Date/Time** | chrono | Date/time handling |
| **Unique IDs** | uuid | Unique identifier generation |

---

## Project Structure

```
yomu-backend-rust/
├── Cargo.toml                 # Rust package manifest and dependencies
├── Cargo.lock                 # Dependency lock file
├── .env.example               # Environment variables template
├── .env                       # Local environment variables (not committed)
├── .gitignore                 # Git ignore patterns
├── README.md                  # This file
├── Dockerfile                 # Production Docker image
├── Dockerfile.dev             # Development Docker image with hot reload
├── docker-compose.yml         # Docker Compose orchestration
├── rustfmt.toml               # Rust formatter configuration
├── .clippy.toml               # Clippy linter configuration
├── .github/
│   └── workflows/
│       └── ci.yml             # GitHub Actions CI/CD pipeline
│
└── src/
    ├── main.rs                    # Application entry point
    ├── lib.rs                     # Library root exports
    ├── config/                    # Configuration management
    │   ├── mod.rs
    │   └── settings.rs            # Environment & connection settings
    │
    ├── shared/                    # Shared Kernel (cross-module utilities)
    │   ├── mod.rs
    │   ├── domain/               # Base types/structs for Domain layer
    │   │   ├── mod.rs
    │   │   ├── entity.rs         # Base entity trait and implementations
    │   │   └── value_object.rs   # Value object utilities
    │   │
    │   └── utils/                 # Common helpers
    │       ├── mod.rs
    │       ├── error.rs           # Shared error types
    │       └── response.rs        # API response utilities
    │
    └── modules/                   # BOUNDED CONTEXTS (Feature modules)
        ├── mod.rs                 # Module exports
        │
        ├── league/               # Clan & League module
        │   ├── mod.rs
        │   ├── domain/           # League domain layer
        │   │   ├── mod.rs
        │   │   ├── entities/     # Clan, Member entities
        │   │   ├── value_objects/# ClanId, MemberRole
        │   │   └── ports/        # Repository traits
        │   ├── application/      # League application layer
        │   │   ├── mod.rs
        │   │   ├── commands/    # Write operations
        │   │   ├── queries/     # Read operations
        │   │   └── dtos/         # Data transfer objects
        │   ├── infrastructure/  # League infrastructure layer
        │   │   ├── mod.rs
        │   │   ├── persistence/ # SQLx adapters
        │   │   └── cache/        # Redis adapters
        │   └── presentation/     # League presentation layer
        │       ├── mod.rs
        │       ├── controllers/ # Axum handlers
        │       └── routes/      # Route definitions
        │
        ├── gamification/         # Achievements & Missions module
        │   ├── mod.rs
        │   ├── domain/
        │   │   ├── mod.rs
        │   │   ├── entities/    # Achievement, Mission, Reward
        │   │   └── ports/
        │   ├── application/
        │   │   ├── mod.rs
        │   │   ├── commands/
        │   │   └── dtos/
        │   ├── infrastructure/
        │   │   └── persistence/
        │   └── presentation/
        │       ├── controllers/
        │       └── routes/
        │
        └── user_sync/            # User Synchronization module
            ├── mod.rs
            ├── domain/
            │   ├── mod.rs
            │   ├── entities/    # ShadowUser entity
            │   └── ports/
            ├── application/
            │   ├── mod.rs
            │   ├── commands/
            │   └── dtos/
            ├── infrastructure/
            │   └── persistence/
            └── presentation/
                ├── controllers/
                └── routes/
```

### Directory Structure Explanation

The project follows a **Feature-Based (Modular)** directory structure that organizes code by business domain (bounded context) rather than technical layer. This approach provides:

1. **Encapsulation**: Each module is self-contained with its own domain logic
2. **Clear Boundaries**: Module dependencies are explicit and controlled
3. **Scalability**: New features can be added as new modules
4. **Team Independence**: Teams can work on different modules without conflicts

Within each module, we apply **Hexagonal + Clean Architecture** (Ports & Adapters) with four distinct layers:

| Layer | Purpose | Dependencies |
|-------|---------|--------------|
| `domain/` | Pure business logic, entities, value objects, ports (traits) | None (zero external dependencies) |
| `application/` | Use cases, DTOs, command/query handlers | Domain layer only |
| `infrastructure/` | Adapters for external services (DB, cache, HTTP) | Application layer + external crates |
| `presentation/` | HTTP controllers and route definitions | Application layer + web framework |


## Getting Started

### Clone the Repository

```bash
# Clone the repository
git clone https://github.com/your-org/yomu-backend-rust.git

# Navigate to project directory
cd yomu-backend-rust
```

### Environment Setup

```bash
# 1. Copy the example environment file
cp .env.example .env

# 2. Review and update .env with your settings
# The default values work with docker-compose out of the box
nano .env  # or your preferred editor
```

### Environment Variables Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `APP_ENV` | Application environment (development, production) | `development` |
| `APP_HOST` | Server bind address | `0.0.0.0` |
| `APP_PORT` | Server port | `8080` |
| `DATABASE_URL` | PostgreSQL connection string | `postgres://yomu:yomu_password@postgres:5432/yomu_engine` |
| `REDIS_URL` | Redis connection string | `redis://redis:6379` |
| `LOG_LEVEL` | Logging verbosity (trace, debug, info, warn, error) | `info` |
| `RUST_BACKTRACE` | Enable backtraces (0 or 1) | `1` |

---

## How to Run

There are three ways to run this project, each suited for different development scenarios.

### Option 1: Full Docker (Production Mode)

Best for testing the final build in an isolated, production-like environment. This option builds the application as a static binary and runs it in a minimal container.

```bash
# Build and start all services (PostgreSQL, Redis, Application)
docker compose up -d

# Verify all containers are running
docker compose ps

# View application logs
docker compose logs -f app

# Stop all services
docker compose down
```

**Use this when:**
- You want to test the production build
- You need an isolated environment
- You don't want to install Rust locally

### Option 2: Full Dev Docker (Hot Reload)

Best if you don't want to install Rust on your local machine but want hot-reload functionality during development. This uses `cargo-watch` inside the container.

```bash
# Start all services with hot reload enabled
docker compose up -d dev

# View dev logs (watch for code changes to trigger rebuilds)
docker compose logs -f dev

# Stop services
docker compose down
```

**Use this when:**
- You don't have Rust installed
- You want hot-reload without local setup
- You need consistent development environment across team

### Option 3: Hybrid Development (Recommended)

Best for fast compilation cycles and full IDE support (intellisense, go-to-definition, refactoring). We run infrastructure (DB, Redis) in Docker, but run the Rust application natively.

**Prerequisites:** Rust 1.75+, Docker, and Docker Compose

```bash
# 1. Copy the environment file (if you haven't already)
cp .env.example .env

# 2. IMPORTANT: Update .env database credentials to match docker-compose!
# Change to: DATABASE_URL=postgres://yomu:yomu_password@localhost:5432/yomu_engine
# Change to: REDIS_URL=redis://localhost:6379

# 3. Start ONLY the infrastructure services (Database & Redis)
docker compose up -d postgres redis

# 4. Wait a few seconds for services to be ready
# Then run the Rust server locally
cargo run

# Or run with hot-reload (requires cargo-watch)
# Install cargo-watch first if needed
cargo install cargo-watch
cargo watch -x run

# 5. Access the application
# Open http://localhost:8080/health in your browser
```

**Use this when:**
- You have Rust installed
- You want fast compile times
- You need full IDE support
- You want to debug with breakpoints

### Verifying the Application

Once the application is running, verify it's working:

```bash
# Health check
curl http://localhost:8080/health

# Expected response:
# {"success":true,"message":"Server is running well","data":{"status":"healthy","version":"0.1.0"}}
```

---

## Development Workflow

### Daily Development Cycle

```bash
# 1. Start infrastructure (if not running)
docker compose up -d postgres redis

# 2. Run the application with hot reload
cargo watch -x run

# 3. Make code changes
# The application will automatically rebuild and restart

# 4. Run tests
cargo test

# 5. Check code quality
cargo fmt --all
cargo clippy --all -- -D warnings
```

### Working with Database Migrations

```bash
# Create a new migration
cargo sqlx migrate add create_clans_table

# Run migrations (automatically runs on startup in development)
# Or run manually:
cargo sqlx database setup

# Revert last migration
cargo sqlx migrate revert
```

### Adding New Dependencies

```bash
# Add a new dependency
cargo add <crate-name>

# Add a dependency with specific version
cargo add <crate-name>@<version>

# Add a development dependency
cargo add --dev <crate-name>

# Update all dependencies
cargo update
```

---

## Useful Commands & Troubleshooting

### Linting & Formatting

```bash
# Format all code automatically
cargo fmt --all

# Check formatting without making changes
cargo fmt --all -- --check

# Run clippy for code quality checks and suggestions
cargo clippy --all -- -D warnings

# Run clippy with more thorough checks
cargo clippy --all --all-targets -- -D warnings

# Fix automatically fixable clippy warnings
cargo clippy --fix --allow-dirty
```

> **Note on `cargo fmt`:** If the formatter ignores your file, make sure the file is registered in the module tree. You must add `pub mod filename;` in the parent folder's `mod.rs`.

### Building & Running

```bash
# Build the project (debug mode)
cargo build

# Build for release (optimized)
cargo build --release

# Run with custom configuration
cargo run -- --help

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Troubleshooting Common Issues

#### Issue: "Connection refused" or "Password authentication failed"

**Cause:** Database credentials in your `.env` don't match docker-compose.yml

**Solution:**
1. Check your `.env` file
2. Ensure `DATABASE_URL` matches: `postgres://yomu:yomu_password@localhost:5432/yomu_engine`
3. Ensure `REDIS_URL` matches: `redis://localhost:6379`
4. Verify containers are running: `docker compose ps`

#### Issue: Server exits immediately on startup

**Cause:** Application is designed to "fail-fast". If it cannot connect to required services (Postgres, Redis), it shuts down to prevent hanging states.

**Solution:**
1. Check terminal logs for specific error
2. Ensure Docker containers are running: `docker compose ps`
3. Wait a few seconds for services to be fully ready
4. Try restarting services: `docker compose restart postgres redis`

#### Issue: Port already in use

**Cause:** Another process is using port 8080

**Solution:**
```bash
# Find what's using port 8080
lsof -i :8080

# Kill the process (replace PID with actual process ID)
kill <PID>

# Or use a different port by updating .env
```

#### Issue: Migration fails

**Cause:** Database schema mismatch or connection issues

**Solution:**
```bash
# Drop and recreate database (WARNING: loses all data)
cargo sqlx database drop
cargo sqlx database create

# Or run specific migration
cargo sqlx migrate run --ignore-missing
```

#### Issue: Clippy warnings about style

**Cause:** Code style doesn't match project conventions

**Solution:**
```bash
# Run formatter first
cargo fmt --all

# Then run clippy
cargo clippy --all -- -D warnings
```

---

## API Endpoints

### Core Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| GET | `/health` | Health check and service status | No |
| GET | `/health/ready` | Readiness probe (includes DB check) | No |
| GET | `/health/live` | Liveness probe | No |

### League Module Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| GET | `/api/v1/clans` | List all clans | Yes |
| POST | `/api/v1/clans` | Create a new clan | Yes |
| GET | `/api/v1/clans/:id` | Get clan details | Yes |
| PUT | `/api/v1/clans/:id` | Update clan details | Yes |
| DELETE | `/api/v1/clans/:id` | Delete a clan | Yes |
| POST | `/api/v1/clans/:id/join` | Join a clan | Yes |
| POST | `/api/v1/clans/:id/leave` | Leave a clan | Yes |
| GET | `/api/v1/clans/:id/members` | List clan members | Yes |
| GET | `/api/v1/leaderboard` | Get global leaderboard | Yes |
| GET | `/api/v1/leaderboard/clan/:id` | Get clan leaderboard | Yes |

### Gamification Module Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| GET | `/api/v1/achievements` | List all achievements | Yes |
| GET | `/api/v1/achievements/:id` | Get achievement details | Yes |
| POST | `/api/v1/achievements/:id/claim` | Claim achievement reward | Yes |
| GET | `/api/v1/missions` | List all missions | Yes |
| GET | `/api/v1/missions/:id` | Get mission details | Yes |
| POST | `/api/v1/missions/:id/progress` | Update mission progress | Yes |
| GET | `/api/v1/user/:userId/progress` | Get user progress | Yes |

### User Sync Module Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| POST | `/api/v1/sync/users` | Sync user from Java Core | Internal |
| GET | `/api/v1/sync/users/:id` | Get shadow user | Internal |
| PUT | `/api/v1/sync/users/:id` | Update shadow user | Internal |
| DELETE | `/api/v1/sync/users/:id` | Remove shadow user | Internal |

---

## Architecture

This project uses a **Module-Based (Feature-Based) Structure** combined with **Hexagonal Architecture** (also known as Ports and Adapters). This architectural approach provides excellent separation of concerns and makes the codebase highly testable and maintainable.

### Why Modular + Hexagonal?

#### Traditional Layered Architecture (Not Used)
```
┌─────────────────────────────────────┐
│         Presentation Layer          │  ← Controllers, Routes
├─────────────────────────────────────┤
│         Application Layer           │  ← Use Cases, DTOs
├─────────────────────────────────────┤
│           Domain Layer              │  ← Entities, Business Rules
├─────────────────────────────────────┤
│        Infrastructure Layer         │  ← DB, Cache, External APIs
└─────────────────────────────────────┘
```

**Problem**: As the application grows, the domain layer gets tangled with infrastructure concerns.

#### Modular + Hexagonal Architecture (Used)
```
┌─────────────────────────────────────────────────────────┐
│                    League Module                        │
│  ┌─────────────────────────────────────────────────┐    │
│  │                  Domain Layer                   │    │
│  │   Entities │ Value Objects │ Ports (Traits)     │    │
│  └─────────────────────────────────────────────────┘    │
│  ┌────────────────┐  ┌────────────────────────────┐     │
│  │ Application    │  │        Ports (Interfaces)  │     │
│  │ Layer          │◄─┤  ┌────────────┐ ┌────────┐ │     │
│  │ Commands/Queries│ │ ClanRepository│Leaderboard │     │
│  └────────────────┘  │  └────────────┘ └────────┘ │     │
│                      └────────────────────────────┘     │
│  ┌─────────────────────────────────────────────────┐    │
│  │              Infrastructure Layer               │    │
│  │  SQLx Adapter │ Redis Adapter │ HTTP Client     │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Benefit**: Each module is a self-contained bounded context with its own hexagonal architecture. Dependencies point inward toward the domain.

### Layer Responsibilities

| Layer | Description | Dependencies |
|-------|-------------|--------------|
| `domain/` | **Pure business logic**. Contains entities, value objects, and port definitions (traits). This layer has ZERO external dependencies and could theoretically be extracted into a separate crate. | None |
| `application/` | **Use cases and orchestration**. Contains command and query handlers, DTOs, and application-specific logic. Depends only on the domain layer. | Domain layer |
| `infrastructure/` | **External integrations**. Contains adapters for PostgreSQL, Redis, HTTP clients, etc. Implements the ports defined in the domain layer. | Application layer + external crates |
| `presentation/` | **HTTP delivery**. Contains Axum controllers, request/response handlers, and route definitions. Depends on the application layer. | Application layer + Axum |

### Dependency Flow

```
         ┌──────────────┐
         │ Presentation │  (depends on)
         └──────┬───────┘
                │
         ┌──────▼───────┐
         │ Application  │  (depends on)
         └──────┬───────┘
                │
         ┌──────▼───────┐
         │    Domain    │  (defines interfaces for)
         └──────┬───────┘
                │
         ┌──────▼───────┐
         │Infrastructure│  (implements interfaces from)
         └──────────────┘
```

### Key Design Patterns

1. **Repository Pattern**: Abstraction over data persistence
2. **Unit of Work**: Transaction management
3. **CQRS**: Separate command and query models for read/write operations
4. **Dependency Injection**: Traits injected at startup
5. **Result Type**: Explicit error handling with `Result<T, E>`

---



## Configuration

### Configuration Files

| File | Purpose |
|------|---------|
| `.env` | Environment variables (local development) |
| `.env.example` | Template for environment variables |
| `rustfmt.toml` | Code formatting rules |
| `.clippy.toml` | Linter configuration |
| `docker-compose.yml` | Container orchestration |

### Configuration Management

The application uses the `config` crate for hierarchical configuration:

1. **Default values** in `config/settings.rs`
2. **Environment variables** override defaults
3. **Secret values** come from `.env` file

### Running with Custom Configuration

```bash
# Use specific environment
APP_ENV=production cargo run

# Override specific settings
cargo run -- --host 127.0.0.1 --port 9000
```

---

## Testing

### Running Tests

```bash
cargo test
```

---

## Dependencies

### Production Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| axum | 0.7 | Web framework |
| tower | 0.4 | HTTP middleware |
| tower-http | 0.5 | HTTP utilities |
| tokio | 1 | Async runtime |
| sqlx | 0.7 | Database ORM |
| sqlx-runtime-tokio | 0.7 | SQLx Tokio runtime |
| redis | 0.25 | Redis client |
| serde | 1 | Serialization |
| serde_json | 1 | JSON handling |
| validator | 0.16 | Request validation |
| thiserror | 1 | Error handling |
| anyhow | 1 | Error handling |
| tracing | 0.1 | Logging |
| tracing-subscriber | 0.3 | Logging subscriber |
| config | 0.14 | Configuration |
| dotenv | 0.15 | Environment variables |
| chrono | 0.4 | Date/time |
| uuid | 1 | Unique IDs |
| reqwest | 0.11 | HTTP client |

### Development Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| cargo-watch | 8 | Hot reload |
| cargo-fmt | 0.6 | Code formatting |
| clippy | 0.1 | Linting |
| sqlx-cli | 0.7 | Database migrations |
| tokio-test | 1 | Async testing |
| mockall | 0.11 | Mocking |
| rstest | 0.18 | Parametrized tests |

---

## CI/CD

The project uses GitHub Actions for continuous integration and deployment. The workflow is defined in `.github/workflows/ci.yml`.

### Pipeline Stages

1. **Lint**
   - Code formatting check (`cargo fmt`)
   - Linting with Clippy (`cargo clippy`)
   - Security audit (`cargo audit`)

2. **Test**
   - Unit tests (`cargo test`)
   - Integration tests
   - Code coverage (minimum 80%)

3. **Build**
   - Debug build check
   - Release build compilation

4. **Docker**
   - Build production image
   - Build development image
   - Push to container registry (on main branch)

### Running CI Locally

```bash
# Simulate CI checks locally
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
cargo test
cargo build --release
```

---
