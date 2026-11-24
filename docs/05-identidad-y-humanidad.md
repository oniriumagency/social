# 05 - Identidad y Prueba de Humanidad

Este es el componente de mayor riesgo del proyecto. **Ningún agente escribe ni modifica circuitos ZK ni primitivas criptográficas.** El país piloto es **Colombia** (§5.1). Todo lo de este documento se valida en el spike S1 y pasa una auditoría externa antes de la beta pública (compuerta G2).

## 1. Objetivos

1. Que cualquier persona pueda registrarse, tenga o no NFC, por al menos un camino.
2. Que cada cuenta lleve un **nivel de confianza** honesto: cuánto cuesta falsificarla.
3. Que solo el nivel más alto signifique "humano único", y que el dinero dependa de ese nivel.
4. Que nadie, ni el operador del registro ni el Estado emisor, pueda relacionar un documento o un método con una cuenta.
5. Que ningún dato del documento ni biométrico salga del dispositivo que lo procesa.
6. Que perder el teléfono no sea perder la cuenta.

## 2. Por qué la biometría y el OCR no prueban unicidad por sí solos

| Método aislado | Qué prueba de verdad | Por qué no basta |
| :-- | :-- | :-- |
| Biometría del dispositivo (Face ID, huella, Passkeys) | Que quien usa el aparato es quien lo configuró. | La plantilla vive dentro del chip seguro y nunca se compara con la de otros usuarios. La misma persona en N dispositivos da N resultados válidos, y una granja puede registrar la misma huella en mil aparatos. Detectar duplicados exigiría una base central de biometría, que el pilar 3 prohíbe. |
| OCR o lectura de MRZ del documento | Que el cliente hizo un cálculo sobre *algún* texto. | El texto impreso no lleva firma. Con cliente de código abierto, un atacante introduce números inventados o filtrados y la prueba ZK sale válida. La prueba de vida corre en un dispositivo que el atacante controla. |

**Principio que sale de aquí:** un método vale tanto como el **tercero difícil de falsificar** que respalda el dato. La firma del Estado en un chip, la firma de una institución en un correo, la atestación del sistema operativo sobre una app, o el cupo limitado de un humano ya verificado.

## 3. Catálogo de métodos de verificación

Cada método es una implementación de la interfaz `HumanityProvider`:

```
verify(prueba, reto, compromiso) → { method, method_nullifier, strength }
```

| Código | Método | Quién respalda el dato | Necesita NFC | Fuerza | Disponible |
| :-- | :-- | :-- | :-- | :-- | :-- |
| `chip` | Chip NFC de pasaporte o documento de identidad, con prueba ZK de la firma del emisor | El Estado emisor | Una vez, en **cualquier** teléfono: propio, prestado o de un punto de inscripción | **Fuerte** | F2 |
| `cred` | Credencial digital firmada por el emisor: código QR firmado, cartera de identidad digital, licencia de conducir móvil | El emisor | No | **Fuerte** | Cuando exista en el país. En Colombia no se encontró ninguna utilizable (§5.1) |
| `doc-app` | Documento por cámara y prueba de vida, ejecutados dentro de una app cuya integridad atesta el sistema operativo | Apple o Google, que certifican que la app no fue modificada | No | Media | F2, si S1 lo valida |
| `cuenta` | Prueba ZK de que se posee una cuenta en una institución que verifica identidad (por ejemplo, un correo con firma DKIM de un banco u operador) | La institución | No | Media | F2 experimental, si S1 lo valida |
| `aval` | Aval de dos humanos de nivel 3 | El cupo limitado de cada avalista | No | Media | F2 |
| `presencial` | Verificación en persona en un punto autorizado (comercio aliado), que entrega un vale firmado a ciegas | El operador del punto, con cupo y auditoría | No | Media | F4, o antes con puntos piloto |
| `dispositivo` | Dispositivo real atestado más passkey con biometría | El fabricante del dispositivo | No | Baja | F2 |

**Notas por método**

