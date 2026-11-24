/**
 * @social/core: envoltorio TypeScript del núcleo de protocolo compilado a WASM.
 *
 * No contiene lógica de protocolo ni criptografía (I-8): cada función llama al núcleo
 * en Rust (`crates/social-core`) a través de `crates/social-core-wasm`. Así la web
 * valida exactamente igual que los servicios.
 *
 * Antes de usar cualquier función hay que llamar una vez a `initCore`.
 */

import initWasm, * as wasm from "./generated/social_core.js";

/** Lo que acepta `initCore`: URL, respuesta de `fetch`, bytes o módulo compilado del `.wasm`. */
export type CoreSource = Parameters<typeof initWasm>[0];

let ready: Promise<void> | undefined;

/** Carga el núcleo WASM. Llamarla varias veces es seguro. */
export function initCore(source?: CoreSource): Promise<void> {
  ready ??= initWasm(source === undefined ? undefined : { module_or_path: source }).then(
    () => undefined,
  );
  return ready;
}

/** Error del núcleo con su código estable (`E_SIG_INVALID`, `E_SCHEMA`...). */
export class CoreError extends Error {
  readonly code: string;

  constructor(code: string, detail: string) {
    super(detail === "" ? code : `${code}: ${detail}`);
    this.name = "CoreError";
    this.code = code;
  }
}

function call<T>(fn: () => T): T {
  try {
    return fn();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    const match = /^(E_[A-Z0-9_]+)(?::\s*(.*))?$/s.exec(message);
    if (match?.[1] !== undefined) {
      throw new CoreError(match[1], match[2] ?? "");
    }
    throw err;
  }
}

/** Versión de la envoltura que implementa el núcleo. */
export function protocolVersion(): number {
  return wasm.protocolVersion();
}

/**
 * Valida tamaño, esquema de la envoltura, identificador y firma (pasos 1 a 4 de
 * `docs/06` §3). Devuelve el evento en JSON canónico o lanza `CoreError`.
 */
export function verifyEnvelope(eventJson: string): string {
  return call(() => wasm.verifyEnvelope(eventJson));
}

/** Identificador hexadecimal de un evento verificado, o lanza `CoreError`. */
export function eventId(eventJson: string): string {
  return call(() => wasm.eventId(eventJson));
}

/** Forma canónica JCS de un JSON del protocolo, o lanza `CoreError`. */
export function canonicalize(json: string): string {
  return call(() => wasm.canonicalize(json));
}

/** Códigos de error estables del núcleo. */
export function errorCodes(): string[] {
  return JSON.parse(wasm.errorCodes()) as string[];
}

/** Resultado de ejecutar un archivo de vectores dorados. */
export interface VectorReport {
  passed: number;
  failures: string[];
}

/** Ejecuta un archivo de vectores dorados de `spec/vectors/` con el núcleo WASM. */
export function runVectorSuite(text: string): VectorReport {
  return JSON.parse(wasm.runVectorSuite(text)) as VectorReport;
}
