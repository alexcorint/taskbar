// Teclas multimedia y de volumen mediante SendInput nativo.
// Reemplaza la dependencia `enigo` en Windows con llamadas directas a Win32.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, VIRTUAL_KEY,
    VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_VOLUME_DOWN,
    VK_VOLUME_MUTE, VK_VOLUME_UP,
};

fn press_key(vk: VIRTUAL_KEY) {
    let key_down = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let key_up = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: KEYBD_EVENT_FLAGS(0x0002), // KEYEVENTF_KEYUP
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        SendInput(&[key_down, key_up], std::mem::size_of::<INPUT>() as i32);
    }
}

pub fn media_play_pause() {
    press_key(VK_MEDIA_PLAY_PAUSE);
}

pub fn media_next() {
    press_key(VK_MEDIA_NEXT_TRACK);
}

pub fn media_prev() {
    press_key(VK_MEDIA_PREV_TRACK);
}

pub fn volume_mute() {
    press_key(VK_VOLUME_MUTE);
}

pub fn volume_up() {
    press_key(VK_VOLUME_UP);
}

pub fn volume_down() {
    press_key(VK_VOLUME_DOWN);
}

pub fn open_start_menu() {
    // La tecla Windows (VK_LWIN) abre el menú de inicio
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_LWIN;
    press_key(VK_LWIN);
}
