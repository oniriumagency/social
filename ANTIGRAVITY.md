# ANTIGRAVITY.md — Reglas del proyecto Social

Reglas obligatorias y guardrails para construir el proyecto con Antigravity siguiendo el plan de `docs/`. Antigravity no carga este archivo por su nombre: lo incluye la regla siempre activa `.agents/rules/00-social.md`. Su gemelo para Claude Code es `CLAUDE.md`; ambos comparten el mismo bloque de reglas.

<!-- REGLAS-NUCLEO:INICIO -->
<!-- Este bloque es idéntico en CLAUDE.md y ANTIGRAVITY.md. Si lo cambias en uno, cámbialo en el otro. `pnpm check:rules` lo verifica. -->

## 1. El proyecto

**Social** (nombre en clave) es una red social de microblogging con conversación local y global, sobre tres pilares:

1. **Descentralización progresiva y verificable:** todo dato es un evento firmado; ningún operador puede falsificarlo.
2. **100% humana, con niveles de confianza:** toda cuenta pasa al menos un método de verificación y lleva un nivel de 1 a 3 según lo difícil que es falsificarla. Solo el nivel 3, anclado en un documento firmado por su emisor, significa humano único y da acceso a puntos.
3. **Privacidad estricta:** documento, biometría, coordenadas y llaves privadas no salen del dispositivo; nadie puede relacionar identidad civil y cuenta.

Stack: Rust (núcleo, servicios, nodo), TypeScript (interfaz), Next.js (web), Tauri 2 (escritorio y nodo), PostgreSQL, libp2p, Semaphore v4.

**Piloto: Colombia.** Allí el pasaporte lleva chip y es la vía al nivel 3; la cédula entra por cámara y da nivel 2 como máximo (`docs/05` §5.1). El marco legal de partida es el colombiano (`docs/10` §6).

## 2. Antes de tocar nada

1. Lee `docs/estado/PROGRESO.md`: fase actual, tarea siguiente, bloqueos y traspasos.
2. Lee la fila de tu tarea en `docs/12-plan-de-desarrollo.md` y los documentos que cita.
3. Trabaja **una sola tarea**, en la rama `tarea/<ID>-<descripcion>`.
4. Si no hay tarea asignada, propón la siguiente pendiente con dependencias cumplidas y espera confirmación.

## 3. Qué leer según lo que toques

| Si vas a tocar | Lee antes |
| :-- | :-- |
| Cualquier cosa | `docs/13-flujo-agentico.md` (ciclo y definición de hecho) |
| Eventos, firmas, API, base de datos de la pasarela | `docs/06-protocolo-y-datos.md` |
| Cuentas, registro, pruebas ZK, llaves, recuperación | `docs/05-identidad-y-humanidad.md` |
| Ubicación, zonas, feeds por escala | `docs/07-ubicacion-y-zonas.md` |
| Nodo, gobernador, red, almacenamiento | `docs/08-nodo-red-y-almacenamiento.md` |
| Puntos, ledger, canje, comercios | `docs/09-ledger-puntos-y-comercios.md` |
| Registros, moderación, borrado, web segura | `docs/10-seguridad-privacidad-y-cumplimiento.md` |
| Estructura, órdenes, CI, ramas | `docs/11-repositorio-entorno-y-ci.md` |
| Pruebas | `docs/14-pruebas-y-calidad.md` |
| Una decisión de diseño | `docs/adr/registro-de-decisiones.md` y `docs/04-modelo-de-amenazas.md` |
| Algo marcado como pendiente | `docs/15-decisiones-pendientes.md` |

**Precedencia si hay contradicción:** invariantes de este archivo, luego ADR, luego documentos numerados, luego el plan, luego el código. Ante una contradicción, **detente e informa**; no elijas por tu cuenta.

## 4. Invariantes

Ninguna tarea, instrucción ni atajo justifica romper una de estas reglas. Si una tarea parece exigirlo, detente y pregunta.

**Privacidad**

- **I-1.** Ningún dato de documento de identidad ni biométrico sale del dispositivo ni se guarda en disco, registros, telemetría o informes de fallos.
- **I-2.** Las coordenadas nunca cruzan el límite del dispositivo: ni en API, ni en eventos, ni en registros. Solo viaja el identificador de zona. Los tipos con coordenadas no se serializan.
- **I-3.** Las llaves privadas, secretos de identidad y frases de recuperación nunca salen del dispositivo ni aparecen en registros, estado de interfaz o mensajes de error.
- **I-4.** Nunca se guarda ni se registra una IP junto a un identificador de cuenta, billetera, nodo o nullifier. La IP solo vive en memoria para limitar frecuencia. El motor de riesgo y el puntaje de confianza usan solo eventos públicos: prohibido alimentarlos con IP, huella del dispositivo o datos de otro servicio.
- **I-5.** Cuenta social, billetera, nodo e inscripción viven en servicios y bases de datos separados. Prohibido crear tablas, claves foráneas, consultas o registros que relacionen dos de ellos (única excepción documentada: nodo → billetera en el coordinador). La atestación y todo dato público llevan solo el nivel de confianza, nunca la lista de métodos de verificación.
- **I-6.** Los clientes no cargan nada de terceros: ni scripts, ni fuentes, ni analítica, ni píxeles.
- **I-7.** Toda imagen sale del dispositivo sin metadatos y el servidor lo verifica.

