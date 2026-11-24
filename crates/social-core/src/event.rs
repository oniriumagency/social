//! Envoltura de evento (`docs/06` §3): forma, identificador, firma y verificación.
//!
//! Este módulo aplica los pasos 1 a 4 de la validación universal: tamaño, esquema de la
//! envoltura (sin decimales, `null` ni campos extra), identificador y firma. Los pasos 5
//! y 6 (delegación y atestación) y el esquema de cada `kind` van en la capa de validación.

use std::collections::BTreeMap;

use social_crypto::{PUBLIC_KEY_LEN, SIGNATURE_LEN, Signature, SigningKey, VerifyingKey, sha256};

use crate::hexfmt::decode_fixed;
use crate::{CoreError, ErrorCode, Json};

/// Versión de la envoltura que implementa este núcleo (`docs/06` §10).
pub const PROTOCOL_VERSION: i64 = 1;
/// Tamaño máximo del evento serializado: 16 KiB (`docs/06` §3, paso 1).
pub const MAX_EVENT_BYTES: usize = 16 * 1024;
/// Número máximo de etiquetas (`docs/06` §3).
pub const MAX_TAGS: usize = 16;
/// Longitud máxima del valor de una etiqueta `t`, en caracteres.
pub const MAX_TOPIC_CHARS: usize = 64;
/// Longitud máxima de `kind`, en bytes.
pub const MAX_KIND_BYTES: usize = 64;

/// Campos de la envoltura. Cualquier otro es `E_UNKNOWN_FIELD`.
pub const ENVELOPE_FIELDS: [&str; 9] = [
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

/// Etiqueta indexable (`docs/06` §3). Solo existen `t` y `p`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    /// `["t", "<etiqueta>"]`: tema, en minúsculas.
    Topic(String),
    /// `["p", "<llave pública>"]`: cuenta mencionada.
    Mention([u8; PUBLIC_KEY_LEN]),
}

impl Tag {
    fn to_json(&self) -> Json {
        match self {
            Self::Topic(t) => Json::Array(vec!["t".into(), t.as_str().into()]),
            Self::Mention(pk) => Json::Array(vec!["p".into(), hex::encode(pk).into()]),
        }
    }

    fn from_json(value: &Json, index: usize) -> Result<Self, CoreError> {
        let schema = |msg: &str| CoreError::new(ErrorCode::Schema, format!("tags[{index}]: {msg}"));
        let pair = value
            .as_array()
            .filter(|a| a.len() == 2)
            .ok_or_else(|| schema("debe ser una lista de dos cadenas"))?;
        let (Some(name), Some(val)) = (pair[0].as_str(), pair[1].as_str()) else {
            return Err(schema("debe ser una lista de dos cadenas"));
        };
        match name {
            "t" => {
                let chars = val.chars().count();
                let valid = (1..=MAX_TOPIC_CHARS).contains(&chars)
                    && val.to_lowercase() == val
                    && !val.chars().any(|c| c.is_whitespace() || c.is_control());
                if valid && !unicode_normalization::is_nfc(val) {
                    return Err(CoreError::new(
                        ErrorCode::TextNotNfc,
                        format!("tags[{index}][1]"),
                    ));
                }
                if !valid {
                    return Err(schema(
                        "una etiqueta t tiene de 1 a 64 caracteres, en minúsculas y sin espacios",
                    ));
                }
                Ok(Self::Topic(val.to_owned()))
            }
            "p" => Ok(Self::Mention(decode_fixed::<PUBLIC_KEY_LEN>(
                val,
                &format!("tags[{index}][1]"),
            )?)),
            _ => Err(schema("solo se admiten las etiquetas t y p")),
        }
    }
}

/// Evento todavía sin identificador ni firma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsignedEvent {
    /// Llave pública raíz de la cuenta.
    pub account: [u8; PUBLIC_KEY_LEN],
    /// Llave pública que firma (la raíz o un dispositivo delegado).
    pub signer: [u8; PUBLIC_KEY_LEN],
    /// Segundos Unix.
    pub created_at: i64,
    /// Tipo de evento, por ejemplo `post.create`.
    pub kind: String,
    /// Etiquetas indexables.
    pub tags: Vec<Tag>,
    /// Contenido, siempre un objeto.
    pub content: BTreeMap<String, Json>,
}

impl UnsignedEvent {
    /// `payload = JCS([v, account, signer, created_at, kind, tags, content])`.
    #[must_use]
    pub fn payload(&self) -> Vec<u8> {
        Json::Array(vec![
            Json::Int(PROTOCOL_VERSION),
            hex::encode(self.account).into(),
            hex::encode(self.signer).into(),
            Json::Int(self.created_at),
            self.kind.as_str().into(),
            Json::Array(self.tags.iter().map(Tag::to_json).collect()),
            Json::Object(self.content.clone()),
        ])
        .to_canonical_vec()
    }

    /// `id = SHA-256(payload)`.
    #[must_use]
    pub fn id(&self) -> [u8; 32] {
        sha256(&self.payload())
    }

