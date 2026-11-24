use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// Secreto de 32 bytes (semilla de llave, secreto de identidad, material de entrada de HKDF).
///
/// Se borra al liberarse. No implementa `Debug`, `Display`, `Clone` ni serialización (I-3).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey32([u8; 32]);

impl SecretKey32 {
    /// Construye el secreto a partir de bytes. El llamador debe borrar su copia.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Genera un secreto aleatorio con el generador del sistema operativo.
    ///
    /// # Errors
    /// [`crate::CryptoError::RandomUnavailable`] si el sistema no entrega aleatoriedad.
    pub fn generate() -> Result<Self, crate::CryptoError> {
        let mut bytes = Zeroizing::new([0u8; 32]);
        crate::fill_random(bytes.as_mut())?;
        Ok(Self(*bytes))
    }

    /// Acceso de solo lectura a los bytes. No los copies a tipos que no se borren.
    #[must_use]
    pub const fn expose(&self) -> &[u8; 32] {
        &self.0
    }
}

impl PartialEq for SecretKey32 {
    /// Comparación en tiempo constante.
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl Eq for SecretKey32 {}

/// Secreto de longitud variable (salida de HKDF, texto descifrado sensible).
///
/// Se borra al liberarse. No implementa `Debug`, `Display`, `Clone` ni serialización (I-3).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    /// Toma posesión de los bytes; se borrarán al liberarse.
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Acceso de solo lectura a los bytes.
    #[must_use]
    pub fn expose(&self) -> &[u8] {
        &self.0
    }

    pub(crate) fn expose_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    /// Longitud en bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Indica si no contiene bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl PartialEq for SecretBytes {
    /// Comparación en tiempo constante (la longitud no se considera secreta).
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl Eq for SecretBytes {}
