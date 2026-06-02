# oh-my-codes

The Best AI Agent Harness — a plugin for [OpenCode](https://opencode.ai) that adds multi-model orchestration, account and workspace management, and token usage tracking.

## Quick Start

```bash
# Install globally
npm install -g oh-my-codes

# Run the interactive setup
omc install
```

Or without installing:

```bash
npx oh-my-codes install
```

See the [Get Started guide](./docs/en/get-started.md) for detailed instructions.

## Documentation

Read the full documentation in your language:

- [English](./docs/en/index.md)
- [简体中文](./docs/zh-CN/index.md)

## CLI Commands

| Command | Description |
|---------|-------------|
| `omc install` | Interactive setup (login, workspace, plugin config) |
| `omc account login <url>` | Log in via device code flow |
| `omc account logout [email]` | Log out from an account |
| `omc account switch` | Switch active workspace |
| `omc account list` | List logged-in accounts |
| `omc model list` | List available models |
| `omc model refresh` | Fetch latest models from API |
| `omc model clear` | Clear cached models |
| `omc token-usages` | View local token usage status |
| `omc token-usages push` | Push cached usage to server |

## Inside OpenCode

Once configured, these commands are available directly in OpenCode:

- `/omc-login` — log in to an OMC account
- `/omc-switch` — switch active workspace

## License

SUL-1.0
