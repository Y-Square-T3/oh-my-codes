import { execSync } from "node:child_process";
import { mkdirSync, chmodSync, copyFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const TARGET_TO_DEST = {
  "aarch64-apple-darwin": "darwin-arm64",
  "x86_64-apple-darwin": "darwin-x64",
  "aarch64-unknown-linux-gnu": "linux-arm64",
  "x86_64-unknown-linux-gnu": "linux-x64",
  "x86_64-pc-windows-msvc": "win32-x64",
};

const PLATFORM_ARCH_TO_TARGET = {
  "darwin/arm64": "aarch64-apple-darwin",
  "darwin/x64": "x86_64-apple-darwin",
  "linux/arm64": "aarch64-unknown-linux-gnu",
  "linux/x64": "x86_64-unknown-linux-gnu",
  "win32/x64": "x86_64-pc-windows-msvc",
};

function deriveDest(target) {
  return TARGET_TO_DEST[target] ?? "";
}

function detectPlatform() {
  const plat = process.platform;
  const arch = process.arch;
  const key = `${plat}/${arch}`;
  const target = PLATFORM_ARCH_TO_TARGET[key];
  if (!target) {
    console.error(`Unsupported platform: ${key}`);
    process.exit(1);
  }
  return { plat, arch, target };
}

function run(cmd, opts = {}) {
  console.log(`$ ${cmd}`);
  execSync(cmd, { stdio: "inherit", ...opts });
}

const targetArg = process.argv[2] || "";
const destArg = process.argv[3] || "";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const cargoDir = resolve(scriptDir, "..");

let target = targetArg;
let dest = destArg;

if (!target) {
  const { plat, arch, target: detected } = detectPlatform();
  target = detected;
  if (!dest) {
    dest = deriveDest(target);
  }
  console.log(`Detected ${plat}/${arch} -> ${dest} (${target})`);
} else {
  console.log(`Building ${target}`);
}

if (!dest) {
  dest = deriveDest(target);
  if (!dest) {
    console.error(`Unknown target: ${target}`);
    process.exit(1);
  }
}

const destDir = join(cargoDir, "dist", dest);
console.log(`Target: ${target} -> ${destDir}`);

try {
  run(`rustup target add ${target}`, { stdio: ["pipe", "pipe", "pipe"] });
} catch {
  // target may already be installed
}

run(
  `cargo build --release --target ${target} --manifest-path ${join(cargoDir, "Cargo.toml")}`,
);

const binDir = join(destDir, "bin");
mkdirSync(binDir, { recursive: true });

const isWindows = target.includes("windows");
const releaseDir = join(cargoDir, "target", target, "release");

if (isWindows) {
  copyFileSync(join(releaseDir, "omc.exe"), join(binDir, "omc.exe"));
  copyFileSync(join(releaseDir, "omcd.exe"), join(binDir, "omcd.exe"));
} else {
  copyFileSync(join(releaseDir, "omc"), join(binDir, "omc"));
  copyFileSync(join(releaseDir, "omcd"), join(binDir, "omcd"));
  chmodSync(join(binDir, "omc"), 0o755);
  chmodSync(join(binDir, "omcd"), 0o755);
}

console.log(`Binaries placed in ${binDir}`);
