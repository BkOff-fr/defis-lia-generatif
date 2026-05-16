# C28 — Prompt Claude Code (v0.7.0 — Mode Équipe self-hosted)

> **Mode d'emploi** : copier-coller le bloc ci-dessous dans une nouvelle session Claude Code (CLI) à la racine du repo. Le prompt démarre par `/using-superpower`.

---

```
/using-superpower

# Mission : C28 — Mode Équipe self-hosted (v0.7.0)

Tu vas implémenter de bout en bout le mode Équipe self-hosted de Sobr.ia :
un binaire Rust standalone (`sobria-team-aggregator`) qu'une entreprise
déploie chez elle pour agréger les estimations de ses N employés, avec un
dashboard admin riche (analytics, exports CSRD/PROV-O) et un dashboard
employé perso. ZÉRO cloud Sobr.ia. Self-hosted = l'entreprise contrôle
son serveur.

## Contexte à charger AVANT toute action

Lis ces fichiers dans l'ordre :

1. `CLAUDE.md` — règles projet, privacy by design (§7), anti-patterns
   (§13), DoD (§5). Notamment "pas d'envoi de prompts vers serveur
   externe" → ici ne s'applique PAS car le serveur est CHEZ l'entreprise,
   pas chez Sobr.ia.
2. `docs/adr/ADR-0013-extension-pairing-team-mode.md` — la décision
   architecturale. Tu implémentes la PHASE 2.
3. `briefs/chantiers/C28-mode-equipe-self-hosted.md` — le brief complet,
   source de vérité pour le périmètre + DoD + découpage.
4. `CHANGELOG.md` entrée [0.6.0] — ce qui a été shippé en C27 (extension
   + pairing perso + bridge_install + socket forward + Argon2id).
5. `crates/sobria-app/src/{pairing.rs, extension_store.rs, bridge_install.rs}`
   — patterns d'auth, hash Argon2id, migrations SQLite déjà éprouvés.
6. `crates/sobria-export/src/csrd.rs` — la génération PDF CSRD existante
   à réutiliser pour les exports équipe agrégés.
7. `extension/src/` — l'extension actuelle. Tu vas ajouter une section
   "Mode Équipe" en Options.
8. `web/src/lib/styles/` (ou équivalent) — design system Sobr.ia à
   réutiliser pour `web-team/`.

## Stratégie + garde-fous

- **Binaire standalone** : pas de docker obligatoire, Dockerfile en bonus.
  Cible : entreprise déploie sur poste admin, NAS, VPS interne.
- **Argon2id** partout pour les passwords (admin + user) + hash des
  enrollment codes + hash des refresh tokens. Réutilise les patterns de
  `pairing.rs`.
- **TLS** : auto-signé par défaut via `rcgen`. Import cert via
  `--cert/--key`. Documenter let's encrypt pour les setups publics.
- **JWT** : access HS256 24 h, refresh UUID v4 hashé Argon2id en table
  `tokens`, TTL 7 jours.
- **Frontend embedded** : `web-team/dist/` embarqué dans le binaire via
  `rust-embed`. Aucune dépendance HTTP runtime au browser pour servir
  l'UI.
- **Protocole REST API v1** : versionné. Le payload `EstimatePayload`
  est compatible v0.6.0 (déjà versionné `{"v": 1, ...}` côté extension).
- **Privacy** : pas de tracking Sobr.ia. Le serveur appartient à
  l'entreprise. Le `User-Agent` HTTP du client ne contient PAS
  d'identifiant Sobr.ia central.
- **Erreurs** : `thiserror` pour les erreurs publiques, `anyhow` pour le
  binaire. Pas d'`unwrap()` en prod.
- **Demande** si une ambiguïté apparaît (CORS pour cert auto-signé,
  format exact des enrollment codes, etc.).

## Plan d'exécution

### C28.1 — Bootstrap crate + TLS + serve (1.5 j)

Voir brief §C28.1. Résumé :

- `crates/sobria-team-aggregator/Cargo.toml` avec dépendances :
  `axum = "0.7"`, `tower-http = { version = "0.5", features = ["fs", "trace", "cors"] }`,
  `tokio = { workspace = true }`, `rustls = "0.23"`, `rustls-pemfile = "2"`,
  `axum-server = { version = "0.7", features = ["tls-rustls"] }`,
  `rcgen = "0.13"`, `jsonwebtoken = "9"`, `rusqlite = { workspace = true }`,
  `argon2 = { workspace = true }`, `ulid = "1"`, `clap = { workspace = true }`,
  `serde = { workspace = true }`, `serde_json = { workspace = true }`,
  `tracing = { workspace = true }`, `tracing-subscriber = { workspace = true }`,
  `rust-embed = "8"`, `mime_guess = "2"`, `thiserror = { workspace = true }`,
  `anyhow = { workspace = true }`, `chrono = { workspace = true }`,
  `uuid = { version = "1", features = ["v4"] }`,
  `rand = { workspace = true }`.
- CLI minimal `init` + `serve` avec `clap`.
- `init` :
  - Crée `team.sqlite` (data dir défaut `./team-data/`).
  - Migration v1 (admins, enrollment_codes, users, tokens, estimations).
  - Génère cert TLS auto-signé via `rcgen` (CN = hostname OS), 10 ans
    validité, écrit `cert.pem` + `key.pem`.
  - Génère JWT signing key 32 bytes random, stocké dans table `config`.
  - Crée admin initial : prompt password ou `--admin-password`, hash
    Argon2id, insert.
  - Affiche URL d'accès + URL localhost.
- `serve` :
  - Charge cert + key (regen si `--regen-cert`).
  - Lance `axum_server::bind_rustls(addr, tls_config).serve(app)`.
  - Route `/health` → `{ok: true, version: env!("CARGO_PKG_VERSION")}`.
  - Tracing requêtes avec `tower_http::trace::TraceLayer`.
- Tests `tests/serve_health.rs` : démarre serveur port aléatoire, ping
  `/health` avec `reqwest` (rustls + accept-invalid-cert pour le cert
  auto-signé), vérifie 200.

DoD C28.1 : `cargo run -p sobria-team-aggregator -- init` puis `serve`
sur localhost:8443 répond à `/health` en HTTPS.

### C28.2 — Auth (JWT + Argon2id) + API REST core (2 j)

Voir brief §C28.2. Résumé :

- `server/auth/{jwt, password, middleware}.rs`.
- Endpoints :
  - `POST /api/v1/enroll {code, password, fingerprint, display_name?}` →
    vérifie code (Argon2id verify), marque used_at, crée user, retourne
    access + refresh tokens.
  - `POST /api/v1/login {username, password, role: "admin"|"user"}` →
    vérifie password, retourne tokens.
  - `POST /api/v1/refresh {refresh_token}` → rotate (révoque ancien,
    émet nouveau).
  - `POST /api/v1/estimations` (auth user) → insère dans `estimations`,
    retourne `{id, ack}`.
  - `GET /api/v1/me/usage` (auth user) → agrège ses estimations.
- Middlewares `RequireUser` et `RequireAdmin` avec extractor Bearer.
- CLI `code create N --ttl-days 7` (génère N codes 12 chiffres OS RNG,
  affiche en clair UNE fois, stocke Argon2id hash). `code list` et
  `code revoke <id>`.
- Tests `tests/integration_api.rs` : flow complet enroll → estimation
  → me/usage via reqwest contre serveur axum réel.

DoD C28.2 : un user peut s'enrôler avec un code, POST une estimation,
voir son usage.

### C28.3 — API admin + analytics (1.5 j)

Voir brief §C28.3. Résumé :

- Endpoints admin (auth RequireAdmin) :
  - `GET /api/v1/admin/users` → liste avec totaux.
  - `POST /api/v1/admin/codes {count, ttl_days}` → crée N codes.
  - `DELETE /api/v1/admin/codes/:id` → révoque.
  - `GET /api/v1/admin/analytics?from&to&group_by={day|week|month}&dim={user|model|method}`.
- `storage/analytics.rs` :
  - `daily_totals`, `top_models`, `top_users`, `method_breakdown` en SQL
    pur avec `GROUP BY date(ts, '+0 days')` etc.
- Tests `tests/analytics.rs` : insère 100 estimations factices, vérifie
  agrégats.

DoD C28.3 : l'admin voit ses N users + leurs totaux + breakdown.

### C28.4 — Dashboard Svelte admin + user (2 j)

Voir brief §C28.4. Résumé :

- Nouveau projet `web-team/` (SvelteKit + adapter-static OU Vite + Svelte 5).
- Pages décrites dans le brief.
- Plot.dev pour les graphiques (déjà utilisé dans `web/`, design cohérent).
- Auth client : JWT en mémoire (sessionStorage acceptable pour refresh
  sur reload, mais PAS localStorage XSS). Refresh auto sur 401.
- Build `web-team/dist/` → `rust-embed` dans le binaire.
- Tests Playwright `tests/e2e/admin-dashboard.spec.ts` : login admin,
  voir users, créer code, révoquer.

DoD C28.4 : `https://localhost:8443/` charge le dashboard Svelte, login
fonctionne, graphiques s'affichent avec données.

