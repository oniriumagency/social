# 15 - Decisiones Ejecutivas Pendientes

Decisiones que corresponden al responsable humano. Cada una tiene un **valor por defecto** con el que la documentación ya está escrita, para que el trabajo no se bloquee. Cuando se decida, se anota aquí la respuesta y la fecha, y se actualizan los documentos afectados.

| ID | Decisión | Por defecto | Bloquea | Estado |
| :-- | :-- | :-- | :-- | :-- |
| D-01 | País y ciudad del piloto | Colombia. Ciudad: Bogotá | S3 (la ciudad) | **Decidida el 2026-11-24** (país y ciudad) |
| D-02 | Cómo se prueba la humanidad | Varios métodos con niveles de confianza | — | **Decidida el 2026-11-23** |
| D-03 | Modelo económico de los puntos | A: descuento financiado por el comercio | F4-T00 | Pendiente |
| D-04 | Licencias | AGPL-3.0 para servicios, nodo y apps; Apache-2.0 para núcleo, especificación y SDK | F0-T03 | Pendiente |
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

## D-03. Modelo económico de los puntos

Ver `09` §7. El modelo A no mueve dinero del proyecto y tiene poco riesgo regulatorio. El modelo B exige fondos y posiblemente licencia. La decisión final se toma con el dictamen legal de F4-T00.

## D-04. Licencias

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

---

## Registro de decisiones tomadas

| Fecha | ID | Decisión | Documentos actualizados |
| :-- | :-- | :-- | :-- |
| 2026-11-23 | D-02 | Varios métodos de verificación con niveles de confianza. Solo el nivel 3 significa humano único. | `01`, `02`, `03`, `04`, `05`, `06`, `07`, `09`, `12`, `14`, `16`, ADR-016, `CLAUDE.md`, `ANTIGRAVITY.md` |
| 2026-11-23 | — | El registro se adapta al dispositivo y al país del documento: nunca ofrece un método imposible. | `05` §5, `12`, `14`, ADR-017 |
| 2026-11-23 | — | Puntaje de confianza dinámico: sube al superar verificaciones, baja al fallarlas o ignorarlas. Las cuentas de puntaje bajo se vigilan más y reciben más verificaciones al azar. | `05` §6, `06`, `10`, `12`, `14`, ADR-018 |
| 2026-11-24 | D-01 | El piloto es Colombia. Ciudad por defecto: Bogotá, pendiente de confirmar. Verificarse nunca se premia. | `00`, `01`, `04`, `05` §5.1, `07` §2.1, `09`, `10` §6, `12`, `14`, `16`, ADR-019, `CLAUDE.md`, `ANTIGRAVITY.md` |
| 2026-11-24 | D-01 | Ciudad piloto confirmada: Bogotá. | `01`, `12`, `15`, ADR-019, `PROGRESO.md` |
| 2026-11-24 | D-17 | Los eventos solo son válidos en su forma canónica JCS exacta (`E_JSON_NOT_CANONICAL`). | `06` §1 y §3, `15`, ADR-020, `spec/vectors/envelope.json`, `PROGRESO.md` |
