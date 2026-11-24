# 01 - Visión y Producto

> "Social" es el nombre en clave del proyecto. El piloto es en **Colombia** (`15` D-01).

## 1. Qué es el proyecto

Una red social de microblogging y conversación (tipo X o Threads) que une **conversación global y local** sobre **tres pilares irrenunciables**:

1. **Descentralización progresiva y verificable.** Todo el contenido es un evento firmado por su autor y verificable por cualquiera, de modo que ningún servidor es dueño de los datos y cualquier operador es reemplazable. El almacenamiento de réplicas reside cada vez más en nodos aportados por los usuarios. Las piezas que hoy necesitan un operador (registro de humanidad, índices, ledger) están identificadas, y cada una tiene una ruta de salida (`02` §3). No se promete "sin servidores" desde el día uno; se promete que nadie puede falsificar, secuestrar ni borrar en silencio lo que publica un usuario.
2. **100% humana, con niveles de confianza.** Crear una cuenta exige al menos un **método de verificación**, y cada cuenta muestra un nivel según lo difícil que es falsificarla. El nivel 3, **humano único**, se obtiene con un documento oficial firmado digitalmente por su emisor: el chip NFC de un pasaporte o cédula, leído una sola vez con cualquier teléfono, o una credencial digital firmada. Quien no tenga esa opción entra por métodos alternativos (documento por cámara dentro de una app atestada, cuenta en una institución, aval de humanos verificados, verificación presencial) y alcanza el nivel 2, **humano probable**. El medidor cuenta la fuerza de los métodos, no su cantidad, y es interno: la cuenta solo muestra el nivel. La biometría del dispositivo protege la llave y aporta el nivel básico. Las cuentas secundarias son de pago, con tope, y exigen nivel 3.
3. **Privacidad estricta.** Ni los datos biométricos, ni las imágenes o datos del documento, ni las coordenadas, ni las métricas internas del hardware salen jamás del dispositivo. En los servidores y en la red solo existen cuentas anónimas basadas en llaves. Ni siquiera el operador del registro puede relacionar un documento con una cuenta.

**Qué no es el producto (para evitar sobrepromesas):**

- No es anónimo frente a un adversario que observe toda la red.
- No garantiza unicidad por debajo del nivel 3: los niveles 1 y 2 significan "humano probable".
- No garantiza que una persona con dos documentos válidos no tenga dos cuentas.
- No convierte la zona de una publicación en prueba de presencia física.
- No es una criptomoneda. Los puntos de la Fase 4 son un programa de recompensas con reglas cerradas.

---

## 2. Ubicación por defecto y conversación en múltiples escalas

- **Zona enlazada por defecto.** Cada publicación lleva una zona geográfica para alimentar feeds cercanos desde el primer uso. La zona se elige una vez en la bienvenida (con sugerencia del dispositivo si el usuario da permiso) y queda fija hasta que el usuario la cambie.
- **Seis escalas.** El usuario explora lo que ocurre en su **barrio**, **localidad**, **ciudad**, **país**, **continente** o a nivel **global**, o consulta y publica en una ciudad o barrio concreto.
- **Privacidad espacial.**
  - Las coordenadas nunca salen del dispositivo. La conversión de ubicación a zona se hace en local con paquetes de catálogo descargados.
  - La precisión por defecto al publicar es **localidad** (o ciudad, si la ciudad no tiene localidades). Publicar a nivel de barrio es opcional.
  - Si una zona tiene menos cuentas activas que el umbral de anonimato, la publicación sube sola al nivel siguiente.
  - La ubicación en vivo está apagada por defecto. Así no queda rastro de movimientos.
- **Transparencia sobre el origen.** Cada publicación indica si se hizo desde la zona fija, en vivo o de forma remota. La red no puede verificarlo, y lo dice.

---

## 3. Experiencia de usuario

