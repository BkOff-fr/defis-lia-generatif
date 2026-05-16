//! Sobr.ia — auto-install des manifests Native Messaging (C27 patch v0.6.0).
//!
//! Remplace les scripts shell/PowerShell `crates/sobria-bridge/scripts/install-dev.*`
//! pour l'utilisateur final : l'app Tauri écrit elle-même les fichiers
//! `com.sobria.bridge.json` aux emplacements OS attendus par chaque
//! WebExtension (Chrome / Firefox / Edge / Brave / Chromium).
//!
//! ## Emplacements supportés
//!
//! ### macOS — `~/Library/Application Support`
//! | Browser   | Sous-dossier |
//! |-----------|--------------|
//! | Chrome    | `Google/Chrome/NativeMessagingHosts` |
//! | Firefox   | `Mozilla/NativeMessagingHosts` |
//! | Edge      | `Microsoft Edge/NativeMessagingHosts` |
//! | Brave     | `BraveSoftware/Brave-Browser/NativeMessagingHosts` |
//! | Chromium  | `Chromium/NativeMessagingHosts` |
//!
//! ### Linux
//! | Browser   | Chemin |
//! |-----------|--------|
//! | Chrome    | `~/.config/google-chrome/NativeMessagingHosts` |
//! | Firefox   | `~/.mozilla/native-messaging-hosts` *(hors `~/.config`)* |
//! | Edge      | `~/.config/microsoft-edge/NativeMessagingHosts` |
//! | Brave     | `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts` |
//! | Chromium  | `~/.config/chromium/NativeMessagingHosts` |
//!
//! ### Windows
//! Le fichier manifest est écrit dans `%APPDATA%\Sobria\bridge\com.sobria.bridge.json`
//! (chemin commun, non-browser-spécifique) et une clé `HKCU\Software\<Vendor>\<Browser>\NativeMessagingHosts\com.sobria.bridge`
//! pointe dessus (valeur par défaut = chemin absolu du JSON). Implémentation
//! via `reg.exe ADD` / `reg.exe DELETE` (binaire Windows standard, pas de
//! crate `winreg` ajoutée — cf. anti-périmètre du patch).
//!
//! ## Sécurité
//!
//! - Les manifests sont écrits avec consentement utilisateur explicite
//!   (dialog dédié dans `/parametres`).
//! - Le `allowed_origins` (Chrome/Edge/Brave/Chromium) restreint le bridge
//!   à un seul `extension_id`.
//! - Le `allowed_extensions` (Firefox) restreint à `sobria@sobr.ia` (cf.
//!   `extension/manifest.firefox.json`).

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Nom du native host (cf. `chrome.runtime.connectNative("com.sobria.bridge")`).
pub const NATIVE_HOST_NAME: &str = "com.sobria.bridge";

/// Description embarquée dans le manifest JSON.
pub const NATIVE_HOST_DESCRIPTION: &str = "Sobr.ia native messaging bridge";

/// ID de l'add-on Firefox (cf. `extension/manifest.firefox.json` → `browser_specific_settings.gecko.id`).
pub const FIREFOX_EXTENSION_ID: &str = "sobria@sobr.ia";

/// Navigateurs supportés par l'auto-install.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserKind {
    Chrome,
    Firefox,
    Edge,
    Brave,
    Chromium,
}

impl BrowserKind {
    /// Liste exhaustive des browsers gérés.
    pub const ALL: [BrowserKind; 5] = [
        Self::Chrome,
        Self::Firefox,
        Self::Edge,
        Self::Brave,
        Self::Chromium,
    ];

    /// Identifiant stable kebab-case (logs, sérialisation).
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Firefox => "firefox",
            Self::Edge => "edge",
            Self::Brave => "brave",
            Self::Chromium => "chromium",
        }
    }

    /// Nom lisible UI.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Chrome => "Google Chrome",
            Self::Firefox => "Mozilla Firefox",
            Self::Edge => "Microsoft Edge",
            Self::Brave => "Brave",
            Self::Chromium => "Chromium",
        }
    }

    /// `true` si ce browser utilise le format `allowed_extensions` (Firefox)
    /// plutôt que `allowed_origins` (Chrome-family).
    #[must_use]
    pub const fn is_firefox(self) -> bool {
        matches!(self, Self::Firefox)
    }
}

/// OS hôte. Utilisé en interne pour rendre les fonctions testables sur
/// n'importe quelle plateforme (le test passe l'OS désiré sans dépendre
/// du runtime).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostOs {
    Macos,
    Linux,
    Windows,
}

