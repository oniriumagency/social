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
  lint: [
    ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"],
    ["pnpm", "exec", "eslint", "--max-warnings", "0", "."],
    ["pnpm", "exec", "turbo", "run", "lint"],
  ],
  typecheck: [
    ["cargo", "check", "--workspace", "--all-targets", "--locked"],
    ["pnpm", "exec", "turbo", "run", "typecheck"],
  ],
  test: [
    ["cargo", "test", "--workspace", "--locked"],
    ["pnpm", "exec", "turbo", "run", "test"],
  ],
};

// `pnpm check` agrupa todo lo que debe pasar antes de un PR (docs/11 §3).
STEPS.check = [
  ["node", "scripts/check-agent-rules.mjs"],
  ...STEPS["fmt:check"],
  ...STEPS.lint,
  ...STEPS.typecheck,
  ...STEPS.test,
  ["pnpm", "run", "test:vectors"],
  ["pnpm", "run", "spec:gen"],
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