- **`chip` con teléfono ajeno.** La prueba se liga al compromiso de identidad del usuario, que viaja por QR al teléfono que lee. El teléfono lector no conserva nada. Esto permite **puntos de inscripción** en comercios aliados. Aviso obligatorio al usuario: los datos del chip se procesan en el teléfono que lee, así que debe ser de confianza.
- **`doc-app`.** Rescata la ruta original de escaneo. Solo funciona en móvil, depende de Apple y Google, y existen técnicas para saltarse la atestación; por eso es fuerza media y no fuerte. Su nullifier se deriva del número de documento, que es de baja entropía: debe calcularse con una función seudoaleatoria oblivia cuya clave custodie un tercero independiente, para que el registro no pueda enumerar números de documento. Diseño a validar en S1.
- **`cuenta`.** Una persona puede tener cuentas en varias instituciones. El método cuenta una sola vez por identidad.
- **`aval`.** Cada humano de nivel 3 puede avalar a 3 identidades por época, y cada identidad necesita 2 avales. Un grupo coludido de nivel 3 produce como mucho 1,5 identidades de nivel 2 por miembro y época. Como las cuentas son inenlazables, no se puede sancionar al avalista por lo que haga el avalado; el cupo es el único freno.
- **`presencial`.** El operador ve a la persona y su documento, como en cualquier trámite, pero el vale ciego le impide saber qué cuenta se crea. Cupo diario por punto y revisión de anomalías.
- **`dispositivo`.** Exige atestación real del hardware o de la app (Play Integrity, App Attest). Una passkey sola no cuenta, porque se pueden crear por software sin límite. En web y en escritorio sin atestación este método no está disponible y el registro deriva al teléfono.
- **Descartados:** biometría con base central, incluida la consulta facial contra la base de un Estado (contradice el pilar 3); número de teléfono (barato de conseguir y revela un dato personal); OCR sin atestación (no resiste a un cliente modificado).

## 4. Niveles de confianza

El nivel se decide por la **combinación más fuerte**, no por la suma. Diez métodos de fuerza baja siguen siendo nivel 1.

| Nivel | Nombre | Requisito | Qué significa |
| :-- | :-- | :-- | :-- |
| 0 | Visitante | Ninguno | Sin cuenta. Solo lee. |
| 1 | Básico | Un método de fuerza baja | Hay un dispositivo real detrás. Falsificar una cuenta cuesta un dispositivo. |
| 2 | Verificado | Un método de fuerza media más `dispositivo`, o dos métodos de fuerza media distintos | Humano probable. Falsificar exige engañar a una institución, a la atestación del sistema o a dos personas verificadas. |
| 3 | Humano único | Un método de fuerza fuerte | Un documento oficial, una identidad. Es el único nivel que respalda la promesa "un humano, una cuenta". |

**Privilegios por nivel** (valores iniciales; decisión D-15)

| Capacidad | Nivel 1 | Nivel 2 | Nivel 3 |
| :-- | :-- | :-- | :-- |
| Publicar y responder | Sí, con límites reducidos (`06` §8) | Sí | Sí |
| Adjuntar imágenes | No | Sí | Sí |
| Cuenta para el umbral de k-anonimato de una zona | No | Sí | Sí |
| Peso en los feeds de niveles altos | Reducido | Normal | Normal |
| Avalar a otros | No | No | Sí |
| Cuentas secundarias de pago | No | No | Sí |
| Billetera y puntos | No | No (D-15) | Sí |
| Operar un nodo | Sí, sin cobrar | Sí, sin cobrar | Sí, cobrando |

**Reglas del medidor**

