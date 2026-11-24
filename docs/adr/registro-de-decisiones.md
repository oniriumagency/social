# Registro de Decisiones de Arquitectura (ADR)

Este archivo es el **único** registro de ADR del proyecto. Cada decisión de arquitectura es una sección numerada, en orden.

**Reglas**

1. Un ADR aceptado solo se cambia con otro ADR que lo sustituya o lo amplíe. El anterior se conserva y se le anota el estado.
2. Un ADR nuevo se añade al final con el número siguiente y la plantilla de abajo. Lo propone un agente o una persona; lo acepta el responsable humano.
3. Si un ADR toca una invariante de `CLAUDE.md` / `ANTIGRAVITY.md` o un punto de `04-modelo-de-amenazas.md`, exige aprobación explícita del responsable humano.
4. En el mismo PR se actualizan los documentos, esquemas y vectores afectados.

**Plantilla**

```
## ADR-NNN. Título corto en forma de decisión

- **Fecha y estado.** AAAA-MM-DD. Propuesta | Aceptada | Sustituida por ADR-XXX | Rechazada. Tarea relacionada: F?-T??.
- **Contexto.** Qué problema hay y qué restricciones aplican. Citar documentos, hallazgos o informes de spike.
- **Decisión.** Qué se hace, en frases claras.
- **Descartado.** Cada alternativa con su motivo.
- **Consecuencias.** Qué mejora, qué empeora, qué riesgo se acepta.
- **Invariantes y amenazas.** Qué invariante o punto de `04` toca, si toca alguno.
```

**Estado.** ADR-001 a ADR-015 salen de la auditoría del 2026-11-23: aceptados, pendientes de confirmación en la compuerta G0. ADR-016 a ADR-019 son decisiones del responsable del proyecto.

| ADR | Decisión |
| :-- | :-- |
| 001 | Descentralización progresiva sobre datos autocertificables |
| 002 | La unicidad humana se ancla en documentos firmados por el emisor (ampliada por 016) |
| 003 | No se escriben circuitos ZK ni criptografía propios |
| 004 | Dos capas de identidad con atestaciones por época |
| 005 | Tres identidades inenlazables: cuenta, billetera y nodo |
| 006 | Catálogo administrativo de zonas, H3 de apoyo, resolución en el dispositivo |
| 007 | Formato de evento propio, JCS, Ed25519 y un solo núcleo en Rust |
| 008 | Copia ancla más réplicas comunitarias |
| 009 | Media cifrada con clave por contenido y servicio de claves |
| 010 | Orden de fases: social, humanidad, nodo, ledger |
| 011 | Licencias: AGPL-3.0 para servicios, nodo y apps; Apache-2.0 para núcleo y SDK |
| 012 | Pasarela como monolito modular; registro y ledger aislados |
| 013 | Emisión de puntos por presupuesto, intransferibles y con vencimiento |
| 014 | El móvil y el navegador son clientes ligeros |
| 015 | Órdenes únicas del repositorio en `package.json` |
| 016 | Varios métodos de verificación con niveles de confianza |
| 017 | Registro adaptativo por dispositivo y país del documento |
| 018 | Puntaje de confianza dinámico, vigilancia por riesgo y verificación adicional |
| 019 | El piloto es Colombia |

---

## ADR-001. Descentralización progresiva sobre datos autocertificables

- **Contexto.** El diseño prometía "descentralización real" pero dependía de un ledger, un registro de nullifiers e índices centrales.
- **Decisión.** Todo dato de usuario es un evento firmado y verificable sin confiar en el operador. Se empieza con operadores del proyecto y cada pieza central tiene una ruta de salida documentada (`02` §3).
- **Consecuencias.** Se puede lanzar pronto. Hay puntos únicos de fallo y censura reconocidos hasta F5. Ningún operador puede falsificar contenido.
- **Descartado.** Red P2P pura desde el inicio: no resuelve índices, unicidad ni durabilidad con nodos domésticos.

## ADR-002. La unicidad humana se ancla en documentos con chip firmados por el emisor

- **Estado.** Ampliada por ADR-016: el chip deja de ser la única vía y pasa a ser el método de mayor fuerza.
- **Contexto.** Biometría local y OCR no prueban unicidad (A-01, A-02).
- **Decisión.** La única fuente de unicidad es la prueba de conocimiento cero sobre el chip NFC de un documento oficial. La biometría del dispositivo solo protege llaves.
- **Consecuencias.** Hace falta un teléfono con NFC para inscribirse. La cobertura depende del país. Resistencia Sybil real.
- **Descartado.** Biometría con base central; grafos de confianza; atestación de dispositivo como prueba de humanidad.

