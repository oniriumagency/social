//! Prototipo del spike S4 (`docs/12` F0-S4, `docs/08` §2.1).
//!
//! Lee cada segundo las señales que usará el gobernador de recursos (batería y corriente,
//! red medida, temperatura, CPU, memoria, disco libre y actividad del usuario), mide cuánto
//! tarda cada lectura, anota cada cambio que observa y escribe un informe de texto.
//!
//! El informe no lleva nombre del equipo, usuario, direcciones de red ni números de serie:
//! solo el sistema operativo, la arquitectura y los valores de las señales (I-4).
//!
//! Uso: `pnpm spike:gobernador [segundos] [archivo]` (por defecto 600 s y
//! `governor-probe-<sistema>.txt`).

mod platform;
mod report;

use std::io::Write;
use std::time::{Duration, Instant};

use report::{Report, Signal};

/// Intervalo de muestreo. La reacción exigida es de 5 s o menos (`docs/08` §2.2).
const INTERVAL: Duration = Duration::from_secs(1);
/// Segundos sin entrada a partir de los cuales el equipo se considera en reposo.
const IDLE_AFTER_SECS: u64 = 60;

/// Valor discreto de cada señal: lo que se compara para detectar cambios.
#[derive(Clone, PartialEq, Eq, Default)]
struct Discrete {
    power: Option<String>,
    metered: Option<String>,
    thermal: Option<String>,
    user: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).filter(|a| a != "--").collect();
    let seconds = match args.first().map(|s| s.parse::<u64>()) {
        None => 600,
        Some(Ok(n)) if n > 0 => n,
        Some(_) => {
            let _ = writeln!(
                std::io::stderr(),
                "uso: pnpm spike:gobernador [segundos] [archivo]"
            );
            std::process::exit(2);
        }
    };
    let out_path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| format!("governor-probe-{}.txt", std::env::consts::OS));

    let mut probe = platform::Probe::new();
    let mut report = Report::new(seconds, INTERVAL);
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(
        stdout,
        "Sonda del gobernador: {seconds} s. Durante la prueba: desenchufa y enchufa el cargador, \
         cambia de wifi a datos del celular y vuelve, y deja el equipo quieto 3 minutos."
    );

    let start = Instant::now();
    let mut previous = Discrete::default();
    let mut first = true;
    while start.elapsed() < Duration::from_secs(seconds) {
        let tick = Instant::now();
        let mut now = Discrete::default();

        let power = timed(|| probe.power());
        report.record(Signal::Power, &power);
        now.power = power.0.ok().flatten().map(|p| p.state);

        let metered = timed(|| probe.metered());
        report.record(Signal::Metered, &metered);
        now.metered = metered.0.ok().flatten();

        let temp = timed(|| probe.temperature());
        report.record(Signal::Temperature, &temp);

        let thermal = timed(|| probe.thermal_state());
        report.record(Signal::ThermalState, &thermal);
        now.thermal = thermal.0.ok().flatten();

        let cpu = timed(|| probe.cpu_and_memory());
        report.record(Signal::CpuMemory, &cpu);

        let disk = timed(|| probe.disk_free());
        report.record(Signal::DiskFree, &disk);

        let idle = timed(|| probe.idle_seconds());
        report.record(Signal::UserIdle, &idle);
        now.user = match idle.0 {
            Ok(Some(secs)) if secs >= IDLE_AFTER_SECS => Some("en reposo".to_owned()),
            Ok(Some(_)) => Some("en uso".to_owned()),
            _ => None,
        };

        let at = start.elapsed();
        if first {
            report.initial(at, &now.power, &now.metered, &now.thermal, &now.user);
            first = false;
        } else {
            for (signal, old, new) in [
                (Signal::Power, &previous.power, &now.power),
                (Signal::Metered, &previous.metered, &now.metered),
                (Signal::ThermalState, &previous.thermal, &now.thermal),
                (Signal::UserIdle, &previous.user, &now.user),
            ] {
                if old != new {
                    let line = report.change(at, signal, old, new);
                    let _ = writeln!(stdout, "{line}");
                }
            }
        }
        previous = now;

        if let Some(rest) = INTERVAL.checked_sub(tick.elapsed()) {
            std::thread::sleep(rest);
        }
    }

    let text = report.render(&platform::system_description());
    match std::fs::write(&out_path, &text) {
        Ok(()) => {
            let _ = writeln!(
                stdout,
                "Informe guardado en {out_path}. Pulsa Enter para cerrar."
            );
            let _ = stdout.flush();
            // Abierto con doble clic, la ventana se cerraría antes de leer el mensaje.
            let _ = std::io::stdin().read_line(&mut String::new());
        }
        Err(e) => {
            let _ = writeln!(
                std::io::stderr(),
                "no se pudo escribir {out_path}: {e}. Informe:\n{text}"
            );
            std::process::exit(1);
        }
    }
}

/// Ejecuta una lectura y devuelve su resultado con lo que tardó.
fn timed<T>(read: impl FnOnce() -> Result<T, String>) -> (Result<T, String>, Duration) {
    let start = Instant::now();
    let value = read();
    (value, start.elapsed())
}