1. **Fuerza, no cantidad.**
2. **Un uso por método.** Cada método produce su propio nullifier; el mismo documento, la misma cuenta bancaria o el mismo dispositivo no sirven para dos identidades.
3. **El medidor es interno.** El registro conoce los métodos de cada compromiso de identidad. La atestación de la cuenta solo lleva el nivel (1, 2 o 3). Publicar la lista de métodos sería una huella que identifica a la persona.
4. **Subir de nivel es añadir un método** a la misma identidad. La cuenta, sus seguidores y su historial se conservan.
5. **El dinero va con el nivel 3.** Las granjas atacan donde hay valor económico.
6. **El nivel se ve.** Los clientes muestran una insignia de nivel y permiten filtrar por nivel mínimo (decisión D-14).
7. **Verificarse nunca se premia.** No se entrega ningún punto, saldo ni ventaja económica por completar o añadir un método. Los puntos pagan almacenamiento auditado (`09` §3). Un premio a cambio de datos de identidad vicia el consentimiento (`10` §6.1) y además atrae a las granjas.

**Límite honesto.** Métodos distintos no se pueden comparar entre sí sin romper la privacidad. Una persona podría crear una identidad con `chip` y otra con `aval`. El número de identidades por persona queda acotado por el número de métodos disponibles, y solo una de ellas alcanza el nivel 3 por documento. Por eso los niveles 1 y 2 se comunican como "humano probable", no como "humano único".

## 5. Registro adaptativo: dispositivo y país del documento

El formulario de registro **nunca ofrece un método que el usuario no puede completar**. No se pide leer un chip a quien tiene un documento sin chip, ni a quien no tiene cómo leerlo.

**Paso 1. Capacidades del dispositivo (detección local)**

| Capacidad | Cómo se detecta | Qué habilita |
| :-- | :-- | :-- |
| NFC capaz de leer chips | API nativa de la app móvil. Un navegador no puede leer chips aunque el teléfono tenga NFC. | `chip` en este dispositivo |
| Atestación de app y de dispositivo | Play Integrity o App Attest, solo en app móvil | `doc-app`, `dispositivo` |
| Autenticador con verificación de usuario | WebAuthn | Passkey y retos de presencia |
| Extensión PRF | WebAuthn | Derivación del secreto de identidad |
| Cámara | Permiso del sistema | `doc-app` y lectura de QR |

La lista de capacidades **no se envía al servidor**: sería una huella del dispositivo. El servidor solo se entera del método que el usuario termina usando.

**Paso 2. País emisor y tipo de documento**

- El usuario indica el **país que emitió su documento**, que no tiene por qué ser el país donde está.
- Se sugiere a partir del país de la zona elegida o de la región configurada en el dispositivo. **Nunca por IP.**
- La elección se queda en el dispositivo.

**Paso 3. Catálogo de métodos por país**

Archivo versionado y firmado, en `spec/methods/`, que el cliente descarga **completo** (todos los países) y filtra en local. Así el servidor no sabe qué país consultó.

```json
{ "catalog": 1, "country": "CO", "status": "draft",
  "documents": [
    { "type": "passport", "chip": true, "chip_since": null, "provider": "chip" },
    { "type": "passport_mrz", "chip": false },
    { "type": "national_id_digital", "chip": false, "mrz": true },
    { "type": "national_id_legacy", "chip": false }
  ],
  "methods": {
    "chip":       { "strength": "strong", "documents": ["passport"] },
    "cred":       { "available": false },
    "doc-app":    { "strength": "medium",
                    "documents": ["national_id_digital", "national_id_legacy", "passport_mrz"] },
    "cuenta":     { "strength": "medium", "issuers": [] },
    "aval":       { "strength": "medium" },
    "presencial": { "strength": "medium", "points": true },
    "dispositivo":{ "strength": "low" }
  },
  "max_level": 3 }
```

- El ejemplo es el **borrador de Colombia** (§5.1). `chip_since` y la lista de `issuers` quedan vacíos hasta que el spike S1 los confirme.
- Los datos del catálogo viven en el repositorio. Cada cambio entra por PR y cita su fuente.
- La versión 1, para Colombia, es un entregable del spike S1.
- Cambiar la fuerza de un método exige ADR.

**Paso 4. Opciones que se muestran**

`opciones = catálogo[país] ∩ capacidades del dispositivo`, ordenadas por el nivel que permiten alcanzar. Cada opción dice qué prueba, qué nivel da y qué hace falta.

