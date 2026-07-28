[![omc-ci](https://github.com/Y-Square-T3/oh-my-codes/actions/workflows/omc-ci.yml/badge.svg)](https://github.com/Y-Square-T3/oh-my-codes/actions/workflows/omc-ci.yml)
[![omc-opencode-ci](https://github.com/Y-Square-T3/oh-my-codes/actions/workflows/omc-opencode-ci.yml/badge.svg)](https://github.com/Y-Square-T3/oh-my-codes/actions/workflows/omc-opencode-ci.yml)
[![npm oh-my-codes version](https://img.shields.io/npm/v/oh-my-codes)](https://www.npmjs.com/package/oh-my-codes)
[![npm oh-my-codes-opencode version](https://img.shields.io/npm/v/oh-my-codes-opencode)](https://www.npmjs.com/package/oh-my-codes-opencode)
[![license](https://img.shields.io/github/license/Y-Square-T3/oh-my-codes)](./LICENSE)

# oh-my-codes

A CLI + daemon system for managing AI model accounts, workspaces, and token usage. Built with Rust for performance and distributed via npm with prebuilt platform-specific binaries.

## Features

- **OAuth 2.0 Device Code Flow** -- Browser-based authentication with automatic token refresh
- **Multi-Account & Multi-Workspace** -- Switch between accounts and workspaces with interactive CLI pickers
- **Background Daemon** -- OS-level service (launchd / systemd / Task Scheduler) with embedded SurrealDB storage
- **Model Discovery & Sync** -- List and sync available AI models from remote servers
- **Token Usage Tracking** -- Track, push, and summarize token usage with daily reports
- **Cross-Platform** -- Prebuilt binaries for macOS (arm64/x64), Linux (arm64/x64), and Windows (x64)

## Architecture

```
CLI (omc)  ──HTTP/Unix Socket──▶  Daemon (omcd)  ──HTTPS──▶  OMC Server
                                      │
                                  SurrealDB (RocksDB)
```

| Component | Description |
|-----------|-------------|
| `omc` | CLI binary -- command parsing and daemon communication |
| `omcd` | Daemon binary -- state management, storage, and server proxy |
| `omc-core` | Shared types, config, and error handling |
| `omc-api` | HTTP client SDK for CLI-to-daemon communication |
| `omc-storage` | Storage trait + SurrealDB embedded backend |
| `omc-server` | Axum HTTP server with route handlers |
| `omc-service` | OS service management (launchd / systemd / Task Scheduler) |

## Installation

```bash
npm install oh-my-codes
```

Or with Yarn:

```bash
yarn add oh-my-codes
```

## Quick Start

```bash
# Install and start the daemon as an OS service
omc daemon install
omc daemon start

# Authenticate with your OMC server
omc account login https://your-omc-server.com

# Select a workspace
omc account switch

# Explore available models
omc model list

# Check token usage
omc token-usage summary
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `omc account login <url>` | Authenticate via device code flow |
| `omc account logout [email]` | Remove an account |
| `omc account switch` | Switch active workspace/account |
| `omc account list` | List all accounts |
| `omc account show` | Show active account |
| `omc model list [--provider]` | List available models |
| `omc model sync` | Sync models from remote server |
| `omc token-usage status` | Check token usage status |
| `omc token-usage push` | Manually push token usage |
| `omc token-usage list [--limit]` | List usage records |
| `omc token-usage summary [--days]` | Daily usage summary |
| `omc config show` | Show resolved configuration |
| `omc config path` | Show config file path |
| `omc daemon install` | Install as OS service |
| `omc daemon uninstall` | Uninstall OS service |
| `omc daemon start` | Start daemon |
| `omc daemon stop` | Stop daemon |
| `omc daemon status` | Check daemon status |
| `omc health` | Health check |

## Configuration

Configuration is resolved in the following priority (highest to lowest):

1. Command-line arguments
2. Environment variables
3. Project config (`.omc/omc.json`)
4. User config (`~/.config/omc/omc.json`)

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024)
- [Node.js](https://nodejs.org/)
- [Yarn 4](https://yarnpkg.com/) (enable via `corepack enable`)

### Setup

```bash
corepack enable
yarn install
```

### Scripts

```bash
yarn omc build          # Compile binaries
yarn omc dev            # Run CLI (cargo run -p omc --)
yarn omc dev:daemon     # Run daemon (cargo run -p omcd --)
yarn omc test           # Run all workspace tests
yarn omc check          # Quick compilation check
yarn omc lint           # Clippy lints (deny warnings)
yarn omc lint:fix       # Clippy with auto-fix
yarn omc fmt            # Format all Rust code
yarn omc fmt:check      # Check formatting
```

## Project Structure

```
oh-my-codes/
├── packages/
│   ├── omc/                    # Rust workspace + npm package
│   │   ├── crates/
│   │   │   ├── omc/            # CLI binary
│   │   │   ├── omcd/           # Daemon binary
│   │   │   ├── omc-core/       # Shared types & config
│   │   │   ├── omc-api/        # HTTP client SDK
│   │   │   ├── omc-storage/    # SurrealDB storage backend
│   │   │   ├── omc-server/     # Axum HTTP server
│   │   │   └── omc-service/    # OS service management
│   │   ├── dist/               # Platform-specific packages
│   │   └── scripts/            # Build scripts
│   └── omc-opencode/           # OpenCode plugin
├── docs/                       # Architecture documentation
└── AGENTS.md                   # AI agent instructions
```

## Documentation

Detailed architecture documentation is available in [`docs/archs/`](./docs/archs/).

## Contributing

Contributions are welcome! Please make sure to:

1. Run `yarn omc fmt` to format code
2. Run `yarn omc lint` to check lints
3. Run `yarn omc test` to verify tests pass

## License

This project is licensed under the terms of the [LICENSE](./LICENSE) file.
