# 15 - Decisiones Ejecutivas Pendientes

Decisiones que corresponden al responsable humano. Cada una tiene un **valor por defecto** con el que la documentación ya está escrita, para que el trabajo no se bloquee. Cuando se decida, se anota aquí la respuesta y la fecha, y se actualizan los documentos afectados.

| ID | Decisión | Por defecto | Bloquea | Estado |
| :-- | :-- | :-- | :-- | :-- |
| D-01 | País y ciudad del piloto | Colombia. Ciudad: Bogotá | S3 (la ciudad) | **Decidida el 2026-11-24** (país y ciudad) |
| D-02 | Cómo se prueba la humanidad | Varios métodos con niveles de confianza | — | **Decidida el 2026-11-23** |
| D-03 | Modelo económico de los puntos | A: descuento financiado por el comercio | F4-T00 | **Decidida el 2026-11-24** (A; se ratifica con el dictamen de F4-T00) |
| D-04 | Licencias | AGPL-3.0 para servicios, nodo y apps; Apache-2.0 para núcleo, especificación y SDK | F0-T03 | **Decidida el 2026-11-24** |
| D-05 | Edad mínima | 18 años | F2 | Pendiente |
| D-06 | Aceptar que la sanción máxima es temporal | Sí | F2 | Pendiente |
| D-07 | Revelar la nacionalidad en la inscripción | No | F2 | Pendiente |
| D-08 | Tope y precio de cuentas secundarias | 5 cuentas; precio por definir | F2-T15 | Pendiente |
| D-09 | Política de contenido | Mínimo legal más acoso y spam | G1 | Pendiente |
| D-10 | Nombre y marca definitivos | "Social" como nombre en clave | Lanzamiento público | Pendiente |
| D-11 | Entidad legal y jurisdicción | Sin definir. La ley colombiana de datos aplica aunque la entidad esté fuera (`10` §6.1) | G1 con usuarios reales | Pendiente |
| D-12 | Presupuesto para auditorías externas y asesoría legal | Sin definir. El abogado debe ejercer en Colombia | G1, G2, G4 | Pendiente |
| D-13 | Proveedor de infraestructura | Contenedores, PostgreSQL gestionado y almacenamiento compatible con S3 sin costo de salida | F1-T27 | Pendiente |
| D-14 | Insignia de nivel visible y filtro por nivel | Visible, con filtro | F2-T10 | Pendiente |
| D-15 | Privilegios por nivel; en particular, si el nivel 2 puede ganar puntos | Solo nivel 3 gana puntos | F2-T10, F4 | Pendiente |
| D-16 | Política de retos: frecuencia, plazos y efecto en el puntaje | Valores de `05` §6.2 | F2-T24 | Pendiente de calibrar |
| D-17 | Exigir que los eventos viajen en su forma canónica exacta | Forma canónica obligatoria | — | **Decidida el 2026-11-24** |
| D-18 | Efecto de una revocación sobre eventos fechados antes de ella | La revocación vale desde su `created_at` (`06` §4) | F3 | Pendiente; propuesta del responsable el 2026-11-24 |
| D-19 | Llaves no extraíbles en web frente a I-8 (sin criptografía en TypeScript), incluida la llave raíz creada en web | La web firma con el núcleo WASM; la llave no es extraíble solo en escritorio y móvil | F1-T18 | Pendiente; hallazgo del spike S5 |

---

## D-01. País y ciudad del piloto — DECIDIDA

**Decisión del responsable (2026-11-24):** el piloto es **Colombia** (ADR-019).

**Ciudad (decidida el 2026-11-24):** **Bogotá**. Es la ciudad del spike S3 y del catálogo de zonas v0.

**Qué implica** (detalle en `05` §5.1, `07` §2.1 y `10` §6):

- **Humanidad:** el pasaporte colombiano lleva chip y es la vía al nivel 3. La cédula, que es el documento que todo adulto tiene, entra por cámara dentro de la app y da nivel 2 como máximo. La mayoría empezará en los niveles 1 y 2.
- **Zonas:** barrio, localidad o comuna, y municipio, con datos del DANE y de los portales de datos abiertos de cada ciudad.
- **Legal:** Ley 1581 de 2012 de protección de datos, con la SIC como autoridad. La SIC cerró Worldcoin en Colombia por recoger biometría a cambio de una compensación; de ahí sale la regla de no premiar nunca la verificación.
- **Comercios:** los importes se manejan en pesos colombianos.

**Lo que sigue abierto y resuelve el spike S1:** cuántos adultos tienen pasaporte con chip; si el proveedor de prueba acepta los certificados del pasaporte colombiano, incluido el modelo de 2026; si la cédula digital tiene algún elemento verificable por terceros; y qué entidades sirven para el método `cuenta`.

## D-02. Cómo se prueba la humanidad — DECIDIDA

