# Chantier C28 — Mode Équipe self-hosted (`sobria-team-aggregator`)

> **Version cible** : v0.7.0
> **Sprint** : S14 (post-v0.6.0 ship)
> **Approche** : binaire Rust standalone unique (~15 MB), SQLite embarqué, Svelte servi statiquement, TLS auto-signé par défaut. Pas de Docker obligatoire. Pas de cloud Sobr.ia.
> **Pré-requis** : v0.6.0 (extension + pairing perso) shippée, **ADR-0013 Phase 1 Implemented**.
> **Cible déploiement** : poste admin / NAS / VPS interne d'une PME. Docker fourni en bonus.

---

## 0. Pourquoi maintenant ?

Phase 1 (v0.6.0) couvre le particulier complètement (pairing perso 6 chiffres). Phase 2 (v0.7.0, ce chantier) ouvre le cas entreprise : un admin déploie le binaire chez lui, distribue des **enrollment codes** à ses N employés, chaque employé saisit son code dans son extension + son app, et le serveur agrège — toujours **sans cloud Sobr.ia**.

Côté défi data.gouv.fr, c'est le différenciateur "CSRD-ready" : reporting d'équipe agrégé, conforme méthodologies AFNOR + EcoLogits, exports PDF + PROV-O signés, dataset Gebru — le tout hébergé par l'entreprise, données souveraines.

---

## 1. Périmètre v0.7.0

### En périmètre

- Crate `sobria-team-aggregator` (binaire Rust standalone ~15 MB).
- CLI : `init`, `serve`, `code {create|list|revoke}`, `user list`, `admin {create|reset-password}`.
- Serveur HTTPS avec TLS auto-signé (regen sur `init` ou import certif via `--cert/--key`).
- SQLite `team.sqlite` (admins, enrollment_codes, users, tokens, estimations).
- API REST JSON `/api/v1/*` (auth Bearer JWT, refresh tokens 7 jours, access tokens 24 h).
- Dashboard Svelte servi statiquement par le binaire (compilé + embedded via `rust-embed`).
- **Dashboard admin riche** :
  - Login admin.
  - Gestion enrollment codes (créer N codes, lister, révoquer).
  - Liste des employés enrollés (fingerprint, dernier vu, statut).
  - Analytics agrégés (graphiques Plot/D3) : évolution journalière/mensuelle, top modèles, top utilisateurs, breakdown méthodologie.
  - Exports CSRD PDF agrégé équipe (réutilise `sobria-export` côté Rust).
  - Exports PROV-O JSON-LD sidecar.
  - Export CSV brut.
  - Alertes seuils (seuil mensuel en gCO₂eq par utilisateur ou équipe).
- **Dashboard employé** :
  - Login via enrollment code + mot de passe choisi à l'enrollment.
  - Mon usage perso (graphiques mensuels, total, comparaison vs moyenne équipe anonymisée).
  - Mon export PROV-O personnel.
- Modifications **extension navigateur** : section "Mode Équipe" dans Options (URL serveur + enrollment code + bascule "Local / Mode Équipe").
- Modifications **app Tauri** : section "Mode Équipe" dans `/parametres` (équivalent).
- Bascule "Local ↔ Mode Équipe" propre : un utilisateur peut coexister les deux (perso local + équipe pro), mais le toggle décide où chaque estimation est envoyée.
- Documentation déploiement (`docs/operations/team-aggregator.md`, ~250 lignes).
- Build CI multi-OS : Linux x86_64, macOS arm64, Windows x86_64. Dockerfile bonus.

### Hors périmètre v0.7.0

- SSO entreprise (SAML, OIDC) → ADR-0013 Phase 3 / v0.8+.
- Multi-device pour un même utilisateur → v0.8+.
- RBAC fin (rôles managers intermédiaires) → v0.8+.
- Synchronisation cluster multi-serveur → backlog v1.x.
- Cloud hébergé Sobr.ia → **jamais** (CLAUDE.md §7).

---

## 2. Architecture

### Crate `sobria-team-aggregator`

