// 1. Importaciones necesarias
use base64::Engine;
use enigo::{Enigo, Key, KeyboardControllable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::size_of;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use tauri::Manager;

#[cfg(target_os = "windows")]
use windows::core::{Interface, BOOL, PCWSTR, PWSTR};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, MAX_PATH, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD,
};
#[cfg(target_os = "windows")]
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
    STGM,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{
    IShellLinkW, SHGetFileInfoW, ShellLink, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
};
#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
use windows::Win32::UI::WindowsAndMessaging::GetClassLongPtrW;
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
use windows::Win32::UI::WindowsAndMessaging::GetClassLongW;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon,
    EnumWindows,
    FindWindowW,
    GetClassNameW, // 🌟 AÑADE ESTO AQUÍ
    GetIconInfo,
    GetWindow,
    GetWindowLongW,
    GetWindowTextW,
    GetWindowThreadProcessId,
    IsIconic,
    IsWindowVisible,
    SendMessageTimeoutW,
    SetForegroundWindow,
    ShowWindow,
    GCLP_HICON,
    GWL_EXSTYLE,
    GW_OWNER,
    HICON,
    ICONINFO,
    ICON_BIG,
    ICON_SMALL2,
    SMTO_ABORTIFHUNG,
    SW_HIDE,
    SW_RESTORE,
    SW_SHOW,
    WM_GETICON,
    WS_EX_TOOLWINDOW,
};

// 2. Definimos la estructura de datos
#[derive(Serialize)]
struct MediaInfo {
    title: String,
    artist: String,
    album: String,
    app_id: String,
    thumbnail_base64: String,
    is_playing: bool,
    position_ms: u64,
    duration_ms: u64,
}

#[derive(Serialize)]
struct BatteryInfo {
    percentage: u8,
    is_charging: bool,
    battery_saver: bool,
}

#[derive(Serialize)]
struct PowerProfile {
    guid: String,
    name: String,
    active: bool,
}

#[derive(Serialize)]
struct VolumeInfo {
    volume: f32,
    is_muted: bool,
}

#[derive(Serialize, Clone, Debug)]
struct TaskbarApp {
    id: String,
    title: String,
    icon_base64: String,
    is_active: bool,
    is_pinned: bool,
    hwnd: isize,
    exec_path: String,
}

fn get_icon_cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// Mantiene el orden estable de los iconos en la barra entre actualizaciones
fn get_app_order() -> &'static Mutex<Vec<String>> {
    static ORDER: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    ORDER.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(target_os = "windows")]
unsafe fn hicon_to_base64(hicon: HICON) -> Option<String> {
    let mut icon_info = ICONINFO::default();
    if GetIconInfo(hicon, &mut icon_info).is_err() {
        return None;
    }

    let hdc = CreateCompatibleDC(None);
    if hdc.is_invalid() {
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());
        return None;
    }

    let mut bmp = BITMAP::default();
    let bytes_read = GetObjectW(
        icon_info.hbmColor.into(),
        size_of::<BITMAP>() as i32,
        Some(&mut bmp as *mut _ as *mut _),
    );

    if bytes_read == 0 {
        let _ = DeleteDC(hdc);
        let _ = DeleteObject(icon_info.hbmColor.into()); // <-- Añadir .into()
        let _ = DeleteObject(icon_info.hbmMask.into()); // <-- Añadir .into()
        return None;
    }

    let width = bmp.bmWidth;
    let height = bmp.bmHeight;

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD::default(); 1],
    };

    let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

    let lines = GetDIBits(
        hdc,
        icon_info.hbmColor,
        0,
        height as u32,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );

    let _ = DeleteDC(hdc);
    let _ = DeleteObject(icon_info.hbmColor.into());
    let _ = DeleteObject(icon_info.hbmMask.into());

    if lines == 0 {
        return None;
    }

    let mut has_alpha = false;
    for chunk in buffer.chunks_exact(4) {
        if chunk[3] != 0 {
            has_alpha = true;
            break;
        }
    }

    for chunk in buffer.chunks_exact_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        chunk[0] = r;
        chunk[2] = b;
        if !has_alpha {
            chunk[3] = 255;
        }
    }

    let img = image::RgbaImage::from_raw(width as u32, height as u32, buffer)?;

    let mut cursor = std::io::Cursor::new(Vec::new());
    if img.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
        return Some(base64::engine::general_purpose::STANDARD.encode(cursor.into_inner()));
    }

    None
}

