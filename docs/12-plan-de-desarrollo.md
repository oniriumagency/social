# 12 - Plan de Desarrollo

Plan completo de cero a fin. Cada tarea está pensada para que un agente la complete en **un solo PR**. El estado de cada tarea se lleva en `docs/estado/PROGRESO.md`. El procedimiento para ejecutar una tarea está en `13-flujo-agentico.md`.

## 1. Cómo leer este plan

- **ID:** `F<fase>-T<número>` para tareas, `F0-S<número>` para spikes, `G<fase>` para compuertas.
- **Dep.:** tareas que deben estar terminadas antes.
- **Criterio de aceptación:** condición verificable. Además, toda tarea cumple la definición de hecho de `13` §4.
- **H:** la tarea exige intervención humana (decisión, cuenta externa, dispositivo físico o aprobación).
- Las fases F0 y F1 están detalladas al nivel de tarea. Las fases F2 a F5 están detalladas al nivel necesario para planificar; **la primera tarea de cada una es refinarla** con lo aprendido, y ese refinamiento lo aprueba un humano.

**Reglas del plan**

1. No se empieza una fase sin aprobar la compuerta anterior.
2. No se implementa nada de una fase futura "ya que estamos".
3. Si una tarea resulta mayor de lo previsto, se divide y se actualiza este documento en el mismo PR.
4. Si una tarea contradice otro documento, se detiene y se resuelve con un ADR.

---

## 2. F0 — Cimientos y spikes de riesgo

**Objetivo:** un repositorio donde cualquier agente pueda trabajar con seguridad, el núcleo de protocolo terminado, y los cinco mayores riesgos técnicos medidos con datos reales.