### C28.5 — Exports CSRD + PROV-O + CSV (1 j)

Voir brief §C28.5. Résumé :

- `exports/csrd_pdf.rs` réutilise `sobria-export::csrd_report` avec un
  agrégat équipe (somme + breakdown).
- `exports/prov_o.rs` génère JSON-LD avec `prov:Activity` par estimation
  + `prov:Agent` par user (anonymisable).
- `exports/csv.rs` CSV brut.
- Endpoints `/api/v1/admin/exports/{csrd|prov-o|csv}`.
- Tests `tests/exports.rs` : génère, parse, valide.

DoD C28.5 : 3 boutons dans `/admin/exports` produisent les 3 fichiers
téléchargeables.

### C28.6 — Mode Équipe dans extension + app Tauri (1.5 j)

Voir brief §C28.6. Résumé :

- **Extension** :
  - `extension/src/lib/team-client.ts` : client REST avec JWT + refresh
    auto + accept fingerprint cert auto-signé (warning visible).
  - Section "Mode Équipe" dans Options : URL serveur, enrollment code,
    display name, password, bouton "Enrôler".
  - Toggle "Estimations vers : ⚪ Local ⚪ Mode Équipe ⚪ Les deux".
  - Au POST estimation, si Mode Équipe activé, fetch HTTPS.