fn extract_icon_base64(path: &str) -> String {
    let mut cache = get_icon_cache().lock().unwrap();
    if let Some(b64) = cache.get(path) {
        return b64.clone();
    }

    #[cfg(target_os = "windows")]
    unsafe {
        let path_wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let mut shfi = SHFILEINFOW::default();

        let res = SHGetFileInfoW(
            PCWSTR(path_wide.as_ptr()),
            FILE_ATTRIBUTE_NORMAL,
            Some(&mut shfi),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if res != 0 && !shfi.hIcon.is_invalid() {
            let hicon = shfi.hIcon;
            if let Some(b64) = hicon_to_base64(hicon) {
                let _ = DestroyIcon(hicon);
                cache.insert(path.to_string(), b64.clone());
                return b64;
            }
            let _ = DestroyIcon(hicon);
        }
    }

    // Empty icon fallback
    let fallback = String::new();
    cache.insert(path.to_string(), fallback.clone());
    fallback
}

#[cfg(target_os = "windows")]
unsafe fn extract_icon_from_hwnd(hwnd: HWND) -> Option<String> {
    let mut hicon_val: usize = 0;

    // 1. WM_GETICON ICON_BIG
    if SendMessageTimeoutW(
        hwnd,
        WM_GETICON,
        WPARAM(ICON_BIG as usize),
        LPARAM(0),
        SMTO_ABORTIFHUNG,
        100,
        Some(&mut hicon_val),
    )
    .0 != 0
        && hicon_val != 0
    {
        if let Some(b64) = hicon_to_base64(HICON(hicon_val as *mut _)) {
            return Some(b64);
        }
    }

    // 2. WM_GETICON ICON_SMALL2
    if SendMessageTimeoutW(
        hwnd,
        WM_GETICON,
        WPARAM(ICON_SMALL2 as usize),
        LPARAM(0),
        SMTO_ABORTIFHUNG,
        100,
        Some(&mut hicon_val),
    )
    .0 != 0
        && hicon_val != 0
    {
        if let Some(b64) = hicon_to_base64(HICON(hicon_val as *mut _)) {
            return Some(b64);
        }
    }

    // 3. GCLP_HICON
    #[cfg(target_pointer_width = "64")]
    let class_icon = GetClassLongPtrW(hwnd, GCLP_HICON) as usize;
    #[cfg(target_pointer_width = "32")]
    let class_icon = GetClassLongW(hwnd, GCLP_HICON) as usize;

    if class_icon != 0 {
        if let Some(b64) = hicon_to_base64(HICON(class_icon as *mut _)) {
            return Some(b64);
        }
    }

    None
}

#[cfg(target_os = "windows")]
struct EnumState {
    apps: Vec<TaskbarApp>,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // 1. Si no es visible, ignorar
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let mut title_buf = [0u16; 512];
    let tlen = GetWindowTextW(hwnd, &mut title_buf);
    let title = String::from_utf16_lossy(&title_buf[..tlen as usize]).to_string();

    // 2. Sin título o Program Manager → ignorar
    if title.is_empty() || title == "Program Manager" {
        return BOOL(1);
    }

    // 3. Comprobar si está cloaked por DWM (escritorios virtuales, etc.)
    let mut cloaked_val: i32 = 0;
    let _ = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked_val as *mut _ as *mut std::ffi::c_void,
        std::mem::size_of::<i32>() as u32,
    );
    if cloaked_val != 0 {
        return BOOL(1);
    }

    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

    // 4. WS_EX_TOOLWINDOW → nunca aparece en la barra de tareas
    if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
        return BOOL(1);
    }

    // 5. Algoritmo mejorado (Estilo IsAltTabWindow):
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    let owner_hwnd = GetWindow(hwnd, GW_OWNER).unwrap_or_default();

    // Si es un toolwindow (ventana de herramientas), la ignoramos
    if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
        return BOOL(1);
    }

    // Si es un AppWindow (ventanas forzadas a la barra de tareas), SIEMPRE la mostramos
    let is_app_window =
        (ex_style & windows::Win32::UI::WindowsAndMessaging::WS_EX_APPWINDOW.0) != 0;

    // Si no es AppWindow explícitamente, comprobamos si tiene un dueño
    if !is_app_window && !owner_hwnd.0.is_null() {
        // Obtenemos el estilo del dueño
        let owner_ex_style = GetWindowLongW(owner_hwnd, GWL_EXSTYLE) as u32;
        // Si el dueño NO es un ToolWindow, asumimos que esta ventana es secundaria y la ocultamos
        if (owner_ex_style & WS_EX_TOOLWINDOW.0) == 0 {
            return BOOL(1);
        }
    }

    // 6. Esta ventana es una candidata. Obtener PID y ruta del ejecutable.
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    let mut exec_path = String::new();

    if pid > 0 {
        let process_handle =
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).unwrap_or_default();
        if !process_handle.is_invalid() {
            let mut path_buf = [0u16; MAX_PATH as usize];
            let mut size = MAX_PATH;

            if QueryFullProcessImageNameW(
                process_handle,
                PROCESS_NAME_FORMAT(0),
                PWSTR(path_buf.as_mut_ptr()),
                &mut size,
            )
            .is_ok()
            {
                exec_path = String::from_utf16_lossy(&path_buf[..size as usize]).to_string();
            }
            let _ = windows::Win32::Foundation::CloseHandle(process_handle);
        }
    }

    // 🌟 Extraemos el Nombre de la Clase de la ventana como plan B
    let mut class_name_buf = [0u16; 256];
    let clen = GetClassNameW(hwnd, &mut class_name_buf);
    let class_name = String::from_utf16_lossy(&class_name_buf[..clen as usize]).to_string();

    // 🌟 ID ESTABLE MEJORADO:
    // 1. Priorizamos la ruta del ejecutable (lo más exacto para agrupar).
    // 2. Si falla (ej: apps de Windows Store/UWP), usamos el nombre de la clase que es inmutable.
    // 3. Fallback finalísimo al hwnd (casi imposible llegar aquí).
    let app_id = if !exec_path.is_empty() {
        exec_path.clone()
    } else if !class_name.is_empty() {
        format!("class-{}", class_name)
    } else {
        format!("hwnd-{}", hwnd.0 as isize)
    };

    let state = &mut *(lparam.0 as *mut EnumState);

    // Si ya existe una entrada para este exe, actualizamos hwnd y título en sitio
    // (varias ventanas del mismo exe se fusionan en una sola entrada)
    if let Some(existing) = state.apps.iter_mut().find(|a| a.id == app_id) {
        existing.hwnd = hwnd.0 as isize;
        existing.title = title;
    } else {
        let mut icon_base64 = extract_icon_from_hwnd(hwnd).unwrap_or_default();
        if icon_base64.is_empty() && !exec_path.is_empty() {
            icon_base64 = extract_icon_base64(&exec_path);
        }

        state.apps.push(TaskbarApp {
            id: app_id,
            title,
            icon_base64,
            is_active: true,
            is_pinned: false,
            hwnd: hwnd.0 as isize,
            exec_path,
        });
    }

    BOOL(1)
}

