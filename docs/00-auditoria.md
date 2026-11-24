# 00 - Informe de Auditoría

**Fecha:** 2026-11-23 · **Alcance:** los tres documentos fundacionales (`01`, `02`, `03`). No existía código, así que la auditoría es de diseño: producto, arquitectura, seguridad, privacidad, economía, legalidad y viabilidad de ejecución con agentes.

**Resultado:** 42 hallazgos (7 críticos, 18 altos, 17 medios). Todos están corregidos en la documentación (columna "Dónde quedó"). Los documentos `01`, `02` y `03` fueron reescritos; los documentos `04` a `16` son nuevos. Tras la auditoría, el responsable del proyecto tomó cuatro decisiones que completan el diseño: varios métodos de verificación con niveles de confianza, registro adaptativo, puntaje de confianza y piloto en Colombia (sección 7).

**Cómo leer la severidad**

- **Crítico:** el pilar no se cumple, o el proyecto no puede lanzarse así.
- **Alto:** fallo de seguridad, privacidad o viabilidad que obliga a rediseñar una pieza.
- **Medio:** imprecisión, sobrepromesa o hueco que causaría retrabajo.

---

## 1. Los cinco cambios que más alteran el diseño original

1. **La prueba de humanidad se reconstruye.** Ni la biometría local ni el escaneo OCR de un documento prueban por sí solos que alguien sea un humano único (A-01, A-02). El ancla más fuerte compatible con el pilar de privacidad es un **documento firmado digitalmente por su emisor**, como el chip NFC de un pasaporte, probado con conocimiento cero. Para no dejar fuera a quien no tiene NFC, hay varios métodos con **niveles de confianza**: solo el nivel 3 significa humano único (sección 7).
2. **El orden de la hoja de ruta se invierte.** El plan original repartía puntos antes de tener defensa anti-bots (A-04). Ahora: social → humanidad → nodo → ledger y comercios → descentralización.
3. **"Descentralización real" pasa a ser "descentralización progresiva y verificable".** El diseño original ya dependía de piezas centrales sin reconocerlo (A-22). Ahora los datos son autocertificables desde el día uno y cada pieza central tiene una ruta de salida documentada.
4. **Los puntos tienen presupuesto.** Sin tope de emisión ni fuente de fondos, los puntos eran una deuda ilimitada y un riesgo regulatorio (A-05). Ahora la emisión es por presupuesto, con tope por humano, sin transferencias entre usuarios y con vencimiento.
5. **La ubicación se resuelve en el dispositivo y con zona fija por defecto.** El diseño original filtraba ubicación por varias vías (A-09 a A-12, A-24).

---

## 2. Hallazgos críticos