- **App Tauri** :
  - `crates/sobria-app/src/team_client.rs` : équivalent Rust.
  - Section `/parametres → Mode Équipe`.
  - IPC `team_enroll`, `team_status`, `team_logout`.

DoD C28.6 : un user peut enrôler son extension + son app au serveur,
chaque prompt remonte au serveur équipe en plus (ou à la place) du
pairing perso.

### C28.7 — Doc déploiement + packaging multi-OS (1 j)

Voir brief §C28.7. Résumé :

- `docs/operations/team-aggregator.md` (~250 lignes) avec sections TPE/PME
  et DSI.
- `.github/workflows/team-aggregator-release.yml` matrix Linux + macOS +
  Windows.
- `Dockerfile` bonus.
- README racine enrichi.
- ADR-0013 statut → Phase 2 Implemented.

DoD C28.7 : binaire release ≤ 25 MB pour chaque OS, doc claire pour un
admin non-expert.

## DoD globale v0.7.0

- [ ] `cargo test --workspace` 100 % vert.
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cargo fmt --check` propre.
- [ ] `cd web && npm run check && npm run lint` propre.
- [ ] `cd web-team && npm run check && npm run lint && npm run test` propre.
- [ ] `cd extension && npm run check && npm run lint && npm run test` propre.
- [ ] Smoke E2E : init → serve → enrôler 2 users via extension + app →
      5 estimations chacun → admin voit les graphiques → export CSRD PDF
      généré + PROV-O valide.
- [ ] Binaire release ≤ 25 MB.
- [ ] CHANGELOG [0.7.0] complète.
- [ ] ADR-0013 statut Phase 2 → Implemented.
- [ ] Versions bumpées : Cargo 0.6.0→0.7.0, tauri.conf.json, web/package.json,
      extension/package.json + manifest.json, web-team/package.json (nouveau).
- [ ] Tag `v0.7.0` créé.

## Convention de commit

```
feat(team): C28.1 bootstrap sobria-team-aggregator + TLS auto-signé + serve
feat(team): C28.2 auth JWT + Argon2id + API REST core (enroll/login/refresh/estimations)
feat(team): C28.3 API admin + analytics SQL agrégés
feat(team): C28.4 dashboard Svelte admin + user (Plot + auth refresh)
feat(team): C28.5 exports CSRD PDF + PROV-O + CSV agrégés équipe
feat(ext,app): C28.6 mode Équipe dans extension + app Tauri (team-client)
docs(team): C28.7 doc déploiement + packaging multi-OS + Dockerfile bonus
chore(release): bump v0.7.0
```

Tag final :

```bash
git tag -a v0.7.0 -m "v0.7.0 — Mode Équipe self-hosted (C28)

