# 07 - Ubicación y Zonas

## 1. Requisitos

1. Feeds por seis escalas: barrio, localidad, ciudad, país, continente, global.
2. Las coordenadas no salen del dispositivo, nunca y por ninguna vía.
3. Una publicación no debe permitir identificar ni seguir a su autor por su ubicación.
4. Funciona en ciudades con buena cartografía y en lugares sin ella.

## 2. Catálogo de zonas

El catálogo es un conjunto de datos versionado que construye `tools/zone-pipeline` a partir de datos abiertos.

| Nivel | Nombre | Fuente | Respaldo |
| :-- | :-- | :-- | :-- |
| 0 | Global | Constante | No aplica |
| 1 | Continente | Tabla fija | No aplica |
| 2 | País | ISO 3166-1 | No aplica |
| 3 | Ciudad | Límites administrativos municipales | Celda H3 de resolución 5 (unos 253 km²) |
| 4 | Localidad | Distritos, comunas o localidades, donde existan | Se omite: el barrio cuelga de la ciudad |
| 5 | Barrio | Límites de barrio, donde sean fiables | Celda H3 de resolución 7 (unos 5,2 km²) |

**Registro de zona**

```json
{ "id": "z_3k9dq7m2x4pa", "level": 5, "parent": "z_8f2m...",
  "names": { "es": "Chapinero Alto" }, "country": "CO",
  "source": "admin", "catalog": 3 }
```

- `id` es opaco y estable entre versiones: `z_` más 60 bits en base32 del hash del identificador de origen.
- Si una zona se divide o se fusiona, una tabla de alias mantiene válidas las publicaciones antiguas.
- Cada zona guarda su **cobertura**: el conjunto compactado de celdas H3 de resolución 9 (unos 0,1 km²) que la componen. Se calcula una sola vez en el pipeline.
- Fuentes candidatas generales: Overture Maps (divisiones), OpenStreetMap, geoBoundaries. Las de Colombia están en §2.1. **Sus licencias (varias son ODbL) se revisan en el spike S3 antes de adoptarlas**; la atribución es obligatoria.

**Por qué H3 no es el catálogo:** la gente piensa en "mi barrio" y "mi ciudad", no en hexágonos. H3 sirve aquí para tres cosas: precomputar coberturas, repartir el catálogo en paquetes y dar un barrio sintético donde no hay datos.

### 2.1 Colombia: cómo se aplican los niveles

El piloto es Colombia, con Bogotá como ciudad por defecto (`15` D-01). Las fuentes de esta tabla son **candidatas**: el spike S3 comprueba su cobertura, su calidad y su licencia antes de adoptarlas.

| Nivel | En Colombia | Fuente candidata |
| :-- | :-- | :-- |
| 2 País | Colombia (`CO`) | ISO 3166-1 |
| 3 Ciudad | **Municipio o distrito.** Bogotá D.C. es una sola ciudad. | Marco Geoestadístico Nacional del DANE, con el código DIVIPOLA como identificador de origen |
| 4 Localidad | **Localidad** en Bogotá (Chapinero, Suba, Kennedy). **Comuna o corregimiento** en ciudades como Medellín y Cali. Se omite en los municipios que no tienen esta división. | Datos abiertos de cada alcaldía; en Bogotá, la infraestructura de datos espaciales del Distrito (IDECA) |
| 5 Barrio | Barrio. En Bogotá, el barrio o sector catastral. | Datos abiertos de cada alcaldía; OpenStreetMap donde no haya dato oficial |

- **El departamento no es un nivel de feed.** Se guarda como atributo de la ciudad para desambiguar la búsqueda, porque hay municipios con el mismo nombre en departamentos distintos.
- **Las divisiones de planeamiento de Bogotá** (unidades entre la localidad y el barrio) no se usan en la versión 1: la gente no se identifica con ellas.
- **Zonas rurales y veredas:** sin dato fiable de barrio, se usa el barrio sintético por celda H3 y la precisión por defecto sube a ciudad.
- **Nombres:** se guarda el nombre oficial y, cuando exista, el nombre de uso común.
- El catálogo v0 de F1 cubre Bogotá completa y la escalera país, continente y global. Las demás ciudades se añaden una a una, alineadas con el lanzamiento (`01` §5).

## 3. Reglas de privacidad

