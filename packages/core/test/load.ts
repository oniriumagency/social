// Carga el núcleo WASM en Node desde el archivo generado.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { initCore } from "../src/index.ts";

export async function loadCoreInNode(): Promise<void> {
  const wasmUrl = new URL("../src/generated/social_core_bg.wasm", import.meta.url);
  await initCore(readFileSync(fileURLToPath(wasmUrl)));
}

export function readVectors(name: string): string {
  return readFileSync(new URL(`../../../spec/vectors/${name}`, import.meta.url), "utf8");
}
