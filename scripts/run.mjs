#!/usr/bin/env node
// Ejecutor de las órdenes únicas del repositorio (docs/11 §3, ADR-015).
// Encadena pasos de Rust y de TypeScript sin scripts de shell, para que funcione
// igual en Windows, macOS y Linux. Se detiene en el primer paso que falle.
// Uso: node scripts/run.mjs <orden>   (normalmente vía `pnpm <orden>`)

import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const isWindows = process.platform === "win32";
// Sin telemetría de herramientas (CLAUDE.md §8).
const env = { ...process.env, TURBO_TELEMETRY_DISABLED: "1", DO_NOT_TRACK: "1" };

/** Pasos de cada orden. Cada paso es [programa, ...argumentos]. */
const STEPS = {
  fmt: [
    ["cargo", "fmt", "--all"],
    ["pnpm", "exec", "prettier", "--write", "--log-level", "warn", "."],
  ],
  "fmt:check": [
    ["cargo", "fmt", "--all", "--check"],
    ["pnpm", "exec", "prettier", "--check", "--log-level", "warn", "."],
  ],
  "lint:rust": [
    ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"],
  ],
  "lint:ts": [
    ["pnpm", "exec", "eslint", "--max-warnings", "0", "."],
    ["pnpm", "exec", "turbo", "run", "lint"],
  ],
  "typecheck:rust": [["cargo", "check", "--workspace", "--all-targets", "--locked"]],
  "typecheck:ts": [["pnpm", "exec", "turbo", "run", "typecheck"]],
  "test:rust": [["cargo", "test", "--workspace", "--locked"]],
  "test:ts": [["pnpm", "exec", "turbo", "run", "test"]],
  // Los mismos vectores en nativo, en WASM sobre Node y en WASM dentro de Chromium.
  "test:vectors": [
    ["cargo", "test", "--locked", "-p", "social-core", "--test", "golden_vectors"],
    ["pnpm", "exec", "turbo", "run", "test:vectors", "--filter", "@social/core"],
  ],
  // Cadena de suministro (docs/11 §5): licencias, orígenes, crates prohibidos y avisos de seguridad.
  "check:supply": [
    ["cargo", "deny", "--locked", "check"],
    ["pnpm", "audit", "--prod", "--audit-level", "moderate"],
  ],
  // Los objetivos de fuzz/ compilan con su archivo de bloqueo (CI, trabajo fuzz-build).
  "check:fuzz": [["cargo", "check", "--locked", "--manifest-path", "fuzz/Cargo.toml", "--bins"]],
};

STEPS.lint = [...STEPS["lint:rust"], ...STEPS["lint:ts"]];
STEPS.typecheck = [...STEPS["typecheck:rust"], ...STEPS["typecheck:ts"]];
STEPS.test = [...STEPS["test:rust"], ...STEPS["test:ts"]];

// Lo que CI ejecuta por lenguaje, en Linux y Windows. Son las mismas órdenes que en local.
STEPS["check:rust"] = [
  ["cargo", "fmt", "--all", "--check"],
  ...STEPS["lint:rust"],
  ...STEPS["test:rust"],
  // Esquemas de spec/ sin deriva (trabajo `spec` de docs/11 §5).
  ["cargo", "run", "--locked", "-p", "xtask", "--", "spec", "--check"],
];
STEPS["check:ts"] = [
  ["pnpm", "exec", "prettier", "--check", "--log-level", "warn", "."],
  ...STEPS["lint:ts"],
  ...STEPS["typecheck:ts"],
  ...STEPS["test:ts"],
];

// `pnpm check` agrupa todo lo que debe pasar antes de un PR (docs/11 §3).
STEPS.check = [
  ["node", "scripts/check-agent-rules.mjs"],
  ...STEPS["check:rust"],
  ...STEPS["check:ts"],
  ["pnpm", "run", "test:vectors"],
  ...STEPS["check:supply"],
  ["node", "scripts/check-secrets.mjs"],
];

const order = process.argv[2];
const steps = STEPS[order];
if (!steps) {
  console.error(`run: orden desconocida "${order}". Órdenes: ${Object.keys(STEPS).join(", ")}`);
  process.exit(2);
}

for (const [program, ...args] of steps) {
  console.log(`\n▶ ${program} ${args.join(" ")}`);
  // En Windows, pnpm es un archivo .cmd y necesita la shell del sistema para resolverse.
  const result = spawnSync(program, args, { cwd: root, stdio: "inherit", shell: isWindows, env });
  if (result.error) {
    console.error(`run: no se pudo ejecutar "${program}": ${result.error.message}`);
    process.exit(1);
  }
  if (result.status !== 0) {
    console.error(`\n✖ ${order}: falló "${program} ${args.join(" ")}" (código ${result.status}).`);
    process.exit(result.status ?? 1);
  }
}
console.log(`\n✔ ${order}: todo en verde.`);