| Situación | Qué hace el formulario |
| :-- | :-- |
| El país no emite documentos con chip verificable | `chip` no aparece. Se muestran los métodos disponibles y el nivel máximo alcanzable. |
| El documento tiene chip y el dispositivo no tiene NFC | Ofrece "usar otro teléfono" (por QR) y "punto de inscripción cercano", además de los métodos que sí puede completar ahora. |
| El país emite chip solo desde cierto año | Pregunta si el documento lleva impreso el símbolo de chip y muestra cómo reconocerlo. |
| Navegador o escritorio | Deriva al teléfono por QR los métodos que exigen app móvil. |
| El nivel máximo del país es 2 | Lo dice con claridad, explica que los puntos no están disponibles todavía y ofrece avisar cuando haya un método fuerte. |
| Ningún método disponible | Modo visitante y lista de espera. No se inventa un método débil para salir del paso. |

El usuario puede empezar con lo que tiene y **subir de nivel después** sin perder su cuenta.

### 5.1 Colombia: punto de partida del catálogo

Datos consultados el 2026-11-24 en fuentes públicas (Cancillería, prensa nacional y un informe de la Misión de Observación Electoral de 2023). **No son fuente primaria de la Registraduría ni resultado de pruebas con documentos reales: el spike S1 los confirma o los corrige antes de publicar el catálogo.**

| Documento o servicio | Qué se sabe | Método | Qué debe confirmar S1 |
| :-- | :-- | :-- | :-- |
| **Pasaporte electrónico.** Modelo anterior con chip y modelo nuevo, que se expide desde el 1 de abril de 2026 (personalización de la Imprenta Nacional, libretas de la Casa de la Moneda de Portugal) | Lleva chip electrónico y la Cancillería declara que cumple las disposiciones de la OACI. Los pasaportes anteriores valen hasta su vencimiento. | `chip`, fuerza fuerte, nivel 3 | Que el proveedor de prueba acepte los certificados de firma de **las dos generaciones**: el cambio de fabricante de 2026 puede traer certificados nuevos. Prueba real con un pasaporte de cada modelo. |
| **Pasaporte de lectura mecánica sin chip** | Sigue vigente hasta su vencimiento. | `doc-app` | Cómo distinguirlo del electrónico en el formulario (símbolo de chip en la tapa). |
| **Cédula digital** (policarbonato, desde 2020) | Zona de lectura mecánica y código QR cifrado que, según las fuentes, solo leen dispositivos especiales. Ninguna fuente consultada describe un chip sin contacto en la tarjeta. | `doc-app`, fuerza media | Con la Registraduría: si la tarjeta lleva algún chip y si el QR tiene una firma que un tercero pueda verificar. Si la tuviera, pasaría a `cred`. |
| **Cédula amarilla con hologramas** | Dejó de producirse en septiembre de 2022 y seguía vigente en las fuentes consultadas. A comienzos de 2023 cerca del 97% de los adultos aún no había cambiado a la digital. | `doc-app`, fuerza media | Vigencia actual y proporción real de cada formato. |
| **App Cédula Digital Colombia** | Se activa con reconocimiento facial y funciona sin conexión. No se encontró documentación de una verificación criptográfica abierta a terceros. | `cred` no disponible | Si existe una interfaz de verificación para terceros y en qué condiciones. |
| **Autenticación facial de la Registraduría** (comparación 1 a 1 contra su base) | Existe como servicio para entidades. | **Descartado** | Nada. Exige enviar el rostro a una base central: contradice el pilar 3 y la invariante I-1. |
| **Cédula de extranjería y Permiso por Protección Temporal** | Sin datos. | Por evaluar para `doc-app` | Formato, elementos de seguridad y volumen de personas. |
| **Cuentas en entidades vigiladas** (bancos y billeteras digitales) | Verifican identidad al abrir la cuenta. | `cuenta`, fuerza media, por validar | Si sus correos llevan firma DKIM estable y un contenido que pruebe titularidad sin revelar el número de cuenta. |

**Consecuencias para el diseño**

