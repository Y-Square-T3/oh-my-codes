# src/cli/ — CLI: install, run, doctor, mcp-oauth, account

**Generated:** 2026-04-18

## OVERVIEW

Commander.js CLI with 8 commands. Entry: `index.ts` → `runCli()` in `cli-program.ts`.

## COMMANDS

| Command                      | Purpose                           | Key Logic                                             |
| ---------------------------- | --------------------------------- | ----------------------------------------------------- |
| `install`                    | Interactive setup                 | Provider selection → config gen → plugin registration |
| `run <message>`              | Non-interactive session launcher  | Agent resolution (flag → env → config → Sisyphus)     |
| `doctor`                     | 4-category health checks          | System, Config, Tools, Models                         |
| `get-local-version`          | Version detection                 | Installed vs npm latest                               |
| `mcp-oauth`                  | OAuth token management            | login (PKCE), logout, status                          |
| `refresh-model-capabilities` | Refresh models.dev cache          | Model capabilities refresh                            |
| `account login <url>`        | Device code flow login            | OAuth device flow → store account                     |
| `account logout [email]`     | Remove account                    | Remove account + tokens from db                       |
| `account switch`             | Switch active workspace           | Interactive workspace selection                       |
| `account list`               | List logged-in accounts           | Display accounts with active indicator                |

## STRUCTURE

```
cli/
├── index.ts                     # Entry point → runCli()
├── cli-program.ts               # Commander.js program (8 commands)
├── install.ts                   # Routes to TUI installer
├── tui-installer.ts             # Interactive (@clack/prompts)
├── model-fallback.ts            # Model config gen by provider availability
├── provider-availability.ts     # Provider detection
├── fallback-chain-resolution.ts # Fallback chain logic
├── config-manager/              # 20 config utilities
│   ├── plugin registration, provider config
│   ├── JSONC operations, auth plugins
│   └── npm dist-tags, binary detection
├── doctor/
│   ├── runner.ts                # Parallel check execution
│   ├── formatter.ts             # Output formatting
│   └── checks/                  # 15 check files in 4 categories
│       ├── system.ts            # Binary, plugin, version
│       ├── config.ts            # JSONC validity, Zod schema
│       ├── tools.ts             # AST-Grep, LSP, GH CLI, MCP
│       └── model-resolution.ts  # Cache, resolution, overrides (6 sub-files)
├── run/                         # Session launcher
│   ├── runner.ts                # Main orchestration
│   ├── agent-resolver.ts        # Flag → env → config → Sisyphus
│   ├── session-resolver.ts      # Create/resume sessions
│   ├── event-handlers.ts        # Event processing
│   └── poll-for-completion.ts   # Wait for todos/background tasks
├── mcp-oauth/                   # OAuth token management
└── account/                     # Workspace account management (Effect.js)
    ├── index.ts                 # createAccountCommand() → Commander registration
    ├── account.ts               # Account.Service (Effect Context.Service)
    ├── repo.ts                  # AccountRepo using Database.Service
    ├── login.ts                 # Device code flow login + workspace picker
    ├── logout.ts                # Remove account(s)
    ├── switch.ts                # Switch active workspace
    ├── list.ts                  # List accounts + workspaces
    ├── api.ts (inline)          # HTTP API (device code, poll, me, workspaces)
    ├── schema.ts                # Effect Schema types (AccountID, Login, PollResult, etc.)
    ├── url.ts                   # normalizeServerUrl()
    └── ui.ts                    # CLI rendering (clack prompts, picocolors)
```

## DATABASE FEATURE

Shared SQLite database feature at `src/features/database/`:

- Uses `bun:sqlite` (no external dependency)
- Effect `Context.GenericTag<DatabaseService>` for dependency injection
- DB path: `{configDir}/oh-my-codes.db` (respects `OPENCODE_CONFIG_DIR`)
- Migration system tracks applied migrations
- Initial migration creates `accounts` + `account_state` tables

## MODEL FALLBACK SYSTEM

No single global priority. CLI install-time resolution uses per-agent fallback chains from `model-fallback-requirements.ts`.

Common patterns: Claude/OpenAI/Gemini are preferred when an agent chain includes them, `librarian` prefers ZAI, `sisyphus` falls back through Kimi then GLM-5, and `hephaestus` requires OpenAI-compatible providers.

## DOCTOR CHECKS

| Category   | Validates                                                              |
| ---------- | ---------------------------------------------------------------------- |
| **System** | Binary found, version >=1.0.150, plugin registered, version match      |
| **Config** | JSONC validity, Zod schema, model override syntax                      |
| **Tools**  | AST-Grep, comment-checker, LSP servers, GH CLI, MCP servers            |
| **Models** | Cache exists, model resolution, agent/category overrides, availability |

## HOW TO ADD A DOCTOR CHECK

1. Create `src/cli/doctor/checks/{name}.ts`
2. Export check function matching `DoctorCheck` interface
3. Register in `checks/index.ts`

## ACCOUNT COMMAND

The `account` CLI command provides workspace account management using OAuth 2.0 device code flow:

```bash
bunx oh-my-codes account login <server-url>   # Log in via device code flow
bunx oh-my-codes account logout [email]       # Log out (interactive if no email)
bunx oh-my-codes account switch               # Switch active workspace
bunx oh-my-codes account list                 # List all accounts
```

### Architecture

- **Effect.js** v3 for typed error handling and dependency injection
- **SQLite** via `bun:sqlite` (stored in `~/.config/opencode/oh-my-codes.db`)
- **HTTP** via native `fetch()` wrapped in Effect.tryPromise
- **Schema** via Effect Schema for branded types and decode/encode
- **UI** via `@clack/prompts` (existing dependency) and `picocolors` (existing)

### Layer Composition

```
Account.defaultLayer
  └─ AccountRepo.layer
       └─ Database.defaultLayer (bun:sqlite)
```

Each CLI command runs via `Effect.runPromiseExit(effect.pipe(Effect.provide(Account.defaultLayer)))`.
