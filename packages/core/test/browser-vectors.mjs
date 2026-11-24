#!/usr/bin/env node
// Vectores dorados en WASM dentro de un navegador real (Chromium con Playwright).
// Sirve el núcleo generado y spec/vectors/ desde un servidor local efímero en 127.0.0.1,
// abre una página que no carga nada de terceros (I-6) y ejecuta cada suite.

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join, normalize, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const SUITES = { "envelope.json": 25, "validation.json": 51 };
const TYPES = {
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
};
const ALLOWED = ["packages/core/src/generated/", "spec/vectors/"];

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
  const path = decodeURIComponent(new URL(req.url ?? "/", "http://x").pathname).slice(1);
  if (path === "") {
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    res.end(page);
    return;
  }
  const file = normalize(join(root, path));
  if (
    !file.startsWith(root) ||
    !ALLOWED.some((p) => path.startsWith(p)) ||
    file.includes(`..${sep}`)
  ) {
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
