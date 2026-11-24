# 02 - Arquitectura y Funcionamiento

> Este documento es el mapa general. El detalle de cada pieza está en los documentos `04` a `10`.

## 1. Principios de arquitectura

1. **Datos autocertificables.** Toda acción es un evento firmado por una llave del autor. Cualquiera puede verificarlo sin confiar en quien se lo entrega (`06`).
2. **El servidor nunca confía en el cliente, y el cliente nunca confía en el servidor.** Cada lado valida todo lo que recibe.
3. **Lo sensible no sale del dispositivo.** Coordenadas, datos del documento, biometría y llaves privadas se procesan en local.
4. **Identidades separadas.** Cuenta social, billetera y nodo son tres identidades que ningún sistema puede unir.
5. **Un solo núcleo.** La lógica de protocolo se escribe una vez en Rust y se usa en servicios, nodo, escritorio y, compilada a WASM, en web.
6. **Central primero, reemplazable siempre.** Se empieza con operadores del proyecto donde la descentralización todavía no es segura, y se documenta la salida.
7. **Integrar antes que inventar.** Nada de criptografía ni circuitos propios mientras exista una pieza pública y revisada por terceros.

---

## 2. Componentes

```
                ┌───────────────── Dispositivos del usuario ─────────────────┐
                │  Web (Next.js)      Escritorio (Tauri)      Móvil (ligero)  │
                │      └── núcleo Rust/WASM: firma, validación, zonas ──┘     │
                │                    Nodo (solo escritorio/servidor)          │
                └───────┬──────────────────┬───────────────────┬─────────────┘
                        │ eventos firmados │ pruebas ZK        │ fragmentos cifrados
                        ▼                  ▼                   ▼
                 ┌────────────┐     ┌────────────┐      ┌──────────────┐
                 │  Pasarela  │     │  Registro  │      │ Coordinador  │
                 │ (gateway)  │     │ de humanidad│     │ de almacenam.│
                 │ ingesta,   │     │ grupo,     │      │ asignación,  │
                 │ feeds,     │     │ nullifiers,│      │ auditorías   │
                 │ media,     │     │ atestación │      └──────┬───────┘
                 │ moderación │     └────────────┘             │
                 └─────┬──────┘                         ┌──────▼───────┐
                       │                                │   Ledger     │
                 PostgreSQL + objetos (copia ancla)     │ (BD aparte)  │
                                                        └──────────────┘
```

| Componente | Qué hace | Fase |
| :-- | :-- | :-- |
| **Núcleo** (`social-core`, `social-crypto`, `social-zones`) | Tipos de evento, canonicalización, firma, validación, resolución de zonas. Compila a nativo y a WASM. | F0 |
| **Pasarela** (`services/gateway`) | Monolito modular: ingesta de eventos, feeds por zona, hilos, media, moderación, flujo en tiempo real. | F1 |
| **Web** (`apps/web`) | Cliente ligero en navegador. | F1 |
| **Escritorio** (`apps/desktop`) | Cliente y, desde F3, nodo. | F1, F3 |
| **Registro de humanidad** (`services/registry`) | Verifica los métodos de verificación, calcula el nivel de confianza, mantiene los grupos de humanos, controla nullifiers y emite atestaciones de cuenta. Servicio y base de datos aislados. | F2 |
| **Coordinador** (`services/coordinator`) | Registra nodos, asigna fragmentos, lanza auditorías, calcula créditos de almacenamiento. | F3 |
| **Servicio de claves de contenido** | Guarda las claves de cifrado de media. Borrar una clave inutiliza todas las réplicas. | F3 |
| **Ledger** (`services/ledger`) | Doble entrada, emisión, canje, comercios. Servicio y base de datos aislados. | F4 |
| **Móvil** | Cliente ligero. La inscripción con NFC usa la app del proveedor de prueba de documento. | F2 |

**Roles de dispositivo**

