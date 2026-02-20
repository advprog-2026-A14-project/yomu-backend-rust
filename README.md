# Yomu Engine Rust

Clean Architecture + Hexagonal Architecture + DDD + SOLID backend service for Yomu.

## Project Structure

```
yomu-engine-rust/
├── Cargo.toml
├── .env.example
├── .gitignore
├── README.md
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

## Architectural Overview

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

## Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- Redis 7+

## Getting Started

1. Copy environment file:
   ```bash
   cp .env.example .env
   ```

2. Update `.env` with your database credentials

3. Run the server:
   ```bash
   cargo run
   ```

## Dependencies

- **Web**: Axum, Tower, Tower-HTTP
- **Database**: SQLx (PostgreSQL), Redis
- **Async**: Tokio
- **Serialization**: Serde
- **Error Handling**: Thiserror, Anyhow
- **Logging**: Tracing

## Development

```bash
# Watch mode
cargo watch -x run
```