**Decisión del responsable (2026-11-23):** muchos dispositivos no tienen NFC, así que debe haber varias alternativas de registro, y un medidor interno de confianza según los métodos que el usuario haya añadido.

**Cómo quedó diseñado** (`05` §3–4, ADR-016):

- Siete métodos con fuerza fuerte, media o baja.
- Tres niveles. El nivel lo decide la fuerza del mejor método o combinación, no la cantidad.
- El medidor es interno; la cuenta solo muestra el nivel.
- Solo el nivel 3 significa humano único y da acceso a puntos.
- El chip se puede leer con un teléfono ajeno o en un punto de inscripción, así que no exige tener NFC propio.

**Consecuencia aceptada:** los niveles 1 y 2 son "humano probable". La promesa pública debe decirlo así.

## D-14. Insignia de nivel

Mostrar el nivel ayuda al lector a calibrar la confianza y motiva a subir de nivel. El riesgo es crear usuarios de primera y de segunda. Alternativa: no mostrarlo y usarlo solo para límites internos y peso en los feeds.

## D-15. Privilegios por nivel

La tabla de `05` §4 es el punto de partida. La pregunta de fondo es si el nivel 2 puede ganar puntos con un tope reducido. Abrirlo amplía la base de nodos remunerados; el riesgo es que las granjas ataquen el método medio más barato.

**En Colombia esta decisión pesa más**, porque el nivel 3 depende del pasaporte y la cédula solo llega al nivel 2. Si S1 muestra que pocos adultos tienen pasaporte con chip, quedarán pocos nodos remunerados. Recomendación: empezar solo con nivel 3, y revisar con la cifra de S1 y con los datos de los puntos de prueba de F3.

## D-03. Modelo económico de los puntos — DECIDIDA

**Decisión del responsable (2026-11-24):** modelo **A**, descuento financiado por el comercio. Cuando haya aliados comerciales, el canje en comercios será la ruta principal.

**Mientras no haya aliados:** la app dice que el nodo gana puntos y muestra el saldo, pero no anuncia al público en qué se pueden canjear. Como mucho se ofrecen ventajas digitales dentro de la propia app (por ejemplo, un emblema) que no le cuesten dinero al proyecto. Nada de esto adelanta fases: los puntos siguen sujetos a las compuertas G2, G3 y G4 y solo los gana el nivel 3 (I-14), y verificarse nunca da ventajas.

**Pendiente:** el dictamen legal de F4-T00 ratifica el modelo antes de abrir el canje. El modelo B (el proyecto paga al comercio) solo se reabre con dictamen favorable y ADR.

Ver `09` §7. El modelo A no mueve dinero del proyecto y tiene poco riesgo regulatorio. El modelo B exige fondos y posiblemente licencia.

## D-04. Licencias — DECIDIDA

**Decisión (2026-11-24):** AGPL-3.0-only en servicios, nodo y apps; Apache-2.0 en núcleo, especificación, SDK y herramientas. El reparto por carpeta está en `LICENSE` y cada paquete lo declara con su identificador SPDX. Los datos geográficos conservan su licencia (ODbL para OpenStreetMap, ver `docs/spikes/S3-zonas.md`).

Opciones que se consideraron:


- **AGPL-3.0** en servicios, nodo y apps: quien opere una versión modificada debe publicar sus cambios. Protege frente a forks cerrados.
- **Apache-2.0** en núcleo, especificación y SDK: facilita que terceros implementen el protocolo.
- Alternativa: todo Apache-2.0 o MIT, que maximiza la adopción y permite forks cerrados.

Debe decidirse antes del primer commit público.

## D-05. Edad mínima

18 años simplifica las obligaciones legales y la prueba ZK lo soporta. En Colombia coincide con la mayoría de edad y con la expedición de la cédula de ciudadanía. Bajarla exige análisis legal específico.

## D-06. Sanción máxima temporal

Ver `05` §12. Es consecuencia directa del pilar de privacidad. La alternativa de expulsión permanente exige guardar la relación documento-cuenta, lo que contradice "sin vínculo reversible con la identidad civil".

## D-07. Nacionalidad

No revelarla maximiza la privacidad. Revelarla puede ser obligatorio por sanciones internacionales o útil para limitar el lanzamiento a un país. Depende del dictamen legal.

## D-08. Cuentas secundarias

Tope sugerido de 5. El precio debe ser lo bastante alto para desincentivar la manipulación y lo bastante bajo para marcas y proyectos legítimos.

## D-09. Política de contenido

Qué se retira y con qué proceso. Debe existir antes de admitir invitados reales.

## D-16. Política de retos

Los valores de `05` §6.2 (frecuencias de 7, 30 y 120 días; plazo de 72 horas; +5, −5 y −15 puntos) son puntos de partida razonables, sin datos reales detrás. Se calibran en la beta: si los retos molestan a usuarios legítimos se espacian, y si no frenan a las granjas se intensifican en el tramo bajo.