| ID | Hallazgo | Por qué es un problema | Corrección aplicada | Dónde quedó |
| :-- | :-- | :-- | :-- | :-- |
| A-01 | La "Ruta 1" usa biometría local (Passkeys, Secure Enclave) como prueba de humanidad única. | La biometría del dispositivo solo prueba "quien desbloqueó este aparato". No exporta plantilla, así que no hay forma de derivar un identificador por persona. El nullifier sería por dispositivo: diez teléfonos, diez cuentas; una granja de teléfonos con la misma huella, mil cuentas. | La biometría local se reasigna a **proteger y desbloquear la llave** de la cuenta. La unicidad humana viene solo de un documento firmado por su emisor (nivel 3). Con atestación del dispositivo, la biometría aporta el método `dispositivo`, de fuerza baja. | `05` §2–4, ADR-002, ADR-016 |
| A-02 | La "Ruta 2" lee el documento por OCR/MRZ en local y deriva el nullifier en un circuito ZK. | Un circuito ZK prueba un cálculo sobre entradas privadas, pero no que esas entradas sean auténticas. Con cliente de código abierto, un atacante alimenta el circuito con MRZ inventados y obtiene nullifiers infinitos. La "prueba de vida" local tampoco es verificable porque corre en un cliente que controla el atacante. Resistencia Sybil real: cero. | El circuito debe verificar la **firma digital del Estado emisor** sobre los datos del chip (ICAO 9303). La ruta OCR solo se admite dentro de una app atestada y con fuerza media (sección 7). Se integra un protocolo existente en lugar de escribir circuitos propios; sus auditorías se comprueban en el spike S1. | `05` §2–3, ADR-002, ADR-003, ADR-016 |
| A-03 | El nullifier se vincula directamente a la cuenta. | Los datos de un documento son de baja entropía y los conoce el emisor. Quien tenga esos datos y acceso al registro puede recalcular el nullifier y encontrar la cuenta. Rompe "sin vínculo reversible con la identidad civil". | Dos capas: el nullifier del documento solo se usa para inscribir un **compromiso de identidad** en un grupo; la cuenta se crea después con una prueba de pertenencia al grupo (Semaphore) que no revela cuál miembro es. | `05` §7–8, ADR-004 |
| A-04 | El MVP 1 acredita puntos por almacenamiento y el anti-bots llega en el MVP 2. | Puntos con valor sin resistencia Sybil equivale a regalar dinero a granjas de nodos falsos. Además el MVP 1 lanzaba cuentas sin prueba de humanidad, contra el pilar 2. | Nuevo orden de fases. La alfa es cerrada por invitación y sin puntos. Ningún punto canjeable antes de humanidad verificada y auditoría del ledger. | `12`, ADR-010 |
| A-05 | Puntos canjeables en comercios sin modelo económico. | Cada punto emitido es una deuda. No hay presupuesto de emisión, ni quién paga al comercio, ni vencimiento. Además un saldo canjeable con terceros puede caer bajo regulación de dinero electrónico o programas de fidelización, y un token posterior bajo regulación de criptoactivos. | Emisión por presupuesto diario, tope por humano, puntos intransferibles entre usuarios, vencimiento, y modelo inicial de descuento financiado por el comercio. Dictamen legal como compuerta obligatoria. | `09`, `10` §6, `15` D-03 |
| A-06 | El contenido se replica entre nodos de usuarios según su cuota. | Un portátil que solo trabaja con Wi-Fi y batería alta está disponible quizá 30% del tiempo. Con 3 réplicas así, la disponibilidad es 1 − 0,7³ ≈ 66%. Con 10 réplicas, 97%. La replicación ingenua pierde datos. | Copia ancla siempre disponible en almacenamiento de objetos hasta que las métricas de la red demuestren que no hace falta. Los nodos comunitarios suman réplicas y ganan puntos, pero la durabilidad no depende de ellos antes de la Fase 5. | `08` §4, ADR-008 |
| A-07 | No hay moderación, retirada de contenido ni borrado. | Una red replicada sin mecanismo de retirada no puede cumplir órdenes legales ni el derecho de supresión, y expone a quien opera un nodo a alojar material ilegal. | Los nodos guardan solo fragmentos cifrados. La clave de contenido vive en un servicio de claves; borrar la clave inutiliza todas las réplicas. Eventos de borrado, reportes, lista de bloqueo por hash y proceso de apelación. | `08` §5, `10` §3–4 |

---

## 3. Hallazgos altos

