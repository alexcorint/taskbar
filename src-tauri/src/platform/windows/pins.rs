use crate::types::PinnedApp;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub fn get_app_data_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let mut p = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push("vibrant-dawn-taskbar");
        let _ = fs::create_dir_all(&p);
        p
    })
}

pub fn get_pins_path() -> PathBuf {
    let mut p = get_app_data_dir().clone();
    p.push("pinned_apps.json");
    p
}

pub fn get_pins_store() -> &'static Mutex<Vec<PinnedApp>> {
    static STORE: OnceLock<Mutex<Vec<PinnedApp>>> = OnceLock::new();
    STORE.get_or_init(|| {
        let path = get_pins_path();
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(pins) = serde_json::from_str(&content) {
                return Mutex::new(pins);
            }
        }
        Mutex::new(Vec::new())
    })
}

#[allow(dead_code)]
pub fn save_pins(pins: &Vec<PinnedApp>) {
    let path = get_pins_path();
    if let Ok(content) = serde_json::to_string_pretty(pins) {
        let _ = fs::write(path, content);
    }
}

#[allow(dead_code)]
pub fn pin_app(app: PinnedApp) {
    let mut store = get_pins_store().lock().unwrap();
    if !store.iter().any(|a| a.id == app.id) {
        store.push(app);
        save_pins(&store);
    }
}

#[allow(dead_code)]
pub fn unpin_app(id: &str) {
    let mut store = get_pins_store().lock().unwrap();
    store.retain(|a| a.id != id);
    save_pins(&store);
}

#[allow(dead_code)]
pub fn get_pinned_apps() -> Vec<PinnedApp> {
    get_pins_store().lock().unwrap().clone()
}

pub fn get_order_path() -> PathBuf {
    let mut p = get_app_data_dir().clone();
    p.push("apps_order.json");
    p
}

pub fn save_order(order: &Vec<String>) {
    let path = get_order_path();
    if let Ok(content) = serde_json::to_string(order) {
        let _ = fs::write(path, content);
    }
}

pub fn load_order() -> Vec<String> {
    let path = get_order_path();
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(order) = serde_json::from_str(&content) {
            return order;
        }
    }
    Vec::new()
}