## D-17. Forma canónica en el cable — DECIDIDA

**Decisión del responsable (2026-11-24):** se exige la forma canónica en el cable. Un evento cuyos bytes no son exactamente su JCS se rechaza con `E_JSON_NOT_CANONICAL` (ADR-020 punto 11, `06` §1 y §3). En sus palabras: tiene más sentido que las aplicaciones de terceros que quieran usar la red se acojan a sus normas.

**Contexto de la decisión.** Hoy un receptor acepta un evento con cualquier espaciado u orden de claves, porque el identificador se calcula sobre la forma canónica. Eso hace que el límite de 16 KiB dependa de cómo se serializó. Opciones: rechazar el evento si sus bytes no son exactamente su JCS (recomendada en la revisión de F0-T06: el límite queda inequívoco y nadie puede alterar el formato de un evento firmado), o medir el límite sobre la forma canónica. Cualquiera de las dos es una regla de protocolo: se añade a ADR-020, `06` §3 y los vectores.

## D-18. Revocación y eventos fechados antes

Hoy una revocación de dispositivo solo afecta a los eventos con `created_at` igual o posterior a ella, y `created_at` lo pone quien firma. El único freno a un evento fechado antes de la revocación es el reloj de la ingesta (paso 7), que nodos, indexadores y clientes no aplican. Hay que decidir antes de F3, cuando los nodos reenvían eventos sin pasar por la pasarela, cómo se trata ese caso (riesgo anotado en `04` §6).

**Ejemplo.** A alguien le roban el teléfono el lunes y revoca ese dispositivo el jueves. El viernes, quien tiene el teléfono firma una publicación y le pone fecha del martes. Como el martes la delegación seguía vigente, un nodo que reciba ese evento lo acepta. La pasarela lo rechaza por el reloj de la ingesta, pero un nodo que lo reenvía en F3 no tiene ese freno.

**Propuesta del responsable (2026-11-24).** Cuando la persona reporta el robo, indica desde cuándo perdió el teléfono (el lunes, en el ejemplo). La app le muestra toda la actividad firmada por ese dispositivo desde esa fecha hasta el reporte, y le deja decidir: borrar todo, borrar algunas publicaciones o conservarlas.

**Lo que esa pantalla resuelve y lo que no.** Resuelve lo que ya se publicó entre el lunes y el jueves: la persona lo ve y lo borra con `post.delete`. No resuelve el ejemplo del viernes: un evento firmado después del reporte con fecha anterior aparece cuando la persona ya revisó su actividad, y sigue siendo válido para los nodos. Para cerrarlo hace falta, además, una regla del protocolo. Opciones:

- **A. Solo la pantalla de revisión.** Simple, pero deja abierto el caso del viernes.
- **B. Revocación con fecha «desde».** `device.revoke` lleva la fecha que la persona indica al reportar (el lunes). Todo evento de ese dispositivo con `created_at` igual o posterior a esa fecha deja de ser válido, en la pasarela y en cualquier nodo, porque la regla solo depende de datos firmados. Lo que la persona quiera conservar de ese intervalo, la app lo vuelve a publicar desde su teléfono nuevo, marcado como «publicado originalmente el …». Cierra el caso del viernes; el costo es que lo conservado pierde sus respuestas y reacciones, porque cambia su identificador.
- **C. Ventana de llegada.** Se acepta un evento fechado antes de la revocación solo si llega pocas horas después de ella. Sirve en la pasarela, pero un nodo no sabe cuándo llegó un evento a otro nodo, así que no se puede aplicar igual en toda la red.

**Recomendación:** la pantalla de revisión del responsable más la opción B. La pantalla es la forma de decidir qué conservar y la fecha «desde» hace que esa decisión valga en toda la red. Cambia el protocolo (`06` §4, ADR, esquemas y vectores), así que se implementa con su tarea antes de F3, no ahora.

## D-19. Llaves no extraíbles en web frente a I-8

`docs/10` §2 y `docs/05` §11 piden que en el navegador la llave sea no extraíble. Eso solo existe como `CryptoKey` de WebCrypto: una llave creada en el núcleo WASM vive en la memoria de la página y un script malicioso podría copiarla. Pero I-8 prohíbe la criptografía en TypeScript. El spike S5 midió que Chrome, Edge y Firefox en Windows crean, conservan y usan una llave Ed25519 de WebCrypto que no se puede exportar (`spikes/S5-llaves.md` §3.2); en otros sistemas no se ha medido. Hay que decidir antes de F1-T18.

