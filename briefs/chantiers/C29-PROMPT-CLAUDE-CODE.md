# C29 — Prompt Claude Code (v0.7.1 — Polish Mode Équipe)

> **Mode d'emploi** : copier-coller le bloc ci-dessous dans une nouvelle session Claude Code (CLI) à la racine du repo. Le prompt démarre par `/using-superpower`.

---

```
/using-superpower

# Mission : C29 — Polish Mode Équipe (v0.7.1)

Tu vas finir le polish de la v0.7.0 (Mode Équipe self-hosted) en
livrant 4 patches concrets sur ~2-3 jours. Pas de feature nouvelle hors
ce qui est listé. C'est un patch release, pas un saut majeur.

## Contexte à charger AVANT toute action

Lis ces fichiers dans l'ordre :

1. `CLAUDE.md` — règles, anti-patterns, DoD.
2. `briefs/chantiers/C29-v0.7.1-polish-mode-equipe.md` — brief complet,
   source de vérité pour le périmètre + DoD + découpage.
3. `CHANGELOG.md` entrée [0.7.0] — récap C28 + section "Différé v0.7.1+"
   qui liste les 4 manques que tu vas combler.
4. `docs/adr/ADR-0013-extension-pairing-team-mode.md` — décision
   architecturale globale.
5. `crates/sobria-app/src/team_settings.rs` + `crates/sobria-app/src/team_client.rs`
   — les 8 IPC `team_*` déjà câblés en Rust (C28.6), à exposer côté
   frontend `web/`.
6. `crates/sobria-app/src/main.rs` — vérifier le wiring exact des 8 IPC.
7. `crates/sobria-team-aggregator/src/commands/` — état actuel des CLI
   `init` / `serve`, à enrichir avec `admin` et `--regen-cert`.
8. `crates/sobria-team-aggregator/src/storage/schema.rs` — schéma v1,
   migration v2 à ajouter pour les alertes.
9. `web-team/src/routes/admin/` — pages existantes (dashboard, codes,
   users, exports), pattern à reprendre pour la nouvelle page `alerts`.
10. `web/src/lib/api.ts` + `web/src/routes/parametres/+page.svelte` —
    point d'extension pour la nouvelle section "Mode Équipe".

## Stratégie + garde-fous

- **Pas de feature nouvelle** hors les 4 patches du brief.
- **Réutiliser** le design system Sobr.ia (palette lime / ambre / coral).
- **Tests-first** quand applicable.
- **Argon2id** pour le reset-password admin (cohérence avec C28).
- **TLS** : pas d'OpenSSL, rustls + ring + rcgen comme déjà fait.
- **DEMANDER** si une ambiguïté apparaît (notamment sur le format exact
  de notification webhook ou le champ SMTP).

## Plan d'exécution

### C29.1 — UI Mode Équipe côté frontend Tauri (1 jour)

Voir brief §C29.1. Résumé :

- `web/src/lib/api.ts` : ajouter les 8 wrappers TypeScript :
  ```ts
  export interface TeamStatusDto { url: string|null; mode: 'local'|'team'|'both';
    accept_invalid_certs: boolean; user_id: string|null;
    fingerprint: string|null; last_seen_at: string|null;
    estimations_sent: number; }
  export function getTeamStatus(): Promise<TeamStatusDto>;
  export function setTeamUrl(url: string): Promise<void>;
  export function setTeamMode(mode: 'local'|'team'|'both'): Promise<void>;
  export function setTeamAcceptInvalidCerts(accept: boolean): Promise<void>;
  export function teamPing(): Promise<{ok: boolean; version?: string; error?: string}>;
  export function teamEnroll(code: string, password: string, displayName: string|null,
    fingerprint: string): Promise<{user_id: string}>;
  export function teamLogout(): Promise<void>;
  // team_push_estimation est déclenché par le dispatcher Rust, pas exposé UI.
  ```
- `web/src/lib/team-store.ts` (nouveau) : store typé `writable<TeamState>`
  + `loadTeam()` + `saveTeamField(field, value)` optimistic + rollback.
- `web/src/routes/parametres/+page.svelte` : nouvelle section "Mode
  Équipe self-hosted" entre Extension navigateur et Runtime (voir brief
  pour le contenu exact des 4 sous-sections : Statut, Configuration,
  Enrôlement, Dispatcher).
- Validation côté frontend :
  - URL : regex `^https?://.+`, avertir si http (cert manuel).
  - Enrollment code : regex `^\d{12}$`.
  - Password : ≥ 8 chars, force basique.
