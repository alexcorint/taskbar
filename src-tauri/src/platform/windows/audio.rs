// Audio: volumen del sistema mediante IAudioEndpointVolume cacheado.
//
// El endpoint COM se inicializa UNA sola vez con OnceLock y se comparte
// entre todas las llamadas vía Mutex, eliminando las 9 instanciaciones COM
// repetidas que existían en get_volume_status / set_volume / toggle_mute.

use crate::types::VolumeInfo;
use std::sync::{Mutex, OnceLock};
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};

// --- Wrapper que permite compartir IAudioEndpointVolume entre hilos MTA ---

struct AudioEndpoint(IAudioEndpointVolume);

// SAFETY: IAudioEndpointVolume vive en el apartamento MTA (COINIT_MULTITHREADED).
// Todos los accesos están protegidos por Mutex, garantizando exclusión mutua.
unsafe impl Send for AudioEndpoint {}
unsafe impl Sync for AudioEndpoint {}

fn init_endpoint() -> Option<Mutex<AudioEndpoint>> {
    unsafe {
        // CoInitializeEx puede retornar S_FALSE si ya está inicializado — ignoramos el resultado
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;

        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .ok()?;

        let endpoint: IAudioEndpointVolume =
            device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).ok()?;

        Some(Mutex::new(AudioEndpoint(endpoint)))
    }
}

fn get_endpoint() -> Option<&'static Mutex<AudioEndpoint>> {
    static ENDPOINT: OnceLock<Option<Mutex<AudioEndpoint>>> = OnceLock::new();
    ENDPOINT.get_or_init(init_endpoint).as_ref()
}

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

pub fn get_volume_status() -> VolumeInfo {
    if let Some(ep) = get_endpoint() {
        if let Ok(guard) = ep.lock() {
            unsafe {
                let volume = guard.0.GetMasterVolumeLevelScalar().unwrap_or(0.0);
                let is_muted = guard
                    .0
                    .GetMute()
                    .unwrap_or(windows::core::BOOL(0))
                    .as_bool();
                return VolumeInfo { volume, is_muted };
            }
        }
    }
    VolumeInfo {
        volume: 0.5,
        is_muted: false,
    }
}

pub fn set_volume(value: f32) {
    if let Some(ep) = get_endpoint() {
        if let Ok(guard) = ep.lock() {
            unsafe {
                let _ = guard.0.SetMasterVolumeLevelScalar(value, std::ptr::null());
            }
        }
    }
}

pub fn toggle_mute() {
    if let Some(ep) = get_endpoint() {
        if let Ok(guard) = ep.lock() {
            unsafe {
                let current = guard.0.GetMute().unwrap_or(windows::core::BOOL(0));
                let _ = guard.0.SetMute(!current.as_bool(), std::ptr::null());
            }
        }
    }
}