impl HostOs {
    /// OS courant (lu à la compilation via `target_os`).
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

/// Statut courant du bridge — surfacé par l'IPC `bridge_status` au frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatus {
    /// Chemin absolu du binaire `sobria-bridge` détecté à côté de l'app,
    /// ou `None` si introuvable.
    pub bridge_path: Option<PathBuf>,
    /// Browsers dont le manifest natif est déjà installé.
    pub installed: Vec<BrowserKind>,
    /// Browsers détectés sur la machine (config dir présent).
    pub detected: Vec<BrowserKind>,
}

/// Détecte les navigateurs présents en regardant l'existence de leur dossier
/// de profils utilisateur. Indicatif — un browser sans config dir peut tout
/// de même supporter le pairing si l'utilisateur le lance plus tard.
#[must_use]
pub fn detect_installed_browsers() -> Vec<BrowserKind> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let appdata = dirs::config_dir();
    let local = dirs::data_local_dir();
    detect_browsers_in(HostOs::current(), &home, appdata.as_deref(), local.as_deref())
}

/// Variante testable : passe l'OS cible et les `home`/`appdata`/`localappdata`
/// explicitement. Utile pour tester la résolution macOS depuis Linux/Windows
/// avec un `tempdir` mocké.
#[must_use]
pub fn detect_browsers_in(
    os: HostOs,
    home: &Path,
    appdata: Option<&Path>,
    local: Option<&Path>,
) -> Vec<BrowserKind> {
    BrowserKind::ALL
        .into_iter()
        .filter(|b| user_data_path(os, *b, home, appdata, local).is_some_and(|p| p.exists()))
        .collect()
}

/// Chemin attendu du dossier user-data du browser. `None` si la combinaison
/// (OS, browser) n'est pas supportée ou si `appdata`/`local` manque.
fn user_data_path(
    os: HostOs,
    browser: BrowserKind,
    home: &Path,
    appdata: Option<&Path>,
    local: Option<&Path>,
) -> Option<PathBuf> {
    match (os, browser) {
        (HostOs::Macos, BrowserKind::Chrome) => {
            Some(home.join("Library/Application Support/Google/Chrome"))
        },
        (HostOs::Macos, BrowserKind::Firefox) => {
            Some(home.join("Library/Application Support/Firefox"))
        },
        (HostOs::Macos, BrowserKind::Edge) => {
            Some(home.join("Library/Application Support/Microsoft Edge"))
        },
        (HostOs::Macos, BrowserKind::Brave) => {
            Some(home.join("Library/Application Support/BraveSoftware/Brave-Browser"))
        },
        (HostOs::Macos, BrowserKind::Chromium) => {
            Some(home.join("Library/Application Support/Chromium"))
        },

        (HostOs::Linux, BrowserKind::Chrome) => Some(home.join(".config/google-chrome")),
        (HostOs::Linux, BrowserKind::Firefox) => Some(home.join(".mozilla/firefox")),
        (HostOs::Linux, BrowserKind::Edge) => Some(home.join(".config/microsoft-edge")),
        (HostOs::Linux, BrowserKind::Brave) => {
            Some(home.join(".config/BraveSoftware/Brave-Browser"))
        },
        (HostOs::Linux, BrowserKind::Chromium) => Some(home.join(".config/chromium")),

        (HostOs::Windows, BrowserKind::Chrome) => local.map(|p| p.join("Google/Chrome/User Data")),
        (HostOs::Windows, BrowserKind::Firefox) => {
            appdata.map(|p| p.join("Mozilla/Firefox/Profiles"))
        },
        (HostOs::Windows, BrowserKind::Edge) => local.map(|p| p.join("Microsoft/Edge/User Data")),
        (HostOs::Windows, BrowserKind::Brave) => {
            local.map(|p| p.join("BraveSoftware/Brave-Browser/User Data"))
        },
        (HostOs::Windows, BrowserKind::Chromium) => local.map(|p| p.join("Chromium/User Data")),
    }
}

/// Chemin d'installation du manifest pour ce browser sur l'OS courant.
pub fn manifest_install_path(browser: BrowserKind) -> Result<PathBuf> {
    let home = dirs::home_dir().context("home_dir introuvable")?;
    let appdata = dirs::config_dir();
    manifest_install_path_for(HostOs::current(), browser, &home, appdata.as_deref())
}

