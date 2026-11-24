use core::fmt;

/// Error criptográfico con código estable. Nunca contiene material secreto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CryptoError {
    /// Una llave pública no es un punto válido de la curva o tiene orden pequeño.
    InvalidPublicKey,
    /// La firma no es válida para el mensaje y la llave, o no es canónica.
    InvalidSignature,
    /// Longitud de entrada incorrecta (llave, firma, nonce o salida).
    InvalidLength,
    /// El cifrado autenticado falló: texto cifrado, nonce, datos asociados o llave incorrectos.
    AeadFailure,
    /// La derivación de llaves pidió más bytes de los que HKDF puede producir.
    KdfOutputTooLong,
    /// El sistema operativo no pudo entregar aleatoriedad.
    RandomUnavailable,
}

impl CryptoError {
    /// Código estable, apto para API y registros.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidPublicKey => "E_CRYPTO_PUBLIC_KEY",
            Self::InvalidSignature => "E_CRYPTO_SIGNATURE",
            Self::InvalidLength => "E_CRYPTO_LENGTH",
            Self::AeadFailure => "E_CRYPTO_AEAD",
            Self::KdfOutputTooLong => "E_CRYPTO_KDF_LENGTH",
            Self::RandomUnavailable => "E_CRYPTO_RANDOM",
        }
    }
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for CryptoError {}
