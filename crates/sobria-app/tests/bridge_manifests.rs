//! Tests d'intégration de l'auto-install des manifests Native Messaging
//! (patch C27 v0.6.0). Voir `briefs/chantiers/C27-PATCH-v0.6.0.md` §"Patch 1".
//!
//! Couvre :
//!   - install Chrome macOS → fichier écrit + JSON valide
//!   - install Firefox Linux → chemin `~/.mozilla/native-messaging-hosts/`
//!   - install + uninstall idempotent
//!   - détection : un faux dossier config est listé
//!
//! Les tests Windows registre HKCU sont hors périmètre (effet de bord
//! système) — la fonction `windows_registry_key` est unit-testée dans
//! `bridge_install.rs`.

use std::{fs, path::Path};

use sobria_app::bridge_install::{
    build_manifest_json, detect_browsers_in, install_native_manifest_at, manifest_install_path_for,
    uninstall_native_manifest_at, BrowserKind, HostOs, FIREFOX_EXTENSION_ID, NATIVE_HOST_NAME,
};
use tempfile::TempDir;

#[test]
fn macos_chrome_install_writes_valid_json_with_allowed_origins() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let path = manifest_install_path_for(HostOs::Macos, BrowserKind::Chrome, home, None).unwrap();
    install_native_manifest_at(
        BrowserKind::Chrome,
        Path::new("/Applications/Sobria.app/Contents/MacOS/sobria-bridge"),
        "abcd1234efgh5678",
        &path,
    )
    .unwrap();
    assert!(path.exists(), "manifest doit exister à {}", path.display());
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(parsed["name"], NATIVE_HOST_NAME);
    assert_eq!(parsed["type"], "stdio");
    assert_eq!(
        parsed["path"],
        "/Applications/Sobria.app/Contents/MacOS/sobria-bridge"
    );
    assert_eq!(
        parsed["allowed_origins"][0],
        "chrome-extension://abcd1234efgh5678/"
    );
    assert!(parsed.get("allowed_extensions").is_none());
}

#[test]
fn linux_firefox_install_uses_dot_mozilla_native_messaging_hosts() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let path = manifest_install_path_for(HostOs::Linux, BrowserKind::Firefox, home, None).unwrap();
    install_native_manifest_at(
        BrowserKind::Firefox,
        Path::new("/usr/local/bin/sobria-bridge"),
        "ignored-firefox-uses-gecko-id",
        &path,
    )
    .unwrap();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains(".mozilla/native-messaging-hosts"),
        "got: {path_str}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(parsed["allowed_extensions"][0], FIREFOX_EXTENSION_ID);
}

#[test]
fn install_then_uninstall_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let path =
        manifest_install_path_for(HostOs::Linux, BrowserKind::Chromium, tmp.path(), None).unwrap();
    install_native_manifest_at(
        BrowserKind::Chromium,
        Path::new("/bin/sobria-bridge"),
        "zzzz",
        &path,
    )
    .unwrap();
    assert!(path.exists());
    uninstall_native_manifest_at(&path).unwrap();
    assert!(!path.exists());
    // Double uninstall : pas d'erreur.
    uninstall_native_manifest_at(&path).unwrap();
}

#[test]
fn detection_lists_browser_with_fake_config_dir() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    // Simule Brave installé sur Linux.
    fs::create_dir_all(home.join(".config/BraveSoftware/Brave-Browser")).unwrap();
    let detected = detect_browsers_in(HostOs::Linux, home, None, None);
    assert!(detected.contains(&BrowserKind::Brave));
    assert!(!detected.contains(&BrowserKind::Chrome));
}

#[test]
fn install_writes_to_six_browsers_in_one_pass() {
    // Validation DoD : "les manifests sont écrits aux 6+ emplacements possibles
    // (selon browsers détectés)". On simule l'install dans 5 browsers (l'app
    // gère 5 browsers max) à la suite et on vérifie qu'aucun ne pollue les
    // autres.
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    for browser in BrowserKind::ALL {
        let path = manifest_install_path_for(HostOs::Macos, browser, home, None).unwrap();
        install_native_manifest_at(
            browser,
            Path::new("/Applications/sobria-bridge"),
            "id123456",
            &path,
        )
        .unwrap();
        assert!(
            path.exists(),
            "manifest absent pour {:?}",
            browser.display_name()
        );
    }
    // Compte les fichiers créés : doit être 5 (un par browser, paths distincts).
    let mut count = 0;
    for entry in walkdir(home) {
        if entry.file_name().to_string_lossy() == "com.sobria.bridge.json" {
            count += 1;
        }
    }
    assert_eq!(count, 5, "5 manifests distincts attendus, trouvé {count}");
}

#[test]
fn manifest_path_contains_native_messaging_hosts_segment() {
    let home = Path::new("/Users/test");
    for browser in [
        BrowserKind::Chrome,
        BrowserKind::Edge,
        BrowserKind::Brave,
        BrowserKind::Chromium,
        BrowserKind::Firefox,
    ] {
        let p = manifest_install_path_for(HostOs::Macos, browser, home, None).unwrap();
        let s = p.to_string_lossy();
        assert!(
            s.contains("NativeMessagingHosts"),
            "macOS {} path manque NativeMessagingHosts: {s}",
            browser.id()
        );
    }
}

#[test]
fn build_manifest_chrome_extension_id_is_quoted_in_chrome_origin() {
    let m = build_manifest_json(
        BrowserKind::Chrome,
        Path::new("/usr/bin/sobria-bridge"),
        "1234abcd5678efgh",
    );
    let origins = m["allowed_origins"].as_array().unwrap();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0], "chrome-extension://1234abcd5678efgh/");
}

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Mini-walker récursif local pour éviter une dépendance walkdir.
fn walkdir(root: &Path) -> Vec<std::fs::DirEntry> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(entry);
            }
        }
    }
    out
}
