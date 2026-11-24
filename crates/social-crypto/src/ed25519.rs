use core::fmt;

use zeroize::Zeroizing;

use crate::{CryptoError, SecretKey32};

/// Longitud de una llave pública Ed25519.
pub const PUBLIC_KEY_LEN: usize = 32;
/// Longitud de una firma Ed25519.
pub const SIGNATURE_LEN: usize = 64;

/// Llave privada Ed25519. Se borra al liberarse.
///
/// No implementa `Debug`, `Display`, `Clone` ni serialización (I-3).
pub struct SigningKey(ed25519_dalek::SigningKey);

impl SigningKey {
    /// Genera una llave nueva con el generador del sistema operativo.
    ///
    /// # Errors
    /// [`CryptoError::RandomUnavailable`] si el sistema no entrega aleatoriedad.
    pub fn generate() -> Result<Self, CryptoError> {
        let seed = SecretKey32::generate()?;
        Ok(Self::from_seed(&seed))
    }

    /// Reconstruye la llave a partir de su semilla de 32 bytes (RFC 8032 §5.1.5).
    #[must_use]
    pub fn from_seed(seed: &SecretKey32) -> Self {
        Self(ed25519_dalek::SigningKey::from_bytes(seed.expose()))
    }

    /// Copia de la semilla, envuelta en un tipo que se borra al liberarse.
    /// Solo para guardarla en el almacén seguro del dispositivo.
    #[must_use]
    pub fn to_seed(&self) -> SecretKey32 {
        let bytes = Zeroizing::new(self.0.to_bytes());
        SecretKey32::from_bytes(*bytes)
    }

    /// Llave pública correspondiente.
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey(self.0.verifying_key())
    }

    /// Firma `message` (Ed25519 puro, determinista).
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Signature {
        use ed25519_dalek::Signer as _;
        Signature(self.0.sign(message).to_bytes())
    }
}

/// Llave pública Ed25519 validada: es un punto de la curva en el subgrupo de orden primo
/// (ni de orden pequeño ni de orden mixto). Las llaves generadas por Ed25519 siempre lo
/// cumplen; rechazar las demás cierra la puerta a firmas ambiguas entre implementaciones.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VerifyingKey(ed25519_dalek::VerifyingKey);

impl VerifyingKey {
    /// Interpreta 32 bytes como llave pública.
    ///
    /// # Errors
    /// [`CryptoError::InvalidPublicKey`] si no es un punto válido, tiene orden pequeño o
    /// tiene componente de torsión.
    pub fn from_bytes(bytes: &[u8; PUBLIC_KEY_LEN]) -> Result<Self, CryptoError> {
        let key = ed25519_dalek::VerifyingKey::from_bytes(bytes)
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        if key.is_weak() || !key.to_edwards().is_torsion_free() {
            return Err(CryptoError::InvalidPublicKey);
        }
        Ok(Self(key))
    }

    /// Interpreta un segmento como llave pública, comprobando la longitud.
    ///
    /// # Errors
    /// [`CryptoError::InvalidLength`] o [`CryptoError::InvalidPublicKey`].
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        let arr: &[u8; PUBLIC_KEY_LEN] =
            bytes.try_into().map_err(|_| CryptoError::InvalidLength)?;
        Self::from_bytes(arr)
    }

    /// Bytes de la llave pública.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LEN] {
        self.0.to_bytes()
    }

    /// Verificación estricta: rechaza firmas no canónicas y llaves de orden pequeño.
    ///
    /// # Errors
    /// [`CryptoError::InvalidSignature`] si la firma no es válida.
    pub fn verify_strict(&self, message: &[u8], signature: &Signature) -> Result<(), CryptoError> {
        let sig = ed25519_dalek::Signature::from_bytes(&signature.0);
        self.0
            .verify_strict(message, &sig)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}

impl fmt::Debug for VerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VerifyingKey(")?;
        for b in self.to_bytes() {
            write!(f, "{b:02x}")?;
        }
        write!(f, ")")
    }
}

/// Firma Ed25519 de 64 bytes. Es pública.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; SIGNATURE_LEN]);

impl Signature {
    /// Construye una firma desde sus bytes. La validez se comprueba al verificar.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; SIGNATURE_LEN]) -> Self {
        Self(bytes)
    }

    /// Construye una firma desde un segmento, comprobando la longitud.
    ///
    /// # Errors
    /// [`CryptoError::InvalidLength`] si no mide 64 bytes.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        let arr: [u8; SIGNATURE_LEN] = bytes.try_into().map_err(|_| CryptoError::InvalidLength)?;
        Ok(Self(arr))
    }

    /// Bytes de la firma.
    #[must_use]
    pub const fn to_bytes(&self) -> [u8; SIGNATURE_LEN] {
        self.0
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature(")?;
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        write!(f, ")")
    }
}

/// Atajo: verifica estrictamente `signature` sobre `message` con la llave en bytes.
///
/// # Errors
/// [`CryptoError::InvalidLength`], [`CryptoError::InvalidPublicKey`] o
/// [`CryptoError::InvalidSignature`].
pub fn verify_strict(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), CryptoError> {
    let key = VerifyingKey::from_slice(public_key)?;
    let sig = Signature::from_slice(signature)?;
    key.verify_strict(message, &sig)
}
