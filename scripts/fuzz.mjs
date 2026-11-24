#!/usr/bin/env node
// `pnpm fuzz`: ejecuta los objetivos de fuzzing de fuzz/ con cargo-fuzz (nightly).
//   pnpm fuzz                      todos los objetivos, 60 s cada uno
//   pnpm fuzz <objetivo> <seg>     un objetivo durante <seg> segundos
// Siembra el corpus con los eventos de spec/vectors/. Un fallo deja el caso en
// fuzz/artifacts/<objetivo>/ y sale con error.

import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const isWindows = process.platform === "win32";
const fuzzDir = join(root, "fuzz");
const TARGETS = ["json_parse", "event_parse", "event_validate"];

const args = process.argv.slice(2).filter((a) => a !== "--");
const targets = args[0] ? [args[0]] : TARGETS;
const seconds = Number.parseInt(args[1] ?? "60", 10);
if (targets.some((t) => !TARGETS.includes(t)) || !(seconds > 0)) {
  console.error(`fuzz: uso: pnpm fuzz [${TARGETS.join("|")}] [segundos]`);
  process.exit(2);
}

const probe = spawnSync("cargo", ["fuzz", "--version"], {
  cwd: fuzzDir,
  stdio: "ignore",
  shell: isWindows,
});
if (probe.status !== 0) {
  console.error(
    "fuzz: falta cargo-fuzz: `rustup toolchain install nightly` y `cargo install cargo-fuzz --locked`.",
  );
  process.exit(1);
}

// Semillas: cada evento de los vectores dorados, válido o inválido.
const seeds = [];
for (const file of ["envelope.json", "validation.json"]) {
  const suite = JSON.parse(readFileSync(join(root, "spec", "vectors", file), "utf8"));
  for (const v of [...suite.valid, ...suite.invalid]) {
    seeds.push(v.event ?? v.expect?.event);
  }
}

for (const target of targets) {
  const corpus = join(fuzzDir, "corpus", target);
  mkdirSync(corpus, { recursive: true });
  seeds.filter(Boolean).forEach((s, i) => writeFileSync(join(corpus, `seed-${i}.json`), s));
  console.log(`\n▶ fuzz ${target} (${seconds} s)`);
  const result = spawnSync(
    "cargo",
    ["fuzz", "run", target, corpus, "--", `-max_total_time=${seconds}`, "-max_len=20000"],
    { cwd: fuzzDir, stdio: "inherit", shell: isWindows },
  );
  if (result.status !== 0) {
    console.error(`✖ fuzz ${target}: fallo; el caso quedó en fuzz/artifacts/${target}/.`);
    process.exit(result.status ?? 1);
  }
}
console.log("\n✔ fuzz: sin fallos.");
