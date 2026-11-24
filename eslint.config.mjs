// Configuración raíz de ESLint (flat config). Cada paquete puede extender
// `@social/config/eslint` con reglas propias de su entorno.
import base from "./packages/config/eslint.config.base.mjs";
import globals from "globals";

export default [
  {
    ignores: [
      "**/node_modules/**",
      "**/dist/**",
      "**/target/**",
      "**/.turbo/**",
      "**/pkg/**",
      "apps/**",
      "packages/*/src/generated/**",
    ],
  },
  ...base,
  {
    files: ["scripts/**/*.mjs", ".githooks/*", "*.mjs", "packages/*/test/**/*.mjs"],
    languageOptions: { globals: { ...globals.node, ...globals.browser } },
    rules: { "no-console": "off" },
  },
];