| Regla | Detalle |
| :-- | :-- |
| **R1. Resolución local** | La conversión de coordenadas a zona ocurre en el dispositivo. El tipo `Coordinates` del núcleo no se puede serializar ni registrar. |
| **R2. Zona fija por defecto** | En la bienvenida el usuario elige su zona. Puede dar permiso de ubicación una sola vez para recibir una sugerencia, o elegirla a mano. Las publicaciones usan esa zona hasta que la cambie. |
| **R3. Ubicación en vivo opcional** | Apagada por defecto. Si se activa, la zona se recalcula al publicar y el modo es `live`. |
| **R4. Precisión por defecto** | Localidad; si no existe, ciudad. Barrio es opcional, por publicación o como ajuste. |
| **R5. Umbral de k-anonimato** | Si la zona elegida tiene menos de `k` cuentas activas de nivel 2 o superior en 30 días, el cliente sube al nivel padre, y repite hasta cumplir. `k` inicial: 30. |
| **R6. Sin jitter** | La zona es determinista. No se añade ruido. |
| **R7. Adherencia** | En modo `live`, la zona solo cambia si el usuario se adentra más de una celda de resolución 9 en la zona vecina. Evita que alguien junto a un borde revele su posición alternando zonas. |
| **R8. Sin historial** | El dispositivo no guarda ubicaciones pasadas. El servidor solo conoce la zona de cada publicación. |
| **R9. Media limpia** | Las imágenes salen sin metadatos (`06` §7). |
| **R10. Consultas** | Leer el feed de una zona no exige cuenta y no se asocia a ninguna. |

La actividad de cada zona se publica solo por intervalos (por ejemplo "menos de 30", "30 a 100", "más de 100"), nunca con la cifra exacta.

## 4. Resolución en el dispositivo

**Paquetes de catálogo**

- El catálogo se reparte en archivos, uno por celda H3 de resolución 3 (unos 12.400 km²).
- Cada paquete contiene el mapa `celda H3 de resolución 9 → zona de barrio` de su área, más los registros de las zonas implicadas y sus ancestros.
- Los paquetes son archivos estáticos firmados, servidos por CDN.
- Descargar un paquete revela al CDN una región del tamaño de un área metropolitana. Es la única fuga, y es deliberadamente gruesa.

**Algoritmo**

```
1. coords      ← ubicación del sistema (o ninguna, si la elección es manual)
2. celda_r3    ← h3(coords, 3)          → determina qué paquete descargar
3. celda_r9    ← h3(coords, 9)
4. zona_barrio ← paquete.buscar(celda_r9)
5. zona        ← subir(zona_barrio, precisión_elegida)
6. mientras actividad(zona) < k: zona ← padre(zona)
7. descartar coords de memoria
```

- El paso 4 es una búsqueda en una tabla, sin geometría. Objetivo: menos de 5 ms en WASM.
- La elección manual usa `GET /v1/zones/search` y no necesita ubicación.
- Si no hay paquete para la región, el barrio es la celda H3 de resolución 7 y la ciudad la de resolución 5.

## 5. La zona es una etiqueta, no una prueba

La red no puede comprobar dónde está nadie. Se asume y se muestra:

- Cada publicación lleva `zone.mode`: `home` (zona fija), `live` (ubicación en vivo) o `remote` (zona elegida a mano para esa publicación).
- Los clientes distinguen visualmente las publicaciones `remote`.
- El cambio de zona fija está limitado a una vez cada 7 días.
- Los indexadores PUEDEN dar menos peso a `remote` en los feeds de barrio y localidad.
- Un humano tiene pocas cuentas, así que inundar un barrio ajeno tiene un techo bajo.

## 6. Indexación y consulta

1. La publicación lleva solo la zona hoja.
2. En la ingesta, el indexador recorre los ancestros con el catálogo y escribe una fila por nivel en `post_zones`.
3. `GET /v1/feed?zone=<id>` sirve cualquier nivel con la misma consulta por índice.
4. Barrio, localidad y ciudad se ordenan por tiempo. País, continente y global usan la muestra por puntuación (`06` §9).

## 7. Parámetros

| Parámetro | Valor inicial |
| :-- | :-- |
| `k` de anonimato | 30 cuentas activas de nivel 2 o superior en 30 días |
| Precisión por defecto | Localidad, o ciudad |
| Resolución H3 de cobertura | 9 |
| Resolución H3 de paquete | 3 |
| Barrio de respaldo | H3 resolución 7 |
| Ciudad de respaldo | H3 resolución 5 |
| Cambio de zona fija | 1 cada 7 días |

## 8. Pruebas obligatorias

1. Ningún tipo que contenga coordenadas implementa serialización (prueba de compilación).
2. Prueba de extremo a extremo que inspecciona todo el tráfico de red de una sesión con ubicación activa y falla si aparece un número con formato de coordenada.
3. Pruebas de propiedades: para cualquier punto, la zona resuelta contiene al punto; subir de nivel siempre devuelve un ancestro.
4. Prueba de borde: un recorrido que oscila sobre un límite no produce más de un cambio de zona.
5. Prueba de k-anonimato con zonas sintéticas poco pobladas.
