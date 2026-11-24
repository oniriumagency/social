//! Esquema y límites de cada `kind` de `docs/06` §4.
//!
//! Cada tipo conocido se declara una sola vez en [`KINDS`]: quién lo firma y qué campos
//! lleva su contenido, con su tipo, formato y límite. De esa tabla salen a la vez la
//! validación (este módulo) y los esquemas JSON publicados en `spec/schemas/`
//! (`crate::schema`, `pnpm spec:gen`), así que no pueden divergir. Las reglas que
//! relacionan varios campos (plazos, `reply_to` con `root`...) están en `check_rules`.
//!
//! En el contenido de un tipo conocido no se admiten campos desconocidos: así ningún
//! cliente puede colar datos no previstos (por ejemplo, coordenadas, I-2).

use std::collections::{BTreeMap, BTreeSet};

use unicode_normalization::is_nfc;
use unicode_segmentation::UnicodeSegmentation;

use crate::{CoreError, ErrorCode, Json};

/// Segundos en un día.
pub const DAY: i64 = 86_400;
/// Retraso mínimo de una rotación de llave raíz: 72 horas.
pub const ROTATE_DELAY: i64 = 3 * DAY;
/// Caducidad máxima de una delegación de dispositivo: 90 días.
pub const MAX_GRANT_DAYS: i64 = 90;

/// Quién puede firmar un tipo de evento (columna «Firma» de `docs/06` §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerRole {
    /// La llave raíz de la cuenta (`signer == account`).
    Root,
    /// Un dispositivo delegado (`signer != account`, con `device.grant` vigente).
    Device,
    /// La raíz o un dispositivo delegado.
    RootOrDevice,
    /// La raíz antigua de la cuenta, o el registro (`account.rotate`).
    RootOrRegistry,
    /// Una llave del registro de humanidad (`signer == account`, llave conocida).
    Registry,
    /// Una llave de operador de moderación (`signer == account`, llave conocida).
    Operator,
}

impl SignerRole {
    /// Nombre para la documentación y los esquemas.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Device => "device",
            Self::RootOrDevice => "root_or_device",
            Self::RootOrRegistry => "root_or_registry",
            Self::Registry => "registry",
            Self::Operator => "operator",
        }
    }

    /// El papel admite una llave de dispositivo con `device.grant`.
    #[must_use]
    pub const fn delegable(self) -> bool {
        matches!(self, Self::Device | Self::RootOrDevice)
    }
}

/// Tipo y formato de un campo del contenido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    /// 32 bytes en hexadecimal en minúsculas (llave pública o id de evento).
    Hex32,
    /// Entero en ±(2⁵³ − 1).
    Int,
    /// Entero no negativo.
    NonNegInt,
    /// Nivel de confianza: 1, 2 o 3.
    Level,
    /// Booleano.
    Bool,
    /// Texto en NFC con un máximo de grafemas.
    Text(usize),
    /// Una de las cadenas dadas.
    Enum(&'static [&'static str]),
    /// Código `[a-z][a-z0-9_]*` de hasta 32 bytes.
    Code,
    /// Identificador de contenido `b3_` más 52 caracteres base32 (RFC 4648) en minúsculas.
    Cid,
    /// Etiqueta de idioma BCP 47 sencilla.
    Lang,
    /// Cadena ASCII de 1 a N bytes.
    Ascii(usize),
    /// Zona: objeto `{id, mode}`.
    Zone,
    /// Lista de hasta N identificadores de contenido.
    CidList(usize),
    /// Lista de 1 a N tipos de evento sin repetir.
    KindList(usize),
}

/// Campo del contenido de un tipo.
#[derive(Debug, Clone, Copy)]
pub struct Field {
    /// Nombre.
    pub name: &'static str,
    /// Tipo y formato.
    pub ty: FieldType,
    /// Obligatorio u opcional.
    pub required: bool,
}

const fn req(name: &'static str, ty: FieldType) -> Field {
    Field {
        name,
        ty,
        required: true,
    }
}

const fn opt(name: &'static str, ty: FieldType) -> Field {
    Field {
        name,
        ty,
        required: false,
    }
}

/// Descripción de un tipo conocido.
#[derive(Debug, Clone, Copy)]
pub struct KindSpec {
    /// Nombre del tipo.
    pub kind: &'static str,
    /// Quién puede firmarlo.
    pub signer: SignerRole,
    /// Campos del contenido. No se admite ningún otro.
    pub fields: &'static [Field],
    /// Reglas entre campos, en palabras, para la documentación y los esquemas.
    pub rules: &'static str,
}

impl KindSpec {
    /// Los eventos del registro y de operadores no exigen atestación de cuenta (paso 6).
    #[must_use]
    pub const fn needs_attestation(&self) -> bool {
        !matches!(self.signer, SignerRole::Registry | SignerRole::Operator)
    }
}

