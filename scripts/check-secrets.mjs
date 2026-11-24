#!/usr/bin/env node
// Escáner de secretos (docs/11 §5, I-10) con gitleaks sobre todo el historial de git.
// En CI es obligatorio. En local se ejecuta si gitleaks está instalado; si no, avisa
// y sale bien, porque el trabajo `secrets` de CI lo vuelve a ejecutar sobre cada PR.

import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const isWindows = process.platform === "win32";
const inCi = process.env.CI === "true";

const probe = spawnSync("gitleaks", ["version"], { stdio: "ignore", shell: isWindows });
if (probe.error || probe.status !== 0) {
  if (inCi) {
    console.error("check-secrets: gitleaks no está instalado en CI.");
    process.exit(1);
  }
  console.warn(
    "check-secrets: gitleaks no está instalado; se omite en local y lo ejecuta CI. " +
      "Instalación: https://github.com/gitleaks/gitleaks#installing",
  );
  process.exit(0);
}

// Nunca se imprime el secreto encontrado (--redact): la salida de CI es pública.
const result = spawnSync(
  "gitleaks",
  ["git", "--redact", "--no-banner", "--config", ".gitleaks.toml", "."],
  { cwd: root, stdio: "inherit", shell: isWindows },
);
process.exit(result.status ?? 1);
