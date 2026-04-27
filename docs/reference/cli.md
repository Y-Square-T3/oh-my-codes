# CLI Reference

Complete reference for the published `oh-my-codes` CLI. OpenCode plugin registration uses `oh-my-codes` inside `opencode.json`.

## Basic Usage

```bash
# Display help
bunx oh-my-codes

# Or with npx
npx oh-my-codes
```

## Commands

| Command                      | Description                                              |
| ---------------------------- | -------------------------------------------------------- |
| `install`                    | Interactive setup wizard                                 |
| `doctor`                     | Environment diagnostics and health checks                |
| `run`                        | OpenCode session runner with task completion enforcement |
| `get-local-version`          | Display local version information and update check       |
| `refresh-model-capabilities` | Refresh the cached models.dev-based model capabilities   |
| `version`                    | Show version information                                 |
| `mcp oauth`                  | MCP OAuth authentication management                      |

---

## install

Interactive installation tool for initial oh-my-codes setup. Provides a TUI based on `@clack/prompts`.

### Usage

```bash
bunx oh-my-codes install
```

### Installation Process

1. **Subscription Selection**: Choose which providers and subscriptions you actually have
2. **Plugin Registration**: Registers `oh-my-codes` in OpenCode settings
3. **Configuration File Creation**: Writes the generated config to `oh-my-codes.json` in the active OpenCode config directory
4. **Authentication Hints**: Shows the `opencode auth login` steps for the providers you selected, unless `--skip-auth` is set
5. **Telemetry Defaults**: Anonymous telemetry remains enabled unless you opt out through environment variables

### Options

| Option                          | Description                              |
| ------------------------------- | ---------------------------------------- |
| `--no-tui`                      | Run in non-interactive mode without TUI  |
| `--claude <no\|yes\|max20>`     | Claude subscription mode                 |
| `--openai <no\|yes>`            | OpenAI / ChatGPT subscription            |
| `--gemini <no\|yes>`            | Gemini integration                       |
| `--copilot <no\|yes>`           | GitHub Copilot subscription              |
| `--opencode-zen <no\|yes>`      | OpenCode Zen access                      |
| `--zai-coding-plan <no\|yes>`   | Z.ai Coding Plan subscription            |
| `--kimi-for-coding <no\|yes>`   | Kimi for Coding subscription             |
| `--opencode-go <no\|yes>`       | OpenCode Go subscription                 |
| `--vercel-ai-gateway <no\|yes>` | Vercel AI Gateway: no, yes (default: no) |
| `--skip-auth`                   | Skip authentication setup hints          |

Anonymous telemetry uses PostHog with a hashed installation identifier. Disable it with `OMO_SEND_ANONYMOUS_TELEMETRY=0` or `OMO_DISABLE_POSTHOG=1`. See [Privacy Policy](../legal/privacy-policy.md).

---

## doctor

Diagnoses your environment to ensure oh-my-codes is functioning correctly. The current checks are grouped into system, config, tools, and models.

The doctor command detects common issues including:

- Legacy plugin entry references in `opencode.json` (warns when using old naming)
- Configuration file validity and JSONC parsing errors
- Model resolution and fallback chain verification
- Missing or misconfigured MCP servers

### Usage

```bash
bunx oh-my-codes doctor
```

### Diagnostic Categories

| Category   | Check Items                                                                             |
| ---------- | --------------------------------------------------------------------------------------- |
| **System** | OpenCode binary, version (>= 1.0.150), plugin registration, legacy package name warning |
| **Config** | Configuration file validity, JSONC parsing, Zod schema validation                       |
| **Tools**  | AST-Grep, LSP servers, GitHub CLI, MCP servers                                          |
| **Models** | Model capabilities cache, model resolution, agent/category overrides, availability      |

### Options

