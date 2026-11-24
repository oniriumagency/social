# 06 - Protocolo y Modelo de Datos

Especificación normativa de la versión 1 del protocolo. Las palabras DEBE, NO DEBE y PUEDE tienen sentido de requisito. La implementación de referencia es `crates/social-core`. Si este documento y el código difieren, se detiene el trabajo y se resuelve con un ADR.

## 1. Convenciones

- Bytes en hexadecimal en minúsculas, sin prefijo.
- Tiempo en segundos Unix (entero).
- Texto en UTF-8, normalizado a NFC por el cliente antes de firmar.
- Números: solo enteros en el rango ±(2⁵³ − 1). Los decimales y `null` están prohibidos en cualquier evento.
- Hash: SHA-256 para identificadores de evento. BLAKE3 para contenido binario.
- Firma: Ed25519 con verificación estricta: se rechazan firmas no canónicas (`s` no reducido, `R` no canónico) y llaves públicas de orden pequeño o con componente de torsión (orden mixto). Lo comprueban los vectores de Wycheproof en `crates/social-crypto`.

## 2. Llaves

| Llave | Algoritmo | Uso |
| :-- | :-- | :-- |
| Raíz de cuenta | Ed25519 | Identifica la cuenta. Firma delegaciones, perfil y rotación. |
| Dispositivo | Ed25519 | Firma los eventos del día a día. |
| Registro | Ed25519 (umbral en F5) | Firma atestaciones. |
| Billetera | Ed25519 | Firma intenciones de canje. |
| Nodo | Ed25519 (identificador de par libp2p) | Identidad en la red de nodos. |

La identidad pública de una cuenta es la llave pública raíz. Para mostrarla se usa Bech32 con prefijo `soc1`.

## 3. Evento

```json
{
  "v": 1,
  "id": "<sha256 hex>",
  "account": "<llave pública raíz, 32 bytes hex>",
  "signer": "<llave pública que firma, 32 bytes hex>",
  "created_at": 1790000000,
  "kind": "post.create",
  "tags": [["t", "vecinos"], ["p", "<llave pública mencionada>"]],
  "content": {
    "text": "Hola barrio #vecinos",
    "lang": "es",
    "zone": { "id": "z_3k9dq7m2x4pa", "mode": "home" }
  },
  "sig": "<firma Ed25519, 64 bytes hex>"
}
```

`tags` contiene solo referencias indexables: `t` (etiqueta, en minúsculas, de 1 a 64 caracteres, sin espacios) y `p` (cuenta mencionada, llave pública en hexadecimal). Cada etiqueta es una lista de exactamente dos cadenas. Máximo 16. Todo lo demás va en `content`.

`kind` son dos o más segmentos `[a-z][a-z0-9_]*` separados por puntos, de 64 bytes como máximo. `created_at` no es negativo. `content` es siempre un objeto. Los campos hexadecimales (`id`, `account`, `signer`, `sig`) tienen la longitud exacta y van en minúsculas.

**Cálculo**

1. `payload = JCS([v, account, signer, created_at, kind, tags, content])`, con JCS según RFC 8785.
2. `id = SHA-256(payload)`.
3. `sig = Ed25519.sign(llave_privada_de_signer, bytes(id))`.

**Validación.** Un receptor DEBE rechazar el evento si falla cualquiera de estos puntos, en este orden:

1. Tamaño total mayor a 16 KiB.
2. Esquema de `kind` inválido, campos desconocidos en la envoltura, claves repetidas, decimales o `null`.
3. `id` distinto del calculado.
4. `sig` inválida para `signer`.
5. `signer` distinto de `account` y sin `device.grant` vigente y no revocado que cubra `kind` en `created_at`.
6. `account` sin atestación vigente en `created_at` (excepto los propios eventos de atestación), o con un nivel insuficiente para ese `kind` o ese contenido.
7. Solo en la ingesta: `created_at` a más de 10 minutos del reloj del servidor.
8. Límites del tipo (§4) o de frecuencia (§8).

Los puntos 1 a 6 son **validación universal**: cliente, pasarela, indexador y nodo los aplican por igual. El punto 7 solo lo aplica quien ingesta.

## 4. Tipos de evento

