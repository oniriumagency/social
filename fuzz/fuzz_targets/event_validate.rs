//! Validación universal completa sobre bytes crudos con el contexto de los vectores
//! dorados: nunca entra en pánico. Ejercita análisis, envoltura y esquema por tipo; los
//! pasos 5 y 6 con entradas nuevas los cubre `event_build`, que firma lo que genera.
#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "context.rs"]
mod context;

fuzz_target!(|data: &[u8]| {
    let _ = social_core::validate_event(data, context::context());
});
