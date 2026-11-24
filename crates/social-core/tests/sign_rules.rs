//! `UnsignedEvent::sign` aplica las mismas reglas que `Event::parse_and_verify`: nunca
//! firma un evento que un receptor rechazaría (revisión del PR de F0-T06).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use social_core::{ErrorCode, Event, Json, MAX_EVENT_BYTES, Tag, UnsignedEvent};
use social_crypto::{SecretKey32, SigningKey};

fn key() -> SigningKey {
    let mut seed = [0u8; 32];
    seed[0] = 7;
    SigningKey::from_seed(&SecretKey32::from_bytes(seed))
}

fn base() -> UnsignedEvent {
    let pk = key().verifying_key().to_bytes();
    let mut content = BTreeMap::new();
    content.insert("text".to_owned(), Json::from("hola"));
    UnsignedEvent {
        account: pk,
        signer: pk,
        created_at: 1_790_000_000,
        kind: "post.create".into(),
        tags: vec![],
        content,
    }
}

fn sign_code(u: UnsignedEvent) -> ErrorCode {
    u.sign(&key())
        .expect_err("sign debía rechazar el evento")
        .code
}

#[test]
fn valid_event_signs_and_verifies() {
    let event = base().sign(&key()).unwrap();
    let again = Event::parse_and_verify(&event.to_canonical_vec()).unwrap();
    assert_eq!(event, again);
}

#[test]
fn created_at_beyond_safe_range() {
    for created_at in [9_007_199_254_740_992, i64::MAX] {
        let mut u = base();
        u.created_at = created_at;
        assert_eq!(sign_code(u), ErrorCode::JsonIntRange, "{created_at}");
    }
}

#[test]
fn negative_created_at() {
    let mut u = base();
    u.created_at = -1;
    assert_eq!(sign_code(u), ErrorCode::Schema);
}

#[test]
fn content_integer_beyond_safe_range() {
    for n in [i64::MAX, i64::MIN, 9_007_199_254_740_992] {
        let mut u = base();
        u.content.insert("n".into(), Json::Int(n));
        assert_eq!(sign_code(u), ErrorCode::JsonIntRange, "{n}");
    }
}

#[test]
fn account_small_order_or_off_curve() {
    let mut small_order = [0u8; 32];
    small_order[0] = 1;
    let mut off_curve = [0u8; 32];
    off_curve[0] = 2;
    for account in [small_order, off_curve] {
        let mut u = base();
        u.account = account;
        assert_eq!(sign_code(u), ErrorCode::PublicKeyInvalid);
    }
}

#[test]
fn signer_must_match_key() {
    let mut u = base();
    u.signer = u.account;
    u.signer[0] ^= 1;
    assert_eq!(sign_code(u), ErrorCode::PublicKeyInvalid);
}

#[test]
fn content_text_of_16_kib_is_too_large() {
    let mut u = base();
    u.content
        .insert("text".into(), Json::from("a".repeat(MAX_EVENT_BYTES)));
    assert_eq!(sign_code(u), ErrorCode::EventTooLarge);
}

#[test]
fn envelope_schema_rules() {
    let mut u = base();
    u.kind = "Post.create".into();
    assert_eq!(sign_code(u), ErrorCode::Schema);

    let mut u = base();
    u.tags = vec![Tag::Topic("Vecinos".into())];
    assert_eq!(sign_code(u), ErrorCode::Schema);

    let mut u = base();
    u.tags = (0..17).map(|i| Tag::Topic(format!("t{i}"))).collect();
    assert_eq!(sign_code(u), ErrorCode::Schema);
}

#[test]
fn content_deeper_than_limit() {
    // Envoltura (1) y content (2): 126 listas llegan a 128 niveles.
    let mut value = Json::Array(vec![]);
    for _ in 1..social_core::json::MAX_DEPTH - 1 {
        value = Json::Array(vec![value]);
    }
    let mut u = base();
    u.content.insert("deep".into(), value);
    assert_eq!(sign_code(u), ErrorCode::JsonInvalid);
}