| Dispositivo | Cliente | Nodo | Inscripción NFC |
| :-- | :-- | :-- | :-- |
| Teléfono | Sí | No | Sí |
| Navegador | Sí | No | No (deriva al teléfono por QR) |
| Portátil o PC | Sí | Sí | No (deriva al teléfono por QR) |
| Servidor | No | Sí | No |

---

## 3. Qué es central en cada fase y cómo se sale

| Pieza | F1–F2 | F3–F4 | F5 (salida) |
| :-- | :-- | :-- | :-- |
| Eventos (publicaciones) | Pasarela del proyecto. Los eventos ya son firmados y exportables. | Igual, más archivo replicado en nodos. | Varias pasarelas e indexadores de terceros con el mismo protocolo. |
| Feeds e índices | Indexador del proyecto. | Igual. | Indexadores independientes; el cliente elige. |
| Media | Almacenamiento de objetos (copia ancla). | Copia ancla más réplicas en nodos comunitarios. | Sin ancla cuando la disponibilidad medida lo permita; codificación de borrado. |
| Registro de humanidad | Operador único, con registro de transparencia público. | Igual. | Varios operadores con firma umbral. |
| Claves de contenido | No aplica. | Servicio del proyecto. | Custodia umbral entre operadores. |
| Ledger | No existe. | PostgreSQL del proyecto con cadena de hashes y raíz diaria publicada. | Evaluación de puente o cadena, con dictamen legal. |

Todo lo "central" de esta tabla es un único punto de censura o caída mientras dure. Es el costo de lanzar algo que funcione; la tabla existe para que nunca se oculte.

---

## 4. Identidad y prueba de humanidad (resumen)

Detalle en `05`.

1. **Métodos de verificación.** Hay varios caminos para registrarse, cada uno con una fuerza: fuerte (chip NFC de un documento oficial, credencial digital firmada), media (documento por cámara en una app atestada, cuenta en una institución, aval de dos humanos verificados, verificación presencial) y baja (dispositivo real con passkey).
2. **Inscripción.** El usuario ejecuta un método en su dispositivo. El registro recibe una prueba y un nullifier de ese método, comprueba que no se ha usado, y asocia el método a un compromiso de identidad que eligió el usuario.
3. **Nivel de confianza.** El registro calcula el nivel de la identidad por la combinación más fuerte, no por la suma: nivel 1 básico, nivel 2 humano probable, nivel 3 humano único. La identidad entra en el **grupo** de su nivel.
4. **Alta de cuenta.** El usuario demuestra, sin revelar quién es, que pertenece al grupo de su nivel y presenta un nullifier de ámbito "cuenta base de esta época". El registro firma una **atestación** para la llave de la cuenta que solo lleva el nivel, nunca los métodos.
5. **Uso.** Pasarelas, indexadores y nodos solo aceptan eventos de cuentas con atestación vigente y aplican los privilegios del nivel.
6. **Subir de nivel.** Añadir un método a la misma identidad sube el nivel sin perder la cuenta.
7. **Renovación.** La atestación vence cada época (180 días) y el cliente la renueva solo.
8. **Cuentas secundarias y billetera.** Solo nivel 3. Mismo mecanismo con otro ámbito; las secundarias exigen además un vale de pago firmado a ciegas.
9. **Biometría del dispositivo.** Desbloquea la llave y aporta el nivel básico. No prueba unicidad.
10. **Registro adaptativo.** El cliente detecta en local las capacidades del dispositivo y cruza el país emisor del documento con un catálogo de métodos por país. Solo ofrece métodos que se pueden completar.
11. **Puntaje de confianza.** Cada cuenta tiene en la pasarela un puntaje interno que sube al superar verificaciones adicionales y baja al fallarlas o ignorarlas. Un motor de riesgo, que solo usa eventos públicos, vigila más de cerca los puntajes bajos. El puntaje nunca cambia el nivel.

