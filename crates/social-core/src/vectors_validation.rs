//! Vectores dorados de la validación universal y de los tipos (`spec/vectors/validation.json`).
//!
//! El archivo incluye el contexto (llaves del registro y de operadores, delegaciones y
//! atestaciones) para que cualquier implementación lo cargue en memoria y obtenga los
//! mismos resultados. Hay un vector inválido por cada regla de rechazo.

use std::collections::BTreeMap;

use social_crypto::{SecretKey32, SigningKey};

use crate::kinds::{DAY, ROTATE_DELAY};
use crate::vectors::SuiteReport;
use crate::{
    Attestation, CoreError, DeviceGrant, ErrorCode, Json, Tag, UnsignedEvent, ValidationContext,
    validate_event,
};

/// Contexto de validación en memoria. Lo usan los vectores, las pruebas y el cliente.
#[derive(Debug, Default, Clone)]
pub struct MemoryContext {
    /// Llaves del registro.
    pub registry_keys: Vec<[u8; 32]>,
    /// Llaves de operadores.
    pub operator_keys: Vec<[u8; 32]>,
    /// Delegaciones por (cuenta, dispositivo).
    pub grants: BTreeMap<([u8; 32], [u8; 32]), DeviceGrant>,
    /// Atestaciones por cuenta.
    pub attestations: BTreeMap<[u8; 32], Vec<Attestation>>,
}

impl ValidationContext for MemoryContext {
    fn is_registry_key(&self, key: &[u8; 32]) -> bool {
        self.registry_keys.contains(key)
    }
    fn is_operator_key(&self, key: &[u8; 32]) -> bool {
        self.operator_keys.contains(key)
    }
    fn device_grant(&self, account: &[u8; 32], device: &[u8; 32]) -> Option<DeviceGrant> {
        self.grants.get(&(*account, *device)).cloned()
    }
    fn attestation(&self, account: &[u8; 32], at: i64) -> Option<Attestation> {
        self.attestations
            .get(account)?
            .iter()
            .find(|a| a.valid_from <= at && at < a.expires_at)
            .copied()
    }
}

const T0: i64 = 1_790_000_000;
const ZONE: &str = "z_3k9dq7m2x4pa";
const CID: &str = "b3_afurvqfaykmqtj5r2xhj5kxprhmpfwoxwoxqkhqpbdddpqhsvqfq";

fn key(n: u8) -> SigningKey {
    let mut seed = [0u8; 32];
    seed[0] = n;
    seed[31] = 0xa7;
    SigningKey::from_seed(&SecretKey32::from_bytes(seed))
}

fn pk(n: u8) -> [u8; 32] {
    key(n).verifying_key().to_bytes()
}

fn pkh(n: u8) -> String {
    hex::encode(pk(n))
}

// Papeles de las llaves de prueba.
const ROOT_A: u8 = 1; // cuenta de nivel 3
const DEV_A: u8 = 2; // dispositivo de A con todos los tipos
const ROOT_B: u8 = 3; // cuenta de nivel 1
const DEV_B: u8 = 4; // dispositivo de B
const REGISTRY: u8 = 5;
const OPERATOR: u8 = 6;
const ROOT_C: u8 = 7; // cuenta sin atestación
const DEV_A_REVOKED: u8 = 8;
const DEV_A_EXPIRED: u8 = 9;
const DEV_A_REACT_ONLY: u8 = 10;
const STRANGER: u8 = 11;

const ALL_DEVICE_KINDS: [&str; 8] = [
    "auth.request",
    "account.profile",
    "post.create",
    "post.delete",
    "post.repost",
    "reaction.set",
    "follow.set",
    "report.create",
];

