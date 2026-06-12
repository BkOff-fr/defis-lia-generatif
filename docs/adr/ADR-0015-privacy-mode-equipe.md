# ADR-0015 — Périmètre privacy du Mode Équipe (k-anonymat + partage opt-in)

- **Statut** : Accepted (2026-06-12) — implémenté en C38
- **Date** : 2026-06-12
- **Décideurs** : Thibault, Cowork
- **Contexte** : positionnement produit « dashboard équipe » pour PME/orgs
  externes (extension → `sobria-team-aggregator` self-hosted). Complète
  ADR-0013 (architecture deux étages, opt-in par étage) et ADR-0014.

## Contexte et problème

Le dashboard admin (C28.3/C28.4) expose `top_users` : classement **nominatif**
des employés par gCO₂eq, et `/admin/users` liste les totaux de consommation
par personne. Aucun seuil d'agrégation n'est appliqué.

Or :

1. **Droit** : un suivi nominatif de l'activité des salariés constitue un
   dispositif de surveillance au sens RGPD (proportionnalité, minimisation,
   art. 5) et du droit du travail français (information-consultation du CSE,
   L2312-38 ; information individuelle préalable, L1222-4). La CNIL exige
   une finalité légitime et proportionnée — « curiosité managériale » n'en
   est pas une.
2. **Adoption** : l'extension est installée par le salarié. Un outil perçu
   comme du flicage est désinstallé ou contourné : le capteur meurt.
3. **Cohérence** : CLAUDE.md §7 (privacy by design, opt-in explicite) et
   ADR-0013 (pas de cloud central, opt-in par étage) sont des arguments de
   la candidature data.gouv.fr.

La finalité du produit est le **pilotage budgétaire et environnemental**
(GreenOps/FinOps IA, CSRD scope 3) — pas l'évaluation individuelle.

## Décision

Quatre règles, appliquées **côté serveur** (jamais seulement dans l'UI) :

1. **Self-service intégral** : chaque employé voit l'intégralité de SES
   données (`/me/usage`). Aucune restriction.
2. **k-anonymat des agrégats équipe** : les analytics admin (séries, top
   modèles, breakdown méthodo, KPI) ne sont servis que si le nombre
   d'utilisateurs **actifs dans la fenêtre interrogée** est ≥ k.
   `k = max(3, config.k_anonymity_min)`, défaut **5**, stocké dans la table
   `config`. Sinon : réponse explicite `k_anonymity.blocked = true`,
   sections vides — l'UI explique pourquoi.
3. **Identification opt-in, contrôlée par le salarié** : `users.share_identified`
   (défaut **0**). Le classement « top users » ne montre nommément QUE les
   employés ayant activé le partage (toggle dans leur dashboard,
   `PUT /api/v1/me/sharing`) ; les autres sont fondus dans une ligne
   agrégée « N autres participants ». `/admin/users` reste une vue de
   GESTION (enrôlement, dernier contact, révocation) : les totaux de
   consommation n'y apparaissent que pour les comptes en partage actif.
4. **Pas de granularité de surveillance** : aucune vue admin par heure ou
   par contenu. Le grain temporel admin minimal est le jour, le grain
   d'attribution est l'équipe (sauf opt-in). Les prompts ne quittent
   jamais le poste (ADR-0013, inchangé — l'événement ne contient que des
   métriques).

## Conséquences

- (+) Argument produit : « mesure d'équipe sans surveillance individuelle,
  k-anonyme par construction » — différenciant et aligné candidature.
- (+) Conformité : minimisation par défaut ; le déployeur reste responsable
  de son information CSE/salariés (à documenter dans le guide d'install —
  `docs/operations/`).
- (−) Petites équipes (< k actifs) : pas d'analytics admin. Assumé — c'est
  précisément le cas où l'agrégat désanonymise.
- (−) Migration : DDL v3 (`share_identified`), valeur par défaut 0 → les
  classements existants se vident jusqu'aux opt-ins. Assumé.
- Rétention : purge des estimations > `retention_days` (défaut 730 j,
  config) — programmée C38.x si non livrée avec C38.

## Options rejetées

- **Anonymisation côté UI** (toggle admin, serveur nominatif) : pseudo-
  protection, l'API reste un outil de surveillance (commentaire historique
  de `UserTop` envisageait cette voie).
- **Drill individuel admin avec « consentement » coché par l'admin** : le
  consentement appartient au salarié, pas à l'admin.
- **Pseudonymes stables côté admin** : ré-identifiables par recoupement
  (volumes, horaires) ; n'apporte rien face au k-anonymat + opt-in.
