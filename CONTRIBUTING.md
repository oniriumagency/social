# Contribuir / Contributing

Gracias por tu interés. Social está en fase F0 (cimientos) y avanza según un plan con
compuertas (`docs/12-plan-de-desarrollo.md`). Antes de proponer cambios grandes, abre
una incidencia para hablarlo.

## Antes de empezar

1. Lee `README.md`, `docs/estado/PROGRESO.md` y las invariantes de `CLAUDE.md` §4. Rigen
   para personas y agentes por igual.
2. Instala los requisitos de `docs/11-repositorio-entorno-y-ci.md` §1 y ejecuta
   `pnpm bootstrap` y luego `pnpm check`.

## Cómo trabajar

- **Una tarea por PR**, en la rama `tarea/<ID>-<descripcion>` (`docs/11` §4). Si una
  tarea supera unas 600 líneas de cambio, se divide.
- **Commits** con Conventional Commits y el ID de tarea:
  `feat(gateway): feed por zona (F1-T08)`.
- **`pnpm check` en verde** antes de abrir el PR. CI debe pasar en Linux y Windows.
- **Rellena la plantilla del PR**, incluida la lista de seguridad.
- **Zonas sensibles** (`docs/10` §8, `.github/CODEOWNERS`): exigen revisión del
  responsable humano.
- **Protocolo:** cambiar un tipo de evento, un campo o una regla de validación exige ADR
  y actualizar `docs/06`, `spec/` y los vectores dorados.
- **Datos de prueba siempre sintéticos.** Nunca documentos, coordenadas ni datos reales.
- **Nada de terceros en los clientes** y **criptografía solo a través de
  `crates/social-crypto`**.
- El desarrollo principal es en Windows: nada de scripts de shell; los auxiliares van en
  Node o en Rust.

## Licencia de tus aportes

El repositorio usa Apache-2.0 para protocolo, núcleo y SDK, y AGPL-3.0-only para
servicios, nodo y aplicaciones (`LICENSE`). Al enviar un PR aceptas que tu aporte se
publique bajo la licencia de la carpeta que modificas.

## Seguridad

Las vulnerabilidades no se reportan en incidencias públicas: sigue `SECURITY.md`.

---

**English.** One task per PR on a `tarea/<ID>-<description>` branch, Conventional
Commits with the task ID, `pnpm check` green, and the PR template filled in, including
the security checklist. Sensitive areas need human review. Test data must be synthetic.
Your contribution is licensed under the license of the folder you change (see
`LICENSE`). Report vulnerabilities privately as described in `SECURITY.md`.
