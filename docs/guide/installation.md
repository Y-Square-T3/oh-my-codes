# Installation

## Interactive Installer

Run the interactive installer:

```bash
npx oh-my-codes install
# or
bunx oh-my-codes install
# or
yarn dlx oh-my-codes install
```

> **Note**: The CLI ships with standalone binaries for all major platforms. No runtime (Bun/Node.js) is required for CLI execution after installation.
>
> **Supported platforms**: macOS (ARM64, x64), Linux (x64, ARM64, Alpine/musl), Windows (x64)

Follow the prompts to configure your Claude, ChatGPT, and Gemini subscriptions. After installation, authenticate your providers as instructed.

Anonymous telemetry is enabled by default to help improve install and runtime reliability. It uses PostHog with a hashed installation identifier and can be disabled with `OMO_SEND_ANONYMOUS_TELEMETRY=0` or `OMO_DISABLE_POSTHOG=1`. See [Privacy Policy](../legal/privacy-policy.md) and [Terms of Service](../legal/terms-of-service.md).

## What's Next?

After installation, read the [Overview Guide](./overview.md) to understand what you have.

## Package Names

The published package and local binary are `oh-my-codes`. Inside `opencode.json`, the plugin entry should be `oh-my-codes`. Plugin config loading uses `oh-my-codes.json[c]`. If you see a "Using legacy package name" warning from `oh-my-codes doctor`, update your `opencode.json` plugin entry to `"oh-my-codes"`.