    /// Calcula el identificador y firma con `key`, que debe corresponder a `signer`.
    ///
    /// El resultado pasa por [`Event::parse_and_verify`] sobre su forma canónica: `sign`
    /// aplica exactamente las mismas reglas que un receptor y nunca devuelve un evento
    /// que la red rechazaría.
    ///
    /// # Errors
    /// `E_PUBKEY_INVALID` si `key` no corresponde a `signer`, o el código del primer paso
    /// de la validación universal (1 a 4) que el evento no cumpla.
    pub fn sign(self, key: &SigningKey) -> Result<Event, CoreError> {
        if key.verifying_key().to_bytes() != self.signer {
            return Err(CoreError::new(
                ErrorCode::PublicKeyInvalid,
                "signer: la llave de firma no corresponde a signer",
            ));
        }
        Event::parse_and_verify(&self.sign_unchecked(key).to_canonical_vec())
    }

    /// Firma sin validar nada. Solo para construir vectores de eventos inválidos.
    pub(crate) fn sign_unchecked(self, key: &SigningKey) -> Event {
        let id = self.id();
        let sig = key.sign(&id).to_bytes();
        Event {
            unsigned: self,
            id,
            sig,
        }
    }
}

/// Evento firmado y verificado en sus pasos 1 a 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    unsigned: UnsignedEvent,
    id: [u8; 32],
    sig: [u8; SIGNATURE_LEN],
}

impl Event {
    /// Analiza un evento recibido y aplica los pasos 1 a 4 de la validación universal:
    /// tamaño, esquema de la envoltura, identificador y firma estricta.
    ///
    /// # Errors
    /// El [`CoreError`] del primer paso que falle, con su código estable.
    pub fn parse_and_verify(bytes: &[u8]) -> Result<Self, CoreError> {
        let event = Self::parse_unverified(bytes)?;
        event.verify()?;
        Ok(event)
    }

    /// Pasos 1 (tamaño) y 2 (JSON estricto y esquema de la envoltura), sin id ni firma.
    /// La validación por tipo intercala aquí el esquema del contenido (`docs/06` §3).
    pub(crate) fn parse_unverified(bytes: &[u8]) -> Result<Self, CoreError> {
        // Paso 1: tamaño, antes de analizar nada.
        if bytes.len() > MAX_EVENT_BYTES {
            return Err(CoreError::new(
                ErrorCode::EventTooLarge,
                format!("{} bytes; el máximo es {MAX_EVENT_BYTES}", bytes.len()),
            ));
        }
        // Paso 2: JSON estricto, forma canónica exacta (D-17) y esquema de la envoltura.
        let value = Json::parse(bytes)?;
        if value.to_canonical_vec() != bytes {
            return Err(CoreError::new(
                ErrorCode::JsonNotCanonical,
                "los bytes no son la forma canónica JCS del evento",
            ));
        }
        Self::from_json(&value)
    }

    /// Pasos 3 (identificador) y 4 (firma estricta de `signer` sobre los bytes de `id`).
    pub(crate) fn verify(&self) -> Result<(), CoreError> {
        if self.unsigned.id() != self.id {
            return Err(CoreError::new(
                ErrorCode::IdMismatch,
                "id distinto del calculado",
            ));
        }
        let signer = VerifyingKey::from_bytes(&self.unsigned.signer)
            .map_err(|_| CoreError::new(ErrorCode::PublicKeyInvalid, "signer"))?;
        signer
            .verify_strict(&self.id, &Signature::from_bytes(self.sig))
            .map_err(|_| CoreError::new(ErrorCode::SigInvalid, "firma inválida para signer"))
    }

    /// Solo el esquema de la envoltura, sin comprobar id ni firma (para vectores).
    pub(crate) fn from_json_unchecked(value: &Json) -> Result<UnsignedEvent, CoreError> {
        Self::from_json(value).map(|e| e.unsigned)
    }

