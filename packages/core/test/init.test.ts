// initCore se puede reintentar tras un fallo de carga (revisión de F0-T08).
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { initCore, protocolVersion } from "../src/index.ts";

describe("initCore", () => {
  it("vuelve a intentar la carga si la anterior falló", async () => {
    await expect(initCore(new Uint8Array([0, 1, 2, 3]))).rejects.toThrow();
    const wasmUrl = new URL("../src/generated/social_core_bg.wasm", import.meta.url);
    await initCore(readFileSync(fileURLToPath(wasmUrl)));
    expect(protocolVersion()).toBe(1);
  });
});
