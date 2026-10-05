use crate::policy::WinTap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, OnceLock,
};
use windows_sys::Win32::{
    Foundation::*,
    UI::{Accessibility::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
pub static DIRTY: AtomicBool = AtomicBool::new(true);
pub static TOGGLE: AtomicBool = AtomicBool::new(false);
pub static DOUBLE_WIN: AtomicBool = AtomicBool::new(false);
pub static STOP: AtomicBool = AtomicBool::new(false);
static TAP: OnceLock<Mutex<WinTap>> = OnceLock::new();
unsafe extern "system" fn win_event(
    _: HWINEVENTHOOK,
    event: u32,
    _: HWND,
    object: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    if event < EVENT_OBJECT_CREATE || object == 0 {
        DIRTY.store(true, Ordering::Relaxed);
    }
}
unsafe extern "system" fn keyboard(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && DOUBLE_WIN.load(Ordering::Relaxed) {
        let data = &*(lparam as *const KBDLLHOOKSTRUCT);
        let win = data.vkCode == VK_LWIN as u32 || data.vkCode == VK_RWIN as u32;
        let down = wparam as u32 == WM_KEYDOWN || wparam as u32 == WM_SYSKEYDOWN;
        if let Ok(mut tap) = TAP.get_or_init(Default::default).lock() {
            if tap.key(
                win,
                down,
                data.time as u64,
                data.flags & LLKHF_INJECTED != 0,
            ) {
                TOGGLE.store(true, Ordering::Relaxed);
            }
        }
    }
    // Observe only. Windows shortcuts and the first Win tap retain their original behavior.
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}
pub fn start() {
    std::thread::spawn(|| unsafe {
        let foreground = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            std::ptr::null_mut(),
            Some(win_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        let objects = SetWinEventHook(
            EVENT_OBJECT_CREATE,
            EVENT_OBJECT_NAMECHANGE,
            std::ptr::null_mut(),
            Some(win_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        let keyboard = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard),
            windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null()),
            0,
        );
        if foreground.is_null() || objects.is_null() || keyboard.is_null() {
            eprintln!("Some Windows hooks unavailable; periodic reconciliation remains active");
        }
        let timer = SetTimer(std::ptr::null_mut(), 0, 100, None);
        let mut message: MSG = std::mem::zeroed();
        while !STOP.load(Ordering::Relaxed)
            && GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0
        {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        KillTimer(std::ptr::null_mut(), timer);
        UnhookWinEvent(foreground);
        UnhookWinEvent(objects);
        UnhookWindowsHookEx(keyboard);
    });
}
