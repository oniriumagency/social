#!/usr/bin/env node
// Compila el núcleo a WASM (`crates/social-core-wasm`) y genera los enlaces de JavaScript
// y los tipos de TypeScript en `packages/core/src/generated` con wasm-bindgen.
// Lo ejecutan `pnpm bootstrap` y el `build` de `@social/core`.

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const isWindows = process.platform === "win32";
const crateDir = join(root, "crates", "social-core-wasm");
const outDir = join(root, "packages", "core", "src", "generated");
const wasm = join(root, "target", "wasm32-unknown-unknown", "release", "social_core_wasm.wasm");

// La versión de wasm-bindgen-cli debe ser la misma que fija Cargo.toml del crate.
const manifest = readFileSync(join(crateDir, "Cargo.toml"), "utf8");
const pinned = /wasm-bindgen = "=([0-9.]+)"/.exec(manifest)?.[1];
if (!pinned) {
  console.error("build-wasm: no se encontró la versión fijada de wasm-bindgen en Cargo.toml.");
  process.exit(1);
}

function run(program, args) {
  const result = spawnSync(program, args, { cwd: root, stdio: "inherit", shell: isWindows });
  if (result.error || result.status !== 0) {
    console.error(`build-wasm: falló "${program} ${args.join(" ")}".`);
    process.exit(result.status ?? 1);
  }
}

const probe = spawnSync("wasm-bindgen", ["--version"], { encoding: "utf8", shell: isWindows });
const installed = probe.status === 0 ? probe.stdout.trim().split(/\s+/)[1] : undefined;
if (installed !== pinned) {
  console.error(
    `build-wasm: hace falta wasm-bindgen-cli ${pinned} (instalado: ${installed ?? "ninguno"}).\n` +
      `  cargo install wasm-bindgen-cli --version ${pinned} --locked   (o \`pnpm bootstrap\`)`,
  );
  process.exit(1);
}

run("cargo", [
  "build",
  "--locked",
  "--release",
  "--package",
  "social-core-wasm",
  "--target",
  "wasm32-unknown-unknown",
]);
if (!existsSync(wasm)) {
  console.error(`build-wasm: no se generó ${wasm}.`);
  process.exit(1);
}
// Rutas relativas a `root` (el cwd): con `shell` en Windows, una ruta absoluta con
// espacios se partiría en varios argumentos.
run("wasm-bindgen", [
  "--target",
  "web",
  "--out-dir",
  relative(root, outDir),
  "--out-name",
  "social_core",
  relative(root, wasm),
]);
console.log(`build-wasm: listo en ${outDir}`);
