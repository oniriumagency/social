// Los esquemas publicados de spec/schemas/ y el núcleo coinciden sobre los vectores
// dorados (revisión de F0-T09): todo evento válido cumple su esquema, y todo inválido
// cuya regla expresa JSON Schema lo incumple. Lo que JSON Schema no expresa va en las
// claves x-* (x-rules, x-nfc, x-max-graphemes...) y se lista aquí con su motivo.
import { readdirSync, readFileSync } from "node:fs";

import { Ajv2020 } from "ajv/dist/2020.js";
import type { ValidateFunction } from "ajv/dist/2020.js";
import { describe, expect, it } from "vitest";

import { readVectors } from "./load.ts";

const schemaDir = new URL("../../../spec/schemas/", import.meta.url);

function readJson(url: URL): Record<string, unknown> {
  return JSON.parse(readFileSync(url, "utf8")) as Record<string, unknown>;
}

function extensionKeywords(value: unknown, out: Set<string>): Set<string> {
  if (Array.isArray(value)) {
    for (const item of value) extensionKeywords(item, out);
  } else if (typeof value === "object" && value !== null) {
    for (const [key, inner] of Object.entries(value)) {
      if (key.startsWith("x-")) out.add(key);
      extensionKeywords(inner, out);
    }
  }
  return out;
}

const envelopeSchema = readJson(new URL("envelope.schema.json", schemaDir));
const kindSchemas = readdirSync(new URL("kinds/", schemaDir)).map((file) => ({
  kind: file.replace(/\.schema\.json$/, ""),
  schema: readJson(new URL(`kinds/${file}`, schemaDir)),
}));

const ajv = new Ajv2020({ strict: true, allErrors: false });
const keywords = new Set<string>();
for (const s of [envelopeSchema, ...kindSchemas.map((k) => k.schema)]) {
  extensionKeywords(s, keywords);
}
for (const keyword of keywords) ajv.addKeyword(keyword);
const validateEnvelope = ajv.compile(envelopeSchema);
const validateKind = new Map<string, ValidateFunction>(
  kindSchemas.map(({ kind, schema }) => [kind, ajv.compile(schema)]),
);

/**
 * `true` si el evento cumple el esquema de la envoltura y, con `withKind`, el de su tipo
 * si es conocido. La suite de la envoltura solo cubre la envoltura.
 */
function schemaAccepts(raw: string, withKind: boolean): boolean {
  let event: unknown;
  try {
    event = JSON.parse(raw);
  } catch {
    return false;
  }
  if (!validateEnvelope(event)) return false;
  if (!withKind) return true;
  const { kind, content } = event as { kind: string; content: unknown };
  const validate = validateKind.get(kind);
  return validate === undefined || validate(content);
}

interface Suite {
  valid?: ({ event?: string; expect?: { event?: string } } | undefined)[];
  accept?: { event: string }[];
  invalid?: { name: string; event: string; expect: { error: string } }[];
}

const suites = ["envelope.json", "validation.json", "ids.json"].map((file) => ({
  ...(JSON.parse(readVectors(file)) as Suite),
  withKind: file !== "envelope.json",
}));

// Códigos cuya regla JSON Schema puede expresar: el esquema debe rechazar el evento.
const EXPRESSIBLE = new Set([
  "E_SCHEMA",
  "E_UNKNOWN_FIELD",
  "E_LIMIT_EXCEEDED",
  "E_VERSION_UNSUPPORTED",
]);

// Vectores con uno de esos códigos cuya regla vive en una clave x-* (no expresable).
const DECLARED_ONLY: Record<string, string> = {
  "etiqueta t en mayúsculas": "x-lowercase",
  "reply_to sin root": "x-rules",
  "publicación vacía": "x-rules",
  "501 grafemas": "x-max-graphemes",
  "nombre de perfil de 51 caracteres": "x-max-graphemes (51 caracteres son 51 grafemas)",
  "delegación de más de 90 días": "x-rules (depende de created_at)",
  "rotación con menos de 72 horas": "x-rules (depende de created_at)",
};

describe("esquemas de spec/schemas frente a los vectores dorados", () => {
  it("todo evento válido cumple su esquema", () => {
    const events = suites.flatMap((s) =>
      [
        ...(s.valid ?? []).map((v) => v?.expect?.event ?? v?.event ?? ""),
        ...(s.accept ?? []).map((v) => v.event),
      ].map((event) => ({ event, withKind: s.withKind })),
    );
    expect(events.length).toBeGreaterThan(150);
    const rejected = events
      .filter(({ event, withKind }) => !schemaAccepts(event, withKind))
      .map(({ event }) => event);
    expect(rejected).toEqual([]);
  });

  it("todo inválido con regla expresable incumple su esquema", () => {
    const accepted = suites
      .flatMap((s) => (s.invalid ?? []).map((v) => ({ ...v, withKind: s.withKind })))
      .filter((v) => EXPRESSIBLE.has(v.expect.error) && !(v.name in DECLARED_ONLY))
      .filter((v) => schemaAccepts(v.event, v.withKind))
      .map((v) => `${v.name} (${v.expect.error})`);
    expect(accepted).toEqual([]);
  });

  it("la lista de reglas no expresables no tiene entradas sobrantes", () => {
    const names = new Set(suites.flatMap((s) => s.invalid ?? []).map((v) => v.name));
    expect(Object.keys(DECLARED_ONLY).filter((n) => !names.has(n))).toEqual([]);
  });
});