```
crates/sobria-team-aggregator/
├── Cargo.toml                    # axum 0.7, rustls, jsonwebtoken, rusqlite, argon2, rust-embed, clap
├── src/
│   ├── main.rs                   # entry point CLI clap
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── init.rs               # init team.sqlite + TLS auto-cert + admin initial
│   │   ├── serve.rs              # lance le serveur axum
│   │   ├── code.rs               # create/list/revoke enrollment codes
│   │   ├── user.rs               # list users
│   │   └── admin.rs              # create / reset-password admin
│   ├── server/
│   │   ├── mod.rs                # router axum
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── enroll.rs         # POST /api/v1/enroll
│   │   │   ├── estimations.rs    # POST /api/v1/estimations
│   │   │   ├── refresh.rs        # POST /api/v1/refresh
│   │   │   ├── me.rs             # GET /api/v1/me/usage, exports
│   │   │   └── admin.rs          # GET/POST/DELETE /api/v1/admin/*
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   ├── jwt.rs            # access + refresh tokens
│   │   │   ├── password.rs       # Argon2id pour admin password
│   │   │   └── middleware.rs     # extractor Bearer + role check
│   │   ├── tls.rs                # rustls config (auto-signé / import)
│   │   └── embedded_web.rs       # rust-embed du build Svelte
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── schema.rs             # migrations SQLite v1
│   │   ├── admins.rs
│   │   ├── enrollment_codes.rs
│   │   ├── users.rs
│   │   ├── tokens.rs
│   │   ├── estimations.rs
│   │   └── analytics.rs          # requêtes agrégées
│   ├── exports/
│   │   ├── csrd_pdf.rs           # réutilise sobria-export
│   │   ├── prov_o.rs
│   │   └── csv.rs
│   └── error.rs                  # thiserror
├── tests/
│   ├── integration_api.rs        # reqwest + tempdir
│   ├── enrollment_flow.rs
│   ├── analytics.rs
│   └── exports.rs
└── README.md
```

### Frontend Svelte servi par le binaire

```
web-team/                          # nouveau sous-projet
├── package.json                   # @sveltejs/kit ou Vite + Svelte 5 runes
├── svelte.config.js
├── src/
│   ├── routes/
│   │   ├── +layout.svelte         # auth guard + nav
│   │   ├── login/+page.svelte     # admin OU user
│   │   ├── admin/
│   │   │   ├── dashboard/+page.svelte    # analytics agrégés
│   │   │   ├── codes/+page.svelte        # gestion enrollment codes
│   │   │   ├── users/+page.svelte        # liste users
│   │   │   └── exports/+page.svelte      # CSRD PDF + PROV-O + CSV
│   │   └── user/
│   │       ├── dashboard/+page.svelte    # mon usage perso
│   │       └── export/+page.svelte       # mon PROV-O perso
│   └── lib/
│       ├── api.ts                 # fetch wrapper avec JWT + refresh auto
│       ├── auth.ts                # store auth + redirect
│       ├── charts/                # composants Plot/D3 réutilisables
│       └── styles/                # design system Sobr.ia (palette lime/ambre/coral)
└── dist/                          # généré par `npm run build`
```

Le binaire Rust embarque `web-team/dist/` via `rust-embed` (zero-config statique). Routes `/` → `index.html`, `/api/v1/*` → router axum, `/assets/*` → fichiers embedded.

### Protocole REST API v1 (versionné, compatible v0.6.0 EstimatePayload)

