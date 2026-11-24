use sha2::{Digest, Sha256};

/// Longitud en bytes de los resúmenes SHA-256 y BLAKE3 que usa el protocolo.
pub const HASH_LEN: usize = 32;

/// SHA-256. Se usa para los identificadores de evento (`docs/06` §3).
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; HASH_LEN] {
    Sha256::digest(data).into()
}

/// BLAKE3 de 32 bytes. Se usa para contenido binario y media (`docs/06` §7).
#[must_use]
pub fn blake3(data: &[u8]) -> [u8; HASH_LEN] {
    *::blake3::hash(data).as_bytes()
}

/// BLAKE3 incremental, para contenido que llega por partes.
#[derive(Default)]
pub struct Blake3Hasher(::blake3::Hasher);

impl Blake3Hasher {
    /// Crea un resumen vacío.
    #[must_use]
    pub fn new() -> Self {
        Self(::blake3::Hasher::new())
    }

    /// Añade datos.
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.0.update(data);
        self
    }

    /// Devuelve el resumen de todo lo añadido.
    #[must_use]
    pub fn finalize(&self) -> [u8; HASH_LEN] {
        *self.0.finalize().as_bytes()
    }
}
