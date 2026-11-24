use crate::CryptoError;

/// Llena `buf` con bytes aleatorios del sistema operativo (CSPRNG).
///
/// # Errors
/// [`CryptoError::RandomUnavailable`] si el sistema no entrega aleatoriedad.
pub fn fill_random(buf: &mut [u8]) -> Result<(), CryptoError> {
    getrandom::fill(buf).map_err(|_| CryptoError::RandomUnavailable)
}

/// Devuelve `N` bytes aleatorios del sistema operativo. Para material secreto usa
/// [`crate::SecretKey32::generate`], que se borra al liberarse.
///
/// # Errors
/// [`CryptoError::RandomUnavailable`] si el sistema no entrega aleatoriedad.
pub fn random_array<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut out = [0u8; N];
    fill_random(&mut out)?;
    Ok(out)
}
