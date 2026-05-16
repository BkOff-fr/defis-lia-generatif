# C27 — PATCH PROMPT — Compléments v0.6.0 avant tag

> **Mode d'emploi** : copier-coller le contenu ci-dessous dans une nouvelle session Claude Code (CLI) à la racine du repo. Le prompt démarre par `/using-superpower`.
>
> **État repo** : v0.6.0 est codée à 95 % (cf. CHANGELOG entrée [0.6.0] dated 2026-05-16). On NE TAG PAS encore. On patche 3 manques avant tag.

---

```
/using-superpower

# Mission : 3 patches sur v0.6.0 avant tag

Tu vas combler 3 manques par rapport au brief C27 initial. Aucun bump
version (toujours v0.6.0), aucune nouvelle feature, focus chirurgical.

## Contexte à charger AVANT toute action

Lis ces fichiers dans l'ordre :

1. `CLAUDE.md` — règles, anti-patterns, DoD.
2. `briefs/chantiers/C27-PATCH-v0.6.0.md` — brief du patch (la source de
   vérité pour ce que tu vas faire).
3. `docs/adr/ADR-0013-extension-pairing-team-mode.md` — décision archi.
4. `CHANGELOG.md` entrée [0.6.0] — ce qui est déjà shippé.
5. `crates/sobria-bridge/src/main.rs` — bridge actuel, à étendre patch 2.
6. `crates/sobria-app/src/{pairing.rs, extension_store.rs, main.rs, lib.rs}`
   — code app actuel, à enrichir patches 1 et 3.
7. `crates/sobria-bridge/scripts/install-dev.{sh,ps1}` — scripts manuels
   actuels, à remplacer par auto-install dans l'app (patch 1, mais GARDER
   les scripts en fallback pour les setups custom).
8. `web/src/routes/parametres/+page.svelte` — UI actuelle à enrichir.
9. `extension/src/options/main.ts` (ou équivalent) — UI extension qui
   appelle `bridgeClient` côté navigateur, à vérifier qu'elle remonte
   bien les erreurs du nouveau socket forward.

## Stratégie

- **Pas de bump version**. Les commits enrichissent l'entrée [0.6.0]
  existante du CHANGELOG.
- **Test-first** quand c'est faisable.
- **Pas de refactor opportuniste**. Reste chirurgical.
- **Demande** si une ambiguïté apparaît (notamment sur les emplacements
  OS du manifest native messaging — vérifie sur la doc Mozilla et Google
  si tu doutes).

## Patch 1 — Auto-install des manifests natifs par l'app Tauri

Voir `briefs/chantiers/C27-PATCH-v0.6.0.md` §"Patch 1". Résumé :

- Créer `crates/sobria-app/src/bridge_install.rs` avec :
  - `BrowserKind` enum (Chrome/Firefox/Edge/Brave/Chromium).
  - `detect_installed_browsers()` retourne ceux dont le dossier config
    existe.
  - `install_native_manifest(browser, bridge_path, extension_id)` écrit
    `com.sobria.bridge.json` au bon emplacement OS (macOS / Linux /
    Windows × Chrome / Firefox / Edge / Brave / Chromium).
  - `uninstall_native_manifest(browser)` supprime.
  - `bridge_status() -> BridgeStatusDto`.
- Ajouter 3 IPC : `bridge_status`, `install_extension_bridge(browsers)`,
  `uninstall_extension_bridge(browsers)`.
- UI Svelte `/parametres → Extension navigateur` : dialog
  `aria-modal="true"` avec checkboxes par browser + texte privacy clair.
  Au premier démarrage post-update v0.6.0 (préférence
  `bridge_install_prompt_shown`), toast non bloquant pour activer.
- Tests : `crates/sobria-app/tests/bridge_install.rs` avec mock HOME
  via tempdir.
- README extension + `crates/sobria-bridge/README.md` mis à jour.

DoD patch 1 :
- L'utilisateur clique 1 bouton dans /parametres, les manifests sont
  écrits aux 6+ emplacements possibles (selon browsers détectés),
  l'extension se connecte au bridge sans script manuel.

## Patch 2 — Socket forward bridge ↔ app desktop

Voir brief §"Patch 2". Résumé :

- `crates/sobria-bridge/src/main.rs` : pour chaque message, tenter
  d'abord d'écrire au socket local (Unix : `/tmp/sobria-bridge.sock` ou
  `$XDG_RUNTIME_DIR/sobria-bridge.sock` ; Windows :
  `\\.\pipe\sobria-bridge`). Si OK, attendre la réponse (timeout 2 s)
  et la forwarder à l'extension. Si KO, fallback spool fichier
  (comportement actuel).
- `crates/sobria-app/src/main.rs` : au setup Tauri, spawn une task
  Tokio qui écoute le socket Unix (Listener) ou named pipe Windows
  (ServerOptions::new().create(...)). Route les requêtes vers
  `pairing::verify_pairing_code`, `pairing::revoke_pairing`,
  `extension_store::insert_event`, retourne la réponse.
- Garder le `drain_extension_spool` polling 5 s comme fallback offline
  (extension qui écrit pendant que l'app est fermée).
- Tests `socket_forward.rs` côté bridge + `socket_server.rs` côté app.
  Sur Windows, named pipe → marquer le test `#[cfg(windows)]`.
