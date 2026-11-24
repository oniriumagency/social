# 04 - Modelo de Amenazas

Este documento define de quién nos defendemos, qué protegemos y qué no prometemos. Toda decisión de diseño de seguridad debe poder trazarse hasta aquí. Se revisa en cada compuerta de fase.

## 1. Activos

| ID | Activo | Daño si se compromete |
| :-- | :-- | :-- |
| AC-1 | Vínculo entre identidad civil y cuenta | Pérdida del anonimato. Irreversible. |
| AC-2 | Ubicación precisa y rastro de movimientos | Riesgo físico para el usuario. |
| AC-3 | Llaves privadas (raíz, dispositivo, billetera, secreto de identidad) | Suplantación, robo de cuenta o de puntos. |
| AC-4 | Unicidad humana (un humano, una cuenta base) | La red se llena de bots; el ledger se vacía. |
| AC-5 | Integridad del ledger | Pérdida económica y de confianza. |
| AC-6 | Disponibilidad e integridad del contenido | Censura o pérdida de datos. |
| AC-7 | Equipo del operador de nodo | Daño al hardware, consumo de datos, exposición legal. |
| AC-8 | Claves de los servicios (registro, vales, claves de contenido) | Emisión de atestaciones o vales falsos; lectura de media retirada. |

## 2. Adversarios

| ID | Adversario | Capacidad | ¿Cubierto? |
| :-- | :-- | :-- | :-- |
| AD-1 | Granjero de bots | Cliente modificado, miles de dispositivos e IP, dinero moderado. | Sí. Es el adversario principal del pilar 2. |
| AD-2 | Granjero de puntos | Nodos falsos, colusión entre nodos, colusión con un comercio. | Sí. |
| AD-3 | Usuario abusivo verificado | Una cuenta real usada para acosar o hacer spam. | Sí, con límites y moderación. Sanción temporal (`05` §12). |
| AD-4 | Operador curioso o coaccionado | Acceso total a las bases de datos y registros de un servicio del proyecto. | Sí para AC-1 y AC-2. El diseño asume que esto ocurrirá. |
| AD-5 | Estado emisor de documentos | Conoce los datos de cada documento que emitió. | Sí para AC-1, si no controla además la red del usuario. |
| AD-6 | Atacante web | XSS, dependencia maliciosa, phishing. | Sí, con defensas en profundidad. |
| AD-7 | Par malicioso en la red de nodos | Datos corruptos, denegación de servicio, sondeo. | Sí. |
| AD-8 | Observador de red local o proveedor de internet | Ve IP y tiempos del usuario. | Parcial: TLS siempre; ruta oculta para operaciones sensibles. |
| AD-9 | Observador de red global | Ve todo el tráfico. | **No.** Fuera de alcance. |
| AD-10 | Dispositivo comprometido | Malware con control del equipo del usuario. | **No.** Solo se limita el daño con caducidad y revocación de llaves. |
| AD-11 | Ladrón o poseedor temporal del documento físico | Acceso al chip del documento de otra persona. | Parcial: retraso y aviso en recuperación (`05` §11). |

## 3. Qué garantiza la privacidad de red en cada fase

| Operación | F1–F2 | F3–F4 | F5 |
| :-- | :-- | :-- | :-- |
| Leer feeds | HTTPS a la pasarela. La pasarela ve IP y zona consultada, y no las guarda juntas. | Igual. | Elección de pasarela; opción por HTTP Oblivious. |
| Publicar | HTTPS. La pasarela ve IP y cuenta en memoria, nunca en disco. | Igual. | Dandelion++ entre nodos. |
| Inscripción y alta de cuenta | HTTP Oblivious o Tor, obligatorio. | Igual. | Igual. |
| Canje y operaciones de billetera | No aplica. | HTTP Oblivious. | Igual. |
| Nodo | No aplica. | La IP del nodo es pública para los pares. El nodo no se relaciona con la cuenta social. | Igual. |

**Aclaración obligatoria en la interfaz y en la documentación pública:** un relevo de la red de nodos sirve para atravesar NAT y **no** oculta la IP.

## 4. Fronteras de confianza y separación de identidades

Tres identidades por persona, cada una con su llave, su servicio y su base de datos:

| Identidad | Vive en | Conoce | No debe conocer nunca |
| :-- | :-- | :-- | :-- |
| Cuenta social | Pasarela | Llave pública de la cuenta, publicaciones, zona declarada | Billetera, nodo, nullifier de documento, IP persistida |
| Billetera | Ledger | Llave pública de la billetera, saldos, canjes | Cuenta social, IP persistida |
| Nodo | Coordinador | Identificador de par, IP, capacidad, billetera de cobro | Cuenta social |
| Inscripción | Registro | Nullifiers de método (protegidos con HMAC), compromiso de identidad y su nivel | Cuenta, billetera, datos del documento |

Reglas duras:

1. Ninguna tabla contiene dos de estos identificadores a la vez, salvo `nodo → billetera` en el coordinador, que es necesaria para pagar.
2. Ningún registro (log) contiene una IP junto a cualquiera de estos identificadores.
3. Los servicios no comparten base de datos ni credenciales.
4. Ninguna función de analítica cruza servicios.

## 5. Modelo de costo Sybil

El objetivo no es que una cuenta falsa sea imposible; es que **cueste más de lo que rinde**.