| ID | Tarea | Entregable y criterio de aceptación | Dep. |
| :-- | :-- | :-- | :-- |
| F0-T01 | Esqueleto del monorepo | Workspaces de Cargo y pnpm, Turborepo, `rust-toolchain.toml`, `.editorconfig`, `.gitattributes` con finales de línea normalizados, órdenes de `11` §3 creadas (pueden estar vacías). `pnpm bootstrap` y `pnpm check` terminan bien en Windows y Linux. | — |
| F0-T02 | Integración continua | Flujos de `11` §5 para lo que ya existe: rust, ts, supply-chain, secrets, rules. Un PR de prueba con un fallo intencional queda en rojo. Protección de `main` activada (H). | T01 |
| F0-T03 | Plantillas y gobierno | Plantilla de PR con lista de seguridad, CODEOWNERS para zonas sensibles, `SECURITY.md`, `CONTRIBUTING.md`, archivos de licencia según D-04 (H). | T01 |
| F0-T04 | Entorno local | Docker Compose con PostgreSQL y MinIO, `.env.example`, `pnpm dev:infra`. Arranca en Windows con Docker Desktop. | T01 |
| F0-T05 | `social-crypto` | Envoltorios de Ed25519 (verificación estricta), SHA-256, BLAKE3, XChaCha20-Poly1305, HKDF, aleatoriedad; tipos de secreto con `zeroize` y sin `Debug`. Pasan los vectores del RFC 8032 y los de Wycheproof para Ed25519. CI falla si otro crate importa una biblioteca criptográfica. | T01 |
| F0-T06 | `social-core`: envoltura de evento | Tipos, canonicalización JCS, cálculo de `id`, firma y verificación, códigos de error estables. Vectores dorados en `spec/vectors/` (válidos e inválidos: decimales, `null`, campos extra, firma no canónica, tamaño). Pruebas de propiedades de ida y vuelta. | T05 |
| F0-T07 | `social-core`: tipos de evento y validación | Esquemas de todos los `kind` de `06` §4 con sus límites; validación universal (pasos 1 a 6) con un almacén de delegaciones y atestaciones abstracto. Un vector dorado por cada regla de rechazo. | T06 |
| F0-T08 | Núcleo en WASM | `social-core-wasm` y `packages/core` con tipos de TypeScript generados. Los mismos vectores dorados pasan en Node y en navegador (Playwright). `pnpm test:vectors` los ejecuta en los tres entornos. | T07 |
| F0-T09 | Esquemas y deriva | `pnpm spec:gen` genera los esquemas JSON desde Rust. CI falla si el resultado difiere del guardado. | T07 |
| F0-T10 | Fuzzing | Objetivos de fuzzing para análisis y validación de eventos. Flujo nocturno programado. Diez minutos de ejecución local sin fallos. | T07 |
| F0-T11 | Comprobación de reglas de agentes | `scripts/check-agent-rules.mjs` integrado en `pnpm check:rules` y en CI. | T02 |
| F0-S1 | **Spike: prueba de humanidad** (H) | Informe `docs/spikes/S1-humanidad.md` que responde: (1) qué proveedores de prueba ZK de documento existen hoy, con licencia, estado y auditorías; (2) si la verificación se puede hacer en nuestro servidor sin depender de una cadena de bloques; (3) si la prueba admite ligar datos propios (compromiso y reto); (4) cómo deriva el nullifier y qué revela; (5) **inventario de documentos de Colombia** a partir de `05` §5.1: cuántos adultos tienen pasaporte con chip vigente, si el proveedor acepta los certificados de firma del modelo anterior y del modelo de 2026, y qué lleva de verdad la cédula digital (consulta a la Registraduría); (6) prueba real con al menos un pasaporte colombiano de cada modelo, una cédula de cada formato y dos teléfonos; (7) Semaphore v4: generación de prueba en navegador y en Tauri, y verificación en Rust, con tiempos; (8) restricciones legales en Colombia para leer el pasaporte y la cédula; (9) **métodos sin NFC en Colombia**: credenciales digitales firmadas que existan (app Cédula Digital Colombia); viabilidad de `doc-app` con Play Integrity y App Attest, incluida la derivación del nullifier con una función seudoaleatoria oblivia; viabilidad de `cuenta` con correos firmados de bancos y billeteras digitales colombianos; y para cada método, costo estimado de falsificación y fuerza propuesta. (10) **primera versión del catálogo de métodos** (`spec/methods/`) para Colombia, que confirma o corrige el borrador de `05` §5, con la fuente de cada dato: qué documentos llevan chip verificable y desde cuándo, y qué métodos son viables. Termina con una recomendación de qué métodos lanzar en F2 y un borrador de ADR. | T01 |
| F0-S2 | **Spike: red entre nodos** (H) | Informe `docs/spikes/S2-red.md`: dos nodos rust-libp2p en redes domésticas distintas; tasa de éxito de conexión directa frente a relevo; consumo de un relevo; transferencia de 100 MB con verificación BLAKE3 por rangos; estado actual de las alternativas de transferencia de bloques. | T01 |
| F0-S3 | **Spike: zonas** | Informe `docs/spikes/S3-zonas.md` y prototipo de `tools/zone-pipeline` para Bogotá (D-01): fuentes de `07` §2.1 y sus licencias; cobertura de barrio y localidad; tamaño del paquete; búsqueda en WASM por debajo de 5 ms; comportamiento en los bordes. | T08 |
| F0-S4 | **Spike: gobernador** (H) | Informe `docs/spikes/S4-gobernador.md`: lectura real de batería, red medida, temperatura y actividad en Windows, macOS y Linux desde un proceso Tauri; latencia de cada señal; crates elegidos y su estado de mantenimiento. Actualiza la tabla de `08` §2.1. | T01 |
| F0-S5 | **Spike: llaves** (H) | Informe `docs/spikes/S5-llaves.md`: soporte de la extensión PRF de WebAuthn por navegador y sistema; llaves Ed25519 no extraíbles en navegadores; almacén seguro en Tauri por sistema; propuesta final de derivación y copia de seguridad. | T08 |
| **G0** | **Compuerta** (H) | Los cinco informes revisados. ADR confirmados o corregidos. Decisiones D-01 (ciudad incluida) a D-04 resueltas. Documentos `05`, `07` y `08` actualizados con los resultados. | Todo F0 |

**El método fuerte de Colombia es el pasaporte con chip** (`05` §5.1). Si el spike S1 concluye que no es viable, porque el proveedor no acepta sus certificados o porque muy poca gente lo tiene, la red puede abrir con niveles 1 y 2, pero la Fase 4 (puntos) queda bloqueada, porque los puntos exigen nivel 3. En ese caso G0 decide entre cambiar de proveedor, desplegar puntos de inscripción, esperar a un método fuerte o revisar la decisión D-15 con un ADR. No se abren los puntos a niveles inferiores para salir del paso.

---

