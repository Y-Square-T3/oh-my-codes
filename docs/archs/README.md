# oh-my-codes Architecture

## Overview

oh-my-codes is a Rust-based daemon and CLI tool for managing code repositories with an embedded SurrealDB database.

## Components

- **omc-core**: Shared types, configuration, and error handling
- **omc-api**: API types and HTTP client SDK
- **omc-storage**: Storage abstraction with SurrealDB embedded backend
- **omc-server**: HTTP server (axum-based)
- **omc-service**: OS service management (launchd/systemd/Task Scheduler)
- **omc-tui**: Terminal UI (ratatui-based)
- **omc**: CLI binary
- **omcd**: Daemon binary

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI (omc)                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  Config  │  │   Repo   │  │  Health  │  │  Daemon  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└─────────────────────────────────────────────────────────────┘
                            │ HTTP/Unix Socket
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Daemon (omcd)                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              HTTP Server (omc-server)                 │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │  │
│  │  │ Health  │ │ Config  │ │  Repo   │ │ Channel │   │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘   │  │
│  └──────────────────────────────────────────────────────┘  │
│                            │                                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Storage Layer (omc-storage)                 │  │
│  │  ┌─────────────────┐  ┌─────────────────────────┐   │  │
│  │  │  MemoryStorage  │  │   SurrealDB (RocksDB)   │   │  │
│  │  └─────────────────┘  └─────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

1. CLI sends HTTP requests to daemon via Unix socket or TCP
2. Daemon routes requests to appropriate handlers
3. Handlers interact with storage layer (SurrealDB)
4. Results are serialized and returned to CLI

## Configuration

Configuration is loaded from:
1. Command-line arguments (highest priority)
2. Environment variables
3. Project config (`.omc/omc.json`)
4. User config (`~/.config/omc/omc.json`)

## Storage

- **SurrealDB**: Embedded database using RocksDB backend
- Data stored in `~/.local/share/omc/data/omc.db/`
- Supports channels and messages with full CRUD operations
