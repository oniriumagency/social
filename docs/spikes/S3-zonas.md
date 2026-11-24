# Spike S3 — Zonas de Bogotá

**Tarea:** F0-S3 (`docs/12` §2) · **Fecha:** 2026-11-24 · **Autor:** Claude Code · **Estado:** en revisión

**Pregunta:** ¿se puede construir el catálogo de zonas de Bogotá (ciudad, localidad y barrio) con datos abiertos, con paquetes pequeños y una búsqueda en el dispositivo por debajo de 5 ms, y cómo se comporta en los bordes?

## Resumen

| Criterio | Resultado | Veredicto |
| :-- | :-- | :-- |
| Búsqueda celda → zona en WASM | **1,9 µs** por punto (coordenada → celda r9 → zona), peor caso 0,06 ms; leer el paquete, 0,9 ms | Cumple con mucho margen (objetivo: 5 ms) |
| Tamaño del paquete | 216 KB (49 KB el segundo); **30 KB con gzip**. El catálogo de toda Bogotá, 88 KB (10 KB con gzip) | Cumple |
| Ciudad (nivel 3) | Bogotá D.C. completa: 14.690 celdas r9 | Cumple |
| Localidad (nivel 4) | **Las 20 localidades**, de OpenStreetMap vía Overture. El 0,6 % de las celdas de la ciudad no cae en ninguna (bordes) y cuelga de la ciudad | Cumple |
| Barrio (nivel 5) | **Insuficiente con datos abiertos globales:** solo 84 barrios utilizables, que cubren el 1,7 % de las celdas. El resto recibe barrio sintético (518 sectores H3 r7) | **No cumple**: hace falta el dato oficial de IDECA |
| Bordes | El 8,9 % de las celdas tocan otra localidad. En 7.977 puntos al azar dentro de alguna localidad, la localidad resuelta coincide con la geométrica en el **96,7 %**; el 3,3 % restante son puntos a menos de una celda del borde | Aceptable con matices (§5) |
| Licencias | Overture «divisions»: ODbL (derivado de OSM). Límite de Bogotá: geoBoundaries, CC BY 4.0. IDECA y DANE: **sin verificar** (§3) | Pendiente |

**Recomendación:** adoptar el diseño de `docs/07` tal como está (tabla celda r9 → zona, paquetes por celda r3, barrio sintético de respaldo). Para el barrio, usar el dato oficial de Bogotá (IDECA, sector catastral o barrio) en lugar de OpenStreetMap, y mientras tanto lanzar con precisión por defecto de localidad (R4), que ya está completa. Antes de F1-T06 hay que confirmar las licencias de IDECA y DANE y que el entorno pueda descargarlos (§6).

## 1. Método

1. **Datos.** Overture Maps, versión `2026-11-18.0`, tema `divisions`, tipo `division_area`, filtrado por país `CO` y el rectángulo de Bogotá. Se leyó directamente del almacenamiento público de Overture con pyarrow (apéndice A). Los portales colombianos (datosabiertos.bogota.gov.co, IDECA, geoportal del DANE) y OpenStreetMap directo (Overpass, Geofabrik) **no son accesibles desde el entorno de desarrollo en la nube**, ni desde la carpeta conectada del equipo del responsable: la política de red los bloquea.
2. **Selección.** Ciudad: el polígono `county` «Bogotá, D.C.» (fuente geoBoundaries). Localidades: los 20 polígonos `locality` de OSM cuyo nombre coincide con las 20 localidades oficiales. Barrios: polígonos `neighborhood` dentro de Bogotá, excluyendo las agrupaciones de UPZ («UPZs Localidad …»), que no son barrios.
3. **Prototipo** `tools/zone-pipeline` (Rust, en este PR): cobertura de cada zona en celdas H3 r9 por centroide (h3o), asignación de padres por solape, barrio sintético por celda r7 colgado de su localidad, identificadores `z_` deterministas, catálogo y un paquete binario `SZP1` por celda r3. Ejecución completa: 0,15 s.
4. **Medición en WASM:** el mismo crate compilado a `wasm32-unknown-unknown` (función `WasmPack.resolve`), 100.000 puntos sintéticos al azar sobre Bogotá urbana en Node 22.
5. **Bordes:** puntos al azar dentro de Bogotá (7.977 caen dentro de alguna localidad); se compara la localidad que da el paquete con la que da la geometría (punto en polígono).