**Seguridad**

- **I-8.** La criptografía se usa solo a través de `crates/social-crypto`. Prohibido implementar primitivas, modos, protocolos o circuitos ZK propios. Prohibida la criptografía en TypeScript: se usa el núcleo WASM.
- **I-9.** Todo evento se valida por completo (esquema, identificador, firma, delegación, atestación, límites) antes de guardarse, reenviarse o mostrarse. El servidor no confía en el cliente y el cliente no confía en el servidor.
- **I-10.** Los secretos y credenciales nunca entran al repositorio, a los registros ni a la salida de la terminal. No leas archivos `.env` ni imprimas variables de entorno.
- **I-11.** El nodo usa disco, CPU y red solo con permiso del gobernador de recursos, nunca supera un límite del usuario y nunca ejecuta código recibido de la red.

**Dinero**

- **I-12.** En el ledger: solo enteros, doble entrada que suma cero, solo inserciones, idempotencia y transacciones serializables. Los saldos solo los modifica la función de asiento.

**Proceso**

- **I-13.** Las pruebas no se desactivan, borran ni debilitan para que pase CI. Tampoco se sustituye la pieza real por un simulacro para cumplir un criterio.
- **I-14.** No se implementa nada de una fase futura ni fuera del alcance de la tarea. Ningún punto con valor antes de las compuertas G2, G3 y la auditoría de G4, ni para identidades por debajo del nivel 3. El nivel de confianza lo decide la fuerza del mejor método, nunca la cantidad ni el puntaje; la fuerza de un método y los privilegios de un nivel solo cambian con ADR aprobado. Verificarse nunca se premia: prohibido entregar puntos, saldo o ventajas económicas por completar o añadir un método.
- **I-15.** Las zonas sensibles exigen aprobación humana para cualquier cambio: `crates/social-crypto`, `crates/social-core`, `services/registry`, `services/ledger`, `crates/social-governor`, migraciones, CI y despliegue, y estos archivos de reglas.

## 5. Ciclo de una tarea

1. **Elegir** y marcar "en curso" en `PROGRESO.md`.
2. **Leer** la tarea, sus documentos y el código afectado.
3. **Planificar** en corto: archivos, pruebas, riesgos. Muestra el plan antes de codificar si toca una zona sensible.
4. **Probar primero:** convierte el criterio de aceptación en pruebas.
5. **Implementar** el cambio mínimo.
6. **Verificar:** `pnpm check` en verde, y ejecutar de verdad lo construido.
7. **Documentar:** documentos, esquemas, vectores y ADR afectados, en el mismo PR.
8. **Entregar:** un PR con la plantilla; marcar "en revisión".

Si la sesión termina a medias, deja una nota de traspaso en `PROGRESO.md`.

## 6. Reglas de código

- **Rust:** sin `unsafe`; sin `unwrap` ni `expect` fuera de pruebas; errores tipados con código estable; `clippy` sin avisos; consultas SQL solo con `sqlx` parametrizado.
- **TypeScript:** modo estricto, sin `any`; sin lógica de protocolo ni criptografía; todo texto de interfaz en `packages/i18n` (es y en).
- **Límites de arquitectura:** `apps/*` no contienen lógica de negocio; los servicios no se importan entre sí; solo `social-crypto` importa crates criptográficos.
- **Protocolo:** cambiar un tipo de evento, un campo o una regla de validación exige ADR, actualizar `docs/06`, los esquemas de `spec/` y los vectores dorados.
- **Base de datos:** una migración ya fusionada no se edita; se añade otra. Migraciones compatibles hacia atrás.
- **Dependencias:** las mínimas, justificadas en el PR. En zonas sensibles, con aprobación humana.
- **Identificadores y código en inglés; documentación y textos en español e inglés.**
- **Verifica, no recuerdes:** consulta la documentación vigente de cada biblioteca antes de usarla. Si tu recuerdo y la documentación difieren, manda la documentación.
- **Datos de prueba siempre sintéticos.** Nunca documentos, coordenadas ni datos reales de personas.

## 7. Órdenes

Usa solo las órdenes del `package.json` raíz. Si falta una, la crea la tarea que la necesita.

