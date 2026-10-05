use crate::{model::Settings, policy::should_restore, storage, windows};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::IsWindowVisible};

#[derive(Clone, Serialize, Deserialize)]
struct ValueChange {
    original: u32,
    applied: u32,
}
#[derive(Clone, Serialize, Deserialize)]
struct BarChange {
    handle: usize,
    original: u32,
    applied: u32,
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct Journal {
    taskbar: Option<ValueChange>,
    bars: Vec<BarChange>,
    icons: Option<ValueChange>,
    #[serde(default)]
    taskbar_handle: usize,
    #[serde(default)]
    icon_handle: usize,
}
// A named mutex coordinates the previous watchdog and a newly started application.
// Windows mutexes are recursive, so apply -> restore is safe on the same thread.
struct RecoveryLock(HANDLE);
impl RecoveryLock {
    fn acquire() -> Result<Self, String> {
        unsafe {
            use windows_sys::Win32::System::Threading::*;
            let handle = CreateMutexW(
                std::ptr::null(),
                0,
                windows::wide("Local\\DesktopIslandRecovery").as_ptr(),
            );
            if handle.is_null() {
                return Err(std::io::Error::last_os_error().to_string());
            }
            let result = WaitForSingleObject(handle, 10000);
            if result != WAIT_OBJECT_0 && result != WAIT_ABANDONED {
                CloseHandle(handle);
                return Err("系统恢复正在被其他实例使用，请稍后重试".into());
            }
            Ok(Self(handle))
        }
    }
}
impl Drop for RecoveryLock {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::System::Threading::ReleaseMutex(self.0);
            CloseHandle(self.0);
        }
    }
}
pub struct Recovery {
    path: PathBuf,
    journal: Journal,
}
impl Recovery {
    pub fn new() -> Self {
        Self {
            path: storage::root().join(format!("recovery-{}.json", std::process::id())),
            journal: Journal::default(),
        }
    }
    fn persist(&self) -> Result<(), String> {
        storage::write_json(&self.path, &self.journal)
    }
    pub fn apply(&mut self, settings: &Settings) -> Result<(), String> {
        let _guard = RecoveryLock::acquire()?;
        self.restore()?;
        let mut journal = Journal::default();
        if settings.taskbar == "autoHide" {
            let original = windows::taskbar_state();
            journal.taskbar = Some(ValueChange {
                original,
                applied: original | 1,
            });
            journal.taskbar_handle = windows::taskbars()
                .first()
                .copied()
                .unwrap_or(std::ptr::null_mut()) as usize;
        }
        if settings.taskbar == "hidden" {
            journal.bars = windows::taskbars()
                .iter()
                .map(|h| BarChange {
                    handle: *h as usize,
                    original: unsafe { (IsWindowVisible(*h) != 0) as u32 },
                    applied: 0,
                })
                .collect();
            if journal.bars.is_empty() {
                return Err("未找到任务栏，无法隐藏".into());
            }
        }
        if settings.manage_desktop_icons {
            let original = windows::icons_visible().ok_or("桌面图标控制暂不可用")? as u32;
            journal.icons = Some(ValueChange {
                original,
                applied: settings.desktop_icons as u32,
            });
            journal.icon_handle = windows::desktop_view().unwrap_or(std::ptr::null_mut()) as usize;
        }
        self.journal = journal;
        self.persist()?; // Write ahead: recovery survives a crash during the next operation.
        let result = (|| {
            if let Some(change) = &self.journal.taskbar {
                windows::set_taskbar_state(change.applied)?;
            }
            if !self.journal.bars.is_empty() {
                windows::set_taskbars_visible(false);
            }
            if let Some(change) = &self.journal.icons {
                windows::set_icons_visible(change.applied != 0)?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = self.restore();
        }
        result
    }
    pub fn restore(&mut self) -> Result<(), String> {
        let _guard = RecoveryLock::acquire()?;
        restore_journal(&self.journal)?;
        self.journal = Journal::default();
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn explorer_changed(&mut self, settings: &Settings) -> Result<(), String> {
        let stale = |h: usize| {
            h != 0
                && unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsWindow(h as HWND) == 0 }
        };
        if stale(self.journal.taskbar_handle)
            || stale(self.journal.icon_handle)
            || self.journal.bars.iter().any(|bar| stale(bar.handle))
        {
            self.apply(settings)?;
        }
        Ok(())
    }
}
fn restore_journal(j: &Journal) -> Result<(), String> {
    if (j.taskbar.is_some() || !j.bars.is_empty()) && windows::taskbars().is_empty() {
        return Err("等待 Explorer 任务栏恢复".into());
    }
    if let Some(change) = &j.taskbar {
        if should_restore(change.original, change.applied, windows::taskbar_state()) {
            windows::set_taskbar_state(change.original)?;
        }
    }
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        for bar in &j.bars {
            let hwnd = bar.handle as HWND;
            if IsWindow(hwnd) != 0
                && should_restore(
                    bar.original,
                    bar.applied,
                    (IsWindowVisible(hwnd) != 0) as u32,
                )
            {
                ShowWindow(
                    hwnd,
                    if bar.original != 0 {
                        SW_SHOWNA
                    } else {
                        SW_HIDE
                    },
                );
            }
        }
    }
    if let Some(change) = &j.icons {
        let current = windows::icons_visible().ok_or("等待 Explorer 桌面恢复")?;
        if should_restore(change.original, change.applied, current as u32) {
            windows::set_icons_visible(change.original != 0)?;
        }
    }
    Ok(())
}
fn restore_file(path: &Path) -> Result<(), String> {
    let _guard = RecoveryLock::acquire()?;
    if !path.exists() {
        return Ok(());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let j = serde_json::from_slice::<Journal>(&bytes).map_err(|e| e.to_string())?;
    restore_journal(&j)?;
    fs::remove_file(path).map_err(|e| e.to_string())
}
pub fn recover_stale() {
    if let Ok(entries) = fs::read_dir(storage::root()) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("recovery-") && name.ends_with(".json") {
                if let Err(e) = restore_file(&path) {
                    eprintln!("Recovery: {e}");
                }
            }
        }
    }
}
pub fn spawn_watchdog() -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    std::process::Command::new(exe)
        .arg("--recovery-watchdog")
        .arg(std::process::id().to_string())
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn run_mode() -> bool {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("--recovery-watchdog") {
        return false;
    }
    let Some(pid) = args.get(2).and_then(|v| v.parse::<u32>().ok()) else {
        return true;
    };
    unsafe {
        use windows_sys::Win32::System::Threading::*;
        let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if !handle.is_null() {
            WaitForSingleObject(handle, INFINITE);
            CloseHandle(handle);
        }
    }
    let path = storage::root().join(format!("recovery-{pid}.json"));
    // Explorer may still be restarting; retain the journal if restoration cannot be completed.
    for _ in 0..10 {
        if restore_file(&path).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    true
}
