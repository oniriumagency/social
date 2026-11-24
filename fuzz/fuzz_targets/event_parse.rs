//! Envoltura (pasos 1 a 4): nunca entra en pánico; un evento aceptado ya estaba en su forma
//! canónica (D-17), así que se reserializa a los mismos bytes y se vuelve a aceptar igual.
#![no_main]

use libfuzzer_sys::fuzz_target;
use social_core::Event;

fuzz_target!(|data: &[u8]| {
    if let Ok(event) = Event::parse_and_verify(data) {
        let wire = event.to_canonical_vec();
        assert_eq!(
            wire.as_slice(),
            data,
            "un evento aceptado no estaba en su forma canónica"
        );
        let again = Event::parse_and_verify(&wire);
        assert_eq!(
            again.as_ref().ok(),
            Some(&event),
            "un evento aceptado no se vuelve a aceptar igual"
        );
    }
});