#[tauri::command]
fn get_taskbar_apps() -> Vec<TaskbarApp> {
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }

    #[cfg(target_os = "windows")]
    {
        let mut state = EnumState { apps: Vec::new() };
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut state as *mut _ as isize),
            );

            if let Ok(appdata) = std::env::var("APPDATA") {
                let pinned_dir = format!(
                    r"{}\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar",
                    appdata
                );
                if let Ok(entries) = std::fs::read_dir(&pinned_dir) {
                    if let Ok(shell_link) =
                        CoCreateInstance::<_, IShellLinkW>(&ShellLink, None, CLSCTX_INPROC_SERVER)
                    {
                        if let Ok(persist_file) = shell_link.cast::<IPersistFile>() {
                            for entry in entries.filter_map(Result::ok) {
                                let path = entry.path();
                                if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                                    let path_str = path.to_str().unwrap_or_default();
                                    let path_wide: Vec<u16> =
                                        path_str.encode_utf16().chain(std::iter::once(0)).collect();

                                    if persist_file
                                        .Load(PCWSTR(path_wide.as_ptr()), STGM(0))
                                        .is_ok()
                                    {
                                        let mut target_path = [0u16; 260];
                                        if shell_link
                                            .GetPath(&mut target_path, std::ptr::null_mut(), 0)
                                            .is_ok()
                                        {
                                            let len = target_path
                                                .iter()
                                                .position(|&c| c == 0)
                                                .unwrap_or(260);
                                            let exec_path =
                                                String::from_utf16_lossy(&target_path[..len])
                                                    .to_string();

                                            if let Some(app) = state.apps.iter_mut().find(|a| {
                                                a.exec_path.to_lowercase()
                                                    == exec_path.to_lowercase()
                                            }) {
                                                app.is_pinned = true;
                                            } else {
                                                let icon_base64 = extract_icon_base64(path_str);
                                                let title = path
                                                    .file_stem()
                                                    .and_then(|s| s.to_str())
                                                    .unwrap_or("App")
                                                    .to_string();

                                                state.apps.push(TaskbarApp {
                                                    id: path_str.to_string(),
                                                    title,
                                                    icon_base64,
                                                    is_active: false,
                                                    is_pinned: true,
                                                    hwnd: 0,
                                                    exec_path: path_str.to_string(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- Orden estable ---
            // Mantiene la posición de apps ya conocidas; las nuevas se añaden al final
            // (apps fijadas delante de las no fijadas entre los nuevos).
            let mut order = get_app_order().lock().unwrap();

            // Quitar las que ya no existen
            order.retain(|id| state.apps.iter().any(|a| &a.id == id));

            // Recoger IDs nuevas (primero fijadas, luego normales) antes de mutar order
            let new_pinned: Vec<String> = state
                .apps
                .iter()
                .filter(|a| a.is_pinned && !order.contains(&a.id))
                .map(|a| a.id.clone())
                .collect();
            let new_unpinned: Vec<String> = state
                .apps
                .iter()
                .filter(|a| !a.is_pinned && !order.contains(&a.id))
                .map(|a| a.id.clone())
                .collect();

            order.extend(new_pinned);
            order.extend(new_unpinned);

            // Ordenar la lista de resultados según el orden estable
            state.apps.sort_by_key(|a| {
                order
                    .iter()
                    .position(|id| id == &a.id)
                    .unwrap_or(usize::MAX)
            });
        }

        state.apps
    }
}

#[tauri::command]
fn interact_app(hwnd: isize, exec_path: String) {
    #[cfg(target_os = "windows")]
    unsafe {
        if hwnd != 0 {
            let h = HWND(hwnd as *mut _);
            // Solo restaurar si está minimizada; si ya es visible, simplemente enfocar
            // sin cambiar su estado maximizado/normal.
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
            let path_wide: Vec<u16> = exec_path.encode_utf16().chain(std::iter::once(0)).collect();
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

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum MediaControlAction {
    PlayPause,
    Next,
    Prev,
}

#[tauri::command]
fn media_control(action: MediaControlAction) {
    let mut enigo = Enigo::new();
    match action {
        MediaControlAction::PlayPause => enigo.key_click(Key::MediaPlayPause),
        MediaControlAction::Next => enigo.key_click(Key::MediaNextTrack),
        MediaControlAction::Prev => enigo.key_click(Key::MediaPrevTrack),
    }
}


#[tauri::command]
fn move_window(window: tauri::Window, x: i32, y: i32, w: u32, h: u32) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = window.hwnd() {
            use windows::Win32::UI::WindowsAndMessaging::{
                SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER,
            };

            unsafe {
                // Al tener la misma versión en Cargo.toml, podemos pasar hwnd directamente.
                let _ = SetWindowPos(
                    hwnd,
                    None, // HWND nulo para ignorar el Z-Order
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

    // Fallback de seguridad en caso de que falle la obtención del HWND
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(w, h)));
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
        x, y,
    )));
}

#[tauri::command]
fn set_window_to_bottom(window: tauri::Window) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = window.hwnd() {
            use windows::Win32::UI::WindowsAndMessaging::{
                SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            };
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_BOTTOM),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }
}

// --- COMANDOS DE VOLUMEN (Enigo) ---
//Unificar volume_mute, volume_up y volume_down en una sola funcion

#[derive(serde::Serialize, serde::Deserialize)]
enum VolumeAction {
    Mute,
    Up,
    Down,
}

#[tauri::command]
fn volume_control(action: VolumeAction) {
    let mut enigo = Enigo::new();
    match action {
        VolumeAction::Mute => enigo.key_click(Key::VolumeMute),
        VolumeAction::Up => enigo.key_click(Key::VolumeUp),
        VolumeAction::Down => enigo.key_click(Key::VolumeDown),
    }
}

#[tauri::command]
fn get_volume_status() -> VolumeInfo {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::{
            eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
        };
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
        };

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            if let Ok(enumerator) =
                CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
            {
                if let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
                    if let Ok(audio_endpoint) =
                        device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                    {
                        let volume = audio_endpoint.GetMasterVolumeLevelScalar().unwrap_or(0.0);
                        let is_muted = audio_endpoint.GetMute().unwrap_or(windows::core::BOOL(0));

                        return VolumeInfo {
                            volume,
                            is_muted: is_muted.as_bool(),
                        };
                    }
                }
            }
        }
    }

    VolumeInfo {
        volume: 0.5,
        is_muted: false,
    }
}

#[tauri::command]
fn set_volume(value: f32) {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::{
            eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
        };
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
        };

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            if let Ok(enumerator) =
                CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
            {
                if let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
                    if let Ok(audio_endpoint) =
                        device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                    {
                        let _ = audio_endpoint.SetMasterVolumeLevelScalar(value, std::ptr::null());
                    }
                }
            }
        }
    }
}