## 2. Cobertura

| Nivel | Fuente usada | Zonas | Celdas r9 cubiertas |
| :-- | :-- | --: | --: |
| 3 Ciudad | geoBoundaries (vía Overture) | 1 | 14.690 (100 %) |
| 4 Localidad | OpenStreetMap (vía Overture) | 20 | 14.608 (99,4 %) |
| 5 Barrio real | OpenStreetMap y Esri Community Maps (vía Overture) | 84 | 252 (1,7 %) |
| 5 Barrio sintético | H3 r7 por localidad | 518 | 14.438 (98,3 %) |

- Overture tiene unos 310 polígonos de barrio en el rectángulo, pero muchos son agrupaciones de UPZ o barrios de municipios vecinos (Soacha, Mosquera). Bogotá tiene muchos cientos de barrios oficiales (la cifra exacta, por confirmar con IDECA): OSM cubre una fracción mínima.
- No hubo celdas reclamadas por dos barrios (los polígonos de OSM no se solapan en Bogotá).
- **El barrio sintético r7 (unos 5 km²) es más grande que muchos barrios reales** y no tiene nombre reconocible. Sirve de respaldo, no como experiencia principal. Confirma el riesgo previsto en `docs/12` §7 («datos de barrio insuficientes en Bogotá»).

## 3. Fuentes y licencias

| Fuente | Qué aporta | Licencia | Estado |
| :-- | :-- | :-- | :-- |
| Overture Maps, divisions | Ciudad, localidades y algunos barrios | ODbL 1.0 (derivado de OSM); exige atribución y que la base derivada se comparta con la misma licencia | Según la documentación de Overture; revisar con el abogado (D-12) |
| geoBoundaries (límite de Bogotá en Overture) | Ciudad | CC BY 4.0 | Atribución |
| Esri Community Maps (algunos barrios en Overture) | Barrio | Incluido en Overture bajo ODbL | Revisar condiciones de Esri |
| IDECA (Infraestructura de Datos Espaciales de Bogotá) | Localidades, UPZ, barrios y sectores catastrales oficiales | Sin verificar desde aquí | **Bloqueado por la red** |
| DANE, Marco Geoestadístico Nacional | Municipios con código DIVIPOLA (Bogotá: 11001) | Sin verificar desde aquí | **Bloqueado por la red** |

**Consecuencia de la ODbL:** si el catálogo mezcla datos de OSM, el catálogo publicado (paquetes incluidos) es una base de datos derivada y debe ofrecerse bajo ODbL con atribución. No afecta al código (AGPL o Apache) ni a las publicaciones de los usuarios. Si el catálogo de Bogotá se construye solo con IDECA y DANE, depende de la licencia de esos datos.

## 4. Paquetes y rendimiento

| Paquete (celda r3) | Celdas r9 | Zonas incluidas (con ancestros) | Bytes | Con gzip |
| :-- | --: | --: | --: | --: |
| `8366e0fffffffff` (Bogotá urbana y norte) | 12.165 | 494 | 215.573 | 30.092 |
| `8366e4fffffffff` (sur, Usme y Sumapaz) | 2.525 | 134 | 49.116 | 7.585 |

- El formato es una lista ordenada de pares (celda u64, zona u32), 12 bytes por celda, más los registros de zona en JSON. Con miles de barrios reales el JSON crecerá, pero el mapa de celdas no: el tamaño está acotado por el área.
- Si hiciera falta reducirlo: celdas compactadas por padre común o codificación delta; no es necesario para el objetivo.
- **Búsqueda en WASM:** 1,93 µs de media por punto, incluida la conversión de coordenadas a celda r9; peor caso medido 0,057 ms; analizar el paquete, 0,86 ms. El WASM de la búsqueda pesa 63 KB sin comprimir.

