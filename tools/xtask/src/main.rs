//! Tareas de mantenimiento del repositorio escritas en Rust para que funcionen igual
//! en Windows, macOS y Linux (ADR-015: sin scripts de shell).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use social_core::CoreError;
use social_core::vectors::{generate_envelope_suite, generate_validation_suite};

const USAGE: &str = "uso: cargo run -p xtask -- <subcomando>\n\nsubcomandos:\n  spec           regenera spec/schemas/ desde social-core\n  spec --check   falla si spec/schemas/ no coincide con lo generado\n  vectors        regenera spec/vectors/ (cambio de protocolo: exige ADR)\n  help           muestra esta ayuda\n";

type Generator = fn() -> Result<String, CoreError>;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn vectors() -> ExitCode {
    let suites: [(&str, Generator); 2] = [
        ("envelope.json", generate_envelope_suite),
        ("validation.json", generate_validation_suite),
    ];
    for (file, generate) in suites {
        let suite = match generate() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("xtask vectors: {file}: {e}");
                return ExitCode::FAILURE;
            }
        };
        let path = repo_root().join("spec").join("vectors").join(file);
        if let Err(e) = std::fs::write(&path, suite) {
            eprintln!("xtask vectors: no se pudo escribir {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        eprintln!("xtask vectors: escrito {}", path.display());
    }
    ExitCode::SUCCESS
}

/// Archivos `*.json` bajo `dir`, como rutas relativas con `/`.
fn list_json(dir: &Path, prefix: &str, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            list_json(&path, &format!("{prefix}{name}/"), out);
        } else if name.ends_with(".json") {
            out.push(format!("{prefix}{name}"));
        }
    }
}

fn spec(check: bool) -> ExitCode {
    let dir = repo_root().join("spec").join("schemas");
    let generated = social_core::schema::generate_schemas();
    let mut problems = Vec::new();
    for (rel, content) in &generated {
        let path = dir.join(rel);
        let current = std::fs::read_to_string(&path)
            .ok()
            .map(|s| s.replace("\r\n", "\n"));
        if current.as_deref() == Some(content.as_str()) {
            continue;
        }
        if check {
            problems.push(format!("difiere o falta: spec/schemas/{rel}"));
            continue;
        }
        if let Some(parent) = path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            eprintln!("xtask spec: {}: {e}", parent.display());
            return ExitCode::FAILURE;
        }
        if let Err(e) = std::fs::write(&path, content) {
            eprintln!("xtask spec: {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        eprintln!("xtask spec: escrito spec/schemas/{rel}");
    }
    let mut existing = Vec::new();
    list_json(&dir, "", &mut existing);
    for rel in existing {
        if !generated.iter().any(|(g, _)| *g == rel) {
            problems.push(format!("sobra (ya no se genera): spec/schemas/{rel}"));
        }
    }
    if problems.is_empty() {
        eprintln!(
            "xtask spec: spec/schemas/ al día ({} esquemas).",
            generated.len()
        );
        ExitCode::SUCCESS
    } else {
        for p in &problems {
            eprintln!("xtask spec: {p}");
        }
        eprintln!(
            "Ejecuta `pnpm spec:gen` y revisa el cambio: un esquema distinto es un cambio de protocolo."
        );
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("vectors") => vectors(),
        Some("spec") => spec(std::env::args().nth(2).as_deref() == Some("--check")),
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
