#[cfg(windows)]
mod events;
mod model;
#[cfg(all(test, windows))]
mod native_tests;
mod policy;
#[cfg(windows)]
mod recovery;
mod storage;
#[cfg(windows)]
mod windows;

use model::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

struct DesktopState {
    config: Mutex<Config>,
    snapshot: Mutex<Snapshot>,
    expanded: AtomicBool,
    #[cfg(windows)]
    recovery: Mutex<recovery::Recovery>,
}
fn lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    mutex.lock().map_err(|_| "内部状态锁不可用".into())
}

#[tauri::command]
fn get_config(state: tauri::State<DesktopState>) -> Result<Config, String> {
    Ok(lock(&state.config)?.clone())
}
#[tauri::command]
fn get_snapshot(state: tauri::State<DesktopState>) -> Result<Snapshot, String> {
    Ok(lock(&state.snapshot)?.clone())
}

fn register_shortcut(app: &tauri::AppHandle, value: &str) -> Result<(), String> {
    let shortcut: Shortcut = value.parse().map_err(|e| format!("快捷键格式无效: {e}"))?;
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _, event| {
            if event.state == ShortcutState::Pressed {
                toggle(app);
            }
        })
        .map_err(|e| format!("快捷键被占用或无法注册: {e}"))
}
fn toggle(app: &tauri::AppHandle) {
    let state = app.state::<DesktopState>();
    let expanded = !state.expanded.load(Ordering::Relaxed);
    if let Err(e) = set_mode_inner(app, expanded, None) {
        let _ = app.emit("native-error", e);
    }
}

#[tauri::command]
fn save_config(
    app: tauri::AppHandle,
    state: tauri::State<DesktopState>,
    config: Config,
) -> Result<Config, String> {
    validate_config(&config)?;
    let old = lock(&state.config)?.clone();
    let shortcut_changed = old.settings.shortcut != config.settings.shortcut;
    if shortcut_changed {
        register_shortcut(&app, &config.settings.shortcut)?;
    }
    let system_changed = old.settings.taskbar != config.settings.taskbar
        || old.settings.desktop_icons != config.settings.desktop_icons
        || old.settings.manage_desktop_icons != config.settings.manage_desktop_icons;
    let result = (|| {
        #[cfg(windows)]
        if system_changed {
            lock(&state.recovery)?.apply(&config.settings)?;
        }
        storage::save(&config)?;
        Ok::<_, String>(())
    })();
    if let Err(error) = result {
        if shortcut_changed {
            let _ = app
                .global_shortcut()
                .unregister(config.settings.shortcut.as_str());
        }
        #[cfg(windows)]
        if system_changed {
            let _ = lock(&state.recovery)?.apply(&old.settings);
        }
        return Err(error);
    }
    if shortcut_changed {
        let _ = app
            .global_shortcut()
            .unregister(old.settings.shortcut.as_str());
    }
    #[cfg(windows)]
    events::DOUBLE_WIN.store(config.settings.double_win, Ordering::Relaxed);
    *lock(&state.config)? = config.clone();
    app.emit("config-changed", &config)
        .map_err(|e| e.to_string())?;
    let _ = position(&app, state.expanded.load(Ordering::Relaxed), None);
    Ok(config)
}
fn validate_config(config: &Config) -> Result<(), String> {
    let s = &config.settings;
    if !["dark", "light"].contains(&s.theme.as_str())
        || !["system", "autoHide", "hidden"].contains(&s.taskbar.as_str())
    {
        return Err("设置选项无效".into());
    }
    if !(0..=160).contains(&s.bottom_offset)
        || !s.latitude.is_finite()
        || !s.longitude.is_finite()
        || s.latitude.abs() > 90.
        || s.longitude.abs() > 180.
    {
        return Err("位置或边距超出范围".into());
    }
    let _: Shortcut = s.shortcut.parse().map_err(|_| "快捷键格式无效")?;
    if config.pinned.len() > 100 {
        return Err("最多固定 100 个应用".into());
    }
    let mut ids = std::collections::HashSet::new();
    for pin in &config.pinned {
        if pin.name.trim().is_empty() || pin.path.is_empty() || !ids.insert(&pin.id) {
            return Err("固定应用名称、路径或标识无效".into());
        }
    }
    Ok(())
}

