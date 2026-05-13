# ADR-0011 — Réduction du périmètre v1.0 à 13 modules essentiels

- **Statut** : Accepted
- **Date** : 2026-05-13
- **Décideurs** : Thibault, Cowork
- **Contexte** : fin de C21 (M18 Batch CSV), backend Rust à 14 modules
  livrés, frontend à 14 routes (dont 3 placeholders M2/M5/M10).

## Contexte

L'ADR-0010 (Personas et gating modulaire) figeait un référentiel de **25
modules** (M1-M25, M4 réservé) avec un objectif "tout livrer en v1.0
avant la soumission défi data.gouv.fr".

Sur la trajectoire actuelle (chantiers C09 → C21), on a :

- **14 modules backend livrés** : M1, M3, M7, M8, M9, M12, M13, M15, M17
  (datasheet seul), M18 (logic seul), M20, M22, M25 + onboarding/gating.
- **14 routes frontend** : dont 3 placeholders (M2 Workbench, M5
  Exporter, M10 Importer) sans logique métier finalisée.
- **10 modules restants** non implémentés : M2 final, M5 final, M6, M10
  final, M11, M14, M16, M18 UI, M19, M21, M23, M24.

Trois symptômes pratiques :

1. **Dispersion** : 24 modules à équilibrer pour un défi notable d'abord
   sur sa **rigueur méthodologique** et son **angle territorial français**,
   pas sur l'exhaustivité de surface.
2. **Modules redondants** : M2 Workbench ⊂ M3 Comparer + M18 Batch ;
   M5 Exporter ⊂ M22 CSRD + M17 datasheet + M18 CSV ; M6 Géoloc
   unitaire ⊂ M12 Datacenters ; M10 Import logs ⊂ M18 Batch.
3. **Modules à hors-scope app** : M11 Extension navigateur (chantier MV3
   séparé), M19 Équipe (multi-utilisateurs = auth back-end), M21 Alertes
   (OS-specific), M23 Marchés publics (très niche, mérite partenariat
   institutionnel).

## Décision

Réduire le périmètre v1.0 (cible défi data.gouv.fr) à **13 modules
essentiels**. Les 11 modules restants sont marqués **v1.1+** dans la
roadmap, leurs `ModuleId` restent dans l'enum (compat fwd) mais ne
figurent dans **aucun bundle persona par défaut**.

### Liste des 13 modules v1.0

#### 🏆 Cœur méthodologique et transparence (5)

| ID | Module | Justification |
|----|--------|---------------|
| M1 | Estimer un prompt | Cœur produit, démontre la méthodo Monte-Carlo |
| M7 | Journal d'audit | Chaîne SHA-256, preuve de rigueur, différencie tous concurrents |
| M9 | Référentiel modèles | Transparence des chiffres, fiches détaillées P5/P50/P95 |
| M8 | Méthodologie interactive | Détail scientifique (AFNOR SPEC 2314) |
| M14 | À propos / Crédits | Licences, contributeurs, légal |

#### 🇫🇷 Angle territorial unique (2)

| ID | Module | Justification |
|----|--------|---------------|
| M20 | Territoire FR (IRIS + Sankey) | Pivot ComparIA × RTE IRIS, **différenciateur clé** |
| M12 | Datacenters Europe (carte) | Wow factor en démo, croisement géo |

#### 💼 Use cases pros / chercheurs (3)

| ID | Module | Justification |
|----|--------|---------------|
| M22 | Rapport CSRD / AGEC | Livrable conformité PDF + PROV-O |
| M17 | Empreinte projet (datasheet Gebru) | Reproductibilité scientifique, standard académique |
| M3 | Comparer modèles | Use case dev concret |

#### 🎓 Pédagogie et rétention (3)

| ID | Module | Justification |
|----|--------|---------------|
| M13 | Simulateur « Et si...? » | Wow effect, frappe en démo |
| M15 | Tableau de bord personnel | Rétention quotidienne (étudiant) |
| M25 | Eco-budget / Objectifs | Transformation d'usage |

### Modules différés v1.1+ (11)

