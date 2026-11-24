// Pruebas del envoltorio: errores tipados y equivalencia con los vectores.
import { beforeAll, describe, expect, it } from "vitest";

import {
  CoreError,
  canonicalize,
  errorCodes,
  eventId,
  protocolVersion,
  verifyEnvelope,
  verifyEnvelopeBytes,
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

  it("calcula en WASM el mismo id que el nativo para todo el corpus generado", () => {
    const corpus = JSON.parse(readVectors("ids.json")) as {
      accept: { event: string; expect: { id: string } }[];
    };
    expect(corpus.accept.length).toBe(128);
    for (const vector of corpus.accept) {
      expect(eventId(vector.event)).toBe(vector.expect.id);
      expect(verifyEnvelopeBytes(new TextEncoder().encode(vector.event))).toBe(vector.event);
    }
  });

  it("valida los bytes exactos: UTF-8 inválido es E_JSON_INVALID", () => {
    const suite = JSON.parse(readVectors("envelope.json")) as { valid: ValidVector[] };
    const event = suite.valid[0]?.expect.event ?? "";
    const bytes = new TextEncoder().encode(event.replace('"hola"', '"hol\u0000"'));
    // Sustituto suelto codificado en UTF-8 (ED A0 80) en lugar de «a»: no es UTF-8 válido.
    const raw = new TextEncoder().encode(event);
    const at = raw.indexOf(0x61, event.indexOf('"hola"') + 4);
    const broken = new Uint8Array([...raw.slice(0, at), 0xed, 0xa0, 0x80, ...raw.slice(at + 1)]);
    expect(() => verifyEnvelopeBytes(broken)).toThrow(/E_JSON_INVALID/);
    expect(() => verifyEnvelopeBytes(bytes)).toThrow(CoreError);
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
