# Spike S4 — Gobernador de recursos

**Estado:** en curso. El prototipo está listo y probado en Linux sin entorno gráfico; faltan las mediciones en equipos reales con Windows, macOS y Linux de escritorio (las hace el responsable, guía de G0, prueba 3).

**Pregunta (`12` F0-S4):** ¿se pueden leer de verdad, en cada sistema, la batería, la red medida, la temperatura y la actividad del usuario, con qué latencia y con qué crates?

## 1. Prototipo

`tools/governor-probe` (`pnpm spike:gobernador [segundos] [archivo]`). Lee cada señal una vez por segundo durante 10 minutos por defecto, mide cuánto tarda cada lectura, anota cada cambio y escribe un informe de texto. El informe no lleva nombre del equipo, usuario, direcciones de red ni números de serie (I-4).

| Señal | Windows | macOS | Linux |
| :-- | :-- | :-- | :-- |
| Batería y corriente | `starship-battery` | `starship-battery` | `starship-battery` |
| Red medida | `windows`: `NetworkInformation`, costo de la conexión | No se prueba (ver §3) | `zbus`: propiedad `Metered` de NetworkManager |
| Temperatura | `sysinfo` (sensores, si el sistema los expone) | `sysinfo` y `ProcessInfo.thermalState` (`objc2-foundation`) | `sysinfo` (`/sys/class/thermal` y hwmon) |
| CPU y memoria | `sysinfo` | `sysinfo` | `sysinfo` |
| Disco libre | `sysinfo` | `sysinfo` | `sysinfo` |
| Actividad del usuario | `user-idle` (última entrada, `GetLastInputInfo`) | `objc2-core-graphics`: `CGEventSourceSecondsSinceLastEventType` | `zbus`: `IdleHint` de logind |

Las dependencias de cada sistema van por `cfg` de destino: Linux no compila nada de Windows ni de macOS y no necesita bibliotecas de X11 ni de D-Bus en C. El prototipo compila sin avisos de `clippy` para Linux, Windows (`x86_64-pc-windows-msvc`) y macOS (`aarch64-apple-darwin`).

**Diferencia con el enunciado:** el enunciado pide leer «desde un proceso Tauri». El prototipo es un ejecutable de Rust normal. Las API que usa son las mismas en un proceso Tauri sin sandbox, que es como se distribuirá el nodo de escritorio. La única diferencia real aparece si la app de macOS se distribuye con App Sandbox (Mac App Store): entonces cambian el acceso a IOKit y a los sensores. Se comprobará al empaquetar la app (F1-T23).

## 2. Crates

| Crate | Versión | Licencia | Última publicación | Descargas recientes | Valoración |
| :-- | :-- | :-- | :-- | :-- | :-- |
| `starship-battery` | 0.12.0 | ISC | 2026-09-27 | 0,5 M | Mantenido (fork del crate `battery`, abandonado). |
| `sysinfo` | 0.40.4 | MIT | 2026-11-16 | 66 M | Muy mantenido. Se usa sin las funciones de GPU, red y usuarios. |
| `windows` | 0.62.2 | MIT o Apache-2.0 | 2025-10-06 | 122 M | Oficial de Microsoft. Las API WinRT se llaman sin `unsafe`. |
| `objc2-foundation` | 0.3.2 | MIT | 2025-10-04 | 63 M | Mantenido. |
| `zbus` | 6.5.0 | MIT | 2026-11-20 | 46 M | Muy mantenido. La versión 6 no tiene API bloqueante; se usa `zbus::block_on`. |
| `objc2-core-graphics` | 0.3.2 | Zlib, Apache-2.0 o MIT | 2025-10-04 | — | Mantenido (mismo proyecto que `objc2-foundation`). Expone la llamada de inactividad de macOS sin `unsafe`. |
| `user-idle` | 0.6.0 | MIT o Apache-2.0 | 2023-10-18 | 0,08 M | **Sin publicaciones desde 2023.** Solo se usa en Windows, donde envuelve `GetLastInputInfo`. En macOS arrastra `IOKit-sys`, `CoreFoundation-sys` y `mach` 0.1.2 y 0.3.2, las dos con el aviso [RUSTSEC-2020-0168](https://rustsec.org/advisories/RUSTSEC-2020-0168) (sin mantenimiento): por eso en macOS se sustituyó por `objc2-core-graphics`. **No es candidato para el gobernador.** |

`cargo deny` pasa con la política actual (licencias, avisos, orígenes y prohibiciones). Con `unmaintained = "all"`, que también hace fallar los crates sin mantenimiento transitivos, falla por `mach`: aunque `user-idle` solo se declara para Windows, `cargo deny` evalúa todos los destinos y cuenta sus dependencias de macOS. Para el gobernador real, la inactividad en Windows necesita otra vía: `GetLastInputInfo` del crate `windows` es una función `unsafe`, y el espacio de trabajo prohíbe `unsafe`. Hay que elegir entre un crate envoltorio mantenido o una excepción revisada con ADR; se decide al implementar `crates/social-governor`.

## 3. Resultados

### 3.1 Linux sin entorno gráfico (contenedor de CI)

Ubuntu 24.04, x86_64, 15 s. Sin batería, sin NetworkManager, sin logind y sin sensores de temperatura: las cuatro señales salen «no disponible», sin errores, y el gobernador aplicaría la regla de respaldo (`08` §2.1). CPU, memoria y disco libre se leen bien: lectura mediana de 0,16 ms y 0,57 ms, máxima de 1,84 ms y 0,96 ms.

Este caso es el de un servidor: confirma que la sonda no falla cuando faltan las señales, pero no dice nada de un equipo de escritorio.

### 3.2 Windows, macOS y Linux de escritorio

Pendiente: las hace el responsable con la guía de G0 (prueba 3) y se anotan aquí.

| Equipo | Batería y corriente | Red medida | Temperatura | Actividad | Latencia de detección |
| :-- | :-- | :-- | :-- | :-- | :-- |
| Portátil con Windows | | | | | |
| Mac | | | | | |
| Linux de escritorio | | | | | |

## 4. Qué se espera y qué queda abierto

- **Latencia.** La sonda detecta un cambio como mucho un intervalo (1 s) más el tiempo de lectura después de que el sistema lo publica. Con lecturas por debajo de 2 ms, el límite de 5 s de `08` §2.2 se cumple con holgura si el sistema publica el cambio a tiempo; eso es lo que miden las pruebas reales.
- **Temperatura en Windows.** `08` §2.1 ya dice que no hay API fiable. La prueba dirá si `sysinfo` la lee (por WMI) en el equipo del responsable. Si no, se usa la regla de respaldo: carga de CPU sostenida.
- **Red medida en macOS.** `NWPathMonitor` necesita los enlaces del framework Network, que este prototipo no incluye. Hasta medirlo, macOS aplica la regla de respaldo: red medida salvo Ethernet.
- **Actividad en Linux.** `IdleHint` de logind solo se activa cuando el escritorio declara el reposo, con su propio retraso. Si resulta poco fiable, el gobernador usa la carga de CPU como en el respaldo.

Cuando lleguen las mediciones reales se completa §3.2, se actualiza la tabla de `08` §2.1 y se cierra el spike.
