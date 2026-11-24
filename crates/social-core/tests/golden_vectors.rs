//! Ejecuta los vectores dorados de `spec/vectors/` contra el núcleo nativo
//! (`pnpm test:vectors`) y comprueba que el generador no ha derivado.

use social_core::vectors::{generate_envelope_suite, run_suite};

const ENVELOPE: &str = include_str!("../../../spec/vectors/envelope.json");

#[test]
fn envelope_vectors_pass() {
    let report = run_suite(ENVELOPE);
    assert!(report.ok(), "fallos: {:#?}", report.failures);
    assert_eq!(report.passed, 29);
}

/// Si esto falla, alguien cambió el núcleo de forma que los vectores ya no coinciden.
/// Regenerar los vectores (`pnpm vectors:gen`) es un cambio de protocolo y exige ADR.
#[test]
fn envelope_vectors_have_not_drifted() {
    let generated = generate_envelope_suite();
    assert!(generated.is_ok(), "{generated:?}");
    assert!(
        generated.as_deref() == Ok(ENVELOPE.replace("\r\n", "\n").as_str()),
        "spec/vectors/envelope.json no coincide con el generador"
    );
}
