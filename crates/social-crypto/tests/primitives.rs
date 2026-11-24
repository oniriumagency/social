//! Pruebas de hash, HKDF, AEAD, aleatoriedad y de la higiene de los tipos secretos.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use social_crypto::{
    AeadKey, Blake3Hasher, CryptoError, SecretBytes, SecretKey32, SigningKey, blake3, hkdf_sha256,
    open, random_array, seal, sha256,
};
use static_assertions::assert_not_impl_any;

// I-3: ningún tipo secreto puede imprimirse, clonarse ni serializarse por accidente.
assert_not_impl_any!(SigningKey: core::fmt::Debug, core::fmt::Display, Clone);
assert_not_impl_any!(SecretKey32: core::fmt::Debug, core::fmt::Display, Clone);
assert_not_impl_any!(SecretBytes: core::fmt::Debug, core::fmt::Display, Clone);
assert_not_impl_any!(AeadKey: core::fmt::Debug, core::fmt::Display, Clone);

fn h(s: &str) -> Vec<u8> {
    hex::decode(s).unwrap()
}

/// FIPS 180-2, ejemplos "abc" y cadena vacía.
#[test]
fn sha256_known_answers() {
    assert_eq!(
        sha256(b"abc").to_vec(),
        h("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    );
    assert_eq!(
        sha256(b"").to_vec(),
        h("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    );
}

/// Vector oficial de BLAKE3 para la entrada vacía.
#[test]
fn blake3_known_answer_and_incremental() {
    assert_eq!(
        blake3(b"").to_vec(),
        h("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262")
    );
    let data = vec![0xabu8; 10_000];
    let mut inc = Blake3Hasher::new();
    for chunk in data.chunks(333) {
        inc.update(chunk);
    }
    assert_eq!(inc.finalize(), blake3(&data));
}

/// RFC 5869, apéndice A.1 (caso de prueba 1, HKDF-SHA256).
#[test]
fn hkdf_rfc5869_case_1() {
    let ikm = h("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
    let salt = h("000102030405060708090a0b0c");
    let info = h("f0f1f2f3f4f5f6f7f8f9");
    let okm = hkdf_sha256(&ikm, Some(&salt), &info, 42).unwrap();
    assert_eq!(
        okm.expose().to_vec(),
        h("3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865")
    );
}

#[test]
fn hkdf_rejects_too_long_output() {
    assert_eq!(
        hkdf_sha256(b"ikm", None, b"info", 255 * 32 + 1).err(),
        Some(CryptoError::KdfOutputTooLong)
    );
}

#[test]
fn aead_round_trip_and_tamper_detection() {
    let key = AeadKey::generate().unwrap();
    let sealed = seal(&key, b"aad", b"mensaje secreto").unwrap();
    assert_eq!(
        open(&key, b"aad", &sealed).unwrap().expose(),
        b"mensaje secreto"
    );

    // Otro AAD, otra llave o cualquier byte alterado: se rechaza.
    assert_eq!(
        open(&key, b"otro", &sealed).err(),
        Some(CryptoError::AeadFailure)
    );
    let other = AeadKey::generate().unwrap();
    assert_eq!(
        open(&other, b"aad", &sealed).err(),
        Some(CryptoError::AeadFailure)
    );
    for i in 0..sealed.len() {
        let mut bad = sealed.clone();
        bad[i] ^= 1;
        assert!(
            open(&key, b"aad", &bad).is_err(),
            "byte {i} alterado aceptado"
        );
    }
    assert_eq!(
        open(&key, b"aad", &sealed[..39]).err(),
        Some(CryptoError::InvalidLength)
    );
}

#[test]
fn aead_uses_fresh_nonces() {
    let key = AeadKey::from_bytes([1u8; 32]);
    let a = seal(&key, b"", b"x").unwrap();
    let b = seal(&key, b"", b"x").unwrap();
    assert_ne!(a, b);
}

#[test]
fn random_is_not_constant() {
    let a: [u8; 32] = random_array().unwrap();
    let b: [u8; 32] = random_array().unwrap();
    assert_ne!(a, b);
}

#[test]
fn sign_verify_and_tamper() {
    let key = SigningKey::generate().unwrap();
    let vk = key.verifying_key();
    let sig = key.sign(b"hola");
    vk.verify_strict(b"hola", &sig).unwrap();
    assert_eq!(
        vk.verify_strict(b"hola!", &sig).err(),
        Some(CryptoError::InvalidSignature)
    );
    let restored = SigningKey::from_seed(&key.to_seed());
    assert_eq!(restored.verifying_key(), vk);
}

#[test]
fn error_codes_are_stable() {
    assert_eq!(CryptoError::InvalidSignature.code(), "E_CRYPTO_SIGNATURE");
    assert_eq!(
        CryptoError::InvalidSignature.to_string(),
        "E_CRYPTO_SIGNATURE"
    );
}

// I-3, docs/14 §8.3: las copias de material secreto dentro de las dependencias también se
// borran al liberarse. Si alguien quita la característica `zeroize` de una dependencia,
// esto deja de compilar.
static_assertions::assert_impl_all!(chacha20poly1305::XChaCha20Poly1305: zeroize::ZeroizeOnDrop);
static_assertions::assert_impl_all!(ed25519_dalek::SigningKey: zeroize::ZeroizeOnDrop);
static_assertions::assert_impl_all!(sha2::Sha256: zeroize::ZeroizeOnDrop);
