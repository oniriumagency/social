# 10 - Seguridad, Privacidad y Cumplimiento

> Este documento no es asesoría legal. Señala las áreas que requieren revisión de un abogado antes de cada lanzamiento. El piloto es Colombia, así que el abogado debe ejercer allí. Las compuertas G2 y G4 exigen ese dictamen.

## 1. Reglas de desarrollo seguro

| Área | Regla |
| :-- | :-- |
| Criptografía | Solo a través de `social-crypto`. Prohibido implementar primitivas, modos o protocolos propios. Prohibida la criptografía en TypeScript: se usa el núcleo WASM. |
| Aleatoriedad | Solo el generador del sistema operativo, vía `social-crypto`. |
| Comparaciones de secretos | En tiempo constante. |
| Secretos en memoria | Tipos que se borran al liberarse (`zeroize`). No implementan `Debug` ni `Display`. |
| Entradas | Toda entrada externa se valida con un esquema y con límites de tamaño antes de usarse. |
| SQL | Solo consultas parametrizadas verificadas por `sqlx`. Prohibido concatenar. |
| `unsafe` en Rust | Prohibido sin ADR y revisión humana. |
| Errores | Sin `unwrap` ni `expect` en servicios ni en el nodo, salvo en pruebas. Los errores hacia fuera no revelan detalles internos. |
| Dependencias | Archivos de bloqueo obligatorios. Una dependencia nueva en `social-crypto`, `social-core`, el registro, el ledger o la red exige aprobación humana. |
| Deserialización | Con límites de profundidad y tamaño. Campos desconocidos rechazados en la envoltura de eventos. |

## 2. Seguridad del cliente web

1. **Política de seguridad de contenido estricta:** sin `unsafe-inline`, sin `unsafe-eval` (salvo lo mínimo que exija WASM), sin orígenes de terceros.
2. **Cero scripts, fuentes, analítica o píxeles de terceros.** Todo recurso se sirve desde el propio dominio.
3. **Llaves no extraíbles** en el navegador. La llave raíz no se guarda en web si el usuario tiene app de escritorio o móvil; la web usa una llave de dispositivo delegada.
4. **Sin HTML de usuario.** El texto se muestra como texto. Los enlaces se construyen, no se interpretan.
5. **Cabeceras:** HSTS, `X-Content-Type-Options`, `Referrer-Policy: no-referrer`, aislamiento de origen cruzado.
6. **Enlaces externos** con `rel="noopener noreferrer"` y aviso al salir.
7. **Sin vistas previas de enlaces generadas por el cliente**, porque revelan la IP del lector al sitio enlazado. Si se quieren vistas previas, las genera la pasarela.

## 3. Moderación

La red es 100% humana, pero los humanos también abusan.

| Capa | Mecanismo | Fase |
| :-- | :-- | :-- |
| Usuario | Silenciar y bloquear, en local y sincronizado entre sus dispositivos. | F1 |
| Reporte | Evento `report.create` hacia el equipo de moderación. | F1 |
| Operador | Acciones `mod.action`: ocultar, retirar, suspender cuenta. Son eventos públicos con código de motivo. | F1 |
| Límites | Frecuencia por cuenta (`06` §8). | F1 |
| Media | Lista de bloqueo por hash para material ilegal conocido. | F1 |
| Vigilancia por riesgo | Motor de riesgo sobre eventos públicos, más sensible con puntajes bajos (`05` §6.1). | F2 |
| Verificación adicional | Retos de presencia al azar y retos de método ante riesgo; el puntaje sube o baja según el resultado (`05` §6.2). | F2 |
| Apelación | Canal de apelación con plazo de respuesta. | F2 |
| Transparencia | Informe periódico de acciones de moderación. | F2 |
| Descentralizada | Cada pasarela e indexador aplica su política; el cliente elige. | F5 |

Puntos que requieren decisión y asesoría (`15` D-09):