| Endpoint | Méthode | Auth | Body | Réponse |
|----------|---------|------|------|---------|
| `/api/v1/enroll` | POST | none | `{code, password}` | `{access_token, refresh_token, user_id, expires_at}` |
| `/api/v1/refresh` | POST | none | `{refresh_token}` | `{access_token, refresh_token, expires_at}` |
| `/api/v1/login` | POST | none | `{username, password, role?}` | `{access_token, refresh_token, role}` |
| `/api/v1/estimations` | POST | user | `EstimatePayload v1` | `{id, ack}` |
| `/api/v1/me/usage` | GET | user | — | `{daily, monthly, totals, team_avg}` |
| `/api/v1/me/exports/prov-o` | GET | user | — | PROV-O JSON-LD |
| `/api/v1/admin/users` | GET | admin | — | `[{id, fingerprint, last_seen, totals}]` |
| `/api/v1/admin/codes` | POST | admin | `{count, ttl_days}` | `[{code, expires_at}]` |
| `/api/v1/admin/codes/:id` | DELETE | admin | — | `{revoked}` |
| `/api/v1/admin/analytics` | GET | admin | query: `from, to, group_by` | aggregates |
| `/api/v1/admin/exports/csrd` | POST | admin | `{from, to, entity_name}` | PDF binary |
| `/api/v1/admin/exports/prov-o` | POST | admin | `{from, to}` | PROV-O JSON-LD |
| `/api/v1/admin/exports/csv` | POST | admin | `{from, to}` | CSV |

### Schéma SQLite `team.sqlite`

```sql
-- v1
CREATE TABLE admins (
    id TEXT PRIMARY KEY,            -- ULID
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,    -- Argon2id PHC
    created_at TEXT NOT NULL,
    last_login_at TEXT
);

CREATE TABLE enrollment_codes (
    id TEXT PRIMARY KEY,            -- ULID
    code_hash TEXT NOT NULL,        -- Argon2id PHC du code 12 chiffres
    created_by TEXT REFERENCES admins(id),
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    used_at TEXT,                   -- NULL = pas encore utilisé (single-use)
    used_by_user_id TEXT REFERENCES users(id),
    revoked_at TEXT
);

CREATE TABLE users (
    id TEXT PRIMARY KEY,            -- ULID
    enrollment_code_id TEXT REFERENCES enrollment_codes(id),
    fingerprint TEXT UNIQUE NOT NULL,  -- "chrome-mac-abc123"
    password_hash TEXT NOT NULL,    -- Argon2id PHC, choisi à l'enrollment
    display_name TEXT,              -- optionnel, choisi à l'enrollment
    created_at TEXT NOT NULL,
    last_seen_at TEXT
);

CREATE TABLE tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id),
    admin_id TEXT REFERENCES admins(id),
    refresh_token_hash TEXT NOT NULL,
    issued_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    revoked_at TEXT,
    CHECK ((user_id IS NULL) <> (admin_id IS NULL))
);

CREATE TABLE estimations (
    id TEXT PRIMARY KEY,            -- ULID time-sortable
    user_id TEXT NOT NULL REFERENCES users(id),
    ts TEXT NOT NULL,
    method TEXT NOT NULL,           -- afnor_sobria | ecologits
    model_id TEXT NOT NULL,
    tokens_in INTEGER NOT NULL,
    tokens_out INTEGER NOT NULL,
    gco2eq_p50 REAL NOT NULL,
    gco2eq_p5 REAL,
    gco2eq_p95 REAL,
    water_ml REAL NOT NULL,
    energy_wh REAL NOT NULL,
    region TEXT,
    raw_payload_json TEXT NOT NULL, -- audit
    received_at TEXT NOT NULL
);
CREATE INDEX idx_estimations_user_ts ON estimations(user_id, ts);
CREATE INDEX idx_estimations_ts ON estimations(ts);
CREATE INDEX idx_estimations_model ON estimations(model_id);
```

---

## 3. Découpage en sous-chantiers

### C28.1 — Bootstrap crate + TLS + serve (1.5 jour)

- `crates/sobria-team-aggregator/Cargo.toml` (axum 0.7, tower-http, rustls, rustls-pemfile, rcgen pour cert auto-signé, jsonwebtoken, rusqlite (workspace), argon2, ulid, clap, serde, tokio).
- CLI minimal `init` + `serve`.
- `init` :
  - Crée `team.sqlite` avec migrations v1.
  - Crée admin initial (prompt password ou `--admin-password`).
  - Génère cert TLS auto-signé via `rcgen` (CN = hostname local), 10 ans validité, sauvegarde `cert.pem` + `key.pem` dans `--data-dir` (défaut `./team-data/`).
  - Affiche URL d'accès (`https://<hostname>:<port>`, défaut 8443) + URL `localhost` pour fallback.
