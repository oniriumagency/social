use hkdf::Hkdf;
use sha2::Sha256;

use crate::{CryptoError, SecretBytes};

/// HKDF con SHA-256 (RFC 5869). Deriva `out_len` bytes de `ikm` con `salt` e `info`.
///
/// `info` debe separar dominios: usa una etiqueta distinta por propósito
/// (por ejemplo, `b"social/v1/media-key"`).
///
/// # Errors
/// [`CryptoError::KdfOutputTooLong`] si `out_len` supera 255 × 32 bytes.
pub fn hkdf_sha256(
    ikm: &[u8],
    salt: Option<&[u8]>,
    info: &[u8],
    out_len: usize,
) -> Result<SecretBytes, CryptoError> {
    let hk = Hkdf::<Sha256>::new(salt, ikm);
    let mut okm = SecretBytes::new(vec![0u8; out_len]);
    hk.expand(info, okm.expose_mut())
        .map_err(|_| CryptoError::KdfOutputTooLong)?;
    Ok(okm)
}
