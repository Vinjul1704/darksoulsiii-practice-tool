use std::collections::{HashMap};
use std::ffi::OsString;
use std::lazy::SyncLazy;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};

use log::*;
use winapi::shared::minwindef::{HMODULE, MAX_PATH};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::{
    GetModuleFileNameW, GetModuleHandleExA, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};
use winapi::um::winuser::{GetAsyncKeyState, GetKeyNameTextW, MapVirtualKeyA};

/// Returns the path of the implementor's DLL.
pub fn get_dll_path() -> Option<PathBuf> {
    let mut hmodule: HMODULE = null_mut();
    // SAFETY
    // This is reckless, but it should never fail, and if it does, it's ok to crash and burn.
    let gmh_result = unsafe {
        GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            "DllMain".as_ptr() as _,
            &mut hmodule,
        )
    };

    if gmh_result == 0 {
        error!("get_dll_path: GetModuleHandleExA error: {:x}", unsafe {
            GetLastError()
        },);
        return None;
    }

    let mut sz_filename = [0u16; MAX_PATH];
    // SAFETY
    // pointer to sz_filename always defined and MAX_PATH bounds manually checked
    let len = unsafe { GetModuleFileNameW(hmodule, sz_filename.as_mut_ptr() as _, MAX_PATH as _) }
        as usize;

    Some(OsString::from_wide(&sz_filename[..len]).into())
}

pub(crate) struct KeyState(i32, AtomicBool);

impl KeyState {
    pub(crate) fn new(vkey: i32) -> Self {
        let state = KeyState::is_key_down(vkey);
        KeyState(vkey, AtomicBool::new(state))
    }

    pub(crate) fn keyup(&self) -> bool {
        let (prev_state, state) = self.update();
        prev_state && !state
    }

    pub(crate) fn keydown(&self) -> bool {
        let (prev_state, state) = self.update();
        !prev_state && state
    }

    fn update(&self) -> (bool, bool) {
        let state = KeyState::is_key_down(self.0);
        let prev_state = self.1.swap(state, Ordering::SeqCst);
        (prev_state, state)
    }

    fn is_key_down(vkey: i32) -> bool {
        (unsafe { GetAsyncKeyState(vkey) } & 0b1) != 0
    }
}

static VK_MAP: SyncLazy<Vec<(String, i32)>> = SyncLazy::new(|| {
    let mut map = Vec::new();

    let mut buf = [0u16; 32];

    for i in 1..128 {
        let scan_code = unsafe { MapVirtualKeyA(i, 0) } as i32;
        let len = unsafe { GetKeyNameTextW(scan_code << 16, &mut buf as *mut _, 32) };
        if len > 0 {
            let k = widestring::WideCStr::from_slice(&buf[..(len + 1) as usize])
                .unwrap()
                .to_string_lossy();
            debug!("{} VK={} SC={}", k, i, scan_code);
            map.push((k, i as i32));
        } else {
            debug!("Could not get keycode {} ({})", i, scan_code);
        }
        buf = [0u16; 32];
    }

    map
});

pub(crate) fn get_key_code(k: &str) -> Option<i32> {
    VK_SYMBOL_MAP.get(&k.to_lowercase()).copied()
}

pub(crate) fn get_key_repr(k: i32) -> Option<&'static str> {
    VK_SYMBOL_MAP_INV.get(&k).map(String::as_str)
}

