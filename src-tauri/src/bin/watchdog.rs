// Evita que el watchdog abra su propia ventana de consola
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, WaitForSingleObject, INFINITE, PROCESS_SYNCHRONIZE,
};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, ShowWindow, SW_SHOW};

fn main() {
    // Leemos el PID que nos pasará la aplicación principal por consola
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }

    let pid: u32 = match args[1].parse() {
        Ok(num) => num,
        Err(_) => return,
    };

    unsafe {
        // Pedimos permiso SOLO para sincronizarnos (esperar a que cierre), sin leer memoria
        let process_handle = OpenProcess(PROCESS_SYNCHRONIZE, false, pid).unwrap_or_default();
        if !process_handle.is_invalid() {
            // El hilo se bloquea aquí indefinidamente (0% CPU) hasta que tu app de Tauri se cierre o crashee
            WaitForSingleObject(process_handle, INFINITE);
            let _ = CloseHandle(process_handle);
        }

        // ¡Tu app ha muerto! Restauramos la barra de Windows inmediatamente
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