- **Leer es libre.** Cualquiera puede leer contenido público sin cuenta. Publicar, responder y reaccionar exige una cuenta de nivel 1 o superior.
- **Registro por el camino que cada uno tenga.** Con NFC, el usuario acerca su documento al teléfono una sola vez; puede ser un teléfono prestado o el de un comercio aliado. Sin NFC, elige otro método y puede subir de nivel más adelante sin perder su cuenta. Si empezó en web o escritorio, un código QR lo lleva al teléfono y de vuelta.
- **El formulario se adapta.** Detecta en local qué puede hacer el dispositivo y, según el país que emitió el documento, muestra solo los métodos que de verdad se pueden completar. Nunca pide leer un chip a quien tiene un documento sin chip.
- **Confianza que se gana.** Cada cuenta tiene un puntaje interno que sube al superar verificaciones ocasionales y baja al fallarlas o ignorarlas. Las cuentas de puntaje bajo reciben más verificaciones. Para una persona real es un toque de vez en cuando.
- **Nivel visible.** Cada cuenta muestra una insignia de nivel y el lector puede filtrar por nivel mínimo.
- **Nodo sin fricción ni sorpresas.** Al instalar la app de escritorio, el sistema detecta el tipo de equipo y su hardware (batería o corriente, RAM, disco libre, CPU, tipo de red) y aplica un **perfil recomendado seguro** (por ejemplo: solo con Wi-Fi o cable, con batería mayor al 60% o enchufado, con topes conservadores de disco, CPU y escritura diaria). El usuario lo acepta con un clic, lo ignora o ajusta cada límite. El objetivo es un **desgaste mínimo y acotado**, con cifras visibles; no existe el desgaste cero.
- **El móvil no es nodo.** Los teléfonos son clientes ligeros. Los sistemas operativos móviles no permiten servir datos en segundo plano de forma fiable.

---

## 4. Economía de puntos

Disponible a partir de la Fase 4, después de la prueba de humanidad y de una auditoría externa.

- **Qué se premia.** El almacenamiento **asignado por la red y auditado** (GB·hora con auditorías aprobadas). No se premia el tráfico servido ni el tiempo encendido autodeclarado, porque ambos se falsifican.
- **Emisión con presupuesto.** Cada día se reparte un presupuesto fijo de puntos entre los nodos que cumplieron, a prorrata. Hay un tope por humano. Así la deuda total es conocida de antemano.
- **Reglas que reducen el riesgo.** Los puntos no se transfieren entre usuarios, vencen, y solo se canjean en comercios aliados. Solo las identidades de nivel 3 tienen billetera.
- **Canje.** El comercio muestra un QR dinámico, el usuario lo escanea y aprueba desde su billetera, y el ledger registra la operación al instante.
- **Privacidad.** La billetera no se puede relacionar con la cuenta social.
- **Evolución.** Conectar el ledger a una blockchain o emitir un token es una decisión de la Fase 5, sujeta a dictamen legal y a tracción real.

---

## 5. Estrategia: código abierto, construcción en público y monetización

- **Transparencia y confianza.** Protocolo, cliente, nodo y mecanismos de privacidad son de código abierto. Construir en público acelera comunidad y auditoría. Límites: no se exponen vulnerabilidades sin parchear, secretos, datos privados ni acuerdos comerciales. Hay política de divulgación responsable (`10` §7).
- **Lanzamiento ciudad por ciudad.** Los feeds locales y la red de comercios solo tienen valor con densidad. Se abre una ciudad piloto, se llena, y se pasa a la siguiente. **El piloto es Colombia**, con Bogotá como ciudad piloto (`15` D-01). Allí el pasaporte lleva chip y la cédula no, así que la mayoría entrará por los niveles 1 y 2 y el nivel 3 dependerá del pasaporte (`05` §5.1).
- **Verificarse nunca se premia.** Los puntos pagan el almacenamiento aportado, no la identidad. Es una regla de privacidad y también legal (`10` §6.1).
- **Monetización sin vender datos.**
  - **Red de comercios aliados:** en el inicio, el comercio financia el descuento que otorga a cambio de visibilidad local y herramientas; más adelante, comisiones por liquidación si el dictamen legal lo permite.
  - **Cuentas secundarias de pago** y capacidades para marcas y comercios.
  - **Promociones locales transparentes**, siempre etiquetadas y segmentadas solo por zona, nunca por perfil personal.
- **Defensa.** La liquidez de comercios, la calidad del producto y la confianza acumulada pesan más que un repositorio privado.

---

## 6. Criterios de éxito por fase

| Fase | Señal de éxito |
| :-- | :-- |
| F1 Alfa cerrada | 200 invitados activos en una ciudad; el feed de localidad tiene publicaciones nuevas cada hora en horario diurno. |
| F2 Beta pública | Más del 80% de quienes inician el registro lo terminan por algún método; cero identidades duplicadas dentro de un mismo método en pruebas de ataque. |
| F3 Nodos | 100 nodos comunitarios; 99% de auditorías aprobadas; ningún reporte de equipo afectado. |
| F4 Comercios | 20 comercios activos; canje completado en menos de 5 segundos; ledger cuadrado a cero cada día. |
| F5 Descentralización | Al menos una pasarela y un indexador operados por terceros sirviendo tráfico real. |

Las cifras son puntos de partida y se revisan en cada compuerta de fase.
