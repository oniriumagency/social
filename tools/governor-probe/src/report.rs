//! Acumula las lecturas y escribe el informe de texto del spike.

use std::fmt::{Display, Write};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Power,
    Metered,
    Temperature,
    ThermalState,
    CpuMemory,
    DiskFree,
    UserIdle,
}

const SIGNALS: [Signal; 7] = [
    Signal::Power,
    Signal::Metered,
    Signal::Temperature,
    Signal::ThermalState,
    Signal::CpuMemory,
    Signal::DiskFree,
    Signal::UserIdle,
];

impl Signal {
    fn label(self) -> &'static str {
        match self {
            Signal::Power => "Batería y corriente",
            Signal::Metered => "Red medida",
            Signal::Temperature => "Temperatura (sensores)",
            Signal::ThermalState => "Estado térmico del sistema",
            Signal::CpuMemory => "CPU y memoria",
            Signal::DiskFree => "Disco libre",
            Signal::UserIdle => "Actividad del usuario",
        }
    }

    fn index(self) -> usize {
        SIGNALS.iter().position(|s| *s == self).unwrap_or(0)
    }
}

#[derive(Default)]
struct Stats {
    ok: u32,
    unavailable: u32,
    errors: u32,
    first_error: Option<String>,
    times: Vec<Duration>,
    last: Option<String>,
}

pub struct Report {
    seconds: u64,
    interval: Duration,
    stats: Vec<Stats>,
    events: Vec<String>,
}

impl Report {
    pub fn new(seconds: u64, interval: Duration) -> Self {
        Self {
            seconds,
            interval,
            stats: SIGNALS.iter().map(|_| Stats::default()).collect(),
            events: Vec::new(),
        }
    }

    /// Anota una lectura: `Ok(None)` es «no disponible en este sistema».
    pub fn record<V: Display>(
        &mut self,
        signal: Signal,
        read: &(Result<Option<V>, String>, Duration),
    ) {
        let Some(stats) = self.stats.get_mut(signal.index()) else {
            return;
        };
        stats.times.push(read.1);
        match &read.0 {
            Ok(Some(v)) => {
                stats.ok += 1;
                stats.last = Some(v.to_string());
            }
            Ok(None) => stats.unavailable += 1,
            Err(e) => {
                stats.errors += 1;
                if stats.first_error.is_none() {
                    stats.first_error = Some(e.clone());
                }
            }
        }
    }

    pub fn initial(
        &mut self,
        at: Duration,
        power: &Option<String>,
        metered: &Option<String>,
        thermal: &Option<String>,
        user: &Option<String>,
    ) {
        self.events.push(format!(
            "{:>6.1} s  inicio: corriente = {}, red medida = {}, estado térmico = {}, usuario = {}",
            at.as_secs_f64(),
            show(power),
            show(metered),
            show(thermal),
            show(user)
        ));
    }

    /// Anota un cambio y devuelve la línea para mostrarla también en pantalla.
    pub fn change(
        &mut self,
        at: Duration,
        signal: Signal,
        old: &Option<String>,
        new: &Option<String>,
    ) -> String {
        let line = format!(
            "{:>6.1} s  {}: {} -> {}",
            at.as_secs_f64(),
            signal.label(),
            show(old),
            show(new)
        );
        self.events.push(line.clone());
        line
    }

    pub fn render(&self, system: &str) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "Informe de la sonda del gobernador (spike S4)");
        let _ = writeln!(out, "Sistema: {system}");
        let _ = writeln!(
            out,
            "Duración: {} s, una lectura cada {} ms",
            self.seconds,
            self.interval.as_millis()
        );
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "Señal | lecturas válidas | no disponible | errores | lectura mediana (ms) | lectura máxima (ms) | último valor"
        );
        for signal in SIGNALS {
            let Some(s) = self.stats.get(signal.index()) else {
                continue;
            };
            let (median, max) = median_max(&s.times);
            let _ = writeln!(
                out,
                "{} | {} | {} | {} | {:.2} | {:.2} | {}",
                signal.label(),
                s.ok,
                s.unavailable,
                s.errors,
                median,
                max,
                s.last.as_deref().unwrap_or("-")
            );
            if let Some(e) = &s.first_error {
                let _ = writeln!(out, "    primer error: {e}");
            }
        }
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "Cambios observados (la detección tarda como mucho un intervalo más la lectura):"
        );
        for e in &self.events {
            let _ = writeln!(out, "{e}");
        }
        out
    }
}

fn show(v: &Option<String>) -> &str {
    v.as_deref().unwrap_or("no disponible")
}

fn median_max(times: &[Duration]) -> (f64, f64) {
    let mut ms: Vec<f64> = times.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    ms.sort_by(f64::total_cmp);
    let median = ms.get(ms.len() / 2).copied().unwrap_or(0.0);
    let max = ms.last().copied().unwrap_or(0.0);
    (median, max)
}