#[tauri::command]
fn add_app(
    app: tauri::AppHandle,
    state: tauri::State<DesktopState>,
    path: String,
) -> Result<Config, String> {
    #[cfg(windows)]
    let pin = windows::resolve_pin(&path)?;
    #[cfg(not(windows))]
    let pin = PinnedApp {
        id: model::identity(&path),
        name: path.clone(),
        match_path: Some(path.clone()),
        path,
        args: String::new(),
        icon: None,
    };
    let mut config = lock(&state.config)?.clone();
    if config.pinned.iter().any(|p| p.id == pin.id) {
        return Err("该应用已固定".into());
    }
    config.pinned.push(pin);
    save_config(app, state, config)
}
#[tauri::command]
fn pin_running(
    app: tauri::AppHandle,
    state: tauri::State<DesktopState>,
    id: String,
) -> Result<Config, String> {
    let running = lock(&state.snapshot)?
        .apps
        .iter()
        .find(|a| a.id == id)
        .cloned()
        .ok_or("应用已退出")?;
    let mut config = lock(&state.config)?.clone();
    if !config.pinned.iter().any(|p| p.id == id) {
        config.pinned.push(PinnedApp {
            id,
            name: running.name,
            match_path: Some(running.path.clone()),
            path: running.launch_path,
            args: String::new(),
            icon: running.icon,
        });
    }
    save_config(app, state, config)
}
#[tauri::command]
fn launch_app(state: tauri::State<DesktopState>, id: String) -> Result<(), String> {
    let config = lock(&state.config)?;
    let pin = config
        .pinned
        .iter()
        .find(|p| p.id == id)
        .ok_or("未找到固定应用")?;
    #[cfg(windows)]
    return windows::launch(&pin.path, &pin.args);
    #[cfg(not(windows))]
    {
        let _ = pin;
        Err("仅支持 Windows".into())
    }
}
#[tauri::command]
fn window_action(handle: String, action: String) -> Result<(), String> {
    #[cfg(windows)]
    return match action.as_str() {
        "activate" => windows::activate(&handle),
        "minimize" => windows::minimize(&handle),
        "close" => windows::close(&handle),
        _ => Err("窗口操作无效".into()),
    };
    #[cfg(not(windows))]
    {
        let _ = (handle, action);
        Err("仅支持 Windows".into())
    }
}

fn position(app: &tauri::AppHandle, expanded: bool, width: Option<f64>) -> Result<(), String> {
    let window = app.get_webview_window("island").ok_or("岛窗口未初始化")?;
    let config = lock(&app.state::<DesktopState>().config)?.clone();
    #[cfg(windows)]
    {
        let monitors = windows::monitors();
        let m = monitors
            .iter()
            .find(|m| m.info.id == config.settings.monitor)
            .or_else(|| monitors.iter().find(|m| m.info.primary))
            .or(monitors.first())
            .ok_or("未找到显示器")?;
        let scale = m.info.scale;
        let available = (m.work.right - m.work.left) as f64 / scale;
        let logical_width = if expanded {
            width.unwrap_or(780.).clamp(360., 1200.)
        } else {
            340.
        }
        .min((available - 24.).max(200.));
        let logical_height = if expanded { 440. } else { 90. };
        let physical_width = (logical_width * scale) as i32;
        let physical_height = (logical_height * scale) as i32;
        let x = m.work.left + ((m.work.right - m.work.left) - physical_width) / 2;
        let y =
            m.work.bottom - physical_height - (config.settings.bottom_offset as f64 * scale) as i32;
        window
            .set_size(tauri::PhysicalSize::new(
                physical_width as u32,
                physical_height as u32,
            ))
            .map_err(|e| e.to_string())?;
        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
        // The expanded native window reserves room above the dock for menus and window choices.
        // Hit testing is updated by the frontend to the visible glass and popup rectangles.
    }
    #[cfg(not(windows))]
    {
        let _ = (config, expanded, width);
    }
    Ok(())
}
fn set_mode_inner(
    app: &tauri::AppHandle,
    expanded: bool,
    width: Option<f64>,
) -> Result<(), String> {
    let state = app.state::<DesktopState>();
    let was_expanded = state.expanded.swap(expanded, Ordering::Relaxed);
    position(app, expanded, width)?;
    let fullscreen =
        lock(&state.snapshot)?.fullscreen && lock(&state.config)?.settings.hide_fullscreen;
    if let Some(window) = app.get_webview_window("island") {
        window.set_focusable(expanded).map_err(|e| e.to_string())?;
        if fullscreen {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            show_without_focus(&window)?;
            if expanded && !was_expanded {
                window.set_focus().map_err(|e| e.to_string())?;
            }
        }
    }
    app.emit("island-mode", if expanded { "dock" } else { "compact" })
        .map_err(|e| e.to_string())
}
fn show_without_focus(window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
            window.hwnd().map_err(|e| e.to_string())?.0 as _,
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNA,
        );
    }
    #[cfg(not(windows))]
    window.show().map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