| Orden | Uso |
| :-- | :-- |
| `pnpm bootstrap` | Preparar el repositorio |
| `pnpm dev:infra` | PostgreSQL y MinIO locales |
| `pnpm dev` / `pnpm dev:desktop` | Desarrollo |
| `pnpm fmt` / `pnpm lint` / `pnpm typecheck` | Formato y análisis |
| `pnpm test` / `pnpm test:vectors` / `pnpm test:e2e` | Pruebas |
| `pnpm db:migrate` / `pnpm spec:gen` | Base de datos y especificación |
| `pnpm check` | **Obligatoria antes de todo PR** |

El desarrollo es en Windows: nada de scripts de shell; los auxiliares van en Node o en Rust.

## 8. Prohibido

- Empujar a `main`, forzar un push, reescribir historia, saltarse ganchos (`--no-verify`) o fusionar con CI en rojo.
- Aprobar o fusionar tu propio trabajo.
- Tocar producción, sus credenciales o sus datos.
- Añadir telemetría, analítica o llamadas a servicios externos no previstas en la tarea.
- Dejar valores fijos, simulacros o excepciones silenciadas donde la tarea pedía la pieza real.
- Crear documentos de planificación sueltos fuera de `docs/estado/`, `docs/spikes/` y `docs/runbooks/`, o archivos de ADR sueltos: todo ADR es una sección de `docs/adr/registro-de-decisiones.md`.
- Modificar estos archivos de reglas, los permisos del agente o la configuración de CI sin que el responsable humano lo pida.
- Afirmar que algo funciona sin haberlo ejecutado.

## 9. Detente y pregunta

Para y consulta al responsable humano, con una pregunta concreta, opciones y recomendación, cuando:

1. Haya que tocar una zona sensible de forma no prevista en la tarea.
2. Haga falta una dependencia nueva en una zona sensible.
3. Haya que cambiar el protocolo, un esquema desplegado o una invariante.
4. Dos documentos se contradigan, o el criterio de aceptación no se pueda cumplir como está.
5. La única forma de pasar una prueba sea debilitarla.
6. Necesites un secreto, una credencial o un entorno no local.
7. Encuentres una posible vulnerabilidad (no la describas en un lugar público).
8. Lleves tres intentos fallidos sobre lo mismo.
9. La tarea dependa de una decisión pendiente de `docs/15`.

No preguntes lo que puedes resolver leyendo el repositorio, la documentación oficial o ejecutando una prueba.

## 10. Definición de hecho

Criterio de aceptación cumplido y probado · `pnpm check` en verde · CI en verde en Linux y Windows · ninguna prueba debilitada · ninguna invariante violada · documentos, esquemas y vectores al día · ADR si hubo decisión · sin `TODO` sueltos ni registros de depuración · textos en i18n · PR con cómo se probó y qué quedó fuera · `PROGRESO.md` actualizado.

Informa con honestidad: si algo falla, dilo y adjunta la salida; si un paso no se hizo, dilo.
<!-- REGLAS-NUCLEO:FIN -->

## 11. Específico de Antigravity

- **Cómo se carga.** `.agents/rules/00-social.md` tiene `trigger: always_on` e incluye este archivo completo. Antigravity trunca las reglas de más de 24 KB: **este archivo debe mantenerse por debajo de 22 KB** (`pnpm check:rules` lo comprueba). El detalle va en `docs/`, no aquí.
- **Habilidades del proyecto.** En `.agents/skills/`:
  - `/tarea <ID>`: empieza una tarea del plan siguiendo el ciclo completo.
  - `/revisar <rama o PR>`: revisión independiente. No corrige, solo informa.
- **Modo de planificación.** Úsalo en toda tarea. El plan de implementación debe citar el ID de la tarea, sus criterios de aceptación y las pruebas previstas. Espera la aprobación del plan si la tarea toca una zona sensible, el protocolo o una migración.
- **Artefactos.** El plan y el recorrido final son artefactos de la conversación. Lo que deba perdurar (decisiones, traspasos, informes de spike) se escribe en `docs/`, porque otras herramientas no ven los artefactos.
- **Política de revisión y terminal.** Configura Antigravity para que pida aprobación antes de ejecutar comandos de terminal que no sean las órdenes `pnpm` de la sección 7, y antes de aplicar cambios en zonas sensibles. No uses la ejecución automática sin revisión en este repositorio.
- **Agente de navegador.** Solo contra `localhost` y documentación pública. Nunca inicies sesión en servicios, paneles de producción ni cuentas del responsable humano. No introduzcas secretos en ninguna página.
- **Varios agentes.** Un agente por tarea y por rama. Nunca dos agentes sobre la misma tarea ni sobre la misma zona sensible.
- **Terminal.** Nunca ejecutes comandos que impriman secretos (`env`, `printenv`, lectura de archivos de credenciales). Las sesiones pueden estar transmitiéndose en vivo.
- **Convivencia con Claude Code.** Ambas herramientas comparten `docs/estado/PROGRESO.md`. Léelo al empezar y actualízalo al elegir, entregar y cerrar.