## ADR-003. No se escriben circuitos ZK ni criptografía propios

- **Contexto.** Un circuito de identidad incorrecto es un fallo silencioso y catastrófico.
- **Decisión.** Fases 0 a 4: solo protocolos y estándares existentes (proveedor de prueba de documento, Semaphore v4, RFC 9474). El spike S1 comprueba que cada uno tiene auditoría pública vigente; el que no la tenga no se adopta. Cualquier circuito propio futuro: Noir, ADR y auditoría externa. Se elimina Halo2.
- **Consecuencias.** Dependencia de terceros, aislada tras `HumanityProvider`. Menos riesgo y menos costo.

## ADR-004. Dos capas de identidad con atestaciones por época

- **Contexto.** Vincular el nullifier del documento a la cuenta permite desanonimizar (A-03).
- **Decisión.** Capa 1: el documento inscribe un compromiso en un grupo. Capa 2: la cuenta se crea con una prueba de pertenencia y un nullifier por ámbito y época. Las atestaciones vencen cada 180 días y se renuevan solas.
- **Consecuencias.** El registro no puede relacionar documento y cuenta. Las cuentas huérfanas caducan. La sanción máxima es temporal. Más complejidad en el cliente.
- **Descartado.** Vínculo directo documento-cuenta: más simple, permite expulsión permanente, rompe el pilar 3.

## ADR-005. Tres identidades inenlazables: cuenta, billetera y nodo

- **Contexto.** Un nodo con IP pública que cobra puntos para una cuenta social une todo (A-14).
- **Decisión.** Llaves, servicios y bases de datos separados. La billetera se crea con un ámbito distinto. Ninguna tabla ni registro une dos identidades, salvo nodo-billetera en el coordinador.
- **Consecuencias.** No hay analítica cruzada. Más servicios que operar.

## ADR-006. Catálogo administrativo como fuente de zonas; H3 como apoyo; resolución en el dispositivo

- **Contexto.** H3 no equivale a barrios ni ciudades; resolver en servidor filtra coordenadas; el jitter no protege (A-09, A-10).
- **Decisión.** Catálogo versionado con coberturas H3 precomputadas, repartido en paquetes. La resolución es una búsqueda local. Zona fija por defecto, precisión de localidad, umbral de k-anonimato, sin jitter.
- **Consecuencias.** Hay que construir y mantener un pipeline de datos. Dependencia de licencias de datos abiertos.

## ADR-007. Formato de evento propio, canonicalización JCS, Ed25519 y un solo núcleo en Rust

- **Contexto.** No había protocolo de datos (A-30).
- **Decisión.** Eventos JSON con identificador SHA-256 sobre la forma canónica JCS (RFC 8785) y firma Ed25519 estricta. Una sola implementación en Rust, compilada a WASM para web. Vectores dorados como definición ejecutable.
- **Consecuencias.** Sin divergencia entre lenguajes. La web depende de WASM.
- **Descartado.** Nostr y AT Protocol: el requisito de humanidad anula la interoperabilidad que aportarían. CBOR determinista: más robusto, menos cómodo de depurar; se puede reconsiderar en la versión 2.

## ADR-008. Copia ancla más réplicas comunitarias

- **Contexto.** Nodos domésticos con baja disponibilidad no garantizan durabilidad (A-06).
- **Decisión.** Toda la media tiene copia ancla en almacenamiento de objetos. Las réplicas comunitarias suman. El ancla se retira solo con 90 días de disponibilidad medida, en F5.
- **Consecuencias.** Costo de almacenamiento para el proyecto. Sin pérdida de datos por abandono de nodos.

## ADR-009. Media cifrada con clave por contenido y servicio de claves

- **Contexto.** Hace falta poder borrar y retirar contenido en una red replicada, y proteger legalmente al operador del nodo (A-07).
- **Decisión.** Cada archivo se cifra con su clave. Los nodos solo guardan texto cifrado. Borrar la clave retira el contenido de todas las réplicas.
- **Consecuencias.** El servicio de claves es una pieza central que puede negar acceso. Se acepta a cambio de cumplir borrados. Custodia umbral en F5.

