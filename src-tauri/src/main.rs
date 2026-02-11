#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;

#[derive(Debug, Serialize)]
struct UiNetworkSnapshot {
    network_mode: &'static str,
    proof_level: &'static str,
    ui_remote_fetch_disabled: bool,
}

// Phase_2_5_Lock_Addendum_v2.5-lock-4.md §1.1 Layer B:
// UI must not make arbitrary network calls; UI uses invoke into Rust.
#[tauri::command]
fn get_network_snapshot() -> UiNetworkSnapshot {
    UiNetworkSnapshot {
        network_mode: "OFFLINE",
        proof_level: "OFFLINE_STRICT",
        ui_remote_fetch_disabled: true,
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_network_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