- **Política de contenido:** qué se retira. Mínimo: lo ilegal en las jurisdicciones donde se opera.
- **Material de abuso sexual infantil:** obligación de detección y denuncia según la jurisdicción; en Colombia, a confirmar con el abogado, las leyes 679 de 2001 y 1336 de 2009 y el canal de denuncia nacional. El acceso a las listas de hashes oficiales exige acuerdos como organización.
- **Sanción máxima:** temporal por diseño (`05` §12).

## 4. Borrado y derechos sobre los datos

| Dato | Dónde está | Cómo se borra |
| :-- | :-- | :-- |
| Publicación | Pasarela, indexadores | `post.delete`; se elimina el contenido y queda la lápida. |
| Media | Copia ancla y nodos, cifrada | Se borra la clave de contenido; las réplicas quedan ilegibles y se limpian. |
| Cuenta | Pasarela | Evento de cierre; se borran perfil y publicaciones; la llave pública queda en lápidas. |
| Nullifiers y compromisos de identidad | Registro | Se conservan mientras sean necesarios para evitar duplicados: el de cuenta, durante su época; el de método, mientras exista la identidad. Están protegidos con HMAC, pero **se tratan como dato personal**: sirven para individualizar y comprobar unicidad, y cifrarlos no los vuelve anónimos (§6.1). El abogado valida esta conservación frente a una solicitud de supresión. |
| Datos del documento y biometría | En ningún servidor | No aplica. |
| Coordenadas | En ningún servidor | No aplica. |
| Asientos del ledger | Ledger | No se borran: obligación contable. No contienen identidad civil. |
| Datos del comercio | Ledger | Según la normativa mercantil y fiscal. |

- Las copias en manos de terceros que no cumplan el protocolo no se pueden borrar. Se advierte al usuario antes de publicar por primera vez.
- Una llave pública y una zona son datos personales seudónimos en muchas legislaciones. Se tratan como tales.
- Antes de F2 se realiza una evaluación de impacto en protección de datos.
- **Antes de admitir al primer usuario real** (alfa de F1 incluida) existen la política de tratamiento de datos, la autorización que acepta el usuario y un canal de consultas y reclamos con responsable y plazos. Es requisito de la compuerta G1.
- El procedimiento de borrado se documenta de forma que se pueda **explicar y demostrar a un tercero**, réplicas en nodos incluidas: qué clave se destruyó, cuándo y qué quedó ilegible.

## 5. Registros, métricas y analítica

1. **Prohibido registrar:** llaves privadas, secretos, frases de recuperación, pruebas ZK completas, coordenadas, contenido de documentos, IP junto a cualquier identificador de cuenta, billetera, nodo o nullifier.
2. **IP:** solo en memoria para limitar frecuencia, con hash y sal rotatoria. Los registros de acceso del balanceador se desactivan o se anonimizan.
3. **Métricas:** agregadas. Ninguna etiqueta de métrica contiene un identificador de usuario.
4. **Motor de riesgo:** solo lee eventos públicos y firmados de la pasarela. Tiene prohibido usar IP, huella del dispositivo, datos de otros servicios o de terceros. Su módulo no tiene acceso a la capa de red ni a otras bases de datos, y una prueba de arquitectura lo comprueba.
5. **Analítica de producto:** solo contadores agregados calculados en el servidor a partir de eventos públicos. Sin SDK de analítica en los clientes.
6. **Informes de fallos:** opcionales, activados por el usuario, sin datos de usuario, revisables antes de enviar.
7. **Retención:** registros operativos durante 14 días.
8. Hay una prueba automática que arranca el sistema, ejecuta un recorrido completo y falla si en los registros aparece una llave, una IP o una coordenada.

## 6. Mapa de riesgos legales

Cada fila necesita el dictamen de un abogado que ejerza en Colombia, el país del piloto. **La columna de referencias es un punto de partida escrito sin abogado: el dictamen la confirma o la corrige.** Al abrir otro país se repite el mapa para ese país.

