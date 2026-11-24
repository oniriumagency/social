//! # social-core-wasm
//!
//! Expone el núcleo de protocolo a JavaScript (web y Node) con `wasm-bindgen`.
//! No contiene lógica propia: cada función delega en `social-core`, para que el
//! navegador valide exactamente igual que los servicios (ADR-007, I-8, I-9).
//!
//! Los errores se lanzan como `Error` de JavaScript cuyo mensaje empieza por el código
//! estable (`E_...`), seguido de dos puntos y el detalle.

use social_core::vectors::run_suite;
use social_core::{CoreError, ErrorCode, Event, Json};
use wasm_bindgen::prelude::*;

fn to_js(err: &CoreError) -> JsError {
    JsError::new(&err.to_string())
}

/// Versión de la envoltura que implementa el núcleo.
#[wasm_bindgen(js_name = protocolVersion)]
#[must_use]
pub fn protocol_version() -> i32 {
    // PROTOCOL_VERSION es 1; la conversión no puede fallar.
    i32::try_from(social_core::PROTOCOL_VERSION).unwrap_or(i32::MAX)
}

/// Pasos 1 a 4 de la validación universal (tamaño, esquema de la envoltura, id y firma).
/// Devuelve el evento en JSON canónico.
///
/// # Errors
/// Lanza un `Error` con el código estable del primer paso que falla.
#[wasm_bindgen(js_name = verifyEnvelope)]
pub fn verify_envelope(event_json: &str) -> Result<String, JsError> {
    Event::parse_and_verify(event_json.as_bytes())
        .map(|e| e.to_json().to_canonical_string())
        .map_err(|e| to_js(&e))
}

/// Como `verifyEnvelope`, pero sobre los bytes exactos recibidos de la red. Una cadena de
/// JavaScript no puede llevar UTF-8 inválido: wasm-bindgen cambiaría en silencio un
/// sustituto suelto por U+FFFD. Con bytes, el núcleo ve exactamente lo que llegó (I-9).
///
/// # Errors
/// Lanza un `Error` con el código estable del primer paso que falla.
#[wasm_bindgen(js_name = verifyEnvelopeBytes)]
pub fn verify_envelope_bytes(event: &[u8]) -> Result<String, JsError> {
    Event::parse_and_verify(event)
        .map(|e| e.to_json().to_canonical_string())
        .map_err(|e| to_js(&e))
}

/// Identificador hexadecimal de un evento ya verificado.
///
/// # Errors
/// Lanza un `Error` con el código estable si el evento no se verifica.
#[wasm_bindgen(js_name = eventId)]
pub fn event_id(event_json: &str) -> Result<String, JsError> {
    Event::parse_and_verify(event_json.as_bytes())
        .map(|e| e.id().iter().map(|b| format!("{b:02x}")).collect())
        .map_err(|e| to_js(&e))
}

/// Forma canónica JCS de un JSON del protocolo (sin `null` ni decimales).
///
/// # Errors
/// Lanza un `Error` con el código estable si el JSON no es válido para el protocolo.
#[wasm_bindgen]
pub fn canonicalize(json: &str) -> Result<String, JsError> {
    Json::parse(json.as_bytes())
        .map(|v| v.to_canonical_string())
        .map_err(|e| to_js(&e))
}

/// Códigos de error estables del núcleo, en JSON (lista de cadenas).
#[wasm_bindgen(js_name = errorCodes)]
#[must_use]
pub fn error_codes() -> String {
    Json::Array(ErrorCode::ALL.iter().map(|c| c.as_str().into()).collect()).to_canonical_string()
}

/// Ejecuta un archivo de vectores dorados. Devuelve `{"passed":n,"failures":[...]}` en JSON.
#[wasm_bindgen(js_name = runVectorSuite)]
#[must_use]
pub fn run_vector_suite(text: &str) -> String {
    let report = run_suite(text);
    let passed = i64::try_from(report.passed).unwrap_or(i64::MAX);
    let mut obj = std::collections::BTreeMap::new();
    obj.insert("passed".to_owned(), Json::Int(passed));
    obj.insert(
        "failures".to_owned(),
        Json::Array(report.failures.into_iter().map(Json::String).collect()),
    );
    Json::Object(obj).to_canonical_string()
}
