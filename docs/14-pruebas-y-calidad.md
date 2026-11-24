# 14 - Pruebas y Calidad

## 1. Principios

1. Todo criterio de aceptación tiene una prueba automática.
2. Las pruebas se escriben antes o junto con el código, nunca después del PR.
3. Una prueba que falla es información. No se silencia.
4. Lo crítico (protocolo, identidad, ledger, gobernador, privacidad) se prueba con propiedades e invariantes, no solo con ejemplos.
5. No se simula (mock) la criptografía ni la base de datos en pruebas de integración.

## 2. Niveles

| Nivel | Herramienta | Qué cubre | Dónde corre |
| :-- | :-- | :-- | :-- |
| Unitarias Rust | `cargo nextest` | Lógica pura | Local y CI |
| Propiedades | `proptest` | Invariantes sobre entradas generadas | Local y CI |
| Vectores dorados | Ejecutor propio | Compatibilidad de protocolo entre nativo, WASM en Node y WASM en navegador | Local y CI |
| Fuzzing | `cargo-fuzz` | Análisis de eventos, mensajes de red, paquetes de zona | CI nocturno |
| Integración de servicios | `sqlx` con PostgreSQL real y efímero | Ingesta, feeds, registro, ledger | Local y CI |
| Unitarias TypeScript | Vitest | Estado y utilidades de interfaz | Local y CI |
| Componentes | Vitest con Testing Library | Componentes y accesibilidad | Local y CI |
| Extremo a extremo | Playwright | Flujos completos en navegador | CI |
| Carga | k6 | Latencias y cuellos de botella | Bajo demanda y antes de cada compuerta |
| Simulación de red | Contenedores | Abandono de nodos, cortes, pares maliciosos | F3 |
| Ataque | Suite propia | Intentos de romper unicidad, ledger y privacidad | F2, F4 |

## 3. Vectores dorados

Viven en `spec/vectors/` como JSON. Son la definición ejecutable del protocolo.

```json
{
  "name": "post.create mínimo válido",
  "input": { "v": 1, "account": "...", "signer": "...", "created_at": 1790000000,
             "kind": "post.create", "tags": [], "content": { "text": "hola", "lang": "es",
             "zone": { "id": "z_3k9dq7m2x4pa", "mode": "home" } } },
  "secret_key": "<llave de prueba, solo para vectores>",
  "expect": { "payload_hex": "...", "id": "...", "sig": "...", "valid": true }
}
```

- Hay vectores válidos e inválidos. Cada inválido indica el código de error exacto.
- Las llaves de los vectores son públicas y solo de prueba. Están marcadas para que el escáner de secretos las ignore.
- Cambiar un vector existente es un cambio de protocolo y exige ADR.

## 4. Invariantes que deben tener prueba

**Protocolo**

- `verificar(firmar(e))` es verdadero para todo evento válido generado.
- Cambiar cualquier byte de un evento firmado invalida la verificación.
- Canonicalizar dos veces da el mismo resultado.
- Nativo y WASM producen el mismo `id` para toda entrada generada.

**Privacidad**

- Ningún tipo con coordenadas es serializable.
- Ningún tipo de secreto implementa `Debug`, `Display` ni serialización.
- El tráfico de una sesión completa no contiene coordenadas.
- Los registros de una sesión completa no contienen llaves, IP ni coordenadas.
- La web no carga ningún recurso de otro origen.
- Una imagen con metadatos se rechaza en el servidor.

**Zonas**

- La zona resuelta contiene al punto.
- Subir de nivel devuelve siempre un ancestro.
- Una zona por debajo del umbral nunca se publica.
- Oscilar sobre un borde no produce más de un cambio.

**Identidad (F2)**

- Un nullifier de método no inscribe dos identidades.
- El nivel lo decide la combinación más fuerte: cualquier cantidad de métodos de fuerza baja da nivel 1.
- La atestación y toda respuesta pública llevan el nivel y nunca la lista de métodos.
- Subir de nivel conserva la cuenta: mismo nullifier de ámbito, misma llave.
- Una cuenta de nivel 1 o 2 no puede crear billetera, cuentas secundarias ni avales.
- Un humano de nivel 3 no puede dar más avales que su cupo por época.
- Un nullifier de ámbito no crea dos cuentas en una época, ni bajo carrera.
- Una prueba ligada a otro compromiso o a otro reto se rechaza.
- Un vale ciego no se gasta dos veces.
- No existe consulta en la base de datos del registro que relacione un nullifier de documento con una cuenta.

