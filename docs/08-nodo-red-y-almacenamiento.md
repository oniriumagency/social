# 08 - Nodo, Red y Almacenamiento

Corresponde a la Fase 3. Nada de este documento se implementa antes de la compuerta G2, salvo los spikes S2 y S4.

## 1. Qué es un nodo

Un proceso que corre dentro de la app de escritorio (Tauri) o como servicio en un servidor. Guarda fragmentos cifrados que le asigna el coordinador, responde auditorías y sirve fragmentos a otros nodos y a clientes de escritorio.

- Un teléfono **no** es nodo. Un navegador **no** es nodo.
- Un nodo tiene su propia identidad (llave de par). No se relaciona con ninguna cuenta social.
- Para cobrar puntos, el nodo se vincula a una billetera con un mensaje firmado por la billetera (F4).

## 2. Gobernador de recursos

`crates/social-governor`. **Es la única puerta por la que el nodo usa disco, CPU y red.** Ningún trabajo en segundo plano arranca sin un permiso del gobernador, y todo trabajo debe poder cancelarse.

### 2.1 Señales por sistema operativo

| Señal | Windows | macOS | Linux |
| :-- | :-- | :-- | :-- |
| Batería y carga | API de energía del sistema, o crate de batería | IOKit, o crate de batería | `/sys/class/power_supply`, o crate de batería |
| Red medida | Costo de conexión de la API de red de Windows | `NWPathMonitor` (`isExpensive`, `isConstrained`) | Propiedad `Metered` de NetworkManager por D-Bus |
| Temperatura | Sin API fiable | `ProcessInfo.thermalState` | `/sys/class/thermal` |
| Uso de CPU y RAM | `sysinfo` | `sysinfo` | `sysinfo` |
| Disco libre | `sysinfo` | `sysinfo` | `sysinfo` |
| Actividad del usuario | Tiempo desde la última entrada | Tiempo de inactividad del sistema | Depende del entorno gráfico |

**Regla de respaldo:** si una señal no está disponible o es dudosa, el gobernador asume el caso más conservador. Sin dato de red medida, se trata como medida salvo que sea Ethernet. Sin dato de temperatura, se usa la carga de CPU sostenida como aproximación.

La cobertura real de esta tabla se confirma en el spike S4. El resultado se guarda en `docs/spikes/S4-gobernador.md` y esta tabla se actualiza.

### 2.2 Estados

```
            condiciones OK durante 60 s
   PAUSADO ─────────────────────────────▶ ACTIVO
      ▲                                     │
      │ violación dura (≤ 5 s)              │ presión (≤ 5 s)
      │                                     ▼
      └──────────── violación dura ───── REDUCIDO
                                            │ sin presión durante 60 s
                                            └──────────▶ ACTIVO
```

- **Violación dura** (pasa a PAUSADO): batería por debajo del umbral y sin corriente, red medida, temperatura crítica, disco libre por debajo de la reserva, o pausa manual.
- **Presión** (pasa a REDUCIDO): el usuario está usando el equipo de forma intensa, temperatura elevada, o CPU del sistema alta.
- **Reacción:** 5 segundos o menos desde que el sistema operativo informa la señal.
- **Histéresis:** se reanuda solo tras 60 segundos seguidos de condiciones correctas.

### 2.3 Perfiles por defecto

| Parámetro | Portátil | Escritorio o servidor |
| :-- | :-- | :-- |
| Red | Wi-Fi o cable, no medida | Wi-Fi o cable, no medida |
| Energía | Enchufado, o batería mayor al 60% | Continuo |
| Cuota de disco | 2 a 5 GB | 10 a 20 GB |
| Reserva de disco libre | El mayor entre 15% y 10 GB | El mayor entre 10% y 10 GB |
| CPU | Prioridad mínima, tope del 10% | Prioridad baja, tope del 15 al 25% |
| Escritura diaria | 2 GB | 10 GB |
| Ancho de banda | 1 Mbps de subida | 5 Mbps de subida |
| GPU | No se usa | No se usa |

- El usuario acepta el perfil con un clic o ajusta cada valor. Los límites del usuario nunca se superan.
- La cuota efectiva nunca supera lo que deja libre la reserva, aunque el usuario pida más.
- El presupuesto de escritura diaria limita el desgaste de la unidad SSD. La interfaz muestra los GB escritos.
- Los procesos del nodo usan la clase de prioridad más baja que ofrezca cada sistema operativo.

## 3. Red entre nodos

`crates/social-net`, sobre rust-libp2p.

| Función | Protocolo |
| :-- | :-- |
| Transporte | QUIC; TCP con Noise y Yamux como respaldo |
| Descubrimiento | Lista de arranque del coordinador; Kademlia entre nodos |
| NAT | AutoNAT para detectar; relay v2 y DCUtR para atravesar |
| Transferencia | Petición-respuesta con verificación BLAKE3 por rangos |
| Difusión | No se usa hasta F5 |

