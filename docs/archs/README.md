# oh-my-codes Architecture

## Overview

oh-my-codes is a Rust-based daemon and CLI tool with an embedded SurrealDB database. It provides account management, workspace switching, and chat functionality.

## Components

- **omc-core**: Shared types, configuration, and error handling
- **omc-api**: API types and HTTP client SDK
- **omc-storage**: Storage abstraction with SurrealDB embedded backend
- **omc-server**: HTTP server (axum-based) with account service
- **omc-service**: OS service management (launchd/systemd/Task Scheduler)
- **omc**: CLI binary
- **omcd**: Daemon binary

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CLI (omc)                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │  Config  │  │  Health  │  │  Daemon  │  │ Account  │               │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘               │
└─────────────────────────────────────────────────────────────────────────┘
                                    │ HTTP/Unix Socket
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Daemon (omcd)                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    HTTP Server (omc-server)                       │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │  │
│  │  │ Health  │ │ Config  │ │ Channel │ │ Message │ │ Account │  │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                  AccountService (omc-server)                      │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │  │
│  │  │    login     │  │     poll     │  │   refresh_token      │   │  │
│  │  └──────────────┘  └──────────────┘  └──────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                   Storage Layer (omc-storage)                     │  │
│  │  ┌─────────────────┐  ┌─────────────────────────────────────┐   │  │
│  │  │  MemoryStorage  │  │   SurrealDB (RocksDB)               │   │  │
│  │  │                 │  │  ┌─────────┐ ┌───────────────────┐  │   │  │
│  │  │                 │  │  │ Account │ │ Workspace         │  │   │  │
│  │  │                 │  │  └─────────┘ └───────────────────┘  │   │  │
│  │  └─────────────────┘  └─────────────────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ HTTPS
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         OMC Server (Remote)                              │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────┐  │
│  │ /auth/device/    │  │ /api/me          │  │ /api/workspaces      │  │
│  │ code             │  │                  │  │                      │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Data Flow

1. CLI sends HTTP requests to daemon via Unix socket or TCP
2. Daemon routes requests to appropriate handlers
3. Handlers interact with storage layer (SurrealDB)
4. Results are serialized and returned to CLI

## Account System

The account system enables OAuth 2.0 Device Code Flow authentication with the OMC server. Key features:

- **Multi-account support**: Store and switch between multiple accounts
- **Workspace management**: Cache and switch between workspaces per account
- **Token management**: Automatic token refresh with 5-minute eager threshold
- **CLI commands**: `login`, `logout`, `switch`, `list`

See [Account System Architecture](./account-system.md) for detailed documentation.

## Configuration

Configuration is loaded from:
1. Command-line arguments (highest priority)
2. Environment variables
3. Project config (`.omc/omc.json`)
4. User config (`~/.config/omc/omc.json`)

## Storage

- **SurrealDB**: Embedded database using RocksDB backend
- Data stored in `~/.local/share/omc/data/omc.db/`
- Tables:
  - `channel` - Chat channels
  - `message` - Chat messages
  - `account` - User accounts with OAuth tokens
  - `workspace` - Cached workspace data per account
  - `active_account` - Singleton tracking active account
