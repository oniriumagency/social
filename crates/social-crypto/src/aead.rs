use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{CryptoError, SecretBytes, random_array};

/// Longitud del nonce de XChaCha20-Poly1305. Con 24 bytes, el nonce aleatorio es seguro.
pub const XCHACHA_NONCE_LEN: usize = 24;
/// Longitud de la etiqueta de autenticación.
pub const XCHACHA_TAG_LEN: usize = 16;

/// Llave de XChaCha20-Poly1305. Se borra al liberarse.
///
/// No implementa `Debug`, `Display`, `Clone` ni serialización (I-3).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct AeadKey([u8; 32]);

impl AeadKey {
    /// Construye la llave a partir de 32 bytes. El llamador debe borrar su copia.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Genera una llave aleatoria.
    ///
    /// # Errors
    /// [`CryptoError::RandomUnavailable`] si el sistema no entrega aleatoriedad.
    pub fn generate() -> Result<Self, CryptoError> {
        let secret = crate::SecretKey32::generate()?;
        Ok(Self(*secret.expose()))
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new((&self.0).into())
    }
}

/// Cifra `plaintext` con XChaCha20-Poly1305 y un nonce aleatorio de 24 bytes.
///
/// Devuelve `nonce || texto_cifrado || etiqueta`. `aad` se autentica pero no se cifra.
///
/// # Errors
/// [`CryptoError::RandomUnavailable`] o [`CryptoError::AeadFailure`].
pub fn seal(key: &AeadKey, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let nonce: [u8; XCHACHA_NONCE_LEN] = random_array()?;
    let ciphertext = key
        .cipher()
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::AeadFailure)?;
    let mut out = Vec::with_capacity(XCHACHA_NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Descifra la salida de [`seal`]. El texto en claro se devuelve en un tipo que se borra.
///
/// # Errors
/// [`CryptoError::InvalidLength`] si la entrada es más corta que nonce y etiqueta;
/// [`CryptoError::AeadFailure`] si la autenticación falla.
pub fn open(key: &AeadKey, aad: &[u8], sealed: &[u8]) -> Result<SecretBytes, CryptoError> {
    if sealed.len() < XCHACHA_NONCE_LEN + XCHACHA_TAG_LEN {
        return Err(CryptoError::InvalidLength);
    }
    let (nonce, ciphertext) = sealed.split_at(XCHACHA_NONCE_LEN);
    let nonce = XNonce::try_from(nonce).map_err(|_| CryptoError::InvalidLength)?;
    key.cipher()
        .decrypt(
            &nonce,
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map(SecretBytes::new)
        .map_err(|_| CryptoError::AeadFailure)
}