## 3. F1 — Core social (alfa cerrada por invitación)

**Objetivo:** un producto usable por un grupo cerrado: publicar, responder y leer feeds por zona, en web y escritorio. Sin puntos, sin nodo, sin prueba de documento. El acceso es por invitación, usando la misma ruta de atestación que usará F2.

### Servicios

| ID | Tarea | Entregable y criterio de aceptación | Dep. |
| :-- | :-- | :-- | :-- |
| F1-T01 | Esqueleto de la pasarela | `services/gateway` con axum: configuración por entorno, errores tipados con los códigos del núcleo, trazas, `GET /v1/info`, cierre ordenado. La capa de registros redacta por tipo: una prueba demuestra que un secreto o una IP no llegan a la salida. | G0 |
| F1-T02 | Base de datos | Migraciones de `06` §9 con `sqlx`; particiones mensuales de `post_zones` creadas por una tarea programada; `pnpm db:migrate`. Las consultas se verifican en compilación. | T01 |
| F1-T03 | Ingesta | `POST /v1/events` con validación completa (pasos 1 a 8), inserción idempotente y almacenes reales de delegaciones y atestaciones. Los vectores dorados inválidos se rechazan con su código exacto. | T02 |
| F1-T04 | Atestación de alfa | `InviteProvider` detrás de `HumanityProvider`; herramienta de administración para emitir invitaciones de un solo uso; emisión de `account.attest` con `tier: alpha`. Una invitación no sirve dos veces, ni bajo carrera. | T03 |
| F1-T05 | Delegación de dispositivos | Proyección de `device.grant` y `device.revoke`. Un evento firmado por un dispositivo revocado o caducado se rechaza. | T03 |
| F1-T06 | Catálogo de zonas v0 | `tools/zone-pipeline` produce el catálogo y los paquetes de la ciudad piloto más la escalera país, continente y global para todo el mundo; carga en la tabla `zones`; paquetes firmados. | G0, S3 |
| F1-T07 | `social-zones` | Resolución en el dispositivo según `07` §4, adherencia de bordes, subida por k-anonimato. El tipo de coordenadas no es serializable (prueba de compilación). Pruebas de `07` §8. | T06 |
| F1-T08 | Feed por zona | Proyección a `posts` y `post_zones`; `GET /v1/feed` con cursor; `GET /v1/zones/*`; `zone_stats` por lotes y publicada por intervalos. p95 menor a 150 ms con 10 millones de publicaciones sintéticas. | T03, T06 |
| F1-T09 | Hilos, republicaciones y borrado | `reply_to` y `root`, `GET /v1/events/{id}/thread`, `post.repost`, `post.delete` con lápida y eliminación de contenido. | T08 |
| F1-T10 | Seguimiento, reacciones y perfil | `follow.set`, `reaction.set`, `account.profile`, feed de seguidos, endpoints de cuenta. | T08 |
| F1-T11 | Niveles altos por puntuación | Muestra por lotes para país, continente y global con la fórmula documentada. | T10 |
| F1-T12 | Media v0 | Subida prefirmada, `cid` BLAKE3, verificación de tipo, tamaño y **ausencia de metadatos**, recodificación. Una imagen con GPS en EXIF se rechaza. | T03 |
| F1-T13 | Tiempo real | `GET /v1/stream` con suscripción a zonas e hilos, contrapresión y límites por conexión. | T08 |
| F1-T14 | Límites de frecuencia | Límites de `06` §8 por cuenta; por IP solo en memoria con hash y sal rotatoria. Prueba de que nada relacionado con la IP llega a disco. | T03 |
| F1-T15 | Moderación v0 | `report.create`, cola de revisión, `mod.action` (ocultar, retirar, suspender), lista de bloqueo de media por hash, panel interno mínimo. | T09, T12 |

### Clientes