use FieldType as F;

/// Tipos conocidos de la versión 1. Añadir uno exige ADR, `docs/06`, esquemas y vectores.
pub const KINDS: &[KindSpec] = &[
    KindSpec {
        kind: "account.attest",
        signer: SignerRole::Registry,
        fields: &[
            req("account", F::Hex32),
            req("tier", F::Enum(&["alpha", "base", "secondary"])),
            req("level", F::Level),
            req("epoch", F::NonNegInt),
            req("expires_at", F::Int),
            req("issuer", F::Hex32),
        ],
        rules: "expires_at > created_at; issuer es la llave que firma.",
    },
    KindSpec {
        kind: "account.presence",
        signer: SignerRole::Registry,
        fields: &[
            req("account", F::Hex32),
            req("challenge", F::Ascii(64)),
            req("passed_at", F::Int),
        ],
        rules: "passed_at <= created_at.",
    },
    KindSpec {
        kind: "auth.request",
        signer: SignerRole::Device,
        fields: &[req("purpose", F::Code), req("nonce", F::Hex32)],
        rules: "Efímero: no se guarda ni se reenvía. La ingesta exige created_at a menos de 60 s.",
    },
    KindSpec {
        kind: "account.profile",
        signer: SignerRole::RootOrDevice,
        fields: &[
            opt("name", F::Text(50)),
            opt("bio", F::Text(300)),
            opt("avatar", F::Cid),
        ],
        rules: "Reemplazable. avatar exige nivel 2 o superior.",
    },
    KindSpec {
        kind: "account.rotate",
        signer: SignerRole::RootOrRegistry,
        fields: &[req("new_account", F::Hex32), req("effective_at", F::Int)],
        rules: "effective_at >= created_at + 72 h.",
    },
    KindSpec {
        kind: "device.grant",
        signer: SignerRole::Root,
        fields: &[
            req("device", F::Hex32),
            req("caps", F::KindList(32)),
            req("expires_at", F::Int),
            req("label", F::Text(50)),
        ],
        rules: "created_at < expires_at <= created_at + 90 días; caps solo con tipos que firma un dispositivo.",
    },
    KindSpec {
        kind: "device.revoke",
        signer: SignerRole::Root,
        fields: &[req("device", F::Hex32)],
        rules: "Efecto inmediato desde created_at.",
    },
    KindSpec {
        kind: "post.create",
        signer: SignerRole::Device,
        fields: &[
            req("text", F::Text(500)),
            req("lang", F::Lang),
            req("zone", F::Zone),
            opt("media", F::CidList(4)),
            opt("reply_to", F::Hex32),
            opt("root", F::Hex32),
            opt("quote", F::Hex32),
        ],
        rules: "Texto o imágenes; reply_to y root van juntos; imágenes exigen nivel 2 o superior.",
    },
    KindSpec {
        kind: "post.delete",
        signer: SignerRole::Device,
        fields: &[req("target", F::Hex32)],
        rules: "Lápida. Solo el autor.",
    },
    KindSpec {
        kind: "post.repost",
        signer: SignerRole::Device,
        fields: &[req("target", F::Hex32)],
        rules: "",
    },
    KindSpec {
        kind: "reaction.set",
        signer: SignerRole::Device,
        fields: &[
            req("target", F::Hex32),
            req("value", F::Enum(&["like", ""])),
        ],
        rules: "Reemplazable por par cuenta-objetivo.",
    },
    KindSpec {
        kind: "follow.set",
        signer: SignerRole::Device,
        fields: &[req("target_account", F::Hex32), req("value", F::Bool)],
        rules: "Reemplazable.",
    },
    KindSpec {
        kind: "report.create",
        signer: SignerRole::Device,
        fields: &[
            req("target", F::Hex32),
            req("reason", F::Code),
            opt("note", F::Text(500)),
        ],
        rules: "No se propaga a otros usuarios.",
    },
    KindSpec {
        kind: "mod.action",
        signer: SignerRole::Operator,
        fields: &[
            req("target", F::Hex32),
            req("action", F::Enum(&["hide", "remove", "suspend"])),
            req("reason_code", F::Code),
        ],
        rules: "Público y auditable.",
    },
];

/// Busca un tipo conocido.
#[must_use]
pub fn spec(kind: &str) -> Option<&'static KindSpec> {
    KINDS.iter().find(|k| k.kind == kind)
}

fn schema(field: &str, msg: &str) -> CoreError {
    CoreError::new(ErrorCode::Schema, format!("content.{field}: {msg}"))
}

fn limit(field: &str, msg: &str) -> CoreError {
    CoreError::new(ErrorCode::LimitExceeded, format!("content.{field}: {msg}"))
}