/// Variante testable : OS explicite. Sur Windows, `appdata` est requis et
/// l'`appdata` passé est utilisé comme base de `Sobria/bridge/com.sobria.bridge.json`.
pub fn manifest_install_path_for(
    os: HostOs,
    browser: BrowserKind,
    home: &Path,
    appdata: Option<&Path>,
) -> Result<PathBuf> {
    let path = match (os, browser) {
        // ── macOS ────────────────────────────────────────────────────────────
        (HostOs::Macos, BrowserKind::Chrome) => home.join(format!(
            "Library/Application Support/Google/Chrome/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"
        )),
        (HostOs::Macos, BrowserKind::Firefox) => home.join(format!(
            "Library/Application Support/Mozilla/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"
        )),
        (HostOs::Macos, BrowserKind::Edge) => home.join(format!(
            "Library/Application Support/Microsoft Edge/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"
        )),
        (HostOs::Macos, BrowserKind::Brave) => home.join(format!(
            "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"
        )),
        (HostOs::Macos, BrowserKind::Chromium) => home.join(format!(
            "Library/Application Support/Chromium/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"
        )),

        // ── Linux ────────────────────────────────────────────────────────────
        (HostOs::Linux, BrowserKind::Chrome) => {
            home.join(format!(".config/google-chrome/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"))
        },
        (HostOs::Linux, BrowserKind::Firefox) => {
            home.join(format!(".mozilla/native-messaging-hosts/{NATIVE_HOST_NAME}.json"))
        },
        (HostOs::Linux, BrowserKind::Edge) => home.join(format!(
            ".config/microsoft-edge/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"
        )),
        (HostOs::Linux, BrowserKind::Brave) => home.join(format!(
            ".config/BraveSoftware/Brave-Browser/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"
        )),
        (HostOs::Linux, BrowserKind::Chromium) => {
            home.join(format!(".config/chromium/NativeMessagingHosts/{NATIVE_HOST_NAME}.json"))
        },

        // ── Windows ──────────────────────────────────────────────────────────
        // Manifest commun à tous les browsers ; la clé registre HKCU par
        // browser pointe dessus.
        (HostOs::Windows, _) => {
            let appdata = appdata.context("APPDATA introuvable")?;
            appdata.join(format!("Sobria/bridge/{NATIVE_HOST_NAME}.json"))
        },
    };
    Ok(path)
}

/// Sur Windows : nom de la clé HKCU\Software\... à pointer vers le manifest.
#[must_use]
pub fn windows_registry_key(browser: BrowserKind) -> String {
    let base = match browser {
        BrowserKind::Chrome => r"HKCU\Software\Google\Chrome\NativeMessagingHosts",
        BrowserKind::Firefox => r"HKCU\Software\Mozilla\NativeMessagingHosts",
        BrowserKind::Edge => r"HKCU\Software\Microsoft\Edge\NativeMessagingHosts",
        BrowserKind::Brave => r"HKCU\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts",
        BrowserKind::Chromium => r"HKCU\Software\Chromium\NativeMessagingHosts",
    };
    format!(r"{base}\{NATIVE_HOST_NAME}")
}

/// Génère le contenu JSON du manifest pour ce browser.
///
/// - Chrome-family : `allowed_origins: ["chrome-extension://<extension_id>/"]`.
/// - Firefox : `allowed_extensions: ["<FIREFOX_EXTENSION_ID>"]` (`extension_id`
///   ignoré, l'ID du `gecko.id` est utilisé à la place).
#[must_use]
pub fn build_manifest_json(
    browser: BrowserKind,
    bridge_path: &Path,
    extension_id: &str,
) -> serde_json::Value {
    let allowed = if browser.is_firefox() {
        serde_json::json!({ "allowed_extensions": [FIREFOX_EXTENSION_ID] })
    } else {
        serde_json::json!({
            "allowed_origins": [format!("chrome-extension://{extension_id}/")]
        })
    };
    let mut base = serde_json::json!({
        "name": NATIVE_HOST_NAME,
        "description": NATIVE_HOST_DESCRIPTION,
        "path": bridge_path.to_string_lossy(),
        "type": "stdio",
    });
    if let (Some(obj), Some(extra)) = (base.as_object_mut(), allowed.as_object()) {
        for (k, v) in extra {
            obj.insert(k.clone(), v.clone());
        }
    }
    base
}