- `serve` :
  - Charge cert + key (ou regen si `--regen-cert`).
  - Lance axum sur HTTPS, log requêtes (tracing).
  - Route `/health` → JSON `{ok: true, version: "0.7.0"}`.
- Tests `tests/serve_health.rs` : démarre serveur, ping `/health`, vérifie 200 OK.

### C28.2 — Auth (JWT + Argon2id) + API REST core (2 jours)

- `server/auth/{jwt, password, middleware}.rs` :
  - Access token JWT HS256 (secret 32 bytes random au `init`, persisté en `team.sqlite.config`), TTL 24 h, claims `{sub, role, iat, exp}`.
  - Refresh token UUID v4, hash Argon2id, persisté en `tokens` table, TTL 7 jours.
  - Middleware extractor `RequireUser` + `RequireAdmin`.
- API REST :
  - `POST /api/v1/enroll` (code 12 chiffres + password + fingerprint).
  - `POST /api/v1/login` (admin OU user, role inféré).
  - `POST /api/v1/refresh`.
  - `POST /api/v1/estimations` (user, body = `EstimatePayload v1` identique à v0.6.0, insert dans `estimations`).
  - `GET /api/v1/me/usage` (user, agrège ses estimations).
- CLI `code create N --ttl-days 7` (génère N codes 12 chiffres, affiche en clair, stocke Argon2id hash).
- CLI `code list` + `code revoke <id>`.
- Tests `tests/integration_api.rs` (reqwest contre serveur axum réel sur port aléatoire avec rustls).

### C28.3 — API admin + analytics (1.5 jour)

- `GET /api/v1/admin/users` → liste avec totaux.
- `GET /api/v1/admin/analytics?from&to&group_by={day|week|month}&dim={user|model|method}` :
  - Agrège `estimations` en SQL pur (GROUP BY).
  - Retourne séries temporelles + top N.
- `storage/analytics.rs` :
  - `daily_totals(user_id?, from, to)`
  - `top_models(user_id?, from, to, n)`
  - `top_users(from, to, n)`
  - `method_breakdown(user_id?, from, to)`
- Alertes seuils (table `alert_thresholds` + check au POST estimations) : optionnel v0.7.0, peut être différé v0.7.1.

### C28.4 — Dashboard Svelte (admin + user) (2 jours)

- Nouveau projet `web-team/` (SvelteKit static adapter).
- Pages :
  - `/login` — formulaire admin OU user (toggle).
  - `/admin/dashboard` — graphiques Plot :
    - LineChart évolution gCO₂eq quotidien/mensuel.
    - BarChart top 10 modèles (gCO₂eq).
    - BarChart top 10 utilisateurs (anonymisable via toggle).
    - PieChart breakdown AFNOR vs EcoLogits.
    - 4 cards "Aujourd'hui / Ce mois / Cette année / Total" (gCO₂eq + eau + énergie + prompts).
  - `/admin/codes` — table + bouton "Créer 10 codes" + révoquer ligne par ligne. Codes affichés en clair UNE seule fois après création (warning).
  - `/admin/users` — table fingerprint, display_name, dernier vu, total gCO₂eq, statut (actif/révoqué).
  - `/admin/exports` — sélecteurs from/to + 3 boutons (CSRD PDF / PROV-O / CSV).
  - `/user/dashboard` — mon usage : LineChart mensuel + 4 cards perso + comparaison vs moyenne équipe anonyme.
  - `/user/export` — bouton "Télécharger mon PROV-O".
- Design system Sobr.ia partagé (palette lime/ambre/coral réutilisée de `web/`).
- Auth client : JWT en mémoire (PAS `localStorage` — XSS), refresh auto sur 401.
- Build → `web-team/dist/` → embarqué via `rust-embed` dans le binaire.

### C28.5 — Exports CSRD + PROV-O agrégés équipe (1 jour)

