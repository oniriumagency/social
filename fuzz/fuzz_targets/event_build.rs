//! Construye un evento a partir de la entrada (tipo, contenido, etiquetas, cuenta,
//! firmante y fecha), lo firma con las llaves de prueba de `validation.json` y lo pasa
//! por la validación universal con su contexto. Así el fuzzer llega a los pasos 5 y 6
//! (papeles de firma, delegaciones, atestación y nivel) con eventos nuevos.
//!
//! Propiedades: nunca hay pánico; lo que `validate_event` acepta lo acepta también
//! `validate_parsed`; y todo evento aceptado respeta los papeles de firma: un tipo no
//! delegable lo firma la cuenta o el registro, y un dispositivo solo firma con una
//! delegación que cubre el tipo y está vigente y sin revocar en `created_at`.
#![no_main]

use std::collections::BTreeMap;

use libfuzzer_sys::arbitrary::{Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use social_core::kinds::{FieldType, KINDS, spec};
use social_core::vectors::{CID, T0, TEST_KEYS, ZONE, sign_with_test_key, test_public_key};
use social_core::{Event, Json, Tag, UnsignedEvent, ValidationContext, validate_parsed};

#[path = "context.rs"]
mod context;

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn key_index(u: &mut Unstructured<'_>) -> Result<u8> {
    u.int_in_range(1..=TEST_KEYS)
}

fn hex32(u: &mut Unstructured<'_>) -> Result<String> {
    Ok(if u.ratio(3, 4)? {
        to_hex(&test_public_key(key_index(u)?))
    } else {
        to_hex(&u.arbitrary::<[u8; 32]>()?)
    })
}

fn instant(u: &mut Unstructured<'_>) -> Result<i64> {
    Ok(match u.int_in_range(0..=3)? {
        0 => T0 + i64::from(u.arbitrary::<i32>()?),
        1 => T0 + i64::from(u.int_in_range(-200_000..=200_000)?) * 60,
        2 => u.arbitrary::<i64>()?,
        _ => *u.choose(&[0, T0, T0 + 3 * 86_400, T0 + 90 * 86_400, -1])?,
    })
}

fn text(u: &mut Unstructured<'_>) -> Result<String> {
    Ok(if u.ratio(1, 8)? {
        "ñ".repeat(u.int_in_range(0..=600)?)
    } else {
        u.arbitrary::<String>()?
    })
}

fn cid(u: &mut Unstructured<'_>) -> Result<String> {
    Ok(if u.ratio(3, 4)? {
        CID.to_owned()
    } else {
        let mut s = String::from("b3_");
        for _ in 0..u.int_in_range(50..=53)? {
            s.push(*u.choose(&['a', 'q', 'r', 'z', '2', '7', 'A', '8'])?);
        }
        s
    })
}

fn kind_name(u: &mut Unstructured<'_>) -> Result<String> {
    Ok(if u.ratio(7, 8)? {
        u.choose(KINDS)?.kind.to_owned()
    } else {
        (*u.choose(&["test.unknown", "post.create_v2", "Post.create", "x"])?).to_owned()
    })
}

fn value(u: &mut Unstructured<'_>, ty: FieldType) -> Result<Json> {
    if u.ratio(1, 16)? {
        // Tipo equivocado.
        return Ok(match u.int_in_range(0..=2)? {
            0 => Json::Bool(u.arbitrary()?),
            1 => Json::Int(u.int_in_range(-9_007_199_254_740_991..=9_007_199_254_740_991)?),
            _ => Json::String(u.arbitrary()?),
        });
    }
    Ok(match ty {
        FieldType::Hex32 => Json::String(hex32(u)?),
        FieldType::Int | FieldType::NonNegInt => {
            Json::Int(instant(u)?.clamp(-9_007_199_254_740_991, 9_007_199_254_740_991))
        }
        FieldType::Level => Json::Int(u.int_in_range(0..=4)?),
        FieldType::Bool => Json::Bool(u.arbitrary()?),
        FieldType::Text(_) | FieldType::Ascii(_) => Json::String(text(u)?),
        FieldType::Enum(options) => Json::String(if u.ratio(7, 8)? {
            (*u.choose(options)?).to_owned()
        } else {
            u.arbitrary()?
        }),
        FieldType::Code => Json::String(if u.ratio(3, 4)? {
            (*u.choose(&["spam", "acoso", "app_open", "a"])?).to_owned()
        } else {
            u.arbitrary()?
        }),
        FieldType::Cid => Json::String(cid(u)?),
        FieldType::Lang => {
            Json::String((*u.choose(&["es", "es-CO", "zh-Hant", "ES", ""])?).to_owned())
        }
        FieldType::Zone => {
            let mut zone = BTreeMap::new();
            zone.insert(
                "id".to_owned(),
                Json::String(if u.ratio(7, 8)? {
                    ZONE.to_owned()
                } else {
                    u.arbitrary()?
                }),
            );
            zone.insert(
                "mode".to_owned(),
                Json::String((*u.choose(&["home", "live", "remote", "away"])?).to_owned()),
            );
            Json::Object(zone)
        }
        FieldType::CidList(_) => {
            let n = u.int_in_range(0..=6)?;
            Json::Array(
                (0..n)
                    .map(|_| cid(u).map(Json::String))
                    .collect::<Result<_>>()?,
            )
        }
        FieldType::KindList(_) => {
            let n = u.int_in_range(0..=4)?;
            Json::Array(
                (0..n)
                    .map(|_| kind_name(u).map(Json::String))
                    .collect::<Result<_>>()?,
            )
        }
    })
}

fn build(u: &mut Unstructured<'_>) -> Result<(UnsignedEvent, u8)> {
    let kind = kind_name(u)?;
    let mut content = BTreeMap::new();
    if let Some(spec) = KINDS.iter().find(|k| k.kind == kind) {
        for field in spec.fields {
            let keep = if field.required {
                u.ratio(15, 16)?
            } else {
                u.ratio(1, 2)?
            };
            if keep {
                content.insert(field.name.to_owned(), value(u, field.ty)?);
            }
        }
    }
    if u.ratio(1, 8)? {
        content.insert(u.arbitrary()?, Json::Bool(true));
    }
    let signer = key_index(u)?;
    let account = if u.ratio(1, 2)? {
        signer
    } else {
        key_index(u)?
    };
    let mut tags = Vec::new();
    for _ in 0..u.int_in_range(0..=3)? {
        tags.push(if u.arbitrary()? {
            Tag::Topic((*u.choose(&["vecinos", "año", "a", "bogotá"])?).to_owned())
        } else {
            Tag::Mention(test_public_key(key_index(u)?))
        });
    }
    let unsigned = UnsignedEvent {
        account: test_public_key(account),
        signer: test_public_key(signer),
        created_at: instant(u)?,
        kind,
        tags,
        content,
    };
    Ok((unsigned, signer))
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok((unsigned, signer)) = build(&mut u) else {
        return;
    };
    // `sign` aplica los pasos 1 a 4: lo que no pasa la envoltura no llega a firmarse.
    let Ok(event) = sign_with_test_key(unsigned, signer) else {
        return;
    };
    let wire = event.to_canonical_vec();
    let ctx = context::context();
    if social_core::validate_event(&wire, ctx).is_ok() {
        let parsed = Event::parse_and_verify(&wire).expect("aceptado por validate_event");
        assert!(
            validate_parsed(&parsed, ctx).is_ok(),
            "validate_event acepta y validate_parsed rechaza"
        );
        let (account, signer, at) = (parsed.account(), parsed.signer(), parsed.created_at());
        let delegable = spec(parsed.kind()).is_none_or(|k| k.signer.delegable());
        if !delegable {
            assert!(
                signer == account || ctx.is_registry_key(signer),
                "tipo no delegable aceptado con la firma de un dispositivo"
            );
        } else if signer != account {
            let grant = ctx
                .device_grant(account, signer, at)
                .expect("dispositivo aceptado sin delegación");
            assert!(
                grant.caps.iter().any(|c| c == parsed.kind())
                    && grant.granted_at <= at
                    && at < grant.expires_at
                    && grant.revoked_at.is_none_or(|r| at < r),
                "dispositivo aceptado con una delegación que no cubre el evento"
            );
        }
    }
});