| `kind` | Firma | Contenido | Notas |
| :-- | :-- | :-- | :-- |
| `account.attest` | Registro | `account`, `tier`, `level`, `epoch`, `expires_at`, `issuer` | `tier`: `alpha`, `base`, `secondary`. `level`: 1, 2 o 3 (`05` §4). Nunca incluye los métodos usados. |
| `account.presence` | Registro | `account`, `challenge`, `passed_at` | Resultado de una verificación adicional superada (`05` §6). |
| `auth.request` | Dispositivo | `purpose`, `nonce` | Efímero: autentica una lectura privada. No se guarda ni se reenvía. `created_at` a menos de 60 segundos. |
| `account.profile` | Raíz o dispositivo | `name` (≤ 50), `bio` (≤ 300), `avatar` (cid) | Reemplazable: vale el más reciente. |
| `account.rotate` | Raíz antigua, o registro | `new_account`, `effective_at` | Retraso de 72 horas. |
| `device.grant` | Raíz | `device`, `caps` (lista de `kind`), `expires_at`, `label` | Caducidad máxima de 90 días. |
| `device.revoke` | Raíz | `device` | Efecto inmediato. |
| `post.create` | Dispositivo | `text` (≤ 500 grafemas), `lang`, `zone` {`id`, `mode`}, `media` (≤ 4), `reply_to`, `root`, `quote` | `mode`: `home`, `live`, `remote`. |
| `post.delete` | Dispositivo | `target` | Lápida. Solo el autor. |
| `post.repost` | Dispositivo | `target` | |
| `reaction.set` | Dispositivo | `target`, `value` (`like` o vacío) | Reemplazable por par cuenta-objetivo. |
| `follow.set` | Dispositivo | `target_account`, `value` (booleano) | Reemplazable. |
| `report.create` | Dispositivo | `target`, `reason`, `note` (≤ 500) | No se propaga a otros usuarios. |
| `mod.action` | Operador | `target`, `action`, `reason_code` | Público y auditable. |

Reglas:

- **Esquema estricto.** En el contenido de un tipo conocido no se admiten campos que no estén en la tabla (`E_UNKNOWN_FIELD`): así ningún cliente puede colar datos no previstos, por ejemplo coordenadas (I-2). Los textos van en NFC (`E_TEXT_NOT_NFC`). Formatos: zona `z_` más 12 caracteres base32 de Crockford en minúsculas; `cid` `b3_` más 52 caracteres base32 (RFC 4648) en minúsculas; `lang` BCP 47; `reason`, `reason_code` y `purpose` son códigos `[a-z][a-z0-9_]*` de hasta 32 bytes. `post.create` lleva `reply_to` y `root` juntos o ninguno, y texto o imágenes. `device.grant` lleva de 1 a 32 `caps` sin repetir y un `label` de hasta 50 caracteres.
- **Límites del tipo** (longitudes, número de imágenes, 90 días de delegación, 72 horas de rotación) se comprueban junto con el esquema y fallan con `E_LIMIT_EXCEEDED`.
- **Quién firma (paso 5).** «Raíz» exige `signer == account`; «Dispositivo» exige un dispositivo delegado distinto de la raíz; «Registro» y «Operador» exigen `signer == account` y una llave publicada del registro o de un operador, y no necesitan atestación de cuenta. `account.rotate` firmado por el registro es la rotación de recuperación. Un tipo desconocido lo puede firmar la raíz o un dispositivo cuya delegación lo cubra. Errores: `E_SIGNER_ROLE`, `E_DELEGATION_MISSING`, `E_DELEGATION_REVOKED` (evento con `created_at` igual o posterior a la revocación), `E_DELEGATION_EXPIRED`, `E_DELEGATION_SCOPE`.
- **Atestación y nivel (paso 6).** `E_ATTESTATION_MISSING` si no hay atestación vigente en `created_at`; `E_LEVEL_INSUFFICIENT` si el nivel no alcanza para el contenido (nivel 1: sin imágenes ni avatar, `05` §4).
- Los vectores dorados de `spec/vectors/validation.json` incluyen un caso por cada regla de rechazo.
- Los esquemas JSON de `spec/schemas/` se generan desde la misma tabla que usa la validación (`pnpm spec:gen`); lo que JSON Schema no puede expresar (grafemas, NFC, reglas entre campos) va en las claves `x-max-graphemes`, `x-nfc` y `x-rules`.
- No hay edición en la versión 1. Corregir es borrar y volver a publicar.
- Un evento de tipo desconocido se guarda y se reenvía si pasa la validación universal, pero no se proyecta.
- Añadir un tipo o un campo exige ADR, actualizar este documento, los esquemas en `spec/` y los vectores dorados.