fn set_island_mode(
    app: tauri::AppHandle,
    expanded: bool,
    width: Option<f64>,
) -> Result<(), String> {
    set_mode_inner(&app, expanded, width)
}

#[tauri::command]
fn set_hit_regions(app: tauri::AppHandle, regions: Vec<[f64; 4]>) -> Result<(), String> {
    let window = app.get_webview_window("island").ok_or("岛窗口未初始化")?;
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Graphics::Gdi::*;
        if regions.is_empty()
            || regions.len() > 8
            || regions.iter().flatten().any(|n| !n.is_finite())
        {
            return Err("交互区域无效".into());
        }
        let scale = window.scale_factor().map_err(|e| e.to_string())?;
        let combined = CreateRectRgn(0, 0, 0, 0);
        for [x, y, w, h] in regions {
            let radius = (36. * scale) as i32;
            let r = CreateRoundRectRgn(
                (x * scale) as i32,
                (y * scale) as i32,
                ((x + w) * scale) as i32 + 1,
                ((y + h) * scale) as i32 + 1,
                radius,
                radius,
            );
            CombineRgn(combined, combined, r, RGN_OR);
            DeleteObject(r);
        }
        if SetWindowRgn(
            window.hwnd().map_err(|e| e.to_string())?.0 as _,
            combined,
            1,
        ) == 0
        {
            DeleteObject(combined);
            return Err("无法更新窗口交互区域".into());
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (window, regions);
    }
    Ok(())
}
#[tauri::command]
fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("index.html?view=settings".into()),
        )
        .title("Desktop Island · 设置")
        .inner_size(1040., 760.)
        .min_inner_size(800., 600.)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
fn restore_system(
    app: tauri::AppHandle,
    state: tauri::State<DesktopState>,
) -> Result<Config, String> {
    #[cfg(windows)]
    lock(&state.recovery)?.restore()?;
    let mut config = lock(&state.config)?.clone();
    config.settings.taskbar = "system".into();
    config.settings.desktop_icons = true;
    config.settings.manage_desktop_icons = false;
    save_config(app, state, config)
}
#[tauri::command]
fn quit_app(app: tauri::AppHandle, state: tauri::State<DesktopState>) -> Result<(), String> {
    #[cfg(windows)]
    lock(&state.recovery)?.restore()?;
    app.exit(0);
    Ok(())
}

pub fn run_recovery_mode() -> bool {
    #[cfg(windows)]
    {
        return recovery::run_mode();
    }
    #[cfg(not(windows))]
    false
}

