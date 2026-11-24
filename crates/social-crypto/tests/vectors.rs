//! Vectores públicos de Ed25519: RFC 8032 §7.1, el conjunto de djb (sign.input,
//! 128 casos, del repositorio de ed25519-dalek) y Wycheproof (C2SP, Apache-2.0).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde::Deserialize;
use social_crypto::{SecretKey32, Signature, SigningKey, VerifyingKey, verify_strict};

fn h(s: &str) -> Vec<u8> {
    hex::decode(s).expect("hex válido en el vector")
}

fn arr32(v: &[u8]) -> [u8; 32] {
    v.try_into().expect("32 bytes")
}

/// RFC 8032 §7.1, TEST 1 a TEST 3 (Ed25519 puro).
#[test]
fn rfc8032_section_7_1() {
    let cases = [
        (
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
            "",
            "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
        ),
        (
            "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb",
            "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
            "72",
            "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00",
        ),
        (
            "c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7",
            "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025",
            "af82",
            "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a",
        ),
    ];
    for (sk, pk, msg, sig) in cases {
        let key = SigningKey::from_seed(&SecretKey32::from_bytes(arr32(&h(sk))));
        assert_eq!(key.verifying_key().to_bytes().to_vec(), h(pk));
        let produced = key.sign(&h(msg));
        assert_eq!(produced.to_bytes().to_vec(), h(sig));
        verify_strict(&h(pk), &h(msg), &h(sig)).expect("firma del RFC válida");
    }
}

/// Conjunto sign.input de djb: `sk||pk:pk:msg:sig||msg:`.
#[test]
fn djb_sign_input() {
    let data = include_str!("vectors/djb_ed25519_sign.input");
    let mut count = 0;
    for line in data.lines().filter(|l| !l.is_empty()) {
        let parts: Vec<&str> = line.split(':').collect();
        let seed = &h(parts[0])[..32];
        let pk = h(parts[1]);
        let msg = h(parts[2]);
        let sig = &h(parts[3])[..64];
        let key = SigningKey::from_seed(&SecretKey32::from_bytes(arr32(seed)));
        assert_eq!(key.verifying_key().to_bytes().to_vec(), pk);
        assert_eq!(key.sign(&msg).to_bytes().as_slice(), sig);
        verify_strict(&pk, &msg, sig).expect("firma de djb válida");
        count += 1;
    }
    assert_eq!(count, 128);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Wycheproof {
    number_of_tests: usize,
    test_groups: Vec<Group>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Group {
    public_key: PublicKey,
    tests: Vec<Case>,
}

#[derive(Deserialize)]
struct PublicKey {
    pk: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    tc_id: u32,
    msg: String,
    sig: String,
    result: String,
}

/// Wycheproof: todo caso `valid` se acepta y todo `invalid` se rechaza.
/// Los casos `acceptable` (llaves de orden pequeño o mixto, que el estándar permite
/// aceptar o rechazar) se rechazan siempre: es la verificación estricta de `docs/06` §1.
#[test]
fn wycheproof_ed25519() {
    let file: Wycheproof =
        serde_json::from_str(include_str!("vectors/wycheproof_ed25519_test.json")).expect("json");
    let mut seen = 0;
    for group in &file.test_groups {
        let pk = h(&group.public_key.pk);
        for case in &group.tests {
            seen += 1;
            let ok = verify_strict(&pk, &h(&case.msg), &h(&case.sig)).is_ok();
            match case.result.as_str() {
                "valid" => assert!(ok, "tcId {} debía aceptarse", case.tc_id),
                "invalid" | "acceptable" => {
                    assert!(!ok, "tcId {} debía rechazarse", case.tc_id);
                }
                other => panic!("resultado desconocido {other}"),
            }
        }
    }
    assert_eq!(seen, file.number_of_tests);
}

/// Una firma con `s` no reducido (s + L) se rechaza aunque matemáticamente verifique.
#[test]
fn rejects_non_canonical_s() {
    let key = SigningKey::from_seed(&SecretKey32::from_bytes([7u8; 32]));
    let msg = b"social";
    let mut sig = key.sign(msg).to_bytes();
    // L = 2^252 + 27742317777372353535851937790883648493, en little-endian.
    let l: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde,
        0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10,
    ];
    let mut carry = 0u16;
    for i in 0..32 {
        let sum = u16::from(sig[32 + i]) + u16::from(l[i]) + carry;
        sig[32 + i] = (sum & 0xff) as u8;
        carry = sum >> 8;
    }
    let vk = key.verifying_key();
    assert!(vk.verify_strict(msg, &Signature::from_bytes(sig)).is_err());
}

/// La llave identidad (orden pequeño) se rechaza al construirla.
#[test]
fn rejects_small_order_public_key() {
    let mut identity = [0u8; 32];
    identity[0] = 1;
    assert!(VerifyingKey::from_bytes(&identity).is_err());
}