- Tests Playwright `web/tests/parametres-mode-equipe.spec.ts` :
  - Stub les 8 IPC via `window.__TAURI__` mock.
  - Vérifie : configuration URL → ping → enrôlement → status visible →
    set_mode → logout → retour état initial.

DoD C29.1 : un utilisateur peut, depuis l'UI app Tauri seul, enrôler son
device sur un serveur Mode Équipe et choisir où ses estimations partent.

### C29.2 — CLI `admin reset-password` + `admin list` (0.5 jour)

Voir brief §C29.2. Résumé :

- `crates/sobria-team-aggregator/src/commands/admin.rs` (nouveau) :
  ```rust
  pub fn list_admins(state: &ServerState) -> Result<Vec<AdminSummary>>;
  pub fn reset_password(state: &ServerState, username: &str, new_password: &str) -> Result<usize>;
  ```
- `src/cli.rs` : sous-commande `admin {reset-password|list}` avec clap.
- `reset-password` :
  - Prompt password 2 fois via `rpassword` (déjà dans le workspace ? sinon ajouter `rpassword = "7"`).
  - Argon2id PHC hash.
  - UPDATE password_hash + last_login_at = NULL.
  - Révoque tous les tokens admin actifs (UPDATE tokens SET revoked_at).
  - Affiche "Mot de passe de <username> réinitialisé. N token(s) révoqué(s)."
- `list` : table `id | username | created_at | last_login_at`.
- Tests `tests/cli_admin.rs` : crée admin, reset, vérifie hash changé +
  tokens révoqués.

### C29.3 — `serve --regen-cert` + rotation TLS (0.5 jour)

Voir brief §C29.3. Résumé :

- `src/commands/serve.rs` : flag `--regen-cert` qui, avant de bind :
  - Backup `cert.pem` + `key.pem` → `cert.pem.bak.<unix_ts>` + `key.pem.bak.<unix_ts>`.
  - Regen via `rcgen` (même CN, même SANs, validité 10 ans).
  - Affiche fingerprint SHA-256 du nouveau cert.
- Section `docs/operations/team-aggregator.md` enrichie : "Rotation TLS".
- Tests `tests/regen_cert.rs` : démarre serveur avec init, regen, vérifie
  cert différent + même CN.

### C29.4 — Alertes seuils (1 jour)

Voir brief §C29.4. Résumé :

- Migration SQLite v2 dans `storage/schema.rs` :
  - Table `alert_thresholds` (id, scope, target_id?, period, gco2eq_max,
    notify_kind, notify_target?, created_by_admin_id, created_at,
    disabled_at).
  - Table `alert_triggers` (id, threshold_id, period_start, period_end,
    observed_gco2eq, triggered_at, notified_at, notify_error).
- Module `src/alerts/` :
  - `alerts/store.rs` : CRUD sur alert_thresholds + alert_triggers.
  - `alerts/checker.rs` : `check_thresholds_for_user(user_id, ts)` →
    pour chaque threshold actif, calcule l'observé, insère trigger si
    dépassement.
  - `alerts/notify.rs` : `notify(trigger, threshold)` → match `notify_kind`
    {webhook, email, log_only}. Webhook : POST JSON timeout 5 s. Email :
    `lettre = "0.11"` features `smtp-transport`, `rustls-tls`. Si SMTP
    pas configuré → fallback log_only + `notify_error` rempli.
- Routes :
  - `POST /api/v1/admin/alerts` → créer.
  - `GET /api/v1/admin/alerts` → lister.
  - `DELETE /api/v1/admin/alerts/:id` → soft delete (disabled_at).
  - `GET /api/v1/admin/alerts/triggers?from&to&limit=50` → historique.
- Wiring : au handler `POST /api/v1/estimations`, après insert,
  appeler `alerts::checker::check_thresholds_for_user`.
