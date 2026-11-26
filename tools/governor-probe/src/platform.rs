//! Lectura de cada señal en cada sistema operativo (`docs/08` §2.1).
//!
//! Cada lectura devuelve `Ok(None)` cuando el sistema no ofrece la señal y `Err` cuando
//! la ofrece pero falló. Los errores se resumen sin rutas, nombres ni direcciones.

use std::fmt;

use starship_battery::State;
use sysinfo::{Components, Disks, System};

pub struct PowerReading {
    pub state: String,
    percent: f32,
}

impl fmt::Display for PowerReading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:.0} %)", self.state, self.percent)
    }
}

pub struct Probe {
    system: Result<System, String>,
    disks: Result<Disks, String>,
    components: Result<Components, String>,
    battery: Result<starship_battery::Manager, String>,
    #[cfg(target_os = "linux")]
    bus: Option<zbus::Connection>,
}

impl Probe {
    pub fn new() -> Self {
        Self {
            system: System::new().map_err(|e| format!("{e:?}")),
            disks: Disks::new_with_refreshed_list().map_err(|e| format!("{e:?}")),
            components: Components::new_with_refreshed_list().map_err(|e| format!("{e:?}")),
            battery: starship_battery::Manager::new().map_err(|e| e.to_string()),
            #[cfg(target_os = "linux")]
            bus: zbus::block_on(zbus::Connection::system()).ok(),
        }
    }

    /// Estado de la batería y porcentaje. `None` en un equipo sin batería.
    pub fn power(&mut self) -> Result<Option<PowerReading>, String> {
        let manager = self.battery.as_ref().map_err(Clone::clone)?;
        let mut batteries = manager.batteries().map_err(|e| e.to_string())?;
        let Some(battery) = batteries.next() else {
            return Ok(None);
        };
        let battery = battery.map_err(|e| e.to_string())?;
        let state = match battery.state() {
            State::Charging => "cargando",
            State::Discharging => "descargando (sin corriente)",
            State::Full => "llena (con corriente)",
            State::Empty => "vacía",
            State::Paused => "en pausa por límite de carga (con corriente)",
            State::Unknown => "desconocido",
        };
        Ok(Some(PowerReading {
            state: state.to_owned(),
            percent: battery.state_of_charge().value * 100.0,
        }))
    }

    /// Si la conexión a internet es medida (cobra por datos o tiene límite).
    pub fn metered(&mut self) -> Result<Option<String>, String> {
        metered_impl(self)
    }

    /// Temperatura más alta entre los sensores que el sistema expone.
    pub fn temperature(&mut self) -> Result<Option<Temperature>, String> {
        let components = self.components.as_mut().map_err(|e| e.clone())?;
        components.refresh(true);
        let max = components
            .iter()
            .filter_map(|c| c.temperature())
            .filter(|t| t.is_finite() && *t > 0.0)
            .reduce(f32::max);
        Ok(max.map(Temperature))
    }

    /// Estado térmico que da el propio sistema (solo macOS).
    pub fn thermal_state(&mut self) -> Result<Option<String>, String> {
        thermal_impl()
    }

    pub fn cpu_and_memory(&mut self) -> Result<Option<String>, String> {
        let system = self.system.as_mut().map_err(|e| e.clone())?;
        system.refresh_cpu_usage();
        system.refresh_memory();
        let total = system.total_memory();
        if total == 0 {
            return Ok(None);
        }
        let used = system.used_memory() as f64 / total as f64 * 100.0;
        Ok(Some(format!(
            "CPU {:.0} %, memoria {:.0} %",
            system.global_cpu_usage(),
            used
        )))
    }

    /// Espacio libre del disco donde se ejecuta la sonda, en GB.
    pub fn disk_free(&mut self) -> Result<Option<String>, String> {
        let disks = self.disks.as_mut().map_err(|e| e.clone())?;
        disks.refresh(true);
        let here = std::env::current_dir().map_err(|e| e.kind().to_string())?;
        let disk = disks
            .iter()
            .filter(|d| here.starts_with(d.mount_point()))
            .max_by_key(|d| d.mount_point().as_os_str().len());
        Ok(disk.map(|d| format!("{:.1} GB", d.available_space() as f64 / 1e9)))
    }

    /// Segundos desde la última entrada del usuario (teclado o ratón).
    pub fn idle_seconds(&mut self) -> Result<Option<u64>, String> {
        idle_impl(self)
    }
}

pub struct Temperature(f32);

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} °C", self.0)
    }
}