Binaire Rust standalone 'sobria-team-aggregator' déployable par une
entreprise sur son infrastructure (poste admin, NAS, VPS interne).
Aucun cloud Sobr.ia. JWT + Argon2id + TLS auto-signé. Dashboard admin
riche (analytics gCO₂eq agrégés, top modèles, top users, breakdown
méthodologie, exports CSRD PDF + PROV-O + CSV). Dashboard employé perso.

Extension et app Tauri reçoivent une section 'Mode Équipe' permettant
de basculer entre pairing perso local et enrôlement équipe HTTPS.

ADR-0013 Phase 2 Implemented. Phase 3 (SSO, multi-device, RBAC) backlog
v0.8+.

Build assets : sobria-team-aggregator-{linux-x86_64,macos-arm64,windows-x86_64}."
```

## Garde-fous

- **JAMAIS** de cloud Sobr.ia central. Le serveur est CHEZ l'entreprise.
- **JAMAIS** d'envoi de prompts (full text) — seuls les metadata + résultats
  (tokens, gCO₂eq, etc.). Les prompts en clair restent locaux à l'extension
  (chrome.storage) ou à l'app (SQLite local).
- **JAMAIS** de tracking vers Sobr.ia depuis le binaire.
- **JAMAIS** de RBAC fin / SSO / multi-device en v0.7.0 (différé v0.8+).
- **TOUJOURS** Argon2id pour les hashes (passwords, codes, refresh tokens).
- **TOUJOURS** stocker les enrollment codes en clair UNE fois après création
  (warning UI), jamais réversiblement après.
- **TOUJOURS** `chmod 600` sur `team.sqlite` + `key.pem` (documenter).
- **TOUJOURS** versionner l'API (`/api/v1/...`) pour compatibilité future.
- **DEMANDER** si un compromis CORS / cert auto-signé / format DTO surgit
  pendant l'implémentation.

Bonne mission. Commence par C28.1.
```

---

## Notes pour Thibault

- Sprint ~9-10 jours, plus long que C26/C27. Tu peux découper en 2 ou 3
  sessions Claude Code si besoin (C28.1+C28.2 d'abord = backend core
  testable, puis C28.3+C28.4 = admin UI, puis C28.5→C28.7 = exports + clients).
- Au retour, tu reviens avec `git diff main..HEAD --stat` + `git log
  --oneline -15`.
- Smoke test E2E manuel critique :
  1. `cargo run --release -p sobria-team-aggregator -- init --data-dir /tmp/sob-team-test --admin-password admin`
  2. `cargo run --release -p sobria-team-aggregator -- serve --data-dir /tmp/sob-team-test`
  3. Browser → `https://localhost:8443/login` → admin/admin → dashboard
  4. Crée 2 codes 12 chiffres.
  5. Extension Chrome → Options → Mode Équipe → saisir code 1 + password.
  6. App Tauri sur un autre device (ou même device, autre user) →
     Paramètres → Mode Équipe → saisir code 2 + password.
  7. Envoyer 5 prompts depuis l'extension + 5 depuis l'app.
  8. Retour admin dashboard : voir 2 users, 10 estimations, graphiques OK.
  9. Bouton Export CSRD PDF → vérifier PDF généré avec les 2 users agrégés.
  10. Bouton Export PROV-O → vérifier JSON-LD valide.
- Si une étape clignote, on patche en `fix(team): ...` avant tag final.
- Pour soumission stores Chrome Web Store + AMO : tjs différée à v0.7.1
  ou v0.8.0 — le mode Équipe + UAT externe doivent être validés d'abord.