#[tauri::command]
fn toggle_mute() {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::{
            eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
        };
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
        };

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            if let Ok(enumerator) =
                CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
            {
                if let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
                    if let Ok(audio_endpoint) =
                        device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                    {
                        let is_muted = audio_endpoint.GetMute().unwrap_or(windows::core::BOOL(0));
                        let _ = audio_endpoint.SetMute(!is_muted.as_bool(), std::ptr::null());
                    }
                }
            }
        }
    }
}

// --- COMANDO DE BATERÍA (Multiplataforma) ---
#[tauri::command]
fn get_battery_status() -> BatteryInfo {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        let mut status = SYSTEM_POWER_STATUS::default();

        if unsafe { GetSystemPowerStatus(&mut status) }.is_ok() {
            return BatteryInfo {
                percentage: status.BatteryLifePercent,
                is_charging: status.ACLineStatus == 1,
                battery_saver: status.SystemStatusFlag == 1,
            };
        }
    }

    #[cfg(target_os = "linux")]
    {
        for i in 0..2 {
            let cap_path = format!("/sys/class/power_supply/BAT{}/capacity", i);
            let stat_path = format!("/sys/class/power_supply/BAT{}/status", i);

            if let Ok(cap_str) = std::fs::read_to_string(&cap_path) {
                if let Ok(cap_val) = cap_str.trim().parse::<u8>() {
                    let charging = std::fs::read_to_string(&stat_path)
                        .map(|s| s.trim() == "Charging" || s.trim() == "Full")
                        .unwrap_or(false);

                    return BatteryInfo {
                        percentage: cap_val,
                        is_charging: charging,
                    };
                }
            }
        }
    }

    BatteryInfo {
        percentage: 100,
        is_charging: true,
        battery_saver: false,
    }
}

