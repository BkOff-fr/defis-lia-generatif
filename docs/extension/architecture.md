# Extension Sobr.ia — Architecture (patch C27 v0.6.0)

Cette page documente le flux de données entre l'extension navigateur, le
binaire `sobria-bridge` (Native Messaging) et l'app Sobr.ia desktop, après
le **patch C27 v0.6.0** qui introduit :

1. **Auto-install** des manifests natifs par l'app (`/parametres → Extension
   navigateur`), remplaçant les scripts shell/PowerShell de v0.6.0 d'origine.
2. **Socket forward** temps réel entre le bridge et l'app desktop (Unix
   socket / Windows named pipe), avec fallback **spool fichier** pour les
   écritures offline (app fermée).
3. **Argon2id** (PHC string) pour le hash du secret pairing — remplace
   SHA-256 + sel séparé de v0.6.0 d'origine.

Voir ADR-0013 pour la décision archi globale (extension + pairing perso
v0.6.0 / mode Équipe v0.7+).

---

## Vue d'ensemble — chemins de données

```
┌─────────────────────────┐     Native Messaging      ┌────────────────────┐
│   Extension Chrome /    │  stdin/stdout JSON L-P    │   sobria-bridge    │
│   Firefox / Edge / ...  │ ◄────────────────────────►│   (binaire local)  │
│                         │                            └─────────┬──────────┘
│  - content scripts      │                                      │
│  - background SW        │                                      │
│  - popup + options      │                                      │
└─────────────────────────┘                                      │
                                                                 │
                              ① socket forward (real-time)       │
                              ────────────────────────────────►  │
                              ┌─────────────────────────────────┘
                              │
                              │  Unix : $XDG_RUNTIME_DIR/sobria-bridge.sock
                              │  Windows : \\.\pipe\sobria-bridge
                              ▼
                    ┌─────────────────────┐
                    │  Sobr.ia desktop    │
                    │  (Tauri 2 + Rust)   │
                    │                     │
                    │  ┌───────────────┐  │
                    │  │ bridge_server │  │  ← tokio task, listen
                    │  │ (socket/pipe) │  │
                    │  └───────┬───────┘  │
                    │          │ dispatch │
                    │          ▼          │
                    │  ┌───────────────┐  │
                    │  │ AppState      │  │
                    │  │ (Mutex<>)     │  │
                    │  │ - pairing     │  │
                    │  │ - extension   │  │
                    │  │   _store      │  │
                    │  └───────────────┘  │
                    └─────────────────────┘
                              ▲
                              │  ② spool fallback (offline)
                              │  ~/.sobria/spool/incoming.jsonl
                              │  drainé toutes les 5 s par
                              │  drain_extension_spool
                              │
                    sobria-bridge → spool (si socket KO)
```

**Légende** :

- **①** Socket forward — chemin **principal** quand l'app desktop tourne.
  Le bridge tente d'abord d'écrire sur le socket Unix ou le named pipe
  Windows ; en cas de succès (timeout 2 s), la réponse de l'app est
  forwardée directement à l'extension. Permet aux flows `Pair` / `Revoke`
  d'être interactifs (réponse en ≤ 2 s).

- **②** Spool fichier — chemin **fallback** quand l'app est fermée. Le
  bridge écrit l'estimation dans `~/.sobria/spool/incoming.jsonl`
  (append-only, rotation 10 MB). L'app, au démarrage et toutes les 5 s
  ensuite, draine ce spool via `drain_extension_spool` (cf.
  `crates/sobria-app/src/extension_store.rs::drain_spool`).

L'extension n'a **jamais** connaissance du chemin choisi (socket vs spool)
— c'est transparent côté bridge. Le retour `ok: true` peut venir soit de
l'app (socket), soit du bridge lui-même après écriture du spool.

---

## Format wire

Length-prefixed JSON (uint32 LE + UTF-8) — identique sur les 3 segments :

1. **Extension → bridge** : Native Messaging WebExtensions standard.
2. **Bridge → app (socket/pipe)** : même format, `sobria-bridge::BridgeRequest`.
3. **App → bridge (socket/pipe)** : `sobria-bridge::BridgeResponse`.

```rust
// crates/sobria-bridge/src/lib.rs

pub enum BridgeRequest {
    Ping     { req_id: String },
    Pair     { req_id: String, code: String },
    Estimate { req_id: String, secret: String, payload: Value },
    Revoke   { req_id: String, secret: String },
}

pub struct BridgeResponse {
    pub req_id: String,
    pub ok: bool,
    pub error: Option<String>,
    pub pong: Option<bool>,
    pub secret: Option<String>,
    pub pairing_id: Option<String>,
    pub fingerprint: Option<String>,
}
```

---

## Auto-install des manifests natifs

Au premier démarrage post-update v0.6.0, l'app affiche un toast non bloquant
dans `/parametres → Extension navigateur`. Clic « Activer la synchronisation »
ouvre un dialog `aria-modal="true"` avec :

- Statut bridge (binaire détecté, navigateurs détectés, manifests installés).
- Checkbox par navigateur détecté.
- Champ `extension_id` Chrome (visible dans `chrome://extensions/`).
- Texte privacy clair : « Aucune donnée n'est envoyée ailleurs. »

Le clic « Installer » appelle l'IPC `install_extension_bridge(browsers,
extension_id)` qui :

1. Écrit `com.sobria.bridge.json` à l'emplacement OS standard
   (`Library/Application Support/...`, `~/.config/...`, `%APPDATA%\Sobria\bridge\`).
2. Sur Windows, inscrit en plus une clé `HKCU\Software\<Vendor>\<Browser>\
   NativeMessagingHosts\com.sobria.bridge` pointant vers le JSON via `reg.exe ADD`.

Cf. `crates/sobria-app/src/bridge_install.rs` (5 navigateurs × 3 OS, mock
HOME via tempdir pour les tests).

---

## Sécurité

- **Pas de port réseau** — tout local OS (socket Unix ou named pipe Windows,
  pas d'écoute TCP/UDP).
- **Argon2id (PHC string)** pour le hash du secret pairing 32-octets
  (paramètres par défaut crate `argon2 = 0.5`). Sel embarqué dans le PHC,
  pas de colonne séparée.
- **Constant-time** pour la comparaison du code 6 chiffres (TTL 5 min,
  single-use).
- **Consentement explicite** pour l'auto-install (dialog `aria-modal`).
- **Révocation** côté app (`/parametres` bouton X par pairing) — la
  ligne `device_pairings.revoked_at` est marquée, le secret n'est plus
  accepté.

Voir [ADR-0013](../adr/ADR-0013-extension-pairing-team-mode.md) pour le
contexte complet.

---

## Tests

| Composant            | Fichier                                                | Couverture |
|----------------------|--------------------------------------------------------|------------|
| Auto-install (Rust)  | `crates/sobria-app/src/bridge_install.rs` + `tests/bridge_manifests.rs` | 5 browsers × 3 OS via `HostOs` enum |
| Argon2id pairing     | `crates/sobria-app/src/pairing.rs`                     | round-trip + migration v2→v3 |
| Bridge socket client | `crates/sobria-bridge/tests/socket_forward.rs`         | Unix socket + Windows pipe |
| App socket server    | `crates/sobria-app/src/bridge_server.rs` + `tests/bridge_server.rs` | dispatch + roundtrip Unix |
| Protocol wire format | `crates/sobria-bridge/tests/protocol.rs`               | length-prefix + JSON tag |