pub static VK_SYMBOL_MAP: SyncLazy<HashMap<String, i32>> = SyncLazy::new(|| {
    use winapi::um::winuser::*;
    [
        ("lbutton", VK_LBUTTON),
        ("rbutton", VK_RBUTTON),
        ("cancel", VK_CANCEL),
        ("mbutton", VK_MBUTTON),
        ("xbutton1", VK_XBUTTON1),
        ("xbutton2", VK_XBUTTON2),
        ("back", VK_BACK),
        ("tab", VK_TAB),
        ("clear", VK_CLEAR),
        ("return", VK_RETURN),
        ("shift", VK_SHIFT),
        ("control", VK_CONTROL),
        ("menu", VK_MENU),
        ("pause", VK_PAUSE),
        ("capital", VK_CAPITAL),
        ("kana", VK_KANA),
        ("hangul", VK_HANGUL),
        ("junja", VK_JUNJA),
        ("final", VK_FINAL),
        ("hanja", VK_HANJA),
        ("kanji", VK_KANJI),
        ("escape", VK_ESCAPE),
        ("convert", VK_CONVERT),
        ("nonconvert", VK_NONCONVERT),
        ("accept", VK_ACCEPT),
        ("modechange", VK_MODECHANGE),
        ("space", VK_SPACE),
        ("prior", VK_PRIOR),
        ("next", VK_NEXT),
        ("end", VK_END),
        ("home", VK_HOME),
        ("left", VK_LEFT),
        ("up", VK_UP),
        ("right", VK_RIGHT),
        ("down", VK_DOWN),
        ("select", VK_SELECT),
        ("print", VK_PRINT),
        ("execute", VK_EXECUTE),
        ("snapshot", VK_SNAPSHOT),
        ("insert", VK_INSERT),
        ("delete", VK_DELETE),
        ("help", VK_HELP),
        ("0", '0' as i32),
        ("1", '1' as i32),
        ("2", '2' as i32),
        ("3", '3' as i32),
        ("4", '4' as i32),
        ("5", '5' as i32),
        ("6", '6' as i32),
        ("7", '7' as i32),
        ("8", '8' as i32),
        ("9", '9' as i32),
        ("a", 'A' as i32),
        ("b", 'B' as i32),
        ("c", 'C' as i32),
        ("d", 'D' as i32),
        ("e", 'E' as i32),
        ("f", 'F' as i32),
        ("g", 'G' as i32),
        ("h", 'H' as i32),
        ("i", 'I' as i32),
        ("j", 'J' as i32),
        ("k", 'K' as i32),
        ("l", 'L' as i32),
        ("m", 'M' as i32),
        ("n", 'N' as i32),
        ("o", 'O' as i32),
        ("p", 'P' as i32),
        ("q", 'Q' as i32),
        ("r", 'R' as i32),
        ("s", 'S' as i32),
        ("t", 'T' as i32),
        ("u", 'U' as i32),
        ("v", 'V' as i32),
        ("w", 'W' as i32),
        ("x", 'X' as i32),
        ("y", 'Y' as i32),
        ("z", 'Z' as i32),
        ("lwin", VK_LWIN),
        ("rwin", VK_RWIN),
        ("apps", VK_APPS),
        ("sleep", VK_SLEEP),
        ("numpad0", VK_NUMPAD0),
        ("numpad1", VK_NUMPAD1),
        ("numpad2", VK_NUMPAD2),
        ("numpad3", VK_NUMPAD3),
        ("numpad4", VK_NUMPAD4),
        ("numpad5", VK_NUMPAD5),
        ("numpad6", VK_NUMPAD6),
        ("numpad7", VK_NUMPAD7),
        ("numpad8", VK_NUMPAD8),
        ("numpad9", VK_NUMPAD9),
        ("multiply", VK_MULTIPLY),
        ("add", VK_ADD),
        ("separator", VK_SEPARATOR),
        ("subtract", VK_SUBTRACT),
        ("decimal", VK_DECIMAL),
        ("divide", VK_DIVIDE),
        ("f1", VK_F1),
        ("f2", VK_F2),
        ("f3", VK_F3),
        ("f4", VK_F4),
        ("f5", VK_F5),
        ("f6", VK_F6),
        ("f7", VK_F7),
        ("f8", VK_F8),
        ("f9", VK_F9),
        ("f10", VK_F10),
        ("f11", VK_F11),
        ("f12", VK_F12),
        ("f13", VK_F13),
        ("f14", VK_F14),
        ("f15", VK_F15),
        ("f16", VK_F16),
        ("f17", VK_F17),
        ("f18", VK_F18),
        ("f19", VK_F19),
        ("f20", VK_F20),
        ("f21", VK_F21),
        ("f22", VK_F22),
        ("f23", VK_F23),
        ("f24", VK_F24),
        ("numlock", VK_NUMLOCK),
        ("scroll", VK_SCROLL),
        ("lshift", VK_LSHIFT),
        ("rshift", VK_RSHIFT),
        ("lcontrol", VK_LCONTROL),
        ("rcontrol", VK_RCONTROL),
        ("lmenu", VK_LMENU),
        ("rmenu", VK_RMENU),
        ("browser_back", VK_BROWSER_BACK),
        ("browser_forward", VK_BROWSER_FORWARD),
        ("browser_refresh", VK_BROWSER_REFRESH),
        ("browser_stop", VK_BROWSER_STOP),
        ("browser_search", VK_BROWSER_SEARCH),
        ("browser_favorites", VK_BROWSER_FAVORITES),
        ("browser_home", VK_BROWSER_HOME),
        ("volume_mute", VK_VOLUME_MUTE),
        ("volume_down", VK_VOLUME_DOWN),
        ("volume_up", VK_VOLUME_UP),
        ("media_next_track", VK_MEDIA_NEXT_TRACK),
        ("media_prev_track", VK_MEDIA_PREV_TRACK),
        ("media_stop", VK_MEDIA_STOP),
        ("media_play_pause", VK_MEDIA_PLAY_PAUSE),
        ("launch_mail", VK_LAUNCH_MAIL),
        ("launch_media_select", VK_LAUNCH_MEDIA_SELECT),
        ("launch_app1", VK_LAUNCH_APP1),
        ("launch_app2", VK_LAUNCH_APP2),
        ("oem_1", VK_OEM_1),
        ("oem_plus", VK_OEM_PLUS),
        ("oem_comma", VK_OEM_COMMA),
        ("oem_minus", VK_OEM_MINUS),
        ("oem_period", VK_OEM_PERIOD),
        ("oem_2", VK_OEM_2),
        ("oem_3", VK_OEM_3),
        ("oem_4", VK_OEM_4),
        ("oem_5", VK_OEM_5),
        ("oem_6", VK_OEM_6),
        ("oem_7", VK_OEM_7),
        ("oem_8", VK_OEM_8),
        ("oem_102", VK_OEM_102),
        ("processkey", VK_PROCESSKEY),
        ("packet", VK_PACKET),
        ("attn", VK_ATTN),
        ("crsel", VK_CRSEL),
        ("exsel", VK_EXSEL),
        ("ereof", VK_EREOF),
        ("play", VK_PLAY),
        ("zoom", VK_ZOOM),
        ("noname", VK_NONAME),
        ("pa1", VK_PA1),
        ("oem_clear", VK_OEM_CLEAR),
    ]
    .iter()
    .map(|&(k, v)| (String::from(k).to_lowercase(), v))
    .collect()
});

pub static VK_SYMBOL_MAP_INV: SyncLazy<HashMap<i32, String>> = SyncLazy::new(|| {
    VK_SYMBOL_MAP.iter().map(|(k, &v)| (v, k.clone())).collect()
});
