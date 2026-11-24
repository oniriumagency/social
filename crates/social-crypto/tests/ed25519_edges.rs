//! Casos límite de Ed25519 que los vectores públicos no cubren (docs/14 §8.1 y §8.4).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use social_crypto::{CryptoError, Signature, VerifyingKey};

fn h<const N: usize>(s: &str) -> [u8; N] {
    hex::decode(s).unwrap().try_into().unwrap()
}

/// Firma sintética con `R` = identidad (orden pequeño) y `s = k·a`, para la semilla
/// `[7u8; 32]` y el mensaje `social`. La verificación no estricta de dalek la acepta; la
/// estricta debe rechazarla. Si alguien cambia `verify_strict` por `verify`, falla.
#[test]
fn rejects_small_order_r_that_lax_verification_accepts() {
    let pk: [u8; 32] = h("ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c");
    let sig: [u8; 64] = h(concat!(
        "0100000000000000000000000000000000000000000000000000000000000000",
        "caba5828ac8238ff79b796fc4c36cdfd10b44bc5422616a00664d575c1be740b"
    ));
    let message = b"social";

    // El vector es significativo: la verificación laxa lo acepta.
    let lax_key = ed25519_dalek::VerifyingKey::from_bytes(&pk).unwrap();
    let lax_sig = ed25519_dalek::Signature::from_bytes(&sig);
    assert!(lax_key.verify_strict(message, &lax_sig).is_err());
    assert!(ed25519_dalek::Verifier::verify(&lax_key, message, &lax_sig).is_ok());

    let key = VerifyingKey::from_bytes(&pk).unwrap();
    assert_eq!(
        key.verify_strict(message, &Signature::from_bytes(sig)),
        Err(CryptoError::InvalidSignature)
    );
}

#[test]
fn rejects_public_key_of_wrong_length() {
    for len in [0, 31, 33, 64] {
        assert_eq!(
            VerifyingKey::from_slice(&vec![9u8; len]),
            Err(CryptoError::InvalidLength)
        );
    }
}

#[test]
fn rejects_point_not_on_curve() {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x02;
    assert_eq!(
        VerifyingKey::from_bytes(&bytes),
        Err(CryptoError::InvalidPublicKey)
    );
}

/// Codificaciones no canónicas: `y ≥ p` (p = 2^255 − 19) y `x = −0` con `y = 1`.
#[test]
fn rejects_non_canonical_encodings() {
    let y_equals_p = h("edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f");
    let y_equals_p_plus_1 = h("eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f");
    let negative_zero_x = h("0100000000000000000000000000000000000000000000000000000000000080");
    for bytes in [y_equals_p, y_equals_p_plus_1, negative_zero_x] {
        assert_eq!(
            VerifyingKey::from_bytes(&bytes),
            Err(CryptoError::InvalidPublicKey),
            "{}",
            hex::encode(bytes)
        );
    }
}
