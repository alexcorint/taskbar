// Brillo de pantalla mediante IOCTL nativo — sin PowerShell.
//
// Usa DeviceIoControl con los IOCTLs de vídeo de Windows:
//   IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS  (0x00230128)
//   IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS    (0x0023012C)
//
// Batería: SYSTEM_POWER_STATUS de Win32.
// Perfiles de energía: PowerEnumerate / PowerSetActiveScheme.

use crate::types::BatteryInfo;
use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;
use windows::core::w;

// ---------------------------------------------------------------------------
// Constantes IOCTL
// ---------------------------------------------------------------------------

const IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS: u32 = 0x00230128;
const IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS: u32 = 0x0023012C;
const BRIGHTNESS_POLICY_ALL: u8 = 0;

#[repr(C)]
struct DisplayBrightness {
    policy: u8,
    ac: u8,
    dc: u8,
}

// ---------------------------------------------------------------------------
// Brillo — IOCTL nativo
// ---------------------------------------------------------------------------

pub fn get_brightness() -> u8 {
    // Intento 1: IOCTL nativo (funciona en monitores externos y algunos intégrados)
    if let Some(value) = get_brightness_ioctl() {
        return value;
    }
    // Fallback: PowerShell WMI (funciona en la mayoría de portátiles)
    get_brightness_wmi().unwrap_or(50)
}

fn get_brightness_ioctl() -> Option<u8> {
    unsafe {
        let handle = CreateFileW(
            w!("\\\\.\\LCD"),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        ).ok().filter(|h| !h.is_invalid())?;

        let mut brightness = DisplayBrightness {
            policy: BRIGHTNESS_POLICY_ALL,
            ac: 50,
            dc: 50,
        };
        let mut bytes_returned = 0u32;

        let ok = DeviceIoControl(
            handle,
            IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS,
            None,
            0,
            Some(&mut brightness as *mut _ as *mut _),
            std::mem::size_of::<DisplayBrightness>() as u32,
            Some(&mut bytes_returned),
            None,
        );

        let _ = CloseHandle(handle);

        if ok.is_ok() && bytes_returned >= 3 {
            Some(brightness.dc)
        } else {
            None
        }
    }
}

fn get_brightness_wmi() -> Option<u8> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command",
               "(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightness).CurrentBrightness"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    let s = String::from_utf8_lossy(&output.stdout);
    s.trim().parse::<u8>().ok()
}

pub fn set_brightness(value: u8) {
    // Intento 1: IOCTL nativo
    if set_brightness_ioctl(value) {
        return;
    }
    // Fallback: PowerShell WMI
    set_brightness_wmi(value);
}

fn set_brightness_ioctl(value: u8) -> bool {
    unsafe {
        let handle = match CreateFileW(
            w!("\\\\.\\LCD"),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        ) {
            Ok(h) if !h.is_invalid() => h,
            _ => return false,
        };

        let brightness = DisplayBrightness {
            policy: BRIGHTNESS_POLICY_ALL,
            ac: value,
            dc: value,
        };
        let mut bytes_returned = 0u32;

        let result = DeviceIoControl(
            handle,
            IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS,
            Some(&brightness as *const _ as *const _),
            std::mem::size_of::<DisplayBrightness>() as u32,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        );

        let _ = CloseHandle(handle);
        result.is_ok()
    }
}

fn set_brightness_wmi(value: u8) {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let cmd = format!(
        "(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods).WmiSetBrightness(0, {})",
        value
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn(); // spawn (no wait) para que no bloquee el hilo
}

// ---------------------------------------------------------------------------
// Batería
// ---------------------------------------------------------------------------

pub fn get_battery_status() -> BatteryInfo {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut status = SYSTEM_POWER_STATUS::default();

    if unsafe { GetSystemPowerStatus(&mut status) }.is_ok() {
        // BatteryLifePercent == 255 significa "Desconocido"
        if status.BatteryLifePercent != 255 {
            return BatteryInfo {
                percentage: status.BatteryLifePercent,
                is_charging: status.ACLineStatus == 1,
                battery_saver: status.SystemStatusFlag == 1,
            };
        }
    }

    // Fallback: WMI (más lento pero más preciso en algunos sistemas)
    if let Some(wmi_percentage) = get_battery_wmi() {
        return BatteryInfo {
            percentage: wmi_percentage,
            is_charging: status.ACLineStatus == 1, // Mantener el estado de carga de la API anterior si es posible
            battery_saver: status.SystemStatusFlag == 1,
        };
    }

    // Fallback final
    BatteryInfo {
        percentage: 100,
        is_charging: true,
        battery_saver: false,
    }
}

fn get_battery_wmi() -> Option<u8> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command",
               "(Get-WmiObject -Class Win32_Battery).EstimatedChargeRemaining"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    let s = String::from_utf8_lossy(&output.stdout);
    s.trim().parse::<u8>().ok()
}
