//! Validación universal completa con el contexto de los vectores dorados: nunca entra en
//! pánico. El corpus inicial son los eventos de `spec/vectors/` (`pnpm fuzz` lo siembra).
#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use social_core::Json;
use social_core::vectors::MemoryContext;

fn context() -> &'static MemoryContext {
    static CTX: OnceLock<MemoryContext> = OnceLock::new();
    CTX.get_or_init(|| {
        let text = include_str!("../../spec/vectors/validation.json");
        Json::parse(text.as_bytes())
            .ok()
            .and_then(|suite| social_core::vectors::load_context(&suite))
            .unwrap_or_default()
    })
}

fuzz_target!(|data: &[u8]| {
    let _ = social_core::validate_event(data, context());
});
