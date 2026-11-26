# Spike S5: llaves

> Estado: **parcial**. Faltan las pruebas en dispositivos reales del responsable.

## 1. Pregunta

Según `docs/12` (F0-S5): soporte de la extensión PRF de WebAuthn por navegador y sistema;
llaves Ed25519 no extraíbles en navegadores; almacén seguro en Tauri por sistema; propuesta
final de derivación y copia de seguridad (`docs/05` §7, `docs/10` §2).

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

| Dispositivo y sistema | Navegador | Gestor de llaves | Código |
| :-- | :-- | :-- | :-- |
| | | | |

## 4. Hallazgo abierto: llaves no extraíbles frente a I-8

`docs/10` §2 y `docs/05` §7 piden llaves no extraíbles en web. En el navegador eso solo existe
como `CryptoKey` de WebCrypto: una llave generada dentro del núcleo WASM vive en memoria de
JavaScript y sí se puede copiar. I-8 prohíbe la criptografía en TypeScript. Hay que decidir
antes de F1-T18 si la web firma con WebCrypto a través de una capa mínima y revisada, o si
renuncia a llaves no extraíbles. Este informe propondrá opciones cuando estén los resultados.

## 5. Pendiente

- Resultados en dispositivos reales (§3.2).
- Almacén seguro de Tauri por sistema.
- Propuesta final de derivación y copia de seguridad.
