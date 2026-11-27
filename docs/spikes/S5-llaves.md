# Spike S5: llaves

> Estado: **cerrado con Windows**. Android y Apple quedan pendientes (§3.3).

## 1. Pregunta

Según `docs/12` (F0-S5): soporte de la extensión PRF de WebAuthn por navegador y sistema;
llaves Ed25519 no extraíbles en navegadores; almacén seguro en Tauri por sistema; propuesta
final de derivación y copia de seguridad (`docs/05` §11, `docs/10` §2).

## 2. Método

Página estática de prueba (sin contenido de terceros, CSP que bloquea toda conexión de red),
servida en el propio equipo por un servidor Node mínimo que solo escucha en `127.0.0.1`. La
página usa WebCrypto y WebAuthn para medir capacidades del navegador; por decisión del
responsable (2026-11-26) **no entra al repositorio**, para no abrir una excepción a la regla de
lint de I-8. Se entrega aparte, con estas huellas SHA-256:

| Archivo | SHA-256 |
| :-- | :-- |
| `index.html` | `dc6ec3ef7f5c0a6b6398b75752d8ee608c61762e98a87f043e82bd48641b584b` |
| `probe.js` | `46388b2b35514b319b4994ab4999cddbab4ce30255fad749c0d1671d343205e5` |
| `servidor.mjs` | `07abd02bebab9c7fad4eddd12748e404dd3059873bee41062102a4ba4417b2ea` |

1. **Crear:** una llave de acceso con la extensión `prf` y una llave Ed25519 de WebCrypto con
   `extractable: false`, guardada en IndexedDB.
2. **Recargar** la página.
3. **Probar:** contexto seguro, autenticador de plataforma, `getClientCapabilities`, dos
   lecturas PRF con la misma sal (deben coincidir; la salida nunca se muestra ni se envía),
   y que la llave Ed25519 sobreviva a la recarga, firme, verifique y no se pueda exportar.
4. **Borrar:** quita los datos locales y avisa al gestor de llaves con
   `signalUnknownCredential` donde exista.

La página da un código como `S5v1 sc1.pk1.uv1.cap1.cprf1.prf1.det1.per1.ed1.nx1`
(`1` sí, `0` no, `-` no se pudo medir).

## 3. Resultados

### 3.1 Contenedor (Chromium sin interfaz, autenticador virtual)

`S5v1 sc1.pk1.uv1.cap1.cprf1.prf1.det1.per1.ed1.nx1`. Solo demuestra que la página funciona;
no dice nada de autenticadores reales.

### 3.2 Dispositivos reales

| Dispositivo y sistema | Navegador | Gestor de llaves | Código | Fecha |
| :-- | :-- | :-- | :-- | :-- |
| Computador del responsable, Windows | Google Chrome | Gestor de contraseñas de Google (pide su PIN al crear) | `S5v1 sc1.pk1.uv1.cap1.cprf1.prf1.det1.per1.ed1.nx1` | 2026-11-26 |
| Computador del responsable, Windows | Firefox | — (falló al crear, `UnknownError`) | `S5v1 sc1.pk1.uv1.cap1.cprf1.prf-.det-.per1.ed1.nx1` | 2026-11-26 |
| Computador del responsable, Windows | Microsoft Edge | Windows Hello | `S5v1 sc1.pk1.uv1.cap1.cprf1.prf1.det1.per1.ed1.nx1` (al crear no anunció PRF) | 2026-11-26 |

Lectura de estos resultados:

- **Ed25519 no exportable** funciona en los tres navegadores de Windows: se crea, sobrevive a
  recargar, firma y no se puede exportar.
- **PRF** funciona con el gestor de Google (Chrome) y con Windows Hello (Edge). En Windows
  Hello la creación no anuncia PRF, pero la lectura sí lo entrega y es estable: el cliente no
  debe fiarse del aviso al crear y debe comprobar PRF en la primera lectura.
- **Firefox en Windows** no creó la llave de acceso (`UnknownError`), así que PRF no se pudo
  medir (`prf-`). Hay que tener una vía sin PRF (secreto aleatorio en almacén seguro, como
  prevé `docs/05` §7).