- `exports/csrd_pdf.rs` : réutilise `sobria-export::csrd_report` en lui passant un agrégat équipe (somme gCO₂eq + breakdown utilisateurs anonymisés ou nommés via toggle admin).
- `exports/prov_o.rs` : JSON-LD avec `prov:wasGeneratedBy` chaque estimation, `prov:wasAssociatedWith` chaque user (anonymisable).
- `exports/csv.rs` : CSV brut avec colonnes ts, user_fingerprint, method, model, tokens_in, tokens_out, gco2eq_p50, etc.
- Endpoints `/api/v1/admin/exports/{csrd|prov-o|csv}` retournent `application/pdf`, `application/ld+json`, `text/csv`.
- Tests `tests/exports.rs` : génère 100 estimations factices, exporte, parse PDF (vérifier headers + signature PROV-O sidecar).

### C28.6 — Mode Équipe dans extension + app (1.5 jour)

- **Extension** :
  - Page Options enrichie : nouvelle section "Mode Équipe" (URL serveur + bouton "Vérifier la connexion" + enrollment code + display name + password + bouton "Enrôler").
  - `extension/src/lib/team-client.ts` : client REST signé JWT + refresh auto.
  - Toggle "Estimations vers : ⚪ Local seul ⚪ Mode Équipe ⚪ Les deux" (les deux = envoie aux deux dest, pour transition).
  - Au POST estimations, si Mode Équipe activé, fetch `https://team-server/api/v1/estimations` avec Bearer JWT.
  - Si cert auto-signé : warning visible dans Options "Certificat auto-signé accepté ? [Voir le fingerprint]". L'utilisateur accepte explicitement le fingerprint au premier enrollment.
- **App Tauri** :
  - `crates/sobria-app/src/team_client.rs` (nouveau) : équivalent Rust du team-client TS.
  - Section `/parametres → Mode Équipe` : équivalent UI.
  - IPC `team_enroll`, `team_status`, `team_logout`.
  - Au POST estimations dans le Journal/Dashboard, si Mode Équipe activé, fetch HTTPS.
- Migration : un user peut être pairé perso ET enrôlé équipe simultanément. Toggle décide où.

### C28.7 — Doc déploiement + packaging multi-OS (1 jour)

- `docs/operations/team-aggregator.md` (~250 lignes) :
  - Quickstart : `wget binaire → chmod +x → ./sobria-team-aggregator init → serve`.
  - Sécurité : firewall, reverse proxy nginx/caddy, let's encrypt si exposition publique.
  - Backup `team.sqlite`.
  - Upgrade entre versions (migrations SQLite).
  - Troubleshooting (cert refusé par extension, code rejeté, refresh token expiré, etc.).
  - Section "Pour les TPE/PME" + section "Pour DSI".
- `.github/workflows/team-aggregator-release.yml` :
  - Trigger : tag `v0.7.0`.
  - Matrix build Linux x86_64, macOS arm64, Windows x86_64.
  - Build aussi `web-team/dist/` avant cargo build.
  - Upload assets : `sobria-team-aggregator-<os>-<arch>`.
- `Dockerfile` à la racine du crate (bonus) :
  - `FROM rust:1.79-slim AS builder` + `FROM debian:bookworm-slim` runtime.
  - Volume `/data` pour `team.sqlite` + certs.
  - Expose 8443.
  - Health check sur `/health`.
- README racine : section "Mode Équipe" ajoutée avec capture dashboard admin + lien doc.

---

## 4. Definition of Done v0.7.0