/// Sistema y arquitectura, sin nada que identifique el equipo.
pub fn system_description() -> String {
    format!(
        "{} ({}), sonda {}",
        System::long_os_version().unwrap_or_else(|_| std::env::consts::OS.to_owned()),
        System::cpu_arch(),
        env!("CARGO_PKG_VERSION")
    )
}

// --- Red medida ---------------------------------------------------------------------------

#[cfg(windows)]
fn metered_impl(_: &mut Probe) -> Result<Option<String>, String> {
    use windows::Networking::Connectivity::{NetworkCostType, NetworkInformation};
    // Sin conexión a internet no hay perfil: la señal no está disponible en ese momento.
    let Ok(profile) = NetworkInformation::GetInternetConnectionProfile() else {
        return Ok(None);
    };
    let cost = profile
        .GetConnectionCost()
        .map_err(|e| e.message().to_string())?;
    let kind = cost
        .NetworkCostType()
        .map_err(|e| e.message().to_string())?;
    let roaming = cost.Roaming().map_err(|e| e.message().to_string())?;
    let over = cost.OverDataLimit().map_err(|e| e.message().to_string())?;
    let label = if kind == NetworkCostType::Unrestricted && !roaming && !over {
        "no medida"
    } else if kind == NetworkCostType::Unknown {
        "desconocida"
    } else {
        "medida"
    };
    Ok(Some(label.to_owned()))
}

#[cfg(target_os = "linux")]
fn metered_impl(probe: &mut Probe) -> Result<Option<String>, String> {
    let Some(bus) = &probe.bus else {
        return Ok(None);
    };
    let value: u32 = zbus::block_on(async {
        let proxy = zbus::Proxy::new(
            bus,
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
        )
        .await?;
        proxy.get_property::<u32>("Metered").await
    })
    .map_err(|e| e.to_string())?;
    // NMMetered: 0 desconocido, 1 sí, 2 no, 3 probablemente sí, 4 probablemente no.
    let label = match value {
        1 | 3 => "medida",
        2 | 4 => "no medida",
        _ => "desconocida",
    };
    Ok(Some(label.to_owned()))
}

#[cfg(not(any(windows, target_os = "linux")))]
fn metered_impl(_: &mut Probe) -> Result<Option<String>, String> {
    // macOS: NWPathMonitor (isExpensive, isConstrained) no se prueba en este prototipo.
    Ok(None)
}

// --- Estado térmico -----------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn thermal_impl() -> Result<Option<String>, String> {
    use objc2_foundation::{NSProcessInfo, NSProcessInfoThermalState};
    let state = NSProcessInfo::processInfo().thermalState();
    let label = if state == NSProcessInfoThermalState::Nominal {
        "normal"
    } else if state == NSProcessInfoThermalState::Fair {
        "elevado"
    } else if state == NSProcessInfoThermalState::Serious {
        "serio"
    } else if state == NSProcessInfoThermalState::Critical {
        "crítico"
    } else {
        "desconocido"
    };
    Ok(Some(label.to_owned()))
}

#[cfg(not(target_os = "macos"))]
fn thermal_impl() -> Result<Option<String>, String> {
    Ok(None)
}

// --- Actividad del usuario ----------------------------------------------------------------

#[cfg(any(windows, target_os = "macos"))]
fn idle_impl(_: &mut Probe) -> Result<Option<u64>, String> {
    user_idle::UserIdle::get_time()
        .map(|t| Some(t.as_seconds()))
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
fn idle_impl(probe: &mut Probe) -> Result<Option<u64>, String> {
    // logind publica si la sesión está en reposo y desde cuándo, sin depender del entorno
    // gráfico; muchos escritorios solo lo marcan tras su propio tiempo de inactividad.
    let Some(bus) = &probe.bus else {
        return Ok(None);
    };
    let (idle, since_us) = zbus::block_on(async {
        let proxy = zbus::Proxy::new(
            bus,
            "org.freedesktop.login1",
            "/org/freedesktop/login1/session/auto",
            "org.freedesktop.login1.Session",
        )
        .await?;
        let idle = proxy.get_property::<bool>("IdleHint").await?;
        let since = proxy.get_property::<u64>("IdleSinceHint").await?;
        Ok::<_, zbus::Error>((idle, since))
    })
    .map_err(|e| e.to_string())?;
    if !idle {
        return Ok(Some(0));
    }
    let now_us = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);
    let secs = now_us.saturating_sub(u128::from(since_us)) / 1_000_000;
    Ok(Some(u64::try_from(secs).unwrap_or(u64::MAX)))
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn idle_impl(_: &mut Probe) -> Result<Option<u64>, String> {
    Ok(None)
}