| ID | Tarea | Entregable y criterio de aceptación | Dep. |
| :-- | :-- | :-- | :-- |
| F1-T16 | Sistema de diseño | `packages/ui`: componentes base, temas claro y oscuro, accesibilidad AA. `packages/i18n` con español e inglés; ningún texto fijo en componentes (regla de lint). | G0 |
| F1-T17 | Cliente de API y estado | `packages/app`: cliente tipado desde OpenAPI, verificación de firma de todo evento recibido, caché de consultas. | T03, F0-T08 |
| F1-T18 | Gestión de llaves en el cliente | Generación de llave raíz y de dispositivo, delegación, almacenamiento según el resultado de S5, frase de recuperación con confirmación. La llave privada nunca sale del módulo de llaves ni aparece en el estado de la interfaz. | T17, S5 |
| F1-T19 | Web: bienvenida | Crear cuenta con invitación, copia de seguridad de la frase, elección de zona (manual o con sugerencia), explicación clara de la privacidad de ubicación. | T04, T07, T18 |
| F1-T20 | Web: feed y composición | Selector de escala (barrio a global) y de zona concreta, composición con zona y modo, adjuntar imágenes con limpieza de metadatos en el cliente. | T08, T12, T19 |
| F1-T21 | Web: hilo, perfil y social | Hilo, perfil, seguir, reaccionar, republicar, borrar, silenciar, bloquear, reportar. | T09, T10, T20 |
| F1-T22 | Web: endurecimiento | CSP y cabeceras de `10` §2; cero recursos de terceros (prueba automática); páginas públicas de lectura con renderizado en servidor. | T20 |
| F1-T23 | Escritorio: cáscara Tauri | `apps/desktop` con las mismas pantallas, llaves en el almacén seguro del sistema, enlace de dispositivo por QR con la web. Compila en los tres sistemas. | T21, S5 |

### Operación y cierre

| ID | Tarea | Entregable y criterio de aceptación | Dep. |
| :-- | :-- | :-- | :-- |
| F1-T24 | Observabilidad | Métricas y trazas con las reglas de `10` §5; tableros; prueba automática de registros limpios. | T03 |
| F1-T25 | Pruebas de extremo a extremo | Playwright: alta, publicar, feed por cada escala, hilo, borrar, reportar. Incluye la prueba de tráfico sin coordenadas. | T21 |
| F1-T26 | Pruebas de carga | Generador de datos sintéticos; escenarios k6 de lectura y escritura; informe con cuellos de botella. | T11 |
| F1-T27 | Preproducción | Despliegue reproducible, copias de seguridad con restauración probada, manuales de operación e interruptores de emergencia (H). | T24 |
| F1-T28 | Revisión de seguridad de F1 | Revisión contra `04` y `10`; informe con hallazgos y correcciones. | T25 |
| **G1** | **Compuerta** (H) | Alfa abierta a invitados reales. Sin hallazgos críticos abiertos. Decisión D-09 (política de contenido) resuelta. Política de tratamiento de datos, autorización y canal de consultas y reclamos publicados y revisados por un abogado en Colombia (`10` §4). Métricas de `01` §6 en seguimiento. | Todo F1 |

---

## 4. F2 — Humanidad (beta pública)

**Objetivo:** sustituir las invitaciones por los métodos de verificación reales, con niveles de confianza, y abrir al público. Los métodos concretos de F2 los fija el spike S1; como mínimo `dispositivo`, `aval` y un método de fuerza fuerte.

