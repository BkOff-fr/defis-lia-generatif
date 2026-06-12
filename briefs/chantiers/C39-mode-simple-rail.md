# C39 — Rail simplifié + finitions C38 — exécuté

> **Statut** : exécuté le 2026-06-12 (session Cowork), à relire + commiter.
> **Origine** : « l'interface peut sembler trop compliquée pour n'importe
> qui » (Thibault). Treize icônes sans labels + jargon de codes modules.

## 1. Rail simplifié (web/src/routes/+layout.svelte)

- **5 essentiels toujours visibles, avec labels** sous l'icône (12px,
  plancher typo respecté) : Estimer · Comparer · Suivi · Modèles ·
  Datacenters. Largeur rail 76 → 96px (`--rail-w`).
- **9 modules derrière « Plus »** (chevron) : Simuler, Budget, Territoire,
  Datasheet, CSRD, Journal, Méthode, Méthodos, À propos. État persisté
  (`localStorage sobria.rail_expanded`) ; **auto-dépliage si la page
  active est dans « Plus »** (on ne cache jamais l'endroit où on est).
- Le gating personas (ADR-0010, `visible()`) s'applique inchangé
  par-dessus. Template refactoré en `{#snippet railBtn}` (3 duplications
  supprimées).

## 2. Jargon retiré de l'UI

Eyebrows « Module M9 · … » humanisés sur les 12 pages (ex. « Catalogue
transparent des modèles », « Tableau de bord personnel »). Les codes Mx
restent dans les commentaires/docs ; plus aucun code interne visible.

## 3. Finitions C38 (aggregator)

- **Rétention** : purge des estimations > `retention_days` (défaut 730 j,
  plancher 30) au boot puis toutes les 24 h (`spawn_retention_task`).
- **CLI `config list|get|set`** : allow-list `k_anonymity_min`,
  `retention_days`, validation des planchers, clés internes inaccessibles.
- **Doc opérateur** (`docs/operations/team-aggregator.md`) : section
  privacy/k-anonymat/rétention + obligations du déployeur (CSE L2312-38,
  L1222-4, registre RGPD) avec disclaimer non-juridique.
- **Formatage web-team** : dérive Prettier préexistante résorbée
  (l'ensemble du package est désormais lint-clean).

## 4. Incident montage (à connaître)

Le montage Windows a corrompu plusieurs écritures durant la session
(truncations silencieuses : `api.ts`, `app.css`, `m15`, puis 4 routes).
Récupération : reconstruction depuis `git show HEAD:` + rejeu scripté des
transformations, puis **politique stricte** : éditions dans /tmp,
`rsync --checksum` vers le repo, vérification SHA-256 bidirectionnelle.
État final vérifié : **0 divergence** sur web/src, web/tests, web-team.
Recommandation : vérifier `git diff` attentivement avant commit, et si un
fichier paraît étrange, le signaler.

## 5. Vérifications exécutées

- web : svelte-check 0 erreur · prettier ✓ · build ✓ · **suite e2e
  complète 29 passed / 2 skipped / 0 failed** (18 specs, rail traversé
  par toutes les pages).
- aggregator : 96 tests lib (4 nouveaux : purge, config ×3) · clippy
  `-D warnings` ✓ · intégration admin ✓.
- Captures : rail replié / déplié / page « Plus » active.

## 6. Restes à faire

1. Tests e2e dédiés au rail (toggle, persistance, auto-dépliage) — la
   suite actuelle le traverse mais ne l'asserte pas.
2. Raccourcis clavier (1-5 pour les essentiels ?) — à concevoir.
3. i18n des labels courts quand `svelte-i18n` sera branché (CDC).
4. UAT C36 : vérifier avec les 5 personas que « Plus » est découvrable.
