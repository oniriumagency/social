# 16 - Glosario

| Término | Significado en este proyecto |
| :-- | :-- |
| **ADR** | Decisión de arquitectura registrada. Todas viven en `docs/adr/registro-de-decisiones.md`. |
| **Ámbito (scope)** | Cadena que fija para qué sirve un nullifier de Semaphore. Mismo ámbito e identidad dan el mismo nullifier. |
| **Atestación** | Evento firmado por el registro que declara que una llave de cuenta pasó la verificación, con su nivel de confianza y su vencimiento. |
| **Atestación de app** | Certificación del sistema operativo (Play Integrity, App Attest) de que la app no fue modificada. |
| **Billetera** | Identidad separada de la cuenta social que posee puntos. Una por humano. |
| **Catálogo de métodos** | Archivo versionado que indica, por país, qué documentos llevan chip y qué métodos de verificación son viables. El cliente lo filtra en local. |
| **Catálogo de zonas** | Conjunto versionado de zonas con su jerarquía y cobertura. |
| **Cédula** | Documento de identidad colombiano. La de ciudadanía se expide a los 18 años. Existe en formato amarillo con hologramas y en formato digital de policarbonato. Aquí entra por el método `doc-app`. |
| **cid** | Identificador de contenido: hash BLAKE3 del archivo. |
| **Clave de contenido** | Clave simétrica aleatoria con la que se cifra un archivo de media. |
| **Compromiso de identidad** | Valor público derivado del secreto de identidad. Es la hoja del árbol del grupo de humanos. |
| **Compuerta (G0–G4)** | Punto de control entre fases que aprueba el responsable humano. |
| **Coordinador** | Servicio que asigna fragmentos a nodos y los audita. |
| **Copia ancla** | Copia siempre disponible de la media en almacenamiento de objetos del proyecto. |
| **DANE y DIVIPOLA** | Departamento de estadística de Colombia y su codificación oficial de departamentos y municipios. Fuente candidata del nivel ciudad. |
| **Delegación** | Permiso firmado por la llave raíz para que una llave de dispositivo firme eventos. |
| **Época** | Periodo de 180 días tras el cual las atestaciones deben renovarse. |
| **Evento** | Unidad de datos del protocolo: JSON firmado con identificador por contenido. |
| **Gobernador de recursos** | Componente del nodo que autoriza o niega todo uso de disco, CPU y red. |
| **Grupo de humanos** | Árbol de Merkle con los compromisos de identidad de las personas inscritas. Hay uno por nivel de confianza. |
| **H3** | Sistema de celdas hexagonales jerárquicas. Aquí sirve de apoyo, no de catálogo. |
| **HTTP Oblivious** | Protocolo (RFC 9458) que oculta la IP del cliente al servicio mediante un relevo independiente. |
| **Indexador** | Parte de la pasarela que construye y sirve los feeds. |
| **Inscripción** | Paso en el que un método de verificación se asocia a un compromiso de identidad y este entra en el grupo de su nivel. |
| **Invariante** | Regla que ninguna tarea puede romper. Listadas en `CLAUDE.md` y `ANTIGRAVITY.md`. |
| **JCS** | Esquema de canonicalización de JSON (RFC 8785). |
| **k-anonimato** | Una zona solo se publica si tiene al menos `k` cuentas activas. |
| **Lápida** | Marca que queda cuando se borra una publicación. |
| **Ledger** | Libro mayor de puntos de doble entrada. |
| **Llave raíz** | Llave Ed25519 que identifica a una cuenta. |
| **Método de verificación** | Camino para demostrar humanidad (`chip`, `cred`, `doc-app`, `cuenta`, `aval`, `presencial`, `dispositivo`). Cada uno tiene una fuerza. |
| **Milipunto** | Unidad entera mínima del ledger. 1000 milipuntos son 1 punto. |
| **Modo de zona** | `home`, `live` o `remote`: cómo se eligió la zona de una publicación. |
| **Motor de riesgo** | Módulo de la pasarela que detecta actividad sospechosa usando solo eventos públicos. |
| **Nivel de confianza** | Número de 1 a 3 que indica lo difícil que es falsificar una cuenta. Lo decide el método más fuerte, no la cantidad. Solo el nivel 3 significa humano único. |
| **Nodo** | Proceso de escritorio o servidor que guarda fragmentos cifrados. |
| **Nullifier de método** | Valor único por método y fuente (documento, cuenta, dispositivo). Impide usar la misma fuente para dos identidades. |
| **Nullifier de ámbito** | Valor único por identidad y ámbito. Impide crear dos cuentas base en una época. |
| **Núcleo** | Crates `social-core`, `social-crypto` y `social-zones`. |
| **Paquete de catálogo** | Archivo con las zonas de una región, que el cliente descarga para resolver en local. |
| **Pasarela (gateway)** | Servicio que recibe eventos y sirve feeds, hilos y media. |
| **Proveedor de humanidad** | Implementación de `HumanityProvider` que verifica un método de verificación. |
| **Punto de inscripción** | Lugar, normalmente un comercio aliado, con un teléfono con NFC para leer el chip de quien no tiene uno, o autorizado para la verificación presencial. |
| **PRF** | Extensión de WebAuthn que deriva un secreto estable a partir de una passkey. |
| **Puntaje de confianza** | Valor interno de 0 a 100 por cuenta. Sube al superar verificaciones y baja al fallarlas o ignorarlas. Nunca cambia el nivel. |
| **Registro de humanidad** | Servicio que verifica inscripciones y emite atestaciones. |
| **Registro de transparencia** | Lista pública, solo de inserción, de raíces del grupo y atestaciones emitidas. |
| **Reto de presencia** | Verificación adicional ligera: prueba de pertenencia más passkey con biometría o PIN. Demuestra que alguien atiende el dispositivo. |
| **Reto de método** | Verificación adicional fuerte: repetir un método o añadir uno nuevo. |
| **Secreto de identidad** | Secreto de Semaphore del usuario. Nunca sale de su dispositivo. |
| **Registraduría** | Registraduría Nacional del Estado Civil: entidad que expide la cédula en Colombia. |
| **Semaphore** | Protocolo de conocimiento cero para probar pertenencia a un grupo de forma anónima. |
| **SIC** | Superintendencia de Industria y Comercio: autoridad de protección de datos y de consumo en Colombia. |
| **Spike** | Investigación acotada con prototipo desechable. Su entregable es un informe. |
| **Sybil** | Ataque que consiste en crear muchas identidades falsas. |
| **Vale ciego** | Comprobante de pago firmado sin que el firmante vea su contenido (RFC 9474). |
| **Vector dorado** | Caso de prueba fijo que define el comportamiento exacto del protocolo. |
| **Zona** | Área geográfica con identificador estable y nivel en la jerarquía. |
| **Zona fija** | Zona elegida por el usuario que se aplica a sus publicaciones por defecto. |
| **Zona sensible** | Parte del repositorio cuya modificación exige revisión humana (`10` §8). |