| Tema | Por qué importa | Referencia en Colombia | Cuándo |
| :-- | :-- | :-- | :-- |
| Protección de datos | Datos seudónimos, zona, documentos de identidad y biometría tratados en el dispositivo. | Ley 1581 de 2012 y su reglamentación; autoridad: Superintendencia de Industria y Comercio (SIC). Registro Nacional de Bases de Datos si la entidad está obligada. | Política y canal antes de G1; dictamen antes de G2 |
| Tratamiento en el dispositivo | Si procesar el documento y el rostro solo en el teléfono del usuario convierte al proyecto en responsable de ese tratamiento. | Ley 1581, régimen de datos sensibles | Antes de G2 |
| Vínculo entre nivel 3 y puntos | Los puntos exigen nivel 3. Hay que descartar que se lea como un incentivo económico a cambio de datos de identidad (§6.1). | Ley 1581, principio de libertad | Antes de G2 y de G4 |
| Edad mínima | Obligaciones reforzadas si se admiten menores. Por defecto: 18 años o más. | Mayoría de edad a los 18; Ley 1098 de 2006 | Antes de G2 |
| Responsabilidad por contenido | Régimen de intermediarios, plazos de retirada, punto de contacto. | Sin un régimen general de exención para intermediarios conocido: lo fija el abogado. Leyes 679 de 2001 y 1336 de 2009 sobre explotación sexual de menores. | Mínimo en la alfa; completo antes de la beta pública |
| Transferencia internacional | Los servidores pueden estar fuera de Colombia. | Ley 1581, reglas de transferencia y transmisión | Antes de elegir región (D-13) |
| Operadores de nodos | Si alojar fragmentos cifrados genera responsabilidad para el usuario. | Sin norma específica conocida | Antes de G3 |
| Puntos | Si constituyen dinero electrónico, captación, instrumento prepago o un programa de fidelización regulado. | Estatuto del Consumidor (Ley 1480 de 2011) para promociones; Superintendencia Financiera si se parecen a un depósito | Antes de G4 |
| Liquidación a comercios | Licencias de pago, prevención de lavado, verificación de comercios. | Superintendencia Financiera | Antes del modelo B |
| Impuestos | Tratamiento fiscal de puntos y descuentos para usuarios y comercios. | DIAN | Antes de G4 |
| Token | Regulación de criptoactivos y valores. | Sin marco específico conocido; criterio de la Superintendencia Financiera | Antes de cualquier trabajo en F5 |
| Lectura de documentos oficiales | Si hay restricciones para leer o procesar el pasaporte y la cédula. | Cancillería y Registraduría Nacional del Estado Civil | Spike S1 |
| Decisiones automatizadas | El puntaje y la pausa automática pueden estar regulados. Exigir explicación, revisión humana y apelación. | Lineamientos de la SIC sobre tratamiento automatizado de datos | Antes de G2 |
| Cifrado y exportación | Controles de exportación de software criptográfico. | Depende también del país de la entidad (D-11) | Antes de publicar binarios |
| Tiendas de apps | Reglas sobre P2P, recompensas, compras digitales y verificación de identidad. | Condiciones de Apple y Google | Antes de F2 móvil |
| Licencias de datos geográficos | Atribución y condiciones de las fuentes del catálogo. | Condiciones de uso del DANE y de los portales de datos abiertos | Spike S3 |
| Sanciones internacionales | Si hay obligación de excluir países o personas. | Depende del país de la entidad (D-11) | Antes de G2 |

### 6.1 El precedente de Worldcoin en Colombia

La SIC ordenó el **cierre inmediato y definitivo** de la operación de World Foundation y Tools for Humanity en Colombia y el borrado de los datos sensibles recogidos, con la Resolución 78798 del 3 de octubre de 2025, confirmada en apelación por la Resolución 45710 del 18 de junio de 2026. Fuente: prensa nacional consultada el 2026-11-24; el abogado debe leer las resoluciones.

Es el caso más cercano a este proyecto: una prueba de humanidad única. Lo que la SIC reprochó, y lo que este diseño hace al respecto:

