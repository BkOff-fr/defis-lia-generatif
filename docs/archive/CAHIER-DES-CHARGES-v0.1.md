# Cahier des charges — FrugalMeter

> **Version** : 0.1 (brouillon de travail)
> **Date** : 12 mai 2026
> **Auteur** : Thibault (étudiant, candidat au défi data.gouv.fr)
> **Défi** : « L'impact environnemental de l'IA générative » — defis.data.gouv.fr
> **Statut** : Document de discussion, à itérer avant gel en v1.0

---

## 1. Contexte et enjeux

L'IA générative connaît une croissance exponentielle depuis 2022, avec une explosion concomitante de la demande en énergie, en eau de refroidissement et en hardware spécialisé (GPU, ASIC). Les ordres de grandeur publics restent flous, fragmentés, parfois contradictoires :

- L'ADEME a publié plusieurs études sur l'empreinte du numérique (2,5 % des émissions françaises en 2020) sans intégrer pleinement la rupture GenAI.
- Le **référentiel général pour l'IA frugale (AFNOR SPEC 2314)**, piloté par l'Ecolab du CGDD, fournit la méthodologie cadre mais reste peu appliqué publiquement.
- **GenAI Impact (Data for Good)** maintient `EcoLogits`, une bibliothèque Python pour estimer les requêtes LLM, mais l'outil reste cantonné aux développeurs.
- **Hugging Face / Salesforce** publient l'AI Energy Score, **CodeCarbon** trace les entraînements, **ML.Energy** benchmarke les inférences — chacun avec ses biais et hypothèses.
- La **feuille de route Numérique & IA** (ministères Transition écologique, septembre 2025) appelle à une transparence accrue et à des indicateurs comparables.

Le constat est qu'il manque **moins de cadres méthodologiques que d'outils accessibles, de données consolidées, et de représentations grand public**. Ce projet vise à combler ce vide.

---

## 2. Objectifs

### 2.1 Objectif général

Produire **une stack complète et open-source** (dataset + outil + méthodologie) permettant à un public non-expert d'évaluer, comparer et communiquer l'impact environnemental d'usages réels d'IA générative à base de LLMs, conformément au référentiel AFNOR SPEC 2314 et aux facteurs d'émission ADEME.

### 2.2 Objectifs spécifiques

1. **Consolider** un dataset ouvert publiable sur data.gouv.fr (modèles LLM, consommations mesurées, mix électriques, facteurs ADEME, eau, hardware).
2. **Outiller** les décideurs et le grand public avec une application desktop/web/mobile (Tauri 2) chiffrant l'impact d'un usage ou d'un scénario.
3. **Documenter** la méthodologie avec un notebook reproductible et un rapport scientifique défendable.
4. **Démontrer** par l'exemple la cohérence d'une stack frugale (Rust + Tauri + SvelteKit) face aux outils SaaS classiques.

### 2.3 Non-objectifs (hors périmètre v1.0)

- Génération d'images, vidéo, audio (à traiter en v2.0).
- Mesure physique en temps réel sur des GPU (instrumentation hardware, RAPL).
- Optimisation des modèles (compression, quantization).
- Recommandations juridiques (RGPD, AI Act) hors angle environnemental.

---

## 3. Personas et cas d'usage

### 3.1 Persona P1 — Claire, chargée RSE en entreprise (35 ans)

> « Je dois justifier au comex pourquoi notre usage de Copilot augmente notre bilan carbone scope 3. J'ai besoin de chiffres défendables, vite. »

**Cas d'usage** : Importe un journal d'usage anonymisé (nb requêtes/mois, types de prompts), obtient un rapport PDF avec CO₂eq / eau / énergie, comparaisons sectorielles, et recommandations d'arbitrage modèle.

### 3.2 Persona P2 — Marc, agent de l'Ecolab / ADEME (42 ans)

> « Je veux pouvoir simuler des scénarios à l'échelle nationale : que se passe-t-il si 30 % des fonctionnaires utilisent un LLM 10 fois par jour ? »

**Cas d'usage** : Configure un scénario macro (taux d'adoption, fréquence, modèle utilisé, mix électrique), visualise la projection 2026-2030, exporte les hypothèses.

### 3.3 Persona P3 — Léa, étudiante en data journalisme (24 ans)

> « Je prépare un article. J'ai besoin de visualisations claires et de chiffres sourcés que je puisse citer. »

**Cas d'usage** : Explore le dataset, génère des graphiques exportables (PNG/SVG), récupère les sources et hypothèses associées.

