//! Vectores dorados de la envoltura (`docs/14` §3), su generador y su ejecutor.
//!
//! - [`generate_envelope_suite`] construye `spec/vectors/envelope.json` de forma
//!   determinista (`pnpm vectors:gen`). Cambiar un vector existente es un cambio de
//!   protocolo y exige ADR.
//! - [`run_suite`] comprueba un archivo de vectores contra este núcleo. Es el mismo
//!   ejecutor en nativo y en WASM (`pnpm test:vectors`).
//!
//! Las llaves de los vectores son públicas y solo de prueba: nunca se usan fuera de aquí.

use std::collections::BTreeMap;

use social_crypto::{SecretKey32, SigningKey};

use crate::{CoreError, ErrorCode, Event, Json, Tag, UnsignedEvent};

pub use crate::vectors_validation::{MemoryContext, generate_validation_suite};

/// Resultado de ejecutar un archivo de vectores.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SuiteReport {
    /// Vectores que pasaron.
    pub passed: usize,
    /// Descripción de cada vector que falló.
    pub failures: Vec<String>,
}

impl SuiteReport {
    /// `true` si no hubo fallos y se ejecutó al menos un vector.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.failures.is_empty() && self.passed > 0
    }
}

/// Semilla pública de prueba número `n` (solo para vectores).
fn test_key(n: u8) -> SigningKey {
    let mut seed = [0u8; 32];
    seed[0] = n;
    seed[31] = 0x5c;
    SigningKey::from_seed(&SecretKey32::from_bytes(seed))
}

fn test_seed_hex(n: u8) -> String {
    let mut seed = [0u8; 32];
    seed[0] = n;
    seed[31] = 0x5c;
    hex::encode(seed)
}

fn obj(pairs: &[(&str, Json)]) -> BTreeMap<String, Json> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.clone()))
        .collect()
}

fn zone(id: &str, mode: &str) -> Json {
    Json::Object(obj(&[("id", id.into()), ("mode", mode.into())]))
}

struct ValidCase {
    name: &'static str,
    key: u8,
    unsigned: UnsignedEvent,
}

fn valid_cases() -> Vec<ValidCase> {
    let root = test_key(1).verifying_key().to_bytes();
    let device = test_key(2).verifying_key().to_bytes();
    let other = test_key(3).verifying_key().to_bytes();
    vec![
        ValidCase {
            name: "post.create mínimo firmado por la raíz",
            key: 1,
            unsigned: UnsignedEvent {
                account: root,
                signer: root,
                created_at: 1_790_000_000,
                kind: "post.create".into(),
                tags: vec![],
                content: obj(&[
                    ("text", "hola".into()),
                    ("lang", "es".into()),
                    ("zone", zone("z_3k9dq7m2x4pa", "home")),
                ]),
            },
        },
        ValidCase {
            name: "post.create de dispositivo con etiquetas t y p",
            key: 2,
            unsigned: UnsignedEvent {
                account: root,
                signer: device,
                created_at: 1_790_000_123,
                kind: "post.create".into(),
                tags: vec![Tag::Topic("vecinos".into()), Tag::Mention(other)],
                content: obj(&[
                    ("text", "Hola barrio #vecinos".into()),
                    ("lang", "es".into()),
                    ("zone", zone("z_3k9dq7m2x4pa", "live")),
                ]),
            },
        },
        ValidCase {
            name: "texto con acentos, emoji y caracteres que se escapan",
            key: 1,
            unsigned: UnsignedEvent {
                account: root,
                signer: root,
                created_at: 1_790_000_200,
                kind: "post.create".into(),
                tags: vec![Tag::Topic("año".into())],
                content: obj(&[
                    (
                        "text",
                        "Ñandú «café» 😀 \"comillas\" \\ barra\nnueva línea\ttab \u{1} control"
                            .into(),
                    ),
                    ("lang", "es".into()),
                    ("zone", zone("z_3k9dq7m2x4pa", "home")),
                ]),
            },
        },
        ValidCase {
            name: "tipo desconocido con claves que se ordenan por UTF-16",
            key: 1,
            unsigned: UnsignedEvent {
                account: root,
                signer: root,
                created_at: 0,
                kind: "test.unknown_kind".into(),
                tags: vec![],
                content: obj(&[
                    ("\u{20ac}", "euro".into()),
                    ("\r", "retorno".into()),
                    ("\u{fb33}", "dalet".into()),
                    ("1", "uno".into()),
                    ("\u{1f600}", "emoji".into()),
                    ("\u{80}", "control".into()),
                    ("\u{f6}", "o diéresis".into()),
                    ("max", Json::Int(crate::json::MAX_SAFE_INT)),
                    ("min", Json::Int(-crate::json::MAX_SAFE_INT)),
                    ("bool", Json::Bool(false)),
                    (
                        "nested",
                        Json::Array(vec![Json::Array(vec![]), Json::Object(BTreeMap::new())]),
                    ),
                ]),
            },
        },
    ]
}

