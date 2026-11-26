# 11 - Repositorio, Entorno y CI

## 1. Requisitos de la máquina de desarrollo

El desarrollo principal ocurre en **Windows**. Todo debe funcionar igual en Windows, macOS y Linux.

| Herramienta | Uso |
| :-- | :-- |
| Git | Control de versiones |
| Rust (rustup, canal estable fijado en `rust-toolchain.toml`) | Núcleo, servicios, nodo |
| Destino `wasm32-unknown-unknown` y `wasm-pack` | Núcleo para web |
| Node.js LTS y pnpm (versión fijada en `package.json`) | Web e interfaz |
| Docker Desktop | PostgreSQL y almacenamiento S3 (SeaweedFS) en local |
| Requisitos de Tauri 2 para el sistema (WebView2 y herramientas de compilación de C++ en Windows) | Escritorio |
| `sqlx-cli`, `cargo-deny` | Base de datos y calidad (`pnpm bootstrap` instala `cargo-deny`) |
| `gitleaks` (opcional en local) | Escáner de secretos; CI lo ejecuta siempre |

Limitaciones a tener presentes:

- Compilar para iOS y macOS exige un Mac. Se cubre con CI en macOS hasta que haya uno.
- Las rutas con espacios y los finales de línea de Windows rompen scripts frágiles. Por eso no se usan scripts de shell.

## 2. Estructura del monorepo

```
social/
├── CLAUDE.md                  Reglas para Claude Code
├── ANTIGRAVITY.md             Reglas para Antigravity
├── README.md
├── SECURITY.md
├── .agents/rules/             Carga automática de reglas en Antigravity
├── .claude/                   Permisos y comandos de Claude Code
├── .github/                   CI, plantilla de PR, CODEOWNERS
├── Cargo.toml                 Workspace de Rust
├── package.json               Órdenes únicas del repositorio
├── pnpm-workspace.yaml
├── turbo.json
├── rust-toolchain.toml
├── deny.toml
├── docs/                      Esta documentación
├── spec/
│   ├── schemas/               Esquemas JSON de eventos (generados)
│   ├── vectors/               Vectores dorados
│   ├── methods/               Catálogo de métodos de verificación por país
│   └── openapi/               OpenAPI (generado)
├── crates/
│   ├── social-crypto/         Único punto de acceso a criptografía
│   ├── social-core/           Eventos, validación, llaves
│   ├── social-zones/          Catálogo y resolución de zonas
│   ├── social-core-wasm/      Enlaces WASM
│   ├── social-governor/       (F3) Gobernador de recursos
│   ├── social-net/            (F3) Red entre nodos
│   └── social-storage/        (F3) Fragmentos, cifrado, auditorías
├── services/
│   ├── gateway/               (F1) Pasarela
│   ├── registry/              (F2) Registro de humanidad
│   ├── coordinator/           (F3) Coordinador
│   └── ledger/                (F4) Ledger
├── apps/
│   ├── web/                   (F1) Next.js
│   ├── desktop/               (F1) Tauri + Vite
│   └── merchant/              (F4) Panel y TPV de comercios
├── packages/
│   ├── core/                  Envoltorio TypeScript del núcleo WASM
│   ├── app/                   Pantallas, estado, llamadas a la API
│   ├── ui/                    Componentes y sistema de diseño
│   ├── i18n/                  Textos (es, en)
│   └── config/                ESLint, TypeScript, Tailwind compartidos
├── tools/
│   ├── zone-pipeline/         Construcción del catálogo de zonas
│   └── xtask/                 Tareas de mantenimiento en Rust
├── infra/
│   ├── compose/               Docker Compose para desarrollo
│   └── deploy/                Despliegue (F1)
└── scripts/                   Scripts de Node multiplataforma
```

**Reglas de dependencias entre paquetes**

```
apps/*  ─▶ packages/app ─▶ packages/ui
                │
                └────────▶ packages/core ─▶ social-core-wasm ─▶ social-core ─▶ social-crypto
services/* ─▶ social-core, social-zones ─▶ social-crypto
```

1. Solo `social-crypto` importa crates criptográficos. `deny.toml` y una comprobación en CI lo hacen cumplir.
2. Ningún paquete de TypeScript importa una biblioteca criptográfica.
3. Los servicios no se importan entre sí. Se comunican por API.
4. `registry` y `ledger` no comparten crates de modelo de datos con `gateway`.
5. `apps/*` no contienen lógica de negocio: solo arranque, enrutado e integración con la plataforma.

