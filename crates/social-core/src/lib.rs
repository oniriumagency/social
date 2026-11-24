//! # social-core
//!
//! Implementación de referencia del protocolo de Social (`docs/06`, ADR-007): el valor
//! JSON estricto del protocolo, la canonicalización JCS (RFC 8785), la envoltura de
//! evento, su identificador, su firma y su verificación.
//!
//! La misma biblioteca se compila a nativo (servicios, nodo, escritorio) y a WASM (web),
//! para que todos los participantes validen exactamente igual. Toda la criptografía pasa
//! por `social-crypto` (I-8).

#![forbid(unsafe_code)]

mod error;
mod event;
mod hexfmt;
pub mod json;
pub mod kinds;
pub mod schema;
mod validate;
pub mod vectors;
mod vectors_validation;

pub use error::{CoreError, ErrorCode};
pub use event::{
    ENVELOPE_FIELDS, Event, MAX_EVENT_BYTES, MAX_TAGS, PROTOCOL_VERSION, Tag, UnsignedEvent,
};
pub use json::Json;
pub use validate::{
    Attestation, DeviceGrant, MIN_LEVEL_MEDIA, ValidationContext, validate_event, validate_parsed,
};