| Option      | Description                          |
| ----------- | ------------------------------------ |
| `--status`  | Show compact system dashboard        |
| `--verbose` | Show detailed diagnostic information |
| `--json`    | Output results in JSON format        |

### Example Output

```
oh-my-codes doctor

┌──────────────────────────────────────────────────┐
│  Oh-My-Codes Doctor                           │
└──────────────────────────────────────────────────┘

System
  ✓ OpenCode version: 1.0.155 (>= 1.0.150)
  ✓ Plugin registered in opencode.json

Config
  ✓ oh-my-codes.jsonc is valid
  ✓ Model resolution: all agents have valid fallback chains
  ⚠ categories.visual-engineering: using default model

Tools
  ✓ AST-Grep available
  ✓ LSP servers configured

Models
  ✓ 11 agents, 8 categories, 0 overrides
  ⚠ Some configured models rely on compatibility fallback

Summary: 10 passed, 1 warning, 0 failed
```

---

## run

Run opencode with todo/background task completion enforcement. Unlike 'opencode run', this command waits until all todos are completed or cancelled, and all child sessions (background tasks) are idle.

### Usage

```bash
bunx oh-my-codes run <message>
```

### Options

| Option                         | Description                                                     |
| ------------------------------ | --------------------------------------------------------------- |
| `-a, --agent <name>`           | Agent to use (default: from CLI/env/config, fallback: Sisyphus) |
| `-m, --model <provider/model>` | Model override (e.g., anthropic/claude-sonnet-4)                |
| `-d, --directory <path>`       | Working directory                                               |
| `-p, --port <port>`            | Server port (attaches if port already in use)                   |
| `--attach <url>`               | Attach to existing opencode server URL                          |
| `--on-complete <command>`      | Shell command to run after completion                           |
| `--json`                       | Output structured JSON result to stdout                         |
| `--no-timestamp`               | Disable timestamp prefix in run output                          |
| `--verbose`                    | Show full event stream (default: messages/tools only)           |
| `--session-id <id>`            | Resume existing session instead of creating new one             |

---

## get-local-version

Show current installed version and check for updates.

### Usage

```bash
bunx oh-my-codes get-local-version
```

### Options

| Option            | Description                            |
| ----------------- | -------------------------------------- |
| `-d, --directory` | Working directory to check config from |
| `--json`          | Output in JSON format for scripting    |

### Output

Shows:

- Current installed version
- Latest available version on npm
- Whether you're up to date
- Special modes (local dev, pinned version)

---

## version

Show version information.

### Usage

```bash
bunx oh-my-codes version
```

`--on-complete` runs through your current shell when possible: `sh` on Unix shells, `pwsh` for PowerShell on non-Windows, `powershell.exe` for PowerShell on Windows, and `cmd.exe` as the Windows fallback.

---

## mcp oauth

Manages OAuth 2.1 authentication for remote MCP servers.

### Usage

```bash
# Login to an OAuth-protected MCP server
bunx oh-my-codes mcp oauth login <server-name> --server-url https://api.example.com

# Login with explicit client ID and scopes
bunx oh-my-codes mcp oauth login my-api --server-url https://api.example.com --client-id my-client --scopes read write

# Remove stored OAuth tokens
bunx oh-my-codes mcp oauth logout <server-name> --server-url https://api.example.com

# Check OAuth token status
bunx oh-my-codes mcp oauth status [server-name]
```

### Options

| Option               | Description                                                                      |
| -------------------- | -------------------------------------------------------------------------------- |
| `--server-url <url>` | MCP server URL (required for login)                                              |
| `--client-id <id>`   | OAuth client ID (optional if server supports Dynamic Client Registration)        |
| `--scopes <scopes>`  | OAuth scopes as separate variadic arguments (for example: `--scopes read write`) |

### Token Storage

Tokens are stored in `~/.config/opencode/mcp-oauth.json` with `0600` permissions (owner read/write only). Key format: `{serverHost}/{resource}`.

---

## Configuration Files