- Doc `docs/extension/architecture.md` : diagramme mis à jour.

DoD patch 2 :
- Quand l'app tourne, saisir un code 6 chiffres dans la popup extension
  → réponse en ≤ 2 s avec le secret. Plus de "App non joignable".
- Quand l'app est fermée, l'extension fonctionne toujours (estimations
  → spool fichier → ingérées au prochain démarrage app).

## Patch 3 — Argon2id pour le hash du secret pairing

Voir brief §"Patch 3". Résumé :

- `crates/sobria-app/Cargo.toml` : `argon2 = "0.5"` (vérifie qu'il n'y
  a pas déjà une autre version dans le workspace via `cargo tree`).
- `crates/sobria-app/src/pairing.rs` : remplacer SHA-256+sel par
  Argon2id avec PHC string. Plus besoin de `salt_hex` séparé.
- `crates/sobria-app/src/extension_store.rs` : migration SQLite v3 :
  `ALTER TABLE device_pairings DROP COLUMN salt_hex;` (ou
  DROP/RECREATE si SQLite < 3.35). Au boot, si des pairings v2 (hash
  SHA-256) existent, UPDATE `revoked_at = now()` + warning log.
- Tests `pairing.rs` adaptés.
- CHANGELOG [0.6.0] : ajouter ligne "Hash secret pairing : SHA-256 →
  Argon2id (PHC string, params standards)".

DoD patch 3 :
- `cargo test -p sobria-app` 100 % vert.
- `cargo audit` reste propre (Argon2 0.5 OK).

## Definition of Done globale

- [ ] `cargo test --workspace` 100 % vert (incluant bridge_install +
      socket_forward + pairing Argon2).
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cargo fmt --check` propre.
- [ ] `cd web && npm run check && npm run lint` propre.
- [ ] `cd extension && npm run check && npm run lint && npm run test`
      propre.
- [ ] Bundle Chrome ≤ 500 KB, Firefox ≤ 500 KB inchangés.
- [ ] CHANGELOG [0.6.0] enrichie de 3 lignes (1 par patch).
- [ ] ADR-0013 reste "Phase 1 Implemented".
- [ ] Pas de bump version (toujours v0.6.0).
- [ ] 3 commits Conventional Commits :
  - `feat(app): C27 patch auto-install bridge manifests natifs`
  - `feat(bridge,app): C27 patch socket forward temps réel`
  - `refactor(app): C27 patch Argon2id pour hash secret pairing`
- [ ] Tag `v0.6.0` créé localement après le 3e commit.

## Commande de tag final

```bash
git tag -a v0.6.0 -m "v0.6.0 — Extension navigateur Sobr.ia + pairing perso (C27)

Extension WebExtension MV3 mesurant l'empreinte des prompts en direct
sur ChatGPT, Claude et Le Chat. Moteur AFNOR + EcoLogits porté en JS
(parité < 2 %). Pairing local par code 6 chiffres avec l'app Sobr.ia
desktop via native messaging — auto-install des manifests par l'app,
socket forward temps réel, Argon2id pour le hash du secret.

100 % local, aucun cloud, aucun compte. ADR-0013 Phase 1 Implemented.

Build assets : sobria-extension-chrome-v0.6.0.zip +
sobria-extension-firefox-v0.6.0.xpi + binaires sobria-bridge."
```

## Garde-fous

- Pas de nouvelle dépendance hors `argon2 = "0.5"`.
- Pas de refactor opportuniste hors les 3 patches.
- Demander confirmation si tu hésites sur un emplacement OS (Linux Brave
  par ex est moins documenté — au pire on l'exclut de v0.6.0 et on note
  TODO).
- Le mode Équipe (C28) reste pour v0.7.0 — ne pas commencer.

Bonne mission. Commence par Patch 3 (Argon2id, le plus simple, 30 LoC),
puis Patch 1 (le plus visible UX), puis Patch 2 (le plus délicat
techniquement avec le cross-platform).
```

---

## Notes pour Thibault

- Tu lances Claude Code avec ce prompt. ~1 jour de boulot estimé.
- Au retour, tu reviens me voir avec `git diff main..HEAD --stat` (sans
  les nouveaux fichiers générés `target/` et `node_modules/`) et `git
  log --oneline -10`.
- Smoke test final avant push :
  - Lancer l'app Tauri fraîche (supprimer `~/.sobria/` pour test propre).
  - Vérifier que le dialog "Activer la synchro" apparaît au boot.
  - Cliquer "Activer" → vérifier que les manifests sont écrits
    (`ls ~/Library/Application\ Support/*/NativeMessagingHosts/` sur macOS).
  - Installer l'extension dev mode Chrome.
  - Saisir le code 6 chiffres → vérifier que le pairing est instantané
    (≤ 2 s, pas 5 s = pas de fallback spool).
  - Envoyer un prompt sur ChatGPT → vérifier badge + entrée Journal app.