1. **La cédula, que es el documento que todo adulto tiene, da nivel 2 como máximo.** El nivel 3 depende del pasaporte electrónico. La mayoría de los primeros usuarios será de nivel 1 o 2, y el producto debe funcionar bien para ellos.
2. **Los puntos de inscripción ayudan a quien tiene pasaporte y no tiene NFC.** No ayudan a quien no tiene pasaporte.
3. **El método `aval` arranca despacio**, porque necesita humanos de nivel 3.
4. **La decisión D-15 pesa más aquí.** Si pocos adultos tienen pasaporte con chip, pocos podrán operar un nodo remunerado. S1 entrega la cifra; hasta entonces el valor por defecto no cambia y no se abren los puntos a niveles inferiores para salir del paso.
5. **La cédula de ciudadanía se expide a los 18 años.** En `doc-app`, el tipo de documento ya indica mayoría de edad, sin leer la fecha de nacimiento.
6. **Del chip del pasaporte solo se lee lo mínimo** que exige la prueba de la firma del emisor. Nunca se leen ni se guardan huellas ni iris, aunque el chip los contenga.
7. **Extranjeros residentes:** el catálogo se consulta por país emisor, así que usan el documento de su país. Los documentos colombianos para extranjeros se evalúan en S1.

## 6. Puntaje de confianza, vigilancia y verificación adicional

Hay dos medidas distintas y no deben mezclarse:

| | Nivel de confianza | Puntaje de confianza |
| :-- | :-- | :-- |
| Qué mide | La fuerza de los métodos de verificación | El comportamiento de la cuenta en el tiempo |
| Valores | 1, 2 o 3 | 0 a 100 |
| Sube cuando | Se añade un método más fuerte | Se superan verificaciones; pasa el tiempo sin incidencias |
| Baja cuando | Un método caduca o se retira | Se falla o se ignora una verificación; hay señales de riesgo o sanciones |
| Quién lo ve | Todos, como insignia | Solo el propio usuario, por tramos (bajo, medio, alto) |
| Dónde vive | Registro, por identidad | Pasarela, por cuenta |

**Regla dura:** el puntaje se mueve dentro de la **banda de su nivel** y nunca concede privilegios de un nivel superior. Una granja puede superar verificaciones; eso demuestra que hay alguien atendiendo el dispositivo, no que sea un humano único.

| Nivel | Puntaje inicial | Tope de la banda |
| :-- | :-- | :-- |
| 1 | 30 | 60 |
| 2 | 50 | 80 |
| 3 | 70 | 100 |

**Para qué sirve el puntaje:** decide la frecuencia de las verificaciones, ajusta los límites de publicación dentro de los del nivel, pondera el peso en los feeds de niveles altos y ordena la cola de revisión de moderación.

### 6.1 Vigilancia

Un **motor de riesgo** en la pasarela observa todas las cuentas, con umbrales más sensibles cuanto menor es el puntaje.

- **Solo usa eventos públicos y firmados:** ritmo de publicación, texto casi duplicado entre cuentas, coordinación en el tiempo, reportes recibidos, anillos de seguimiento y reacción, cambios de zona, proporción de publicaciones `remote`, antigüedad.
- **Tiene prohibido usar:** IP, huella del dispositivo, datos del registro, del ledger o del coordinador, y cualquier dato de terceros.
- Las **categorías** de señales son públicas. Los umbrales no, para que no se puedan esquivar.
- **Los reportes no restan puntaje por sí solos.** Un reporte abre una revisión; solo una señal confirmada baja el puntaje. Así un grupo no puede hundir a alguien a base de reportes falsos.

### 6.2 Verificación adicional

| Tipo | En qué consiste | Cuándo |
| :-- | :-- | :-- |
| **Reto de presencia** | Prueba Semaphore del mismo ámbito, más passkey con verificación de usuario (biometría o PIN del dispositivo), más atestación del dispositivo donde exista. Para una persona es un toque. | Al azar y ante señales de riesgo leves |
| **Reto de método** | Repetir un método ya usado o añadir uno nuevo. | Ante riesgo alto o tras fallos repetidos |

