import color from "picocolors"
import type {
  BooleanArg,
  ClaudeSubscription,
  DetectedConfig,
  InstallConfig,
} from "./types"

export const SYMBOLS = {
  check: color.green("[OK]"),
  cross: color.red("[X]"),
  arrow: color.cyan("->"),
  bullet: color.dim("*"),
  info: color.blue("[i]"),
  warn: color.yellow("[!]"),
  star: color.yellow("*"),
}

function formatProvider(
  name: string,
  enabled: boolean,
  detail?: string,
): string {
  const status = enabled ? SYMBOLS.check : color.dim("○")
  const label = enabled ? color.white(name) : color.dim(name)
  const suffix = detail ? color.dim(` (${detail})`) : ""
  return `  ${status} ${label}${suffix}`
}

export function formatConfigSummary(config: InstallConfig): string {
  const lines: string[] = []

  lines.push(color.bold(color.white("Configuration Summary")))
  lines.push("")

  const claudeDetail = config.hasClaude
    ? config.isMax20
      ? "max20"
      : "standard"
    : undefined
  lines.push(formatProvider("Claude", config.hasClaude, claudeDetail))
  lines.push(
    formatProvider("OpenAI/ChatGPT", config.hasOpenAI, "GPT-5.4 for Oracle"),
  )
  lines.push(formatProvider("Gemini", config.hasGemini))
  lines.push(formatProvider("GitHub Copilot", config.hasCopilot, "fallback"))
  lines.push(
    formatProvider("OpenCode Zen", config.hasOpencodeZen, "opencode/ models"),
  )
  lines.push(
    formatProvider(
      "Z.ai Coding Plan",
      config.hasZaiCodingPlan,
      "Librarian/Multimodal",
    ),
  )
  lines.push(
    formatProvider(
      "Kimi For Coding",
      config.hasKimiForCoding,
      "Sisyphus/Prometheus fallback",
    ),
  )
  lines.push(
    formatProvider(
      "Vercel AI Gateway",
      config.hasVercelAiGateway,
      "universal proxy",
    ),
  )

  lines.push("")
  lines.push(color.dim("─".repeat(40)))
  lines.push("")

  lines.push(color.bold(color.white("Model Assignment")))
  lines.push("")
  lines.push(
    `  ${SYMBOLS.info} Models auto-configured based on provider priority`,
  )
  lines.push(
    `  ${SYMBOLS.bullet} Priority: Native > Copilot > OpenCode Zen > Z.ai`,
  )

  return lines.join("\n")
}

export function createDefaultInstallConfig(): InstallConfig {
  return {
    hasClaude: false,
    isMax20: false,
    hasOpenAI: false,
    hasGemini: false,
    hasCopilot: false,
    hasOpencodeZen: false,
    hasZaiCodingPlan: false,
    hasKimiForCoding: false,
    hasOpencodeGo: false,
    hasVercelAiGateway: false,
  }
}

export function detectedToInitialValues(detected: DetectedConfig): {
  claude: ClaudeSubscription
  openai: BooleanArg
  gemini: BooleanArg
  copilot: BooleanArg
  opencodeZen: BooleanArg
  zaiCodingPlan: BooleanArg
  kimiForCoding: BooleanArg
  opencodeGo: BooleanArg
  vercelAiGateway: BooleanArg
} {
  let claude: ClaudeSubscription = "no"
  if (detected.hasClaude) {
    claude = detected.isMax20 ? "max20" : "yes"
  }

  return {
    claude,
    openai: detected.hasOpenAI ? "yes" : "no",
    gemini: detected.hasGemini ? "yes" : "no",
    copilot: detected.hasCopilot ? "yes" : "no",
    opencodeZen: detected.hasOpencodeZen ? "yes" : "no",
    zaiCodingPlan: detected.hasZaiCodingPlan ? "yes" : "no",
    kimiForCoding: detected.hasKimiForCoding ? "yes" : "no",
    opencodeGo: detected.hasOpencodeGo ? "yes" : "no",
    vercelAiGateway: detected.hasVercelAiGateway ? "yes" : "no",
  }
}
