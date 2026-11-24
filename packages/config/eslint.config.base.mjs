// Reglas comunes de ESLint para todo el TypeScript y JavaScript del repositorio.
// CLAUDE.md §6: modo estricto, sin `any`, sin criptografía en TypeScript (I-8).
import js from "@eslint/js";
import tseslint from "typescript-eslint";

/** Bibliotecas criptográficas prohibidas en TypeScript: la criptografía pasa por el núcleo WASM. */
const CRYPTO_LIBS = [
  "crypto",
  "node:crypto",
  "tweetnacl",
  "tweetnacl-util",
  "libsodium-wrappers",
  "libsodium-wrappers-sumo",
  "elliptic",
  "crypto-js",
  "jose",
  "@noble/*",
  "@scure/*",
  "@stablelib/*",
  "node-forge",
  "bcryptjs",
];

export default [
  js.configs.recommended,
  ...tseslint.configs.strict,
  {
    rules: {
      "@typescript-eslint/no-explicit-any": "error",
      "no-console": ["error", { allow: ["warn", "error"] }],
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: CRYPTO_LIBS,
              message:
                "Criptografía prohibida en TypeScript (I-8): usa @social/core (núcleo WASM).",
            },
          ],
        },
      ],
      "no-restricted-globals": [
        "error",
        {
          name: "crypto",
          message: "Criptografía prohibida en TypeScript (I-8): usa @social/core.",
        },
      ],
    },
  },
  {
    // Los scripts de Node del repositorio informan por consola a propósito.
    files: ["scripts/**/*.mjs", ".githooks/*"],
    rules: { "no-console": "off" },
  },
];
