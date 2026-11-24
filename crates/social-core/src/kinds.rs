//! Esquema y límites de cada `kind` de `docs/06` §4.
//!
//! Cada tipo conocido declara: quién puede firmarlo, si exige atestación, qué nivel
//! mínimo exige su contenido, su esquema (paso 2) y sus límites (paso 8). En el
//! contenido de un tipo conocido no se admiten campos desconocidos: así ningún cliente
//! puede colar datos no previstos (por ejemplo, coordenadas, I-2).

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

/// Descripción de un tipo conocido.
#[derive(Debug, Clone, Copy)]
pub struct KindSpec {
    /// Nombre del tipo.
    pub kind: &'static str,
    /// Quién puede firmarlo.
    pub signer: SignerRole,
}

impl KindSpec {
    /// Los eventos del registro y de operadores no exigen atestación de cuenta (paso 6).
    #[must_use]
    pub const fn needs_attestation(&self) -> bool {
        !matches!(self.signer, SignerRole::Registry | SignerRole::Operator)
    }
}

/// Tipos conocidos de la versión 1. Añadir uno exige ADR, `docs/06`, esquemas y vectores.
pub const KINDS: &[KindSpec] = &[
    KindSpec {
        kind: "account.attest",
        signer: SignerRole::Registry,
    },
    KindSpec {
        kind: "account.presence",
        signer: SignerRole::Registry,
    },
    KindSpec {
        kind: "auth.request",
        signer: SignerRole::Device,
    },
    KindSpec {
        kind: "account.profile",
        signer: SignerRole::RootOrDevice,
    },
    KindSpec {
        kind: "account.rotate",
        signer: SignerRole::RootOrRegistry,
    },
    KindSpec {
        kind: "device.grant",
        signer: SignerRole::Root,
    },
    KindSpec {
        kind: "device.revoke",
        signer: SignerRole::Root,
    },
    KindSpec {
        kind: "post.create",
        signer: SignerRole::Device,
    },
    KindSpec {
        kind: "post.delete",
        signer: SignerRole::Device,
    },
    KindSpec {
        kind: "post.repost",
        signer: SignerRole::Device,
    },
    KindSpec {
        kind: "reaction.set",
        signer: SignerRole::Device,
    },
    KindSpec {
        kind: "follow.set",
        signer: SignerRole::Device,
    },
    KindSpec {
        kind: "report.create",
        signer: SignerRole::Device,
    },
    KindSpec {
        kind: "mod.action",
        signer: SignerRole::Operator,
    },
];

/// Busca un tipo conocido.
#[must_use]
pub fn spec(kind: &str) -> Option<&'static KindSpec> {
    KINDS.iter().find(|k| k.kind == kind)
}

/// Lectura tipada del contenido, que registra qué campos se consultaron para rechazar
/// después los desconocidos.
pub(crate) struct Content<'a> {
    map: &'a BTreeMap<String, Json>,
    seen: BTreeSet<&'static str>,
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

impl<'a> Content<'a> {
    pub(crate) fn new(map: &'a BTreeMap<String, Json>) -> Self {
        Self {
            map,
            seen: BTreeSet::new(),
        }
    }