## ADR-010. Orden de fases: social, humanidad, nodo, ledger

- **Contexto.** El plan original repartía puntos antes de tener defensa anti-bots (A-04).
- **Decisión.** F1 alfa por invitación sin puntos; F2 humanidad; F3 nodo con puntos de prueba; F4 ledger y comercios. Ningún punto con valor antes de G2, G3 y la auditoría de G4.
- **Consecuencias.** Los puntos llegan más tarde. No hay ventana para granjas.

## ADR-011. Licencias: AGPL-3.0 para servicios, nodo y apps; Apache-2.0 para núcleo y SDK

- **Estado.** Aceptada el 2026-11-24 (D-04).
- **Decisión.** AGPL-3.0-only para servicios, nodo y apps. Apache-2.0 para núcleo, especificación, SDK y herramientas. El reparto por carpeta vive en `LICENSE`; cada paquete lo declara con su identificador SPDX en `Cargo.toml` o `package.json`.
- **Consecuencias.** Hay que vigilar la compatibilidad de licencias de dependencias y de datos geográficos.

## ADR-012. Pasarela como monolito modular; registro y ledger como servicios aislados

- **Contexto.** Microservicios desde el inicio multiplican la operación; pero registro y ledger exigen aislamiento por seguridad y privacidad.
- **Decisión.** `gateway` agrupa ingesta, feeds, media y moderación en módulos. `registry`, `coordinator` y `ledger` son servicios con base de datos y credenciales propias.
- **Consecuencias.** Operación simple en F1. Fronteras de privacidad reales.

## ADR-013. Emisión de puntos por presupuesto, intransferibles y con vencimiento

- **Contexto.** Puntos sin tope son una deuda ilimitada y un riesgo regulatorio (A-05).
- **Decisión.** Presupuesto diario repartido a prorrata por GB·hora auditados, con tope por humano. Sin transferencias entre usuarios, sin compra, con vencimiento.
- **Consecuencias.** La obligación total es predecible. Menos flexibilidad para el usuario.

## ADR-014. El móvil y el navegador son clientes ligeros

- **Contexto.** Los sistemas móviles suspenden procesos en segundo plano; el navegador no es un par completo (A-17).
- **Decisión.** Solo escritorio y servidor operan nodos.
- **Consecuencias.** Menos nodos potenciales. Expectativas honestas.

## ADR-015. Órdenes únicas del repositorio en `package.json`, sin scripts de shell

- **Contexto.** El desarrollo es en Windows y los agentes necesitan órdenes inequívocas.
- **Decisión.** Toda operación se expone como orden `pnpm` en la raíz. Los scripts auxiliares se escriben en Node o en Rust (`xtask`).
- **Consecuencias.** Mismo comportamiento en los tres sistemas y en CI.

## ADR-016. Varios métodos de verificación con niveles de confianza

- **Fecha y estado.** 2026-11-23. Aceptada (decisión D-02). Amplía ADR-002. Tareas: F0-S1 y F2.
- **Contexto.** ADR-002 exigía el chip NFC de un documento oficial como única vía. Muchos dispositivos no tienen NFC y en varios países el documento de identidad no lleva chip. El responsable pidió alternativas y un medidor interno de confianza según los métodos que el usuario haya añadido.
- **Decisión.**
  1. La prueba de humanidad se compone de **métodos de verificación**, cada uno una implementación de `HumanityProvider` con una fuerza: fuerte, media o baja.
  2. La fuerza depende de **quién respalda el dato** y de cuánto cuesta falsificarlo, no de lo cómodo que sea.
  3. Cada identidad tiene un **nivel de confianza** de 1 a 3, decidido por la combinación más fuerte, nunca por la suma.
  4. Solo el **nivel 3** (un método fuerte) significa humano único.
  5. El medidor es **interno**: el registro conoce los métodos de cada compromiso de identidad; la atestación de la cuenta solo lleva el nivel.
  6. Hay un grupo Semaphore por nivel. Subir de nivel conserva la cuenta, porque el nullifier de ámbito no depende del grupo.
  7. **Billetera, puntos, avales y cuentas secundarias exigen nivel 3.**
  8. La tabla de métodos, fuerzas y privilegios es configuración versionada del registro. Cambiarla exige ADR.
