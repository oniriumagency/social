#!/usr/bin/env node
// Vectores dorados en WASM dentro de un navegador real (Chromium con Playwright).
// Sirve el núcleo generado y spec/vectors/ desde un servidor local efímero en 127.0.0.1,
// abre una página que no carga nada de terceros (I-6) y ejecuta cada suite.

import { createServer, request } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const SUITES = { "envelope.json": 33, "validation.json": 63, "ids.json": 128 };
const TYPES = {
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
};
// Lista cerrada de rutas exactas: el servidor no resuelve nada más del disco.
const FILES = new Map(
  [
    "packages/core/src/generated/social_core.js",
    "packages/core/src/generated/social_core_bg.wasm",
    ...Object.keys(SUITES).map((name) => `spec/vectors/${name}`),
  ].map((path) => [`/${path}`, join(root, path)]),
);

const page = `<!doctype html><meta charset="utf-8"><title>vectores</title>
<script type="module">
  import init, { runVectorSuite } from "/packages/core/src/generated/social_core.js";
  await init();
  const out = {};
  for (const name of ${JSON.stringify(Object.keys(SUITES))}) {
    const text = await (await fetch("/spec/vectors/" + name)).text();
    out[name] = JSON.parse(runVectorSuite(text));
  }
  window.__report = out;
</script>`;

const server = createServer(async (req, res) => {
  // Se compara la ruta tal como llega, sin decodificar: `%2f` o `..` nunca coinciden.
  const path = new URL(req.url ?? "/", "http://x").pathname;
  if (path === "/") {
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    res.end(page);
    return;
  }
  const file = FILES.get(path);
  if (file === undefined) {
    res.writeHead(404).end();
    return;
  }
  try {
    const body = await readFile(file);
    res.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream" });
    res.end(body);
  } catch {
    res.writeHead(404).end();
  }
});

await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const { port } = server.address();
const browser = await chromium.launch();
let failed = false;
// Prueba negativa: nada fuera de la lista, ni con barras codificadas ni con `..`.
for (const probe of [
  "/spec/vectors/..%2f..%2fpackage.json",
  "/spec/vectors/%2e%2e/%2e%2e/package.json",
  "/spec/vectors/../../package.json",
  "/package.json",
  "/spec/vectors/",
]) {
  const status = await new Promise((resolve, reject) => {
    const r = request({ host: "127.0.0.1", port, path: probe }, (res) => {
      res.resume();
      resolve(res.statusCode);
    });
    r.on("error", reject);
    r.end();
  });
  if (status !== 404) {
    console.error(`✖ el servidor de pruebas respondió ${status} a ${probe}`);
    failed = true;
  }
}
try {
  const tab = await browser.newPage();
  const external = [];
  tab.on("request", (r) => {
    if (!r.url().startsWith(`http://127.0.0.1:${port}/`)) external.push(r.url());
  });
  await tab.goto(`http://127.0.0.1:${port}/`);
  await tab.waitForFunction(() => window.__report !== undefined, null, { timeout: 30_000 });
  const report = await tab.evaluate(() => window.__report);
  for (const [name, expected] of Object.entries(SUITES)) {
    const r = report[name];
    const ok = r.failures.length === 0 && r.passed === expected;
    console.log(`${ok ? "✔" : "✖"} navegador ${name}: ${r.passed}/${expected}`);
    for (const f of r.failures) console.error(`  ${f}`);
    failed ||= !ok;
  }
  if (external.length > 0) {
    console.error(`✖ la página pidió recursos externos: ${external.join(", ")}`);
    failed = true;
  }
} finally {
  await browser.close();
  server.close();
}
process.exit(failed ? 1 : 0);
