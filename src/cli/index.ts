#!/usr/bin/env bun
import { runCli } from "./cli-program"

runCli().catch((err) => {
  console.error(err)
  process.exit(1)
})
