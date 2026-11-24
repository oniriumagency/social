//! Esquemas JSON (draft 2020-12) de la envoltura y del contenido de cada tipo, generados
//! desde la misma tabla que usa la validación (`crate::kinds::KINDS`).
//!
//! `pnpm spec:gen` los escribe en `spec/schemas/`; `pnpm check` falla si lo guardado no
//! coincide con lo generado. Son documentación ejecutable para otras implementaciones:
//! la validación normativa es la de `social-core` y sus vectores dorados.
//!
//! JSON Schema no puede expresar todo (grafemas, NFC, reglas entre campos). Esas reglas
//! van en las claves de extensión `x-max-graphemes`, `x-nfc`, `x-signer` y `x-rules`.

use std::collections::BTreeMap;

use crate::event::{MAX_EVENT_BYTES, MAX_KIND_BYTES, MAX_TAGS, MAX_TOPIC_CHARS};
use crate::json::MAX_SAFE_INT;
use crate::kinds::{FieldType, KINDS, KindSpec};
use crate::{Json, PROTOCOL_VERSION};

const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";
const BASE_ID: &str = "https://social.invalid/spec/schemas/";

const HEX32: &str = "^[0-9a-f]{64}$";
const HEX64: &str = "^[0-9a-f]{128}$";
const KIND: &str = "^[a-z][a-z0-9_]*(\\.[a-z][a-z0-9_]*)+$";
const CODE: &str = "^[a-z][a-z0-9_]{0,31}$";
const CID: &str = "^b3_[a-z2-7]{51}[aq]$";
const ZONE: &str = "^z_[0-9a-hjkmnp-tv-z]{12}$";
const LANG: &str = "^[a-z]{2,3}(-[A-Za-z0-9]{2,8})*$";

