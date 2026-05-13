# Roadmap Sobr.ia — 16 semaines (v2)

> **Période cible** : S0 démarrage 12 mai 2026 → soumission S16 (début septembre 2026).
> **Cadence** : 1 sprint = 1 semaine. Revue chaque vendredi.
> **Format des sprints** : brief détaillé dans `briefs/sprints/Sx-*.md`.
>
> **Changelog v2 (13 mai 2026)** : extension de scope suite à la décision CDC v1.3 / ADR-0010.
> - 25 modules à livrer en v1.0 (au lieu de 12) couvrant 5 personas.
> - Calendrier étendu de 4 semaines (S12 → S16).
> - Travail en parallèle Cowork (Rust + briefs) ↔ Claude Code (frontend) via les prompts C-Claude-Code.
> - Plus aucune feature nouvelle après S14 (scope freeze).

## Vue calendaire

| Sprint | Dates | Thème | Bloquants levés | Statut |
|--------|-------|-------|-----------------|:------:|
| S0 | 12-18 mai | Revue bibliographique + cadrage scientifique | Méthodologie défendable | 🟡 |
| S1 | 19-25 mai | Bootstrap technique + foundation pipeline (C01) | Repo, CI, ADR figés, trait DataLayer + helpers | ✅ |
| S2 | 26 mai-1 juin | Pipeline médaillon — **ComparIA** (Tier 1, C02) | 1ʳᵉ source officielle ingérée | ✅ |
| S3 | 2-8 juin | Pipeline médaillon — **RTE IRIS** (Tier 1, C03) | 2ᵉ source officielle (territoriale) ingérée | ✅ |
| S4 | 9-15 juin | Assemblage Gold (referentiel.sqlite + analytics.parquet, C04) | Pipeline complet Copper→Silver→Gold | ✅ |
| S5 | 16-22 juin | Estimateur Rust + presets + validation (C05-C07) + audit ledger (C08) | Cœur scientifique fonctionnel + traçabilité ACID | ✅ |
| S6 | 23-29 juin | Tauri runtime + écran M1 Estimer + Journal d'audit M7 (C09) | App native lance, estime, journalise | 🔜 |
| S7 | 30 juin-6 juillet | **C10 Onboarding personas + gating** + **C11 M13 Simulateur "Et si...?"** | Wizard 4 étapes + 7 leviers temps réel | |
| S8 | 7-13 juillet | **C12 M12 Datacenters Europe** (Leaflet, drill-down) + **C13 M20 Territoire FR** (Sankey + IRIS) | Cartographie complète + dimension territoriale | |
| S9 | 14-20 juillet | **C14 M16 Forecaster 12 mois** + **C15 M15 Dashboard personnel** + **M9 Référentiel modèles** | Visu temporelle + vue personnelle | |
| S10 | 21-27 juillet | **C16 Reporting bundle** (M5 Rapports + M18 Batch + M22 Rapport CSRD) | PDF signé + JSON-LD PROV-O | |
| S11 | 28 juillet-3 août | **C17 Workflow bundle** (M2 Workbench + M3 Comparer + M10 Import + M17 Empreinte projet + M19 Équipe + M21 Alertes) | Modules pros + chercheurs | |
| S12 | 4-10 août | **C18 Pédagogie & spécifiques** (M6 Géoloc + M8 Méthodologie + M14 À propos + M23 Marchés publics + M24 Apprendre + M25 Objectifs) | Bundle pédagogique + collectivités | |
| S13 | 11-17 août | **M11 Extension navigateur MV3** + tests Playwright e2e | Capture vie réelle Chrome/Firefox | |
| S14 | 18-24 août | Notebook Quarto + rapport méthodologique + a11y RGAA AA | Reproductibilité scientifique | |
| S15 | 25-31 août | Tests utilisateurs (5 entretiens) + correctifs P0/P1 + builds multi-cibles | Retours intégrés, builds signés | |
| S16 | 1-7 septembre | Soumission : vidéo + dépôt data.gouv.fr + extension stores | Livré | |

---

## Sprint S0 — Revue bibliographique (semaine 1)

**Objectif** : maîtriser le terrain scientifique avant d'écrire la moindre ligne de code.

**Livrables** :
- `research/biblio/synthese-bibliographique.md` (10-15 p.)
- `research/biblio/references.bib` (BibTeX, ≥ 30 entrées)
- `docs/methodology/AFNOR-SPEC-2314-synthese.md` (3-5 p.)
- Liste figée des 3 études à reproduire (validation croisée)
- Identification des distributions d'incertitude par paramètre

**Definition of Done** :
- [ ] ≥ 30 références ajoutées au `.bib` et lues (résumés synthétiques)
- [ ] Formule de référence justifiée paramètre par paramètre
- [ ] Risques méthodologiques cartographiés
- [ ] Validation par mentor Ecolab/ADEME planifiée