#[tauri::command]
fn get_power_profiles() -> Vec<PowerProfile> {
    let mut profiles = Vec::new();
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::System::Power::{
            PowerEnumerate, PowerGetActiveScheme, PowerReadFriendlyName, ACCESS_SCHEME,
        };
        use windows::core::GUID;

        let mut active_guid_ptr: *mut GUID = std::ptr::null_mut();
        let _ = PowerGetActiveScheme(None, &mut active_guid_ptr);
        let active_guid = if !active_guid_ptr.is_null() {
            let guid = *active_guid_ptr;
            windows::Win32::System::Com::CoTaskMemFree(Some(active_guid_ptr as *const _));
            Some(guid)
        } else {
            None
        };

        let mut index = 0;
        loop {
            let mut guid = GUID::default();
            let mut size = std::mem::size_of::<GUID>() as u32;
            let res = PowerEnumerate(
                None,
                None,
                None,
                ACCESS_SCHEME,
                index,
                Some(&mut guid as *mut _ as *mut u8),
                &mut size,
            );

            if res.0 != 0 {
                break;
            }

            let mut name_size = 0;
            let _ = PowerReadFriendlyName(None, Some(&guid), None, None, None, &mut name_size);

            let mut name = String::new();
            if name_size > 0 {
                let mut buffer = vec![0u16; (name_size / 2) as usize];
                let _ = PowerReadFriendlyName(
                    None,
                    Some(&guid),
                    None,
                    None,
                    Some(buffer.as_mut_ptr() as *mut u8),
                    &mut name_size,
                );
                name = String::from_utf16_lossy(&buffer)
                    .trim_matches(char::from(0))
                    .to_string();
            }

            profiles.push(PowerProfile {
                guid: format!("{:?}", guid),
                name,
                active: Some(guid) == active_guid,
            });

            index += 1;
        }
    }
    profiles
}