fn obj(pairs: Vec<(&str, Json)>) -> Json {
    Json::Object(pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}

fn context_json() -> Json {
    let grant =
        |account: u8, device: u8, caps: &[&str], from: i64, to: i64, revoked: Option<i64>| {
            let mut fields = vec![
                ("account", pkh(account).into()),
                ("device", pkh(device).into()),
                (
                    "caps",
                    Json::Array(caps.iter().map(|c| (*c).into()).collect()),
                ),
                ("granted_at", Json::Int(from)),
                ("expires_at", Json::Int(to)),
            ];
            if let Some(r) = revoked {
                fields.push(("revoked_at", Json::Int(r)));
            }
            obj(fields)
        };
    let att = |account: u8, level: i64, from: i64, to: i64| {
        obj(vec![
            ("account", pkh(account).into()),
            ("level", Json::Int(level)),
            ("valid_from", Json::Int(from)),
            ("expires_at", Json::Int(to)),
        ])
    };
    obj(vec![
        ("registry_keys", Json::Array(vec![pkh(REGISTRY).into()])),
        ("operator_keys", Json::Array(vec![pkh(OPERATOR).into()])),
        (
            "grants",
            Json::Array(vec![
                grant(
                    ROOT_A,
                    DEV_A,
                    &ALL_DEVICE_KINDS,
                    T0 - DAY,
                    T0 + 30 * DAY,
                    None,
                ),
                grant(
                    ROOT_B,
                    DEV_B,
                    &ALL_DEVICE_KINDS,
                    T0 - DAY,
                    T0 + 30 * DAY,
                    None,
                ),
                grant(
                    ROOT_A,
                    DEV_A_REVOKED,
                    &ALL_DEVICE_KINDS,
                    T0 - DAY,
                    T0 + 30 * DAY,
                    Some(T0 - 60),
                ),
                grant(
                    ROOT_A,
                    DEV_A_EXPIRED,
                    &ALL_DEVICE_KINDS,
                    T0 - 20 * DAY,
                    T0 - DAY,
                    None,
                ),
                grant(
                    ROOT_A,
                    DEV_A_REACT_ONLY,
                    &["reaction.set"],
                    T0 - DAY,
                    T0 + 30 * DAY,
                    None,
                ),
            ]),
        ),
        (
            "attestations",
            Json::Array(vec![
                att(ROOT_A, 3, T0 - 10 * DAY, T0 + 30 * DAY),
                att(ROOT_B, 1, T0 - 10 * DAY, T0 + 30 * DAY),
            ]),
        ),
    ])
}

fn hex32(s: &str) -> Option<[u8; 32]> {
    let mut out = [0u8; 32];
    hex::decode_to_slice(s, &mut out).ok()?;
    Some(out)
}

/// Construye el contexto en memoria de un archivo de vectores de validación ya analizado.
#[must_use]
pub fn load_context(suite: &Json) -> Option<MemoryContext> {
    load_context_object(suite.as_object()?.get("context")?)
}

fn load_context_object(ctx: &Json) -> Option<MemoryContext> {
    let o = ctx.as_object()?;
    let keys = |name: &str| -> Option<Vec<[u8; 32]>> {
        o.get(name)?
            .as_array()?
            .iter()
            .map(|k| hex32(k.as_str()?))
            .collect()
    };
    let mut out = MemoryContext {
        registry_keys: keys("registry_keys")?,
        operator_keys: keys("operator_keys")?,
        ..MemoryContext::default()
    };
    for g in o.get("grants")?.as_array()? {
        let g = g.as_object()?;
        let caps = g
            .get("caps")?
            .as_array()?
            .iter()
            .map(|c| c.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()?;
        out.grants.insert(
            (
                hex32(g.get("account")?.as_str()?)?,
                hex32(g.get("device")?.as_str()?)?,
            ),
            DeviceGrant {
                caps,
                granted_at: g.get("granted_at")?.as_int()?,
                expires_at: g.get("expires_at")?.as_int()?,
                revoked_at: g.get("revoked_at").and_then(Json::as_int),
            },
        );
    }
    for a in o.get("attestations")?.as_array()? {
        let a = a.as_object()?;
        out.attestations
            .entry(hex32(a.get("account")?.as_str()?)?)
            .or_default()
            .push(Attestation {
                level: u8::try_from(a.get("level")?.as_int()?).ok()?,
                valid_from: a.get("valid_from")?.as_int()?,
                expires_at: a.get("expires_at")?.as_int()?,
            });
    }
    Some(out)
}

/// Firma un evento de prueba: `account` es la cuenta y `signer` la llave que firma.
fn ev(account: u8, signer: u8, at: i64, kind: &str, content: Json) -> Result<String, CoreError> {
    ev_tags(account, signer, at, kind, vec![], content)
}

fn ev_tags(
    account: u8,
    signer: u8,
    at: i64,
    kind: &str,
    tags: Vec<Tag>,
    content: Json,
) -> Result<String, CoreError> {
    let content = match content {
        Json::Object(m) => m,
        _ => BTreeMap::new(),
    };
    let unsigned = UnsignedEvent {
        account: pk(account),
        signer: pk(signer),
        created_at: at,
        kind: kind.into(),
        tags,
        content,
    };
    Ok(unsigned.sign(&key(signer))?.to_json().to_canonical_string())
}

fn zone(mode: &str) -> Json {
    obj(vec![("id", ZONE.into()), ("mode", mode.into())])
}

fn post(text: &str) -> Vec<(&'static str, Json)> {
    vec![
        ("text", text.into()),
        ("lang", "es".into()),
        ("zone", zone("home")),
    ]
}

fn with(mut base: Vec<(&'static str, Json)>, extra: Vec<(&'static str, Json)>) -> Json {
    for (k, v) in extra {
        base.retain(|(bk, _)| *bk != k);
        base.push((k, v));
    }
    obj(base)
}

fn without(base: Vec<(&'static str, Json)>, field: &str) -> Json {
    obj(base.into_iter().filter(|(k, _)| *k != field).collect())
}

const EVENT_ID: &str = "4b2a3b6c1f0e9d8c7b6a5f4e3d2c1b0a99887766554433221100ffeeddccbbaa";

fn valid_cases() -> Result<Vec<(&'static str, String)>, CoreError> {
    let attest = obj(vec![
        ("account", pkh(ROOT_A).into()),
        ("tier", "base".into()),
        ("level", Json::Int(3)),
        ("epoch", Json::Int(42)),
        ("expires_at", Json::Int(T0 + 30 * DAY)),
        ("issuer", pkh(REGISTRY).into()),
    ]);
    let rotate = obj(vec![
        ("new_account", pkh(STRANGER).into()),
        ("effective_at", Json::Int(T0 + ROTATE_DELAY)),
    ]);
    Ok(vec![
        (
            "post.create de un dispositivo delegado",
            ev(ROOT_A, DEV_A, T0, "post.create", obj(post("Hola barrio")))?,
        ),
        (
            "post.create con etiquetas, imágenes y nivel 3",
            ev_tags(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                vec![Tag::Topic("vecinos".into()), Tag::Mention(pk(ROOT_B))],
                with(
                    post("Mira esto #vecinos"),
                    vec![("media", Json::Array(vec![CID.into()]))],
                ),
            )?,
        ),
        (
            "post.create solo con imagen",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                with(post(""), vec![("media", Json::Array(vec![CID.into()]))]),
            )?,
        ),
        (
            "respuesta con reply_to y root",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                with(
                    post("De acuerdo"),
                    vec![
                        ("reply_to", EVENT_ID.into()),
                        ("root", EVENT_ID.into()),
                        ("quote", EVENT_ID.into()),
                    ],
                ),
            )?,
        ),
        (
            "post.create de nivel 1 sin imágenes",
            ev(ROOT_B, DEV_B, T0, "post.create", obj(post("Hola")))?,
        ),
        (
            "post.create con 500 grafemas compuestos",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                obj(post(&"👩‍👩‍👧".repeat(500))),
            )?,
        ),
        (
            "account.profile firmado por la raíz",
            ev(
                ROOT_A,
                ROOT_A,
                T0,
                "account.profile",
                obj(vec![
                    ("name", "Ana".into()),
                    ("bio", "Vecina de Chapinero".into()),
                    ("avatar", CID.into()),
                ]),
            )?,
        ),
        (
            "account.profile vacío firmado por un dispositivo",
            ev(ROOT_A, DEV_A, T0, "account.profile", obj(vec![]))?,
        ),
        (
            "device.grant de 90 días",
            ev(
                ROOT_A,
                ROOT_A,
                T0,
                "device.grant",
                obj(vec![
                    ("device", pkh(STRANGER).into()),
                    (
                        "caps",
                        Json::Array(vec!["post.create".into(), "reaction.set".into()]),
                    ),
                    ("expires_at", Json::Int(T0 + 90 * DAY)),
                    ("label", "Portátil".into()),
                ]),
            )?,
        ),
        (
            "device.revoke",
            ev(
                ROOT_A,
                ROOT_A,
                T0,
                "device.revoke",
                obj(vec![("device", pkh(DEV_A).into())]),
            )?,
        ),
        (
            "post.delete",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.delete",
                obj(vec![("target", EVENT_ID.into())]),
            )?,
        ),
        (
            "post.repost",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.repost",
                obj(vec![("target", EVENT_ID.into())]),
            )?,
        ),
        (
            "reaction.set like",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "reaction.set",
                obj(vec![("target", EVENT_ID.into()), ("value", "like".into())]),
            )?,
        ),
        (
            "reaction.set vacía con dispositivo de alcance limitado",
            ev(
                ROOT_A,
                DEV_A_REACT_ONLY,
                T0,
                "reaction.set",
                obj(vec![("target", EVENT_ID.into()), ("value", "".into())]),
            )?,
        ),
        (
            "follow.set",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "follow.set",
                obj(vec![
                    ("target_account", pkh(ROOT_B).into()),
                    ("value", Json::Bool(true)),
                ]),
            )?,
        ),
        (
            "report.create",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "report.create",
                obj(vec![
                    ("target", EVENT_ID.into()),
                    ("reason", "spam".into()),
                    ("note", "Publicidad repetida".into()),
                ]),
            )?,
        ),
        (
            "auth.request",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "auth.request",
                obj(vec![
                    ("purpose", "account_status".into()),
                    ("nonce", EVENT_ID.into()),
                ]),
            )?,
        ),
        (
            "account.attest del registro",
            ev(REGISTRY, REGISTRY, T0, "account.attest", attest)?,
        ),
        (
            "account.presence del registro",
            ev(
                REGISTRY,
                REGISTRY,
                T0,
                "account.presence",
                obj(vec![
                    ("account", pkh(ROOT_A).into()),
                    ("challenge", "ch_01".into()),
                    ("passed_at", Json::Int(T0 - 5)),
                ]),
            )?,
        ),
        (
            "account.rotate por la raíz con 72 horas",
            ev(ROOT_A, ROOT_A, T0, "account.rotate", rotate.clone())?,
        ),
        (
            "account.rotate de recuperación firmada por el registro",
            ev(ROOT_C, REGISTRY, T0, "account.rotate", rotate)?,
        ),
        (
            "mod.action de un operador",
            ev(
                OPERATOR,
                OPERATOR,
                T0,
                "mod.action",
                obj(vec![
                    ("target", EVENT_ID.into()),
                    ("action", "hide".into()),
                    ("reason_code", "harassment".into()),
                ]),
            )?,
        ),
        (
            "tipo desconocido firmado por la raíz: pasa la validación universal",
            ev(
                ROOT_A,
                ROOT_A,
                T0,
                "test.future_kind",
                obj(vec![("anything", Json::Int(1))]),
            )?,
        ),
    ])
}

