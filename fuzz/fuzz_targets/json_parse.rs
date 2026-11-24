//! JSON estricto: nunca entra en pánico, y lo que acepta se canonicaliza de forma estable.
#![no_main]

use libfuzzer_sys::fuzz_target;
use social_core::Json;

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = Json::parse(data) {
        let canonical = value.to_canonical_string();
        let reparsed = Json::parse(canonical.as_bytes());
        assert_eq!(
            reparsed.as_ref().ok(),
            Some(&value),
            "la forma canónica no se relee igual"
        );
        let again = reparsed.map(|v| v.to_canonical_string());
        assert_eq!(
            again.as_deref().ok(),
            Some(canonical.as_str()),
            "canonicalizar no es idempotente"
        );
    }
});