pub fn run() {
    let builder = tauri::Builder::default()
        .on_window_event(|window, event| {
            if window.label() == "island"
                && matches!(
                    event,
                    tauri::WindowEvent::Resized(_) | tauri::WindowEvent::ScaleFactorChanged { .. }
                )
            {
                let _ = window.emit("island-geometry", ());
            }
        })
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let _ = set_mode_inner(app, true, None);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_snapshot,
            save_config,
            add_app,
            pin_running,
            launch_app,
            window_action,
            set_island_mode,
            set_hit_regions,
            open_settings,
            restore_system,
            quit_app
        ])
        .setup(|app| {
            #[cfg(windows)]
            recovery::recover_stale();
            let mut config = storage::load();
            if validate_config(&config).is_err() {
                config = Config::default();
            }
            #[cfg(windows)]
            let mut recovery = recovery::Recovery::new();
            #[cfg(windows)]
            {
                if let Err(e) = recovery::spawn_watchdog() {
                    return Err(std::io::Error::other(format!("无法启动恢复进程: {e}")).into());
                }
                if let Err(e) = recovery.apply(&config.settings) {
                    eprintln!("System settings: {e}");
                    config.settings.taskbar = "system".into();
                    config.settings.desktop_icons = true;
                    config.settings.manage_desktop_icons = false;
                }
                events::DOUBLE_WIN.store(config.settings.double_win, Ordering::Relaxed);
            }
            app.manage(DesktopState {
                config: Mutex::new(config.clone()),
                snapshot: Mutex::new(Snapshot {
                    apps: vec![],
                    fullscreen: false,
                    monitors: vec![],
                    desktop_icons_visible: None,
                }),
                expanded: AtomicBool::new(false),
                #[cfg(windows)]
                recovery: Mutex::new(recovery),
            });
            if let Err(e) = register_shortcut(app.handle(), &config.settings.shortcut) {
                eprintln!("{e}");
            }
            setup_tray(app)?;
            position(app.handle(), false, None).map_err(std::io::Error::other)?;
            if let Some(window) = app.get_webview_window("island") {
                #[cfg(windows)]
                {
                    let _ = window_vibrancy::apply_blur(&window, Some((18, 24, 34, 120)));
                }
                window.set_focusable(false)?;
                show_without_focus(&window).map_err(std::io::Error::other)?;
            }
            #[cfg(windows)]
            {
                events::start();
                start_worker(app.handle().clone());
            }
            Ok(())
        });
    let app = builder
        .build(tauri::generate_context!())
        .expect("failed to initialize Desktop Island");
    app.run(|app, event| {
        if let tauri::RunEvent::Exit = event {
            #[cfg(windows)]
            {
                events::STOP.store(true, Ordering::Relaxed);
                if let Ok(mut recovery) = app.state::<DesktopState>().recovery.lock() {
                    if let Err(e) = recovery.restore() {
                        eprintln!("Recovery: {e}");
                    }
                }
            }
        }
    });
}
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::{
        menu::{Menu, MenuItem},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    };
    let toggle_item = MenuItem::with_id(app, "toggle", "展开 / 收起", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let restore = MenuItem::with_id(app, "restore", "恢复任务栏和桌面图标", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "恢复系统并退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle_item, &settings, &restore, &quit])?;
    let image = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))?;
    TrayIconBuilder::new()
        .icon(image)
        .tooltip("Desktop Island · Ctrl+Alt+Space")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let result = match event.id.as_ref() {
                "toggle" => {
                    toggle(app);
                    Ok(())
                }
                "settings" => open_settings(app.clone()),
                "restore" => restore_system(app.clone(), app.state()).map(|_| ()),
                "quit" => quit_app(app.clone(), app.state()),
                _ => Ok(()),
            };
            if let Err(e) = result {
                let _ = app.emit("native-error", e);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
#[cfg(windows)]
fn start_worker(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        unsafe {
            let _ = windows_sys::Win32::System::Com::CoInitializeEx(
                std::ptr::null(),
                windows_sys::Win32::System::Com::COINIT_MULTITHREADED as u32,
            );
        }
        let mut last_scan = std::time::Instant::now() - std::time::Duration::from_secs(5);
        let mut last_active = None::<String>;
        while !events::STOP.load(Ordering::Relaxed) {
            if events::TOGGLE.swap(false, Ordering::Relaxed) {
                toggle(&app);
            }
            if events::DIRTY.swap(false, Ordering::Relaxed) || last_scan.elapsed().as_secs() >= 5 {
                let state = app.state::<DesktopState>();
                let config = match state.config.lock() {
                    Ok(c) => c.clone(),
                    Err(_) => break,
                };
                let monitors = windows::monitors();
                let selected = monitors
                    .iter()
                    .find(|m| m.info.id == config.settings.monitor)
                    .or_else(|| monitors.iter().find(|m| m.info.primary))
                    .or(monitors.first());
                let full = selected.is_some_and(windows::fullscreen);
                let mut apps = windows::apps();
                if let Some(active) = apps.iter().flat_map(|a| &a.windows).find(|w| w.active) {
                    last_active = Some(active.handle.clone());
                } else {
                    let foreground = unsafe {
                        windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow()
                    };
                    let mut pid = 0;
                    unsafe {
                        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
                            foreground, &mut pid,
                        );
                    }
                    if pid == std::process::id() {
                        for w in apps.iter_mut().flat_map(|a| &mut a.windows) {
                            w.active = last_active.as_deref() == Some(&w.handle);
                        }
                    }
                }
                let snapshot = Snapshot {
                    apps,
                    fullscreen: full,
                    monitors: monitors.iter().map(|m| m.info.clone()).collect(),
                    desktop_icons_visible: windows::icons_visible(),
                };
                let (changed, display_changed) = if let Ok(mut current) = state.snapshot.lock() {
                    let changed = *current != snapshot;
                    let display_changed = current.monitors != snapshot.monitors;
                    if changed {
                        *current = snapshot.clone();
                    }
                    (changed, display_changed)
                } else {
                    (false, false)
                };
                if changed {
                    let _ = app.emit("desktop-snapshot", &snapshot);
                }
                if display_changed {
                    let _ = position(&app, state.expanded.load(Ordering::Relaxed), None);
                }
                if let Some(window) = app.get_webview_window("island") {
                    if full && config.settings.hide_fullscreen {
                        let _ = window.hide();
                    } else if !window.is_visible().unwrap_or(false) {
                        let _ = show_without_focus(&window);
                    }
                }
                if let Ok(mut recovery) = state.recovery.lock() {
                    let _ = recovery.explorer_changed(&config.settings);
                }
                last_scan = std::time::Instant::now();
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        unsafe {
            windows_sys::Win32::System::Com::CoUninitialize();
        }
    });
}