/// Écrit le manifest natif à l'emplacement OS standard. Sur Windows, écrit
/// aussi la clé registre HKCU pointant vers le fichier (`reg.exe ADD`).
///
/// Retourne le chemin du fichier JSON écrit.
pub fn install_native_manifest(
    browser: BrowserKind,
    bridge_path: &Path,
    extension_id: &str,
) -> Result<PathBuf> {
    let path = manifest_install_path(browser)?;
    install_native_manifest_at(browser, bridge_path, extension_id, &path)?;
    #[cfg(target_os = "windows")]
    {
        register_windows_native_host(browser, &path)?;
    }
    Ok(path)
}

/// Variante testable : écrit le manifest à `path` sans toucher au registre
/// Windows. Utilisée par les tests pour valider le contenu JSON sans effet
/// de bord système.
pub fn install_native_manifest_at(
    browser: BrowserKind,
    bridge_path: &Path,
    extension_id: &str,
    path: &Path,
) -> Result<()> {
    if !browser.is_firefox() && extension_id.trim().is_empty() {
        bail!(
            "extension_id requis pour {} (non-Firefox)",
            browser.display_name()
        );
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    let manifest = build_manifest_json(browser, bridge_path, extension_id);
    let pretty = serde_json::to_string_pretty(&manifest)?;
    fs::write(path, pretty.as_bytes())
        .with_context(|| format!("write manifest {}", path.display()))?;
    tracing::info!(
        browser = browser.id(),
        path = %path.display(),
        "bridge: manifest natif installé"
    );
    Ok(())
}

/// Désinstalle le manifest natif. Sur Windows, supprime aussi la clé HKCU
/// (best-effort — pas d'erreur si la clé est absente).
pub fn uninstall_native_manifest(browser: BrowserKind) -> Result<()> {
    let path = manifest_install_path(browser)?;
    uninstall_native_manifest_at(&path)?;
    #[cfg(target_os = "windows")]
    {
        let _ = unregister_windows_native_host(browser);
    }
    Ok(())
}

/// Variante testable : supprime juste le fichier (idempotent — pas d'erreur
/// s'il est absent).
pub fn uninstall_native_manifest_at(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path).with_context(|| format!("rm {}", path.display()))?;
        tracing::info!(path = %path.display(), "bridge: manifest natif supprimé");
    }
    Ok(())
}

/// Statut global : binaire bridge, manifests installés, browsers détectés.
#[must_use]
pub fn bridge_status(bridge_path: Option<PathBuf>) -> BridgeStatus {
    let detected = detect_installed_browsers();
    let installed = BrowserKind::ALL
        .into_iter()
        .filter(|b| {
            manifest_install_path(*b)
                .ok()
                .is_some_and(|p| p.exists())
        })
        .collect();
    BridgeStatus {
        bridge_path,
        installed,
        detected,
    }
}