The runtime loads user config as the base config, then merges project config on top:

1. **Project Level**: `.opencode/oh-my-codes.jsonc` or `.opencode/oh-my-codes.json`
2. **User Level**: `~/.config/opencode/oh-my-codes.jsonc` or `~/.config/opencode/oh-my-codes.json`

**Naming Note**: The published package and binary are `oh-my-codes`. Inside `opencode.json`, the plugin entry is `oh-my-codes`. Plugin config loading uses `oh-my-codes.*` basenames.

### Filename Compatibility

Both `.jsonc` and `.json` extensions are supported. JSONC (JSON with Comments) is preferred as it allows:

- Comments (both `//` and `/* */` styles)
- Trailing commas in arrays and objects

If both `.jsonc` and `.json` exist in the same directory, the `.jsonc` file takes precedence.

### JSONC Support

Configuration files support **JSONC (JSON with Comments)** format. You can use comments and trailing commas.

```jsonc
{
  // Agent configuration
  "sisyphus_agent": {
    "disabled": false,
    "planner_enabled": true,
  },

  /* Category customization */
  "categories": {
    "visual-engineering": {
      "model": "google/gemini-3.1-pro",
    },
  },
}
```

---

## Troubleshooting

### "OpenCode version too old" Error

```bash
# Update OpenCode
npm install -g opencode@latest
# or
bun install -g opencode@latest
```

### "Plugin not registered" Error

```bash
# Reinstall plugin
bunx oh-my-codes install
```

### Doctor Check Failures

```bash
# Diagnose with detailed information
bunx oh-my-codes doctor --verbose

# Show compact system dashboard
bunx oh-my-codes doctor --status

# JSON output for scripting
bunx oh-my-codes doctor --json
```

---

## refresh-model-capabilities

Refreshes the cached model capabilities snapshot from models.dev. This updates the local cache used by capability resolution and compatibility diagnostics.

### Usage

```bash
bunx oh-my-codes refresh-model-capabilities
```

### Options

| Option               | Description                                       |
| -------------------- | ------------------------------------------------- |
| `-d, --directory`    | Working directory to read oh-my-codes config from |
| `--source-url <url>` | Override the models.dev source URL                |
| `--json`             | Output refresh summary as JSON                    |

### Configuration

Configure automatic refresh behavior in your plugin config:

```jsonc
{
  "model_capabilities": {
    "enabled": true,
    "auto_refresh_on_start": true,
    "refresh_timeout_ms": 5000,
    "source_url": "https://models.dev/api.json",
  },
}
```

---

## Non-Interactive Mode

Use JSON output for CI or scripted diagnostics.

```bash
# Run doctor in CI environment
bunx oh-my-codes doctor --json

# Save results to file
bunx oh-my-codes doctor --json > doctor-report.json
```

---

## Developer Information

### CLI Structure

```
src/cli/
├── cli-program.ts        # Commander.js-based main entry
├── install.ts            # @clack/prompts-based TUI installer
├── config-manager/       # JSONC parsing, multi-source config management
│   └── *.ts
├── doctor/               # Health check system
│   ├── index.ts          # Doctor command entry
│   └── checks/           # 17+ individual check modules
├── run/                  # Session runner
│   └── *.ts
└── mcp-oauth/            # OAuth management commands
    └── *.ts
```

### Adding New Doctor Checks

Create `src/cli/doctor/checks/my-check.ts`:

```typescript
import type { DoctorCheck } from "../types"

export const myCheck: DoctorCheck = {
  name: "my-check",
  category: "environment",
  check: async () => {
    // Check logic
    const isOk = await someValidation()

    return {
      status: isOk ? "pass" : "fail",
      message: isOk ? "Everything looks good" : "Something is wrong",
    }
  },
}
```

Register in `src/cli/doctor/checks/index.ts`:

```typescript
export { myCheck } from "./my-check"
```
