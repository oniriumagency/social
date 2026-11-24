//! Valor JSON estricto del protocolo y su forma canónica (JCS, RFC 8785).
//!
//! El protocolo solo admite un subconjunto de JSON (`docs/06` §1):
//! - Sin `null` y sin números decimales ni exponenciales.
//! - Enteros en el rango ±(2⁵³ − 1), representables sin pérdida en JavaScript.
//! - Sin claves repetidas en un objeto (I-JSON, RFC 7493, que JCS exige).
//! - Como mucho [`MAX_DEPTH`] niveles de listas y objetos anidados.
//!
//! Con esas restricciones, JCS se reduce a: claves ordenadas por unidades UTF-16, sin
//! espacios, enteros en decimal y cadenas escapadas como `JSON.stringify` de ECMAScript.
//! Por eso se implementa aquí en pocas líneas, sin dependencia externa, y se prueba
//! contra los ejemplos del RFC y contra la serialización de JavaScript en `social-core-wasm`.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};

use crate::{CoreError, ErrorCode};

/// Mayor entero admitido: 2⁵³ − 1 (`Number.MAX_SAFE_INTEGER`).
pub const MAX_SAFE_INT: i64 = 9_007_199_254_740_991;

/// Máximo de niveles de listas y objetos anidados en un documento, contando el exterior
/// (`docs/06` §1). Un nivel más es `E_JSON_INVALID`. serde_json aplica por su cuenta el
/// mismo límite; este contador fija la regla del protocolo aunque cambie el analizador.
pub const MAX_DEPTH: usize = 127;

/// Valor JSON del protocolo. No puede representar `null` ni decimales.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Json {
    /// `true` o `false`.
    Bool(bool),
    /// Entero en ±(2⁵³ − 1).
    Int(i64),
    /// Cadena UTF-8.
    String(String),
    /// Lista.
    Array(Vec<Json>),
    /// Objeto. El orden de `BTreeMap` no importa: la forma canónica reordena por UTF-16.
    Object(BTreeMap<String, Json>),
}

impl Json {
    /// Analiza bytes JSON aplicando las restricciones del protocolo.
    ///
    /// Un literal entero sin parte decimal ni exponente que no cabe en ±(2⁵³ − 1) es
    /// `E_JSON_INT_RANGE`, también si no cabe en 64 bits. `-0` es `E_JSON_FLOAT`: no
    /// tiene forma canónica como entero.
    ///
    /// # Errors
    /// `E_JSON_INVALID`, `E_JSON_DUPLICATE_KEY`, `E_JSON_FLOAT`, `E_JSON_NULL` o
    /// `E_JSON_INT_RANGE`.
    pub fn parse(bytes: &[u8]) -> Result<Self, CoreError> {
        let mut de = serde_json::Deserializer::from_slice(bytes);
        let value = de
            .deserialize_any(StrictVisitor { depth: 1 })
            .map_err(|e| map_serde_error(e, bytes))?;
        de.end().map_err(|e| map_serde_error(e, bytes))?;
        Ok(value)
    }

    /// Construye un entero comprobando el rango.
    ///
    /// # Errors
    /// `E_JSON_INT_RANGE` si queda fuera de ±(2⁵³ − 1).
    pub fn int(n: i64) -> Result<Self, CoreError> {
        if (-MAX_SAFE_INT..=MAX_SAFE_INT).contains(&n) {
            Ok(Self::Int(n))
        } else {
            Err(CoreError::new(
                ErrorCode::JsonIntRange,
                "entero fuera de ±(2^53 − 1)",
            ))
        }
    }

    /// Forma canónica JCS (RFC 8785) en bytes UTF-8.
    #[must_use]
    pub fn to_canonical_vec(&self) -> Vec<u8> {
        let mut out = String::new();
        self.write_canonical(&mut out);
        out.into_bytes()
    }

    /// Forma canónica JCS como cadena.
    #[must_use]
    pub fn to_canonical_string(&self) -> String {
        let mut out = String::new();
        self.write_canonical(&mut out);
        out
    }