### 3.4 Persona P4 — Thomas, dev intégrant un LLM dans son SaaS (29 ans)

> « Je veux estimer avant déploiement le coût environnemental annuel d'intégrer Mistral Large dans mon produit, et le comparer à GPT-4o-mini. »

**Cas d'usage** : Saisit un trafic estimé, un panel de modèles candidats, obtient une matrice comparative et une recommandation argumentée.

---

## 4. Périmètre fonctionnel

### 4.1 Modules de l'application

| ID | Module | Description |
|----|--------|-------------|
| M1 | **Référentiel** | Base SQLite versionnée des modèles, hardware, datacenters, facteurs d'émission |
| M2 | **Estimateur** | Moteur de calcul Rust pour un prompt unitaire (tokens entrée/sortie, modèle, datacenter) |
| M3 | **Workbench** | Exploration interactive du référentiel, filtres, tris, recherche |
| M4 | **Simulateur de scénarios** | Construction de scénarios organisationnels ou macro, projections temporelles |
| M5 | **Comparateur** | Matrice modèles × indicateurs (CO₂eq, eau, énergie, latence, coût) |
| M6 | **Rapports & exports** | Génération PDF (rapport synthétique), CSV/Parquet (données), JSON-LD (audit), Quarto (notebook reproductible) |
| M7 | **Audit ledger** | Journal ACID immuable de toutes les estimations effectuées (traçabilité réglementaire) |
| M8 | **Aide & méthodologie** | Documentation embarquée, glossaire, références AFNOR SPEC 2314 |

### 4.2 Indicateurs calculés

Pour chaque estimation, l'outil produit (avec intervalles d'incertitude propagés par Monte-Carlo) :

- **CO₂eq** (gCO₂eq) — émissions opérationnelles + embarquées (amorties)
- **Énergie** (Wh) — décomposée en compute, idle, networking, cooling (PUE)
- **Eau** (L) — WUE direct (refroidissement) + indirect (production électrique)
- **Métaux critiques** (mg équivalent terre rare) — proxy à partir de l'embodied hardware
- **Coût** (€) — coût utilisateur facturé, pour mise en perspective économique
- **Équivalents parlants** — km voiture, douches, écrans-heures, etc.

---

## 5. Exigences fonctionnelles (extraits)

### 5.1 EF-01 — Estimation unitaire d'un prompt

**Description** : L'utilisateur saisit un prompt (texte ou nb tokens estimés), choisit un modèle dans la liste référentielle, optionnellement un datacenter / mix électrique.
**Acceptance criteria** :
- AC1 : Le calcul retourne CO₂eq, énergie, eau en < 200 ms (cible : < 50 ms).
- AC2 : Les hypothèses utilisées sont affichées et clic-cliquables vers la source.
- AC3 : Les intervalles d'incertitude (P5/P95) sont affichés.
- AC4 : Le résultat est journalisé en SQLite avec horodatage et signature SHA-256.

### 5.2 EF-02 — Simulation de scénario macro

**Description** : L'utilisateur configure une population (taille, taux d'adoption), un usage moyen (req/jour, longueur), un modèle, une période (mois ou années).
**Acceptance criteria** :
- AC1 : Projection temporelle générée en < 1 s pour un scénario à 5 ans / population 10⁷.
- AC2 : Visualisation aire/courbe avec bande d'incertitude.
- AC3 : Export du scénario (JSON) reproductible.

### 5.3 EF-03 — Comparaison multi-modèles

**Description** : Sélection de 2 à 8 modèles, affichage côte à côte avec normalisation possible.
**Acceptance criteria** :
- AC1 : Matrice triable par chaque indicateur.
- AC2 : Score composite paramétrable (poids ajustables).
- AC3 : Indication des données manquantes ou estimées par proxy.

*(Liste complète des EF à compléter en v0.2 — env. 20 à 25 EF visées)*

---

## 6. Exigences non-fonctionnelles

