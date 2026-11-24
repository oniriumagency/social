#!/usr/bin/env node
// `pnpm bootstrap`: prepara el repositorio en una máquina nueva (docs/11 §3).
// 1. Comprueba las herramientas necesarias. 2. Instala dependencias de Node.
// 3. Descarga y compila las dependencias de Rust. 4. Activa los ganchos de git.

import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const isWindows = process.platform === "win32";
// Sin telemetría de herramientas (CLAUDE.md §8).
const env = { ...process.env, TURBO_TELEMETRY_DISABLED: "1", DO_NOT_TRACK: "1" };

function run(program, args, { optional = false } = {}) {
  console.log(`\n▶ ${program} ${args.join(" ")}`);
  const result = spawnSync(program, args, { cwd: root, stdio: "inherit", shell: isWindows, env });
  if (result.error || result.status !== 0) {
    if (optional) return false;
    console.error(`\n✖ bootstrap: falló "${program} ${args.join(" ")}".`);
    process.exit(result.status ?? 1);
  }
  return true;
}

function has(program) {
  const result = spawnSync(program, ["--version"], { stdio: "ignore", shell: isWindows });
  return !result.error && result.status === 0;
}

const missing = ["git", "cargo", "rustup", "pnpm"].filter((p) => !has(p));
if (missing.length > 0) {
  console.error(
    `bootstrap: faltan herramientas: ${missing.join(", ")}. Requisitos en docs/11-repositorio-entorno-y-ci.md §1.`,
  );
  process.exit(1);
}

run("pnpm", ["install"]);
// rustup instala el canal, los componentes y el destino WASM de rust-toolchain.toml al usar cargo.
run("cargo", ["fetch", "--locked"]);
run("cargo", ["build", "--workspace", "--locked"]);

// Los ganchos viven en .githooks y son scripts de Node (sin shell).
if (has("git") && run("git", ["rev-parse", "--git-dir"], { optional: true })) {
  run("git", ["config", "core.hooksPath", ".githooks"]);
}

console.log("\n✔ bootstrap: repositorio listo. Siguiente paso: `pnpm check`.");