| ID | Tarea | Entregable y criterio de aceptación | Dep. |
| :-- | :-- | :-- | :-- |
| F2-T00 | Refinamiento de fase (H) | Este apartado se reescribe con el resultado de S1 y lo aprendido en F1. Lista cerrada de métodos de F2 con su fuerza. | G1 |
| F2-T01 | Servicio de registro | `services/registry` con base de datos y credenciales aisladas, claves de firma en KMS, reto de un solo uso. | T00 |
| F2-T02 | Motor de métodos y niveles | Registro de métodos por compromiso con nullifier por método; cálculo de nivel por combinación más fuerte según `05` §4, como tabla de configuración versionada. Pruebas de propiedades: ninguna combinación de métodos bajos supera el nivel 1; añadir un método nunca baja el nivel. | T01 |
| F2-T03 | Grupos de humanos | Un árbol de Semaphore v4 por nivel, inserción por lotes cada 10 minutos, historial de raíces, descarga del camino por bloques. | T02 |
| F2-T04 | Identidad en el cliente | Derivación del secreto de identidad (según S5), generación de la prueba Semaphore en WASM, copia cifrada de la llave raíz. | T03 |
| F2-T05 | Alta de cuenta y subida de nivel | Verificación de la prueba, nullifier por ámbito y época, atestación con `level`. Dos altas con la misma identidad en la misma época: la segunda se rechaza, también bajo carrera. Subir de nivel conserva la cuenta. | T03, T04 |
| F2-T06 | Método `dispositivo` | Atestación de dispositivo y passkey. Nivel 1. | T02 |
| F2-T07 | Método `chip` | Integración del proveedor de prueba ZK de documento: verificación en servidor, ligadura a compromiso y reto, edad, lista de emisores. Flujo con teléfono propio y con teléfono ajeno por QR. Una prueba repetida o ligada a otro compromiso se rechaza. | T02 |
| F2-T08 | Método `aval` | Aval con prueba de pertenencia a G3 y nullifier por cupo. Un humano no puede superar su cupo, tampoco bajo carrera. | T05 |
| F2-T09 | Métodos de fuerza media validados en S1 | `doc-app` y `cuenta`, cada uno en su propio PR y solo si S1 lo aprobó. Cada uno con su suite de ataque. | T02 |
| F2-T10 | Privilegios por nivel | Pasarela: validación del nivel por tipo de evento, límites por nivel, peso en feeds, conteo de k-anonimato. Clientes: insignia y filtro por nivel mínimo. | T05 |
| F2-T23 | Catálogo de métodos por país | Formato y esquema de `spec/methods/`, firma, distribución completa por CDN, validación en CI (todo país referencia métodos existentes; toda entrada cita fuente). Función pura en el núcleo: `opciones(catálogo, país, capacidades)`. Pruebas de propiedades: nunca devuelve un método que el país o el dispositivo no admiten. | T02 |
| F2-T24 | Puntaje de confianza, motor de riesgo y verificación adicional | Puntaje por cuenta con bandas por nivel; motor de riesgo que solo lee eventos públicos; retos de presencia y de método con plazos; pausa reversible; `account.presence`; `POST /v1/accounts/status`. Pruebas: el puntaje nunca supera el tope de su nivel ni concede privilegios de otro nivel; ignorar penaliza una sola vez; el módulo de riesgo no puede leer IP ni otras bases de datos. | T10 |
| F2-T11 | Renovación de época | Renovación automática en el cliente; vencimiento y gracia en la validación. | T05 |
| F2-T12 | Recuperación | Escalera de `05` §11 completa, incluida la repetición de método con su límite y la rotación con retraso. | T05 |
| F2-T13 | Ruta que oculta la IP | HTTP Oblivious o Tor para inscripción, alta y renovación, con un relevo operado por un tercero (H). | T05 |
| F2-T14 | Experiencia de registro adaptativo | Detección local de capacidades, elección del país emisor, opciones según el catálogo (`05` §5), paso por QR al teléfono, explicación honesta de qué prueba cada método y qué nivel da, invitación a subir de nivel, accesibilidad. Prueba de extremo a extremo por cada situación de la tabla de `05` §5. La lista de capacidades no sale del dispositivo. | T06, T07, T08, T23 |
| F2-T15 | Vales ciegos y cuentas secundarias | Firmas RSA ciegas (RFC 9474), integración de pagos (H), cuentas secundarias con tope, solo nivel 3. | T05 |
| F2-T16 | Registro de transparencia | Registro público, solo de inserción, de raíces de los grupos y atestaciones emitidas; verificador independiente. | T03 |
| F2-T17 | Migración de la alfa | Las cuentas `alpha` pasan a `base` con el nivel que obtengan; las que no se verifiquen caducan. | T05 |
| F2-T18 | Cliente móvil | Cliente ligero en móvil; revisión de políticas de tiendas (H). | T14 |
| F2-T19 | Apelaciones y transparencia de moderación | Canal de apelación e informe periódico. | T00 |
| F2-T20 | Evaluación de impacto de privacidad y revisión legal en Colombia (H) | Dictamen sobre las filas de `10` §6 que vencen en G2, incluidos el tratamiento en el dispositivo y el vínculo entre nivel 3 y puntos. Documento firmado; ajustes aplicados. | T14 |
| F2-T21 | Pruebas de ataque | Suite que intenta: doble inscripción por método, repetición de prueba, alta duplicada bajo carrera, vale gastado dos veces, superar el cupo de avales, alcanzar nivel 2 solo con métodos bajos, deducir los métodos a partir de datos públicos, relacionar inscripción y cuenta en la base de datos, subir de nivel a base de superar retos, y relacionar una cuenta con su identidad a través de un reto. Todas fallan. | T15, T24 |
| F2-T22 | **Auditoría externa** (H) | Auditoría del registro, del motor de niveles y de la integración ZK. Hallazgos críticos y altos corregidos. | T21 |
| **G2** | **Compuerta** (H) | Auditoría aprobada, dictamen legal de F2 favorable, beta pública abierta en el país piloto. | Todo F2 |