| ID | Module | Raison du différ |
|----|--------|------------------|
| M2 | Workbench multi-prompts | Chevauche M3 + M18 |
| M5 | Rapports génériques (CSV/JSON/Parquet) | Chevauche M22 + M17 + M18 |
| M6 | Géoloc datacenter unitaire | Chevauche M12 |
| M10 | Import logs | Chevauche M18 Batch CSV (à finaliser v1.1 si Batch frontend pas fait) |
| M11 | Extension navigateur MV3 | Chantier majeur séparé (Chrome + Firefox stores) |
| M16 | Forecaster 12 mois | Backend prêt mais UI non livrée — bonus v1.1 |
| M18 | Batch CSV (frontend) | Backend prêt mais UI non livrée — bonus v1.1 |
| M19 | Équipe / multi-utilisateurs | Auth backend + partage = chantier majeur |
| M21 | Alertes & seuils | Notifications système = OS-specific |
| M23 | Marchés publics IA frugale | Niche, partenariat institutionnel d'abord |
| M24 | Apprendre (mini-cours) | Volume sans différenciateur clé |

### Bundles persona v1.4

Recomposés sur les 13 modules retenus :

| Persona | Bundle par défaut |
|---------|-------------------|
| `student` | M1, M8, M13, M14, M15, M25 (6) |
| `pro_tech` | M1, M3, M7, M8, M9, M13, M14 (7) |
| `enterprise` | M1, M7, M12, M14, M15, M17, M20, M22, M25 (9) |
| `public_sector` | M1, M8, M12, M14, M17, M20, M22 (7) |
| `researcher` | M1, M3, M7, M8, M9, M14, M17 (7) |

M1 reste dans tous les bundles (point fixe).

### Action sur le code

- `sobria-core::preferences::Persona::default_modules()` : update les 5
  bundles.
- `sobria-core::preferences::ModuleId` : **inchangé** (les 24 variantes
  restent, dont M2/M5/M6/M10/M11/M16/M18/M19/M21/M23/M24).
- Tests : adapter les invariants de bundles (M1 dans tous, longueurs).
- `web/src/routes/+layout.svelte` : retirer du rail M2/M5/M10
  (placeholders sans valeur), ajouter M14 et M17 quand les routes
  seront créées.
- CDC v1.0 → bump v1.4 avec changelog.

## Conséquences

### Positives

- **Focus** : 13 modules cohérents, démontrables, polissables en bonus
  end-game (screencast, dossier, README) plutôt que disperser sur 24.
- **Story claire** : un dossier de candidature peut décrire 13 modules
  bien intégrés vs 24 en surface.
- **Backend gratuit** : tout le code Rust livré (M16 Forecaster, M18
  Batch CSV, modules dashboard, etc.) reste compilé, testé, et
  activable plus tard. Pas de perte de travail.
- **Performance binaire** : aucun impact (gating frontend, pas de gating
  binaire).

### Négatives / Risques

- **Backlog v1.1 important** : 11 modules à reprendre après la
  candidature. Documenté.
- **Workbench / Batch CSV différés** : pour les pros qui veulent uploader
  un fichier, la fonctionnalité reste accessible via la commande IPC
  `run_batch_from_csv` (mais nécessite intervention CLI v1.0).
  Documenté dans M22 Rapport CSRD comme alternative en attendant M18 UI.
- **Module M14 À propos** : à créer rapidement (1h frontend). Non-bloquant
  mais nécessaire pour les licences/contact légaux.

### Neutres

- Aucun impact sur la méthodologie scientifique du moteur.
- Aucun impact sur l'audit ledger (déjà journalise tout).
- Aucun impact sur les conventions de personas / gating (toujours actifs).

## Alternatives écartées

| Alternative | Raison du rejet |
|---|---|
| **Tout livrer v1.0 (24 modules)** | Charge frontend disproportionnée, dispersion, modules redondants. |
| **Garder 18 modules** (compromis) | Aucun module pivot à laisser flou — coupe au scalpel. |
| **Réduire à 8 modules** | Trop court, on perdrait la diversité de personas. |
| **Faire un binaire pro vs étudiant** | Anti-pattern (ADR-0010), démultiplie la maintenance. |

## Plan de mise en œuvre

1. **C22.0** — ce ADR (this).
2. **C22.1** — `sobria-core::preferences` : update `default_modules()`.
3. **C22.2** — Rail + CDC v1.4 mis à jour.
4. **Tag git v0.3.0-mvp-13-modules** une fois 1-3 mergés.
5. **Polishing end-game** : screencast 3 min, dossier candidature, README
   focalisé sur les 13 modules.
6. **M14 À propos** : prompt Claude Code (5-10 lignes, contenu statique
   Markdown sur licences + contributeurs + URL).

## Références

- ADR-0010 (Personas et gating modulaire) — base architecturale conservée.
- Brief C22 (à créer si besoin de tracker M14 séparément).
- CDC v1.4 §3 (personas v3) et §4 (13 modules v1.0 + 11 modules v1.1+).
