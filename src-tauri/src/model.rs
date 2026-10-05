use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub monitor: String,
    pub bottom_offset: i32,
    pub auto_collapse: bool,
    pub hide_fullscreen: bool,
    pub double_win: bool,
    pub shortcut: String,
    pub taskbar: String,
    pub desktop_icons: bool,
    #[serde(default)]
    pub manage_desktop_icons: bool,
    pub weather_enabled: bool,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub reduced_motion: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            monitor: String::new(),
            bottom_offset: 18,
            auto_collapse: true,
            hide_fullscreen: true,
            double_win: false,
            shortcut: "Ctrl+Alt+Space".into(),
            taskbar: "system".into(),
            desktop_icons: true,
            manage_desktop_icons: false,
            weather_enabled: false,
            city: "北京".into(),
            latitude: 39.9042,
            longitude: 116.4074,
            reduced_motion: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PinnedApp {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub match_path: Option<String>,
    pub args: String,
    pub icon: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub pinned: Vec<PinnedApp>,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppWindow {
    pub handle: String,
    pub title: String,
    pub active: bool,
    pub minimized: bool,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunningApp {
    pub id: String,
    pub name: String,
    pub path: String,
    pub launch_path: String,
    pub icon: Option<String>,
    pub windows: Vec<AppWindow>,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub primary: bool,
    pub scale: f64,
    pub width: u32,
    pub height: u32,
    pub work_area: [i32; 4],
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Snapshot {
    pub apps: Vec<RunningApp>,
    pub fullscreen: bool,
    pub monitors: Vec<MonitorInfo>,
    #[serde(rename = "desktopIconsVisible")]
    pub desktop_icons_visible: Option<bool>,
}

pub fn identity(path: &str) -> String {
    path.replace('/', "\\").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn app_identity_normalizes_windows_paths() {
        assert_eq!(
            identity("C:/Apps/Editor.EXE"),
            identity("c:\\apps\\editor.exe")
        );
    }
}
