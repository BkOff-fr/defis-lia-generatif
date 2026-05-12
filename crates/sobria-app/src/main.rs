//! Binaire de l'application Tauri Sobr.ia — voir ADR-0001.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Sobr.ia démarre — squelette S1");
    // TODO(sobria-004): Tauri builder + commandes IPC en S6.
    // tauri::Builder::default()
    //     .invoke_handler(tauri::generate_handler![])
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");
}
