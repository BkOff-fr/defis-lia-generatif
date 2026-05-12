# Roadmap Sobr.ia — 12 semaines

> **Période cible** : S0 démarrage 12 mai 2026 → soumission S12 fin juillet 2026.
> **Cadence** : 1 sprint = 1 semaine. Revue chaque vendredi.
> **Format des sprints** : brief détaillé dans `briefs/sprints/Sx-*.md`.

## Vue calendaire

| Sprint | Dates | Thème | Bloquants levés |
|--------|-------|-------|-----------------|
| S0 | 12-18 mai | Revue bibliographique + cadrage scientifique | Méthodologie défendable |
| S1 | 19-25 mai | Bootstrap technique + schémas | Repo, CI, ADR figés |
| S2 | 26 mai-1 juin | Pipeline médaillon — sources ADEME + RTE | Copper/Silver/Gold opérationnels |
| S3 | 2-8 juin | Pipeline médaillon — sources HF + EcoLogits + papers | Référentiel complet en Gold |
| S4 | 9-15 juin | Estimateur Rust pt.1 (formule + Monte-Carlo) | Cœur scientifique fonctionnel |
| S5 | 16-22 juin | Estimateur Rust pt.2 + audit ledger | Validation croisée passée |
| S6 | 23-29 juin | UI MVP pt.1 + module géolocalisation M9 | App lance + estime + géolocalise |
| S7 | 30 juin-6 juillet | UI MVP pt.2 + import logs M10 | Workbench + comparateur + CSV |
| S8 | 7-13 juillet | Simulateur scénarios M4 + extension navigateur M11 | Scénarios + badge live |
| S9 | 14-20 juillet | Notebook Quarto + rapport méthodologique | Reproductibilité scientifique |
| S10 | 21-27 juillet | Exports M6 + builds mobile/Wasm + a11y | Multi-cibles + a11y AA |
| S11 | 28 juillet-3 août | Tests utilisateurs + itération UX | 5 entretiens conduits, retours intégrés |
| S12 | 4-10 août | Soumission : vidéo + dépôt data.gouv.fr | Livré |

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

## Sprint S2 — Pipeline médaillon : ADEME + RTE (semaine 3)

**Objectif** : valider le pattern médaillon sur les 2 premières sources les plus importantes.

**Livrables** :
- Crate `sobria-ingest` : trait `DataLayer`, registry, runner
- Source 1 : ADEME Base Empreinte (facteurs d'émission électricité + hardware)
- Source 2 : RTE eco2mix (mix électrique français)
- Couches Copper, Silver, Gold opérationnelles pour ces 2 sources
- Tests : `proptest` + golden files
- Première CI nocturne réussie

**Definition of Done** :
- [ ] `cargo run -p sobria-ingest -- pipeline run` produit un Gold valide
- [ ] Lineage complet (chaque ligne Silver pointe vers un hash Copper)
- [ ] Schémas Silver v1 figés et versionnés
- [ ] Couverture tests crate ingest ≥ 75 %
- [ ] `dvc repro` reproduit à l'identique

---

## Sprint S3 — Pipeline médaillon : HF + EcoLogits + papers (semaine 4)

**Objectif** : compléter le référentiel avec les sources LLM-spécifiques.

**Livrables** :
- Source 3 : Hugging Face AI Energy Score
- Source 4 : GenAI Impact / EcoLogits (modèles caractéristiques)
- Source 5 : CodeCarbon (mesures d'entraînement)
- Source 6 : ML.Energy Leaderboard (benchmarks inférence)
- Source 7 : Papers académiques (extraction manuelle assistée)
- Source 8 : GeoLite2 (IP → zone)
- Gold final : `referentiel.sqlite` indexé FTS5 + `analytics.parquet`
- Datasheet Gebru et al. complétée

**Definition of Done** :
- [ ] 8 sources intégrées et validées
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

À partir de **S9 inclus**, plus aucune feature nouvelle. Uniquement : bug fixes, polish, doc, a11y. Toute idée nouvelle va dans `BACKLOG-v2.md`.
