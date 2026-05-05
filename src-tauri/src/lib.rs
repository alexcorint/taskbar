// lib.rs — Punto de entrada de la librería Tauri.
//
// Este fichero solo contiene:
//  1. Declaración de módulos
//  2. Comandos Tauri (wrappers delgados sobre platform::*)
//  3. Función run()
//
// Toda la lógica de negocio reside en platform/ y types.rs.

mod platform;
mod types;
mod watchdog;

use types::*;
use tauri::Manager;

// ---------------------------------------------------------------------------
// MEDIA
// ---------------------------------------------------------------------------

#[tauri::command]
fn media_control(action: MediaControlAction) {
    match action {
        MediaControlAction::PlayPause => platform::input::media_play_pause(),
        MediaControlAction::Next => platform::input::media_next(),
        MediaControlAction::Prev => platform::input::media_prev(),
    }
}

#[tauri::command]
async fn get_current_media() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use base64::Engine;
        use windows::Media::Control::{
            GlobalSystemMediaTransportControlsSessionManager,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus,
        };
        use windows::Storage::Streams::DataReader;

        let mut info = MediaInfo {
            title: "Now listening...".to_string(),
            artist: String::new(),
            album: String::new(),
            app_id: "Generic".to_string(),
            thumbnail_base64: String::new(),
            is_playing: false,
            position_ms: 0,
            duration_ms: 0,
        };

        if let Ok(op) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            if let Ok(manager) = op.get() {
                if let Ok(session) = manager.GetCurrentSession() {
                    if let Ok(playback) = session.GetPlaybackInfo() {
                        if let Ok(status) = playback.PlaybackStatus() {
                            info.is_playing = status
                                == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
                        }
                    }

                    if let Ok(timeline) = session.GetTimelineProperties() {
                        if let (Ok(end), Ok(start)) =
                            (timeline.EndTime(), timeline.StartTime())
                        {
                            let dur = end.Duration - start.Duration;
                            if dur > 0 {
                                info.duration_ms = (dur / 10_000) as u64;
                            }
                        }

                        if let Ok(pos) = timeline.Position() {
                            let mut pos_100ns = pos.Duration.max(0);
                            if info.is_playing {
                                if let Ok(last_updated) = timeline.LastUpdatedTime() {
                                    let unix_now = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default();
                                    let now_100ns = (unix_now.as_nanos() / 100) as i64
                                        + 116_444_736_000_000_000i64;
                                    let elapsed = now_100ns - last_updated.UniversalTime;
                                    if elapsed > 0 {
                                        pos_100ns += elapsed;
                                    }
                                }
                            }
                            info.position_ms = (pos_100ns / 10_000) as u64;
                            if info.duration_ms > 0 && info.position_ms > info.duration_ms {
                                info.position_ms = info.duration_ms;
                            }
                        }
                    }

                    if let Ok(app_id) = session.SourceAppUserModelId() {
                        info.app_id = app_id.to_string();
                    }

                    if let Ok(op2) = session.TryGetMediaPropertiesAsync() {
                        if let Ok(props) = op2.get() {
                            info.title =
                                props.Title().map(|s| s.to_string()).unwrap_or_default();
                            info.artist =
                                props.Artist().map(|s| s.to_string()).unwrap_or_default();
                            info.album =
                                props.AlbumTitle().map(|s| s.to_string()).unwrap_or_default();

                            if let Ok(thumb) = props.Thumbnail() {
                                if let Ok(stream_op) = thumb.OpenReadAsync() {
                                    if let Ok(stream) = stream_op.get() {
                                        let size = stream.Size().unwrap_or(0);
                                        if let Ok(reader) = DataReader::CreateDataReader(&stream) {
                                            if let Ok(op3) = reader.LoadAsync(size as u32) {
                                                if op3.get().is_ok() {
                                                    let mut buf = vec![0u8; size as usize];
                                                    if reader.ReadBytes(&mut buf).is_ok() {
                                                        info.thumbnail_base64 =
                                                            base64::engine::general_purpose::STANDARD
                                                                .encode(&buf);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        return serde_json::to_string(&info).map_err(|e| e.to_string());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let info = MediaInfo {
            title: "Now listening...".to_string(),
            artist: String::new(),
            album: String::new(),
            app_id: "Generic".to_string(),
            thumbnail_base64: String::new(),
            is_playing: false,
            position_ms: 0,
            duration_ms: 0,
        };
        serde_json::to_string(&info).map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// VENTANAS
// ---------------------------------------------------------------------------

#[tauri::command]
fn move_window(window: tauri::Window, x: i32, y: i32, w: u32, h: u32) {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER,
        };
        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    x,
                    y,
                    w as i32,
                    h as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            return;
        }
    }
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(w, h)));
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)));
}

#[tauri::command]
fn set_window_to_bottom(window: tauri::Window) {
    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = window.hwnd() {
        use windows::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        };
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_BOTTOM),
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

#[tauri::command]
fn manage_window(app: tauri::AppHandle, label: String, action: WindowAction) {
    let Some(window) = app.get_webview_window(&label) else { return };

    match action {
        WindowAction::Show => { let _ = window.show(); }
        WindowAction::ShowAt { x, y } => {
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)));
            let _ = window.show();
        }
        WindowAction::Hide => { let _ = window.hide(); }
        WindowAction::Update { x, y, w, h } => {
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::UI::WindowsAndMessaging::{
                    SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW,
                };
                if let Ok(hwnd) = window.hwnd() {
                    unsafe {
                        let _ = SetWindowPos(hwnd, None, x, y, w, h,
                            SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW);
                    }
                    return;
                }
            }
            let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(w as u32, h as u32)));
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)));
            let _ = window.show();
        }
        WindowAction::UpdateLogical { x, y, w, h } => {
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(w, h)));
            let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
            let _ = window.show();
        }
    }
}