| Vía de ataque | Costo aproximado por cuenta | Defensa |
| :-- | :-- | :-- |
| Cliente modificado sin documento | Infinito para el nivel 3 | La prueba exige la firma del Estado sobre el chip. En Colombia, el del pasaporte. |
| Reutilizar un documento | Infinito | Nullifier de documento único. |
| Alquilar documentos físicos | Alto: hay que tener cada documento en la mano | Una inscripción por documento; re-inscripción limitada por época. |
| Varias cuentas con un mismo humano | Precio de la cuenta secundaria, hasta el tope | Vales de pago y tope por humano. |
| Humano con dos documentos | Bajo, pero limitado a dos o tres | Riesgo residual aceptado. |
| Granja de dispositivos reales | Un dispositivo por cuenta | Solo alcanza el nivel 1: límites reducidos, sin imágenes, sin puntos, sin peso en el k-anonimato. |
| Burlar la atestación de la app (`doc-app`) | Medio: exige dispositivos manipulados y datos de documentos | Nivel 2 como máximo; sin puntos. Vigilancia de anomalías. |
| Anillo de avales entre humanos de nivel 3 | Bajo, pero con techo de 1,5 identidades por miembro y época | Cupo por avalista; nivel 2 como máximo; sin puntos. |
| Punto presencial corrupto | Soborno al operador | Cupo diario, auditoría, retirada de la autorización. |
| Documento falso con firma válida | Requiere comprometer a un Estado emisor | Lista de emisores aceptados revisable. |

El rendimiento por cuenta falsa se limita con tres frenos: los puntos exigen nivel 3 (`05` §4), hay un tope de puntos por humano (`09` §3), y los límites de frecuencia dependen del nivel (`06` §8). La regla de diseño es que **el valor económico solo es accesible por el método más caro de falsificar**.

## 6. Amenazas por componente y mitigación

| Componente | Amenaza | Mitigación |
| :-- | :-- | :-- |
| Núcleo | Dos implementaciones calculan identificadores distintos | Un solo núcleo en Rust; vectores dorados; pruebas de propiedades y fuzzing. |
| Núcleo | Maleabilidad de firmas | Verificación estricta de Ed25519; rechazo de codificaciones no canónicas. |
| Pasarela | Inyección de eventos falsos o repetidos | Validación completa antes de persistir; identificador por contenido; inserción idempotente. |
| Pasarela | Granjas en niveles bajos | Motor de riesgo sobre eventos públicos y retos de presencia en momentos impredecibles; el puntaje nunca sube el nivel. |
| Pasarela | Inundación | Límites por cuenta y por IP en memoria; tamaño máximo; coste por operación. |
| Pasarela | Media maliciosa o con metadatos | Recodificación de imágenes; eliminación y verificación de EXIF; lista de tipos permitidos; análisis fuera del proceso principal. |
| Web | XSS y robo de llaves | CSP estricta sin `unsafe-inline`; cero scripts de terceros; llaves no extraíbles; llaves de dispositivo con caducidad. |
| Web | El servidor sirve JavaScript malicioso | Builds reproducibles y publicados; la app de escritorio como alternativa verificable; riesgo residual documentado. |
| Registro | Repetición o robo de una prueba | La prueba compromete el compromiso de identidad y un reto del servidor. |
| Registro | Fuga de la base de datos | Nullifiers guardados con HMAC y clave en KMS; ninguna relación con cuentas. |
| Registro | Emisión de atestaciones falsas por un operador | Registro de transparencia público de raíces y atestaciones; firma umbral en F5. |
| Nodo | Daño al equipo del operador | Gobernador de recursos como única puerta; topes duros; pausa ante señales dudosas. |
| Nodo | Contenido ilegal en su disco | Solo fragmentos cifrados sin clave; borrado de clave al retirar. |
| Nodo | Ejecución remota de código | Sin ejecución de código de terceros; análisis de mensajes con límites; actualizaciones firmadas. |
| Coordinador | Nodo que finge almacenar | Auditorías aleatorias con plazo; solo datos asignados. |
| Ledger | Doble gasto, condiciones de carrera | Aislamiento serializable; claves de idempotencia; restricción de saldo no negativo. |
| Ledger | Manipulación interna | Solo inserciones; cadena de hashes; raíz diaria publicada; conciliación automática. |
| Canje | Captura o repetición del QR | El QR lo genera el comercio, es de un solo uso y caduca en 120 segundos. |
| Cadena de suministro | Dependencia comprometida | `cargo-deny`, `cargo-audit`, auditoría de pnpm, archivos de bloqueo, aprobación humana para dependencias nuevas en zonas sensibles. |
| Proceso | Secretos expuestos en una transmisión | Secretos fuera del repositorio y de la terminal; escáner en CI y antes de cada commit. |

## 7. Supuestos

1. Las primitivas criptográficas elegidas son seguras.
2. El proveedor de prueba de documento y Semaphore v4 son correctos. Se revisa con auditoría externa en G2.
3. Los Estados emisores aceptados no emiten documentos falsos a escala.
4. El sistema operativo y el almacén seguro del dispositivo del usuario no están comprometidos.
5. El KMS que guarda las claves de servicio es íntegro.
6. En Colombia, país del piloto, el diseño es legal mientras ningún dato biométrico ni del documento salga del dispositivo y la verificación no se premie (`10` §6.1). Lo confirma el dictamen de F2-T20. El regulador puede ordenar el cierre de una operación que trate datos sensibles sin esas garantías.

Si un supuesto deja de cumplirse, se abre un ADR y se revisa este documento.