| ID | Catégorie | Exigence | Cible |
|----|-----------|----------|-------|
| NF-01 | Performance | Temps de lancement à froid | < 800 ms |
| NF-02 | Performance | Empreinte RAM moyenne | < 100 Mo |
| NF-03 | Performance | Taille binaire desktop | < 20 Mo |
| NF-04 | Frugalité | Empreinte CO₂eq par session de 30 min | mesurée et publiée |
| NF-05 | Robustesse | Couverture de tests Rust | ≥ 80 % |
| NF-06 | Robustesse | Audit ledger ACID intègre | 100 % (SQLite WAL) |
| NF-07 | Sécurité | Pas de télémétrie sans opt-in | obligatoire |
| NF-08 | Sécurité | Communications HTTPS uniquement, sources signées | obligatoire |
| NF-09 | Accessibilité | Conformité RGAA 4.1 niveau AA | obligatoire |
| NF-10 | i18n | Langues | FR + EN |
| NF-11 | Reproductibilité | Builds déterministes | obligatoire |
| NF-12 | Open source | Licence | MIT (code) + Etalab 2.0 (données) |
| NF-13 | Documentation | Doc utilisateur + dev complète | obligatoire |
| NF-14 | Multi-plateforme | Cibles supportées | Windows / macOS / Linux / Web (Wasm) / Android / iOS |

---

## 7. Architecture technique

### 7.1 Vue d'ensemble

```
┌─────────────────────────────────────────────────────────┐
│  Tauri 2.x — wrapper natif (desktop/mobile/web)         │
│                                                          │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Frontend : SvelteKit + TypeScript                 │ │
│  │  ├─ stores réactifs (estimations, scénarios)       │ │
│  │  ├─ dataviz : Observable Plot + D3 (Sankey, etc.)  │ │
│  │  ├─ UI : skeleton CSS custom (pas de framework UI  │ │
│  │  │       lourd, cohérence frugale)                 │ │
│  │  └─ a11y : RGAA AA + lecteurs d'écran              │ │
│  └────────────────┬───────────────────────────────────┘ │
│                   │ IPC Tauri (commandes typées)         │
│  ┌────────────────▼───────────────────────────────────┐ │
│  │  Cœur Rust (workspace cargo, ≥ 6 crates)           │ │
│  │  ├─ frugalmeter-core      : types, traits, errors  │ │
│  │  ├─ frugalmeter-referentiel : accès SQLite + cache │ │
│  │  ├─ frugalmeter-estimator : moteur AFNOR SPEC 2314 │ │
│  │  ├─ frugalmeter-ingest    : ETL sources externes   │ │
│  │  ├─ frugalmeter-export    : PDF, Quarto, JSON-LD   │ │
│  │  ├─ frugalmeter-audit     : ledger immuable signé  │ │
│  │  └─ frugalmeter-app       : commandes Tauri        │ │
│  └─────────┬──────────────────────────────────────────┘ │
│            │                                             │
│   ┌────────┴────────────┐                                │
│   ▼                     ▼                                │
│ SQLite (rusqlite)   DuckDB (duckdb-rs)                   │
│ • référentiel       • requêtes analytiques               │
│ • audit ledger      • agrégations scénarios              │
│ • WAL + signatures  • lecture parquet                    │
└──────────────────────────────────────────────────────────┘
                          │
                          ▼ (CI nocturne)
┌──────────────────────────────────────────────────────────┐
│ Pipeline GitHub Actions                                   │
│ • ingestion sources externes (ADEME, RTE, HF, papers)    │
│ • validation, dédup, normalisation                       │
│ • génération SQLite + Parquet versionnés (DVC)           │
│ • publication release GitHub + dataset data.gouv.fr      │
└──────────────────────────────────────────────────────────┘
```

### 7.2 Choix techniques justifiés

| Choix | Justification |
|-------|---------------|
| **Rust** | Performance, sûreté mémoire, binaire compact (cohérence frugale) |
| **Tauri 2** | Native multi-plateforme (desktop + mobile + web via Wasm) avec backend Rust |
| **SvelteKit** | Compilation à la build, runtime minimal, idéal dataviz fluide |
| **SQLite (WAL)** | ACID natif, embarqué, format universel, parfait pour référentiel et audit |
| **DuckDB** | OLAP embarqué pour scénarios macro, lecture native Parquet |
| **Observable Plot + D3** | Dataviz grammaire-of-graphics + customisation fine |
| **Quarto** | Notebook reproductible bilingue (FR/EN), export PDF/HTML/Word |
| **DVC** | Versionnage des données massives (référentiel évolue) |
| **MIT + Etalab 2.0** | Compatibilité écosystème français (data.gouv.fr) et international |

---

## 8. Sources de données identifiées

### 8.1 Sources primaires (intégrées au référentiel)