// ---------------------------------------------------------------------------
// APPS DE LA BARRA DE TAREAS
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_taskbar_apps() -> Vec<TaskbarApp> {
    platform::taskbar::get_taskbar_apps()
}

#[tauri::command]
fn interact_app(hwnd: isize, exec_path: String) {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
        };

        if hwnd != 0 {
            let h = HWND(hwnd as *mut _);
            if IsIconic(h).as_bool() {
                let _ = ShowWindow(h, SW_RESTORE);
            } else {
                let _ = ShowWindow(h, SW_SHOW);
            }
            let _ = SetForegroundWindow(h);
        } else if !exec_path.is_empty() {
            use windows::core::PCWSTR;
            use windows::Win32::UI::Shell::ShellExecuteW;
            use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
            let path_wide: Vec<u16> =
                exec_path.encode_utf16().chain(std::iter::once(0)).collect();
            let op_wide: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();
            let _ = ShellExecuteW(
                None,
                PCWSTR(op_wide.as_ptr()),
                PCWSTR(path_wide.as_ptr()),
                PCWSTR(std::ptr::null()),
                PCWSTR(std::ptr::null()),
                SW_SHOWNORMAL,
            );
        }
    }
}

#[tauri::command]
fn reorder_apps(ordered_ids: Vec<String>) {
    platform::taskbar::reorder_apps(ordered_ids);
}

// ---------------------------------------------------------------------------
// VOLUMEN
// ---------------------------------------------------------------------------

#[tauri::command]
fn volume_control(action: VolumeAction) {
    match action {
        VolumeAction::Mute => platform::input::volume_mute(),
        VolumeAction::Up => platform::input::volume_up(),
        VolumeAction::Down => platform::input::volume_down(),
    }
}

#[tauri::command]
fn get_volume_status() -> VolumeInfo {
    #[cfg(target_os = "windows")]
    return platform::audio::get_volume_status();
    #[cfg(not(target_os = "windows"))]
    platform::get_volume_status()
}

#[tauri::command]
fn set_volume(value: f32) {
    #[cfg(target_os = "windows")]
    platform::audio::set_volume(value);
    #[cfg(not(target_os = "windows"))]
    platform::set_volume(value);
}

#[tauri::command]
fn toggle_mute() {
    #[cfg(target_os = "windows")]
    platform::audio::toggle_mute();
    #[cfg(not(target_os = "windows"))]
    platform::toggle_mute();
}

// ---------------------------------------------------------------------------
// BATERÍA Y ENERGÍA
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_battery_status() -> BatteryInfo {
    #[cfg(target_os = "windows")]
    return platform::power::get_battery_status();
    #[cfg(not(target_os = "windows"))]
    platform::get_battery_status()
}