## 5. Borrado

1. `post.delete` marca el evento como borrado. Las pasarelas dejan de servirlo y eliminan el contenido, conservando solo `id` y la lápida.
2. La media asociada se retira borrando su clave de contenido (`08` §5).
3. Los nodos y terceros que cumplan el protocolo DEBEN respetar las lápidas. No se puede garantizar el borrado en copias de terceros que no cumplan; la interfaz lo dice con claridad.

## 6. API de la pasarela (v1)

Sin sesiones ni cookies. Toda escritura es un evento firmado. Las lecturas de contenido público no exigen autenticación.

| Método y ruta | Función |
| :-- | :-- |
| `POST /v1/events` | Ingesta de un evento. Idempotente por `id`. |
| `GET /v1/events/{id}` | Un evento. |
| `GET /v1/events/{id}/thread?cursor=` | Hilo: ancestros y respuestas. |
| `GET /v1/feed?zone={id}&cursor=&limit=` | Feed de una zona, de cualquier nivel. |
| `GET /v1/feed/following?account={pk}&cursor=` | Feed de cuentas seguidas. |
| `GET /v1/accounts/{pk}` | Perfil, atestación y nivel. Nunca el puntaje. |
| `POST /v1/accounts/status` | Estado privado de la propia cuenta: tramo de puntaje, retos pendientes, pausa y motivo por categoría. El cuerpo es un evento `auth.request` firmado. |
| `GET /v1/accounts/{pk}/events?cursor=` | Publicaciones de una cuenta. |
| `GET /v1/zones/{id}` | Zona, ancestros e intervalo de actividad. |
| `GET /v1/zones/search?q=&parent=` | Búsqueda por nombre. |
| `GET /v1/zones/packs/{catalog}/{celda_h3}.bin` | Paquete de catálogo (estático, por CDN). |
| `POST /v1/media` | Inicia una subida; devuelve URL prefirmada. |
| `POST /v1/media/{cid}/complete` | Cierra y valida la subida. |
| `GET /v1/stream` (WebSocket) | Suscripción a zonas e hilos. |
| `GET /v1/info` | Versión de protocolo, llaves del registro, límites, versión de catálogo. |

- **Paginación:** por cursor opaco que codifica `(created_at, id)`. Nunca por desplazamiento.
- **Errores:** `{ "error": { "code": "E_SIG_INVALID", "message": "..." } }` con códigos estables enumerados en `social-core` (`ErrorCode`). Envoltura (pasos 1 a 4): `E_EVENT_TOO_LARGE`, `E_JSON_INVALID`, `E_JSON_DUPLICATE_KEY`, `E_JSON_FLOAT`, `E_JSON_NULL`, `E_JSON_INT_RANGE`, `E_SCHEMA`, `E_UNKNOWN_FIELD`, `E_VERSION_UNSUPPORTED`, `E_PUBKEY_INVALID`, `E_ID_MISMATCH`, `E_SIG_INVALID`.
- **Respuestas de feed:** eventos completos con firma, para que el cliente verifique.
- **No existe ningún endpoint que reciba coordenadas.**
- Una cuenta en pausa recibe `E_STEP_UP_REQUIRED` al publicar, con el identificador del reto pendiente. Puede seguir leyendo.
- La especificación OpenAPI se genera desde el código y se comprueba en CI.

## 7. Media

1. El cliente recodifica la imagen, **elimina todos los metadatos (EXIF, XMP, IPTC)**, calcula `cid = "b3_" + base32(BLAKE3(bytes))` y sube.
2. La pasarela verifica tipo real, tamaño, dimensiones y ausencia de metadatos; rechaza si falla. Vuelve a comprobar el `cid`.
3. Límites iniciales: imágenes JPEG, PNG o WebP de hasta 5 MB y 4096 px por lado. Vídeo fuera de la versión 1.
4. En F3 la media se cifra y se replica (`08`).

## 8. Límites de frecuencia

Valores iniciales, por cuenta y ajustables por configuración:

| Acción | Límite |
| :-- | :-- |
| Publicaciones y respuestas | 30 por hora, 200 por día |
| Reacciones | 300 por hora |
| Seguimientos | 100 por hora |
| Reportes | 20 por día |
| Cambio de zona fija | 1 cada 7 días |
| Subidas de media | 40 por día |

