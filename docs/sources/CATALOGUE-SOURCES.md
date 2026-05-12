# Catalogue des sources de données — Sobr.ia

> Chaque source traverse le pipeline médaillon (ADR-0009). Cette fiche est la base de la crate `sobria-ingest`.
> Toute modification = mise à jour du schéma Silver correspondant + bump de version.

---

## S01 — ADEME Base Empreinte

| Champ | Valeur |
|-------|--------|
| **URL principale** | https://base-empreinte.ademe.fr/ |
| **API** | https://data.ademe.fr/datasets/base-empreinte |
| **Format ingéré (Copper)** | CSV + JSON |
| **Licence** | Licence Ouverte Etalab 2.0 |
| **Compatibilité Sobr.ia** | ✅ Oui (Etalab ↔ MIT/CC-BY) |
| **Fréquence MAJ amont** | Trimestrielle |
| **Politique fetch** | Hebdomadaire (CI) |
| **Volume estimé** | ~50 MB CSV |
| **Entités Silver produites** | `electricity_factors`, `hardware_factors`, `transport_factors` (pour comparaisons) |
| **Authentification** | Aucune |

**Champs critiques pour Sobr.ia** :
- Facteur d'émission électricité par pays (gCO₂eq/kWh)
- Évolution temporelle des facteurs
- Facteurs d'émission hardware (production GPU, serveurs)

**Risques** :
- Granularité géographique parfois pays uniquement (pas régional sauf FR).
- Méthodologie de calcul propre ADEME (potentielles différences vs IPCC).

**Validation à l'ingestion** :
- Schéma `schemas/silver/electricity_factors-v1.json`
- Cohérence des unités (gCO₂eq, kgCO₂eq → normalisation SI)

---

## S02 — RTE eco2mix

| Champ | Valeur |
|-------|--------|
| **URL principale** | https://www.rte-france.com/eco2mix |
| **API** | https://digital.iservices.rte-france.com/ (OAuth2) |
| **Format ingéré (Copper)** | JSON |
| **Licence** | Licence Ouverte Etalab 2.0 |
| **Compatibilité Sobr.ia** | ✅ Oui |
| **Fréquence MAJ amont** | Temps réel (toutes les 15 min) |
| **Politique fetch** | Horaire (CI + on-demand) |
| **Volume estimé** | ~200 KB/jour |
| **Entités Silver produites** | `mix_hourly_fr`, `mix_daily_fr` |
| **Authentification** | OAuth2 client credentials (clé gratuite après inscription) |

**Champs critiques** :
- Production par source (nucléaire, gaz, hydro, éolien, solaire, charbon, fioul)
- Mix horaire complet
- Facteur d'émission moyen horaire calculé

**Risques** :
- API quota : 10 req/min en niveau gratuit.
- Authentification token à renouveler (3h).

**Validation** :
- Cohérence des sommes (somme par source = total).
- Pas de trous temporels > 1h.

---

## S03 — Electricity Maps

| Champ | Valeur |
|-------|--------|
| **URL** | https://app.electricitymaps.com/ |
| **API** | https://api.electricitymap.org/ |
| **Format** | JSON |
| **Licence** | CC-BY-SA 4.0 (données publiques) / commerciale (premium) |
| **Compatibilité Sobr.ia** | ✅ Oui (niveau public) |
| **Fréquence MAJ** | Horaire |
| **Politique fetch** | Quotidienne (zones non-FR) |
| **Volume estimé** | ~5 MB/jour pour 50 zones |
| **Entités Silver** | `mix_hourly_world` |
| **Authentification** | API key gratuite (limité) |

**Champs critiques** :
- Mix électrique par zone (granularité régionale dans certains pays)
- Carbon intensity (gCO₂eq/kWh) live et forecast

**Risques** :
- Plan gratuit limité à certaines zones + historique court.
- Licence CC-BY-SA → propagation virale sur les datasets dérivés (attention).

---

## S04 — Hugging Face AI Energy Score

| Champ | Valeur |
|-------|--------|
| **URL** | https://huggingface.co/AIEnergyScore |
| **API** | HF Hub API + dataset |
| **Format** | JSON / Parquet |
| **Licence** | Apache 2.0 (modèles), CC-BY (scores) |
| **Compatibilité Sobr.ia** | ✅ Oui |
| **Fréquence MAJ** | Ad hoc (nouveaux modèles évalués) |
| **Politique fetch** | Hebdomadaire |
| **Volume estimé** | ~10 MB |
| **Entités Silver** | `model_energy_score` |
| **Authentification** | Aucune (lecture) |

