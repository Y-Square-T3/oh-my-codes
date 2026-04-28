import packageJson from "../../package.json" with { type: "json" }
import { runTuiInstaller } from "./tui-installer"

const VERSION = packageJson.version

export async function install(): Promise<number> {
  return runTuiInstaller(VERSION)
}