- [ ] `cargo test --workspace` 100 % vert (incluant `sobria-team-aggregator`).
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cargo fmt --check` propre.
- [ ] `cd web && npm run check && npm run lint` propre.
- [ ] `cd web-team && npm run check && npm run lint && npm run test` propre.
- [ ] `cd extension && npm run check && npm run lint && npm run test` propre.
- [ ] Smoke test bout-en-bout : `init` → `serve` → enrôler 2 users via extension + app → envoyer 5 estimations chacun → admin voit les graphiques agrégés → export CSRD PDF généré + PROV-O valide.
- [ ] Binaire ≤ 25 MB (release optimisé `opt-level = "z"`, lto, strip).
- [ ] Build CI matrix Linux + macOS + Windows produit 3 binaires en release.
- [ ] CHANGELOG entrée `[0.7.0] — YYYY-MM-DD` complète.
- [ ] ADR-0013 mis à jour : statut Phase 2 `Planned → Implemented (v0.7.0)`.
- [ ] Bump versions :
  - `Cargo.toml` workspace.package : `0.6.0 → 0.7.0`
  - `crates/sobria-app/tauri.conf.json` : `0.6.0 → 0.7.0`
  - `web/package.json` : `0.6.0 → 0.7.0`
  - `extension/package.json` + `extension/manifest.json` : `0.6.0 → 0.7.0`
  - `web-team/package.json` : nouveau, `0.7.0`
- [ ] Commits Conventional Commits par sous-chantier + tag `v0.7.0`.

---

## 5. Anti-périmètre (différé v0.8+)

- SSO entreprise (SAML, OIDC, Azure AD, Google Workspace).
- Multi-device (un user, plusieurs extensions/apps).
- RBAC fin (rôles managers entre admin et user).
- Cluster multi-serveur (failover, replication).
- Cloud Sobr.ia hébergé → **jamais**.
- Mode "anonymisation forte" k-anonymity / différentielle.

---

## 6. Risques + mitigations

| Risque | Probabilité | Mitigation |
|--------|-------------|------------|
| Cert auto-signé refusé par extension (CORS / fetch strict) | Haute | Workflow d'acceptation explicite du fingerprint au premier enrôlement. Doc claire pour ajouter le cert au trust store OS (optionnel). |
| Migration code 6 chiffres → enrollment 12 chiffres confuse pour l'utilisateur | Moyenne | UI claire : "Mode perso" vs "Mode équipe", explication + lien doc. |
| Binaire trop gros (> 25 MB) | Faible | Stripping + lto + UPX en CI si vraiment besoin. Le frontend embedded compresse via Brotli. |
| JWT secret leak si fichier `team.sqlite` mal protégé | Moyenne | Doc déploiement insiste sur `chmod 600 team.sqlite`. Possible rotation secret au `init`. |
| Refus de soumission Chrome Web Store quand on déclare `host_permissions` HTTPS large (Mode Équipe = n'importe quel hostname) | Moyenne | Garder `host_permissions` limité aux 3 sites cibles. Le client REST team utilise `fetch` avec URL utilisateur-saisie (pas besoin de permission supplémentaire). |
| Performance analytics SQL sur 1M+ estimations | Moyenne | Index sur (user_id, ts) + (ts) + (model_id). Materialized views si lent (différé v0.7.1). |

---

## 7. Découpage temporel

| Jour | Sous-chantier | Livrable |
|------|---------------|----------|
| J1 | C28.1 | Crate + TLS auto-signé + `init`/`serve` |
| J2 | C28.1 fin + C28.2 début | Auth JWT + enroll + login |
| J3 | C28.2 fin | API REST core + tests intégration |
| J4 | C28.3 | API admin + analytics + alertes |
| J5-J6 | C28.4 | Dashboard Svelte admin + user |
| J7 | C28.5 | Exports CSRD + PROV-O + CSV |
| J8 | C28.6 | Mode Équipe extension + app |
| J9 | C28.6 fin + C28.7 | Doc + packaging multi-OS |
| J10 | Ship | Smoke E2E + CHANGELOG + tag v0.7.0 |

Total estimé : **9-10 jours**.

---

## 8. Livrables annexes

- README racine : section "Mode Équipe — déploiement self-hosted" avec captures.
- Dossier candidature data.gouv.fr : démo dashboard admin + exports CSRD.
- Politique de confidentialité `docs/extension/privacy-policy.md` enrichie (clause "Si vous utilisez le Mode Équipe, vos données vont chez votre employeur, pas chez Sobr.ia").
- ADR-0013 statut → Phase 2 Implemented.
