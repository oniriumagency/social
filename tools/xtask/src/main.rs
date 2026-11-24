//! Tareas de mantenimiento del repositorio escritas en Rust para que funcionen igual
//! en Windows, macOS y Linux (ADR-015: sin scripts de shell).

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "uso: cargo run -p xtask -- <subcomando>\n\nsubcomandos:\n  vectors   regenera spec/vectors/ (cambio de protocolo: exige ADR)\n  help      muestra esta ayuda\n";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn vectors() -> ExitCode {
    let suite = match social_core::vectors::generate_envelope_suite() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("xtask vectors: {e}");
            return ExitCode::FAILURE;
        }
    };
    let path = repo_root()
        .join("spec")
        .join("vectors")
        .join("envelope.json");
    if let Err(e) = std::fs::write(&path, suite) {
        eprintln!("xtask vectors: no se pudo escribir {}: {e}", path.display());
        return ExitCode::FAILURE;
    }
    eprintln!("xtask vectors: escrito {}", path.display());
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("vectors") => vectors(),
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