// ─── Helpers Windows (reg.exe) ───────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn register_windows_native_host(browser: BrowserKind, manifest_path: &Path) -> Result<()> {
    let key = windows_registry_key(browser);
    let path_str = manifest_path
        .to_str()
        .context("manifest path non-UTF-8 — incompatible reg.exe")?;
    let output = std::process::Command::new("reg")
        .args(["ADD", &key, "/ve", "/d", path_str, "/f"])
        .output()
        .context("invoke reg.exe ADD")?;
    if !output.status.success() {
        bail!(
            "reg.exe ADD a échoué pour {} : {}",
            browser.display_name(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    tracing::info!(browser = browser.id(), key = %key, "bridge: clé registre HKCU écrite");
    Ok(())
}

#[cfg(target_os = "windows")]
fn unregister_windows_native_host(browser: BrowserKind) -> Result<()> {
    let key = windows_registry_key(browser);
    let _ = std::process::Command::new("reg")
        .args(["DELETE", &key, "/f"])
        .output()
        .context("invoke reg.exe DELETE")?;
    Ok(())
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn browser_kind_id_and_display_name_are_stable() {
        for b in BrowserKind::ALL {
            assert!(!b.id().is_empty());
            assert!(!b.display_name().is_empty());
        }
        assert_eq!(BrowserKind::Chrome.id(), "chrome");
        assert!(BrowserKind::Firefox.is_firefox());
        assert!(!BrowserKind::Chrome.is_firefox());
    }

    /// Normalise les séparateurs Windows (`\`) vers `/` pour les
    /// assertions cross-platform.
    fn norm(p: &Path) -> String {
        p.to_string_lossy().replace('\\', "/")
    }

    #[test]
    fn manifest_path_macos_chrome_matches_spec() {
        let home = Path::new("/Users/test");
        let p = manifest_install_path_for(HostOs::Macos, BrowserKind::Chrome, home, None).unwrap();
        assert_eq!(
            norm(&p),
            "/Users/test/Library/Application Support/Google/Chrome/NativeMessagingHosts/com.sobria.bridge.json"
        );
    }

    #[test]
    fn manifest_path_macos_firefox_matches_spec() {
        let home = Path::new("/Users/test");
        let p = manifest_install_path_for(HostOs::Macos, BrowserKind::Firefox, home, None).unwrap();
        assert_eq!(
            norm(&p),
            "/Users/test/Library/Application Support/Mozilla/NativeMessagingHosts/com.sobria.bridge.json"
        );
    }

    #[test]
    fn manifest_path_linux_firefox_uses_dot_mozilla() {
        // Firefox sur Linux : `~/.mozilla/native-messaging-hosts/...`
        // (PAS `~/.config/mozilla/...`).
        let home = Path::new("/home/test");
        let p = manifest_install_path_for(HostOs::Linux, BrowserKind::Firefox, home, None).unwrap();
        assert_eq!(
            norm(&p),
            "/home/test/.mozilla/native-messaging-hosts/com.sobria.bridge.json"
        );
    }

    #[test]
    fn manifest_path_linux_chrome_uses_dot_config() {
        let home = Path::new("/home/test");
        let p = manifest_install_path_for(HostOs::Linux, BrowserKind::Chrome, home, None).unwrap();
        assert_eq!(
            norm(&p),
            "/home/test/.config/google-chrome/NativeMessagingHosts/com.sobria.bridge.json"
        );
    }

    #[test]
    fn manifest_path_windows_uses_common_appdata_path_for_all_browsers() {
        let home = Path::new("C:/Users/test");
        let appdata = Path::new("C:/Users/test/AppData/Roaming");
        // Tous les browsers Windows pointent vers le même fichier — c'est
        // la clé registre qui les distingue.
        for b in BrowserKind::ALL {
            let p = manifest_install_path_for(HostOs::Windows, b, home, Some(appdata)).unwrap();
            assert!(
                p.ends_with("Sobria/bridge/com.sobria.bridge.json")
                    || p.ends_with("Sobria\\bridge\\com.sobria.bridge.json"),
                "unexpected path: {}",
                p.display()
            );
        }
    }

    #[test]
    fn manifest_path_windows_requires_appdata() {
        let home = Path::new("C:/Users/test");
        let err = manifest_install_path_for(HostOs::Windows, BrowserKind::Chrome, home, None)
            .unwrap_err();
        assert!(err.to_string().contains("APPDATA"));
    }

    #[test]
    fn windows_registry_key_format() {
        assert_eq!(
            windows_registry_key(BrowserKind::Chrome),
            r"HKCU\Software\Google\Chrome\NativeMessagingHosts\com.sobria.bridge"
        );
        assert_eq!(
            windows_registry_key(BrowserKind::Firefox),
            r"HKCU\Software\Mozilla\NativeMessagingHosts\com.sobria.bridge"
        );
        assert_eq!(
            windows_registry_key(BrowserKind::Brave),
            r"HKCU\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\com.sobria.bridge"
        );
    }

    #[test]
    fn build_manifest_json_chrome_has_allowed_origins() {
        let bridge = Path::new("/usr/local/bin/sobria-bridge");
        let manifest = build_manifest_json(BrowserKind::Chrome, bridge, "abcdef1234567890");
        assert_eq!(manifest["name"], "com.sobria.bridge");
        assert_eq!(manifest["type"], "stdio");
        assert_eq!(manifest["path"], "/usr/local/bin/sobria-bridge");
        assert_eq!(
            manifest["allowed_origins"][0],
            "chrome-extension://abcdef1234567890/"
        );
        // Firefox-only key absent.
        assert!(manifest.get("allowed_extensions").is_none());
    }

    #[test]
    fn build_manifest_json_firefox_has_allowed_extensions_fixed_id() {
        let bridge = Path::new("/usr/local/bin/sobria-bridge");
        // L'extension_id passé est ignoré pour Firefox — on utilise le gecko.id stable.
        let manifest = build_manifest_json(BrowserKind::Firefox, bridge, "ignored");
        assert_eq!(manifest["allowed_extensions"][0], FIREFOX_EXTENSION_ID);
        assert!(manifest.get("allowed_origins").is_none());
    }

    #[test]
    fn install_native_manifest_at_writes_valid_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("NativeMessagingHosts/com.sobria.bridge.json");
        let bridge = Path::new("/usr/local/bin/sobria-bridge");
        install_native_manifest_at(BrowserKind::Chrome, bridge, "deadbeefdeadbeef", &path).unwrap();
        assert!(path.exists());
        let written = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(parsed["name"], NATIVE_HOST_NAME);
        assert_eq!(
            parsed["allowed_origins"][0],
            "chrome-extension://deadbeefdeadbeef/"
        );
    }

    #[test]
    fn install_native_manifest_at_rejects_empty_extension_id_for_chrome() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");
        let bridge = Path::new("/bin/sobria-bridge");
        let err = install_native_manifest_at(BrowserKind::Chrome, bridge, "", &path).unwrap_err();
        assert!(err.to_string().contains("extension_id requis"));
        assert!(!path.exists(), "no file should be written on validation error");
    }

    #[test]
    fn install_native_manifest_at_allows_empty_extension_id_for_firefox() {
        // Firefox ignore extension_id (utilise gecko.id stable).
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");
        install_native_manifest_at(BrowserKind::Firefox, Path::new("/b"), "", &path).unwrap();
        assert!(path.exists());
        let parsed: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(parsed["allowed_extensions"][0], FIREFOX_EXTENSION_ID);
    }

    #[test]
    fn uninstall_native_manifest_at_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ghost.json");
        // Fichier absent : pas d'erreur.
        uninstall_native_manifest_at(&path).unwrap();
        // Fichier présent : supprimé.
        fs::write(&path, "{}").unwrap();
        uninstall_native_manifest_at(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn detect_browsers_in_returns_only_those_with_user_data_dir() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();
        // Simule un Chrome installé sur Linux.
        fs::create_dir_all(home.join(".config/google-chrome")).unwrap();
        // Et un Firefox.
        fs::create_dir_all(home.join(".mozilla/firefox")).unwrap();
        let detected = detect_browsers_in(HostOs::Linux, home, None, None);
        assert!(detected.contains(&BrowserKind::Chrome));
        assert!(detected.contains(&BrowserKind::Firefox));
        assert!(!detected.contains(&BrowserKind::Edge));
        assert!(!detected.contains(&BrowserKind::Brave));
    }

    #[test]
    fn detect_browsers_in_macos_paths() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();
        fs::create_dir_all(home.join("Library/Application Support/Google/Chrome")).unwrap();
        fs::create_dir_all(home.join("Library/Application Support/BraveSoftware/Brave-Browser"))
            .unwrap();
        let detected = detect_browsers_in(HostOs::Macos, home, None, None);
        assert!(detected.contains(&BrowserKind::Chrome));
        assert!(detected.contains(&BrowserKind::Brave));
        assert!(!detected.contains(&BrowserKind::Firefox));
    }

    #[test]
    fn detect_browsers_in_windows_uses_local_appdata() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();
        let appdata = tmp.path().join("Roaming");
        let local = tmp.path().join("Local");
        fs::create_dir_all(appdata.join("Mozilla/Firefox/Profiles")).unwrap();
        fs::create_dir_all(local.join("Google/Chrome/User Data")).unwrap();
        let detected = detect_browsers_in(HostOs::Windows, home, Some(&appdata), Some(&local));
        assert!(detected.contains(&BrowserKind::Chrome));
        assert!(detected.contains(&BrowserKind::Firefox));
        assert!(!detected.contains(&BrowserKind::Edge));
    }

    #[test]
    fn detect_browsers_in_returns_empty_when_no_browser_dir() {
        let tmp = TempDir::new().unwrap();
        let detected = detect_browsers_in(HostOs::Linux, tmp.path(), None, None);
        assert!(detected.is_empty());
    }

    #[test]
    fn round_trip_install_then_uninstall() {
        let dir = TempDir::new().unwrap();
        let path = dir
            .path()
            .join("Library/Application Support/Google/Chrome/NativeMessagingHosts")
            .join("com.sobria.bridge.json");
        install_native_manifest_at(
            BrowserKind::Chrome,
            Path::new("/usr/local/bin/sobria-bridge"),
            "abc123",
            &path,
        )
        .unwrap();
        assert!(path.exists());
        uninstall_native_manifest_at(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn host_os_current_returns_expected_value_for_target() {
        let os = HostOs::current();
        if cfg!(target_os = "macos") {
            assert_eq!(os, HostOs::Macos);
        } else if cfg!(target_os = "windows") {
            assert_eq!(os, HostOs::Windows);
        } else {
            assert_eq!(os, HostOs::Linux);
        }
    }
}