---

## 5. Ubicación, filtros jerárquicos y privacidad geográfica (resumen)

Detalle en `07`.

1. **Catálogo de zonas.** Un catálogo versionado define las zonas de cada nivel y su jerarquía: barrio → localidad → ciudad → país → continente → global. Donde no hay polígonos de barrio fiables, el barrio es una celda H3.
2. **Resolución en el dispositivo.** El cliente descarga el paquete de catálogo de su región y convierte la ubicación en zona en local. Nunca se envían coordenadas.
3. **Zona fija y precisión mínima.** Sin jitter. Zona fija por defecto, precisión de localidad por defecto, y subida automática de nivel si no se alcanza el umbral de k-anonimato.
4. **Indexación.** Cada publicación lleva su zona hoja. El indexador la expande a todos sus ancestros para servir cada escala con una consulta por índice.
5. **Privacidad de red.** Las consultas van por HTTPS a la pasarela. Las operaciones sensibles (inscripción, alta de cuenta, canje) van por una ruta que oculta la IP.

---

## 6. Nodo: detección de hardware y gobernanza de recursos (resumen)

Detalle en `08`.

1. **Autodetección.** Tipo de equipo, batería y estado de carga, tipo de red (medida o no), disco libre, CPU y RAM.
2. **Perfiles por defecto.**
   - **Portátil:** solo con Wi-Fi o cable no medidos, enchufado o con batería mayor al 60%, 2 a 5 GB, CPU en prioridad mínima.
   - **Escritorio o servidor:** continuo, 10 a 20 GB, 15 a 25% de CPU en prioridad baja.
3. **Gobernador de recursos.** Es la única puerta para usar disco, CPU y red. Pausa o reduce la carga en 5 segundos o menos ante caída de batería, cambio a red medida, calentamiento o uso intenso del equipo, y reanuda con histéresis para no oscilar.
4. **Ajuste libre.** El usuario cambia cualquier límite cuando quiera. Los límites del usuario nunca se superan.

---

## 7. Almacenamiento distribuido y ledger (resumen)

Detalle en `08` y `09`.

- **Almacenamiento.** La media se trocea, se cifra con una clave por contenido y se replica. Siempre existe una copia ancla. Los nodos guardan solo datos cifrados que no pueden leer.
- **Auditorías.** El coordinador reta a cada nodo a devolver un fragmento aleatorio con su prueba en un plazo corto. Solo el almacenamiento asignado y auditado genera crédito.
- **Ledger.** Doble entrada, solo enteros, solo inserciones, con cadena de hashes. Emisión diaria por presupuesto con tope por humano.
- **Canje.** QR dinámico del comercio, intención firmada por la billetera, operación atómica e idempotente.
- **Cómputo distribuido.** Fuera de alcance hasta la Fase 5. No hay demanda definida ni forma barata de verificar resultados.

---

## 8. Flujos principales

**Publicar**

1. El cliente arma el evento `post.create` con texto, idioma, zona y modo de zona.
2. El núcleo lo canonicaliza, calcula el identificador y lo firma con la llave del dispositivo.
3. La pasarela valida esquema, firma, delegación del dispositivo, atestación de la cuenta, reloj y límites de frecuencia.
4. Se guarda el evento y se proyecta a las tablas de feed (una fila por nivel de zona).
5. Se notifica a los clientes suscritos.

**Leer un feed de zona**

1. El cliente pide `GET /v1/feed?zone=<id>` con un cursor.
2. El indexador responde con eventos completos y firmados.
3. El cliente verifica cada firma antes de mostrar.

**Canjear puntos**

1. El comercio genera un QR con importe, identificador de operación y caducidad.
2. La billetera lo escanea, muestra el detalle y firma la intención.
3. El ledger valida firma, saldo, caducidad e idempotencia, y asienta el débito y el crédito en una sola transacción.
4. Comercio y usuario reciben la confirmación.