fn base_event() -> Result<Event, CoreError> {
    valid_cases()
        .into_iter()
        .next()
        .ok_or_else(|| CoreError::new(ErrorCode::Schema, "sin casos"))?
        .unsigned
        .sign(&test_key(1))
}

fn base_string() -> Result<String, CoreError> {
    Ok(base_event()?.to_json().to_canonical_string())
}

fn replace_once(s: &str, from: &str, to: &str) -> Result<String, CoreError> {
    if !s.contains(from) {
        return Err(CoreError::new(
            ErrorCode::Schema,
            format!("no aparece {from}"),
        ));
    }
    Ok(s.replacen(from, to, 1))
}

/// Suma L (orden del grupo) a `s`: la firma sigue verificando en modo laxo, no en estricto.
fn non_canonical_sig(sig: &[u8; 64]) -> [u8; 64] {
    const L: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde,
        0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10,
    ];
    let mut out = *sig;
    let mut carry = 0u16;
    for i in 0..32 {
        let sum = u16::from(out[32 + i]) + u16::from(L[i]) + carry;
        out[32 + i] = (sum & 0xff) as u8;
        carry = sum >> 8;
    }
    out
}

fn invalid_cases() -> Result<Vec<(&'static str, String, ErrorCode)>, CoreError> {
    let base = base_event()?;
    let s = base_string()?;
    let sig_hex = hex::encode(base.sig());
    let id_hex = hex::encode(base.id());
    let root = hex::encode(test_key(1).verifying_key().to_bytes());

    let resign = |mut u: UnsignedEvent| -> Result<String, CoreError> {
        u.signer = u.account;
        Ok(u.sign(&test_key(1))?.to_json().to_canonical_string())
    };

    let mut big = base.unsigned().clone();
    big.content
        .insert("text".into(), "a".repeat(crate::MAX_EVENT_BYTES).into());
    let big = resign(big)?;

    let mut flipped = base.sig().to_owned();
    flipped[0] ^= 1;

    let other_sig = test_key(3).sign(base.id()).to_bytes();

    let mut small_order = [0u8; 32];
    small_order[0] = 1;

    let mut seventeen = String::from("[");
    for i in 0..17 {
        if i > 0 {
            seventeen.push(',');
        }
        seventeen.push_str(&format!("[\"t\",\"t{i}\"]"));
    }
    seventeen.push(']');

    Ok(vec![
        ("mayor de 16 KiB", big, ErrorCode::EventTooLarge),
        ("no es JSON", "{\"v\":1,".into(), ErrorCode::JsonInvalid),
        (
            "created_at decimal",
            replace_once(&s, "1790000000", "1790000000.0")?,
            ErrorCode::JsonFloat,
        ),
        (
            "created_at en notación exponencial",
            replace_once(&s, "1790000000", "1.79e9")?,
            ErrorCode::JsonFloat,
        ),
        (
            "null en el contenido",
            replace_once(&s, "\"lang\":\"es\"", "\"lang\":null")?,
            ErrorCode::JsonNull,
        ),
        (
            "entero fuera de rango",
            replace_once(&s, "1790000000", "9007199254740992")?,
            ErrorCode::JsonIntRange,
        ),
        (
            "clave repetida en la envoltura",
            replace_once(&s, "\"v\":1", "\"v\":1,\"v\":1")?,
            ErrorCode::JsonDuplicateKey,
        ),
        (
            "campo extra en la envoltura",
            replace_once(&s, "\"v\":1", "\"v\":1,\"extra\":true")?,
            ErrorCode::UnknownField,
        ),
        (
            "falta un campo",
            replace_once(&s, ",\"tags\":[]", "")?,
            ErrorCode::Schema,
        ),
        (
            "versión 2",
            replace_once(&s, "\"v\":1", "\"v\":2")?,
            ErrorCode::VersionUnsupported,
        ),
        (
            "id en mayúsculas",
            replace_once(&s, &id_hex, &id_hex.to_uppercase())?,
            ErrorCode::Schema,
        ),
        (
            "content no es un objeto",
            replace_once(&s, "\"content\":{", "\"content\":[{")?.replacen(
                "},\"created_at\"",
                "}],\"created_at\"",
                1,
            ),
            ErrorCode::Schema,
        ),
        (
            "kind con mayúsculas",
            replace_once(&s, "post.create", "Post.create")?,
            ErrorCode::Schema,
        ),
        (
            "etiqueta desconocida",
            replace_once(&s, "\"tags\":[]", "\"tags\":[[\"x\",\"y\"]]")?,
            ErrorCode::Schema,
        ),
        (
            "etiqueta t en mayúsculas",
            replace_once(&s, "\"tags\":[]", "\"tags\":[[\"t\",\"Vecinos\"]]")?,
            ErrorCode::Schema,
        ),
        (
            "diecisiete etiquetas",
            replace_once(&s, "\"tags\":[]", &format!("\"tags\":{seventeen}"))?,
            ErrorCode::Schema,
        ),
        (
            "signer de orden pequeño",
            s.replace(
                &format!("\"signer\":\"{root}\""),
                &format!("\"signer\":\"{}\"", hex::encode(small_order)),
            ),
            ErrorCode::PublicKeyInvalid,
        ),
        (
            "contenido alterado después de firmar",
            replace_once(&s, "\"text\":\"hola\"", "\"text\":\"adiós\"")?,
            ErrorCode::IdMismatch,
        ),
        (
            "firma con un bit cambiado",
            replace_once(&s, &sig_hex, &hex::encode(flipped))?,
            ErrorCode::SigInvalid,
        ),
        (
            "firma no canónica (s + L)",
            replace_once(&s, &sig_hex, &hex::encode(non_canonical_sig(base.sig())))?,
            ErrorCode::SigInvalid,
        ),
        (
            "firmado por otra llave",
            replace_once(&s, &sig_hex, &hex::encode(other_sig))?,
            ErrorCode::SigInvalid,
        ),
    ])
}

