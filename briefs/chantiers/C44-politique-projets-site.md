# C44 — Politique de visibilité, dimension projet, site restructuré

> **Statut** : exécuté le 2026-06-12 (session Cowork), à relire + commiter.
> **⚠ Vérification partielle** : l'environnement d'exécution est mort en
> cours de chantier (VM sandbox) — voir §5 pour la frontière exacte
> vérifié / non vérifié et les commandes à lancer.
> **Décisions produit** (Thibault) : politique sélectionnable au
> déploiement et configurable ensuite ; projet choisi par conversation
> (extension + app) ; refonte structure + pages du site, DA conservée.

## 1. Politique de visibilité (ADR-0016, aggregator)

- `config.visibility_policy` : `anonymous` | `opt_in` (défaut) |
  `identified`. CLI : `config set visibility_policy identified
  --attest "CSE informé le …"` — refus sans attestation ; attestation
  (texte, date, via) stockée et affichée ; purgée au retour à un mode
  protecteur. `config list/get` exposent la politique.
- `/admin/analytics` : champ `policy` + matrice — `anonymous` : opt-in
  ignorés (tout agrégé) ; `opt_in` : ADR-0015 inchangé ; `identified` :
  `top_users_all` nominatif intégral, k-gate désactivé (sans objet).
- **Nouveau** `GET /admin/users/:id/analytics` : détail par employé
  (totaux, série quotidienne, modèles, méthodos) — 403 si la politique
  ne l'autorise pas (la garde vit CÔTÉ SERVEUR).
- `/admin/users` : totaux selon politique (tous / opt-in / aucun).
- `/me/sharing` : renvoie aussi `policy` — chaque salarié sait sous quel
  régime il travaille.

## 2. Dimension projet (DDL v4 + bout en bout)

- `estimations.project` (TEXT NULL, indexé) ; payload REST top-level
  `project` (normalisé : trim, 64 chars).
- `project_breakdown(from, to, k)` : agrégats par projet ; sous k
  contributeurs distincts → repli « autres projets » (`folded`) — un
  projet d'une personne est une personne. k désactivé en `identified`.
- **Extension** : étiquette par CONVERSATION (clé = host+pathname de
  l'onglet). `shared/projects.ts` (storage local liste + mapping),
  sélecteur dans le popup (visible si site suivi + équipe enrôlée),
  résolution au dispatch dans le service worker via `sender.tab.url`
  (zéro modification des content scripts). +1 spec vitest (fonctions
  pures).
- **web-team** : badge politique en tête du dashboard, panneau « Par
  projet » + carte de lecture, page `/admin/users/[id]` (SPA, ssr off),
  noms cliquables dans la liste, ligne de politique dans l'espace
  salarié (textes adaptés par mode).

## 3. Hors périmètre (assumé)

Le sélecteur de projet côté **app desktop** (Composer) touche
`crates/sobria-app` — saturé de WIP non commité : à faire dans un
chantier dédié APRÈS tes commits (`C45`), avec le champ `project` dans
le push `team_client` et le ledger local.

## 4. Site (agent — vérifié avant la panne)

`/produit`, `/equipe` (3 politiques avec tableau comparatif — le
nominatif jamais en avant), `/methode`, `/telecharger` restructuré
(ancres), `/cloud` → redirection `/equipe`, topbar/footer réécrits.
Build 51 pages vert, 0 lien cassé, captures `site_produit/equipe/
methode.png`, checksums 0 divergent.

## 5. Vérifications — frontière exacte

**Vérifié avant la panne de l'environnement :**
- Aggregator parties 1-3 (schema v4, policy.rs, project colonne+payload,
  analytics projet/top_users_all, matrice handler, user_detail, users
  unmasked) : `cargo check` VERT.
- Site : build Astro 51 pages + captures + checksums (rapport agent).
- Extension C43 (popup agent) : 84/84 vitest, build, tsc, eslint.

**Écrit APRÈS la panne — code relu, fins de fichiers contrôlées, mais
NI COMPILÉ NI TESTÉ :**
- Aggregator partie 4 : CLI `--attest` (cli.rs, commands/config.rs,
  +1 test), `/me/sharing.policy` (me.rs).
- web-team : dashboard (policy badge, projets, drill), users/[id],
  users liste, espace salarié.
- Extension : projects.ts, payload `project`, service-worker sender,
  popup picker (html/ts/css), tests/unit/projects.spec.ts.

**À lancer (dans cet ordre) :**
```bash
cargo test -p sobria-team-aggregator && cargo clippy -p sobria-team-aggregator --lib --tests -- -D warnings
cd web-team && npm run check && npm run lint && npm run build
cd ../extension && npm test && npm run build
cd ../site && npm run lint && npx astro build   # déjà vert, re-contrôle
```
Points d'attention probables : imports inutilisés éventuels
(`user_detail.rs`), le spec `parametres-mode-equipe`/`integration_admin`
si le shape JSON les surprend (champs additifs normalement sans effet),
et la route SPA `[id]` au build adapter-static (ssr=false posé).

## 6. Restes

1. C45 — projet côté app desktop (après commits).
2. UI admin pour changer la politique (aujourd'hui : CLI) + affichage
   de l'attestation dans le dashboard.
3. Doc opérateur : section politique (note ajoutée) à enrichir
   d'exemples ; emails CSE à décliner par politique.
4. Filtre par projet dans les vues (drill-down projet → membres selon
   politique).
5. UAT + tout le reliquat des briefs C40-C43.
