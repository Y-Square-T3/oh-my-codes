# Account System Architecture

## Overview

The account system enables users to authenticate with the OMC server using OAuth 2.0 Device Code Flow, manage multiple accounts, and switch between workspaces. This feature was migrated from the legacy TypeScript implementation to Rust.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CLI (omc)                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │  login   │  │  logout  │  │  switch  │  │   list   │               │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘               │
└─────────────────────────────────────────────────────────────────────────┘
                                    │ HTTP/Unix Socket
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Daemon (omcd)                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    HTTP Server (omc-server)                       │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │  │
│  │  │  /account/ │ │  /account/ │ │  /account/ │ │  /account/ │   │  │
│  │  │   login    │ │   poll     │ │  switch    │ │   list     │   │  │
│  │  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                  AccountService (omc-server)                      │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │  │
│  │  │    login     │  │     poll     │  │   refresh_token      │   │  │
│  │  │              │  │              │  │   resolve_token      │   │  │
│  │  └──────────────┘  └──────────────┘  └──────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                  OmcServerClient (omc-server)                     │  │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐  │  │
│  │  │ request_device_  │  │ poll_device_     │  │ fetch_user    │  │  │
│  │  │ code             │  │ token            │  │ fetch_        │  │  │
│  │  │                  │  │                  │  │ workspaces    │  │  │
│  │  └──────────────────┘  └──────────────────┘  └───────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                   Storage Layer (omc-storage)                     │  │
│  │  ┌─────────────────────┐  ┌─────────────────────────────────┐   │  │
│  │  │  AccountStore       │  │  WorkspaceStore                 │   │  │
│  │  │  (SQLite)           │  │  (SQLite)                       │   │  │
│  │  └─────────────────────┘  └─────────────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ HTTPS
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         OMC Server (Remote)                              │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────┐  │
│  │ /auth/device/    │  │ /auth/device/    │  │ /api/me              │  │
│  │ code             │  │ token            │  │ /api/workspaces      │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Data Model

### SQLite Schema

```sql
-- Account table (stores credentials and metadata)
CREATE TABLE account (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    url TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    token_expiry INTEGER NOT NULL,
    active_workspace_id TEXT
);

-- Workspace table (cached workspace data)
CREATE TABLE workspace (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    is_admin INTEGER NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account(id)
);
CREATE INDEX idx_workspace_account ON workspace(account_id);

-- Active account state (singleton)
CREATE TABLE active_account (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    account_id TEXT,
    FOREIGN KEY (account_id) REFERENCES account(id)
);
```

### Domain Types (omc-core)

```rust
// Full account with credentials (stored in SQLite)
pub struct Account {
    pub id: String,
    pub email: String,
    pub url: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiry: i64,
    pub active_workspace_id: Option<String>,
}

// Safe view without secrets (returned to CLI)
pub struct AccountInfo {
    pub id: String,
    pub email: String,
    pub url: String,
    pub active_workspace_id: Option<String>,
}

// Cached workspace
pub struct Workspace {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub is_admin: bool,
}
```

## Authentication Flow

### OAuth 2.0 Device Code Flow

```
┌─────────┐         ┌─────────┐         ┌─────────────┐
│   CLI   │         │  Daemon │         │ OMC Server  │
└────┬────┘         └────┬────┘         └──────┬──────┘
     │                   │                     │
     │  POST /account/   │                     │
     │  login {url}      │                     │
     │──────────────────>│                     │
     │                   │  POST /auth/device/ │
     │                   │  code               │
     │                   │────────────────────>│
     │                   │  {device_code,      │
     │                   │   user_code,        │
     │                   │   verification_uri} │
     │                   │<────────────────────│
     │  {device_code,    │                     │
     │   user_code,      │                     │
     │   verification_   │                     │
     │   uri}            │                     │
     │<──────────────────│                     │
     │                   │                     │
     │  Open browser     │                     │
     │  Display code     │                     │
     │                   │                     │
     │  POST /account/   │                     │
     │  poll             │                     │
     │──────────────────>│                     │
     │                   │  POST /auth/device/ │
     │                   │  token              │
     │                   │────────────────────>│
     │                   │  {access_token,     │
     │                   │   refresh_token}    │
     │                   │<────────────────────│
     │                   │  GET /api/me        │
     │                   │────────────────────>│
     │                   │  {id, email}        │
     │                   │<────────────────────│
     │                   │  GET /api/          │
     │                   │  workspaces         │
     │                   │────────────────────>│
     │                   │  [{id, name}]       │
     │                   │<────────────────────│
      │                   │  Store in SQLite    │
     │  {email}          │                     │
     │<──────────────────│                     │
     │                   │                     │
```

### Token Management

- **Eager Refresh**: Tokens are refreshed 5 minutes before expiry (`EAGER_REFRESH_SECS = 300`)
- **Auto-Refresh**: `resolve_token()` checks freshness and refreshes if needed
- **Storage**: Tokens stored in plaintext in SQLite (future: OS keychain integration)

## Component Responsibilities

### omc-core

- **account.rs**: Domain types (`Account`, `AccountInfo`, `Workspace`)
- **url.rs**: URL normalization utility
- **error.rs**: Error variants (`Auth`, `TokenExpired`)

### omc-storage

- **account_store.rs**: `AccountStore` trait for account CRUD operations
- **workspace_store.rs**: `WorkspaceStore` trait for workspace CRUD operations
- **sqlite.rs**: SQLite implementations (`SqliteAccountStore`, `SqliteWorkspaceStore`)