    fn from_json(value: &Json) -> Result<Self, CoreError> {
        let obj = value
            .as_object()
            .ok_or_else(|| CoreError::new(ErrorCode::Schema, "el evento debe ser un objeto"))?;
        if let Some(extra) = obj.keys().find(|k| !ENVELOPE_FIELDS.contains(&k.as_str())) {
            return Err(CoreError::new(ErrorCode::UnknownField, extra.clone()));
        }
        let field = |name: &str| {
            obj.get(name)
                .ok_or_else(|| CoreError::new(ErrorCode::Schema, format!("{name}: falta")))
        };
        let string = |name: &str| {
            field(name)?.as_str().ok_or_else(|| {
                CoreError::new(ErrorCode::Schema, format!("{name}: debe ser una cadena"))
            })
        };
        let int = |name: &str| {
            field(name)?.as_int().ok_or_else(|| {
                CoreError::new(ErrorCode::Schema, format!("{name}: debe ser un entero"))
            })
        };

        if int("v")? != PROTOCOL_VERSION {
            return Err(CoreError::new(ErrorCode::VersionUnsupported, "v"));
        }
        let id = decode_fixed::<32>(string("id")?, "id")?;
        let account = decode_fixed::<PUBLIC_KEY_LEN>(string("account")?, "account")?;
        let signer = decode_fixed::<PUBLIC_KEY_LEN>(string("signer")?, "signer")?;
        let sig = decode_fixed::<SIGNATURE_LEN>(string("sig")?, "sig")?;
        let created_at = int("created_at")?;
        check_created_at(created_at)?;
        let kind = string("kind")?.to_owned();
        check_kind(&kind)?;

        let tags_json = field("tags")?
            .as_array()
            .ok_or_else(|| CoreError::new(ErrorCode::Schema, "tags: debe ser una lista"))?;
        if tags_json.len() > MAX_TAGS {
            return Err(CoreError::new(ErrorCode::Schema, "tags: máximo 16"));
        }
        let tags = tags_json
            .iter()
            .enumerate()
            .map(|(i, t)| Tag::from_json(t, i))
            .collect::<Result<Vec<_>, _>>()?;

        let content = field("content")?
            .as_object()
            .ok_or_else(|| CoreError::new(ErrorCode::Schema, "content: debe ser un objeto"))?
            .clone();

        for (name, key) in [("account", &account), ("signer", &signer)] {
            VerifyingKey::from_bytes(key)
                .map_err(|_| CoreError::new(ErrorCode::PublicKeyInvalid, name))?;
        }

        Ok(Self {
            unsigned: UnsignedEvent {
                account,
                signer,
                created_at,
                kind,
                tags,
                content,
            },
            id,
            sig,
        })
    }

    /// El evento completo como JSON canónico, tal como viaja por la red.
    #[must_use]
    pub fn to_json(&self) -> Json {
        let u = &self.unsigned;
        let mut obj = BTreeMap::new();
        obj.insert("v".into(), Json::Int(PROTOCOL_VERSION));
        obj.insert("id".into(), hex::encode(self.id).into());
        obj.insert("account".into(), hex::encode(u.account).into());
        obj.insert("signer".into(), hex::encode(u.signer).into());
        obj.insert("created_at".into(), Json::Int(u.created_at));
        obj.insert("kind".into(), u.kind.as_str().into());
        obj.insert(
            "tags".into(),
            Json::Array(u.tags.iter().map(Tag::to_json).collect()),
        );
        obj.insert("content".into(), Json::Object(u.content.clone()));
        obj.insert("sig".into(), hex::encode(self.sig).into());
        Json::Object(obj)
    }

    /// Bytes canónicos del evento completo.
    #[must_use]
    pub fn to_canonical_vec(&self) -> Vec<u8> {
        self.to_json().to_canonical_vec()
    }

    /// Identificador (SHA-256 del payload).
    #[must_use]
    pub const fn id(&self) -> &[u8; 32] {
        &self.id
    }

    /// Firma Ed25519 de `signer` sobre `id`.
    #[must_use]
    pub const fn sig(&self) -> &[u8; SIGNATURE_LEN] {
        &self.sig
    }

    /// Campos firmados.
    #[must_use]
    pub const fn unsigned(&self) -> &UnsignedEvent {
        &self.unsigned
    }

    /// Llave pública raíz de la cuenta.
    #[must_use]
    pub const fn account(&self) -> &[u8; PUBLIC_KEY_LEN] {
        &self.unsigned.account
    }

    /// Llave que firmó.
    #[must_use]
    pub const fn signer(&self) -> &[u8; PUBLIC_KEY_LEN] {
        &self.unsigned.signer
    }

    /// Tipo de evento.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.unsigned.kind
    }

    /// Segundos Unix.
    #[must_use]
    pub const fn created_at(&self) -> i64 {
        self.unsigned.created_at
    }

    /// Contenido.
    #[must_use]
    pub const fn content(&self) -> &BTreeMap<String, Json> {
        &self.unsigned.content
    }

    /// Etiquetas.
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.unsigned.tags
    }
}

fn check_created_at(created_at: i64) -> Result<(), CoreError> {
    if created_at < 0 {
        return Err(CoreError::new(
            ErrorCode::Schema,
            "created_at: no puede ser negativo",
        ));
    }
    Ok(())
}

/// `kind`: segmentos en minúsculas ASCII separados por puntos, como `post.create`.
pub(crate) fn check_kind(kind: &str) -> Result<(), CoreError> {
    let segment_ok = |s: &str| {
        let mut chars = s.chars();
        chars.next().is_some_and(|c| c.is_ascii_lowercase())
            && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    };
    let valid = kind.len() <= MAX_KIND_BYTES
        && kind.split('.').count() >= 2
        && kind.split('.').all(segment_ok);
    if valid {
        Ok(())
    } else {
        Err(CoreError::new(
            ErrorCode::Schema,
            "kind: segmentos [a-z][a-z0-9_]* separados por puntos, máximo 64 bytes",
        ))
    }
}