/// Genera el archivo `spec/vectors/envelope.json` (JSON canónico más salto de línea final).
///
/// # Errors
/// Solo si los casos definidos aquí son incoherentes.
pub fn generate_envelope_suite() -> Result<String, CoreError> {
    let mut valid = Vec::new();
    for case in valid_cases() {
        let payload = case.unsigned.payload();
        let event = case.unsigned.clone().sign(&test_key(case.key))?;
        let u = &case.unsigned;
        let input = obj(&[
            ("v", Json::Int(crate::PROTOCOL_VERSION)),
            ("account", hex::encode(u.account).into()),
            ("signer", hex::encode(u.signer).into()),
            ("created_at", Json::Int(u.created_at)),
            ("kind", u.kind.as_str().into()),
            (
                "tags",
                event
                    .to_json()
                    .as_object()
                    .and_then(|o| o.get("tags"))
                    .cloned()
                    .unwrap_or(Json::Array(vec![])),
            ),
            ("content", Json::Object(u.content.clone())),
        ]);
        let expect = obj(&[
            ("payload_hex", hex::encode(&payload).into()),
            ("id", hex::encode(event.id()).into()),
            ("sig", hex::encode(event.sig()).into()),
            ("event", event.to_json().to_canonical_string().into()),
            ("valid", Json::Bool(true)),
        ]);
        valid.push(Json::Object(obj(&[
            ("name", case.name.into()),
            ("secret_key", test_seed_hex(case.key).into()),
            ("input", Json::Object(input)),
            ("expect", Json::Object(expect)),
        ])));
    }
    let invalid = invalid_cases()?
        .into_iter()
        .map(|(name, raw, code)| {
            Json::Object(obj(&[
                ("name", name.into()),
                ("event", raw.into()),
                (
                    "expect",
                    Json::Object(obj(&[
                        ("valid", Json::Bool(false)),
                        ("error", code.as_str().into()),
                    ])),
                ),
            ]))
        })
        .collect();
    let suite = Json::Object(obj(&[
        ("suite", "envelope".into()),
        ("protocol", Json::Int(crate::PROTOCOL_VERSION)),
        (
            "note",
            "Generado por `pnpm vectors:gen`. Las llaves son públicas y solo de prueba. Cambiar un vector exige ADR (docs/14 §3).".into(),
        ),
        ("valid", Json::Array(valid)),
        ("invalid", Json::Array(invalid)),
    ]));
    let mut text = suite.to_canonical_string();
    text.push('\n');
    Ok(text)
}

