#!/usr/bin/env node
// Verifica que CLAUDE.md y ANTIGRAVITY.md comparten exactamente el mismo bloque de reglas
// y que ANTIGRAVITY.md cabe en el límite de tamaño de las reglas de Antigravity.
// Uso: node scripts/check-agent-rules.mjs   (o `pnpm check:rules`)

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const START = "<!-- REGLAS-NUCLEO:INICIO -->";
const END = "<!-- REGLAS-NUCLEO:FIN -->";
const MAX_ANTIGRAVITY_BYTES = 22_000; // Antigravity trunca a 24 KB; se deja margen.
const REQUIRED_INVARIANTS = 15;

const errors = [];

function read(name) {
  try {
    return readFileSync(join(root, name), "utf8").replace(/\r\n/g, "\n");
  } catch {
    errors.push(`No se pudo leer ${name}.`);
    return "";
  }
}

function coreBlock(name, text) {
  const start = text.indexOf(START);
  const end = text.indexOf(END);
  if (start === -1 || end === -1 || end < start) {
    errors.push(`${name}: faltan los marcadores del bloque de reglas.`);
    return "";
  }
  if (text.indexOf(START, start + 1) !== -1 || text.indexOf(END, end + 1) !== -1) {
    errors.push(`${name}: los marcadores del bloque de reglas aparecen más de una vez.`);
  }
  return text.slice(start, end + END.length);
}

const claude = read("CLAUDE.md");
const antigravity = read("ANTIGRAVITY.md");
const claudeCore = coreBlock("CLAUDE.md", claude);
const antigravityCore = coreBlock("ANTIGRAVITY.md", antigravity);

if (claudeCore && antigravityCore && claudeCore !== antigravityCore) {
  const a = claudeCore.split("\n");
  const b = antigravityCore.split("\n");
  let line = 0;
  while (line < a.length && line < b.length && a[line] === b[line]) line++;
  errors.push(
    `El bloque de reglas difiere entre CLAUDE.md y ANTIGRAVITY.md (primera diferencia en la línea ${line + 1} del bloque):\n` +
      `  CLAUDE.md:      ${a[line] ?? "(fin del bloque)"}\n` +
      `  ANTIGRAVITY.md: ${b[line] ?? "(fin del bloque)"}`,
  );
}

for (let i = 1; i <= REQUIRED_INVARIANTS; i++) {
  if (claudeCore && !claudeCore.includes(`**I-${i}.**`)) {
    errors.push(`Falta la invariante I-${i} en el bloque de reglas.`);
  }
}

const antigravityBytes = Buffer.byteLength(antigravity, "utf8");
if (antigravityBytes > MAX_ANTIGRAVITY_BYTES) {
  errors.push(
    `ANTIGRAVITY.md pesa ${antigravityBytes} bytes; el máximo es ${MAX_ANTIGRAVITY_BYTES}. Mueve el detalle a docs/.`,
  );
}

const rule = read(".agents/rules/00-social.md");
if (rule && !/^trigger:\s*always_on\s*$/m.test(rule)) {
  errors.push(".agents/rules/00-social.md: falta `trigger: always_on` en el encabezado.");
}
if (rule && !rule.includes("(../../ANTIGRAVITY.md)")) {
  errors.push(".agents/rules/00-social.md: no incluye ANTIGRAVITY.md.");
}

if (errors.length > 0) {
  console.error("Reglas de agentes: ERROR\n");
  for (const e of errors) console.error(`- ${e}`);
  process.exit(1);
}

console.log(
  `Reglas de agentes: OK (bloque común idéntico, ${REQUIRED_INVARIANTS} invariantes, ANTIGRAVITY.md ${antigravityBytes} bytes).`,
);