- UI `web-team/src/routes/admin/alerts/+page.svelte` :
  - Form création : scope (radio user/team), target user (select si user),
    period (radio day/week/month), gCO₂eq max (numeric), notify_kind
    (radio + champs conditionnels), bouton "Créer".
  - Table des thresholds avec bouton "Désactiver" par ligne.
  - Section "Historique" : table des 50 derniers triggers (timestamp,
    threshold, observé, notification OK/KO).
- Tests `tests/alerts.rs` : crée threshold, insère 10 estimations qui
  dépassent, vérifie 1 trigger inséré + notification simulée (mock webhook).

### C29.5 — Doc + tag (0.5 jour)

- CHANGELOG entrée `[0.7.1] — YYYY-MM-DD — Polish Mode Équipe (C29)` avec
  4 sections (C29.1 UI + C29.2 admin + C29.3 regen-cert + C29.4 alerts).
- README racine : section "Mode Équipe" enrichie.
- `docs/operations/team-aggregator.md` enrichi : rotation TLS, reset
  password, configuration alertes.
- Bump versions :
  - Cargo.toml workspace : 0.7.0 → 0.7.1
  - tauri.conf.json : 0.7.0 → 0.7.1
  - web/package.json : 0.7.0 → 0.7.1
  - extension/package.json + manifest.json : 0.7.0 → 0.7.1
  - web-team/package.json : 0.7.0 → 0.7.1

## DoD globale

- [ ] `cargo test --workspace` 100 % vert.
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cargo fmt --check` propre.
- [ ] `cd web && npm run check && npm run lint && npm run test` propre.
- [ ] `cd web-team && npm run check && npm run lint && npm run test` propre.
- [ ] `cd extension && npm run check && npm run lint && npm run test` propre.
- [ ] Smoke test E2E manuel : enrôlement complet via UI app Tauri sans
      éditer la SQLite, reset-password admin via CLI, regen cert, alerte
      déclenchée par dépassement seuil.
- [ ] CHANGELOG [0.7.1] complète.
- [ ] Bump versions cohérent.
- [ ] Tag v0.7.1 créé.

## Convention de commit

```
feat(web): C29.1 UI Mode Équipe câblée aux 8 IPC team_* (app Tauri)
feat(team): C29.2 CLI admin reset-password + list
feat(team): C29.3 serve --regen-cert + doc rotation TLS
feat(team): C29.4 alertes seuils (table v2 + API + UI + notify webhook/email)
chore(release): bump v0.7.1
```

Tag final :

```bash
git tag -a v0.7.1 -m "v0.7.1 — Polish Mode Équipe (C29)

Polish de la v0.7.0 : UI Mode Équipe câblée côté app Tauri (8 IPC
team_* exposés et utilisables sans éditer la SQLite), CLI admin
reset-password + list, serve --regen-cert pour rotation TLS, alertes
seuils (table v2, API REST, notifications webhook/email/log_only).

ADR-0013 Phase 2 reste Implemented. Sprint candidature data.gouv.fr
(v1.0.0) prochaine étape."
```

## Garde-fous

- **JAMAIS** de feature hors les 4 patches du brief.
- **JAMAIS** de cloud Sobr.ia (les notifications sortent vers les
  serveurs DES utilisateurs, pas vers nous).
- **TOUJOURS** Argon2id PHC pour les hashs.
- **TOUJOURS** rustls + ring (pas d'OpenSSL).
- **DEMANDER** si SMTP configuration ambiguë — il vaut mieux fallback
  log_only que crash.
- Le cas SMTP non configuré DOIT marcher en log_only (graceful degrade).

Bonne mission. Commence par C29.1 (l'item le plus visible UX), puis
C29.4 (le plus de valeur métier), puis C29.2 + C29.3 (rapides), puis
C29.5 (ship).
```

---

## Notes pour Thibault

- Sprint court (2-3 jours), prompt plus serré que C28.
- Au retour, comme d'habitude : `git log --oneline -10` + on review
  avant tag.
- Si tu veux pouvoir tester les notifications email en local, prépare
  un SMTP simple (mailhog ou maildev) — la doc `docs/operations/team-
  aggregator.md` aura un exemple.
- Après v0.7.1 → **v1.0.0 candidature data.gouv.fr** : c'est le vrai
  sprint final (~1 semaine), dossier candidature + vidéo démo + dataset
  consolidé DVC + binaires signés + UAT 5 testeurs. On en reparle dès
  v0.7.1 shippée.