- **Descartado.** Chip como única vía: excluye a quien no tiene NFC ni documento con chip. Puntuación por suma de métodos: una granja acumula métodos baratos. Publicar los métodos de cada cuenta: la combinación es una huella. OCR sin atestación: no resiste a un cliente modificado. Número de teléfono: barato y revela un dato personal. Biometría con base central: contradice el pilar de privacidad. Puntos para todos los niveles: atrae a las granjas hacia el método más barato.
- **Consecuencias.** Cualquier persona puede registrarse por algún camino. La promesa "100% humana" pasa a tener grados: nivel 3 es humano único; niveles 1 y 2 son humano probable, y la comunicación pública debe decirlo así. Una persona puede tener una identidad por cada método distinto, porque métodos diferentes no se pueden comparar sin romper la privacidad. Cada método tiene su suite de ataque y entra en la auditoría de G2. Hay dependencia de Apple y Google en los métodos con atestación; ningún método es obligatorio. Si un país no tiene ningún método fuerte viable, la red abre allí pero los puntos quedan bloqueados.
- **Invariantes y amenazas.** No relaja ninguna invariante. I-1 se aplica a todos los métodos. I-5 se refuerza: la lista de métodos no aparece en atestaciones ni en datos públicos. `04` §5 incorpora el costo de falsificación de cada método.

## ADR-017. Registro adaptativo por dispositivo y país del documento

- **Fecha y estado.** 2026-11-23. Aceptada. Tareas: F0-S1, F2-T23 y F2-T14.
- **Contexto.** Con varios métodos, no todos sirven para todos. Ofrecer un método que el usuario no puede completar lo deja atascado: no se puede pedir leer un chip cuando el documento del país no lo tiene.
- **Decisión.**
  1. El cliente **detecta en local** las capacidades del dispositivo: lectura de chips por NFC, atestación de app, autenticador con verificación de usuario, cámara.
  2. El usuario indica el **país emisor** de su documento. Se sugiere por la zona elegida o la región del dispositivo, nunca por IP.
  3. Un **catálogo de métodos por país**, versionado y firmado en `spec/methods/`, indica qué documentos llevan chip verificable y qué métodos son viables.
  4. El formulario ofrece la **intersección** de catálogo y capacidades, ordenada por nivel alcanzable. Nunca ofrece un método imposible.
  5. Ni las capacidades ni el país elegido salen del dispositivo: el cliente descarga el catálogo completo y filtra en local.
- **Descartado.** Mostrar todos los métodos y dejar que el usuario pruebe: frustra en el paso más delicado. Detectar el país por IP: viola I-4, falla con VPN y confunde ubicación con país emisor. Decidir las opciones en el servidor: el servidor recibiría una huella del dispositivo. Catálogo incrustado en el código: cada cambio exigiría publicar apps nuevas.
- **Consecuencias.** El registro no tiene caminos sin salida. El catálogo es un conjunto de datos vivo, con fuente citada por entrada, validado en CI y revisado en cada compuerta. Un catálogo erróneo bloquea o engaña al usuario; por eso el spike S1 produce la primera versión con pruebas reales.
- **Invariantes y amenazas.** Refuerza I-4 e I-6. El catálogo va firmado: uno falsificado podría empujar a los usuarios hacia el método más débil.

## ADR-018. Puntaje de confianza dinámico, vigilancia por riesgo y verificación adicional

- **Fecha y estado.** 2026-11-23. Aceptada. Valores numéricos pendientes de calibrar (D-16). Tarea: F2-T24.
- **Contexto.** Los niveles 1 y 2 admiten métodos que una granja puede superar con esfuerzo. El responsable pidió vigilar más de cerca las cuentas de puntaje bajo, pedirles verificación adicional al azar, y que el puntaje suba cuando la superan y baje cuando la fallan o la ignoran.
- **Decisión.**
  1. Se separan dos medidas. El **nivel** (1 a 3) mide la fuerza de los métodos y vive en el registro. El **puntaje** (0 a 100) mide el comportamiento en el tiempo y vive en la pasarela.
  2. El puntaje se mueve dentro de la **banda de su nivel** (topes de 60, 80 y 100) y **nunca cambia el nivel ni concede sus privilegios**.
  3. Un **motor de riesgo** vigila todas las cuentas, con umbrales más sensibles cuanto menor es el puntaje. Solo usa eventos públicos y firmados.
  4. La **verificación adicional** tiene dos formas: reto de presencia (ligero, al azar) y reto de método (fuerte, ante riesgo alto). Llegan en momentos impredecibles y con más frecuencia a los puntajes bajos.
  5. Superar un reto sube el puntaje. Fallarlo lo baja más que ignorarlo. Ignorarlo penaliza una sola vez y deja la cuenta **en pausa reversible**.
  6. Suspender una atestación exige revisión humana y admite apelación.
  7. El puntaje es privado: solo lo ve su dueño, por tramos.
