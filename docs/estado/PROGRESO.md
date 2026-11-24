# Estado del Proyecto

**Este archivo es la memoria compartida entre sesiones y herramientas.** Todo agente lo lee al empezar y lo actualiza al elegir, entregar y cerrar una tarea. El detalle de cada tarea está en `docs/12-plan-de-desarrollo.md`.

- **Fase actual:** F0 — Cimientos y spikes de riesgo
- **Siguiente compuerta:** G0
- **País del piloto:** Colombia. Ciudad: Bogotá (D-01, confirmada el 2026-11-24)
- **Última actualización:** 2026-11-24 (Bogotá confirmada como ciudad piloto; decisión D-01: piloto en Colombia, ADR-019; los ADR pasan a un solo archivo, `docs/adr/registro-de-decisiones.md`)

## Estados

`pendiente` · `en curso` · `en revisión` · `hecha` · `bloqueada`

## Bloqueos actuales

| Qué | Depende de |
| :-- | :-- |
| Compuerta G0 | Decisión D-03 y los cinco spikes |

## F0 — Cimientos y spikes

| ID | Tarea | Estado | Herramienta | Rama o PR | Fecha |
| :-- | :-- | :-- | :-- | :-- | :-- |
| F0-T01 | Esqueleto del monorepo | en revisión | Claude Code | `tarea/F0-T01-esqueleto-monorepo`, [#1](https://github.com/oniriumagency/social/pull/1) | 2026-11-24 |
| F0-T02 | Integración continua | en revisión | Claude Code | `tarea/F0-T02-integracion-continua`, [#2](https://github.com/oniriumagency/social/pull/2) | 2026-11-24 |
| F0-T03 | Plantillas y gobierno | en revisión | Claude Code | `tarea/F0-T03-plantillas-y-gobierno`, [#12](https://github.com/oniriumagency/social/pull/12) | 2026-11-24 |
| F0-T04 | Entorno local | en revisión | Claude Code | `tarea/F0-T04-entorno-local`, [#4](https://github.com/oniriumagency/social/pull/4) | 2026-11-24 |
| F0-T05 | `social-crypto` | en revisión | Claude Code | `tarea/F0-T05-social-crypto`, [#5](https://github.com/oniriumagency/social/pull/5) | 2026-11-24 |
| F0-T06 | `social-core`: envoltura de evento | en revisión | Claude Code | `tarea/F0-T06-envoltura-evento`, [#6](https://github.com/oniriumagency/social/pull/6) | 2026-11-24 |
| F0-T07 | `social-core`: tipos y validación | en revisión | Claude Code | `tarea/F0-T07-tipos-y-validacion`, [#7](https://github.com/oniriumagency/social/pull/7) | 2026-11-24 |
| F0-T08 | Núcleo en WASM | en revisión | Claude Code | `tarea/F0-T08-nucleo-wasm`, [#8](https://github.com/oniriumagency/social/pull/8) | 2026-11-24 |
| F0-T09 | Esquemas y deriva | en revisión | Claude Code | `tarea/F0-T09-esquemas-y-deriva`, [#9](https://github.com/oniriumagency/social/pull/9) | 2026-11-24 |
| F0-T10 | Fuzzing | en revisión | Claude Code | `tarea/F0-T10-fuzzing`, [#10](https://github.com/oniriumagency/social/pull/10) | 2026-11-24 |
| F0-T11 | Comprobación de reglas de agentes | en revisión | Claude Code | `tarea/F0-T11-reglas-en-ci`, [#3](https://github.com/oniriumagency/social/pull/3) | 2026-11-24 |
| F0-S1 | Spike: prueba de humanidad (Colombia) | pendiente | | | |
| F0-S2 | Spike: red entre nodos | pendiente | | | |
| F0-S3 | Spike: zonas | en revisión | Claude Code | `tarea/F0-S3-zonas`, [#11](https://github.com/oniriumagency/social/pull/11) | 2026-11-24 |
| F0-S4 | Spike: gobernador | pendiente | | | |
| F0-S5 | Spike: llaves | pendiente | | | |
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
- Hecho: F0-T01, T02, T04 a T11 implementadas en ramas apiladas (`tarea/F0-T01-...` → `tarea/F0-T10-fuzzing`, cada una sale de la anterior). `pnpm check` en verde en Linux en cada una.
- Hecho también: ramas subidas y un PR por tarea (#1 a #12, apilados: se fusionan en ese orden).
- Falta: comprobar CI en Windows; activar la protección de `main` (H); PR de prueba con fallo intencional (criterio de F0-T02). F0-T03 hecha tras decidir D-04 (rama sobre S3).
- Probado: bootstrap y check desde un clon limpio (T01); `pnpm dev:infra` con Docker real (T04); 76 vectores en nativo, Node y Chromium (T08); 10 minutos de fuzzing por objetivo sin fallos (T10).
- Spike S3 (`tarea/F0-S3-zonas`, sale de T10): informe en `docs/spikes/S3-zonas.md` y prototipo `tools/zone-pipeline`. Ciudad y 20 localidades completas con datos abiertos; barrios de OSM insuficientes (1,7 % de la ciudad); búsqueda en WASM de 2 µs. Falta el dato oficial de IDECA (bloqueado por la red) y sus licencias.
- Preocupa: CI en Windows no se ha podido ejecutar todavía. MinIO ya no publica imágenes comunitarias: se usa SeaweedFS en local (`docs/03`, `docs/11`). Los spikes S1, S2, S4 y S5 necesitan dispositivos y redes reales.

## Historial de compuertas

| Compuerta | Fecha | Resultado | Informe |
| :-- | :-- | :-- | :-- |
| — | — | — | — |
