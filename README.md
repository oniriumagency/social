# Social

Nombre en clave de una red social de microblogging con conversación local y global, 100% humana con niveles de confianza, privada y de descentralización progresiva.

**Estado:** fase F0 (cimientos). Piloto en Bogotá, Colombia. El esqueleto del monorepo está en marcha (ver `docs/estado/PROGRESO.md`). Este repositorio contiene el diseño auditado, el plan completo y las reglas para construirlo con herramientas de programación agéntica.

## Por dónde empezar

| Si eres | Lee |
| :-- | :-- |
| El responsable del proyecto | `docs/00-auditoria.md` y `docs/15-decisiones-pendientes.md` |
| Un agente (Claude Code) | `CLAUDE.md`, luego `docs/estado/PROGRESO.md` |
| Un agente (Antigravity) | `ANTIGRAVITY.md` (se carga solo), luego `docs/estado/PROGRESO.md` |
| Alguien nuevo | `docs/01-vision-y-producto.md` y `docs/02-arquitectura-y-funcionamiento.md` |

## Documentación

| Nº | Documento | Contenido |
| :-- | :-- | :-- |
| 00 | [Auditoría](docs/00-auditoria.md) | 42 hallazgos sobre el diseño original, cómo se corrigieron y las decisiones posteriores |
| 01 | [Visión y producto](docs/01-vision-y-producto.md) | Pilares, experiencia, economía, estrategia |
| 02 | [Arquitectura](docs/02-arquitectura-y-funcionamiento.md) | Componentes, qué es central en cada fase, flujos |
| 03 | [Stack](docs/03-stack-y-justificacion.md) | Decisiones técnicas, descartes, hoja de ruta |
| 04 | [Modelo de amenazas](docs/04-modelo-de-amenazas.md) | Activos, adversarios, mitigaciones, supuestos |
| 05 | [Identidad y humanidad](docs/05-identidad-y-humanidad.md) | Métodos de verificación, niveles, registro adaptativo, Colombia, puntaje, cuentas, recuperación |
| 06 | [Protocolo y datos](docs/06-protocolo-y-datos.md) | Eventos, validación, API, esquema de base de datos |
| 07 | [Ubicación y zonas](docs/07-ubicacion-y-zonas.md) | Catálogo, zonas de Colombia, reglas de privacidad, resolución local |
| 08 | [Nodo, red y almacenamiento](docs/08-nodo-red-y-almacenamiento.md) | Gobernador, red, réplicas, auditorías |
| 09 | [Ledger y comercios](docs/09-ledger-puntos-y-comercios.md) | Emisión, modelo de datos, canje, antifraude |
| 10 | [Seguridad y cumplimiento](docs/10-seguridad-privacidad-y-cumplimiento.md) | Desarrollo seguro, moderación, borrado, mapa legal de Colombia |
| 11 | [Repositorio y CI](docs/11-repositorio-entorno-y-ci.md) | Estructura, órdenes, git, integración continua |
| 12 | [Plan de desarrollo](docs/12-plan-de-desarrollo.md) | Fases, tareas, criterios de aceptación, compuertas |
| 13 | [Flujo agéntico](docs/13-flujo-agentico.md) | Ciclo de tarea, definición de hecho, cuándo preguntar |
| 14 | [Pruebas y calidad](docs/14-pruebas-y-calidad.md) | Niveles, invariantes, objetivos de rendimiento |
| 15 | [Decisiones](docs/15-decisiones-pendientes.md) | Decisiones ejecutivas tomadas y pendientes, con su valor por defecto |
| 16 | [Glosario](docs/16-glosario.md) | Términos del proyecto |
| — | [Registro de ADR](docs/adr/registro-de-decisiones.md) | Las 19 decisiones de arquitectura, en un solo archivo |
| — | [Estado](docs/estado/PROGRESO.md) | Qué está hecho y qué sigue |

## Reglas para agentes

- `CLAUDE.md` y `ANTIGRAVITY.md` contienen el mismo bloque de reglas e invariantes, más una sección propia de cada herramienta.
- `.agents/rules/00-social.md` hace que Antigravity cargue `ANTIGRAVITY.md` en cada conversación.
- `.claude/settings.json` deniega a Claude Code leer secretos y ejecutar comandos destructivos de git.
- `/tarea <ID>` y `/revisar <rama>` existen en ambas herramientas.
- `node scripts/check-agent-rules.mjs` comprueba que los dos archivos de reglas no se han desincronizado.

## Primeros pasos

1. Instalar los requisitos de `docs/11-repositorio-entorno-y-ci.md` §1 (Git, Rust con rustup, Node.js 22 o superior, pnpm 10, Docker Desktop).
2. `pnpm bootstrap` y luego `pnpm check`.
3. Leer `CONTRIBUTING.md` antes de abrir un PR.

## Seguridad

No abras incidencias públicas para vulnerabilidades: sigue `SECURITY.md`.

## Licencia

Apache-2.0 para protocolo, núcleo, SDK y herramientas; AGPL-3.0-only para servicios, nodo y aplicaciones. El reparto por carpeta está en `LICENSE`.
