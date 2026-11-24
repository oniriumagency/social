// Vectores dorados en WASM sobre Node (docs/14 §3): mismos archivos que el núcleo nativo.
import { beforeAll, describe, expect, it } from "vitest";

import { runVectorSuite } from "../src/index.ts";
import { loadCoreInNode, readVectors } from "./load.ts";

beforeAll(loadCoreInNode);

describe("vectores dorados en WASM (Node)", () => {
  for (const [file, expected] of [
    ["envelope.json", 29],
    ["validation.json", 60],
    ["ids.json", 128],
  ] as const) {
    it(file, () => {
      const report = runVectorSuite(readVectors(file));
      expect(report.failures).toEqual([]);
      expect(report.passed).toBe(expected);
    });
  }
});
