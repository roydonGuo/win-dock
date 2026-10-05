//! Read-only integration smoke checks: never launch apps or change Shell settings.
#[test]
fn windows_adapter_reads_monitors_windows_shortcuts_and_decodable_icons() {
    unsafe {
        let _ = windows_sys::Win32::System::Com::CoInitializeEx(
            std::ptr::null(),
            windows_sys::Win32::System::Com::COINIT_MULTITHREADED as u32,
        );
    }
    let monitors = crate::windows::monitors();
    assert!(
        !monitors.is_empty(),
        "Windows desktop must expose at least one monitor"
    );
    assert!(monitors
        .iter()
        .all(|m| m.info.scale > 0. && m.work.right > m.work.left));
    let apps = crate::windows::apps();
    assert!(apps
        .iter()
        .all(|a| !a.id.is_empty() && !a.windows.is_empty()));
    let path = std::path::PathBuf::from(std::env::var_os("WINDIR").unwrap()).join("explorer.exe");
    let pin =
        crate::windows::resolve_pin(&path.to_string_lossy()).expect("resolve Explorer executable");
    assert_eq!(pin.id, crate::model::identity(&path.to_string_lossy()));
    assert!(pin.match_path.is_some());
    // Build a real shortcut, resolve it, and verify we keep the .lnk launch path.
    // Never execute the shortcut or its arguments.
    unsafe {
        use windows::{
            core::{Interface, PCWSTR},
            Win32::{
                System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER},
                UI::Shell::{IShellLinkW, ShellLink},
            },
        };
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).unwrap();
        link.SetPath(PCWSTR(
            crate::windows::wide(&path.to_string_lossy()).as_ptr(),
        ))
        .unwrap();
        link.SetArguments(PCWSTR(crate::windows::wide("/select,C:\\").as_ptr()))
            .unwrap();
        let shortcut =
            std::env::temp_dir().join(format!("desktop-island-test-{}.lnk", std::process::id()));
        let persist: IPersistFile = link.cast().unwrap();
        persist
            .Save(
                PCWSTR(crate::windows::wide(&shortcut.to_string_lossy()).as_ptr()),
                true,
            )
            .unwrap();
        let result = crate::windows::resolve_pin(&shortcut.to_string_lossy());
        std::fs::remove_file(&shortcut).unwrap();
        let shortcut_pin = result.expect("resolve real Shell shortcut");
        assert_eq!(shortcut_pin.id, pin.id);
        assert_eq!(
            shortcut_pin
                .match_path
                .as_deref()
                .map(crate::model::identity),
            pin.match_path.as_deref().map(crate::model::identity)
        );
        assert_eq!(shortcut_pin.path, shortcut.to_string_lossy());
        assert!(
            shortcut_pin.args.is_empty(),
            "ShellExecute launches original link to preserve its arguments"
        );
    }
    let data = pin.icon.expect("Explorer icon must be readable");
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data.strip_prefix("data:image/png;base64,").unwrap())
        .unwrap();
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .expect("valid PNG header");
    let mut buffer = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buffer).expect("valid PNG pixels");
    assert_eq!((frame.width, frame.height), (64, 64));
    println!(
        "Native read-only smoke: {} monitors, {} application groups; icon PNG decoded",
        monitors.len(),
        apps.len()
    );
    unsafe {
        windows_sys::Win32::System::Com::CoUninitialize();
    }
}
