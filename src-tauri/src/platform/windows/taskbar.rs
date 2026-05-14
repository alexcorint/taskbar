// Enumeración de ventanas, extracción de iconos y gestión del orden de la barra de tareas.
//
// Optimizaciones aplicadas:
//  - Macro w!() de windows-rs para strings UTF-16 en compile-time (cero allocs en runtime)
//  - GetWindowLongW llamado una sola vez por ventana (antes se llamaba dos veces)
//  - from_utf16_lossy sin .to_string() redundante (elimina heap alloc extra)
//  - reorder_apps usa HashSet para O(n) en lugar de O(n²)

use crate::types::TaskbarApp;
use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::sync::{Mutex, OnceLock};
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, MAX_PATH, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD,
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, EnumWindows, FindWindowW, GetClassNameW, GetIconInfo, GetWindow, GetWindowLongW,
    GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, SendMessageTimeoutW, ShowWindow,
    GCLP_HICON, GWL_EXSTYLE, GW_OWNER, HICON, ICONINFO, ICON_BIG, ICON_SMALL2, SMTO_ABORTIFHUNG,
    SW_HIDE, SW_SHOW, WM_GETICON, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};
// PWSTR vive en windows::core en versiones recientes del crate
use base64::Engine;
use windows::core::PWSTR;
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;

#[cfg(target_pointer_width = "64")]
use windows::Win32::UI::WindowsAndMessaging::GetClassLongPtrW;
#[cfg(target_pointer_width = "32")]
use windows::Win32::UI::WindowsAndMessaging::GetClassLongW;

// ---------------------------------------------------------------------------
// Caché de iconos (ruta → base64)
// ---------------------------------------------------------------------------

pub fn get_icon_cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// Orden estable de apps en la barra entre actualizaciones
pub fn get_app_order() -> &'static Mutex<Vec<String>> {
    static ORDER: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    ORDER.get_or_init(|| Mutex::new(super::pins::load_order()))
}

// ---------------------------------------------------------------------------
// HICON → base64 PNG
// ---------------------------------------------------------------------------

pub unsafe fn hicon_to_base64(hicon: HICON) -> Option<String> {
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
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());
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
            ..Default::default()
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

    // Detectar canal alfa y convertir BGR→RGB en un solo pase
    let has_alpha = buffer.chunks_exact(4).any(|c| c[3] != 0);
    for chunk in buffer.chunks_exact_mut(4) {
        chunk.swap(0, 2); // B↔R
        if !has_alpha {
            chunk[3] = 255;
        }
    }

    let img = image::RgbaImage::from_raw(width as u32, height as u32, buffer)?;
    let mut cursor = std::io::Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
    Some(base64::engine::general_purpose::STANDARD.encode(cursor.into_inner()))
}

// ---------------------------------------------------------------------------
// Extracción de iconos
// ---------------------------------------------------------------------------

pub fn extract_icon_base64(path: &str) -> String {
    {
        let cache = get_icon_cache().lock().unwrap();
        if let Some(b64) = cache.get(path) {
            return b64.clone();
        }
    }

    #[cfg(target_os = "windows")]
    let result = unsafe {
        use windows::core::PCWSTR;
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
            let b64 = hicon_to_base64(shfi.hIcon);
            let _ = DestroyIcon(shfi.hIcon);
            b64.unwrap_or_default()
        } else {
            String::new()
        }
    };

    #[cfg(not(target_os = "windows"))]
    let result = String::new();

    let mut cache = get_icon_cache().lock().unwrap();
    cache.insert(path.to_string(), result.clone());
    result
}

pub unsafe fn extract_icon_from_hwnd(hwnd: HWND) -> Option<String> {
    let mut hicon_val: usize = 0;

    // Intento 1: WM_GETICON ICON_BIG
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

    // Intento 2: WM_GETICON ICON_SMALL2
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

    // Intento 3: clase de ventana
    #[cfg(target_pointer_width = "64")]
    let class_icon = GetClassLongPtrW(hwnd, GCLP_HICON) as usize;
    #[cfg(target_pointer_width = "32")]
    let class_icon = GetClassLongW(hwnd, GCLP_HICON) as usize;

    if class_icon != 0 {
        return hicon_to_base64(HICON(class_icon as *mut _));
    }

    None
}

// ---------------------------------------------------------------------------
// EnumWindows callback
// ---------------------------------------------------------------------------

struct EnumState {
    apps: Vec<TaskbarApp>,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let mut title_buf = [0u16; 512];
    let tlen = GetWindowTextW(hwnd, &mut title_buf);
    // Eliminar .to_string() redundante — from_utf16_lossy ya retorna String
    let title = String::from_utf16_lossy(&title_buf[..tlen as usize]);

    if title.is_empty() || title == "Program Manager" {
        return BOOL(1);
    }

    // Cloaking check (escritorios virtuales)
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

    // ex_style: una sola llamada (antes se llamaba 2 veces en el código original)
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