fn invalid_cases() -> Result<Vec<(&'static str, String, ErrorCode)>, CoreError> {
    use ErrorCode as E;
    let rotate_soon = obj(vec![
        ("new_account", pkh(STRANGER).into()),
        ("effective_at", Json::Int(T0 + ROTATE_DELAY - 1)),
    ]);
    let grant_91 = obj(vec![
        ("device", pkh(STRANGER).into()),
        ("caps", Json::Array(vec!["post.create".into()])),
        ("expires_at", Json::Int(T0 + 90 * DAY + 1)),
        ("label", "Teléfono".into()),
    ]);
    let five_media = Json::Array(vec![CID.into(); 5]);
    Ok(vec![
        // Paso 2: esquema del tipo.
        (
            "post.create sin zona",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                without(post("Hola"), "zone"),
            )?,
            E::Schema,
        ),
        (
            "identificador de zona inválido",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                with(
                    post("Hola"),
                    vec![(
                        "zone",
                        obj(vec![("id", "z_LOWERCASEX".into()), ("mode", "home".into())]),
                    )],
                ),
            )?,
            E::Schema,
        ),
        (
            "modo de zona desconocido",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                with(post("Hola"), vec![("zone", zone("everywhere"))]),
            )?,
            E::Schema,
        ),
        (
            "reply_to sin root",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                with(post("Hola"), vec![("reply_to", EVENT_ID.into())]),
            )?,
            E::Schema,
        ),
        (
            "publicación vacía",
            ev(ROOT_A, DEV_A, T0, "post.create", obj(post("")))?,
            E::Schema,
        ),
        (
            "idioma inválido",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                with(post("Hola"), vec![("lang", "Español".into())]),
            )?,
            E::Schema,
        ),
        (
            "reacción con valor desconocido",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "reaction.set",
                obj(vec![("target", EVENT_ID.into()), ("value", "love".into())]),
            )?,
            E::Schema,
        ),
        (
            "atestación de nivel 4",
            ev(
                REGISTRY,
                REGISTRY,
                T0,
                "account.attest",
                obj(vec![
                    ("account", pkh(ROOT_A).into()),
                    ("tier", "base".into()),
                    ("level", Json::Int(4)),
                    ("epoch", Json::Int(1)),
                    ("expires_at", Json::Int(T0 + DAY)),
                    ("issuer", pkh(REGISTRY).into()),
                ]),
            )?,
            E::Schema,
        ),
        (
            "coordenadas en el contenido (I-2)",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                with(
                    post("Hola"),
                    vec![("lat", Json::Int(4)), ("lon", Json::Int(-74))],
                ),
            )?,
            E::UnknownField,
        ),
        (
            "texto sin normalizar a NFC",
            ev(ROOT_A, DEV_A, T0, "post.create", obj(post("cafe\u{301}")))?,
            E::TextNotNfc,
        ),
        // Límites del tipo.
        (
            "501 grafemas",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                obj(post(&"a".repeat(501))),
            )?,
            E::LimitExceeded,
        ),
        (
            "cinco imágenes",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "post.create",
                with(post("Hola"), vec![("media", five_media)]),
            )?,
            E::LimitExceeded,
        ),
        (
            "nombre de perfil de 51 caracteres",
            ev(
                ROOT_A,
                ROOT_A,
                T0,
                "account.profile",
                obj(vec![("name", "n".repeat(51).into())]),
            )?,
            E::LimitExceeded,
        ),
        (
            "delegación de más de 90 días",
            ev(ROOT_A, ROOT_A, T0, "device.grant", grant_91)?,
            E::LimitExceeded,
        ),
        (
            "rotación con menos de 72 horas",
            ev(ROOT_A, ROOT_A, T0, "account.rotate", rotate_soon)?,
            E::LimitExceeded,
        ),
        // Paso 5: quién firma.
        (
            "post.create firmado por la raíz",
            ev(ROOT_A, ROOT_A, T0, "post.create", obj(post("Hola")))?,
            E::SignerRole,
        ),
        (
            "device.revoke firmado por un dispositivo",
            ev(
                ROOT_A,
                DEV_A,
                T0,
                "device.revoke",
                obj(vec![("device", pkh(DEV_A_EXPIRED).into())]),
            )?,
            E::SignerRole,
        ),
        (
            "atestación firmada por quien no es el registro",
            ev(
                STRANGER,
                STRANGER,
                T0,
                "account.attest",
                obj(vec![
                    ("account", pkh(STRANGER).into()),
                    ("tier", "base".into()),
                    ("level", Json::Int(3)),
                    ("epoch", Json::Int(1)),
                    ("expires_at", Json::Int(T0 + DAY)),
                    ("issuer", pkh(STRANGER).into()),
                ]),
            )?,
            E::SignerRole,
        ),
        (
            "acción de moderación de quien no es operador",
            ev(
                ROOT_A,
                ROOT_A,
                T0,
                "mod.action",
                obj(vec![
                    ("target", EVENT_ID.into()),
                    ("action", "remove".into()),
                    ("reason_code", "spam".into()),
                ]),
            )?,
            E::SignerRole,
        ),
        (
            "dispositivo sin delegación",
            ev(ROOT_A, STRANGER, T0, "post.create", obj(post("Hola")))?,
            E::DelegationMissing,
        ),
        (
            "dispositivo revocado",
            ev(ROOT_A, DEV_A_REVOKED, T0, "post.create", obj(post("Hola")))?,
            E::DelegationRevoked,
        ),
        (
            "delegación caducada",
            ev(ROOT_A, DEV_A_EXPIRED, T0, "post.create", obj(post("Hola")))?,
            E::DelegationExpired,
        ),
        (
            "delegación anterior al evento",
            ev(
                ROOT_A,
                DEV_A,
                T0 - 2 * DAY,
                "post.create",
                obj(post("Hola")),
            )?,
            E::DelegationExpired,
        ),
        (
            "delegación que no cubre el tipo",
            ev(
                ROOT_A,
                DEV_A_REACT_ONLY,
                T0,
                "post.create",
                obj(post("Hola")),
            )?,
            E::DelegationScope,
        ),
        // Paso 6: atestación y nivel.
        (
            "cuenta sin atestación",
            ev(
                ROOT_C,
                ROOT_C,
                T0,
                "account.profile",
                obj(vec![("name", "Carla".into())]),
            )?,
            E::AttestationMissing,
        ),
        (
            "atestación caducada",
            ev(
                ROOT_B,
                ROOT_B,
                T0 + 31 * DAY,
                "account.profile",
                obj(vec![("name", "Bea".into())]),
            )?,
            E::AttestationMissing,
        ),
        (
            "imágenes con nivel 1",
            ev(
                ROOT_B,
                DEV_B,
                T0,
                "post.create",
                with(post("Hola"), vec![("media", Json::Array(vec![CID.into()]))]),
            )?,
            E::LevelInsufficient,
        ),
        (
            "avatar con nivel 1",
            ev(
                ROOT_B,
                ROOT_B,
                T0,
                "account.profile",
                obj(vec![("avatar", CID.into())]),
            )?,
            E::LevelInsufficient,
        ),
    ])
}