| Lo que reprochó la SIC | Cómo responde el diseño |
| :-- | :-- |
| Recoger un dato biométrico (el iris), que es dato sensible, sin autorización previa, expresa e informada. | Ningún dato biométrico ni del documento sale del dispositivo (I-1). Aun así, la app pide autorización clara antes de usar la cámara o leer el chip. |
| Ofrecer una **compensación económica** a cambio del dato, lo que condiciona la voluntad y vicia el consentimiento. | **Verificarse nunca se premia** (`05` §4, I-14). Los puntos pagan almacenamiento auditado. El vínculo entre nivel 3 y puntos lo revisa el abogado. |
| Operar sin políticas de tratamiento ajustadas a la ley colombiana y sin procedimientos eficaces de consultas y reclamos. | Política, autorización y canal existen antes del primer usuario real (§4, compuerta G1). |
| No informar de forma clara y suficiente sobre las finalidades. | Cada método dice en pantalla qué prueba, qué se procesa, dónde y qué llega al servidor (`05` §5). |
| Sostener que un dato cifrado y fragmentado ya no es personal. La SIC respondió que no es anónimo si el sistema sigue pudiendo individualizar y verificar unicidad. | Nullifiers y compromisos de identidad se tratan como dato personal (§4). Ningún texto público ni legal afirma que dejan de ser datos personales por estar cifrados. |
| No explicar cómo se borra de verdad un dato repartido en fragmentos. | Borrado por destrucción de clave, documentado y demostrable (§4). |
| Alegar que la ley colombiana no aplica a una entidad extranjera. La SIC respondió que aplica a quien trate datos en Colombia, tenga o no sede allí. | La decisión D-11 no cambia la ley aplicable. |

**Regla para los agentes:** ninguna tarea añade recogida de datos biométricos o de documentos en un servidor, ni recompensas por verificarse, ni textos que afirmen que un dato cifrado o seudónimo deja de ser dato personal. Cualquiera de las tres es motivo para detenerse y preguntar.

## 7. Construcción en público y divulgación responsable

**Reglas para transmisiones y repositorio público**

1. Los secretos viven en un gestor de secretos o en variables de entorno locales, nunca en el repositorio.
2. Ningún comando que imprima secretos se ejecuta en una terminal compartida. Los agentes tienen prohibido leer archivos `.env`.
3. Un escáner de secretos corre antes de cada commit y en CI.
4. Las vulnerabilidades no se discuten en incidencias públicas.
5. No se muestran en pantalla paneles de producción, datos de usuarios ni acuerdos comerciales.
6. Si un secreto se expone, se rota de inmediato y después se investiga.

**Divulgación responsable**

- Archivo `SECURITY.md` con un canal privado de reporte y una llave pública para cifrar.
- Compromiso de respuesta inicial en 72 horas.
- Divulgación coordinada: se publica tras el parche o a los 90 días.
- Programa de recompensas por fallos a partir de F2, con prioridad en registro y ledger.

## 8. Auditorías y revisiones

| Momento | Revisión |
| :-- | :-- |
| Cada PR | Lista de comprobación de seguridad en la plantilla; revisión humana obligatoria en zonas sensibles. |
| Fin de cada fase | Revisión interna contra `04`. |
| G2 | **Auditoría externa** del registro y de la integración ZK. |
| G3 | Revisión externa del nodo: superficie de red y actualizaciones. |
| G4 | **Auditoría externa** del ledger y del canje. |
| Continuo | Análisis de dependencias, fuzzing programado, escáner de secretos. |

**Zonas sensibles** (toda modificación exige revisión humana): `crates/social-crypto`, `crates/social-core`, `services/registry`, `services/ledger`, `crates/social-governor`, migraciones de base de datos, configuración de CI y despliegue, y las reglas de agentes.

## 9. Respuesta a incidentes

1. Manual de respuesta en `docs/runbooks/` desde F1: quién decide, cómo se rota una clave, cómo se pausa la ingesta, cómo se comunica.
2. Las claves de servicio (registro, vales, claves de contenido) viven en un KMS con rotación documentada y ensayada.
3. Copias de seguridad cifradas con restauración probada cada mes.
4. Interruptores de emergencia: pausar altas, pausar emisión, pausar canje. Cada uno con su prueba.