/// Ejecuta un archivo de vectores (cualquier suite) contra este núcleo.
#[must_use]
pub fn run_suite(text: &str) -> SuiteReport {
    let mut report = SuiteReport::default();
    let suite = match Json::parse(text.as_bytes()) {
        Ok(Json::Object(o)) => o,
        _ => {
            report
                .failures
                .push("el archivo no es un objeto JSON válido".into());
            return report;
        }
    };
    if suite.get("suite").and_then(Json::as_str) == Some("validation") {
        crate::vectors_validation::run(&suite, &mut report);
        return report;
    }
    let list = |name: &str| {
        suite
            .get(name)
            .and_then(Json::as_array)
            .map(<[Json]>::to_vec)
            .unwrap_or_default()
    };
    for vector in list("valid") {
        match check_valid(&vector) {
            Ok(()) => report.passed += 1,
            Err(e) => report.failures.push(e),
        }
    }
    for vector in list("invalid") {
        match check_invalid(&vector) {
            Ok(()) => report.passed += 1,
            Err(e) => report.failures.push(e),
        }
    }
    report
}

fn get<'a>(v: &'a Json, path: &[&str]) -> Option<&'a Json> {
    path.iter().try_fold(v, |acc, k| acc.as_object()?.get(*k))
}

fn get_str<'a>(v: &'a Json, path: &[&str]) -> Result<&'a str, String> {
    get(v, path)
        .and_then(Json::as_str)
        .ok_or_else(|| format!("falta {}", path.join(".")))
}

fn check_valid(vector: &Json) -> Result<(), String> {
    let name = get_str(vector, &["name"])?;
    let fail = |what: &str| format!("[válido] {name}: {what}");
    let seed = hex::decode(get_str(vector, &["secret_key"])?).map_err(|_| fail("secret_key"))?;
    let seed: [u8; 32] = seed.try_into().map_err(|_| fail("secret_key"))?;
    let key = SigningKey::from_seed(&SecretKey32::from_bytes(seed));

    let input = get(vector, &["input"]).ok_or_else(|| fail("falta input"))?;
    let mut probe = input.as_object().cloned().ok_or_else(|| fail("input"))?;
    // Se reconstruye el evento a partir de `input`, sin id ni firma, y se firma de nuevo.
    probe.insert("id".into(), "00".repeat(32).into());
    probe.insert("sig".into(), "00".repeat(64).into());
    let unsigned = Event::from_json_unchecked(&Json::Object(probe))
        .map_err(|e| fail(&format!("input no cumple el esquema: {e}")))?;

    let payload_hex = hex::encode(unsigned.payload());
    if payload_hex != get_str(vector, &["expect", "payload_hex"])? {
        return Err(fail("payload distinto"));
    }
    let event = unsigned.sign(&key).map_err(|e| fail(&e.to_string()))?;
    if hex::encode(event.id()) != get_str(vector, &["expect", "id"])? {
        return Err(fail("id distinto"));
    }
    if hex::encode(event.sig()) != get_str(vector, &["expect", "sig"])? {
        return Err(fail("firma distinta"));
    }
    let wire = get_str(vector, &["expect", "event"])?;
    if event.to_json().to_canonical_string() != wire {
        return Err(fail("serialización canónica distinta"));
    }
    Event::parse_and_verify(wire.as_bytes()).map_err(|e| fail(&format!("rechazado: {e}")))?;
    Ok(())
}

fn check_invalid(vector: &Json) -> Result<(), String> {
    let name = get_str(vector, &["name"])?;
    let raw = get_str(vector, &["event"])?;
    let expected = get_str(vector, &["expect", "error"])?;
    match Event::parse_and_verify(raw.as_bytes()) {
        Ok(_) => Err(format!(
            "[inválido] {name}: se aceptó; se esperaba {expected}"
        )),
        Err(e) if e.code.as_str() == expected => Ok(()),
        Err(e) => Err(format!(
            "[inválido] {name}: se esperaba {expected} y se obtuvo {}",
            e.code
        )),
    }
}
