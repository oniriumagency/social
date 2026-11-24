//! # social-crypto
//!
//! Único punto de acceso a criptografía del proyecto (invariante I-8, ADR-003).
//! Envuelve crates de uso amplio y revisión pública (RustCrypto, dalek y BLAKE3) y no
//! implementa ninguna primitiva propia.
//!
//! Reglas de este crate:
//! - Los tipos con material secreto ([`SigningKey`], [`SecretKey32`], [`AeadKey`],
//!   [`SecretBytes`]) se borran de memoria al liberarse y **no** implementan `Debug`,
//!   `Display`, `Clone` ni serialización, para que no puedan acabar en un registro (I-3).
//! - Ed25519 se verifica siempre en modo estricto: se rechazan firmas no canónicas y
//!   llaves de orden pequeño (`docs/06` §1).
//! - Los errores tienen un código estable y nunca incluyen material secreto.

#![forbid(unsafe_code)]

mod aead;
mod ed25519;
mod error;
mod hash;
mod kdf;
mod rand;
mod secret;

pub use aead::{AeadKey, XCHACHA_NONCE_LEN, XCHACHA_TAG_LEN, open, seal};
pub use ed25519::{
    PUBLIC_KEY_LEN, SIGNATURE_LEN, Signature, SigningKey, VerifyingKey, verify_strict,
};
pub use error::CryptoError;
pub use hash::{Blake3Hasher, HASH_LEN, blake3, sha256};
pub use kdf::hkdf_sha256;
pub use rand::{fill_random, random_array};
pub use secret::{SecretBytes, SecretKey32};
