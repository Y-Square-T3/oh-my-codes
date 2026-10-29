# oh-my-codes-opencode

OpenCode plugin for [oh-my-codes](../../README.md). It connects OpenCode to your OMC account through the local `omcd` daemon:

- **Provider injection** — registers the providers and models from your OMC workspace with their credentials, so they appear in OpenCode like any built-in provider.
- **Token usage reporting** — sends per-message token usage (input/output/reasoning/cache read/cache write) to the daemon, which stores and pushes it to the OMC server.

## Requirements

- OpenCode **v2**, or OpenCode **v1 ≥ 1.18.29** (object plugin entrypoints)
- The `omcd` daemon reachable at `http://127.0.0.1:9823` (override with `OMC_DAEMON_URL`)

## Install

Recommended — via the `omc` CLI, which detects your OpenCode version and writes the right config key:

```bash
omc opencode install    # add --v1 or --v2 to force
omc opencode uninstall  # removes the entry from both keys
```

Manual — add to your global `~/.config/opencode/opencode.json(c)`:

```jsonc
// OpenCode v2
{ "plugins": ["oh-my-codes-opencode"] }
```

```jsonc
// OpenCode v1 (>= 1.18.29)
{ "plugin": ["oh-my-codes-opencode"] }
```

## How it works on each host version

A single package serves both versions through one default export that carries a V2 `setup` and a V1 `server` implementation:

|                    | OpenCode v1                                            | OpenCode v2                                                            |
| ------------------ | ------------------------------------------------------ | ---------------------------------------------------------------------- |
| Entrypoint         | `server(input)` hooks                                  | `setup(ctx)`                                                           |
| Provider injection | mutates `config.provider`                              | `ctx.provider.transform` → `editor.add`                                |
| Token usage source | `message.updated` event (`properties`, `finish: true`) | `session.step.started` + `session.step.ended` events (`data` envelope) |
| Diagnostics        | `client.app.log`                                       | `console.log` / `console.warn` (server log)                            |

Notes:

- Providers/models/credentials are fetched from the daemon once at load; restart or reload OpenCode to pick up changes.
- An existing provider with the same ID (your own config on v1; any registered provider on v2) is never overridden — injection skips it.
- Steps from `compaction`/`summarize` agents are excluded. In v2 each assistant message is its own step, so one `step.ended` is one completed message — reported exactly once (repeated events for the same message are deduped). `agent`/`model` come from the matching `step.started`.

## Development

From the repo root (Yarn 4 via `corepack enable`):

```bash
yarn oc build       # bundle to dist/
yarn oc test        # vitest (V1 + V2 mocked host suites)
yarn oc typecheck
yarn oc lint
yarn oc fmt:check
```

## Environment variables

| Variable         | Default                 | Purpose                       |
| ---------------- | ----------------------- | ----------------------------- |
| `OMC_DAEMON_URL` | `http://127.0.0.1:9823` | Base URL of the `omcd` daemon |