| Source | Données | Format | Licence | Fréquence MAJ |
|--------|---------|--------|---------|---------------|
| **ADEME Base Empreinte** | Facteurs d'émission électricité, hardware | API REST + CSV | Etalab 2.0 | Trimestrielle |
| **RTE eco2mix** | Mix électrique français temps réel | API JSON | Etalab 2.0 | Temps réel |
| **Electricity Maps** | Mix électrique mondial | API REST | CC-BY-SA | Horaire |
| **GenAI Impact / EcoLogits** | Modèles LLM caractéristiques | Python lib + GitHub JSON | MIT | Ad hoc |
| **Hugging Face AI Energy Score** | Score énergétique modèles | API | Apache 2.0 | Ad hoc |
| **CodeCarbon** | Mesures d'entraînement | GitHub JSON | MIT | Ad hoc |
| **ML.Energy Leaderboard** | Benchmarks inférence LLM | Web scraping/CSV | CC-BY | Mensuelle |
| **Papers académiques** | Mesures de référence (Patterson, Luccioni, etc.) | PDF / supplementary | varies | Ad hoc |
| **Datasheets GPU** | TDP, embodied carbon | PDF | constructeur | Ad hoc |

### 8.2 Sources méthodologiques (référence, non intégrées)

- AFNOR SPEC 2314 — référentiel général pour l'IA frugale
- ISO/IEC 21031:2024 — méthodologie d'évaluation environnementale ICT
- ITU-T L.1410 — méthodologie LCA pour les TIC
- GHG Protocol — scope 3 catégorie 1 (achats)

---

## 9. Méthodologie de calcul (synthèse)

### 9.1 Formule de référence (estimation inférence)

```
CO₂eq(prompt) =
  [ E_compute × PUE × IF_électrique
  + E_embodied / N_amortissement ]
  + propagation d'incertitude (Monte-Carlo, N=10⁴)

avec :
  E_compute     = (T_in × ε_prefill + T_out × ε_decode) × η_modèle
  PUE           = ratio datacenter (par défaut 1.2-1.6 selon zone)
  IF_électrique = facteur émission temps réel (RTE, Electricity Maps)
  E_embodied    = embodied carbon hardware / nb requêtes amorties
```

### 9.2 Propagation d'incertitude

Chaque paramètre est représenté par une distribution (uniforme, normale, log-normale selon nature). Le moteur fait tourner **10 000 simulations Monte-Carlo** par estimation et restitue P5, P50, P95.

### 9.3 Validation croisée

Le moteur est validé par :
- Reproduction de 3 études de référence (Luccioni 2023, Patterson 2021, EcoLogits 2024).
- Comparaison aux mesures CodeCarbon publiées (≥ 30 cas de test).
- Revue par un expert ADEME / Ecolab (si possible).

---

## 10. Livrables

| ID | Livrable | Format | Public |
|----|----------|--------|--------|
| L1 | Application FrugalMeter | Binaires Win/Mac/Linux + Android/iOS + Wasm | Grand public |
| L2 | Dataset consolidé | SQLite + Parquet + CSV (versionnés) | data.gouv.fr |
| L3 | Notebook de validation | Quarto (.qmd → HTML + PDF) | Communauté scientifique |
| L4 | Rapport méthodologique | PDF 30-40 p. FR + EN | ADEME / Ecolab / jury |
| L5 | Note de policy brief | PDF 4 p. FR | Décideurs publics |
| L6 | Code source | GitHub (workspace Cargo + SvelteKit) | Développeurs |
| L7 | Documentation | mdBook + site statique | Tous |
| L8 | Vidéo démo | MP4 3-5 min sous-titrée | Jury, communication |

---

## 11. Roadmap détaillée (12 semaines)

| Sem. | Phase | Livrables intermédiaires | Risques surveillés |
|------|-------|--------------------------|--------------------|
| S1 | Cadrage | CDC v1.0 figé, revue biblio, schéma DB, repo init | Sous-estimation scope |
| S2 | Référentiel pt.1 | Schéma SQLite, ingest ADEME + RTE, tests unitaires | Disponibilité API |
| S3 | Référentiel pt.2 | Ingest HF + EcoLogits + papers, dédup, normalisation | Hétérogénéité formats |
| S4 | Estimateur pt.1 | Crate `estimator` + Monte-Carlo, validation papers | Erreurs méthodo |
| S5 | Estimateur pt.2 | Audit ledger ACID, exports JSON-LD, benchmarks | Performance Rust |
| S6 | UI MVP pt.1 | Shell Tauri + Svelte, écran estimation unitaire | Courbe apprentissage |
| S7 | UI MVP pt.2 | Workbench référentiel + comparateur | Dataviz complexité |
| S8 | Simulateur | Scénarios macro + projections temporelles | Lisibilité graphes |
| S9 | Méthodologie | Notebook Quarto reproductible, rapport rédigé | Temps de rédaction |
| S10 | Exports & polish | PDF, Parquet, mobile builds, Wasm, a11y | Bugs multi-plateforme |
| S11 | Tests utilisateurs | 5 entretiens (1 par persona), itération UX | Disponibilité testeurs |
| S12 | Soumission | Vidéo démo, dépôt data.gouv.fr, communication | Détails admin |