La tabla aplica a los niveles 2 y 3 con puntaje medio o alto; con puntaje bajo, los límites se reducen a la mitad (`05` §6). Las cuentas de **nivel 1** tienen límites reducidos: 5 publicaciones por hora, 20 por día y sin media.

Por IP se limita solo en memoria, con un hash con sal que rota cada hora. Nunca se escribe en disco.

## 9. Modelo de datos de la pasarela

Esquema orientativo. Las migraciones reales viven en `services/gateway/migrations`.

```sql
CREATE TABLE events (
  id          bytea PRIMARY KEY,           -- 32 bytes
  account     bytea NOT NULL,
  signer      bytea NOT NULL,
  kind        text  NOT NULL,
  created_at  bigint NOT NULL,
  received_at timestamptz NOT NULL DEFAULT now(),
  raw         jsonb NOT NULL,              -- evento completo y firmado
  deleted_at  timestamptz
);
CREATE INDEX events_account_idx ON events (account, created_at DESC, id);

CREATE TABLE accounts (
  account        bytea PRIMARY KEY,
  tier           text NOT NULL,
  level          smallint NOT NULL,     -- 1, 2 o 3
  attested_until bigint NOT NULL,
  status         text NOT NULL DEFAULT 'active',  -- active | paused | suspended
  trust_score    smallint NOT NULL,     -- 0 a 100, interno; nunca se expone a terceros
  profile_event  bytea
);

CREATE TABLE challenges (
  id          uuid PRIMARY KEY,
  account     bytea NOT NULL REFERENCES accounts(account),
  kind        text NOT NULL,             -- presence | method
  reason      text NOT NULL,             -- random | risk
  issued_at   bigint NOT NULL,
  due_at      bigint NOT NULL,
  outcome     text                       -- passed | ignored | failed
);

CREATE TABLE device_grants (
  account    bytea NOT NULL,
  device     bytea NOT NULL,
  caps       text[] NOT NULL,
  expires_at bigint NOT NULL,
  revoked_at bigint,
  PRIMARY KEY (account, device)
);

CREATE TABLE posts (
  id         bytea PRIMARY KEY REFERENCES events(id),
  account    bytea NOT NULL,
  zone_id    text NOT NULL,
  zone_mode  text NOT NULL,
  reply_to   bytea,
  root       bytea,
  lang       text,
  created_at bigint NOT NULL
);
CREATE INDEX posts_root_idx ON posts (root, created_at, id);

-- Una fila por cada nivel de la zona: barrio, localidad, ciudad, país, continente y global.
CREATE TABLE post_zones (
  zone_id    text   NOT NULL,
  created_at bigint NOT NULL,
  post_id    bytea  NOT NULL,
  PRIMARY KEY (zone_id, created_at, post_id)
) PARTITION BY RANGE (created_at);         -- particiones mensuales

CREATE TABLE follows (
  follower bytea NOT NULL, followee bytea NOT NULL, since bigint NOT NULL,
  PRIMARY KEY (follower, followee)
);

CREATE TABLE zones (
  id text PRIMARY KEY, level smallint NOT NULL, parent_id text,
  names jsonb NOT NULL, country text, catalog_version int NOT NULL
);

CREATE TABLE zone_stats (
  zone_id text PRIMARY KEY, active_accounts_30d int NOT NULL, updated_at timestamptz NOT NULL
);
```

**Decisiones de rendimiento**

- El feed de zona es un recorrido de índice sobre `post_zones`, sin ordenar en memoria. Costo: seis filas por publicación a cambio de lecturas constantes.
- Solo las publicaciones raíz entran en `post_zones`; las respuestas se leen por `root`.
- El feed de seguidos se resuelve en lectura mientras el número de seguidos sea moderado. Si la medición lo pide, se pasa a reparto en escritura con ADR.
- Los niveles país, continente y global reciben demasiado volumen para un orden cronológico puro. Desde F1 se sirven con una muestra por puntuación simple (reacciones y respuestas con decaimiento temporal), calculada por lotes. La fórmula es pública.
- No hay tabla que guarde IP.

## 10. Versionado y compatibilidad

- El campo `v` identifica la versión de la envoltura. Un cambio incompatible sube `v` y exige ADR.
- Los clientes anuncian la versión que entienden y las pasarelas publican las que aceptan en `/v1/info`.
- Los vectores dorados en `spec/vectors/` son la definición ejecutable. Todo cambio de protocolo empieza por ellos.