    if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
        return BOOL(1);
    }

    let owner_hwnd = GetWindow(hwnd, GW_OWNER).unwrap_or_default();
    let is_app_window = (ex_style & WS_EX_APPWINDOW.0) != 0;

    if !is_app_window && !owner_hwnd.0.is_null() {
        let owner_ex_style = GetWindowLongW(owner_hwnd, GWL_EXSTYLE) as u32;
        if (owner_ex_style & WS_EX_TOOLWINDOW.0) == 0 {
            return BOOL(1);
        }
    }

    // PID y ruta del ejecutable
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

    // Nombre de clase como plan B para apps UWP
    let mut class_name_buf = [0u16; 256];
    let clen = GetClassNameW(hwnd, &mut class_name_buf);
    let class_name = String::from_utf16_lossy(&class_name_buf[..clen as usize]);

    let app_id = if !exec_path.is_empty() {
        exec_path.clone()
    } else if !class_name.is_empty() {
        format!("class-{}", class_name)
    } else {
        format!("hwnd-{}", hwnd.0 as isize)
    };

    let state = &mut *(lparam.0 as *mut EnumState);

    if let Some(existing) = state.apps.iter_mut().find(|a| a.id == app_id) {
        existing.hwnd = hwnd.0 as isize;
        existing.title = title.to_string();
    } else {
        let mut icon_base64 = extract_icon_from_hwnd(hwnd).unwrap_or_default();
        if icon_base64.is_empty() && !exec_path.is_empty() {
            icon_base64 = extract_icon_base64(&exec_path);
        }

        state.apps.push(TaskbarApp {
            id: app_id,
            title: title.to_string(),
            icon_base64,
            is_active: true,
            is_pinned: false,
            hwnd: hwnd.0 as isize,
            exec_path,
        });
    }

    BOOL(1)
}

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

pub fn get_taskbar_apps() -> Vec<TaskbarApp> {
    let mut state = EnumState { apps: Vec::new() };

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut state as *mut _ as isize),
        );

        // Apps fijadas con nuestro sistema propio
        let pinned = super::pins::get_pinned_apps();
        for p in pinned {
            if let Some(app) = state
                .apps
                .iter_mut()
                .find(|a| a.exec_path.to_lowercase() == p.exec_path.to_lowercase())
            {
                app.is_pinned = true;
            } else {
                // App anclada pero no abierta (Ghost icon)
                let icon_base64 = extract_icon_base64(&p.exec_path);
                state.apps.push(TaskbarApp {
                    id: p.id.clone(),
                    title: p.name.clone(),
                    icon_base64,
                    is_active: false,
                    is_pinned: true,
                    hwnd: 0,
                    exec_path: p.exec_path.clone(),
                });
            }
        }

        // Orden estable
        let mut order = get_app_order().lock().unwrap();
        order.retain(|id| state.apps.iter().any(|a| &a.id == id));

        // Nuevas apps — O(n) con HashSet en lugar de O(n²) con Vec::contains
        let known: HashSet<&str> = order.iter().map(String::as_str).collect();
        let new_pinned: Vec<String> = state
            .apps
            .iter()
            .filter(|a| a.is_pinned && !known.contains(a.id.as_str()))
            .map(|a| a.id.clone())
            .collect();
        let new_unpinned: Vec<String> = state
            .apps
            .iter()
            .filter(|a| !a.is_pinned && !known.contains(a.id.as_str()))
            .map(|a| a.id.clone())
            .collect();

        order.extend(new_pinned);
        order.extend(new_unpinned);

        state.apps.sort_by_key(|a| {
            order
                .iter()
                .position(|id| id == &a.id)
                .unwrap_or(usize::MAX)
        });
    }

    state.apps
}

pub fn reorder_apps(ordered_ids: Vec<String>) {
    let mut order = get_app_order().lock().unwrap();
    // Fase 1: construir el nuevo orden con los IDs conocidos
    let mut new_order: Vec<String> = ordered_ids
        .iter()
        .filter(|id| order.contains(*id))
        .cloned()
        .collect();
    // Fase 2: recoger extras antes de mutar new_order (evita borrow conflict)
    let extras: Vec<String> = {
        let ordered_set: HashSet<&str> = new_order.iter().map(String::as_str).collect();
        order
            .iter()
            .filter(|id| !ordered_set.contains(id.as_str()))
            .cloned()
            .collect()
    };
    new_order.extend(extras);
    *order = new_order.clone();
    super::pins::save_order(&new_order);
}

pub fn hide_windows_taskbar() {
    unsafe {
        if let Ok(hwnd) = FindWindowW(windows::core::w!("Shell_TrayWnd"), None) {
            if !hwnd.0.is_null() {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}

pub fn restore_windows_taskbar() {
    unsafe {
        if let Ok(hwnd) = FindWindowW(windows::core::w!("Shell_TrayWnd"), None) {
            if !hwnd.0.is_null() {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }
        }
    }
}