| ID | Hallazgo | Corrección aplicada | Dónde quedó |
| :-- | :-- | :-- | :-- |
| A-08 | Se afirma que los "nodos relevo" ocultan la IP de origen. Un relevo de libp2p ve ambos extremos; sirve para atravesar NAT, no para anonimato. | Se define qué adversario se cubre en cada fase. Altas y operaciones sensibles van por HTTP Oblivious (RFC 9458) o Tor. Para gossip, Dandelion++ en Fase 5. No se promete anonimato frente a un adversario global. | `04` §3, `08` §3 |
| A-09 | Se propone *jitter* aleatorio para evitar triangulación. Si se publica un identificador de zona, el jitter no aporta nada; y si cambia la zona cerca de un borde, promediar varias publicaciones revela la posición. | Se elimina el jitter. La zona se calcula de forma determinista en el dispositivo y se usa una **zona fija** elegida por el usuario. | `07` §3 |
| A-10 | H3 se presenta como jerarquía barrio → continente. Los hexágonos no coinciden con divisiones administrativas ni anidan de forma exacta. Además, convertir coordenadas a zona en un servidor filtraría las coordenadas. | Catálogo de zonas administrativas versionado como fuente primaria; H3 se usa para precomputar coberturas y como respaldo donde no hay polígonos de barrio. La resolución ocurre **solo en el dispositivo** con paquetes descargables. | `07` §2, §4, ADR-006 |
| A-11 | Zonas con pocos usuarios y ubicación por publicación. Un barrio con cinco usuarios activos, más horarios y contenido, identifica a la persona. La ubicación en vivo por publicación deja rastro de movimientos. | Umbral de k-anonimato: si la zona no alcanza k cuentas activas, se sube de nivel automáticamente. Precisión por defecto a nivel localidad o ciudad; barrio es opcional. Zona fija por defecto, ubicación en vivo solo si el usuario la activa. | `07` §3 |
| A-12 | La ubicación la declara el cliente, así que cualquiera puede publicar en cualquier barrio. | Se asume y se documenta: la zona es una etiqueta, no una prueba de presencia. Cada publicación declara su modo (`home`, `live`, `remote`), los clientes lo muestran, y el cambio de zona fija tiene límite de frecuencia. | `07` §5 |
| A-13 | "GB servidos y verificados" y "tiempo de disponibilidad" sin mecanismo. Lo autodeclarado se falsifica; premiar el tráfico servido invita a descargarse a uno mismo. | Solo cuenta el almacenamiento **asignado por el coordinador y auditado** con desafíos aleatorios. No se premia el tráfico. La disponibilidad se mide por respuestas a auditorías. | `08` §6 |
| A-14 | El nodo tiene IP conocida y gana puntos para una cuenta. Eso une IP, cuenta social y billetera. | Tres identidades separadas e inenlazables: cuenta social, billetera y nodo. La billetera se crea con una prueba de pertenencia de otro ámbito. Servicios y bases de datos separados. | `05` §10, `04` §4, ADR-005 |
| A-15 | Las cuentas secundarias se pagan. Un pago con tarjeta une la identidad civil con la cuenta. | Vales firmados a ciegas (RFC 9474): se paga, se recibe un vale que el servidor no puede relacionar con el pago, y se canjea de forma anónima. Tope de cuentas secundarias por humano. | `05` §9 |
| A-16 | No hay recuperación de cuenta. Si la cuenta base es única de por vida, perder el dispositivo es perderla para siempre. | Secreto de identidad derivado de una passkey sincronizada más frase de recuperación. Si se pierde todo, re-inscripción con el documento, limitada a una por época. | `05` §11 |
| A-17 | Se contempla el móvil como nodo y la web como par de la red. iOS y Android suspenden procesos en segundo plano; un navegador no puede ser un par libp2p completo. | Roles claros: el nodo es de escritorio o servidor. Móvil y web son clientes ligeros que hablan con pasarelas. | `02` §2, `08` §1 |
| A-18 | Propagación por Gossipsub con un tema por zona. Son millones de temas de barrio; suscribirse a uno revela el barrio a los pares; el tema global es una manguera que ningún dispositivo aguanta. | Los feeds se sirven desde indexadores con consultas por zona. El gossip queda para replicación entre nodos a nivel grueso, en Fase 5. | `06` §6, `08` §3 |
| A-19 | Firma en el navegador sin diseño de llaves. Riesgo de XSS, de JavaScript malicioso servido por el propio servidor, y sin revocación de dispositivos. | Jerarquía de llaves: llave raíz de cuenta que delega en llaves de dispositivo con caducidad y revocación. En web, llaves no extraíbles, CSP estricta y cero scripts de terceros. | `06` §2, `10` §2 |
| A-20 | Circuitos ZK propios en "Noir / Halo2". Circuitos de identidad escritos sin auditoría son el punto de fallo más caro del proyecto; dos stacks duplican el riesgo. | No se escriben circuitos propios en las fases 0 a 4. Se integran protocolos existentes (proveedor de prueba de documento y Semaphore v4), cuyas auditorías se comprueban en el spike S1. Si algún día hace falta un circuito propio: Noir y auditoría externa obligatoria. Se elimina Halo2. | `03`, ADR-003 |
| A-21 | Cómputo distribuido y GPU con recompensa. No hay quién demande ese cómputo ni cómo verificar que se hizo bien. | Fuera de alcance hasta la Fase 5. Se elimina wgpu. Wasmtime queda como candidato futuro. | `02` §7, `12` F5 |
| A-22 | "Descentralización real" con ledger en PostgreSQL, registro único de nullifiers e índices de feed. Son piezas centrales no reconocidas. | Tabla explícita de qué es central en cada fase y cómo se sale de ello. Datos firmados y verificables desde el día uno para que cualquier operador sea reemplazable. | `02` §3, ADR-001 |
| A-23 | El stack asume que el registro puede hacerse en web o escritorio. Leer el chip NFC exige app nativa en un teléfono; Web NFC no permite ese tipo de lectura. | El método de chip se ejecuta una sola vez desde cualquier teléfono con NFC, propio o ajeno, enlazado por QR desde web o escritorio. | `05` §3 |
| A-24 | Las fotos llevan metadatos EXIF con GPS exacto. | Eliminación de metadatos en el cliente antes de subir y verificación en el servidor. Es una invariante. | `06` §7, `CLAUDE.md` I-7 |
| A-25 | No se define edad mínima. Una red abierta a menores multiplica las obligaciones legales. | Por defecto, mayores de 18, probado en conocimiento cero dentro de la misma prueba del documento. Queda como decisión ejecutiva. | `05` §7, `15` D-05 |

---

## 4. Hallazgos medios

