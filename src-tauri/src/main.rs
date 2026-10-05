#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if desktop_island_lib::run_recovery_mode() {
        return;
    }
    desktop_island_lib::run()
}