/// Genera `spec/vectors/validation.json`.
///
/// # Errors
/// Solo si los casos definidos aquí son incoherentes.
pub fn generate_validation_suite() -> Result<String, CoreError> {
    let valid = valid_cases()?
        .into_iter()
        .map(|(name, event)| obj(vec![("name", name.into()), ("event", event.into())]))
        .collect();
    let invalid = invalid_cases()?
        .into_iter()
        .map(|(name, event, code)| {
            obj(vec![
                ("name", name.into()),
                ("event", event.into()),
                ("expect", obj(vec![("error", code.as_str().into())])),
            ])
        })
        .collect();
    let suite = obj(vec![
        ("suite", "validation".into()),
        ("protocol", Json::Int(crate::PROTOCOL_VERSION)),
        (
            "note",
            "Generado por `pnpm vectors:gen`. Validación universal (docs/06 §3, pasos 1 a 6) y tipos (§4) con el contexto en memoria de `context`. Llaves públicas y solo de prueba. Cambiar un vector exige ADR.".into(),
        ),
        ("context", context_json()),
        ("valid", Json::Array(valid)),
        ("invalid", Json::Array(invalid)),
    ]);
    let mut text = suite.to_canonical_string();
    text.push('\n');
    Ok(text)
}

/// Ejecuta una suite de validación ya analizada.
pub(crate) fn run(suite: &BTreeMap<String, Json>, report: &mut SuiteReport) {
    let Some(ctx) = suite.get("context").and_then(load_context_object) else {
        report.failures.push("context ausente o mal formado".into());
        return;
    };
    let list = |name: &str| {
        suite
            .get(name)
            .and_then(Json::as_array)
            .map(<[Json]>::to_vec)
            .unwrap_or_default()
    };
    for v in list("valid") {
        let name = v
            .as_object()
            .and_then(|o| o.get("name"))
            .and_then(Json::as_str)
            .unwrap_or("?");
        let raw = v
            .as_object()
            .and_then(|o| o.get("event"))
            .and_then(Json::as_str)
            .unwrap_or("");
        match validate_event(raw.as_bytes(), &ctx) {
            Ok(_) => report.passed += 1,
            Err(e) => report
                .failures
                .push(format!("[válido] {name}: rechazado con {e}")),
        }
    }
    for v in list("invalid") {
        let o = v.as_object();
        let name = o
            .and_then(|o| o.get("name"))
            .and_then(Json::as_str)
            .unwrap_or("?");
        let raw = o
            .and_then(|o| o.get("event"))
            .and_then(Json::as_str)
            .unwrap_or("");
        let expected = o
            .and_then(|o| o.get("expect"))
            .and_then(Json::as_object)
            .and_then(|e| e.get("error"))
            .and_then(Json::as_str)
            .unwrap_or("?");
        match validate_event(raw.as_bytes(), &ctx) {
            Ok(_) => report.failures.push(format!(
                "[inválido] {name}: se aceptó; se esperaba {expected}"
            )),
            Err(e) if e.code.as_str() == expected => report.passed += 1,
            Err(e) => report.failures.push(format!(
                "[inválido] {name}: se esperaba {expected} y se obtuvo {e}"
            )),
        }
    }
}