| ID | Hallazgo | Corrección aplicada | Dónde quedó |
| :-- | :-- | :-- | :-- |
| A-26 | *Circuit breakers* "en milisegundos" y crates `sysinfo`, `battery`, `netdev` como solución completa. El sistema operativo entrega eventos de batería y temperatura en segundos. La red medida y la temperatura no tienen crate multiplataforma; el crate `battery` debe verificarse por falta de mantenimiento. | Objetivo de reacción de 5 segundos o menos, con histéresis. Matriz de señales por sistema operativo con respaldo conservador. Spike S4 para validar. | `08` §2 |
| A-27 | "Cero desgaste". Es una promesa imposible: escribir en SSD y ciclar batería desgastan. | Se reformula como "desgaste mínimo y acotado", con presupuesto de escritura diario y topes. | `01` §3, `08` §2 |
| A-28 | Bitswap en Rust. No forma parte del repositorio principal de rust-libp2p (verificar en spike S2). | Transferencia por petición-respuesta con verificación BLAKE3 por rangos. | `08` §3, `03` |
| A-29 | Redis en el MVP. No hace falta todavía. | PostgreSQL solo. Redis o Valkey entran cuando una medición lo justifique. | `03` |
| A-30 | No hay protocolo de datos: formato de evento, canonicalización, identificadores. Dos implementaciones (Rust y TypeScript) divergirían. | Especificación de eventos firmados, un solo núcleo en Rust compilado a WASM, y vectores dorados compartidos. | `06`, ADR-007 |
| A-31 | Sanciones y anonimato sin resolver. Con cuentas inenlazables no se puede expulsar a un humano para siempre. | Se documenta la consecuencia: la sanción máxima dura hasta el fin de la época. Queda como decisión ejecutiva. | `05` §12, `15` D-06 |
| A-32 | Next.js dentro de Tauri. Tauri necesita una app estática; el renderizado en servidor no funciona ahí. | Next.js solo para web. Escritorio con Tauri y Vite. Pantallas y lógica compartidas en paquetes. | `03`, `11` §2 |
| A-33 | Distribución no contemplada: firma de código en Windows y macOS, notarización, políticas de tiendas, y compilación para iOS que exige un Mac. | Listado como requisito con costos y tiempos en el plan. | `11` §6, `12` F3 |
| A-34 | Licencias sin decidir, y datos geográficos con licencias de tipo ODbL. | Propuesta por defecto: AGPL-3.0 para servicios y nodo, Apache-2.0 para protocolo y SDK. Revisión de licencias de datos en el spike S3. | `15` D-04, ADR-011 |
| A-35 | *Build in public* con transmisiones en vivo sin reglas. Riesgo de mostrar secretos en pantalla. | Reglas para agentes y humanos: secretos fuera del repositorio y de la terminal, política de divulgación responsable. | `10` §7, `CLAUDE.md` |
| A-36 | Arranque en frío. Un feed de barrio vacío no retiene a nadie. | Lanzamiento ciudad por ciudad, alineado con la red de comercios. | `01` §5, `15` D-01 |
| A-37 | Sin estrategia de pruebas, integración continua, observabilidad ni despliegue. | Documentos `11` y `14`. | `11`, `14` |
| A-38 | Cobertura de documentos con chip. En muchos países poca gente tiene pasaporte electrónico y la cédula no siempre lleva chip. | Riesgo de mercado explícito, cubierto con varios métodos y registro adaptativo. En Colombia, país del piloto, el pasaporte lleva chip y la cédula no. Spike S1 produce el inventario definitivo. | `05` §5 y §13, `15` D-01, ADR-016, ADR-017, ADR-019 |
| A-39 | Canje por QR sin diseño. Riesgos: captura de pantalla, repetición, doble gasto, colusión entre comercio y nodo. | QR dinámico que presenta el comercio, intención firmada por la billetera, idempotencia, caducidad corta y reglas antifraude. | `09` §5–6 |
| A-40 | Alquiler o compra de documentos ajenos. | Límite inherente a cualquier prueba de humanidad. Se documenta como riesgo residual con el modelo de costo por cuenta falsa. | `04` §5, `05` §13 |
| A-41 | Un humano verificado también puede hacer spam. | Límites de frecuencia por cuenta desde la Fase 1. RLN como candidato en Fase 5. | `06` §8 |
| A-42 | Orden, edición y borrado de publicaciones sin definir. | Reglas de reloj en la ingesta, sin edición en v1, borrado por evento de lápida. | `06` §4–5 |

---

## 5. Lo que se conserva del diseño original