## 3. Órdenes únicas

Definidas en el `package.json` raíz. Son las únicas que usan humanos, agentes y CI.

| Orden | Qué hace |
| :-- | :-- |
| `pnpm bootstrap` | Instala dependencias, `cargo-deny`, `wasm-bindgen-cli` (versión fijada) y Chromium para pruebas; compila el núcleo WASM en `packages/core/src/generated/`; prepara ganchos de git. |
| `pnpm dev:infra` | Levanta PostgreSQL y el almacenamiento S3 local (SeaweedFS) con Docker Compose. Acciones: `down`, `reset`, `status`. |
| `pnpm dev` | Arranca pasarela y web en modo desarrollo. |
| `pnpm dev:desktop` | Arranca la app de escritorio. |
| `pnpm fmt` | Formatea Rust y TypeScript. |
| `pnpm lint` | `cargo clippy` con avisos como errores, y ESLint. |
| `pnpm typecheck` | `cargo check` y `tsc`. |
| `pnpm test` | Pruebas unitarias y de integración de Rust y TypeScript. |
| `pnpm test:vectors` | Vectores dorados en nativo, en WASM sobre Node y en WASM dentro de Chromium (Playwright). |
| `pnpm test:e2e` | Playwright. |
| `pnpm db:migrate` | Aplica migraciones. |
| `pnpm spec:gen` | Regenera `spec/schemas/` (esquemas JSON de la envoltura y de cada tipo, desde la tabla de `social-core`). La OpenAPI se añade con la pasarela en F1. `pnpm spec:check` comprueba que no hay diferencias y forma parte de `pnpm check`. |
| `pnpm vectors:gen` | Regenera `spec/vectors/` desde `social-core`. Cambiar un vector existente es un cambio de protocolo y exige ADR. |
| `pnpm fuzz` | Fuzzing con cargo-fuzz (nightly fijada en `fuzz/rust-toolchain.toml`, solo en `fuzz/`): `json_parse`, `event_parse`, `event_validate` (bytes crudos) y `event_build` (construye y firma eventos con las llaves de prueba para llegar a papeles, delegaciones y atestación), sembrados con todas las listas de los vectores dorados. Por defecto 60 s por objetivo; `pnpm fuzz <objetivo> <segundos>`. Compila para el triple anfitrión que da `rustc -vV` en `fuzz/`, no para el destino por defecto de cargo-fuzz, que en un binario precompilado puede ser musl. CI lo ejecuta cada noche y conserva el corpus. `pnpm check:fuzz` comprueba que los objetivos compilan con su `Cargo.lock`. |
| `pnpm spike:gobernador` | Sonda del spike S4 (`tools/governor-probe`): lee cada segundo batería, red medida, temperatura, CPU, memoria, disco libre y actividad del usuario, y escribe un informe de texto sin datos que identifiquen el equipo. `pnpm spike:gobernador <segundos> <archivo>`; por defecto 600 s. |
| `pnpm check:rules` | Verifica que `CLAUDE.md` y `ANTIGRAVITY.md` comparten el mismo bloque de reglas. |
| `pnpm check:rust` / `pnpm check:ts` | Formato, lint, tipos y pruebas de un solo lenguaje. Son los trabajos `rust` y `ts` de CI. |
| `pnpm check:supply` | `cargo-deny` (licencias, orígenes, crates prohibidos, avisos de RustSec) y `pnpm audit`. |
| `pnpm check:secrets` | gitleaks sobre todo el historial. En local se omite con un aviso si gitleaks no está instalado; en CI es obligatorio. |
| `pnpm check` | **Todo lo anterior que debe pasar antes de un PR:** fmt, lint, typecheck, test, vectors, spec sin cambios, reglas, dependencias y secretos. |

Si una orden no existe todavía, la tarea que la necesita la crea. No se inventan órdenes alternativas.

## 4. Git y revisiones

