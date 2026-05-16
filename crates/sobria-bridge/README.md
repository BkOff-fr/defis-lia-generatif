# sobria-bridge

Native Messaging bridge entre l'extension navigateur Sobr.ia et l'app
Sobr.ia desktop (C27.5).

## Rôle

Binaire éphémère local invoqué par le navigateur via Native Messaging.
Lit `stdin` (uint32 LE + JSON UTF-8), traite la requête, écrit la
réponse au même format sur `stdout`. Aucun port réseau, aucun service
permanent — sécurité OS de Native Messaging WebExtensions.

## Statut v0.6.0 (POC C27.5.a)

- ✅ Protocole length-prefixed JSON conforme WebExtensions Native Messaging
- ✅ `Ping` → `{ pong: true }`
- ✅ `Estimate{ secret, payload }` → spool fichier append-only
  (`~/.sobria/spool/incoming.jsonl`, rotation 10 MB)
- ⏳ `Pair{ code }` / `Revoke{ secret }` → erreur tant que l'app desktop
  n'est pas joignable (cf. C27.5.b/c — module `sobria-app::pairing` à
  livrer)

## Installation manuelle (en attendant C27.5.b auto-install par l'app)

### Étape 1 — build du binaire

```bash
cargo build -p sobria-bridge --release
# → target/release/sobria-bridge (Linux/macOS)
# → target\release\sobria-bridge.exe (Windows)
```

### Étape 2 — rendre le manifest natif accessible au navigateur

Copier `manifest/com.sobria.bridge.json.tmpl` et substituer :

- `{{BRIDGE_PATH}}` : chemin absolu vers le binaire `sobria-bridge`
- `{{ALLOWED_ORIGINS}}` : `"chrome-extension://<ID>/"` (ID de l'extension
  visible dans `chrome://extensions/`). Plusieurs origines séparées par
  virgule pour autoriser plusieurs navigateurs.

Exemple de manifest final pour Chrome :

```json
{
  "name": "com.sobria.bridge",
  "description": "Sobr.ia native messaging bridge",
  "path": "/Users/<you>/sobria/target/release/sobria-bridge",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://abcdefghijklmnopqrstuvwxyz123456/"
  ]
}
```

### Étape 3 — déployer le manifest aux emplacements OS

**macOS — Chrome / Chromium / Brave / Edge** :

```bash
mkdir -p ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/
cp com.sobria.bridge.json \
   ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/com.sobria.bridge.json
```

(Adapter `Google/Chrome` en `Chromium`, `BraveSoftware/Brave-Browser`, `Microsoft Edge` selon le navigateur.)

**macOS — Firefox** :

```bash
mkdir -p ~/Library/Application\ Support/Mozilla/NativeMessagingHosts/
cp com.sobria.bridge.json \
   ~/Library/Application\ Support/Mozilla/NativeMessagingHosts/com.sobria.bridge.json
```

**Linux — Chrome** :

```bash
mkdir -p ~/.config/google-chrome/NativeMessagingHosts/
cp com.sobria.bridge.json \
   ~/.config/google-chrome/NativeMessagingHosts/com.sobria.bridge.json
```

**Linux — Firefox** :

```bash
mkdir -p ~/.mozilla/native-messaging-hosts/
cp com.sobria.bridge.json \
   ~/.mozilla/native-messaging-hosts/com.sobria.bridge.json
```

**Windows — Chrome** (PowerShell, en tant qu'utilisateur) :

```powershell
New-Item -Path "HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.sobria.bridge" `
  -Force | Out-Null
Set-Item -Path "HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.sobria.bridge" `
  -Value "C:\absolute\path\to\com.sobria.bridge.json"
```

**Windows — Firefox** :

```powershell
New-Item -Path "HKCU:\Software\Mozilla\NativeMessagingHosts\com.sobria.bridge" `
  -Force | Out-Null
Set-Item -Path "HKCU:\Software\Mozilla\NativeMessagingHosts\com.sobria.bridge" `
  -Value "C:\absolute\path\to\com.sobria.bridge.json"
```

### Étape 4 — vérifier

Ouvrir le devtools du service worker de l'extension (chrome://extensions/ →
détails → inspecter le service worker), puis :

```js
const port = chrome.runtime.connectNative('com.sobria.bridge');
port.onMessage.addListener((m) => console.log('bridge response:', m));
port.postMessage({ type: 'ping', reqId: 'manual-1' });
// → attend `{ reqId: 'manual-1', ok: true, pong: true }`
```

Si le port est immédiatement déconnecté avec une erreur, vérifier :
- chemin absolu correct dans le manifest
- binaire exécutable (`chmod +x` sur Unix)
- ID de l'extension correct dans `allowed_origins`

## Spool fichier

Si `Pair` n'a pas été établi côté app desktop, les `Estimate` sont quand
même acceptés par le bridge mais marqués avec le `secret_hash` (short
FNV) au lieu d'être validés. L'app desktop draine
`~/.sobria/spool/incoming.jsonl` au démarrage (C27.5.d).

Rotation : si le fichier dépasse 10 MB, il est renommé en
`incoming.jsonl.bak` et un nouveau spool repart à zéro. Évite
l'accumulation sans bornes si l'app desktop reste éteinte longtemps.

## Tests

```bash
cargo test -p sobria-bridge
```

## Prochaines étapes (C27.5.b/c/d)

- `crates/sobria-app::pairing` : module Rust avec table SQLite
  `device_pairings`, génération code 6 chiffres TTL 5 min, vérification
  Argon2id du secret.
- `crates/sobria-app::bridge_install` : auto-install du manifest natif
  par l'app desktop (`install_extension_bridge` IPC).
- `crates/sobria-app::extension_ingest` : `drain_extension_spool` IPC +
  timer Tokio 5 s, insertion dans table `extension_events`.