---

## 12. Gouvernance, licences, open source

- **Code** : MIT (permissive, compatible communauté Rust)
- **Données** : Etalab 2.0 (compatible data.gouv.fr et CC-BY 4.0)
- **Documentation** : CC-BY 4.0
- **Modèle de gouvernance** : repo GitHub solo initial, ouverture aux contributions via DCO (Developer Certificate of Origin) à partir de v1.0.
- **Versionnage** : SemVer pour l'app, CalVer (YYYY.MM.DD) pour le référentiel de données.
- **CI/CD** : GitHub Actions — build cross-platform, tests, mise à jour nocturne du référentiel, publication releases.

---

## 13. Critères de succès et indicateurs de réussite (KPI)

### 13.1 KPI techniques (avant jury)
- Couverture tests Rust ≥ 80 %
- Temps de calcul moyen estimation unitaire < 50 ms
- Empreinte binaire desktop < 20 Mo
- 0 vulnérabilité critique (cargo audit / npm audit)
- Conformité RGAA AA validée

### 13.2 KPI projet (post-jury)
- Dataset téléchargé ≥ 200 fois sur data.gouv.fr en 6 mois
- App téléchargée ≥ 500 fois cumulé toutes plateformes
- ≥ 1 mention dans la presse spécialisée (Next, Numerama, MIT Tech Review FR…)
- ≥ 1 retour officiel ADEME ou Ecolab

### 13.3 KPI candidature
- Cohérence du dossier (technique, méthodologie, frugalité)
- Reproductibilité de bout en bout
- Qualité visuelle et accessibilité de l'app
- Solidité de la méthodologie scientifique

---

## 14. Risques et mitigations

| Risque | P | I | Mitigation |
|--------|---|---|-----------|
| Courbe d'apprentissage Rust + Tauri 2 + SvelteKit | H | H | Démarrer par tutoriels officiels S0, prototyper S6 sans optimisation |
| Sources de données indisponibles ou changeantes | M | H | Versionner via DVC, mocks pour tests, fallback CSV statiques |
| Méthodologie contestable scientifiquement | M | H | Validation croisée stricte, transparence totale sur hypothèses |
| Scope creep (envie d'ajouter images/audio) | H | M | CDC strict, backlog v2.0, refus assumé |
| Bugs multi-plateforme (Tauri mobile encore jeune) | M | M | Prioriser desktop d'abord, mobile en bonus en S10 |
| Solo : surcharge / abandon | M | H | Sprints hebdo, jalons fermes, journal d'avancement public |
| Validation jury : critères non publics | M | M | Lire archives défis passés, contacter Ecolab si possible |

---

## 15. Questions ouvertes à trancher pour v1.0

1. **Datacenters** : intégrer ou non un module géolocalisation utilisateur → datacenter probable ?
2. **Mode entreprise** : prévoir un import CSV de logs d'usage ou laisser ça pour v2 ?
3. **Plug-in navigateur** : extension pour capturer les requêtes ChatGPT en vie réelle (souhaitable mais ambitieux) ?
4. **Modèle économique post-projet** : associatif, fondation, contribution Ecolab, abandonware ?
5. **Communication** : créer un nom définitif (FrugalMeter / IAvert / Sobr.IA / autre) et un logo ?
6. **Mentor** : démarcher un référent Ecolab/ADEME pour relecture méthodologique ?
7. **Évaluation utilisateur** : recruter 5 testeurs dès S6 ou attendre S11 ?
8. **Distribution** : Microsoft Store / Mac App Store / Flathub / Snap, ou GitHub releases seulement ?

---

## 16. Prochaines étapes (immédiates)

1. **Tu valides ou ajustes** ce CDC en commentaires / discussions.
2. On répond ensemble aux 8 questions ouvertes (§15).
3. Je rédige une **v1.0** figée avec EF complètes (≈25 items) et schéma DB détaillé.
4. On crée le repo GitHub, le squelette Tauri + Cargo workspace, et le README bilingue.
5. On démarre la phase S1 du roadmap.

---

*Document vivant — version 0.1. Toute remarque, désaccord ou ambition supplémentaire est bienvenu.*
