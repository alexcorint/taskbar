// Monitores: posición, dimensiones y DPI.

use crate::types::MonitorRect;
use windows::core::BOOL;
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

pub fn get_monitor_rects() -> Vec<MonitorRect> {
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

        if unsafe { GetMonitorInfoW(hmonitor, &mut info) }.as_bool() {
            let mut dpi_x: u32 = 96;
            let mut dpi_y: u32 = 96;
            unsafe { let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y); }

            rects.push(MonitorRect {
                x: info.rcMonitor.left,
                y: info.rcMonitor.top,
                width: info.rcMonitor.right - info.rcMonitor.left,
                height: info.rcMonitor.bottom - info.rcMonitor.top,
                scale_factor: dpi_x as f64 / 96.0,
            });
        }

        BOOL(1)
    }

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_proc),
            LPARAM(&mut rects as *mut Vec<MonitorRect> as isize),
        );
    }

    rects
}