### omc-server

- **server_client.rs**: HTTP client for OMC server endpoints
  - `request_device_code()` - Initiate device code flow
  - `poll_device_token()` - Poll for token
  - `refresh_token()` - Refresh access token
  - `fetch_user()` - Get user info
  - `fetch_workspaces()` - Get workspace list

- **account_service.rs**: Business logic
  - `login()` - Initiate login, return device code info
  - `poll()` - Poll for token, fetch user/workspaces, persist to DB
  - `refresh_token()` - Force token refresh
  - `resolve_token()` - Get fresh token (auto-refresh if needed)
  - `active()` - Get active account
  - `list()` - List all accounts with workspaces
  - `switch()` - Set active account/workspace
  - `remove()` - Delete account and associated data

- **routes/account.rs**: HTTP route handlers
  - `POST /account/login` - Initiate login
  - `POST /account/poll` - Poll for token
  - `GET /account/active` - Get active account
  - `GET /account/list` - List accounts
  - `POST /account/switch` - Switch workspace
  - `POST /account/remove` - Remove account
  - `GET /account/workspaces` - Get workspaces for account

### omc-api

- **client.rs**: Daemon client methods
  - `account_login()` - Call login endpoint
  - `account_poll()` - Call poll endpoint
  - `account_active()` - Get active account
  - `account_list()` - List accounts
  - `account_switch()` - Switch workspace
  - `account_remove()` - Remove account
  - `account_workspaces()` - Get workspaces

- **types.rs**: Request/response types for daemon API

### omc (CLI)

- **main.rs**: CLI commands
  - `omc account login <url>` - Device code flow with browser + spinner
  - `omc account logout [email]` - Remove account (interactive picker)
  - `omc account switch` - Interactive workspace selector
  - `omc account list` - Display accounts and workspaces

### omcd (Daemon)

- **main.rs**: Wires up account stores and service
  - Creates `SqliteAccountStore` and `SqliteWorkspaceStore`
  - Creates `OmcServerClient` and `AccountService`
  - Passes `AccountService` to `DaemonState`

## API Endpoints

### Daemon API (CLI → Daemon)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/account/login` | POST | Initiate device code flow |
| `/account/poll` | POST | Poll for authentication token |
| `/account/active` | GET | Get active account |
| `/account/list` | GET | List all accounts with workspaces |
| `/account/switch` | POST | Switch active workspace |
| `/account/remove` | POST | Remove account |
| `/account/workspaces` | GET | Get workspaces for account |

### OMC Server API (Daemon → Server)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/auth/device/code` | POST | Request device code |
| `/auth/device/token` | POST | Exchange device code for token |
| `/api/me` | GET | Get current user info |
| `/api/workspaces` | GET | List user's workspaces |

## CLI Commands

### Login

```bash
omc account login <url>
```

1. Normalizes server URL
2. Requests device code from OMC server
3. Opens browser to verification URL
4. Displays user code for manual entry
5. Polls for token with spinner
6. On success: fetches user info and workspaces
7. Auto-selects workspace if only one exists
8. Sets account as active

### Logout

```bash
omc account logout [email]
```

- If email provided: removes that account directly
- If no email: shows interactive picker to select account
- Clears active state if removed account was active
- Deletes account and associated workspaces from DB

### Switch

```bash
omc account switch
```

- Lists all accounts and workspaces
- Shows interactive picker with active marker
- Updates active account and workspace in DB

### List

```bash
omc account list
```

- Displays all accounts with their workspaces
- Marks active workspace with `*`
- Shows account email and server URL

## Security Considerations

### Current Implementation

- **Tokens stored in plaintext** in SQLite
- **No encryption** at rest
- **Tokens transmitted** over HTTPS to OMC server

### Future Improvements

- **OS Keychain Integration**: Use `keyring` crate for secure token storage
- **Token Encryption**: Encrypt tokens before storing in DB
- **Token Rotation**: Implement automatic token refresh in background

## Dependencies

### New Dependencies Added

| Crate | Version | Purpose |
|-------|---------|---------|
| `url` | 2 | URL parsing and normalization |
| `dialoguer` | 0.11 | Interactive CLI prompts |
| `indicatif` | 0.17 | Progress spinners |
| `open` | 5 | Cross-platform browser opener |

### Modified Dependencies

- `omc-server`: Added `reqwest` for HTTP client
- `omc`: Added `dialoguer`, `indicatif`, `open`

## Migration Notes

This implementation was migrated from the legacy TypeScript codebase (`oh-my-codes-legacy`). Key differences:

1. **Storage**: Migrated from SurrealDB to SQLite
2. **Architecture**: CLI no longer accesses DB directly; all operations go through daemon
3. **Token Storage**: Currently plaintext; future: OS keychain
4. **Error Handling**: Uses Rust's `thiserror` instead of Effect-TS typed errors
5. **Type Safety**: Rust's type system provides compile-time guarantees vs runtime validation

## Testing

The implementation passes all checks:
- `cargo check --workspace` - Compilation
- `cargo test --workspace` - Unit tests
- `cargo clippy --workspace --all-targets -- -D warnings` - Linting
- `cargo fmt --all --check` - Formatting

## Future Enhancements

1. **Background Token Refresh**: Daemon refreshes tokens proactively
2. **Multi-Server Support**: Better handling of multiple OMC servers
3. **Account Sync**: Sync accounts across devices
4. **Audit Integration**: Track token usage per account
5. **Model Discovery**: Fetch available models per workspace