    fn write_canonical(&self, out: &mut String) {
        match self {
            Self::Bool(true) => out.push_str("true"),
            Self::Bool(false) => out.push_str("false"),
            Self::Int(n) => out.push_str(&n.to_string()),
            Self::String(s) => write_string(s, out),
            Self::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write_canonical(out);
                }
                out.push(']');
            }
            Self::Object(map) => {
                // RFC 8785 §3.2.3: orden por unidades de código UTF-16, no por bytes UTF-8.
                let mut entries: Vec<(&String, &Json)> = map.iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
                out.push('{');
                for (i, (key, value)) in entries.into_iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_string(key, out);
                    out.push(':');
                    value.write_canonical(out);
                }
                out.push('}');
            }
        }
    }

    /// Devuelve el objeto, si lo es.
    #[must_use]
    pub const fn as_object(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Self::Object(m) => Some(m),
            _ => None,
        }
    }

    /// Devuelve la cadena, si lo es.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Devuelve el entero, si lo es.
    #[must_use]
    pub const fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Devuelve la lista, si lo es.
    #[must_use]
    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Devuelve el booleano, si lo es.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Nombre del tipo, para mensajes de error.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "booleano",
            Self::Int(_) => "entero",
            Self::String(_) => "cadena",
            Self::Array(_) => "lista",
            Self::Object(_) => "objeto",
        }
    }
}

impl From<&str> for Json {
    fn from(s: &str) -> Self {
        Self::String(s.to_owned())
    }
}

