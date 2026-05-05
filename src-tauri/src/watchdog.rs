// Proceso watchdog: monitoriza el PID de la app principal y restaura
// la barra de tareas nativa cuando ésta termina.

#[cfg(target_os = "windows")]
pub fn run_watchdog(pid: u32) {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, WaitForSingleObject, INFINITE, PROCESS_SYNCHRONIZE,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_SYNCHRONIZE, false, pid).unwrap_or_default();
        if !handle.is_invalid() {
            // Bloquea hasta que la app principal muera
            let _ = WaitForSingleObject(handle, INFINITE);
            let _ = CloseHandle(handle);
        }
        crate::platform::windows::taskbar::restore_windows_taskbar();
    }
}

#[cfg(not(target_os = "windows"))]
pub fn run_watchdog(_pid: u32) {}
