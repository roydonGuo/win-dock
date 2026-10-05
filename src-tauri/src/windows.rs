//! Windows-only platform adapter. HWND values never leave the backend except as opaque strings.
use crate::model::*;
use base64::Engine;
use std::{
    collections::BTreeMap,
    ptr::null_mut,
    sync::{Mutex, OnceLock},
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::{Dwm::*, Gdi::*},
    System::Threading::*,
    UI::{Shell::*, WindowsAndMessaging::*},
};

pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn text(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; len as usize + 1];
        let count = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        String::from_utf16_lossy(&buf[..count.max(0) as usize])
    }
}
fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut buffer = vec![0u16; 32768];
        let mut len = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut len);
        CloseHandle(process);
        (ok != 0).then(|| String::from_utf16_lossy(&buffer[..len as usize]))
    }
}
pub fn display_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Application".into())
}

// Prefer the Shell application identity when available, then normalized executable path.
fn window_app_id(hwnd: HWND) -> Option<String> {
    unsafe {
        use windows::{
            core::PCWSTR,
            Win32::{
                Foundation::HWND as WHWND,
                System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc},
                UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow},
            },
        };
        let result: windows::core::Result<IPropertyStore> =
            SHGetPropertyStoreForWindow(WHWND(hwnd));
        if let Ok(store) = result {
            let key = windows::Win32::Foundation::PROPERTYKEY {
                fmtid: windows::core::GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3),
                pid: 5,
            };
            if let Ok(mut value) = store.GetValue(&key) {
                let mut result = None;
                if let Ok(s) = PropVariantToStringAlloc(&value) {
                    let id = PCWSTR(s.0).to_string().unwrap_or_default();
                    windows::Win32::System::Com::CoTaskMemFree(Some(s.0.cast()));
                    if !id.is_empty() {
                        result = Some(id);
                    }
                }
                let _ = PropVariantClear(&mut value);
                if result.is_some() {
                    return result;
                }
            }
        }
    }
    None
}
fn process_app_id(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut length = 0;
        use windows_sys::Win32::Storage::Packaging::Appx::GetApplicationUserModelId;
        GetApplicationUserModelId(process, &mut length, null_mut());
        if length == 0 || length > 32768 {
            CloseHandle(process);
            return None;
        }
        let mut buffer = vec![0u16; length as usize];
        let code = GetApplicationUserModelId(process, &mut length, buffer.as_mut_ptr());
        CloseHandle(process);
        if code != 0 {
            return None;
        }
        let end = buffer.iter().position(|v| *v == 0).unwrap_or(buffer.len());
        Some(String::from_utf16_lossy(&buffer[..end]))
    }
}
unsafe extern "system" fn child_process(hwnd: HWND, data: LPARAM) -> BOOL {
    let pair = &mut *(data as *mut (u32, u32));
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid != pair.0 && IsWindowVisible(hwnd) != 0 && process_app_id(pid).is_some() {
        pair.1 = pid;
        return 0;
    }
    1
}