**Champs critiques** :
- Score énergie par modèle (Wh par tâche standardisée)
- Métadonnées modèle (taille, architecture, modalité)
- Méthodologie de mesure

**Risques** :
- Tâches standardisées peuvent ne pas refléter usage utilisateur.
- Couverture modèles fermés limitée.

---

## S05 — GenAI Impact / EcoLogits

| Champ | Valeur |
|-------|--------|
| **URL** | https://github.com/genai-impact/ecologits |
| **API** | Python library + JSON public |
| **Format** | JSON + Python (parse AST possible) |
| **Licence** | MIT |
| **Compatibilité Sobr.ia** | ✅ Oui |
| **Fréquence MAJ** | Ad hoc (releases) |
| **Politique fetch** | Hebdomadaire (poll GitHub releases) |
| **Volume estimé** | ~2 MB |
| **Entités Silver** | `ecologits_models`, `ecologits_assumptions` |
| **Authentification** | Aucune |

**Champs critiques** :
- Catalogue modèles avec paramètres énergie/token
- Méthodologie documentée
- Distributions d'incertitude

**Risques** :
- Bibliothèque jeune, évolutions rapides.
- Certains chiffres extrapolés (transparence dans le code).

---

## S06 — CodeCarbon

| Champ | Valeur |
|-------|--------|
| **URL** | https://github.com/mlco2/codecarbon |
| **API** | GitHub + dataset CodeCarbon Hub |
| **Format** | CSV / JSON |
| **Licence** | MIT |
| **Compatibilité Sobr.ia** | ✅ Oui |
| **Fréquence MAJ** | Continue |
| **Politique fetch** | Mensuelle (le hub) |
| **Volume estimé** | ~20 MB |
| **Entités Silver** | `training_runs`, `inference_runs` |
| **Authentification** | Aucune |

**Champs critiques** :
- Mesures réelles d'entraînement (énergie, CO₂eq)
- Hardware utilisé
- Mix électrique de la zone

**Risques** :
- Hétérogénéité des soumissions (qualité variable).
- Biais : surreprésentation académique vs production.

---

## S07 — ML.Energy Leaderboard

| Champ | Valeur |
|-------|--------|
| **URL** | https://ml.energy/leaderboard |
| **API** | Web scraping + CSV publié |
| **Format** | CSV |
| **Licence** | CC-BY 4.0 |
| **Compatibilité Sobr.ia** | ✅ Oui |
| **Fréquence MAJ** | Mensuelle |
| **Politique fetch** | Mensuelle |
| **Volume estimé** | ~1 MB |
| **Entités Silver** | `inference_benchmarks` |
| **Authentification** | Aucune |

**Champs critiques** :
- Énergie par requête (Wh) par modèle
- Hardware standardisé (NVIDIA A100/H100)
- Throughput tokens/s

**Risques** :
- Modèles fermés (GPT, Claude) absents par construction.
- Hardware unique = pas de variance datacenter.

---

## S08 — Papers académiques

| Champ | Valeur |
|-------|--------|
| **Sources** | arXiv, ACL Anthology, NeurIPS, ICML, Nature, etc. |
| **Format Copper** | PDF |
| **Licence** | Variable (arXiv = libre dépôt, journaux = à vérifier) |
| **Compatibilité Sobr.ia** | ✅ Pour citations/extracts ; ❌ pour reproduction intégrale |
| **Fréquence MAJ** | Ad hoc |
| **Politique fetch** | Manuelle (curatée) |
| **Volume estimé** | ~100 MB (PDFs des papers retenus) |
| **Entités Silver** | `extracted_measures` (table de chiffres extraits manuellement) |
| **Authentification** | Selon source |

**Papers prioritaires (S0)** :
- Luccioni, Viguier, Ligozat (2023) — BLOOM carbon footprint
- Patterson et al. (2021) — Carbon emissions of NN training
- Faiz et al. (2024) — LLMCarbon
- Mytton (2021) — Data centre water
- Li, Ren et al. (2023) — Making AI Less Thirsty
- Gupta et al. (2022) — Chasing Carbon (embodied)
- Wu et al. (2022) — Sustainable AI

**Processus d'extraction** :
- PDF stocké en Copper (avec hash + URL DOI).
- Extraction manuelle dans `papers-extracts.csv` (Silver).
- Référence BibTeX dans `research/biblio/references.bib`.