- El atravesamiento de NAT no siempre funciona. Una parte del tráfico irá por relevos que opera el proyecto, y ese ancho de banda es un costo. El spike S2 mide la tasa de éxito real.
- **El relevo no da anonimato.** La IP de un nodo es visible para sus pares. Quien no quiera exponerla no debe operar un nodo.
- Todo mensaje entrante tiene tamaño máximo, plazo y límite por par. Un par que envía datos inválidos se penaliza y, si reincide, se bloquea.
- Los navegadores no hablan con los nodos; obtienen la media de la pasarela. Los clientes de escritorio intentan primero con los nodos y recurren a la pasarela.

## 4. Durabilidad y réplicas

La disponibilidad de un dato con `r` réplicas en nodos que están en línea una fracción `p` del tiempo es `1 − (1 − p)^r`.

| Tipo de nodo | `p` | `r` = 3 | `r` = 10 |
| :-- | :-- | :-- | :-- |
| Portátil doméstico | 0,3 | 65,7% | 97,2% |
| Escritorio encendido de día | 0,6 | 93,6% | 99,99% |
| Servidor | 0,99 | 99,9999% | — |

Conclusión: la red comunitaria sola no garantiza los datos en sus primeras etapas.

**Política**

1. **Copia ancla.** Toda la media tiene siempre una copia en almacenamiento de objetos operado por el proyecto. Es la fuente de verdad de durabilidad.
2. **Réplicas comunitarias.** Objetivo de 3 réplicas por fragmento, repartidas en nodos de redes distintas.
3. **Reparación.** Cuando un nodo falla auditorías o desaparece, el coordinador reasigna sus fragmentos desde la copia ancla.
4. **Retirada del ancla.** Solo en F5, y solo si la disponibilidad medida de las réplicas comunitarias supera el objetivo durante 90 días. Entonces se introduce codificación de borrado.

## 5. Formato de almacenamiento y cifrado

1. **Clave de contenido.** Cada archivo recibe una clave aleatoria de 256 bits.
2. **Cifrado.** XChaCha20-Poly1305 por bloques de 256 KiB, con el número de bloque autenticado.
3. **Direccionamiento.** El texto cifrado se identifica por su hash BLAKE3. La estructura en árbol de BLAKE3 permite probar un bloque sin tener el archivo entero.
4. **Servicio de claves.** Guarda `cid → clave`. Entrega la clave a cualquier lector mientras el contenido sea público y esté vigente.
5. **Borrado y retirada.** Borrar la clave inutiliza todas las réplicas en todos los nodos a la vez. Los nodos eliminan los fragmentos en su siguiente limpieza.

Consecuencias:

- El operador de un nodo no puede leer lo que guarda, lo que reduce su exposición legal.
- El servicio de claves es una pieza central que puede negar el acceso. Es una concesión consciente para poder cumplir con borrados y retiradas legales. Su descentralización (custodia umbral) es trabajo de F5.

La elección exacta entre el árbol nativo de BLAKE3 y un manifiesto propio se decide en la primera tarea de F3 con un ADR.

## 6. Auditorías de almacenamiento y crédito

**Qué cuenta:** solo los fragmentos que el coordinador asignó a ese nodo. Un nodo no puede declarar datos por su cuenta.

**Desafío**

1. El coordinador elige un nodo, un fragmento asignado y un bloque al azar, y envía un reto con un valor aleatorio.
2. El nodo responde con el bloque y su prueba de pertenencia.
3. El plazo es de 10 segundos más la latencia medida de ese nodo.
4. El coordinador verifica contra el hash conocido.

**Reglas**

- Los desafíos llegan en momentos impredecibles, varias veces al día por nodo.
- Un nodo PAUSADO por el gobernador responde "en pausa". No se penaliza, pero ese tiempo no genera crédito.
- Tres fallos seguidos: se reasignan sus fragmentos y se anula el crédito de la ventana.
- El crédito se mide en **GB·hora auditados**. Es la única entrada del cálculo de puntos (`09` §3).
- No se premia el tráfico servido: es trivial descargarse a uno mismo.
- No se premia el tiempo encendido autodeclarado.

**Ataques conocidos y respuesta**

| Ataque | Respuesta |
| :-- | :-- |
| Un operador con muchos nodos falsos | El tope de puntos es por billetera, y hay una billetera por humano. |
| Guardar una sola copia y responder desde varios nodos | Asignación por coordinador a redes distintas; plazo corto; tope por humano. |
| Pedir el bloque a otro nodo al recibir el reto | Plazo ajustado a la latencia. Riesgo residual aceptado en F3 y F4. |
| Colusión con el coordinador | El coordinador es del proyecto. Registro auditable de asignaciones y auditorías. |

## 7. Seguridad del nodo

1. El nodo no ejecuta código recibido de la red. Nunca.
2. Escribe solo dentro de su directorio de datos.
3. Las actualizaciones van firmadas y la app verifica la firma antes de instalar.
4. Los binarios se firman con certificados de código en Windows y macOS (`11` §6).
5. El nodo se puede pausar y desinstalar por completo con una sola acción, y al desinstalar se borra todo lo que almacenó.
6. La interfaz muestra en todo momento: estado del gobernador, GB almacenados, GB escritos hoy, ancho de banda usado y motivo de la última pausa.

## 8. Fuera de alcance hasta F5

- Cómputo distribuido y GPU.
- Difusión de eventos por gossip.
- Réplica del archivo de eventos de texto en nodos.
- Codificación de borrado.
