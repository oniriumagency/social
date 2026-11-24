/* tslint:disable */
/* eslint-disable */

/**
 * Forma canónica JCS de un JSON del protocolo (sin `null` ni decimales).
 *
 * # Errors
 * Lanza un `Error` con el código estable si el JSON no es válido para el protocolo.
 */
export function canonicalize(json: string): string;

/**
 * Códigos de error estables del núcleo, en JSON (lista de cadenas).
 */
export function errorCodes(): string;

/**
 * Identificador hexadecimal de un evento ya verificado.
 *
 * # Errors
 * Lanza un `Error` con el código estable si el evento no se verifica.
 */
export function eventId(event_json: string): string;

/**
 * Versión de la envoltura que implementa el núcleo.
 */
export function protocolVersion(): number;

/**
 * Ejecuta un archivo de vectores dorados. Devuelve `{"passed":n,"failures":[...]}` en JSON.
 */
export function runVectorSuite(text: string): string;

/**
 * Pasos 1 a 4 de la validación universal (tamaño, esquema de la envoltura, id y firma).
 * Devuelve el evento en JSON canónico.
 *
 * # Errors
 * Lanza un `Error` con el código estable del primer paso que falla.
 */
export function verifyEnvelope(event_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly canonicalize: (a: number, b: number) => [number, number, number, number];
    readonly errorCodes: () => [number, number];
    readonly eventId: (a: number, b: number) => [number, number, number, number];
    readonly protocolVersion: () => number;
    readonly runVectorSuite: (a: number, b: number) => [number, number];
    readonly verifyEnvelope: (a: number, b: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