---

## Sprint S1 — Bootstrap technique (semaine 2)

**Objectif** : repo prêt, CI verte, schémas DB drafts, équipe outillée.

**Livrables** :
- Repo GitHub initialisé (workspace Cargo + frontend SvelteKit + extension TS)
- `scripts/bootstrap.sh` qui installe toutes les deps
- CI GitHub Actions : lint + tests + build matrix (3 OS)
- Pre-commit hooks (rustfmt, clippy, eslint, prettier)
- Schémas SQLite v0 (référentiel + audit) commités
- Schémas JSON Silver v0 commités
- `dvc.yaml` initialisé avec les 3 stages (copper, silver, gold)
- Tous les ADR créés sont mergés
- README bilingue (FR + EN)
- CHANGELOG.md initialisé

**Definition of Done** :
- [ ] `cargo build --workspace` passe
- [ ] `cd web && npm run build` passe
- [ ] CI verte sur main
- [ ] `./scripts/bootstrap.sh` testé sur les 3 OS (au moins Linux et Windows)

---

## Sprint S2 — Pipeline médaillon : ComparIA + RTE IRIS (Tier 1 défi) (semaine 3)

**Objectif** : valider le pattern médaillon sur les **2 datasets officiels du défi data.gouv.fr**. C'est notre socle.

**Livrables** :
- Crate `sobria-ingest` : trait `DataLayer`, registry, runner
- Source S01 : **ComparIA** (3 fichiers Parquet, 5 GB) — conversations, votes, réactions
- Source S02 : **RTE/NaTran/Teréga IRIS** (CSV + GeoJSON + Shapefile, ~200 MB) — consommation industrielle par maille IRIS
- Couches Copper, Silver, Gold opérationnelles pour ces 2 sources
- Schémas Silver versionnés (`comparia_*-v1.json`, `iris_*-v1.json`)
- Tests : `proptest` + golden files
- Première CI nocturne réussie + DVC remote configuré (5 GB)

**Definition of Done** :
- [ ] `cargo run -p sobria-ingest -- pipeline run --source comparia` produit un Gold valide
- [ ] `cargo run -p sobria-ingest -- pipeline run --source rte-iris` produit un Gold valide
- [ ] Lineage complet (chaque ligne Silver pointe vers un hash Copper)
- [ ] Schémas Silver v1 figés et versionnés
- [ ] Couverture tests crate ingest ≥ 75 %
- [ ] `dvc repro` reproduit à l'identique
- [ ] Premier croisement ComparIA × IRIS visualisé (preuve de concept M12)

---

## Sprint S3 — Pipeline médaillon : Tier 2 (semaine 4)

**Objectif** : compléter le référentiel avec les sources complémentaires (toutes sans authentification).