unsafe extern "system" fn enum_window(hwnd: HWND, param: LPARAM) -> BOOL {
    let apps = &mut *(param as *mut BTreeMap<String, RunningApp>);
    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }
    let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    if style & WS_EX_TOOLWINDOW != 0
        || (!GetWindow(hwnd, GW_OWNER).is_null() && style & WS_EX_APPWINDOW == 0)
    {
        return 1;
    }
    let mut cloaked = 0u32;
    DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED as u32,
        (&mut cloaked as *mut u32).cast(),
        4,
    );
    if cloaked != 0 {
        return 1;
    }
    let title = text(hwnd);
    if title.is_empty() {
        return 1;
    }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == std::process::id() {
        return 1;
    }
    let Some(mut path) = process_path(pid) else {
        return 1;
    };
    if display_name(&path).eq_ignore_ascii_case("ApplicationFrameHost") {
        let mut pair = (pid, 0);
        EnumChildWindows(
            hwnd,
            Some(child_process),
            (&mut pair as *mut (u32, u32)) as isize,
        );
        if pair.1 != 0 {
            if let Some(child_path) = process_path(pair.1) {
                pid = pair.1;
                path = child_path;
            }
        }
    }
    let app_id = window_app_id(hwnd).or_else(|| process_app_id(pid));
    let id = app_id
        .as_ref()
        .map(|id| format!("aumid:{}", id.to_lowercase()))
        .unwrap_or_else(|| identity(&path));
    let launch_path = app_id
        .filter(|id| id.contains('!'))
        .map(|id| format!("shell:AppsFolder\\{id}"))
        .unwrap_or_else(|| path.clone());
    let entry = apps.entry(id.clone()).or_insert_with(|| RunningApp {
        id,
        name: display_name(&path),
        icon: icon(&path),
        launch_path,
        path,
        windows: vec![],
    });
    entry.windows.push(AppWindow {
        handle: (hwnd as usize).to_string(),
        title,
        active: hwnd == GetForegroundWindow(),
        minimized: IsIconic(hwnd) != 0,
    });
    1
}
pub fn apps() -> Vec<RunningApp> {
    unsafe {
        let mut map = BTreeMap::new();
        EnumWindows(
            Some(enum_window),
            (&mut map as *mut BTreeMap<String, RunningApp>) as isize,
        );
        map.into_values().collect()
    }
}
pub fn validate_window(handle: &str) -> Result<HWND, String> {
    let raw = handle.parse::<usize>().map_err(|_| "窗口标识无效")?;
    let hwnd = raw as HWND;
    // Only expose operations on windows that currently belong to our application list.
    if raw == 0
        || !apps()
            .iter()
            .any(|a| a.windows.iter().any(|w| w.handle == handle))
    {
        return Err("窗口已关闭或不可操作".into());
    }
    Ok(hwnd)
}
pub fn activate(handle: &str) -> Result<(), String> {
    unsafe {
        let hwnd = validate_window(handle)?;
        if IsIconic(hwnd) != 0 {
            ShowWindowAsync(hwnd, SW_RESTORE);
        }
        if SetForegroundWindow(hwnd) == 0 {
            let flash = FLASHWINFO {
                cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
                hwnd,
                dwFlags: FLASHW_TRAY,
                uCount: 3,
                dwTimeout: 0,
            };
            FlashWindowEx(&flash);
            return Err("Windows 阻止了前台切换，请再次点击应用".into());
        }
        Ok(())
    }
}
pub fn minimize(handle: &str) -> Result<(), String> {
    unsafe {
        ShowWindowAsync(validate_window(handle)?, SW_MINIMIZE);
    }
    Ok(())
}
pub fn close(handle: &str) -> Result<(), String> {
    unsafe {
        if PostMessageW(validate_window(handle)?, WM_CLOSE, 0, 0) == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}
pub fn launch(path: &str, args: &str) -> Result<(), String> {
    unsafe {
        let op = wide("open");
        let p = wide(path);
        let a = wide(args);
        let result = ShellExecuteW(
            null_mut(),
            op.as_ptr(),
            p.as_ptr(),
            a.as_ptr(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
        if result as usize <= 32 {
            return Err(format!("无法启动应用（Shell 错误 {}）", result as usize));
        }
        Ok(())
    }
}

pub fn resolve_pin(path: &str) -> Result<PinnedApp, String> {
    let source = std::path::Path::new(path);
    if !source.is_file() {
        return Err("请选择存在的应用或快捷方式".into());
    }
    let ext = source
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    if !["exe", "lnk", "url"].contains(&ext.as_str()) {
        return Err("支持 .exe、.lnk 和 .url".into());
    }
    let mut target = path.to_string();
    let mut args = String::new();
    let mut app_id = None;
    if ext == "lnk" {
        unsafe {
            use windows::{
                core::{Interface, PCWSTR},
                Win32::{
                    Foundation::PROPERTYKEY,
                    System::Com::StructuredStorage::PropVariantToStringAlloc,
                    System::Com::{
                        CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ,
                    },
                    UI::Shell::{IShellLinkW, PropertiesSystem::IPropertyStore, ShellLink},
                },
            };
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| e.to_string())?;
            let persist: IPersistFile = link.cast().map_err(|e| e.to_string())?;
            persist
                .Load(PCWSTR(wide(path).as_ptr()), STGM_READ)
                .map_err(|e| e.to_string())?;
            let mut buf = vec![0u16; 32768];
            if link.GetPath(&mut buf, null_mut(), 0).is_ok() {
                let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
                if end > 0 {
                    target = String::from_utf16_lossy(&buf[..end]);
                }
            }
            buf.fill(0);
            if link.GetArguments(&mut buf).is_ok() {
                let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
                args = String::from_utf16_lossy(&buf[..end]);
            }
            if let Ok(store) = link.cast::<IPropertyStore>() {
                let key = PROPERTYKEY {
                    fmtid: windows::core::GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3),
                    pid: 5,
                };
                if let Ok(mut v) = store.GetValue(&key) {
                    if let Ok(s) = PropVariantToStringAlloc(&v) {
                        let id = PCWSTR(s.0).to_string().unwrap_or_default();
                        windows::Win32::System::Com::CoTaskMemFree(Some(s.0.cast()));
                        if !id.is_empty() {
                            app_id = Some(format!("aumid:{}", id.to_lowercase()));
                        }
                    }
                    let _ =
                        windows::Win32::System::Com::StructuredStorage::PropVariantClear(&mut v);
                }
            }
        }
    }
    let id = app_id.unwrap_or_else(|| identity(&target));
    // Launch the original shortcut to preserve working directory, arguments and packaged activation.
    Ok(PinnedApp {
        id,
        name: display_name(path),
        icon: icon(&target).or_else(|| icon(path)),
        path: path.to_string(),
        match_path: Some(target),
        args: if ext == "lnk" { String::new() } else { args },
    })
}

static ICONS: OnceLock<Mutex<BTreeMap<String, Option<String>>>> = OnceLock::new();
pub fn icon(path: &str) -> Option<String> {
    let cache = ICONS.get_or_init(Default::default);
    if let Some(value) = cache.lock().ok()?.get(path).cloned() {
        return value;
    }
    let value = unsafe { icon_inner(path) };
    if let Ok(mut map) = cache.lock() {
        if map.len() > 512 {
            map.clear();
        }
        map.insert(path.to_string(), value.clone());
    }
    value
}
unsafe fn icon_inner(path: &str) -> Option<String> {
    let mut info: SHFILEINFOW = std::mem::zeroed();
    if SHGetFileInfoW(
        wide(path).as_ptr(),
        0,
        &mut info,
        std::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_ICON | SHGFI_LARGEICON,
    ) == 0
        || info.hIcon.is_null()
    {
        return None;
    }
    let size = 64;
    let dc = CreateCompatibleDC(null_mut());
    let mut bitmap_info: BITMAPINFO = std::mem::zeroed();
    bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bitmap_info.bmiHeader.biWidth = size;
    bitmap_info.bmiHeader.biHeight = -size;
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32;
    let mut bits = null_mut();
    let bitmap = CreateDIBSection(dc, &bitmap_info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    if bitmap.is_null() || bits.is_null() {
        DestroyIcon(info.hIcon);
        DeleteDC(dc);
        return None;
    }
    let old = SelectObject(dc, bitmap);
    std::ptr::write_bytes(bits, 0, (size * size * 4) as usize);
    DrawIconEx(dc, 0, 0, info.hIcon, size, size, 0, null_mut(), DI_NORMAL);
    let mut rgba =
        std::slice::from_raw_parts(bits as *const u8, (size * size * 4) as usize).to_vec();
    let alpha_empty = rgba.chunks_exact(4).all(|p| p[3] == 0);
    for p in rgba.chunks_exact_mut(4) {
        p.swap(0, 2);
        if alpha_empty {
            p[3] = if p[0..3] == [0, 0, 0] { 0 } else { 255 };
        } else if p[3] > 0 {
            for i in 0..3 {
                p[i] = ((p[i] as u32 * 255) / p[3] as u32).min(255) as u8;
            }
        }
    }
    SelectObject(dc, old);
    DeleteObject(bitmap);
    DeleteDC(dc);
    DestroyIcon(info.hIcon);
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, size as u32, size as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().ok()?.write_image_data(&rgba).ok()?;
    }
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

#[derive(Clone)]
pub struct Monitor {
    pub info: MonitorInfo,
    pub work: RECT,
    pub screen: RECT,
}
unsafe extern "system" fn enum_monitor(
    handle: HMONITOR,
    _: HDC,
    _: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let monitors = &mut *(data as *mut Vec<Monitor>);
    let mut info: MONITORINFOEXW = std::mem::zeroed();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    if GetMonitorInfoW(handle, (&mut info as *mut MONITORINFOEXW).cast()) != 0 {
        let end = info
            .szDevice
            .iter()
            .position(|v| *v == 0)
            .unwrap_or(info.szDevice.len());
        let id = String::from_utf16_lossy(&info.szDevice[..end]);
        let mut x = 96;
        let mut y = 96;
        windows_sys::Win32::UI::HiDpi::GetDpiForMonitor(
            handle,
            windows_sys::Win32::UI::HiDpi::MDT_EFFECTIVE_DPI,
            &mut x,
            &mut y,
        );
        monitors.push(Monitor {
            info: MonitorInfo {
                name: id.replace("\\\\.\\", "显示器 "),
                id,
                primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
                scale: x as f64 / 96.,
                width: (info.monitorInfo.rcMonitor.right - info.monitorInfo.rcMonitor.left) as u32,
                height: (info.monitorInfo.rcMonitor.bottom - info.monitorInfo.rcMonitor.top) as u32,
                work_area: [
                    info.monitorInfo.rcWork.left,
                    info.monitorInfo.rcWork.top,
                    info.monitorInfo.rcWork.right,
                    info.monitorInfo.rcWork.bottom,
                ],
            },
            work: info.monitorInfo.rcWork,
            screen: info.monitorInfo.rcMonitor,
        });
    }
    1
}
pub fn monitors() -> Vec<Monitor> {
    unsafe {
        let mut result = vec![];
        EnumDisplayMonitors(
            null_mut(),
            std::ptr::null(),
            Some(enum_monitor),
            (&mut result as *mut Vec<Monitor>) as isize,
        );
        result
    }
}
pub fn fullscreen(monitor: &Monitor) -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() || IsIconic(hwnd) != 0 {
            return false;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == std::process::id() {
            return false;
        }
        let mut class = [0u16; 256];
        let len = GetClassNameW(hwnd, class.as_mut_ptr(), 256);
        let class = String::from_utf16_lossy(&class[..len.max(0) as usize]);
        if [
            "Progman",
            "WorkerW",
            "Shell_TrayWnd",
            "Shell_SecondaryTrayWnd",
        ]
        .contains(&class.as_str())
        {
            return false;
        }
        let mut rect: RECT = std::mem::zeroed();
        GetWindowRect(hwnd, &mut rect);
        rect.left <= monitor.screen.left
            && rect.top <= monitor.screen.top
            && rect.right >= monitor.screen.right
            && rect.bottom >= monitor.screen.bottom
    }
}

pub fn taskbars() -> Vec<HWND> {
    unsafe {
        let mut bars = vec![];
        let main = FindWindowW(wide("Shell_TrayWnd").as_ptr(), std::ptr::null());
        if !main.is_null() {
            bars.push(main);
        }
        let mut after = null_mut();
        loop {
            let bar = FindWindowExW(
                null_mut(),
                after,
                wide("Shell_SecondaryTrayWnd").as_ptr(),
                std::ptr::null(),
            );
            if bar.is_null() {
                break;
            }
            bars.push(bar);
            after = bar;
        }
        bars
    }
}
pub fn taskbar_state() -> u32 {
    unsafe {
        let mut data: APPBARDATA = std::mem::zeroed();
        data.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
        data.hWnd = taskbars().first().copied().unwrap_or(null_mut());
        SHAppBarMessage(ABM_GETSTATE, &mut data) as u32
    }
}
pub fn set_taskbar_state(state: u32) -> Result<(), String> {
    unsafe {
        if taskbars().is_empty() {
            return Err("未找到 Windows 任务栏".into());
        }
        let mut data: APPBARDATA = std::mem::zeroed();
        data.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
        data.hWnd = taskbars()[0];
        data.lParam = state as isize;
        SHAppBarMessage(ABM_SETSTATE, &mut data);
        if taskbar_state() != state {
            return Err("Windows 未应用任务栏设置".into());
        }
        Ok(())
    }
}
pub fn set_taskbars_visible(visible: bool) {
    unsafe {
        for bar in taskbars() {
            ShowWindow(bar, if visible { SW_SHOWNA } else { SW_HIDE });
        }
    }
}
pub fn desktop_view() -> Option<HWND> {
    unsafe {
        let progman = FindWindowW(wide("Progman").as_ptr(), std::ptr::null());
        let def = FindWindowExW(
            progman,
            null_mut(),
            wide("SHELLDLL_DefView").as_ptr(),
            std::ptr::null(),
        );
        if !def.is_null() {
            let list = FindWindowExW(
                def,
                null_mut(),
                wide("SysListView32").as_ptr(),
                std::ptr::null(),
            );
            if !list.is_null() {
                return Some(list);
            }
        }
        let mut worker = null_mut();
        loop {
            worker = FindWindowExW(
                null_mut(),
                worker,
                wide("WorkerW").as_ptr(),
                std::ptr::null(),
            );
            if worker.is_null() {
                return None;
            }
            let def = FindWindowExW(
                worker,
                null_mut(),
                wide("SHELLDLL_DefView").as_ptr(),
                std::ptr::null(),
            );
            if !def.is_null() {
                let list = FindWindowExW(
                    def,
                    null_mut(),
                    wide("SysListView32").as_ptr(),
                    std::ptr::null(),
                );
                if !list.is_null() {
                    return Some(list);
                }
            }
        }
    }
}
pub fn icons_visible() -> Option<bool> {
    desktop_view().map(|h| unsafe { IsWindowVisible(h) != 0 })
}
pub fn set_icons_visible(visible: bool) -> Result<(), String> {
    unsafe {
        let list = desktop_view().ok_or("未找到 Explorer 桌面图标窗口")?;
        ShowWindow(list, if visible { SW_SHOWNA } else { SW_HIDE });
        if icons_visible() != Some(visible) {
            return Err("桌面图标状态未生效".into());
        }
        Ok(())
    }
}
