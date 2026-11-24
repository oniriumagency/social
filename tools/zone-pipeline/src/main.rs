//! Prototipo de `tools/zone-pipeline` (spike S3): construye el catálogo de zonas y los
//! paquetes de una ciudad a partir de límites administrativos en GeoJSON.
//!
//! Uso: `cargo run --release -p zone-pipeline -- <entrada.geojson> <directorio-salida>`
//!
//! La entrada es una `FeatureCollection` cuyas propiedades son `source_id` (identificador
//! de origen estable), `level` (3 ciudad, 4 localidad, 5 barrio), `name` y `dataset`.
//! La salida es `catalog.json`, un paquete `pack-<celda r3>.bin` por celda de resolución 3
//! y `stats.json` con las cifras del informe.
//!
//! Reglas aplicadas (`docs/07` §2 y §4):
//! - Cobertura de cada zona: celdas H3 de resolución 9 cuyo centroide cae dentro.
//! - Zona hoja de cada celda: el barrio que la cubre; si no hay, un barrio sintético por
//!   celda H3 de resolución 7, colgado de la localidad (o de la ciudad).
//! - Si dos barrios reclaman una celda, gana el más pequeño (el más específico).
//! - `id = "z_" + base32 de Crockford de los primeros 60 bits de SHA-256(source_id)`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use geo::{MultiPolygon, Polygon};
use h3o::geom::{ContainmentMode, TilerBuilder};
use h3o::{CellIndex, Resolution};
use serde_json::{Value, json};
use zone_pipeline::{CELL_RES, PACK_RES, Pack};

const CROCKFORD: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

fn zone_id(source_id: &str) -> String {
    let digest = social_crypto::sha256(source_id.as_bytes());
    let mut first = [0u8; 8];
    first.copy_from_slice(&digest[..8]);
    let bits = u64::from_be_bytes(first) >> 4; // 60 bits
    let mut out = String::from("z_");
    for i in (0..12).rev() {
        let idx = usize::try_from((bits >> (i * 5)) & 31).unwrap_or(0);
        out.push(char::from(CROCKFORD[idx]));
    }
    out
}

struct Zone {
    id: String,
    level: u8,
    name: String,
    source: String,
    dataset: String,
    parent: Option<usize>,
    cells: BTreeSet<CellIndex>,
}

fn res(n: u8) -> Resolution {
    Resolution::try_from(n).unwrap_or(Resolution::Nine)
}

fn coverage(geometry: &geojson::Geometry) -> Result<BTreeSet<CellIndex>, String> {
    let polygons: Vec<Polygon> = match MultiPolygon::<f64>::try_from(geometry.value.clone()) {
        Ok(mp) => mp.0,
        Err(_) => {
            vec![Polygon::<f64>::try_from(geometry.value.clone()).map_err(|e| e.to_string())?]
        }
    };
    let mut tiler = TilerBuilder::new(res(CELL_RES))
        .containment_mode(ContainmentMode::ContainsCentroid)
        .build();
    for p in polygons {
        tiler.add(p).map_err(|e| e.to_string())?;
    }
    Ok(tiler.into_coverage().collect())
}