**Livrables** :
- Source S03 : ADEME Base Empreinte (facteurs d'émission élec + hardware)
- Source S04 : GenAI Impact / EcoLogits (méthodologie officielle + catalogue modèles)
- Source S05 : Hugging Face AI Energy Score
- Source S06 : CodeCarbon (mesures d'entraînement)
- Source S07 : ML.Energy Leaderboard (benchmarks inférence)
- Source S08 : Papers académiques (extraction manuelle assistée S0)
- (Optionnel) Source S09 : RTE eco2mix live si clé obtenue
- Gold final : `referentiel.sqlite` indexé FTS5 + `analytics.parquet`
- Datasheet Gebru et al. complétée

**Definition of Done** :
- [ ] 6+ sources Tier 2 intégrées et validées
- [ ] Référentiel Gold contient ≥ 50 modèles LLM, ≥ 30 datacenters, ≥ 20 facteurs d'émission
- [ ] Datasheet.jsonld signée GPG
- [ ] Premier dataset publié en preview sur data.gouv.fr (privé)

---

## Sprint S4 — Estimateur pt.1 (semaine 5)

**Objectif** : moteur de calcul fonctionnel avec propagation Monte-Carlo.

**Livrables** :
- Crate `sobria-estimator` : formule AFNOR SPEC 2314
- Monte-Carlo N=10⁴ avec seed déterministe
- Distributions par défaut (log-normale, uniforme, normale)
- API : `estimate(prompt, model, datacenter) -> EstimationResult`
- Tests : reproduction Luccioni 2023 (±15 %)
- Bench `criterion` : < 50 ms par estimation cible

**Definition of Done** :
- [ ] Couverture tests ≥ 85 %
- [ ] Test de reproduction Luccioni 2023 passe à ±15 %
- [ ] Bench moyen < 50 ms en release sur CI standard
- [ ] Documentation `cargo doc` complète sur l'API publique

---

## Sprint S5 — Estimateur pt.2 + audit ledger (semaine 6)

**Objectif** : validation scientifique complète + traçabilité ACID.

**Livrables** :
- Test de reproduction Patterson 2021
- Test de reproduction EcoLogits 2024
- Crate `sobria-audit` : ledger SQLite WAL chaîné SHA-256
- API : `journal_estimation(estimation_result) -> AuditEntry`
- Vérification d'intégrité du ledger
- Export NDJSON signé

**Definition of Done** :
- [ ] 3 études de référence reproduites à ±15 %
- [ ] Ledger reste intègre après 10 000 insertions (test)
- [ ] Documentation méthodologique mise à jour

---

## Sprint S6 — UI MVP pt.1 + géolocalisation M9 (semaine 7)

**Objectif** : première app installable + détection datacenter.

**Livrables** :
- Shell Tauri + SvelteKit configuré
- Écran "Estimer un prompt" fonctionnel
- IPC typée entre frontend et backend
- Crate `sobria-geoloc` : détection IP → zone via GeoLite2 embarquée
- Mapping provider → datacenter probable (heuristique documentée)
- Override manuel utilisateur possible
- Premier rendu visuel avec Skeleton CSS custom

**Definition of Done** :
- [ ] App installable sur Win/Mac/Linux
- [ ] Lancement à froid < 1 s en debug
- [ ] Estimation visible en < 500 ms perçus
- [ ] Géoloc fonctionne offline

---

## Sprint S7 — UI MVP pt.2 + import logs M10 (semaine 8)

**Objectif** : workbench + comparateur + import entreprise.

**Livrables** :
- Module M3 — Workbench (filtres, recherche, tri, fiches modèles)
- Module M5 — Comparateur (matrice + score composite)
- Module M10 — Import CSV/JSONL avec mappeur de colonnes interactif
- Visualisations : heatmap, barres, ridge plot (Observable Plot)
- Tests Playwright sur les parcours principaux

**Definition of Done** :
- [ ] Import CSV de 10 000 lignes traité en < 2 s
- [ ] Comparateur supporte 2 à 8 modèles
- [ ] Workbench filtrable sur ≥ 5 critères
- [ ] Tests e2e couvrent les 3 parcours principaux

---

## Sprint S8 — Simulateur M4 + extension navigateur M11 (semaine 9)

**Objectif** : scénarios macro + capture vie réelle.

**Livrables** :
- Module M4 — Simulateur (population, taux, fréquence, période, projections)
- Module M11 — Extension MV3 Chrome + Firefox
- Content scripts : ChatGPT, Claude.ai, Mistral, Gemini, Le Chat
- Comptage tokens local (tiktoken-wasm)
- Badge UI overlay
- Background service worker + IndexedDB
- Bridge optionnel vers l'app desktop (localhost token-protégé)

**Definition of Done** :
- [ ] Scénario à 5 ans / 10⁷ pop projeté en < 1 s
- [ ] Extension détecte 5 interfaces LLM
- [ ] Badge live actualisé < 200 ms après envoi prompt
- [ ] Extension passe la review locale Chrome + Firefox

---

## Sprint S9 — Notebook Quarto + rapport (semaine 10)

**Objectif** : reproductibilité scientifique de bout en bout.

**Livrables** :
- Notebook Quarto `notebook/validation.qmd` reproductible
- Rapport méthodologique 30-40 p. (FR + EN) en .qmd
- Génération PDF automatisée
- Policy brief 4 p. (FR)
- Datasheet Gebru complétée et publiée

**Definition of Done** :
- [ ] `quarto render notebook/` produit HTML + PDF identiques en CI
- [ ] Notebook re-exécute le pipeline complet sans warning
- [ ] Rapport bilingue FR/EN finalisé
- [ ] Relecture méthodologique mentor planifiée

---

## Sprint S10 — Exports + multi-cibles + a11y (semaine 11)

**Objectif** : finitions multi-plateforme.

**Livrables** :
- Crate `sobria-export` : PDF, CSV, Parquet, JSON-LD
- Builds Android + iOS (Tauri 2 mobile)
- Build Wasm (démo web)
- Audit RGAA AA passé (axe-core CI)
- Performance : binaire desktop < 20 Mo confirmé
- Vidéo démo brouillon

**Definition of Done** :
- [ ] Toutes les cibles produites par `./scripts/build-all.sh`
- [ ] RGAA AA validé sur 100 % des écrans
- [ ] Taille binaire desktop confirmée < 20 Mo
- [ ] Extension < 500 Ko confirmée

---

## Sprint S11 — Tests utilisateurs (semaine 12)

**Objectif** : 5 entretiens conduits, retours intégrés.

**Livrables** :
- 5 entretiens (1 par persona P1-P5)
- Grille d'entretien standardisée
- Synthèse des retours
- Priorisation correctifs (P0 = bloquant)
- Implémentation correctifs P0
- Implémentation correctifs P1 si temps

**Definition of Done** :
- [ ] 5 entretiens conduits et synthétisés
- [ ] 100 % des P0 corrigés
- [ ] Document de synthèse publié

---

## Sprint S12 — Soumission (semaine 13)

**Objectif** : livraison défi data.gouv.fr.

**Livrables** :
- Vidéo démo 3-5 min sous-titrée FR + EN
- Dataset publié sur data.gouv.fr
- App publiée sur GitHub Releases (binaires signés)
- Extension publiée (Chrome Web Store + Firefox Add-ons en review)
- Site statique (mdBook) déployé
- Annonce publique (blog post, LinkedIn, X)
- Candidature défi soumise

**Definition of Done** :
- [ ] Tous les livrables L1 à L10 publiés
- [ ] Candidature soumise avant la deadline
- [ ] Communication relayée

---

## Buffers et politique de retard

- **Buffer hebdomadaire** : viser 4 jours de travail effectif sur 5 (1 jour de buffer).
- **Si retard < 1 jour** : on absorbe dans le buffer.
- **Si retard 1-3 jours** : on déprioritise un sous-livrable du sprint suivant (jamais un bloquant).
- **Si retard > 3 jours** : on déclenche une revue de scope. Le mobile (Tauri 2 mobile) est le premier sacrifice.

## Politique de scope freeze

À partir de **S14 inclus**, plus aucune feature nouvelle. Uniquement : bug fixes, polish, doc, a11y. Toute idée nouvelle va dans `BACKLOG-v2.md`.

---

## Index des chantiers (C01 → C18)

| Chantier | Brief | Modules livrés | Statut |
|---|---|---|:--:|
| C01 | Foundation pipeline médaillon | (trait DataLayer, registry) | ✅ |
| C02 | Source ComparIA | (M9 partiel — référentiel modèles) | ✅ |
| C03 | Source RTE IRIS | (M20 partiel — données territoire) | ✅ |
| C04 | Gold assembly | (M1 référentiel SQLite + analytics.parquet) | ✅ |
| C05 | Estimateur Monte-Carlo | (M1 cœur, M2 partiel) | ✅ |
| C06 | Model presets calibration | (M9 fiches modèles) | ✅ |
| C07 | Validation croisée | (M8 méthodologie partielle) | ✅ |
| C08 | Audit ledger | (M7 Journal d'audit complet backend) | ✅ |
| C09 | Tauri intégration + écran Estimer | (M1 UI + M7 UI Journal) | 🔜 |
| **C10** | **Onboarding personas + gating** | (infrastructure pour tous) | |
| **C11** | **M13 Simulateur « Et si...? »** | M13 complet | |
| **C12** | **M12 Datacenters Europe** | M12 complet | |
| **C13** | **M20 Territoire FR + Sankey** | M20 complet | |
| **C14** | **M16 Forecaster 12 mois** | M16 complet | |
| **C15** | **M15 Dashboard personnel** | M15 complet (+ M9 polish) | |
| **C16** | **Reporting bundle** | M5 + M18 + M22 | |
| **C17** | **Workflow bundle** | M2 + M3 + M10 + M17 + M19 + M21 | |
| **C18** | **Pédagogie & spécifiques** | M6 + M8 + M14 + M23 + M24 + M25 | |

---

## Découpage de travail Cowork ↔ Claude Code

Cowork pilote :
- briefs (`briefs/chantiers/Cxx-*.md`), ADRs, CDC, ROADMAP, méthodologie,
- chantiers Rust pure (sobria-core, estimator, ingest, audit),
- types IPC et DTO (frontière contractuelle),
- prompts `Cxx-PROMPT-CLAUDE-CODE.md` pour transmettre les missions frontend.

Claude Code pilote :
- frontend `web/` (SvelteKit + composants + dataviz),
- intégration design depuis Claude Design,
- tests Playwright,
- extension navigateur (M11) au S13.

À chaque chantier C-bundle (C16/C17/C18), 2 commits :
- 1 commit Cowork : briefs + types Rust + tests Rust + DTO IPC,
- 1 commit Claude Code : routes + composants + tests Playwright.

---

## Risques majeurs identifiés

| Risque | Probabilité | Impact | Parade |
|---|---|---|---|
| Scope 25 modules ingérable en 11 sprints (S6-S16) | Élevé | Critique | Bundles thématiques C16-C18 (6 modules / sprint via factorisation composants) |
| Tests utilisateurs S15 trop tard | Moyen | Élevé | Tests intermédiaires sur étudiants accessibles dès S10 |
| Extension navigateur MV3 (M11) repoussée à S13 | Faible | Moyen | Possible de switcher S11/S13 si UI prête |
| Deadline data.gouv.fr antérieure à début septembre | Inconnue | Critique | À confirmer côté Thibault — si oui, on cuts M17/M19/M23 vers v1.1 |