impl From<String> for Json {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<bool> for Json {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

/// Escapa una cadena como `JSON.stringify` (RFC 8785 §3.2.2.2).
fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

// Los errores del visitante llevan el código estable como mensaje; `map_serde_error`
// lo recupera. serde_json añade la posición al final del mensaje.
const TAG_FLOAT: &str = "E_JSON_FLOAT";
const TAG_NULL: &str = "E_JSON_NULL";
const TAG_DUP: &str = "E_JSON_DUPLICATE_KEY";
const TAG_RANGE: &str = "E_JSON_INT_RANGE";
const TAG_DEPTH: &str = "E_JSON_INVALID: profundidad";

fn map_serde_error(err: serde_json::Error, bytes: &[u8]) -> CoreError {
    let msg = err.to_string();
    let code = [
        (TAG_FLOAT, ErrorCode::JsonFloat),
        (TAG_NULL, ErrorCode::JsonNull),
        (TAG_DUP, ErrorCode::JsonDuplicateKey),
        (TAG_RANGE, ErrorCode::JsonIntRange),
    ]
    .into_iter()
    .find(|(tag, _)| msg.starts_with(tag))
    .map_or(ErrorCode::JsonInvalid, |(_, code)| code);
    // serde_json entrega como f64 tanto los decimales como los enteros que no caben en
    // 64 bits. El primer número del documento que no es un entero de 64 bits es el que
    // falló, porque el análisis va en orden: se mira su forma literal.
    let code = if code == ErrorCode::JsonFloat {
        first_non_i64_number(bytes).map_or(code, classify_number)
    } else {
        code
    };
    CoreError::new(code, msg)
}

/// Código de un literal numérico que serde_json no entrega como entero.
fn classify_number(token: &[u8]) -> ErrorCode {
    let is_integer_syntax = !token.iter().any(|b| matches!(b, b'.' | b'e' | b'E'));
    if is_integer_syntax && token != b"-0" {
        ErrorCode::JsonIntRange
    } else {
        ErrorCode::JsonFloat
    }
}

/// Primer literal numérico, fuera de cadenas, que no es un entero de 64 bits.
fn first_non_i64_number(bytes: &[u8]) -> Option<&[u8]> {
    let mut i = 0;
    let mut in_string = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            match b {
                b'\\' => i += 1,
                b'"' => in_string = false,
                _ => {}
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if b == b'-' || b.is_ascii_digit() {
            let start = i;
            while i < bytes.len()
                && matches!(bytes[i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
            {
                i += 1;
            }
            let token = &bytes[start..i];
            let as_int = std::str::from_utf8(token)
                .ok()
                .filter(|t| t.parse::<i64>().is_ok() || t.parse::<u64>().is_ok());
            if as_int.is_none() || token == b"-0" {
                return Some(token);
            }
            continue;
        }
        i += 1;
    }
    None
}

/// Visitante estricto. `depth` es el nivel del valor que visita: 1 para el exterior.
struct StrictVisitor {
    depth: usize,
}

impl StrictVisitor {
    fn enter<E: de::Error>(&self) -> Result<StrictSeed, E> {
        if self.depth > MAX_DEPTH {
            return Err(E::custom(TAG_DEPTH));
        }
        Ok(StrictSeed {
            depth: self.depth + 1,
        })
    }
}

impl<'de> Visitor<'de> for StrictVisitor {
    type Value = Json;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("un valor JSON del protocolo")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Json, E> {
        Ok(Json::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Json, E> {
        Json::int(v).map_err(|_| E::custom(TAG_RANGE))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Json, E> {
        i64::try_from(v)
            .ok()
            .and_then(|n| Json::int(n).ok())
            .ok_or_else(|| E::custom(TAG_RANGE))
    }

    fn visit_f64<E: de::Error>(self, _v: f64) -> Result<Json, E> {
        // Incluye "1.0", "1e3" y enteros que no caben en 64 bits.
        Err(E::custom(TAG_FLOAT))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Json, E> {
        Ok(Json::String(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Json, E> {
        Ok(Json::String(v))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Json, E> {
        Err(E::custom(TAG_NULL))
    }

    fn visit_none<E: de::Error>(self) -> Result<Json, E> {
        Err(E::custom(TAG_NULL))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Json, A::Error> {
        let child = self.enter()?;
        let mut items = Vec::new();
        while let Some(item) = seq.next_element_seed(child)? {
            items.push(item);
        }
        Ok(Json::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Json, A::Error> {
        let child = self.enter()?;
        let mut out = BTreeMap::new();
        while let Some(key) = map.next_key::<String>()? {
            let value = map.next_value_seed(child)?;
            if out.insert(key, value).is_some() {
                return Err(de::Error::custom(TAG_DUP));
            }
        }
        Ok(Json::Object(out))
    }
}

#[derive(Clone, Copy)]
struct StrictSeed {
    depth: usize,
}

impl<'de> de::DeserializeSeed<'de> for StrictSeed {
    type Value = Json;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Json, D::Error> {
        d.deserialize_any(StrictVisitor { depth: self.depth })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(input: &str) -> ErrorCode {
        Json::parse(input.as_bytes()).unwrap_err().code
    }

    #[test]
    fn rejects_forbidden_values() {
        assert_eq!(code("1.0"), ErrorCode::JsonFloat);
        assert_eq!(code("1e3"), ErrorCode::JsonFloat);
        assert_eq!(code("[0.5]"), ErrorCode::JsonFloat);
        assert_eq!(code(r#"{"a":null}"#), ErrorCode::JsonNull);
        assert_eq!(code(r#"{"a":1,"a":2}"#), ErrorCode::JsonDuplicateKey);
        assert_eq!(code("9007199254740992"), ErrorCode::JsonIntRange);
        assert_eq!(code("-9007199254740992"), ErrorCode::JsonIntRange);
        assert_eq!(code("18446744073709551616"), ErrorCode::JsonIntRange);
        assert_eq!(code("-9223372036854775809"), ErrorCode::JsonIntRange);
        assert_eq!(
            code("[1,\"-0.5 99999999999999999999\",99999999999999999999]"),
            ErrorCode::JsonIntRange
        );
        assert_eq!(code("[99999999999999999999,1.5]"), ErrorCode::JsonIntRange);
        assert_eq!(code("[1.5,99999999999999999999]"), ErrorCode::JsonFloat);
        assert_eq!(code("-0"), ErrorCode::JsonFloat);
        assert_eq!(code("[-0]"), ErrorCode::JsonFloat);
        assert_eq!(code("1E3"), ErrorCode::JsonFloat);
        assert_eq!(code("{"), ErrorCode::JsonInvalid);
        assert_eq!(code("1 2"), ErrorCode::JsonInvalid);
        assert_eq!(code("\"\\ud800\""), ErrorCode::JsonInvalid);
    }

    #[test]
    fn accepts_extreme_safe_integers() {
        assert_eq!(
            Json::parse(b"9007199254740991").unwrap(),
            Json::Int(MAX_SAFE_INT)
        );
        assert_eq!(
            Json::parse(b"-9007199254740991").unwrap(),
            Json::Int(-MAX_SAFE_INT)
        );
    }

    fn nested(levels: usize) -> String {
        format!("{}{}", "[".repeat(levels), "]".repeat(levels))
    }

    #[test]
    fn depth_limit() {
        assert!(Json::parse(nested(MAX_DEPTH).as_bytes()).is_ok());
        assert_eq!(code(&nested(MAX_DEPTH + 1)), ErrorCode::JsonInvalid);
        let objects = format!("{}1{}", "{\"a\":".repeat(MAX_DEPTH), "}".repeat(MAX_DEPTH));
        assert!(Json::parse(objects.as_bytes()).is_ok());
        let objects = format!("[{objects}]");
        assert_eq!(code(&objects), ErrorCode::JsonInvalid);
    }

    /// El contador propio actúa aunque el analizador admitiera más niveles.
    #[test]
    fn own_depth_counter_rejects() {
        let mut de = serde_json::Deserializer::from_str("[[]]");
        let err = de
            .deserialize_any(StrictVisitor { depth: MAX_DEPTH })
            .unwrap_err();
        assert_eq!(map_serde_error(err, b"").code, ErrorCode::JsonInvalid);
        let custom = <serde_json::Error as de::Error>::custom(TAG_DEPTH);
        assert!(map_serde_error(custom, b"").detail.contains("profundidad"));
    }

    /// RFC 8785 §3.2.2.2: ejemplo de escape de cadenas.
    #[test]
    fn rfc8785_string_escaping() {
        let input = "\"\\u20ac$\\u000F\\u000aA'\\u0042\\u0022\\u005c\\\\\\\"\\/\"";
        let v = Json::parse(input.as_bytes()).unwrap();
        assert_eq!(
            v.to_canonical_string(),
            "\"€$\\u000f\\nA'B\\\"\\\\\\\\\\\"/\""
        );
    }

    /// RFC 8785 §3.2.3: orden de claves por unidades UTF-16.
    #[test]
    fn rfc8785_key_order() {
        let input = r#"{"\u20ac":"Euro Sign","\r":"Carriage Return","\ufb33":"Hebrew Letter Dalet With Dagesh","1":"One","\ud83d\ude00":"Emoji: Grinning Face","\u0080":"Control","\u00f6":"Latin Small Letter O With Diaeresis"}"#;
        let v = Json::parse(input.as_bytes()).unwrap();
        let Json::Object(map) = &v else { panic!() };
        let mut keys: Vec<&String> = map.keys().collect();
        keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
        let values: Vec<&str> = keys.iter().map(|k| map[*k].as_str().unwrap()).collect();
        assert_eq!(
            values,
            [
                "Carriage Return",
                "One",
                "Control",
                "Latin Small Letter O With Diaeresis",
                "Euro Sign",
                "Emoji: Grinning Face",
                "Hebrew Letter Dalet With Dagesh",
            ]
        );
        let canon = v.to_canonical_string();
        assert!(canon.starts_with("{\"\\r\":\"Carriage Return\",\"1\":\"One\""));
        assert!(canon.find("Emoji").unwrap() < canon.find("Hebrew").unwrap());
    }

    #[test]
    fn canonical_is_compact_and_idempotent() {
        let v = Json::parse(br#" { "b" : [ 1 , true , "x" ] , "a" : { "z" : -3 } } "#).unwrap();
        let c = v.to_canonical_string();
        assert_eq!(c, r#"{"a":{"z":-3},"b":[1,true,"x"]}"#);
        assert_eq!(Json::parse(c.as_bytes()).unwrap().to_canonical_string(), c);
    }
}