**La llave raíz también está en juego.** F1 es primero web: F1-T19 crea la cuenta y la frase de recuperación en el navegador antes de que exista la app de escritorio (F1-T23). Esa llave raíz nace en web y tiene que poder copiarse al menos una vez, para la frase o la copia cifrada. Cada opción debe decir qué pasa con ella.

- **A. Excepción acotada a I-8 en TypeScript.** Un único módulo de `packages/core`, mínimo y revisado, crea, guarda y usa con WebCrypto la llave de dispositivo (Ed25519, no extraíble). Todo lo demás sigue en WASM. Llave raíz: se crea en WASM, se respalda (frase o copia) y después no se guarda en el navegador; solo queda la llave de dispositivo. Cambia la invariante y exige ADR, además de una regla para navegadores sin Ed25519 en WebCrypto.
- **B. WebCrypto llamado desde `social-crypto` compilado a WASM.** La revisión del spike hizo un prototipo fuera del repositorio (Rust con `web-sys`, ejecutado en Node 22): crea una llave Ed25519 no extraíble, firma, verifica, y la exportación de la privada se rechaza; la llave privada no entra en la memoria de WASM, Rust solo guarda el asa. La regla «sin criptografía en TypeScript» y su lint quedan como están. A cambio, `social-crypto` gana dependencias (`web-sys`, `js-sys`, `wasm-bindgen-futures`), que es zona sensible, y en web firma el navegador, no el crate auditado: exige ADR. Llave raíz: igual que en A. Falta probarlo en navegadores y con IndexedDB, y la misma regla para navegadores sin Ed25519.
- **C. Sin llave no extraíble en web.** La llave de dispositivo web se crea en WASM con caducidad corta. Cumple I-8 tal cual, pero un fallo de XSS puede copiar la llave y usarla hasta que caduque o se revoque. Llave raíz: igual que la de dispositivo, extraíble mientras viva en la web. Obliga a cambiar `docs/10` §2 punto 3, `docs/05` §11 y la fila «XSS y robo de llaves» de `docs/04`.
- **D. La web no firma.** Firma la app (escritorio o móvil) y la web solo lee o pide la firma por QR. Lo más seguro, pero la web queda muy limitada y F1 ya no podría ser primero web. Llave raíz: nunca existe en web. Obliga a cambiar los mismos documentos que C y el orden de F1.

**Recomendación:** B si se confirma en navegadores; si no, A. Las dos dan lo que el modelo de amenazas ya da por hecho (`04`, fila «XSS y robo de llaves»). B además deja la criptografía en `social-crypto`, que es el espíritu de I-8. Cualquiera de las dos cambia una invariante o una zona sensible, así que lo decide el responsable con ADR.

---

## Registro de decisiones tomadas

| Fecha | ID | Decisión | Documentos actualizados |
| :-- | :-- | :-- | :-- |
| 2026-11-23 | D-02 | Varios métodos de verificación con niveles de confianza. Solo el nivel 3 significa humano único. | `01`, `02`, `03`, `04`, `05`, `06`, `07`, `09`, `12`, `14`, `16`, ADR-016, `CLAUDE.md`, `ANTIGRAVITY.md` |
| 2026-11-23 | — | El registro se adapta al dispositivo y al país del documento: nunca ofrece un método imposible. | `05` §5, `12`, `14`, ADR-017 |
| 2026-11-23 | — | Puntaje de confianza dinámico: sube al superar verificaciones, baja al fallarlas o ignorarlas. Las cuentas de puntaje bajo se vigilan más y reciben más verificaciones al azar. | `05` §6, `06`, `10`, `12`, `14`, ADR-018 |
| 2026-11-24 | D-01 | El piloto es Colombia. Ciudad por defecto: Bogotá, pendiente de confirmar. Verificarse nunca se premia. | `00`, `01`, `04`, `05` §5.1, `07` §2.1, `09`, `10` §6, `12`, `14`, `16`, ADR-019, `CLAUDE.md`, `ANTIGRAVITY.md` |
| 2026-11-24 | D-01 | Ciudad piloto confirmada: Bogotá. | `01`, `12`, `15`, ADR-019, `PROGRESO.md` |
| 2026-11-24 | D-04 | AGPL-3.0-only para servicios, nodo y apps; Apache-2.0 para núcleo, especificación, SDK y herramientas. | `03`, `15`, ADR-011, `LICENSE`, `README.md`, `PROGRESO.md` |
| 2026-11-24 | D-03 | Modelo A: descuento financiado por el comercio. Sin aliados, solo se muestra el saldo y ventajas digitales sin costo dentro de la app; se ratifica con el dictamen de F4-T00. | `09` §7, `15`, `PROGRESO.md` |
| 2026-11-24 | D-17 | Los eventos solo son válidos en su forma canónica JCS exacta (`E_JSON_NOT_CANONICAL`). | `06` §1 y §3, `15`, ADR-020, `spec/vectors/envelope.json`, `PROGRESO.md` |