El método `presencial` y los puntos de inscripción para `chip` dependen de la red de comercios; se planifican en F4-T06 o antes si hay puntos piloto.

---

## 5. F3 — Nodo y almacenamiento

**Objetivo:** que cualquier usuario de escritorio pueda aportar almacenamiento sin riesgo para su equipo, y medir la red con puntos de prueba.

| ID | Tarea | Entregable y criterio de aceptación | Dep. |
| :-- | :-- | :-- | :-- |
| F3-T00 | Refinamiento de fase (H) | Reescritura con S2, S4 y lo aprendido. ADR del formato de almacenamiento. | G2 |
| F3-T01 | Gobernador: señales | `social-governor` con la matriz de `08` §2.1 y la regla de respaldo. Fuentes de señal simulables para pruebas. | T00 |
| F3-T02 | Gobernador: estados y permisos | Máquina de estados, histéresis, permisos cancelables, presupuestos de disco, escritura, CPU y red. Pruebas con señales simuladas: toda violación dura detiene el trabajo en 5 segundos o menos. | T01 |
| F3-T03 | Interfaz del nodo | Activación con un clic, perfil recomendado, ajuste manual, panel de estado con las cifras de `08` §7. | T02 |
| F3-T04 | Red | `social-net`: identidad, transportes, NAT, descubrimiento, límites por par. | T00 |
| F3-T05 | Almacén local | `social-storage`: cifrado por bloques, almacén con cuota, limpieza, todo bajo el gobernador. | T02 |
| F3-T06 | Transferencia verificada | Petición-respuesta con verificación por rangos; reanudación; penalización de pares. | T04, T05 |
| F3-T07 | Coordinador | Registro de nodos, asignación a redes distintas, reparación desde la copia ancla. | T04 |
| F3-T08 | Servicio de claves de contenido | Custodia en KMS, entrega a lectores, borrado con efecto en todas las réplicas. | T05 |
| F3-T09 | Auditorías | Desafíos impredecibles, plazos por latencia, contabilidad de GB·hora, registro auditable. | T06, T07 |
| F3-T10 | Lectura desde nodos | El cliente de escritorio obtiene media de los nodos con respaldo en la pasarela. | T06, T08 |
| F3-T11 | Puntos de prueba | Cálculo de emisión de `09` §3 con puntos sin valor; panel de calibración. | T09 |
| F3-T12 | Distribución | Firma de código en Windows y macOS (H), actualizaciones firmadas, instaladores, desinstalación limpia. | T03 |
| F3-T13 | Red de prueba simulada | 50 nodos en contenedores con abandono y cortes simulados; informe de disponibilidad y reparación. | T09 |
| F3-T14 | Revisión externa del nodo (H) | Superficie de red, actualizaciones y gobernador. | T13 |
| **G3** | **Compuerta** (H) | Red de prueba estable; cero incidentes de daño o consumo no deseado en equipos de prueba; dictamen sobre responsabilidad de operadores. | Todo F3 |

---

## 6. F4 — Ledger y comercios

**Objetivo:** puntos con valor real, canjeables en comercios de la ciudad piloto.

