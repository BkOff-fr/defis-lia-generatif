# C38 — Dashboard équipe « sans surveillance » (ADR-0015) — exécuté

> **Statut** : exécuté le 2026-06-12 (session Cowork), à relire + commiter.
> **Décision produit** (Thibault, 2026-06-12) : cible = PME/orgs externes ;
> le « produit équipe simple » est le dashboard web embarqué de
> `sobria-team-aggregator`, PAS une seconde app desktop. Rebranding
> « Sobr.ia Research » différé post-UAT.

## 1. Problème

Le dashboard admin C28 exposait un classement nominatif `top_users` et des
totaux de consommation par employé, sans seuil — un outil de surveillance
individuelle de fait (risque RGPD/CSE, tueur d'adoption, contraire à
CLAUDE.md §7). L'« anonymisation » existante était un checkbox côté client.

## 2. Livré

### Backend (`sobria-team-aggregator`)
- **DDL v3** : `users.share_identified` (défaut 0) +
  `config.k_anonymity_min` (défaut '5'). Migration progressive v2→v3.
- **k-anonymat serveur** (`/api/v1/admin/analytics`) : si utilisateurs
  actifs sur la fenêtre < `max(3, k)`, sections vides +
  `k_anonymity{required, active_users, blocked}` explicite.
- **`top_users` → `top_users_shared`** : seuls les opt-in apparaissent
  nommés ; les autres fondus dans `{anonymous_users, anonymous_count,
  anonymous_gco2eq_g}`.
- **`GET|PUT /api/v1/me/sharing`** : le consentement appartient au salarié
  (route user-only ; un token admin reçoit 403 — testé).
- **`/api/v1/admin/users`** : vue de gestion ; `totals` devient `null`
  sans opt-in (masquage côté serveur, jamais UI).

### Frontend (`web-team/`)
- Dashboard admin : carte « Agrégats protégés par le k-anonymat » quand
  bloqué ; « Top employés » + checkbox « Anonymiser » (client-side)
  remplacés par « Participants (partage opt-in) » + barre agrégée
  « N participants (anonyme) ».
- Espace salarié : carte « Partage identifié avec l'admin » avec toggle
  (seule écriture possible du flag), explication du défaut k-anonyme.
- `/admin/users` : « — partage non activé » à la place des totaux ;
  tagline login : « aucune surveillance individuelle sans consentement ».

## 3. Vérifications exécutées

- `cargo check` ✓ · `cargo clippy --lib -D warnings` ✓
- `cargo test --lib` : **92 passed** (dont 5 nouveaux : migration v3,
  opt-in storage ×2, top_users_shared ×2, active_user_count)
- `cargo test --test integration_admin` ✓ — flow complet : totaux masqués
  par défaut → opt-in Alice → 403 admin sur /me/sharing → analytics
  bloqués (1 actif < k=5) → k=3 + 3 actifs → débloqués, Alice seule
  nommée, 2 anonymes agrégés.
- `web-team` : svelte-check 0 erreur ; build ✓ ; binaire debug recompilé
  avec UI embarquée ; **serveur lancé en sandbox + seed 3 employés** ;
  captures Playwright : état bloqué k=5, état débloqué, /admin/users,
  espace salarié avec toggle.
- Non vérifié ici : `cargo tarpaulin`, release Windows, e2e sur binaire
  release.

## 4. Restes à faire

1. **Rétention** : purge `estimations` > `retention_days` (défaut 730 j) —
   tâche périodique au boot (ADR-0015 « Conséquences »).
2. **CLI admin** pour `k_anonymity_min` (aujourd'hui : sqlite direct) +
   doc opérateur (`docs/operations/`) incluant le volet info CSE/salariés.
3. **Lint web-team** : dérive Prettier préexistante (~20 fichiers, hors
   périmètre C38) — commit de formatage dédié.
4. **LineChart** : ticks Y au formatage incohérent (1.85 vs 925.0) —
   bug viz préexistant constaté sur captures.
5. **C39 — mode Simple desktop** (décision validée, non engagé) : rail
   avec labels, 4-5 modules par défaut, jargon derrière toggle Expert.
6. Positionnement : page « Mode Équipe » sur le site + argumentaire
   « GreenOps IA k-anonyme » dans la candidature.
