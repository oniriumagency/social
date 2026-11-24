//! Prueba que falla a propósito para comprobar que CI bloquea un PR en rojo
//! (criterio de aceptación de F0-T02). Esta rama no se fusiona nunca.

#[test]
fn ci_must_go_red() {
    assert_eq!(1 + 1, 3, "fallo intencional: CI debe marcar este PR en rojo");
}