Los retos llegan en **momentos impredecibles**. A una granja la obligan a tener a alguien atendiendo cada dispositivo sin saber cuándo.

| Tramo de puntaje | Frecuencia media del reto al azar |
| :-- | :-- |
| Bajo (menos de 30) | Cada 7 días |
| Medio (30 a 70) | Cada 30 días |
| Alto (más de 70) | Cada 120 días |
| Nivel 3 con puntaje alto | Solo ante señales de riesgo |

Plazo para responder: 72 horas.

| Resultado | Efecto en el puntaje | Efecto en la cuenta |
| :-- | :-- | :-- |
| Superado | +5, hasta el tope de la banda | El siguiente reto se aleja |
| Ignorado (vence el plazo) | −5, una sola vez | La cuenta queda **en pausa** (solo lectura) hasta superarlo. Al superarlo se reanuda sin más penalización. |
| Fallido (prueba inválida) | −15 | Nuevo reto inmediato |
| Tres fallidos seguidos, o puntaje 0 | — | Atestación suspendida hasta superar un reto de método o hasta revisión humana |
| Semana activa sin incidencias | +1 | — |
| Señal de riesgo confirmada | −10 a −30 | Según la gravedad, reto de método |

**Ausentarse no es fallar.** Quien se va de viaje encuentra su cuenta en pausa, supera el reto y sigue. Una cuenta inactiva no acumula penalizaciones.

### 6.3 Privacidad y garantías

1. **El reto no enlaza cuenta e identidad.** La prueba Semaphore con el mismo ámbito produce el mismo nullifier que el alta, así que el registro confirma que responde la misma identidad dueña de la cuenta sin saber cuál es. El registro firma un evento `account.presence`.
2. **El estado es privado.** Solo el propio usuario consulta su puntaje, sus retos pendientes y el motivo por categoría, con una petición firmada. El público solo ve la insignia de nivel.
3. **Sin terceros.** El reto no carga servicios externos.
4. **Ninguna decisión definitiva es automática.** La pausa es reversible por el propio usuario. Suspender una atestación exige revisión humana y admite apelación.
5. **Sin discriminación por país o por método.** El motor se evalúa por su tasa de falsos positivos, desglosada por nivel, y esa tasa se publica en el informe de transparencia.
6. **Accesible.** El reto funciona con lector de pantalla y admite PIN en lugar de biometría.

## 7. Capa 1: inscripción de métodos

**Actor:** el dispositivo del usuario (o el teléfono lector, en `chip`).

1. El usuario genera en su dispositivo un **secreto de identidad** (§11) y calcula su **compromiso de identidad** `C` (formato Semaphore v4).
2. Pide al registro un reto de un solo uso.
3. Ejecuta el método elegido. El resultado es una prueba que afirma lo propio del método, entrega el nullifier del método y está ligada a `C` y al reto. Para `chip`, la prueba afirma además que el documento es de un emisor aceptado, está vigente y su titular tiene 18 años o más.
4. El registro verifica, comprueba que el nullifier de ese método no existe, y guarda `HMAC(clave_KMS, método ‖ nullifier) → C`.
5. El registro recalcula el nivel de `C` y lo encola para el **grupo** de ese nivel.
6. Cada 10 minutos se añaden los lotes a los árboles y se publican las raíces en el registro de transparencia.

Hay **un grupo Semaphore por nivel** (G1, G2, G3). Una identidad está en el grupo de su nivel y en los inferiores.

- El registro **nunca** recibe nombre, número, foto, nacionalidad ni fecha de nacimiento. La nacionalidad solo se revela si una decisión legal lo exige (D-07).
- En la alfa de F1 el único método es `InviteProvider`, que usa el mismo camino.
- El lote de 10 minutos agrupa a todos los inscritos de la ventana para dificultar la correlación por hora.

## 8. Capa 2: alta de cuenta