    fn get(&mut self, field: &'static str) -> Option<&'a Json> {
        self.seen.insert(field);
        self.map.get(field)
    }

    fn opt_str(&mut self, field: &'static str) -> Result<Option<&'a str>, CoreError> {
        match self.get(field) {
            None => Ok(None),
            Some(v) => v
                .as_str()
                .map(Some)
                .ok_or_else(|| schema(field, "debe ser una cadena")),
        }
    }

    fn req_str(&mut self, field: &'static str) -> Result<&'a str, CoreError> {
        self.opt_str(field)?.ok_or_else(|| schema(field, "falta"))
    }

    fn req_int(&mut self, field: &'static str) -> Result<i64, CoreError> {
        self.get(field)
            .ok_or_else(|| schema(field, "falta"))?
            .as_int()
            .ok_or_else(|| schema(field, "debe ser un entero"))
    }

    fn req_bool(&mut self, field: &'static str) -> Result<bool, CoreError> {
        self.get(field)
            .ok_or_else(|| schema(field, "falta"))?
            .as_bool()
            .ok_or_else(|| schema(field, "debe ser un booleano"))
    }

    fn req_hex(&mut self, field: &'static str, bytes: usize) -> Result<&'a str, CoreError> {
        let s = self.req_str(field)?;
        if is_lower_hex(s, bytes) {
            Ok(s)
        } else {
            Err(schema(
                field,
                "hexadecimal en minúsculas de longitud exacta",
            ))
        }
    }

    fn opt_hex(&mut self, field: &'static str, bytes: usize) -> Result<Option<&'a str>, CoreError> {
        match self.opt_str(field)? {
            None => Ok(None),
            Some(s) if is_lower_hex(s, bytes) => Ok(Some(s)),
            Some(_) => Err(schema(
                field,
                "hexadecimal en minúsculas de longitud exacta",
            )),
        }
    }

    fn req_enum(&mut self, field: &'static str, allowed: &[&str]) -> Result<&'a str, CoreError> {
        let s = self.req_str(field)?;
        if allowed.contains(&s) {
            Ok(s)
        } else {
            Err(schema(
                field,
                &format!("valor no admitido; se espera uno de {allowed:?}"),
            ))
        }
    }

    fn finish(self) -> Result<(), CoreError> {
        match self.map.keys().find(|k| !self.seen.contains(k.as_str())) {
            Some(extra) => Err(CoreError::new(
                ErrorCode::UnknownField,
                format!("content.{extra}"),
            )),
            None => Ok(()),
        }
    }
}

/// Texto en NFC (paso 2) y con un máximo de grafemas (paso 8).
fn text(field: &str, s: &str, max_graphemes: usize) -> Result<(), CoreError> {
    if !is_nfc(s) {
        return Err(CoreError::new(
            ErrorCode::TextNotNfc,
            format!("content.{field}"),
        ));
    }
    if s.graphemes(true).count() > max_graphemes {
        return Err(limit(field, &format!("máximo {max_graphemes} grafemas")));
    }
    Ok(())
}

/// Identificador de zona: `z_` más 12 caracteres base32 de Crockford en minúsculas (`docs/07` §2).
fn zone_id(s: &str) -> bool {
    s.strip_prefix("z_").is_some_and(|rest| {
        rest.len() == 12
            && rest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b.is_ascii_lowercase() && !b"ilou".contains(&b)))
    })
}

/// Identificador de contenido: `b3_` más BLAKE3 en base32 (RFC 4648) en minúsculas sin relleno.
fn cid(s: &str) -> bool {
    s.strip_prefix("b3_").is_some_and(|rest| {
        rest.len() == 52
            && rest
                .bytes()
                .all(|b| b.is_ascii_lowercase() || (b'2'..=b'7').contains(&b))
    })
}

/// Etiqueta de idioma BCP 47 sencilla: `es`, `es-CO`, `zh-Hant`.
fn lang(s: &str) -> bool {
    let mut parts = s.split('-');
    let primary_ok = parts
        .next()
        .is_some_and(|p| (2..=3).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_lowercase()));
    primary_ok
        && s.len() <= 35
        && parts.all(|p| (2..=8).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// Código corto: `[a-z][a-z0-9_]*`, hasta 32 bytes.
fn code(s: &str) -> bool {
    let mut b = s.bytes();
    s.len() <= 32
        && b.next().is_some_and(|c| c.is_ascii_lowercase())
        && b.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

/// Resumen del contenido que necesita la validación de nivel (paso 6).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContentFacts {
    /// El evento adjunta imágenes (prohibido en nivel 1, `docs/05` §4).
    pub has_media: bool,
}

/// Aplica el esquema y los límites del tipo. Devuelve los datos que necesita el paso 6.
pub(crate) fn check_content(
    kind: &str,
    created_at: i64,
    signer_hex: &str,
    content: &BTreeMap<String, Json>,
) -> Result<ContentFacts, CoreError> {
    let mut c = Content::new(content);
    let mut facts = ContentFacts::default();
    match kind {
        "account.attest" => {
            c.req_hex("account", 32)?;
            c.req_enum("tier", &["alpha", "base", "secondary"])?;
            let level = c.req_int("level")?;
            if !(1..=3).contains(&level) {
                return Err(schema("level", "debe ser 1, 2 o 3"));
            }
            if c.req_int("epoch")? < 0 {
                return Err(schema("epoch", "no puede ser negativo"));
            }
            if c.req_int("expires_at")? <= created_at {
                return Err(schema("expires_at", "debe ser posterior a created_at"));
            }
            if c.req_hex("issuer", 32)? != signer_hex {
                return Err(schema("issuer", "debe ser la llave que firma"));
            }
        }
        "account.presence" => {
            c.req_hex("account", 32)?;
            let challenge = c.req_str("challenge")?;
            if challenge.is_empty() || challenge.len() > 64 || !challenge.is_ascii() {
                return Err(schema("challenge", "de 1 a 64 caracteres ASCII"));
            }
            if c.req_int("passed_at")? > created_at {
                return Err(schema("passed_at", "no puede ser posterior a created_at"));
            }
        }
        "auth.request" => {
            if !code(c.req_str("purpose")?) {
                return Err(schema(
                    "purpose",
                    "código [a-z][a-z0-9_]* de hasta 32 bytes",
                ));
            }
            c.req_hex("nonce", 32)?;
        }
        "account.profile" => {
            if let Some(name) = c.opt_str("name")? {
                text("name", name, 50)?;
            }
            if let Some(bio) = c.opt_str("bio")? {
                text("bio", bio, 300)?;
            }
            if let Some(avatar) = c.opt_str("avatar")? {
                if !cid(avatar) {
                    return Err(schema("avatar", "identificador de contenido b3_ inválido"));
                }
                facts.has_media = true;
            }
        }
        "account.rotate" => {
            c.req_hex("new_account", 32)?;
            if c.req_int("effective_at")? < created_at.saturating_add(ROTATE_DELAY) {
                return Err(limit("effective_at", "el retraso mínimo es de 72 horas"));
            }
        }
        "device.grant" => {
            c.req_hex("device", 32)?;
            let caps = c
                .get("caps")
                .and_then(Json::as_array)
                .ok_or_else(|| schema("caps", "debe ser una lista de tipos"))?;
            let mut unique = BTreeSet::new();
            for cap in caps {
                let cap = cap
                    .as_str()
                    .ok_or_else(|| schema("caps", "debe ser una lista de tipos"))?;
                if crate::event::check_kind(cap).is_err() || !unique.insert(cap) {
                    return Err(schema("caps", "tipos válidos y sin repetir"));
                }
            }
            if caps.is_empty() || caps.len() > 32 {
                return Err(limit("caps", "de 1 a 32 tipos"));
            }
            let expires_at = c.req_int("expires_at")?;
            if expires_at <= created_at {
                return Err(schema("expires_at", "debe ser posterior a created_at"));
            }
            if expires_at > created_at.saturating_add(MAX_GRANT_DAYS * DAY) {
                return Err(limit("expires_at", "caducidad máxima de 90 días"));
            }
            text("label", c.req_str("label")?, 50)?;
        }
        "device.revoke" => {
            c.req_hex("device", 32)?;
        }
        "post.create" => {
            let body = c.req_str("text")?;
            text("text", body, 500)?;
            if !lang(c.req_str("lang")?) {
                return Err(schema("lang", "etiqueta de idioma BCP 47"));
            }
            let zone = c
                .get("zone")
                .and_then(Json::as_object)
                .ok_or_else(|| schema("zone", "debe ser un objeto {id, mode}"))?;
            let mut z = Content::new(zone);
            if !zone_id(z.req_str("id")?) {
                return Err(schema("zone.id", "identificador de zona inválido"));
            }
            z.req_enum("mode", &["home", "live", "remote"])?;
            z.finish()?;
            if let Some(media) = c.get("media") {
                let list = media
                    .as_array()
                    .ok_or_else(|| schema("media", "debe ser una lista"))?;
                if list.iter().any(|m| !m.as_str().is_some_and(cid)) {
                    return Err(schema("media", "identificadores de contenido b3_"));
                }
                if list.len() > 4 {
                    return Err(limit("media", "máximo 4"));
                }
                facts.has_media = !list.is_empty();
            }
            if body.is_empty() && !facts.has_media {
                return Err(schema(
                    "text",
                    "una publicación sin imágenes necesita texto",
                ));
            }
            let reply_to = c.opt_hex("reply_to", 32)?;
            let root = c.opt_hex("root", 32)?;
            if reply_to.is_some() != root.is_some() {
                return Err(schema("reply_to", "reply_to y root van juntos"));
            }
            c.opt_hex("quote", 32)?;
        }
        "post.delete" | "post.repost" => {
            c.req_hex("target", 32)?;
        }
        "reaction.set" => {
            c.req_hex("target", 32)?;
            c.req_enum("value", &["like", ""])?;
        }
        "follow.set" => {
            c.req_hex("target_account", 32)?;
            c.req_bool("value")?;
        }
        "report.create" => {
            c.req_hex("target", 32)?;
            if !code(c.req_str("reason")?) {
                return Err(schema("reason", "código [a-z][a-z0-9_]* de hasta 32 bytes"));
            }
            if let Some(note) = c.opt_str("note")? {
                text("note", note, 500)?;
            }
        }
        "mod.action" => {
            c.req_hex("target", 32)?;
            c.req_enum("action", &["hide", "remove", "suspend"])?;
            if !code(c.req_str("reason_code")?) {
                return Err(schema(
                    "reason_code",
                    "código [a-z][a-z0-9_]* de hasta 32 bytes",
                ));
            }
        }
        // Un tipo desconocido no tiene esquema propio (`docs/06` §4).
        _ => return Ok(facts),
    }
    c.finish()?;
    Ok(facts)
}