- Los tres pilares, con el pilar 1 reformulado como descentralización progresiva.
- Rust y Tauri para el nodo, TypeScript y Next.js para la web, PostgreSQL para el ledger, libp2p para la red entre nodos.
- Los perfiles por defecto del nodo (Wi-Fi, batería mayor al 60%, topes conservadores) y la configuración de un clic.
- El ledger de doble entrada, el canje por QR y la red de comercios aliados.
- La estrategia de código abierto y construcción en público.
- Los seis niveles de zona: barrio, localidad, ciudad, país, continente y global.

---

## 6. Riesgos residuales que ningún diseño elimina

Son límites honestos del producto. Deben conocerse antes de prometer nada en público.

1. **Un humano, varios documentos.** Quien tenga dos documentos con chip, por ejemplo por doble nacionalidad, puede inscribirse más de una vez.
2. **Documentos prestados o comprados.** El costo por cuenta falsa sube mucho, pero no es infinito.
3. **Correlación temporal.** Mientras haya pocos usuarios, quien observe la inscripción y el alta de cuenta puede relacionarlas por la hora. Se mitiga con lotes y rutas que ocultan la IP.
4. **Adversario de red global.** Un observador con visibilidad de toda la red puede correlacionar tráfico.
5. **Sanción temporal.** El anonimato fuerte impide la expulsión permanente de una persona.
6. **Dependencia de terceros en la prueba de documento.** El proveedor elegido en el spike S1 es un punto de confianza hasta que exista alternativa.

---

## 7. Decisiones del responsable que completan el diseño

**Varios métodos con niveles de confianza (ADR-016).** Los hallazgos A-01 y A-02 siguen siendo ciertos: ni la biometría local ni el OCR prueban unicidad **por sí solos**. Pero exigir el chip NFC como única vía dejaba fuera a quien no tiene un teléfono con NFC o un documento con chip (A-38). El diseño final combina ambas cosas:

| Corrección inicial de la auditoría | Cómo quedó |
| :-- | :-- |
| Vía única por chip NFC | Siete métodos de verificación, cada uno con una fuerza según quién respalda el dato (`05` §3). |
| Ruta OCR eliminada | Rescatada como método `doc-app`: solo dentro de una app atestada por el sistema operativo, con fuerza media. |
| Biometría solo para la llave | Además aporta el método `dispositivo`, de fuerza baja. |
| Chip exige teléfono propio con NFC | El chip se puede leer con un teléfono prestado o en un punto de inscripción de un comercio aliado. |
| "Un humano, una cuenta" para todos | Tres niveles. Solo el nivel 3 significa humano único. Los niveles 1 y 2 significan humano probable. |
| Puntos para toda cuenta verificada | Puntos solo para el nivel 3, porque el valor económico debe exigir el método más caro de falsificar. |

Reglas que impiden engañar al medidor: cuenta la fuerza y no la cantidad; cada método se usa una sola vez; el medidor es interno y la cuenta solo muestra el nivel.

**Registro adaptativo (ADR-017, `05` §5).** El formulario detecta en local lo que puede hacer el dispositivo y, con un catálogo de métodos por país, ofrece solo lo que se puede completar. No se pide un chip donde el documento no lo tiene.

**Puntaje de confianza y vigilancia (ADR-018, `05` §6).** Cada cuenta tiene un puntaje interno que sube al superar verificaciones y baja al fallarlas o ignorarlas. Las cuentas de puntaje bajo se vigilan más y reciben más verificaciones al azar. El puntaje nunca cambia el nivel.

**Piloto en Colombia (ADR-019).** Tres consecuencias que el resto de documentos ya recoge:

1. El pasaporte lleva chip y es la vía al nivel 3. La cédula entra por cámara dentro de la app y da nivel 2 como máximo, así que la mayoría empezará en los niveles 1 y 2 (`05` §5.1).
2. La autoridad colombiana de datos cerró Worldcoin por recoger biometría a cambio de una compensación. El diseño ya mantenía la biometría en el dispositivo; se añade la regla de **no premiar nunca la verificación** y se exige política de datos y canal de reclamos antes del primer usuario real (`10` §6.1).
3. Las zonas se construyen con municipios, localidades o comunas y barrios (`07` §2.1).

**Riesgos que se aceptan con estas decisiones**

- Una persona puede tener una identidad por cada método distinto, porque métodos diferentes no se pueden comparar entre sí sin romper la privacidad. Queda acotado por el número de métodos y porque solo el nivel 3 accede a puntos.
- En Colombia, pocos usuarios de nivel 3 significan pocos nodos remunerados y pocos avalistas hasta que exista otro método fuerte. El spike S1 mide cuántos.
