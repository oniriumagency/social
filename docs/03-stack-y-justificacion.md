# 03 - Stack Tecnológico y Justificación

> Las versiones exactas se fijan en los archivos de bloqueo (`Cargo.lock`, `pnpm-lock.yaml`). Los agentes deben consultar la documentación vigente de cada herramienta antes de usarla y no fiarse de la memoria.

## 1. Tabla de decisiones técnicas

| Capa | Elección | Justificación | Cambio frente a v1 |
| :-- | :-- | :-- | :-- |
| **Núcleo de protocolo** | **Rust**, un solo crate `social-core` compilado a nativo y a **WASM** (`wasm-bindgen`) | Una única implementación de firma, canonicalización y validación para servicios, nodo, escritorio y web. Evita divergencias entre lenguajes. | Nuevo |
| **Criptografía** | Crates de uso amplio y revisión pública de RustCrypto y dalek detrás de `social-crypto`: Ed25519, X25519, XChaCha20-Poly1305, BLAKE3, SHA-256, HKDF, RSA ciego (RFC 9474) | Ninguna primitiva propia. Un solo punto de importación facilita la auditoría. | Nuevo |
| **Web** | **TypeScript + Next.js (React) + Tailwind CSS** | Agilidad para feeds, selector de escala y vistas de canje. Solo interfaz: toda la criptografía pasa por el núcleo WASM. | Se mantiene |
| **Escritorio y nodo** | **Rust + Tauri 2**, interfaz con **Vite + React** | Binario ligero con acceso nativo a hardware. Tauri necesita una app estática, por eso el escritorio no usa Next.js. | Se separa de Next.js |
| **Código de interfaz compartido** | Paquetes `packages/ui` (componentes) y `packages/app` (pantallas y estado) | Web y escritorio son cáscaras finas sobre el mismo código. | Nuevo |
| **Móvil** | Cliente ligero con Tauri 2 móvil (a confirmar en F2). Inscripción NFC mediante la app del proveedor de prueba de documento | El móvil no es nodo. Leer el chip exige código nativo. | El móvil deja de ser nodo |
| **Servicios** | **Rust**: `axum`, `tokio`, `sqlx`, `tracing`, `utoipa` (OpenAPI) | Mismo lenguaje y mismos tipos que el núcleo. Consultas SQL verificadas en compilación. | Nuevo |
| **Base de datos** | **PostgreSQL 16 o superior** | ACID para ledger y registro; índices compuestos y particionado para feeds. | Se mantiene |
| **Caché y colas** | Ninguna al inicio. **Redis o Valkey** solo cuando una medición lo pida | Menos piezas que operar. | Redis sale del MVP |
| **Almacenamiento de objetos** | Compatible con S3 (MinIO en local) | Copia ancla de media. Conviene un proveedor sin costo de salida de datos. | Nuevo |
| **Detección de hardware y energía** | `sysinfo`, un crate de batería mantenido (candidato `starship-battery`, verificar en S4), `netdev`, más **APIs de cada sistema operativo** para red medida y temperatura | Ningún crate cubre red medida ni temperatura en todos los sistemas. Matriz en `08` §2. | Se corrige la lista y la promesa |
| **Zonas geográficas** | **Catálogo de zonas administrativas** versionado como fuente primaria + **H3** para coberturas precomputadas, paquetes regionales y barrio de respaldo | H3 no sustituye a las divisiones administrativas. La resolución es local. | Se invierte el papel de H3 |
| **Prueba de humanidad** | **Varios métodos detrás de la interfaz `HumanityProvider`, con niveles de confianza** (`05` §3–4): prueba ZK de documento con chip NFC (candidatos: Self, ZKPassport; se elige en S1), credenciales digitales firmadas, documento por cámara en app atestada (Play Integrity, App Attest), prueba ZK de cuenta institucional, avales y verificación presencial. **Semaphore v4** para pertenencia anónima y nullifiers por ámbito | Protocolos existentes y de código abierto, en lugar de circuitos propios. Sus auditorías y licencias se comprueban en el spike S1. La interfaz permite cambiar de proveedor. | Sustituye a "OCR/MRZ + Noir/Halo2" |
| **Protección de llaves** | **WebAuthn/Passkeys** (extensión PRF) y almacén seguro del sistema operativo | La biometría local desbloquea la llave; no prueba unicidad. | Cambia de papel |
| **Vales de pago anónimos** | Firmas RSA a ciegas (RFC 9474) | Desvincula el pago de la cuenta secundaria. | Nuevo |
| **Red entre nodos** | **rust-libp2p**: QUIC y TCP, Noise, identify, autonat, relay v2, DCUtR, Kademlia, petición-respuesta | Atraviesa NAT y transfiere fragmentos. El relevo no da anonimato. Bitswap no se usa. Gossipsub queda para F5. | Se quita Bitswap y el anonimato por relevo |
| **Verificación de fragmentos** | **BLAKE3** con verificación por rangos | Permite auditar un fragmento sin descargar todo el archivo. | Nuevo |
| **Privacidad de red** | **HTTP Oblivious (RFC 9458)** o Tor para operaciones sensibles | Oculta la IP frente al servicio en inscripción, alta y canje. | Nuevo |
| **Cómputo en sandbox** | Aplazado a F5. Candidato: **Wasmtime** | Sin demanda ni verificación definidas. | Sale del MVP. Se elimina wgpu |
| **Ledger** | **PostgreSQL** en servicio y base de datos aislados | Doble entrada, enteros, aislamiento serializable, cadena de hashes. | Se mantiene, se aísla |
| **Observabilidad** | `tracing` + OpenTelemetry + métricas Prometheus | Con reglas de privacidad de registros (`10` §5). | Nuevo |
| **Pruebas** | `cargo test`, `proptest`, `cargo-fuzz`, Vitest, Playwright, k6 | Detalle en `14`. | Nuevo |
| **Monorepo y tareas** | Workspace de Cargo + pnpm + Turborepo; órdenes únicas en el `package.json` raíz | Las mismas órdenes en Windows, macOS y Linux, sin scripts de shell. | Nuevo |
| **Integración continua** | GitHub Actions en Linux y Windows (macOS para builds de Tauri) | El desarrollo ocurre en Windows; CI debe cubrirlo. | Nuevo |
| **Licencias** | Por defecto: **AGPL-3.0** para servicios, nodo y apps; **Apache-2.0** para núcleo, especificación y SDK | Protege contra forks cerrados del servicio y facilita la adopción del protocolo. Pendiente de confirmación (`15` D-04). | Se concreta |

