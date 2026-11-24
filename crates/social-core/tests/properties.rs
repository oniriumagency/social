//! Pruebas de propiedades de la envoltura (`docs/14` §4, protocolo).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use proptest::prelude::*;
use social_core::{Event, Json, Tag, UnsignedEvent, json::MAX_SAFE_INT};
use social_crypto::{SecretKey32, SigningKey};

fn json_leaf() -> impl Strategy<Value = Json> {
    prop_oneof![
        any::<bool>().prop_map(Json::Bool),
        (-MAX_SAFE_INT..=MAX_SAFE_INT).prop_map(Json::Int),
        any::<String>().prop_map(Json::String),
    ]
}

fn json_value() -> impl Strategy<Value = Json> {
    json_leaf().prop_recursive(4, 32, 6, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..6).prop_map(Json::Array),
            prop::collection::btree_map(any::<String>(), inner, 0..6).prop_map(Json::Object),
        ]
    })
}

fn content() -> impl Strategy<Value = BTreeMap<String, Json>> {
    prop::collection::btree_map(any::<String>(), json_value(), 0..6)
}

fn tags() -> impl Strategy<Value = Vec<Tag>> {
    prop::collection::vec(
        prop_oneof![
            "[a-zñ0-9_]{1,20}".prop_map(Tag::Topic),
            any::<[u8; 32]>().prop_map(|seed| Tag::Mention(
                SigningKey::from_seed(&SecretKey32::from_bytes(seed))
                    .verifying_key()
                    .to_bytes()
            )),
        ],
        0..5,
    )
}

fn signed_event() -> impl Strategy<Value = Event> {
    (
        any::<[u8; 32]>(),
        0i64..=MAX_SAFE_INT,
        "[a-z]{1,8}\\.[a-z][a-z_]{0,7}",
        tags(),
        content(),
    )
        .prop_map(|(seed, created_at, kind, tags, content)| {
            let key = SigningKey::from_seed(&SecretKey32::from_bytes(seed));
            let pk = key.verifying_key().to_bytes();
            UnsignedEvent {
                account: pk,
                signer: pk,
                created_at,
                kind,
                tags,
                content,
            }
            .sign(&key)
            .unwrap()
        })
        .prop_filter("cabe en 16 KiB", |e| {
            e.to_canonical_vec().len() <= social_core::MAX_EVENT_BYTES
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// verificar(firmar(e)) es verdadero para todo evento válido generado, y la ida y
    /// vuelta por la red conserva el evento.
    #[test]
    fn sign_then_verify_round_trips(event in signed_event()) {
        let wire = event.to_canonical_vec();
        let parsed = Event::parse_and_verify(&wire).unwrap();
        prop_assert_eq!(parsed, event);
    }

    /// Cambiar cualquier byte de un evento firmado invalida la verificación.
    #[test]
    fn any_byte_change_is_rejected(event in signed_event(), pos in any::<prop::sample::Index>(), delta in 1u8..=255) {
        let mut wire = event.to_canonical_vec();
        let i = pos.index(wire.len());
        wire[i] = wire[i].wrapping_add(delta);
        prop_assert!(Event::parse_and_verify(&wire).is_err());
    }

    /// Canonicalizar dos veces da el mismo resultado.
    #[test]
    fn canonicalization_is_idempotent(value in json_value()) {
        let once = value.to_canonical_string();
        let twice = Json::parse(once.as_bytes()).unwrap().to_canonical_string();
        prop_assert_eq!(&once, &twice);
        prop_assert_eq!(Json::parse(once.as_bytes()).unwrap(), value);
    }
}