## 5. Comportamiento en los bordes

- La asignación por centroide garantiza que **cada celda pertenece a una sola zona**: no hay huecos ni dobles asignaciones entre zonas adyacentes.
- El precio es la cuantización: una celda r9 tiene unos 174 m de lado. Un punto a menos de una celda del límite puede resolverse a la localidad vecina. Medido: **3,3 %** de los puntos al azar.
- Para la privacidad es neutro o favorable: la zona es una etiqueta (`docs/07` §5) y el error no depende de quién publica.
- Para la experiencia, alguien que vive junto a un límite podría ver sugerida la localidad vecina. Por eso la sugerencia se confirma siempre a mano en la bienvenida (R2).
- La adherencia en modo `live` (R7: cambiar de zona solo tras adentrarse más de una celda) vive en el cliente (F1-T07) y trabaja sobre las mismas celdas r9, así que el diseño encaja sin cambios.
- Mejora opcional si el error molesta: guardar las celdas de borde a resolución 10 (unos 66 m), lo que añade un 9 % de entradas aproximadamente.

## 6. Qué hace falta para cerrar (decisiones y acciones)

1. **Acceso a los datos oficiales.** Habilitar en la red del entorno los dominios de IDECA y datos abiertos de Bogotá y del DANE, o que el responsable descargue a mano las capas de localidades y barrios o sectores catastrales de IDECA y el MGN del DANE a la carpeta del proyecto. Con eso se repite este spike con el dato oficial (una hora de trabajo).
2. **Licencias** de IDECA y DANE, y si se acepta ODbL para el catálogo, con el abogado (D-12).
3. **Barrio o sector catastral:** el sector catastral cubre toda la ciudad sin huecos, pero sus nombres no siempre coinciden con el barrio que la gente dice. Propuesta: sector catastral como geometría y nombre de barrio de uso común cuando exista (`docs/07` §2.1, «nombres»).
4. **Actualizar `docs/07`** tras G0: confirmar resolución r9, paquete r3 y el respaldo sintético; añadir la decisión sobre la ODbL.

## 7. Cómo reproducirlo

```
# 1. Extraer los datos (apéndice A) a bogota_input.geojson
# 2. Construir catálogo y paquetes
cargo run --release -p zone-pipeline -- bogota_input.geojson salida/
# 3. Búsqueda en WASM
cargo build --release -p zone-pipeline --lib --no-default-features --features wasm --target wasm32-unknown-unknown
wasm-bindgen --target nodejs --out-dir wasm target/wasm32-unknown-unknown/release/zone_pipeline.wasm
```

Los datos descargados no se guardan en el repositorio (licencia ODbL y tamaño).

## Apéndice A. Extracción de Overture

Ejecutado una sola vez fuera del repositorio (Python 3 con pyarrow y shapely; el repositorio no admite scripts de Python, y F1-T06 hará la extracción en Rust):

```python
import pyarrow.dataset as ds, pyarrow.fs as fs, pyarrow.compute as pc
s3 = fs.S3FileSystem(anonymous=True, region="us-west-2")
d = ds.dataset("overturemaps-us-west-2/release/2026-11-18.0/theme=divisions/type=division_area/",
               filesystem=s3, format="parquet")
f = ((pc.field("bbox", "xmax") > -74.5) & (pc.field("bbox", "xmin") < -73.9)
     & (pc.field("bbox", "ymax") > 3.7) & (pc.field("bbox", "ymin") < 4.85)
     & (pc.field("country") == "CO"))
rows = d.to_table(filter=f, columns=["division_id", "subtype", "names", "geometry", "sources"]).to_pylist()
# Ciudad: subtype "county" con nombre «Bogotá, D.C.».
# Localidades: subtype "locality" cuyo nombre normalizado es una de las 20 localidades.
# Barrios: subtype "neighborhood" dentro de Bogotá, sin las agrupaciones «UPZs …».
# Salida: FeatureCollection con properties {source_id, level, name, dataset}.
```
