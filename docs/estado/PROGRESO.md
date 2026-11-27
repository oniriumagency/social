# Estado del Proyecto

**Este archivo es la memoria compartida entre sesiones y herramientas.** Todo agente lo lee al empezar y lo actualiza al elegir, entregar y cerrar una tarea. El detalle de cada tarea está en `docs/12-plan-de-desarrollo.md`.

- **Fase actual:** F0 — Cimientos y spikes de riesgo
- **Siguiente compuerta:** G0
- **País del piloto:** Colombia. Ciudad: Bogotá (D-01, confirmada el 2026-11-24)
- **Última actualización:** 2026-11-24 (F0-T01 a T11 y el spike S3 fusionados en `main`; D-04 y D-17 decididas)

## Estados

`pendiente` · `en curso` · `en revisión` · `hecha` · `bloqueada`

## Bloqueos actuales

| Qué | Depende de |
| :-- | :-- |
| Compuerta G0 | Los spikes S1, S2, S4 y S5 (pruebas físicas del responsable) |
| Spike S5 en Android, Apple, Linux y segundo dispositivo (no bloquea G0) | Un despliegue con HTTPS (decisión del responsable, 2026-11-26; `spikes/S5-llaves.md` §3.3) |

## F0 — Cimientos y spikes

| ID | Tarea | Estado | Herramienta | Rama o PR | Fecha |
| :-- | :-- | :-- | :-- | :-- | :-- |
| F0-T01 | Esqueleto del monorepo | hecha | Claude Code | `tarea/F0-T01-esqueleto-monorepo`, [#1](https://github.com/oniriumagency/social/pull/1) | 2026-11-24 |
| F0-T02 | Integración continua | hecha | Claude Code | `tarea/F0-T02-integracion-continua`, [#2](https://github.com/oniriumagency/social/pull/2) | 2026-11-24 |
| F0-T03 | Plantillas y gobierno | hecha | Claude Code | `tarea/F0-T03-plantillas-y-gobierno`, [#12](https://github.com/oniriumagency/social/pull/12) | 2026-11-24 |
| F0-T04 | Entorno local | hecha | Claude Code | `tarea/F0-T04-entorno-local`, [#4](https://github.com/oniriumagency/social/pull/4) | 2026-11-24 |
| F0-T05 | `social-crypto` | hecha | Claude Code | `tarea/F0-T05-social-crypto`, [#5](https://github.com/oniriumagency/social/pull/5) | 2026-11-24 |
| F0-T06 | `social-core`: envoltura de evento | hecha | Claude Code | `tarea/F0-T06-envoltura-evento`, [#6](https://github.com/oniriumagency/social/pull/6) | 2026-11-24 |
| F0-T07 | `social-core`: tipos y validación | hecha | Claude Code | `tarea/F0-T07-tipos-y-validacion`, [#7](https://github.com/oniriumagency/social/pull/7) | 2026-11-24 |
| F0-T08 | Núcleo en WASM | hecha | Claude Code | `tarea/F0-T08-nucleo-wasm`, [#8](https://github.com/oniriumagency/social/pull/8) | 2026-11-24 |
| F0-T09 | Esquemas y deriva | hecha | Claude Code | `tarea/F0-T09-esquemas-y-deriva`, [#9](https://github.com/oniriumagency/social/pull/9) | 2026-11-24 |
| F0-T10 | Fuzzing | hecha | Claude Code | `tarea/F0-T10-fuzzing`, [#10](https://github.com/oniriumagency/social/pull/10) | 2026-11-24 |
| F0-T11 | Comprobación de reglas de agentes | hecha | Claude Code | `tarea/F0-T11-reglas-en-ci`, [#3](https://github.com/oniriumagency/social/pull/3) | 2026-11-24 |
| F0-S1 | Spike: prueba de humanidad (Colombia) | pendiente | | | |
| F0-S2 | Spike: red entre nodos | pendiente | | | |
| F0-S3 | Spike: zonas | hecha | Claude Code | `tarea/F0-S3-zonas`, [#11](https://github.com/oniriumagency/social/pull/11) | 2026-11-24 |
| F0-S4 | Spike: gobernador | en curso | Claude Code | `tarea/F0-S4-gobernador` | 2026-11-26 |
| F0-S5 | Spike: llaves | en revisión | Claude Code | `tarea/F0-S5-llaves` | 2026-11-26 |
| G0 | Compuerta | pendiente | | | |

**Orden recomendado para empezar:** F0-T01, luego F0-T02 y F0-T04 en paralelo, luego F0-T05 → T06 → T07 → T08. Los spikes S1, S2, S4 y S5 pueden arrancar en cuanto exista F0-T01. S1 es el de mayor riesgo: conviene empezarlo cuanto antes.

## F1 a F5

Las tablas de estado de cada fase se añaden aquí cuando se aprueba la compuerta anterior.

## Traspasos

Notas de tareas que quedaron a medias. Formato:

```
### F0-T06 — 2026-12-01 — Claude Code
- Hecho:
- Falta:
- Probado:
- Preocupa:
```

### F0 (código) — 2026-11-24 — Claude Code
- Hecho: F0-T01 a T11 y el spike S3 fusionados en `main` (PR #1 a #12 y #14), con revisión independiente de los PR 5 a 10. CI en verde en Linux y Windows. El PR de prueba con fallo intencional (#13) quedó en rojo y se cerró sin fusionar.
- Hecho también (2026-11-24): protección de `main` activada por el responsable con un ruleset (PR obligatorio, 10 checks requeridos, incluido `fuzz-build`, sin bypass, sin force push ni borrado). Con esto se cumple el último criterio de F0-T02.
- Falta: dato oficial de barrios de IDECA y sus licencias para S3 (bloqueado por la red).
- Probado: bootstrap y check desde un clon limpio (T01); `pnpm dev:infra` con Docker real (T04); vectores en nativo, Node y Chromium (T08); fuzzing por objetivo sin fallos (T10).
- Preocupa: los spikes S1, S2, S4 y S5 necesitan dispositivos y redes reales.

### F0-T06 a F0-T10 — 2026-11-24 — Claude Code
- Hecho: revisión independiente de los PR 6 a 10 con todos los hallazgos requeridos corregidos. D-17 decidida: forma canónica obligatoria en el cable (`E_JSON_NOT_CANONICAL`).
- Falta (aplazado, no bloquea): sacar del WASM de producción el ejecutor y los generadores de vectores y las utilidades de prueba de `social_core::vectors` (hallazgo 6 del PR 6, 7 del PR 8); reglas de etiquetas `t` ligadas a la versión de Unicode (hallazgo 5 del PR 6); desempate de dos `device.grant` con el mismo `created_at` y regla escrita de a qué delegación afecta una revocación (para F1); afinar por papel las afirmaciones de `event_build`. El flujo nocturno `Fuzz` ya se probó de principio a fin (PR #15, run 45484629602).
- Preocupa: D-18 (revocación y eventos fechados antes) sigue abierta antes de F3.

## Historial de compuertas

| Compuerta | Fecha | Resultado | Informe |
| :-- | :-- | :-- | :-- |
| — | — | — | — |
