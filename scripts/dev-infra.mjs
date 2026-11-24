#!/usr/bin/env node
// `pnpm dev:infra`: levanta PostgreSQL y el almacenamiento S3 locales con Docker Compose
// y espera a que estén sanos.
//   pnpm dev:infra          levanta (acción por defecto: up)
//   pnpm dev:infra down     los para; los datos se conservan
//   pnpm dev:infra reset    los para y vacía sus volúmenes de datos
//   pnpm dev:infra status   muestra su estado
// Lee la configuración de `.env`; si no existe, la crea a partir de `.env.example`.
// Nunca imprime los valores (I-10).

import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const isWindows = process.platform === "win32";
const envFile = join(root, ".env");
const composeFile = join(root, "infra", "compose", "compose.yaml");

const action = process.argv.slice(2).find((a) => a !== "--") ?? "up";
const ACTIONS = {
  up: ["up", "--detach", "--wait", "--wait-timeout", "120"],
  down: ["down"],
  reset: ["down", "--volumes"],
  status: ["ps"],
};
if (!Object.hasOwn(ACTIONS, action)) {
  console.error(
    `dev:infra: acción desconocida "${action}". Acciones: ${Object.keys(ACTIONS).join(", ")}`,
  );
  process.exit(2);
}

const docker = spawnSync("docker", ["info"], { stdio: "ignore", shell: isWindows });
if (docker.error || docker.status !== 0) {
  console.error("dev:infra: Docker no responde. Abre Docker Desktop y vuelve a intentarlo.");
  process.exit(1);
}

if (!existsSync(envFile)) {
  copyFileSync(join(root, ".env.example"), envFile);
  console.warn("dev:infra: se creó .env a partir de .env.example. Cambia las contraseñas locales.");
}

const args = ["compose", "--file", composeFile, "--env-file", envFile, ...ACTIONS[action]];
const result = spawnSync("docker", args, { cwd: root, stdio: "inherit", shell: isWindows });
if (result.status !== 0) {
  console.error(`dev:infra: falló "docker compose ${ACTIONS[action].join(" ")}".`);
  process.exit(result.status ?? 1);
}
if (action === "up") {
  console.log("\n✔ dev:infra: PostgreSQL y S3 locales listos (puertos en .env, solo 127.0.0.1).");
}