---

## 2. Lo que se descartó y por qué

| Descartado | Motivo |
| :-- | :-- |
| Biometría local como prueba de unicidad | No prueba unicidad (A-01). |
| OCR/MRZ sin atestación de la app | No verificable (A-02). Se admite solo dentro de una app atestada por el sistema operativo, con fuerza media. |
| Circuitos ZK propios, Halo2 | Riesgo y costo de auditoría (A-20). |
| Jitter de ubicación | Inútil y contraproducente (A-09). |
| Gossipsub por zona para feeds | No escala y filtra ubicación (A-18). |
| Bitswap | No está en el repositorio principal de rust-libp2p (a confirmar en el spike S2); hay una alternativa más simple (A-28). |
| Móvil como nodo | Los sistemas operativos no lo permiten (A-17). |
| GPU y cómputo remunerado | Sin demanda ni verificación (A-21). |
| Electron | Consumo de memoria; Tauri cubre la necesidad. |
| Construir sobre Nostr o AT Protocol | El requisito de humanidad obliga a pasarelas cerradas a cuentas no atestadas, lo que anula la interoperabilidad; y el formato propio es pequeño. Se toma su idea central: eventos firmados y operadores reemplazables (ADR-007). |
| Next.js dentro de Tauri | Exige exportación estática y limita la web (A-32). |

---

## 3. Hoja de ruta por fases

Detalle, tareas y compuertas en `12-plan-de-desarrollo.md`.

| Fase | Contenido | Compuerta de salida |
| :-- | :-- | :-- |
| **F0 Cimientos** | Monorepo, CI, núcleo de protocolo con vectores dorados, y cinco spikes de riesgo (humanidad, red, zonas, gobernador, llaves). | G0: spikes revisados y decisiones ejecutivas D-01 a D-04 resueltas. |
| **F1 Core social (alfa cerrada)** | Pasarela, feeds por zona, hilos, media, moderación básica, web y escritorio como clientes. Acceso por invitación. Sin puntos. | G1: alfa con invitados reales y revisión de seguridad interna. |
| **F2 Humanidad (beta pública)** | Registro, métodos de verificación con niveles de confianza, grupos Semaphore, atestaciones por época, recuperación, cuentas secundarias con vales ciegos, cliente móvil. | G2: auditoría externa del registro aprobada. |
| **F3 Nodo y almacenamiento** | Gobernador de recursos, red entre nodos, réplicas cifradas, auditorías de almacenamiento, puntos de prueba no canjeables. | G3: red de prueba estable bajo simulación de abandono de nodos. |
| **F4 Ledger y comercios** | Ledger, billetera por humano, emisión por presupuesto, canje por QR, panel de comercios, piloto en una ciudad. | G4: dictamen legal y auditoría externa del ledger aprobados. |
| **F5 Descentralización progresiva** | Operadores de terceros, registro con firma umbral, codificación de borrado, retirada de la copia ancla, evaluación de token. | Por hitos. |

**Por qué cambió el orden original:** el plan anterior entregaba nodo y puntos en el primer MVP y la defensa anti-bots en el segundo. Puntos con valor sin prueba de humanidad son un regalo a las granjas de nodos falsos.