#[tauri::command]
fn get_brightness() -> u8 {
    #[cfg(target_os = "windows")]
    return platform::power::get_brightness();
    #[cfg(not(target_os = "windows"))]
    platform::get_brightness()
}

#[tauri::command]
fn set_brightness(value: u8) {
    #[cfg(target_os = "windows")]
    platform::power::set_brightness(value);
    #[cfg(not(target_os = "windows"))]
    platform::set_brightness(value);
}

#[tauri::command]
fn get_power_profiles() -> Vec<PowerProfile> {
    #[cfg(target_os = "windows")]
    return platform::power::get_power_profiles();
    #[cfg(not(target_os = "windows"))]
    platform::get_power_profiles()
}

#[tauri::command]
fn set_power_profile(guid_str: String) {
    #[cfg(target_os = "windows")]
    platform::power::set_power_profile(&guid_str);
    #[cfg(not(target_os = "windows"))]
    platform::set_power_profile(&guid_str);
}

// ---------------------------------------------------------------------------
// RED Y RADIOS
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_network_status() -> NetworkStatus {
    #[cfg(target_os = "windows")]
    return platform::network::get_network_status();
    #[cfg(not(target_os = "windows"))]
    platform::get_network_status()
}

#[tauri::command]
async fn get_radio_states() -> Result<RadioStates, String> {
    #[cfg(target_os = "windows")]
    return platform::radios::get_radio_states().await;
    #[cfg(not(target_os = "windows"))]
    platform::get_radio_states().await
}

#[tauri::command]
async fn toggle_radio(kind: String, enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return platform::radios::toggle_radio(&kind, enable).await;
    #[cfg(not(target_os = "windows"))]
    platform::toggle_radio(&kind, enable).await
}

#[tauri::command]
async fn get_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    #[cfg(target_os = "windows")]
    return platform::radios::get_wifi_networks().await;
    #[cfg(not(target_os = "windows"))]
    platform::get_wifi_networks().await
}

#[tauri::command]
async fn get_bluetooth_devices() -> Result<Vec<BluetoothDeviceInfo>, String> {
    #[cfg(target_os = "windows")]
    return platform::radios::get_bluetooth_devices().await;
    #[cfg(not(target_os = "windows"))]
    platform::get_bluetooth_devices().await
}

// ---------------------------------------------------------------------------
// MONITOR Y SISTEMA
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_monitor_rects() -> Vec<MonitorRect> {
    #[cfg(target_os = "windows")]
    return platform::display::get_monitor_rects();
    #[cfg(not(target_os = "windows"))]
    platform::get_monitor_rects()
}

#[tauri::command]
fn open_start_menu() {
    platform::input::open_start_menu();
}

#[tauri::command]
fn init_taskbar_environment() {
    use std::process::Command;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let my_pid = std::process::id();
        let my_exe = std::env::current_exe().unwrap_or_default();
        let _ = Command::new(my_exe)
            .arg("--watchdog")
            .arg(my_pid.to_string())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();

        platform::taskbar::hide_windows_taskbar();
    }
}

// ---------------------------------------------------------------------------
// RUN
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    {
        let args: Vec<String> = std::env::args().collect();
        if args.len() >= 3 && args[1] == "--watchdog" {
            if let Ok(pid) = args[2].parse::<u32>() {
                watchdog::run_watchdog(pid);
                return;
            }
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            media_control,
            get_current_media,
            move_window,
            volume_control,
            get_battery_status,
            get_volume_status,
            get_taskbar_apps,
            interact_app,
            reorder_apps,
            open_start_menu,
            get_monitor_rects,
            manage_window,
            init_taskbar_environment,
            set_window_to_bottom,
            get_brightness,
            set_brightness,
            get_power_profiles,
            set_power_profile,
            set_volume,
            toggle_mute,
            get_network_status,
            toggle_radio,
            get_wifi_networks,
            get_bluetooth_devices,
            get_radio_states,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                #[cfg(target_os = "windows")]
                platform::taskbar::restore_windows_taskbar();
            }
        });
}

// ---------------------------------------------------------------------------
// TESTS
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_taskbar_apps() {
        let apps = get_taskbar_apps();
        for app in &apps {
            println!(
                "APP: {} | has_icon={} | pinned={}",
                app.title,
                !app.icon_base64.is_empty(),
                app.is_pinned,
            );
        }
        // Al menos no paniquea
        assert!(apps.len() < 10000);
    }
}
