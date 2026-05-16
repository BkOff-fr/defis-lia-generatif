# C27 — Patch v0.6.0 — Compléments avant tag

> **Contexte** : v0.6.0 est codée à 95 % (cf. CHANGELOG entrée [0.6.0]) mais 3 manques par rapport au brief C27 initial. On ne tag PAS encore — on patche d'abord.
>
> **Périmètre patch** : 3 chantiers focalisés, ~1 jour total. Ce patch est encore v0.6.0 (les commits viennent enrichir l'entrée existante du CHANGELOG, pas en créer une nouvelle).

---

## Patch 1 — Auto-install des manifests natifs par l'app Tauri

**Manque actuel** : pour activer le pont, l'utilisateur doit lancer manuellement `crates/sobria-bridge/scripts/install-dev.{sh,ps1}`. C'est inacceptable pour un utilisateur non-tech.

**Livrable** :

- `crates/sobria-app/src/bridge_install.rs` (nouveau module) :
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum BrowserKind { Chrome, Firefox, Edge, Brave, Chromium }

  pub fn detect_installed_browsers() -> Vec<BrowserKind>;
  pub fn install_native_manifest(browser: BrowserKind, bridge_path: &Path, extension_id: &str) -> Result<PathBuf>;
  pub fn uninstall_native_manifest(browser: BrowserKind) -> Result<()>;
  pub fn manifest_install_path(browser: BrowserKind) -> Result<PathBuf>;
  pub fn bridge_status() -> BridgeStatus;
  ```
  Emplacements OS attendus (à coder, pas à googler — c'est le contrat WebExtensions standard) :
  - **macOS Chrome** : `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/com.sobria.bridge.json`
  - **macOS Firefox** : `~/Library/Application Support/Mozilla/NativeMessagingHosts/com.sobria.bridge.json`
  - **macOS Edge** : `~/Library/Application Support/Microsoft Edge/NativeMessagingHosts/...`
  - **Linux Chrome** : `~/.config/google-chrome/NativeMessagingHosts/...`
  - **Linux Firefox** : `~/.mozilla/native-messaging-hosts/com.sobria.bridge.json`
  - **Windows Chrome** : clé registre `HKEY_CURRENT_USER\Software\Google\Chrome\NativeMessagingHosts\com.sobria.bridge` (valeur par défaut = chemin du fichier manifest, qu'on écrit dans `%APPDATA%\Sobria\bridge\com.sobria.bridge.json`).
  - **Windows Firefox** : clé registre `HKEY_CURRENT_USER\Software\Mozilla\NativeMessagingHosts\com.sobria.bridge` (idem).
  - Pour Edge/Brave/Chromium : mêmes patterns, dossiers ajustés (consulter doc Mozilla et Google si doute).

  Detection : tester l'existence des dossiers config (Unix) ou des sous-clés `HKCU\Software\<Vendor>\<Browser>` (Windows).

  Le manifest JSON écrit contient :
  ```json
  {
    "name": "com.sobria.bridge",
    "description": "Sobria native messaging bridge",
    "path": "<bridge_path>",
    "type": "stdio",
    "allowed_origins": ["chrome-extension://<extension_id>/"]
  }
  ```
  Pour Firefox : remplacer `allowed_origins` par `allowed_extensions: ["sobria@sobr.ia"]`.

- `crates/sobria-app/src/main.rs` : ajouter 3 commandes IPC :
  - `bridge_status() -> BridgeStatusDto` (browsers détectés, browsers déjà pairés, chemin bridge actuel).
  - `install_extension_bridge(browsers: Vec<BrowserKind>) -> Result<Vec<PathBuf>>` (écrit les manifests pour les browsers demandés).
  - `uninstall_extension_bridge(browsers: Vec<BrowserKind>) -> Result<()>`.

- Côté frontend `web/src/routes/parametres/+page.svelte` :
  - Section "Extension navigateur" enrichie :
    - Statut bridge + liste navigateurs détectés (icônes).
    - Bouton "Activer la synchronisation" → ouvre un dialog `aria-modal="true"` avec checkboxes par browser détecté + texte privacy clair : "Sobr.ia va écrire un petit fichier dans la config de chaque navigateur sélectionné. Aucune donnée n'est envoyée ailleurs."
    - Après install : "Synchro activée pour Chrome, Firefox" + bouton "Désactiver".

- Au premier démarrage post-update v0.6.0 (détecter avec préférence `bridge_install_prompt_shown` dans `app_preferences`), afficher un toast non bloquant : "Connecter l'extension navigateur Sobr.ia ? → Activer". Si l'utilisateur clique "Plus tard" ou "Ne plus demander", stocker la préférence.

- Tests `crates/sobria-app/tests/bridge_install.rs` :
  - Mock du HOME / APPDATA via tempdir.
  - Install Chrome macOS → vérifier le fichier écrit + JSON valide.
  - Uninstall → vérifier suppression.
  - Detection : créer un faux dossier config → doit être listé.

- README extension + `crates/sobria-bridge/README.md` mis à jour : "L'auto-install par l'app Sobr.ia est désormais la méthode recommandée. Les scripts manuels restent dans `scripts/` pour les setups custom."

---

## Patch 2 — Socket forward bridge ↔ app desktop

**Manque actuel** : le bridge écrit dans le spool fichier, mais `Pair{ code }` et `Revoke{ secret }` retournent erreur si l'app n'est pas joignable. L'extension affiche "App non joignable" au pairing initial même si l'app tourne.

**Livrable** :

- `crates/sobria-bridge/src/main.rs` étendu :
  - Avant d'écrire dans le spool, tenter de se connecter à un socket local.
  - **Unix** (macOS/Linux) : `UnixStream::connect("/tmp/sobria-bridge.sock")` (ou `$XDG_RUNTIME_DIR/sobria-bridge.sock` si défini).
  - **Windows** : `tokio::net::windows::named_pipe::ClientOptions::new().open(r"\\.\pipe\sobria-bridge")`.
  - Protocole simple : envoyer la requête (Pair / Estimate / Revoke / Ping) en JSON length-prefixed, lire la réponse. Timeout 2 s.
  - Si socket KO → fallback spool fichier (comportement actuel).
  - Si socket OK → la réponse de l'app est forwardée à l'extension (les codes pair/revoke fonctionnent en temps réel).

- `crates/sobria-app/src/main.rs` :
  - Au démarrage Tauri, lancer un task Tokio qui écoute le socket :
    - **Unix** : `UnixListener::bind(...)`.
    - **Windows** : `ServerOptions::new().create(...)` + boucle.
  - Pour chaque connexion : lire la requête, router vers `pairing::verify_pairing_code`, `pairing::revoke_pairing`, `extension_store::insert_event`, etc. Renvoyer la réponse.
  - Garder le `drain_extension_spool` polling 5 s comme fallback offline (l'extension a pu écrire pendant que l'app était fermée).

- Tests :
  - `crates/sobria-bridge/tests/socket_forward.rs` (Unix only acceptable, Windows en CI matrix) : démarre un serveur factice, bridge se connecte, roundtrip OK.
  - `crates/sobria-app/tests/socket_server.rs` : Tokio mock client, vérifie router.

- Documentation `docs/extension/architecture.md` : diagramme flux mis à jour avec le socket en chemin principal, spool en fallback.

---

## Patch 3 — Argon2id pour le hash du secret pairing

**Manque actuel** : `extension_store.rs` utilise SHA-256 + sel 16 octets. Le brief demandait Argon2id (params standards).

**Livrable** :

- `crates/sobria-app/Cargo.toml` : ajouter dep `argon2 = "0.5"` (la même version que celle déjà utilisée pour les autres hashs dans le workspace, s'il y en a — check `cargo tree`).
- `crates/sobria-app/src/pairing.rs` :
  - Remplacer `PairingSecret::hash(secret, salt)` par `argon2::Argon2::default().hash_password(secret_bytes, &salt_string)`.
  - Stocker le `password_hash` complet (PHC string format) en `device_pairings.secret_hash`. Plus besoin de stocker `salt_hex` séparément (le PHC string inclut le sel).
  - `verify_secret(secret, stored_phc)` utilise `argon2::Argon2::default().verify_password(secret_bytes, &PasswordHash::new(&stored_phc)?)`.
- Migration SQLite v3 :
  - `ALTER TABLE device_pairings DROP COLUMN salt_hex;` (ou DROP/RECREATE si SQLite < 3.35).
  - Stratégie : si des pairings existent en SHA-256+salt format, ils sont **invalidés** (révoqués automatiquement avec note "migration v3 → re-pairing requis"). Au boot, l'app détecte les anciens hashes et fait un UPDATE `revoked_at`. L'utilisateur devra re-saisir le code dans l'extension. Justifié : v0.6.0 vient de sortir, peu de pairings en prod.
- Tests `pairing.rs` adaptés : `verify_secret` retourne Ok(()) pour secret correct, Err pour mauvais ou hash invalide.
- CHANGELOG entrée [0.6.0] : ajouter ligne "Hash secret pairing : SHA-256 → Argon2id (PHC string)".

---

## Definition of Done globale (patch v0.6.0)

- [ ] `cargo test --workspace` → 100 % vert (les nouveaux tests bridge_install + socket_forward + pairing Argon2 inclus).
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cargo fmt --check` propre.
- [ ] `cd web && npm run check && npm run lint` propre.
- [ ] `cd extension && npm run check && npm run lint && npm run test` propre.
- [ ] L'app Tauri démarre + dialog "Activer la synchro" apparaît au premier lancement post-update (toggle préférence `bridge_install_prompt_shown`).
- [ ] Pairing en temps réel via socket : extension envoie un code 6 chiffres → app répond avec secret en ≤ 2 s. Plus de "App non joignable" si l'app tourne.
- [ ] Bundle Chrome ≤ 500 KB, Firefox ≤ 500 KB inchangés.
- [ ] CHANGELOG entrée [0.6.0] enrichie de 3 lignes (patches 1/2/3).
- [ ] ADR-0013 reste "Phase 1 Implemented".
- [ ] Pas de bump version (toujours v0.6.0).
- [ ] Commits Conventional Commits : `feat(app): C27 patch auto-install bridge`, `feat(bridge): C27 patch socket forward`, `refactor(app): C27 patch Argon2id pour secret pairing`.
- [ ] Tag `v0.6.0` créé après le 3e commit (donc on n'a PAS encore tagué — on tag à la fin).

---

## Anti-périmètre du patch

- Pas de nouvelles fonctionnalités hors les 3 patches.
- Pas de refactor "tant qu'on y est".
- Le mode Équipe (C28) reste pour v0.7.0.
- La signature codesign/signtool des binaires bridge reste différée à v0.6.1.