#[tauri::command]
fn set_power_profile(guid_str: String) {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::System::Power::{PowerEnumerate, PowerSetActiveScheme, ACCESS_SCHEME};
        use windows::core::GUID;

        let mut index = 0;
        loop {
            let mut guid = GUID::default();
            let mut size = std::mem::size_of::<GUID>() as u32;
            let res = PowerEnumerate(
                None,
                None,
                None,
                ACCESS_SCHEME,
                index,
                Some(&mut guid as *mut _ as *mut u8),
                &mut size,
            );

            if res.0 != 0 {
                break;
            }

            if format!("{:?}", guid) == guid_str {
                let _ = PowerSetActiveScheme(None, Some(&guid));
                break;
            }
            index += 1;
        }
    }
}

// 4. Transformamos el extractor para devolver un JSON estructurado
#[tauri::command]
async fn get_current_media() -> Result<String, String> {
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

    #[cfg(target_os = "windows")]
    {
        use windows::Media::Control::{
            GlobalSystemMediaTransportControlsSessionManager,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus,
        };
        use windows::Storage::Streams::DataReader;

        if let Ok(manager_op) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            if let Ok(manager) = manager_op.get() {
                if let Ok(session) = manager.GetCurrentSession() {
                    if let Ok(playback_info) = session.GetPlaybackInfo() {
                        if let Ok(status) = playback_info.PlaybackStatus() {
                            info.is_playing = status
                                == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
                        }
                    }

                    if let Ok(timeline) = session.GetTimelineProperties() {
                        if let Ok(end) = timeline.EndTime() {
                            if let Ok(start) = timeline.StartTime() {
                                let dur = end.Duration - start.Duration;
                                if dur > 0 {
                                    info.duration_ms = (dur / 10_000) as u64;
                                }
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

                    if let Ok(properties_op) = session.TryGetMediaPropertiesAsync() {
                        if let Ok(properties) = properties_op.get() {
                            info.title = properties
                                .Title()
                                .map(|s| s.to_string())
                                .unwrap_or_default();
                            info.artist = properties
                                .Artist()
                                .map(|s| s.to_string())
                                .unwrap_or_default();
                            info.album = properties
                                .AlbumTitle()
                                .map(|s| s.to_string())
                                .unwrap_or_default();

                            if let Ok(thumbnail_ref) = properties.Thumbnail() {
                                if let Ok(stream_op) = thumbnail_ref.OpenReadAsync() {
                                    if let Ok(stream) = stream_op.get() {
                                        let size = stream.Size().unwrap_or(0);
                                        if let Ok(reader) = DataReader::CreateDataReader(&stream) {
                                            if let Ok(op) = reader.LoadAsync(size as u32) {
                                                if op.get().is_ok() {
                                                    let mut buffer = vec![0u8; size as usize];
                                                    if reader.ReadBytes(&mut buffer).is_ok() {
                                                        info.thumbnail_base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
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
    }

    serde_json::to_string(&info).map_err(|e| e.to_string())
}

#[tauri::command]
fn reorder_apps(ordered_ids: Vec<String>) {
    let mut order = get_app_order().lock().unwrap();
    // Sustituir el orden con la lista recibida del frontend
    let mut new_order: Vec<String> = ordered_ids
        .into_iter()
        .filter(|id| order.contains(id))
        .collect();
    // Añadir al final cualquier app que no estuviese en la lista (recien abiertas)
    for id in order.iter() {
        if !new_order.contains(id) {
            new_order.push(id.clone());
        }
    }
    *order = new_order;
}

#[tauri::command]
fn open_start_menu() {
    let mut enigo = Enigo::new();
    enigo.key_click(Key::Meta);
}

#[derive(Serialize)]
struct MonitorRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    scale_factor: f64,
}

#[tauri::command]
fn get_monitor_rects() -> Vec<MonitorRect> {
    #[cfg(not(target_os = "windows"))]
    {
        return Vec::new();
    }

    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::Foundation::RECT;
        use windows::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
        };
        use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

        let mut rects: Vec<MonitorRect> = Vec::new();

        unsafe extern "system" fn monitor_proc(
            hmonitor: HMONITOR,
            _hdc: HDC,
            _rect: *mut RECT,
            lparam: LPARAM,
        ) -> BOOL {
            let rects = &mut *(lparam.0 as *mut Vec<MonitorRect>);
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
                // Obtener el DPI efectivo del monitor para calcular el scale factor
                let mut dpi_x: u32 = 96;
                let mut dpi_y: u32 = 96;
                let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
                let scale_factor = dpi_x as f64 / 96.0;

                rects.push(MonitorRect {
                    x: info.rcMonitor.left,
                    y: info.rcMonitor.top,
                    width: info.rcMonitor.right - info.rcMonitor.left,
                    height: info.rcMonitor.bottom - info.rcMonitor.top,
                    scale_factor,
                });
            }
            BOOL(1)
        }

        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_proc),
            LPARAM(&mut rects as *mut Vec<MonitorRect> as isize),
        );

        rects
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase", tag = "type", content = "payload")]
enum WindowAction {
    Show,
    ShowAt { x: i32, y: i32 },
    Hide,
    Update { x: i32, y: i32, w: i32, h: i32 },
    UpdateLogical { x: f64, y: f64, w: f64, h: f64 },
}

#[tauri::command]
fn manage_window(app: tauri::AppHandle, label: String, action: WindowAction) {
    if let Some(window) = app.get_webview_window(&label) {
        match action {
            WindowAction::Show => {
                let _ = window.show();
            }
            WindowAction::ShowAt { x, y } => {
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
                    x, y,
                )));
                let _ = window.show();
            }
            WindowAction::Hide => {
                let _ = window.hide();
            }
            WindowAction::Update { x, y, w, h } => {
                #[cfg(target_os = "windows")]
                {
                    if let Ok(hwnd) = window.hwnd() {
                        use windows::Win32::UI::WindowsAndMessaging::{
                            SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW,
                        };
                        unsafe {
                            let _ = SetWindowPos(
                                hwnd,
                                None,
                                x,
                                y,
                                w,
                                h,
                                SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                            );
                        }
                        return;
                    }
                }

                // Fallback
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
                    w as u32, h as u32,
                )));
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
                    x, y,
                )));
                let _ = window.show();
            }
            WindowAction::UpdateLogical { x, y, w, h } => {
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(w, h)));
                let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
                    x, y,
                )));
                let _ = window.show();
            }
        }
    }
}