fn is_lower_hex(s: &str, bytes: usize) -> bool {
    s.len() == bytes * 2
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Identificador de zona: `z_` más 12 caracteres base32 de Crockford en minúsculas (`docs/07` §2).
pub(crate) fn is_zone_id(s: &str) -> bool {
    s.strip_prefix("z_").is_some_and(|rest| {
        rest.len() == 12
            && rest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b.is_ascii_lowercase() && !b"ilou".contains(&b)))
    })
}

/// Identificador de contenido: `b3_` más BLAKE3 en base32 (RFC 4648) en minúsculas sin relleno.
///
/// 32 bytes son 256 bits y 52 caracteres llevan 260: los 4 bits sobrantes del último
/// carácter deben ser cero, para que cada contenido tenga una sola escritura (`a` o `q`).
fn is_cid(s: &str) -> bool {
    s.strip_prefix("b3_").is_some_and(|rest| {
        rest.len() == 52
            && rest
                .bytes()
                .all(|b| b.is_ascii_lowercase() || (b'2'..=b'7').contains(&b))
            && rest.ends_with(['a', 'q'])
    })
}

fn is_lang(s: &str) -> bool {
    let mut parts = s.split('-');
    let primary_ok = parts
        .next()
        .is_some_and(|p| (2..=3).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_lowercase()));
    primary_ok
        && s.len() <= 35
        && parts.all(|p| (2..=8).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_alphanumeric()))
}

fn is_code(s: &str) -> bool {
    let mut b = s.bytes();
    s.len() <= 32
        && b.next().is_some_and(|c| c.is_ascii_lowercase())
        && b.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

fn check_text(name: &str, s: &str, max_graphemes: usize) -> Result<(), CoreError> {
    if !is_nfc(s) {
        return Err(CoreError::new(
            ErrorCode::TextNotNfc,
            format!("content.{name}"),
        ));
    }
    if s.graphemes(true).count() > max_graphemes {
        return Err(limit(name, &format!("máximo {max_graphemes} grafemas")));
    }
    Ok(())
}

/// Comprueba un campo contra su tipo: esquema (paso 2) y límite (paso 8).
fn check_field(name: &str, ty: FieldType, value: &Json) -> Result<(), CoreError> {
    let string = || {
        value
            .as_str()
            .ok_or_else(|| schema(name, "debe ser una cadena"))
    };
    match ty {
        F::Hex32 => {
            if !is_lower_hex(string()?, 32) {
                return Err(schema(name, "hexadecimal en minúsculas de 64 caracteres"));
            }
        }
        F::Int => {
            value
                .as_int()
                .ok_or_else(|| schema(name, "debe ser un entero"))?;
        }
        F::NonNegInt => {
            if value
                .as_int()
                .ok_or_else(|| schema(name, "debe ser un entero"))?
                < 0
            {
                return Err(schema(name, "no puede ser negativo"));
            }
        }
        F::Level => {
            let n = value
                .as_int()
                .ok_or_else(|| schema(name, "debe ser un entero"))?;
            if !(1..=3).contains(&n) {
                return Err(schema(name, "debe ser 1, 2 o 3"));
            }
        }
        F::Bool => {
            value
                .as_bool()
                .ok_or_else(|| schema(name, "debe ser un booleano"))?;
        }
        F::Text(max) => check_text(name, string()?, max)?,
        F::Enum(allowed) => {
            if !allowed.contains(&string()?) {
                return Err(schema(name, &format!("se espera uno de {allowed:?}")));
            }
        }
        F::Code => {
            if !is_code(string()?) {
                return Err(schema(name, "código [a-z][a-z0-9_]* de hasta 32 bytes"));
            }
        }
        F::Cid => {
            if !is_cid(string()?) {
                return Err(schema(name, "identificador de contenido b3_ inválido"));
            }
        }
        F::Lang => {
            if !is_lang(string()?) {
                return Err(schema(name, "etiqueta de idioma BCP 47"));
            }
        }
        F::Ascii(max) => {
            let s = string()?;
            if s.is_empty() || s.len() > max || !s.is_ascii() {
                return Err(schema(name, &format!("de 1 a {max} caracteres ASCII")));
            }
        }
        F::Zone => {
            let zone = value
                .as_object()
                .ok_or_else(|| schema(name, "debe ser un objeto {id, mode}"))?;
            let id = zone.get("id").and_then(Json::as_str);
            if !id.is_some_and(is_zone_id) {
                return Err(schema("zone.id", "identificador de zona inválido"));
            }
            let mode = zone.get("mode").and_then(Json::as_str);
            if !mode.is_some_and(|m| ["home", "live", "remote"].contains(&m)) {
                return Err(schema("zone.mode", "se espera home, live o remote"));
            }
            if let Some(extra) = zone.keys().find(|k| *k != "id" && *k != "mode") {
                return Err(CoreError::new(
                    ErrorCode::UnknownField,
                    format!("content.zone.{extra}"),
                ));
            }
        }
        F::CidList(max) => {
            let list = value
                .as_array()
                .ok_or_else(|| schema(name, "debe ser una lista"))?;
            if list.iter().any(|m| !m.as_str().is_some_and(is_cid)) {
                return Err(schema(name, "identificadores de contenido b3_"));
            }
            if list.len() > max {
                return Err(limit(name, &format!("máximo {max}")));
            }
        }
        F::KindList(max) => {
            let list = value
                .as_array()
                .ok_or_else(|| schema(name, "debe ser una lista de tipos"))?;
            let mut unique = BTreeSet::new();
            for item in list {
                let k = item
                    .as_str()
                    .ok_or_else(|| schema(name, "debe ser una lista de tipos"))?;
                if crate::event::check_kind(k).is_err() || !unique.insert(k) {
                    return Err(schema(name, "tipos válidos y sin repetir"));
                }
            }
            if list.is_empty() || list.len() > max {
                return Err(limit(name, &format!("de 1 a {max} tipos")));
            }
        }
    }
    Ok(())
}

/// Resumen del contenido que necesita la validación de nivel (paso 6).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContentFacts {
    /// El evento adjunta imágenes (prohibido en nivel 1, `docs/05` §4).
    pub has_media: bool,
}

