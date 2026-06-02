# Get Started

This guide walks you through installing OpenCode and configuring oh-my-codes.

---

## Table of Contents

- [Part 1: Install OpenCode](#part-1-install-opencode)
- [Part 2: Install oh-my-codes](#part-2-install-oh-my-codes)
- [Part 3: Configure oh-my-codes](#part-3-configure-oh-my-codes)
- [Next Steps](#next-steps)

---

## Part 1: Install OpenCode

OpenCode is an open source AI coding agent available as a terminal interface, desktop app, and IDE extension. oh-my-codes requires OpenCode to be installed first.

### Prerequisites

- **Node.js** (v18 or later) or a modern terminal emulator
- An LLM provider API key (or use [OpenCode Zen](https://opencode.ai/zen) for curated models)

### Install via curl

The fastest way to get OpenCode:

```bash
curl -fsSL https://opencode.ai/install | bash
```

### Install via npm

```bash
npm install -g opencode-ai
```

### Verify the installation

```bash
opencode --version
```
---

## Part 2: Install oh-my-codes

oh-my-codes extends OpenCode with:

- **Account and workspace management** — log in to multiple workspaces, switch between them
- **Token usage tracking** — monitor and push your local token usage data

You can run oh-my-codes either as a one-off command with `npx` or install it globally.

### Run without installing (npx)

No installation needed — just use `npx`:

```bash
npx oh-my-codes install
```

### Install globally

For frequent use, install oh-my-codes globally so you can use it from anywhere:

```bash
npm install -g oh-my-codes
```

Once installed, you can use the full name `oh-my-codes` or the short alias `omc`:

```bash
omc install
omc account login <server-url>
```

> **Tip:** The `omc` alias is available automatically when you install oh-my-codes globally. You can use it interchangeably with `oh-my-codes` in all examples below.

---

## Part 3: Configure oh-my-codes

### Option A: Interactive installer (recommended)

Run the interactive installer from your terminal:

```bash
omc install
```

The installer will guide you through:

1. **Account login** — authenticate via device code flow (opens your browser)
2. **Workspace selection** — pick which workspace to activate
3. **Plugin registration** — automatically adds oh-my-codes to your OpenCode config

If you already have accounts set up, you can skip the login step:

```bash
omc install --skip-login
```

### Option B: Manual setup

Add `oh-my-codes@latest` to the `plugin` array in your OpenCode config file.

The config file is located at `~/.config/opencode/opencode.json` (or `opencode.jsonc`):

```jsonc
{
  "plugin": [
    "oh-my-codes@latest"
  ]
}
```

Then log in to your account:

```bash
omc account login <server-url>
```

You will be prompted to open a URL in your browser and enter a device code. After authorizing, select a workspace to activate.

---

## Account Management

oh-my-codes provides CLI commands to manage your accounts and workspaces.

### Log in

```bash
omc account login <server-url>
```

Authenticates you via device code flow. After successful login, you can select a workspace.

### Log out

```bash
omc account logout [email]
```

Logs you out from a specific account (by email) or the current active account.

### Switch workspace

```bash
omc account switch
```

Interactively switch between your logged-in workspaces.

### List accounts

```bash
omc account list
```

Show all logged-in accounts and their active workspaces.

---

## Model Management

oh-my-codes can discover and manage models available through your connected accounts.

### List models

```bash
omc model list
omc model list --provider <provider-id>
omc model list --json
```

Lists all models available from your account's model API.

### Refresh models

```bash
omc model refresh
```

Fetches the latest model list from your account's API.

### Clear models

```bash
omc model clear
omc model clear --provider <provider-id>
```

Clears cached models for the active account.

---

## Token Usage Tracking

oh-my-codes automatically tracks your token usage locally. You can view and push this data.

### Check status

```bash
omc token-usages
omc tu status
```

Shows how many unpushed usage records are cached locally.

### Push usage data

```bash
omc token-usages push
omc tu push --json
```

Pushes all cached usage records to your account server.

---

## Inside OpenCode

Once oh-my-codes is configured, you get two commands available directly in OpenCode:

- **`/omc-login`** — log in to an OMC account from within OpenCode
- **`/omc-switch`** — switch your active workspace without leaving OpenCode

These commands use the same device code flow and workspace selection as the CLI.

---

## Config File Reference

| File | Location |
|------|----------|
| OpenCode config | `~/.config/opencode/opencode.json` or `opencode.jsonc` |
| oh-my-codes data | Managed internally (SQLite database) |

You can override the config directory by setting `OPENCODE_CONFIG_DIR`:

```bash
export OPENCODE_CONFIG_DIR=/path/to/custom/config
```

---

## Next Steps

- Explore the [OpenCode documentation](https://opencode.ai/docs) for more on using OpenCode