fn load(input: &Path) -> Result<Vec<Zone>, String> {
    let text = std::fs::read_to_string(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let fc: geojson::FeatureCollection = text
        .parse::<geojson::GeoJson>()
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|e: geojson::Error| e.to_string())?;
    let mut zones = Vec::new();
    for f in fc.features {
        let props = f.properties.as_ref().ok_or("feature sin propiedades")?;
        let get = |k: &str| {
            props
                .get(k)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        let level = props
            .get("level")
            .and_then(Value::as_u64)
            .and_then(|l| u8::try_from(l).ok())
            .ok_or("level")?;
        let geometry = f.geometry.as_ref().ok_or("feature sin geometría")?;
        let source = get("source_id");
        zones.push(Zone {
            id: zone_id(&source),
            level,
            name: get("name"),
            source,
            dataset: get("dataset"),
            parent: None,
            cells: coverage(geometry)?,
        });
    }
    Ok(zones)
}

/// Padre de cada zona: la de nivel inmediatamente superior que contiene más de sus celdas.
fn assign_parents(zones: &mut [Zone]) {
    let owners: Vec<(usize, u8, BTreeSet<CellIndex>)> = zones
        .iter()
        .enumerate()
        .map(|(i, z)| (i, z.level, z.cells.clone()))
        .collect();
    for zone in zones.iter_mut() {
        let best = owners
            .iter()
            .filter(|(_, level, _)| *level < zone.level)
            .map(|(i, level, cells)| (*level, cells.intersection(&zone.cells).count(), *i))
            .filter(|(_, overlap, _)| *overlap > 0)
            .max();
        zone.parent = best.map(|(_, _, i)| i);
    }
}

fn run(input: &Path, out_dir: &Path) -> Result<(), String> {
    let mut zones = load(input)?;
    assign_parents(&mut zones);
    let city = zones
        .iter()
        .position(|z| z.level == 3)
        .ok_or("falta la ciudad (level 3)")?;
    let city_cells = zones[city].cells.clone();

    // Zona hoja de cada celda: barrio real más pequeño, o barrio sintético por celda r7.
    let mut leaf: BTreeMap<CellIndex, usize> = BTreeMap::new();
    let mut claims: HashMap<CellIndex, usize> = HashMap::new();
    let barrios: Vec<usize> = (0..zones.len()).filter(|i| zones[*i].level == 5).collect();
    for &b in &barrios {
        for cell in &zones[b].cells {
            if !city_cells.contains(cell) {
                continue;
            }
            *claims.entry(*cell).or_default() += 1;
            let better = leaf
                .get(cell)
                .is_none_or(|cur| zones[b].cells.len() < zones[*cur].cells.len());
            if better {
                leaf.insert(*cell, b);
            }
        }
    }
    let real_barrio_cells = leaf.len();
    let locality_of = |cell: &CellIndex, zones: &[Zone]| {
        (0..zones.len()).find(|i| zones[*i].level == 4 && zones[*i].cells.contains(cell))
    };
    let mut synthetic: HashMap<(CellIndex, Option<usize>), usize> = HashMap::new();
    for cell in &city_cells {
        if leaf.contains_key(cell) {
            continue;
        }
        let r7 = cell.parent(res(7)).unwrap_or(*cell);
        let loc = locality_of(cell, &zones);
        let idx = match synthetic.get(&(r7, loc)) {
            Some(i) => *i,
            None => {
                let parent_source =
                    loc.map_or_else(|| zones[city].source.clone(), |l| zones[l].source.clone());
                let source = format!("h3r7:{r7}@{parent_source}");
                zones.push(Zone {
                    id: zone_id(&source),
                    level: 5,
                    name: format!("Sector {r7}"),
                    source,
                    dataset: "sintético H3 r7".into(),
                    parent: Some(loc.unwrap_or(city)),
                    cells: BTreeSet::new(),
                });
                synthetic.insert((r7, loc), zones.len() - 1);
                zones.len() - 1
            }
        };
        zones[idx].cells.insert(*cell);
        leaf.insert(*cell, idx);
    }

    // Catálogo: registros de zona, con la escalera ciudad → país → continente → global.
    let record = |z: &Zone, zones: &[Zone]| {
        let parent = z
            .parent
            .map_or_else(|| zone_id("iso3166:CO"), |p| zones[p].id.clone());
        json!({"id": z.id, "level": z.level, "parent": parent, "names": {"es": z.name},
               "country": "CO", "source": z.dataset})
    };
    let mut catalog: Vec<Value> = vec![
        json!({"id": zone_id("global"), "level": 0, "names": {"es": "Global"}, "source": "constante"}),
        json!({"id": zone_id("continent:SA"), "level": 1, "parent": zone_id("global"), "names": {"es": "Sudamérica"}, "source": "tabla fija"}),
        json!({"id": zone_id("iso3166:CO"), "level": 2, "parent": zone_id("continent:SA"), "names": {"es": "Colombia"}, "country": "CO", "source": "ISO 3166-1"}),
    ];
    catalog.extend(zones.iter().map(|z| record(z, &zones)));
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    std::fs::write(
        out_dir.join("catalog.json"),
        serde_json::to_string(&catalog).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    // Paquetes por celda r3: mapa celda → índice local, y zonas implicadas con ancestros.
    let mut by_pack: BTreeMap<CellIndex, Vec<(CellIndex, usize)>> = BTreeMap::new();
    for (cell, z) in &leaf {
        by_pack
            .entry(cell.parent(res(PACK_RES)).unwrap_or(*cell))
            .or_default()
            .push((*cell, *z));
    }
    let mut packs = Vec::new();
    for (pack_cell, entries) in &by_pack {
        let mut local: BTreeMap<usize, u32> = BTreeMap::new();
        let mut records = Vec::new();
        let include =
            |mut z: Option<usize>, local: &mut BTreeMap<usize, u32>, records: &mut Vec<Value>| {
                while let Some(i) = z {
                    if local.contains_key(&i) {
                        break;
                    }
                    local.insert(i, u32::try_from(records.len()).unwrap_or(u32::MAX));
                    records.push(record(&zones[i], &zones));
                    z = zones[i].parent;
                }
            };
        for (_, z) in entries {
            include(Some(*z), &mut local, &mut records);
        }
        let mapped: Vec<(u64, u32)> = entries
            .iter()
            .map(|(c, z)| (u64::from(*c), local[z]))
            .collect();
        let json = serde_json::to_string(&records).map_err(|e| e.to_string())?;
        let bytes = Pack::encode(
            &mapped,
            u32::try_from(records.len()).unwrap_or(u32::MAX),
            &json,
        );
        let path = out_dir.join(format!("pack-{pack_cell}.bin"));
        std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
        packs.push(json!({"cell": pack_cell.to_string(), "cells": entries.len(), "zones": records.len(), "bytes": bytes.len()}));
    }

    let count = |level: u8, synthetic_only: bool| {
        zones
            .iter()
            .filter(|z| z.level == level && (z.dataset.starts_with("sintético") == synthetic_only))
            .count()
    };
    let city_without_locality = city_cells
        .iter()
        .filter(|c| locality_of(c, &zones).is_none())
        .count();
    let stats = json!({
        "city_cells_r9": city_cells.len(),
        "localities": count(4, false),
        "city_cells_without_locality": city_without_locality,
        "real_barrios": count(5, false),
        "synthetic_barrios": count(5, true),
        "cells_with_real_barrio": real_barrio_cells,
        "cells_claimed_by_two_or_more_barrios": claims.values().filter(|n| **n > 1).count(),
        "packs": packs,
    });
    std::fs::write(
        out_dir.join("stats.json"),
        serde_json::to_string_pretty(&stats).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&stats).unwrap_or_default()
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [input, out_dir] = args.as_slice() else {
        eprintln!("uso: zone-pipeline <entrada.geojson> <directorio-salida>");
        return ExitCode::from(2);
    };
    match run(input, out_dir) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("zone-pipeline: {e}");
            ExitCode::FAILURE
        }
    }
}