**Tecnología:** Semaphore v4, sin modificaciones. Permite probar "soy miembro del grupo" sin decir cuál, y produce un nullifier determinista por **ámbito** que no depende del grupo.

1. El cliente genera la llave raíz de la cuenta (Ed25519).
2. Obtiene su camino de Merkle sin revelar su hoja: descarga el bloque de hojas que la contiene más la parte alta del árbol.
3. Genera la prueba Semaphore contra el grupo de su nivel con:
   - raíz: una de las vigentes (últimas 24 horas);
   - ámbito: `social/v1/acct/0/epoch/<N>`;
   - mensaje: `hash(llave_pública_de_cuenta ‖ N)`.
4. Envía prueba y llave pública al registro **por la ruta que oculta la IP**.
5. El registro verifica, comprueba que el nullifier de ámbito no existe, lo guarda con el nivel y firma la **atestación**:

```json
{ "kind": "account.attest", "account": "<pubkey>", "tier": "base", "level": 2,
  "epoch": 12, "expires_at": 1790000000, "issuer": "<registry key id>" }
```

6. Pasarelas, indexadores y nodos verifican la atestación con la llave pública del registro, sin consultarle.

**Subir de nivel.** Tras añadir un método, el cliente presenta una prueba contra el grupo superior con el mismo ámbito. El nullifier es el mismo, así que el registro reconoce la cuenta y emite una atestación con el nivel nuevo.

**Épocas.** Una época dura 180 días. En los 30 días previos al vencimiento, el cliente renueva solo con el ámbito de la época siguiente y una firma de la llave de la cuenta. Las cuentas huérfanas caducan solas.

## 9. Cuentas secundarias de pago

Solo nivel 3.

1. El usuario paga. El pago no se asocia a ninguna cuenta.
2. El cliente pide un **vale firmado a ciegas** (RFC 9474).
3. Más tarde, por la ruta que oculta la IP, presenta el vale, una prueba Semaphore contra G3 con ámbito `social/v1/acct/<i>/epoch/<N>` (con `i` de 1 al tope) y la llave pública de la cuenta nueva.
4. El registro emite una atestación con `tier: "secondary"`.

Tope por defecto: 5 por humano (D-08). No se pueden relacionar entre sí ni con la base. En tiendas de apps, las compras digitales pueden exigir el sistema de pago de la tienda.

## 10. Billetera

En F4, cada identidad de nivel 3 crea una billetera con el ámbito `social/v1/wallet/epoch/<N>`. Es otra llave, otra atestación y otro servicio, inenlazable con la cuenta social. El tope de puntos se aplica por billetera.

## 11. Llaves, dispositivos y recuperación

| Secreto | Dónde vive | Cómo se recupera |
| :-- | :-- | :-- |
| Secreto de identidad (Semaphore) | Derivado con HKDF de la salida PRF de una passkey sincronizada, o aleatorio en el almacén seguro si no hay PRF | Passkey sincronizada; frase de recuperación de 24 palabras |
| Llave raíz de cuenta | Almacén seguro del sistema operativo; en web, llave no extraíble | Copia cifrada con una clave derivada del secreto de identidad; frase de recuperación |
| Llave de dispositivo | En cada dispositivo, nunca se copia | No se recupera: se emite una nueva |
| Llave de billetera | Igual que la llave raíz | Igual que la llave raíz |

**Delegación.** La llave raíz firma `device.grant` para cada dispositivo, con permisos y caducidad. `device.revoke` lo anula.

**Escalera de recuperación**

1. **Otro dispositivo propio activo:** autoriza al nuevo con un QR.
2. **Passkey sincronizada o frase:** se regenera el secreto de identidad y se descifra la copia de la llave raíz.
3. **Se perdió todo:** se repite un método ya usado. El registro reconoce su nullifier, retira el compromiso anterior de los árboles y añade el nuevo con el nivel que ese método otorga. Límite: una vez por época y método. La cuenta anterior caduca al final de su época; la identidad nueva crea una cuenta nueva. Seguidores e historial no se trasladan, porque nada relaciona ambas cuentas.