#[tauri::command]
fn init_taskbar_environment() {
    #[cfg(target_os = "windows")]
    unsafe {
        // 1. Lanzar el Watchdog pasándole nuestro propio PID
        let my_pid = std::process::id();

        // Asumimos que el watchdog compilado está en la misma carpeta que el ejecutable principal
        let mut watchdog_path = std::env::current_exe().unwrap_or_default();
        watchdog_path.pop(); // Subir al directorio padre
        watchdog_path.push("watchdog.exe");

        if watchdog_path.exists() {
            let _ = Command::new(watchdog_path).arg(my_pid.to_string()).spawn();
        // Spawn lo ejecuta en segundo plano y se desentiende
        } else {
            println!("Advertencia: No se encontró el ejecutable watchdog.exe");
        }

        // 2. Ocultar la barra de tareas de Windows de forma segura
        let class_name: Vec<u16> = "Shell_TrayWnd"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        if let Ok(hwnd) = FindWindowW(
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR(std::ptr::null()),
        ) {
            if !hwnd.0.is_null() {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn restore_windows_taskbar() {
    unsafe {
        let class_name: Vec<u16> = "Shell_TrayWnd"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        if let Ok(hwnd) = FindWindowW(
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR(std::ptr::null()),
        ) {
            if !hwnd.0.is_null() {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }
        }
    }
}

#[tauri::command]
fn get_brightness() -> u8 {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-Command",
                "Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness | Select-Object -ExpandProperty CurrentBrightness",
            ])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return s.parse::<u8>().unwrap_or(50);
            }
            Ok(out) => println!("[Brightness] PS Error: {}", String::from_utf8_lossy(&out.stderr)),
            Err(e) => println!("[Brightness] Command Error: {}", e),
        }
    }
    50
}

#[tauri::command]
fn set_brightness(value: u8) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-Command",
                &format!(
                    "Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Brightness={}; Timeout=0}} | Out-Null",
                    value
                ),
            ])
            .status();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            toggle_mute
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            #[cfg(target_os = "windows")]
            if let tauri::RunEvent::Exit = event {
                restore_windows_taskbar();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_taskbar_apps() {
        let apps = get_taskbar_apps();
        for app in apps {
            println!(
                "TEST APP: {} - {} (Has Icon: {})",
                app.title,
                app.exec_path,
                !app.icon_base64.is_empty()
            );
        }
    }
}