**Registro adaptativo y puntaje (F2)**

- Para todo país del catálogo y toda combinación de capacidades, el formulario no ofrece ningún método que no se pueda completar.
- La lista de capacidades del dispositivo y el país elegido no aparecen en ninguna petición de red.
- Con el catálogo de Colombia: una cédula nunca recibe la opción `chip`; un pasaporte con chip en un teléfono sin NFC recibe "usar otro teléfono" y "punto de inscripción"; `cred` no aparece.
- Ningún flujo entrega puntos, saldo ni ventajas económicas por completar o añadir un método de verificación.
- El puntaje nunca supera el tope de la banda de su nivel.
- Ninguna cantidad de retos superados cambia el nivel ni concede billetera, avales o cuentas secundarias.
- Ignorar un reto penaliza una sola vez y deja la cuenta en pausa, no suspendida.
- Superar el reto pendiente reanuda la cuenta.
- El módulo del motor de riesgo no compila si importa la capa de red o el acceso a otra base de datos.
- El puntaje no aparece en ninguna respuesta pública ni en ningún evento.
- Un reto de presencia no permite al registro relacionar la cuenta con un compromiso de identidad.

**Gobernador (F3)**

- Toda violación dura detiene el trabajo en 5 segundos o menos.
- No se reanuda antes de 60 segundos de condiciones correctas.
- Nunca se supera un límite fijado por el usuario.
- Una señal no disponible se trata como el caso más conservador.

**Ledger (F4)**

- Las siete invariantes de `09` §4.
- Repetir una petición con la misma clave de idempotencia no cambia nada.
- Miles de canjes simultáneos sobre un mismo saldo no producen saldo negativo.

## 5. Objetivos de rendimiento

Valores iniciales. Se miden antes de cada compuerta.

| Operación | Objetivo |
| :-- | :-- |
| Feed de zona (p95) | Menos de 150 ms con 10 millones de publicaciones |
| Ingesta de evento (p95) | Menos de 100 ms |
| Verificación de firma en WASM | Menos de 1 ms por evento |
| Resolución de zona en el dispositivo | Menos de 5 ms |
| Primera carga de la web en móvil de gama media | Interactiva en menos de 3 s |
| Generación de prueba Semaphore en el cliente | Menos de 5 s |
| Canje completo | Menos de 5 s |
| Memoria del nodo en reposo | Menos de 150 MB |

## 6. Calidad de código

| Aspecto | Regla |
| :-- | :-- |
| Rust | `rustfmt`; `clippy` con avisos como errores; sin `unsafe`; sin `unwrap` fuera de pruebas |
| TypeScript | Modo estricto; sin `any`; ESLint; Prettier |
| Nombres | Código e identificadores en inglés; documentación y textos de interfaz en español e inglés |
| Comentarios | Explican el porqué, no el qué. Toda función pública del núcleo está documentada. |
| Errores | Tipados, con código estable. Los mensajes al usuario pasan por i18n. |
| Accesibilidad | Nivel AA; navegación por teclado; pruebas automáticas con axe en Playwright |
| Dependencias | Las mínimas. Cada una nueva se justifica en el PR. |

## 7. Datos de prueba

- Todo dato de prueba es sintético y se genera con semilla.
- Prohibido usar datos reales de usuarios, documentos reales o coordenadas reales de personas en pruebas, ejemplos o documentación.
- Para F2, los documentos de prueba del proveedor o documentos propios del responsable humano, nunca de terceros.

## 8. Cobertura

No se persigue un porcentaje. Se exige:

1. Toda regla de validación del protocolo tiene al menos un caso que pasa y uno que falla.
2. Toda invariante de §4 tiene prueba.
3. Todo error corregido lleva una prueba que lo habría detectado.
4. `social-crypto`, `social-core`, registro y ledger: toda rama alcanzable está cubierta.
