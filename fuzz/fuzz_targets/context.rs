//! Contexto de validación de `spec/vectors/validation.json`, compartido por los objetivos
//! que llegan a los pasos 5 y 6. Si no se puede cargar, el fuzzing se detiene: seguir con
//! un contexto vacío daría verde sin probar papeles, delegaciones ni atestaciones.

use std::sync::OnceLock;

use social_core::Json;
use social_core::vectors::MemoryContext;

pub fn context() -> &'static MemoryContext {
    static CTX: OnceLock<MemoryContext> = OnceLock::new();
    CTX.get_or_init(|| {
        let text = include_str!("../../spec/vectors/validation.json");
        let ctx = Json::parse(text.as_bytes())
            .ok()
            .and_then(|suite| social_core::vectors::load_context(&suite));
        match ctx {
            Some(ctx) if !ctx.grants.is_empty() && !ctx.attestations.is_empty() => ctx,
            _ => {
                eprintln!(
                    "fuzz: no se pudo cargar el contexto de spec/vectors/validation.json; \
                     sin él los pasos 5 y 6 no se prueban."
                );
                std::process::exit(2);
            }
        }
    })
}