Matiz importante para la recuperación: la llave de Chrome quedó en el gestor de contraseñas de
Google, que la **sincroniza** entre dispositivos; la de Edge quedó en Windows Hello, que la
guarda **solo en ese equipo**. Las dos entregan PRF, pero solo la sincronizada sirve como vía
de recuperación del secreto de identidad (`docs/05` §11).

### 3.3 Pendiente

- **Android:** la conexión por USB con Chrome no autorizó la depuración en el teléfono del
  responsable. Se medirá cuando haya un despliegue con HTTPS (decisión del responsable,
  2026-11-26).
- **Apple (macOS, iOS):** sin dispositivo disponible. Mismo momento.

Ninguno bloquea G0: la propuesta de §6 tiene vía sin PRF para cualquier navegador que no lo
ofrezca.

## 4. Almacén seguro en Tauri

Evaluación de escritorio, sin prueba física (se verifica en F1-T23, que debe compilar y
ejecutarse en los tres sistemas).

| Opción | Versión (2026-11-26) | Qué es | Valoración |
| :-- | :-- | :-- | :-- |
| `keyring-core` + almacenes nativos (`keyring` 4.x) | 4.2.0, 43 M de descargas | Credential Manager en Windows, Keychain en macOS, Secret Service (zbus) en Linux; Android e iOS con crates de almacén aparte | **Recomendada.** Usa el almacén del sistema, que es lo que pide `docs/05` §11. La 4.x separa API (`keyring-core`) y almacenes: se eligen solo los necesarios por destino |
| `tauri-plugin-stronghold` | 2.4.0 | Bóveda cifrada en archivo, protegida con contraseña | No es el almacén del sistema; exige gestionar otra contraseña. Descartada |
| `tauri-plugin-keyring-store`, `tauri-plugin-keychain` | 0.3.2 / 2.0.2 | Envoltorios de terceros con poco uso | Descartados: poca adopción; mejor usar `keyring-core` directamente desde Rust |

Riesgo en Linux: Secret Service necesita un servicio de llavero activo (GNOME Keyring,
KWallet). Sin él, la app debe avisar y no guardar la llave raíz en claro.

## 5. Hallazgo abierto: llaves no extraíbles frente a I-8

`docs/10` §2 y `docs/05` §11 piden llaves no extraíbles en web. En el navegador eso solo existe
como `CryptoKey` de WebCrypto: una llave generada dentro del núcleo WASM vive en memoria de
JavaScript y sí se puede copiar. I-8 prohíbe la criptografía en TypeScript. Los resultados de
§3.2 muestran que la vía WebCrypto funciona en los tres navegadores medidos. Las opciones y la
recomendación están en `docs/15`, D-19; se decide antes de F1-T18.

## 6. Propuesta de derivación y copia de seguridad

Propuesta para confirmar con ADR antes de F1-T18 y F2-T04. Toda la criptografía va en
`social-crypto` (núcleo WASM en web); JavaScript solo pasa bytes.

1. **Secreto de identidad con PRF.** La passkey se evalúa con una sal fija del protocolo
   (por ejemplo `social/v1/identity`), igual en todos los dispositivos. El secreto es
   `HKDF-SHA256(salida PRF, info = "social/v1/semaphore-identity")`. La app comprueba PRF en la
   primera lectura, no al crear (§3.2, Edge).
2. **Solo cuenta como vía de recuperación si la passkey está sincronizada.** Si el gestor la
   guarda solo en el equipo (Windows Hello), la frase de recuperación es obligatoria.
3. **Sin PRF** (Firefox en Windows, o el usuario no quiere passkey): secreto aleatorio de 32
   bytes en el almacén seguro (§4) y frase de recuperación de 24 palabras obligatoria.
4. **Llave raíz:** se genera en `social-crypto` y vive en el almacén del sistema (escritorio y
   móvil). En web, la llave raíz no se guarda si hay app; la web usa una llave de dispositivo
   delegada (`docs/10` §2), sujeta al hallazgo de §5.
5. **Copia cifrada de la llave raíz:** cifrado autenticado con una clave
   `HKDF(secreto de identidad, info = "social/v1/root-backup")`. Dónde se guarda la copia
   queda para el ADR.

## 7. Pendiente

- Android y Apple (§3.3).
- Prueba física del almacén seguro (F1-T23).
- Decidir el hallazgo de §5 y confirmar §6 con ADR.