/// Reglas que relacionan varios campos o la envoltura (las describe `KindSpec::rules`).
fn check_rules(
    kind: &str,
    created_at: i64,
    signer_hex: &str,
    c: &BTreeMap<String, Json>,
) -> Result<ContentFacts, CoreError> {
    let int = |name: &str| c.get(name).and_then(Json::as_int).unwrap_or_default();
    let present = |name: &str| c.contains_key(name);
    let mut facts = ContentFacts::default();
    match kind {
        "account.attest" => {
            if int("expires_at") <= created_at {
                return Err(schema("expires_at", "debe ser posterior a created_at"));
            }
            if c.get("issuer").and_then(Json::as_str) != Some(signer_hex) {
                return Err(schema("issuer", "debe ser la llave que firma"));
            }
        }
        "account.presence" if int("passed_at") > created_at => {
            return Err(schema("passed_at", "no puede ser posterior a created_at"));
        }
        "account.profile" => facts.has_media = present("avatar"),
        "account.rotate" if int("effective_at") < created_at.saturating_add(ROTATE_DELAY) => {
            return Err(limit("effective_at", "el retraso mínimo es de 72 horas"));
        }
        "device.grant" => {
            // Solo se delega lo que un dispositivo puede firmar (ADR-021).
            let caps = c.get("caps").and_then(Json::as_array).unwrap_or_default();
            if caps
                .iter()
                .filter_map(Json::as_str)
                .any(|cap| spec(cap).is_some_and(|s| !s.signer.delegable()))
            {
                return Err(schema("caps", "solo tipos que un dispositivo puede firmar"));
            }
            let expires_at = int("expires_at");
            if expires_at <= created_at {
                return Err(schema("expires_at", "debe ser posterior a created_at"));
            }
            if expires_at > created_at.saturating_add(MAX_GRANT_DAYS * DAY) {
                return Err(limit("expires_at", "caducidad máxima de 90 días"));
            }
        }
        "post.create" => {
            facts.has_media = c
                .get("media")
                .and_then(Json::as_array)
                .is_some_and(|m| !m.is_empty());
            if c.get("text").and_then(Json::as_str) == Some("") && !facts.has_media {
                return Err(schema(
                    "text",
                    "una publicación sin imágenes necesita texto",
                ));
            }
            if present("reply_to") != present("root") {
                return Err(schema("reply_to", "reply_to y root van juntos"));
            }
        }
        _ => {}
    }
    Ok(facts)
}

/// Aplica el esquema y los límites del tipo. Devuelve los datos que necesita el paso 6.
/// Un tipo desconocido no tiene esquema propio (`docs/06` §4).
pub(crate) fn check_content(
    kind: &str,
    created_at: i64,
    signer_hex: &str,
    content: &BTreeMap<String, Json>,
) -> Result<ContentFacts, CoreError> {
    let Some(spec) = spec(kind) else {
        return Ok(ContentFacts::default());
    };
    for field in spec.fields {
        match content.get(field.name) {
            Some(value) => check_field(field.name, field.ty, value)?,
            None if field.required => return Err(schema(field.name, "falta")),
            None => {}
        }
    }
    if let Some(extra) = content
        .keys()
        .find(|k| !spec.fields.iter().any(|f| f.name == k.as_str()))
    {
        return Err(CoreError::new(
            ErrorCode::UnknownField,
            format!("content.{extra}"),
        ));
    }
    check_rules(kind, created_at, signer_hex, content)
}