- **Descartado.** Un solo número que mezcle métodos y comportamiento: una granja subiría de nivel superando retos baratos. Que el puntaje dé acceso a puntos o avales: el valor económico debe exigir el método más caro de falsificar. Vigilar con IP y huella del dispositivo: viola las invariantes y construye el perfilado que el proyecto rechaza. Puntaje público: estigmatiza y ayuda a identificar cuentas. Penalizar igual ignorar que fallar: castiga a quien se ausenta. CAPTCHA de terceros: carga servicios externos y es poco accesible. Suspensión automática: los falsos positivos dejarían sin voz a personas legítimas.
- **Consecuencias.** Mantener una granja exige atender cada dispositivo en momentos impredecibles. Las personas reales reciben un toque de vez en cuando, más a menudo en los niveles bajos; es un incentivo para subir de nivel y un costo de experiencia que hay que medir. El motor tendrá falsos positivos: su tasa se publica y se calibra en la beta. Aparece estado privado por cuenta en la pasarela. Las decisiones automatizadas sobre personas entran en el mapa legal antes de G2.
- **Invariantes y amenazas.** I-4 se amplía: el motor de riesgo y el puntaje no se alimentan con IP, huella del dispositivo ni datos de otro servicio. I-5 se mantiene: el reto usa una prueba del mismo ámbito. I-14 se amplía: el puntaje nunca decide el nivel. `04` §6 incorpora la mitigación frente a granjas en niveles bajos.

## ADR-019. El piloto es Colombia

- **Fecha y estado.** 2026-11-24. Aceptada (decisión D-01). Ciudad: Bogotá, confirmada el mismo día. Tareas: F0-S1, F0-S3, F2-T20 y F4-T00.
- **Contexto.** El país piloto condiciona qué métodos de verificación existen, la calidad de los datos de zonas y el marco legal. Datos consultados el 2026-11-24 en fuentes públicas (`05` §5.1, `10` §6): el pasaporte colombiano lleva chip electrónico; no se encontró ninguna fuente que atribuya a la cédula, ni a la amarilla ni a la digital, un chip sin contacto verificable por terceros; y la SIC ordenó el cierre definitivo de Worldcoin en Colombia por tratar datos biométricos a cambio de una compensación económica.
- **Decisión.**
  1. El primer catálogo de métodos (`spec/methods/`), el primer catálogo de zonas y los primeros dictámenes legales son los de Colombia.
  2. En Colombia el método `chip` se apoya en el **pasaporte electrónico**. La cédula entra por `doc-app`, que da nivel 2 como máximo, hasta que el spike S1 demuestre otra cosa.
  3. **Verificarse nunca se premia.** Ningún punto, saldo o ventaja económica se entrega por completar o añadir un método de verificación. Los puntos pagan almacenamiento auditado.
  4. El servicio de autenticación facial de la Registraduría queda descartado: exige enviar el rostro a una base central.
  5. Antes de admitir al primer usuario real existen la política de tratamiento de datos y el canal de consultas y reclamos que exige la Ley 1581 de 2012.
- **Descartado.** Esperar a elegir el país con los datos de S1: retrasa los spikes S1 y S3, que necesitan un país concreto. Dar puntos al nivel 2 para compensar la baja cobertura del pasaporte: sigue siendo la decisión D-15, que se revisa con los datos de S1 y no se abre para salir del paso.
- **Consecuencias.** La mayoría de los primeros usuarios será de nivel 1 o 2, porque el documento que todo adulto tiene es la cédula. El nivel 3, y con él la billetera, los puntos, los avales y el nodo remunerado, depende de cuánta gente tiene pasaporte con chip: S1 debe medirlo. El método `aval` necesita humanos de nivel 3, así que arranca despacio. El precedente de la SIC refuerza el pilar de privacidad y obliga a que el abogado revise el vínculo entre nivel 3 y puntos antes de G2 y G4.
- **Invariantes y amenazas.** No relaja ninguna invariante. I-14 incorpora la regla de no premiar la verificación. `04` §7 incorpora el supuesto regulatorio.
