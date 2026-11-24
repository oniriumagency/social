//! Envoltura (pasos 1 a 4): nunca entra en pánico; un evento aceptado se reserializa a
//! los mismos bytes canónicos y se vuelve a aceptar.
#![no_main]

use libfuzzer_sys::fuzz_target;
use social_core::Event;

fuzz_target!(|data: &[u8]| {
    if let Ok(event) = Event::parse_and_verify(data) {
        let wire = event.to_canonical_vec();
        let again = Event::parse_and_verify(&wire);
        assert_eq!(
            again.as_ref().ok(),
            Some(&event),
            "un evento aceptado no se vuelve a aceptar igual"
        );
    }
});
