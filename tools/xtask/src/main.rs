//! Tareas de mantenimiento del repositorio escritas en Rust para que funcionen igual
//! en Windows, macOS y Linux (ADR-015: sin scripts de shell).
//!
//! Cada subcomando lo añade la tarea del plan que lo necesita (por ejemplo, `seed`
//! en F1). Hoy solo existe `help`.

use std::process::ExitCode;

const USAGE: &str =
    "uso: cargo run -p xtask -- <subcomando>\n\nsubcomandos:\n  help    muestra esta ayuda\n";

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        None | Some("help" | "--help" | "-h") => {
            eprint!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("xtask: subcomando desconocido `{other}`\n");
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
