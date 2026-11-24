// Pruebas del envoltorio: errores tipados y equivalencia con los vectores.
import { beforeAll, describe, expect, it } from "vitest";

import {
  CoreError,
  canonicalize,
  errorCodes,
  eventId,
  protocolVersion,
  verifyEnvelope,
} from "../src/index.ts";
import { loadCoreInNode, readVectors } from "./load.ts";

interface ValidVector {
  expect: { id: string; event: string };
}

beforeAll(loadCoreInNode);

describe("@social/core", () => {
  it("expone la versión del protocolo y los códigos de error", () => {
    expect(protocolVersion()).toBe(1);
    expect(errorCodes()).toContain("E_SIG_INVALID");
  });

  it("verifica un evento válido y calcula su id como el núcleo nativo", () => {
    const suite = JSON.parse(readVectors("envelope.json")) as { valid: ValidVector[] };
    for (const vector of suite.valid) {
      expect(verifyEnvelope(vector.expect.event)).toBe(vector.expect.event);
      expect(eventId(vector.expect.event)).toBe(vector.expect.id);
    }
  });

  it("lanza CoreError con el código estable", () => {
    expect.assertions(2);
    try {
      canonicalize('{"a":1.5}');
    } catch (err) {
      expect(err).toBeInstanceOf(CoreError);
      expect((err as CoreError).code).toBe("E_JSON_FLOAT");
    }
  });

  it("canonicaliza como RFC 8785", () => {
    expect(canonicalize('{ "b": [1, true], "a": "\\u20ac" }')).toBe('{"a":"€","b":[1,true]}');
  });
});
