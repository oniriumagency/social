//! Prototipo del spike S3 (`docs/spikes/S3-zonas.md`): formato de paquete de zonas y
//! búsqueda celda → zona.
//!
//! Un paquete cubre una celda H3 de resolución 3 y contiene el mapa
//! `celda H3 de resolución 9 → zona hoja` ordenado por celda, más los registros de las
//! zonas implicadas y sus ancestros en JSON canónico (`docs/07` §4).
//!
//! Formato `SZP1` (enteros en little-endian):
//!
//! ```text
//! "SZP1" | versión u32 | n_celdas u32 | n_zonas u32 | bytes_json u32
//! n_celdas × (celda u64, índice de zona u32)   ordenadas por celda
//! JSON canónico: lista de n_zonas registros de zona
//! ```
//!
//! Es un prototipo: la versión de F1 (F1-T06, F1-T07) añade la firma del paquete y el
//! tipo `Coordinates` no serializable de `social-zones`.

#![forbid(unsafe_code)]

/// Resolución H3 de las celdas del mapa (`docs/07` §7).
pub const CELL_RES: u8 = 9;
/// Resolución H3 de los paquetes.
pub const PACK_RES: u8 = 3;

const MAGIC: &[u8; 4] = b"SZP1";
const VERSION: u32 = 1;
const HEADER: usize = 20;
const ENTRY: usize = 12;

/// Error al leer un paquete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackError {
    /// No empieza por `SZP1` o la versión no es la 1.
    BadHeader,
    /// La longitud no cuadra con la cabecera.
    Truncated,
    /// Las celdas no están ordenadas o un índice de zona no existe.
    Corrupt,
}

/// Paquete en memoria, listo para buscar.
#[derive(Debug, Clone)]
pub struct Pack {
    cells: Vec<u64>,
    zone_of: Vec<u32>,
    zones_json: String,
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, PackError> {
    let raw: [u8; 4] = bytes
        .get(at..at + 4)
        .and_then(|s| s.try_into().ok())
        .ok_or(PackError::Truncated)?;
    Ok(u32::from_le_bytes(raw))
}

fn u64_at(bytes: &[u8], at: usize) -> Result<u64, PackError> {
    let raw: [u8; 8] = bytes
        .get(at..at + 8)
        .and_then(|s| s.try_into().ok())
        .ok_or(PackError::Truncated)?;
    Ok(u64::from_le_bytes(raw))
}

fn to_usize(n: u32) -> Result<usize, PackError> {
    usize::try_from(n).map_err(|_| PackError::Truncated)
}

impl Pack {
    /// Codifica un paquete. `entries` se ordena por celda.
    #[must_use]
    pub fn encode(entries: &[(u64, u32)], zone_count: u32, zones_json: &str) -> Vec<u8> {
        let mut sorted = entries.to_vec();
        sorted.sort_unstable();
        let mut out = Vec::with_capacity(HEADER + sorted.len() * ENTRY + zones_json.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(
            &u32::try_from(sorted.len())
                .unwrap_or(u32::MAX)
                .to_le_bytes(),
        );
        out.extend_from_slice(&zone_count.to_le_bytes());
        out.extend_from_slice(
            &u32::try_from(zones_json.len())
                .unwrap_or(u32::MAX)
                .to_le_bytes(),
        );
        for (cell, zone) in sorted {
            out.extend_from_slice(&cell.to_le_bytes());
            out.extend_from_slice(&zone.to_le_bytes());
        }
        out.extend_from_slice(zones_json.as_bytes());
        out
    }

    /// Lee y comprueba un paquete.
    ///
    /// # Errors
    /// [`PackError`] si el formato no es válido.
    pub fn parse(bytes: &[u8]) -> Result<Self, PackError> {
        if bytes.get(..4) != Some(MAGIC.as_slice()) || u32_at(bytes, 4)? != VERSION {
            return Err(PackError::BadHeader);
        }
        let n = to_usize(u32_at(bytes, 8)?)?;
        let zones = u32_at(bytes, 12)?;
        let json_len = to_usize(u32_at(bytes, 16)?)?;
        let json_at = HEADER + n * ENTRY;
        if bytes.len() != json_at + json_len {
            return Err(PackError::Truncated);
        }
        let mut cells = Vec::with_capacity(n);
        let mut zone_of = Vec::with_capacity(n);
        for i in 0..n {
            let at = HEADER + i * ENTRY;
            let cell = u64_at(bytes, at)?;
            let zone = u32_at(bytes, at + 8)?;
            if zone >= zones || cells.last().is_some_and(|last| *last >= cell) {
                return Err(PackError::Corrupt);
            }
            cells.push(cell);
            zone_of.push(zone);
        }
        let zones_json = std::str::from_utf8(&bytes[json_at..])
            .map_err(|_| PackError::Corrupt)?
            .to_owned();
        Ok(Self {
            cells,
            zone_of,
            zones_json,
        })
    }

    /// Zona hoja de una celda de resolución 9: búsqueda binaria, sin geometría.
    #[must_use]
    pub fn find(&self, cell: u64) -> Option<u32> {
        self.cells
            .binary_search(&cell)
            .ok()
            .map(|i| self.zone_of[i])
    }

    /// Registros de zona en JSON canónico.
    #[must_use]
    pub fn zones_json(&self) -> &str {
        &self.zones_json
    }

    /// Número de celdas.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Indica si el paquete no tiene celdas.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

/// Medición en WASM: cargar un paquete y resolver puntos (solo para el spike).
#[cfg(feature = "wasm")]
mod wasm {
    use h3o::{LatLng, Resolution};
    use wasm_bindgen::prelude::*;

    use crate::Pack;

    /// Paquete cargado en WASM.
    #[wasm_bindgen]
    pub struct WasmPack(Pack);

    #[wasm_bindgen]
    impl WasmPack {
        /// Lee un paquete.
        ///
        /// # Errors
        /// Si el paquete no es válido.
        #[wasm_bindgen(constructor)]
        pub fn new(bytes: &[u8]) -> Result<Self, JsError> {
            Pack::parse(bytes)
                .map(Self)
                .map_err(|e| JsError::new(&format!("{e:?}")))
        }

        /// Pasos 3 y 4 de `docs/07` §4: punto → celda r9 → índice de zona, o -1.
        #[must_use]
        pub fn resolve(&self, lat: f64, lng: f64) -> i64 {
            LatLng::new(lat, lng)
                .ok()
                .map(|p| u64::from(p.to_cell(Resolution::Nine)))
                .and_then(|cell| self.0.find(cell))
                .map_or(-1, i64::from)
        }

        /// Número de celdas del paquete.
        #[must_use]
        pub fn cells(&self) -> usize {
            self.0.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_lookup() {
        let bytes = Pack::encode(&[(30, 1), (10, 0), (20, 1)], 2, "[]");
        let pack = Pack::parse(&bytes).unwrap();
        assert_eq!(pack.find(10), Some(0));
        assert_eq!(pack.find(30), Some(1));
        assert_eq!(pack.find(15), None);
        assert_eq!(pack.zones_json(), "[]");
    }

    #[test]
    fn rejects_corrupt_packs() {
        let bytes = Pack::encode(&[(10, 5)], 2, "[]");
        assert_eq!(Pack::parse(&bytes).err(), Some(PackError::Corrupt));
        let good = Pack::encode(&[(10, 0)], 1, "[]");
        assert_eq!(
            Pack::parse(&good[..good.len() - 1]).err(),
            Some(PackError::Truncated)
        );
        assert_eq!(Pack::parse(b"XXXX").err(), Some(PackError::BadHeader));
    }
}
