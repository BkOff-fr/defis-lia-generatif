// Build script Tauri 2 — voir https://v2.tauri.app/develop/
//
// Lit `tauri.conf.json` et `capabilities/*.json`, génère les bindings
// pour `tauri::generate_context!()` et embarque les icônes.
fn main() {
    tauri_build::build();
}
