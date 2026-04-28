import * as p from "@clack/prompts"
import type { DetectedConfig, InstallConfig } from "./types"

async function selectOrCancel<
  TValue extends Readonly<string | boolean | number>,
>(params: {
  message: string
  options: p.Option<TValue>[]
  initialValue: TValue
}): Promise<TValue | null> {
  if (!process.stdin.isTTY || !process.stdout.isTTY) return null

  const value = await p.select<TValue>({
    message: params.message,
    options: params.options,
    initialValue: params.initialValue,
  })
  if (p.isCancel(value)) {
    p.cancel("Installation cancelled.")
    return null
  }
  return value as TValue
}

export { selectOrCancel }

export async function promptInstallConfig(
  _detected: DetectedConfig,
): Promise<InstallConfig | null> {
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
