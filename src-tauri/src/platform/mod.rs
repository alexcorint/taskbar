// Módulo de plataforma — selección condicional en compile-time.
//
// Todos los comandos Tauri llaman a `platform::*` sin saber en qué SO están.
// En tiempo de compilación se elige la implementación correcta.

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