fn obj(pairs: Vec<(&str, Json)>) -> Json {
    Json::Object(pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}

fn int(n: usize) -> Json {
    Json::Int(i64::try_from(n).unwrap_or(MAX_SAFE_INT))
}

fn string_pattern(pattern: &str) -> Json {
    obj(vec![("type", "string".into()), ("pattern", pattern.into())])
}

fn integer(min: i64, max: i64) -> Json {
    obj(vec![
        ("type", "integer".into()),
        ("minimum", Json::Int(min)),
        ("maximum", Json::Int(max)),
    ])
}

fn field_schema(ty: FieldType) -> Json {
    match ty {
        FieldType::Hex32 => string_pattern(HEX32),
        FieldType::Int => integer(-MAX_SAFE_INT, MAX_SAFE_INT),
        FieldType::NonNegInt => integer(0, MAX_SAFE_INT),
        FieldType::Level => integer(1, 3),
        FieldType::Bool => obj(vec![("type", "boolean".into())]),
        FieldType::Text(max) => obj(vec![
            ("type", "string".into()),
            ("x-max-graphemes", int(max)),
            ("x-nfc", Json::Bool(true)),
        ]),
        FieldType::Enum(values) => obj(vec![(
            "enum",
            Json::Array(values.iter().map(|v| (*v).into()).collect()),
        )]),
        FieldType::Code => string_pattern(CODE),
        FieldType::Cid => string_pattern(CID),
        FieldType::Lang => obj(vec![
            ("type", "string".into()),
            ("pattern", LANG.into()),
            ("maxLength", Json::Int(35)),
        ]),
        FieldType::Ascii(max) => obj(vec![
            ("type", "string".into()),
            ("minLength", Json::Int(1)),
            ("maxLength", int(max)),
            ("pattern", "^[\\u0020-\\u007e]*$".into()),
        ]),
        FieldType::Zone => obj(vec![
            ("type", "object".into()),
            (
                "properties",
                obj(vec![
                    ("id", string_pattern(ZONE)),
                    (
                        "mode",
                        obj(vec![(
                            "enum",
                            Json::Array(vec!["home".into(), "live".into(), "remote".into()]),
                        )]),
                    ),
                ]),
            ),
            ("required", Json::Array(vec!["id".into(), "mode".into()])),
            ("additionalProperties", Json::Bool(false)),
        ]),
        FieldType::CidList(max) => obj(vec![
            ("type", "array".into()),
            ("items", string_pattern(CID)),
            ("maxItems", int(max)),
        ]),
        FieldType::KindList(max) => obj(vec![
            ("type", "array".into()),
            (
                "items",
                obj(vec![
                    ("type", "string".into()),
                    ("pattern", KIND.into()),
                    ("maxLength", int(MAX_KIND_BYTES)),
                    // Solo tipos que un dispositivo puede firmar (ADR-021).
                    (
                        "not",
                        obj(vec![(
                            "enum",
                            Json::Array(
                                crate::kinds::KINDS
                                    .iter()
                                    .filter(|k| !k.signer.delegable())
                                    .map(|k| k.kind.into())
                                    .collect(),
                            ),
                        )]),
                    ),
                ]),
            ),
            ("minItems", Json::Int(1)),
            ("maxItems", int(max)),
            ("uniqueItems", Json::Bool(true)),
        ]),
    }
}

fn kind_schema(spec: &KindSpec) -> Json {
    let properties: BTreeMap<String, Json> = spec
        .fields
        .iter()
        .map(|f| (f.name.to_owned(), field_schema(f.ty)))
        .collect();
    let required: Vec<Json> = spec
        .fields
        .iter()
        .filter(|f| f.required)
        .map(|f| f.name.into())
        .collect();
    obj(vec![
        ("$schema", DRAFT.into()),
        (
            "$id",
            format!("{BASE_ID}kinds/{}.schema.json", spec.kind).into(),
        ),
        ("title", format!("Contenido de {}", spec.kind).into()),
        ("type", "object".into()),
        ("properties", Json::Object(properties)),
        ("required", Json::Array(required)),
        ("additionalProperties", Json::Bool(false)),
        ("x-signer", spec.signer.as_str().into()),
        ("x-rules", spec.rules.into()),
    ])
}

fn envelope_schema() -> Json {
    let tag = obj(vec![(
        "oneOf",
        Json::Array(vec![
            obj(vec![
                ("type", "array".into()),
                (
                    "prefixItems",
                    Json::Array(vec![
                        obj(vec![("const", "t".into())]),
                        obj(vec![
                            ("type", "string".into()),
                            ("minLength", Json::Int(1)),
                            ("x-max-chars", int(MAX_TOPIC_CHARS)),
                            ("x-lowercase", Json::Bool(true)),
                            ("x-nfc", Json::Bool(true)),
                        ]),
                    ]),
                ),
                ("items", Json::Bool(false)),
                ("minItems", Json::Int(2)),
            ]),
            obj(vec![
                ("type", "array".into()),
                (
                    "prefixItems",
                    Json::Array(vec![
                        obj(vec![("const", "p".into())]),
                        string_pattern(HEX32),
                    ]),
                ),
                ("items", Json::Bool(false)),
                ("minItems", Json::Int(2)),
            ]),
        ]),
    )]);
    let fields = [
        "v",
        "id",
        "account",
        "signer",
        "created_at",
        "kind",
        "tags",
        "content",
        "sig",
    ];
    obj(vec![
        ("$schema", DRAFT.into()),
        ("$id", format!("{BASE_ID}envelope.schema.json").into()),
        ("title", "Envoltura de evento (docs/06 §3)".into()),
        ("type", "object".into()),
        (
            "properties",
            obj(vec![
                ("v", obj(vec![("const", Json::Int(PROTOCOL_VERSION))])),
                ("id", string_pattern(HEX32)),
                ("account", string_pattern(HEX32)),
                ("signer", string_pattern(HEX32)),
                ("created_at", integer(0, MAX_SAFE_INT)),
                (
                    "kind",
                    obj(vec![
                        ("type", "string".into()),
                        ("pattern", KIND.into()),
                        ("maxLength", int(MAX_KIND_BYTES)),
                    ]),
                ),
                (
                    "tags",
                    obj(vec![
                        ("type", "array".into()),
                        ("items", tag),
                        ("maxItems", int(MAX_TAGS)),
                    ]),
                ),
                ("content", obj(vec![("type", "object".into())])),
                ("sig", string_pattern(HEX64)),
            ]),
        ),
        (
            "required",
            Json::Array(fields.iter().map(|f| (*f).into()).collect()),
        ),
        ("additionalProperties", Json::Bool(false)),
        ("x-max-bytes", int(MAX_EVENT_BYTES)),
        (
            "x-forbidden",
            "null, números decimales o exponenciales y claves repetidas".into(),
        ),
    ])
}

/// Todos los esquemas: (ruta relativa dentro de `spec/schemas/`, contenido con salto final).
#[must_use]
pub fn generate_schemas() -> Vec<(String, String)> {
    let render = |json: &Json| {
        let mut s = json.to_canonical_string();
        s.push('\n');
        s
    };
    let mut out = vec![(
        "envelope.schema.json".to_owned(),
        render(&envelope_schema()),
    )];
    for spec in KINDS {
        out.push((
            format!("kinds/{}.schema.json", spec.kind),
            render(&kind_schema(spec)),
        ));
    }
    out
}