**Risques** :
- Erreurs de transcription manuelle → double validation Thibault + mentor.
- Chiffres parfois en supplementary material PDF compliqué.

---

## S09 — GeoLite2 (MaxMind)

| Champ | Valeur |
|-------|--------|
| **URL** | https://dev.maxmind.com/geoip/geolite2-free-geolocation-data |
| **Format** | CSV / MMDB binaire |
| **Licence** | CC-BY-SA 4.0 |
| **Compatibilité Sobr.ia** | ✅ Oui (avec attribution obligatoire) |
| **Fréquence MAJ** | Hebdomadaire |
| **Politique fetch** | Mensuelle (suffisant pour pays/région) |
| **Volume estimé** | ~70 MB MMDB |
| **Entités Silver** | `ip_to_country`, `ip_to_region` |
| **Authentification** | Compte gratuit MaxMind (clé API) |

**Usage** : module M9 — détecter la zone de l'utilisateur ou d'une IP cible pour estimer le mix électrique probable.

**Risques** :
- CC-BY-SA = attribution obligatoire dans la doc.
- Précision région pas garantie hors UE/US.

---

## S10 — Datasheets GPU (NVIDIA, AMD, Google TPU)

| Champ | Valeur |
|-------|--------|
| **Sources** | Sites constructeurs (PDF datasheets) |
| **Format Copper** | PDF |
| **Licence** | © constructeurs (citation autorisée, redistribution non) |
| **Compatibilité Sobr.ia** | ✅ Pour citation/extraction ; PDFs non redistribués |
| **Fréquence MAJ** | À chaque release matériel |
| **Politique fetch** | Manuelle |
| **Volume Copper** | ~50 MB (datasheets pertinentes) |
| **Entités Silver** | `gpu_specs`, `gpu_embodied_carbon` |
| **Authentification** | Aucune |

**Champs critiques** :
- TDP nominal (W)
- Embodied carbon (souvent dans rapports ESG constructeur, pas datasheet)
- Performance (TFLOPS, INT8 throughput)
- Mémoire (VRAM)

**Risques** :
- Embodied carbon rarement publié, à compléter via Gupta et al. 2022 + estimations.

---

## Vue d'ensemble — matrice rapide

| ID | Source | Catégorie | Bloquant ? | Premier sprint |
|----|--------|-----------|------------|----------------|
| S01 | ADEME Base Empreinte | Émissions | Oui | S2 |
| S02 | RTE eco2mix | Mix élec FR | Oui | S2 |
| S03 | Electricity Maps | Mix élec monde | Recommandé | S3 |
| S04 | HF AI Energy Score | Modèles | Oui | S3 |
| S05 | EcoLogits | Modèles | Oui | S3 |
| S06 | CodeCarbon | Mesures | Oui | S3 |
| S07 | ML.Energy | Bench | Recommandé | S3 |
| S08 | Papers | Validation | Oui | S0 + S3 |
| S09 | GeoLite2 | Géoloc | Oui (M9) | S6 |
| S10 | Datasheets GPU | Hardware | Oui | S3 |

---

## Schémas Silver (vue d'ensemble)

```
schemas/silver/
├── electricity_factors-v1.json    (S01)
├── hardware_factors-v1.json       (S01)
├── mix_hourly_fr-v1.json          (S02)
├── mix_hourly_world-v1.json       (S03)
├── model_energy_score-v1.json     (S04)
├── ecologits_models-v1.json       (S05)
├── training_runs-v1.json          (S06)
├── inference_runs-v1.json         (S06)
├── inference_benchmarks-v1.json   (S07)
├── extracted_measures-v1.json     (S08)
├── ip_to_country-v1.json          (S09)
├── ip_to_region-v1.json           (S09)
├── gpu_specs-v1.json              (S10)
└── gpu_embodied_carbon-v1.json    (S10)
```

---

## Pré-requis avant ingestion (S2)

- [ ] Clé API RTE obtenue (Thibault)
- [ ] Clé API Electricity Maps obtenue (Thibault)
- [ ] Clé API MaxMind obtenue (Thibault)
- [ ] Stockage DVC remote configuré (R2 ou S3-compatible)
- [ ] Schémas Silver v1 figés et committés
- [ ] Tests `proptest` template prêts

---

*Ce catalogue est vivant. Toute nouvelle source = nouvelle fiche + nouvelle implémentation `DataLayer`.*
