use crate::{CoreError, ErrorCode};

/// Decodifica hexadecimal en minúsculas de exactamente `N` bytes (`docs/06` §1).
pub(crate) fn decode_fixed<const N: usize>(s: &str, field: &str) -> Result<[u8; N], CoreError> {
    let ok = s.len() == N * 2
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if !ok {
        return Err(CoreError::new(
            ErrorCode::Schema,
            format!(
                "{field}: se esperaban {} caracteres hexadecimales en minúsculas",
                N * 2
            ),
        ));
    }
    let mut out = [0u8; N];
    hex::decode_to_slice(s, &mut out)
        .map_err(|_| CoreError::new(ErrorCode::Schema, format!("{field}: hexadecimal inválido")))?;
    Ok(out)
}
