# oh-my-codes

## Package Manager

- **Yarn 4.15.0** — declared via `packageManager` in `package.json`.
- Linker: `node-modules` (set in `.yarnrc.yml`). Runs on `yarn install` without PnP.
- Run `corepack enable` before `yarn install` to use the correct yarn version.

## Monorepo

- Workspaces: `packages/omc` and `packages/omc/dist/*` (platform packages).
- Cross-workspace dependencies use the `workspace:` protocol (e.g. `workspace:*`).
- Root scripts:
  - `yarn omc <script>` — shorthand for `yarn workspace oh-my-codes <script>`

## Rust Package (omc)

- Located in `packages/omc`. Cargo workspace with integrated npm distribution.
- npm package name: `oh-my-codes`
- Three binaries: `omc` (CLI), `omcd` (daemon).
- Workspace crates under `packages/omc/crates/`:
  - `omc-core` — shared types, config, errors
  - `omc-api` — API types + HTTP client SDK
  - `omc-storage` — storage trait + SurrealDB embedded backend
  - `omc-server` — axum HTTP server + route handlers
  - `omc-service` — OS service management
  - `omc-tui` — terminal UI
  - `omc` — CLI binary
  - `omcd` — daemon binary
- Available scripts:
  - `yarn omc build` — compile binaries via `scripts/build-binary.sh`
  - `yarn omc dev` — run the CLI (`cargo run -p omc --`)
  - `yarn omc dev:daemon` — run the daemon (`cargo run -p omcd --`)
  - `yarn omc test` — run all workspace tests (`cargo test --workspace`)
  - `yarn omc check` — quick compilation check (`cargo check --workspace`)
  - `yarn omc lint` — run clippy lints, deny warnings (`cargo clippy --workspace --all-targets -- -D warnings`)
  - `yarn omc lint:fix` — run clippy with auto-fix (`cargo clippy --workspace --all-targets --fix -- -D warnings`)
  - `yarn omc fmt` — format all Rust code (`cargo fmt --all`)
  - `yarn omc fmt:check` — check formatting without modifying (`cargo fmt --all --check`)
- Build: `cargo build --workspace --release`
- Test: `cargo test --workspace`

## Platform Distribution

- Platform-specific packages in `packages/omc/dist/<platform>/`:
  - `oh-my-codes-darwin-arm64`
  - `oh-my-codes-darwin-x64`
  - `oh-my-codes-linux-arm64`
  - `oh-my-codes-linux-x64`
  - `oh-my-codes-win32-x64`
- Each platform package contains prebuilt `omc` and `omcd` binaries under `bin/`.
- `bin/omc.js` (root of `oh-my-codes`) is a Node.js wrapper that dispatches to the correct platform binary based on `process.platform`/`process.arch`.
- Build script: `packages/omc/scripts/build-binary.sh [target-triple]` — auto-detects host platform if no target given, places binaries into the matching `dist/<platform>/bin/`.

## CI

- GitHub Actions workflow (`.github/workflows/omc-ci.yml`) triggered on push to `main`.
- Runs on `ubuntu-latest` with Rust stable.
- Steps (in order, fail-fast): `fmt:check` → `check` → `lint` → `test`.
- Uses `yarn omc` scripts for all checks.

## Release

- GitHub Actions workflow (`.github/workflows/omc-release.yml`) triggered via `workflow_dispatch`.
- Inputs: `bump` (patch/minor/major) or explicit `version`.
- Builds on macOS, Ubuntu, and Windows for all supported targets.
- Publishes platform packages + main package to npm as `oh-my-codes`.
- Creates a git tag (`v<version>`) and GitHub release with auto-generated changelog.
- Changelog generation script: `scripts/generate-changelog.mjs`.

## Conventions

- **Formatter**: `cargo fmt` (rustfmt)
- **Linter**: `cargo clippy --workspace --all-targets -- -D warnings`
- CI enforces both on push to `main`.
- **Tests**: Place all tests in `tests/` directory as integration tests. Do not use inline `#[cfg(test)] mod tests` in `src/` files. This ensures tests exercise the crate's public API and keeps source files focused on implementation.