- **Rama principal:** `main`, protegida. Nadie empuja directo.
- **Ramas:** `tarea/<ID>-<descripcion-corta>`, por ejemplo `tarea/F1-T08-feed-por-zona`.
- **Commits:** Conventional Commits en español o inglés, con el ID de tarea: `feat(gateway): feed por zona (F1-T08)`.
- **Un PR por tarea.** Si una tarea necesita más de unas 600 líneas de cambio, se divide.
- **Plantilla de PR** con: tarea, qué cambia, cómo se probó, lista de seguridad, documentos actualizados.
- **CODEOWNERS:** las zonas sensibles (`10` §8) exigen aprobación del responsable humano.
- **Fusión:** por squash, con CI en verde y una aprobación humana.
- Prohibido: reescribir historia en `main`, saltarse ganchos, fusionar con CI en rojo.

## 5. Integración continua

GitHub Actions. Matriz en `ubuntu-latest` y `windows-latest`; `macos-latest` para compilar escritorio.

| Trabajo | Contenido |
| :-- | :-- |
| `rust` | fmt, clippy, pruebas con `cargo test`, vectores dorados y deriva de `spec/schemas/` (`pnpm check:rust`, que incluye `spec:check`) |
| `wasm` | Compilación del núcleo a WASM y vectores en nativo, Node y navegador (`pnpm test:vectors`) |
| `ts` | ESLint, tsc, Vitest |
| `e2e` | Playwright contra pasarela y base de datos efímeras |
| `spec` | Cuando llegue la OpenAPI en F1: regenerarla y fallar si hay diferencias. Hoy la deriva de esquemas la comprueba el trabajo `rust` |
| `supply-chain` | `cargo-deny` (incluye los avisos de RustSec que revisa `cargo-audit`) y auditoría de pnpm |
| `secrets` | Escáner de secretos sobre el diff y el historial |
| `rules` | `pnpm check:rules`; límites de arquitectura |
| `privacy` | Prueba de registros limpios y prueba de tráfico sin coordenadas |
| `fuzz-build` | `pnpm check:fuzz`: los objetivos de `fuzz/` compilan y `fuzz/Cargo.lock` está al día |
| `fuzz` (programado) | Fuzzing nocturno de análisis y validación de eventos (y de mensajes de red cuando existan). Las dependencias de `fuzz/` (`libfuzzer-sys`, con licencia NCSA, y sus dependencias de compilación) quedan fuera de `cargo deny`: son solo de pruebas, no entran en ningún binario distribuido y su grafo no comparte el del workspace |
| `desktop-build` | Compilación de Tauri en los tres sistemas |

Reglas:

1. CI ejecuta exactamente las mismas órdenes que el desarrollador.
2. Ninguna prueba se marca como omitida para pasar. Una prueba inestable se arregla o se abre una incidencia con responsable.
3. Las acciones de terceros se fijan por hash de commit.
4. Los secretos de CI no están disponibles para PR desde forks.

## 6. Publicación y distribución

| Elemento | Requisito | Fase |
| :-- | :-- | :-- |
| Web | Despliegue desde `main` a preproducción; producción con aprobación manual. | F1 |
| Servicios | Imágenes de contenedor con etiqueta por commit; migraciones compatibles hacia atrás. | F1 |
| Escritorio Windows | **Certificado de firma de código.** Sin él, Windows muestra avisos que frenan la adopción. Tiene costo y días de trámite. | F1 beta, obligatorio en F3 |
| Escritorio macOS | **Cuenta de desarrollador de Apple**, firma y notarización. | F3 |
| Escritorio Linux | AppImage y paquete deb. | F3 |
| Actualizaciones | Firmadas; la app verifica antes de instalar; la llave de firma está fuera de CI. | F3 |
| Móvil | Cuentas de desarrollador y revisión de tiendas. Revisar políticas sobre P2P, recompensas y verificación de identidad antes de construir. | F2 |
| Reproducibilidad | Mejor esfuerzo: versiones fijadas y procedimiento de compilación documentado. | F3 |

## 7. Entornos

| Entorno | Datos | Acceso |
| :-- | :-- | :-- |
| Local | Sintéticos, generados por semilla | Todos |
| CI | Efímeros | Automático |
| Preproducción | Sintéticos | Equipo |
| Producción | Reales | Mínimo, con registro de accesos |

- Los agentes **nunca** reciben credenciales de producción.
- La configuración llega por variables de entorno. `.env.example` documenta cada variable sin valores reales.
- Existe un generador de datos sintéticos (`tools/xtask seed`) para feeds de prueba y de carga.