| ID | Tarea | Entregable y criterio de aceptación | Dep. |
| :-- | :-- | :-- | :-- |
| F4-T00 | Refinamiento de fase y **dictamen legal** (H) | Modelo económico (D-03) decidido con el dictamen en la mano. | G3 |
| F4-T01 | Servicio de ledger | Esquema de `09` §4, función de asiento, permisos sin `UPDATE` ni `DELETE`, cadena de hashes. Una prueba por cada invariante. | T00 |
| F4-T02 | Pruebas de concurrencia | Pruebas de propiedades y de carga con operaciones simultáneas: ningún saldo negativo, suma global cero, sin asientos duplicados. | T01 |
| F4-T03 | Billetera | Alta con prueba de pertenencia a G3 de ámbito billetera (solo nivel 3); vínculo nodo-billetera; cliente de billetera en escritorio y móvil. | T01 |
| F4-T04 | Emisión | Emisión diaria por presupuesto con topes, mínimo de participación y vencimiento. | T03 |
| F4-T05 | Canje | Flujo de `09` §5 completo, por la ruta que oculta la IP. | T03 |
| F4-T06 | Comercios | Alta con verificación (H), roles, panel, conciliación, exportación. Incluye los **puntos de inscripción**: lectura de chip para quien no tiene NFC y método `presencial` con vales ciegos y cupo. | T01 |
| F4-T07 | TPV | Aplicación web instalable para cobrar. | T05, T06 |
| F4-T08 | Antifraude | Controles de `09` §6, alertas y revisión manual. | T05 |
| F4-T09 | Transparencia | Publicación diaria de raíz y totales; prueba de inclusión para billeteras. | T01 |
| F4-T10 | Operación | Conciliación automática, interruptores de emergencia, manuales, doble aprobación de asientos manuales. | T04 |
| F4-T11 | **Auditoría externa** (H) | Ledger y canje. Hallazgos críticos y altos corregidos. | T08 |
| F4-T12 | Piloto (H) | Comercios reales en la ciudad piloto con límites bajos. | T11 |
| **G4** | **Compuerta** (H) | Auditoría y dictamen aprobados; ledger cuadrado cada día durante 30 días de piloto. | Todo F4 |

---

## 7. F5 — Descentralización progresiva

Se planifica por hitos independientes. Cada uno empieza con un ADR y su propio refinamiento.

| Hito | Contenido | Condición para empezar |
| :-- | :-- | :-- |
| F5-H1 | Especificación pública de pasarela e indexador; guía y pruebas de conformidad para operadores externos. | G2 |
| F5-H2 | Réplica del archivo de eventos en nodos; clientes que leen de varios operadores. | F5-H1 |
| F5-H3 | Registro con varios operadores y firma umbral. | G2, auditoría |
| F5-H4 | Difusión entre nodos con Dandelion++. | F5-H2 |
| F5-H5 | Codificación de borrado y retirada de la copia ancla. | 90 días de disponibilidad medida por encima del objetivo |
| F5-H6 | Custodia umbral de claves de contenido. | F5-H3 |
| F5-H7 | Límite de frecuencia anónimo (RLN) y listas de bloqueo anónimas. | Investigación previa |
| F5-H8 | Cómputo en sandbox con verificación por redundancia. | Demanda real identificada |
| F5-H9 | Evaluación de token o puente. | Dictamen legal y tracción |

---

## 8. Camino crítico y riesgos de calendario

```
F0-T01 → T05 → T06 → T07 → T08 ─┬→ F1 servicios → F1 clientes → G1
                                 │
F0-S1 (humanidad) ───────────────┼──────────────────────────────→ F2 → G2 → F3 → G3 → F4 → G4
F0-S3 (zonas) ───────────────────┘
```

| Riesgo | Efecto | Cuándo se sabe |
| :-- | :-- | :-- |
| El pasaporte con chip no es viable como método fuerte en Colombia | La red abre con niveles 1 y 2, pero F4 (puntos) queda bloqueada | Fin de S1 |
| Pocos adultos colombianos tienen pasaporte con chip | Pocos usuarios de nivel 3: pocos nodos remunerados y pocos avalistas | Fin de S1 |
| Trámites: certificados de firma, cuentas de desarrollador, tiendas | Semanas de espera | Iniciar en F1 |
| Auditorías externas | Semanas de agenda y presupuesto | Contratar al empezar F2 y F4 |
| Dictámenes legales | Pueden cambiar el modelo de puntos | Encargar al empezar F2 |
| Datos de barrio insuficientes en Bogotá | Barrios sintéticos | Fin de S3 |
| El abogado objeta el vínculo entre nivel 3 y puntos | Rediseño de quién puede ganar puntos | F2-T20 |
| Baja tasa de conexión directa entre nodos | Más costo de relevos | Fin de S2 |

**Qué puede avanzar en paralelo con varios agentes:** dentro de F1, los bloques de servicios y de clientes a partir de T03 y T17; los cinco spikes entre sí; y las tareas de documentación y pruebas. Nunca dos agentes sobre la misma tarea ni sobre la misma zona sensible a la vez.