**Rotación de llave raíz comprometida:** `account.rotate` firmado por la llave antigua, o autorizado por el registro con una prueba del mismo ámbito y época. Retraso de 72 horas, cancelable por la llave antigua.

## 12. Sanciones y anonimato

- Una sanción se aplica a **una cuenta**: se revoca su atestación.
- El nullifier de la cuenta base de esa época queda consumido.
- **La sanción máxima efectiva dura lo que quede de época**, 180 días como mucho.

Es consecuencia directa del pilar de privacidad (D-06). Las listas de bloqueo anónimas son una línea de investigación para F5.

## 13. Riesgos y límites

| Riesgo | Estado |
| :-- | :-- |
| **Cobertura.** No todo el mundo tiene documento con chip ni teléfono con NFC. En Colombia el único documento con chip confirmado es el pasaporte. | Mitigado con varios métodos y con puntos de inscripción. El spike S1 mide cuántos adultos tienen pasaporte con chip y qué métodos son viables (§5.1). |
| El proveedor de prueba no acepta los certificados del pasaporte colombiano, o solo los de una generación | Sin método fuerte en Colombia hasta resolverlo: los puntos quedan bloqueados (`12`, compuerta G0). |
| Premiar la verificación vicia el consentimiento | Prohibido por la regla 7 de §4 y por la invariante I-14. |
| Un humano con varias identidades por métodos distintos | Aceptado y acotado. Solo el nivel 3 da acceso a puntos. |
| Un humano con varios documentos con chip | Aceptado. Acotado a dos o tres. |
| Documento prestado, alquilado o robado | Aceptado. El costo por cuenta es alto. |
| Atestación del sistema operativo burlada (`doc-app`, `dispositivo`) | Por eso son fuerza media y baja. Se vigila la tasa de anomalías. |
| Anillos de avales | Acotado por cupo. Sin puntos en nivel 2. |
| Punto presencial corrupto | Cupo, auditoría y retirada de la autorización. |
| Correlación por hora entre inscripción y alta | Mitigado con lotes y ruta oculta. Débil con pocos usuarios. |
| Dependencia de proveedores (prueba de documento, Apple, Google) | Cada uno aislado tras `HumanityProvider`. Ningún método es obligatorio. |
| Enumeración de números de documento en `doc-app` | Función seudoaleatoria oblivia con clave en un tercero. A validar en S1. |
| La descarga del camino de Merkle revela la hoja | Descarga por bloques con muchas hojas. |
| La insignia de nivel crea usuarios de primera y de segunda | Riesgo de producto. Decisión D-14. |
| El motor de riesgo penaliza a personas legítimas | Pausa reversible, revisión humana antes de suspender, tasa de falsos positivos publicada. Decisión D-16. |
| Una granja supera los retos de presencia | Por eso el puntaje nunca sube el nivel. El reto encarece la granja; no la hace imposible. |
| El catálogo de métodos por país queda desactualizado | Datos versionados con fuente citada; revisión en cada compuerta. |

## 14. Parámetros

| Parámetro | Valor inicial |
| :-- | :-- |
| Duración de época | 180 días |
| Ventana de renovación | 30 días antes del vencimiento |
| Gracia tras vencimiento | 30 días |
| Intervalo de lote de los árboles | 10 minutos |
| Vigencia de raíces | 24 horas |
| Repetición de un método para recuperar | 1 por época y método |
| Avales que puede dar un humano de nivel 3 | 3 por época |
| Avales necesarios | 2 |
| Cuentas secundarias por humano de nivel 3 | 5 |
| Edad mínima | 18 años (comprobable solo con `chip` y `cred`; en los demás métodos se declara) |
| Retraso de rotación de llave raíz | 72 horas |
| Caducidad de llave de dispositivo | 90 días, renovable |
| Puntaje inicial por nivel | 30, 50 y 70 |
| Tope de puntaje por nivel | 60, 80 y 100 |
| Plazo de respuesta a un reto | 72 horas |
| Frecuencia media de retos al azar | 7, 30 y 120 días según el tramo |
