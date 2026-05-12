# Catalogue des sources de données — Sobr.ia

> **Version** : 2.0 — pivot ComparIA (12 mai 2026).
> Chaque source traverse le pipeline médaillon (ADR-0009).
> Toute modification = mise à jour du schéma Silver correspondant + bump de version.

---

## Stratégie de priorisation

| Tier | Critère | Sources |
|------|---------|---------|
| **Tier 1** | Datasets officiels référencés par le défi data.gouv.fr | ComparIA, RTE-NaTran-Teréga IRIS |
| **Tier 2** | Sources publiques, gratuites, sans authentification | ADEME, EcoLogits, ML.Energy, CodeCarbon, HF AI Energy Score, Papers |
| **Tier 3** | Source nationale stratégique avec compte gratuit | RTE eco2mix (mix élec FR live) |
| **Supprimées** | Sources nécessitant compte payant ou clé restrictive | Electricity Maps, MaxMind GeoLite2 |

**Engagement v1.0** : zéro source bloquante derrière un paywall. Toutes les sources Tier 1 et Tier 2 sont téléchargeables anonymement.

---

# 🥇 Tier 1 — Datasets officiels du défi

## S01 — ComparIA (Ministère de la Culture / Beta.gouv)

| Champ | Valeur |
|-------|--------|
| **URL data.gouv.fr** | https://www.data.gouv.fr/datasets/compar-ia |
| **Plateforme** | https://comparia.beta.gouv.fr/ |
| **Code source** | https://github.com/betagouv/ComparIA |
| **Producteur** | Ministère de la Culture (DINUM / Beta.gouv) |
| **Licence** | **Licence Ouverte / Open Licence v2.0 (Etalab 2.0)** |
| **Compatibilité Sobr.ia** | ✅ Parfaite (Etalab ↔ MIT/CC-BY) |
| **Fréquence MAJ** | Trimestrielle (snapshot dernière MAJ : 5 mars 2026) |
| **Politique fetch** | Hebdomadaire (poll metadata) |
| **Authentification** | ❌ Aucune |
| **Status défi** | 🎯 Dataset central |

### Fichiers à ingérer

| Fichier | Format | Taille | URL téléchargement |
|---------|--------|--------|---------------------|
| Conversations | Parquet | 682 MB | https://www.data.gouv.fr/api/1/datasets/r/7651fd0b-f222-43b3-8db8-ed6ae660d313 |
| Votes | Parquet | 733 MB | https://www.data.gouv.fr/api/1/datasets/r/4ffc86e1-84a4-4fdc-9726-66408e596fef |
| Réactions | Parquet | 3,4 GB | https://www.data.gouv.fr/api/1/datasets/r/9dd3d51f-4299-4193-ab46-81ae039fe1be |
| Documentation | CSV | 8,9 KB | https://www.data.gouv.fr/api/1/datasets/r/278fb12a-f75a-4891-add8-fff1d82f1622 |

**Volume total : ~5 GB** — gros, mais Parquet colonnaire permet une ingestion fluide en DuckDB / polars.

### Entités Silver produites

- `comparia_conversations.parquet` — questions + réponses + métadonnées modèle + impact env. par message
- `comparia_votes.parquet` — préférences utilisateur par paire de modèles
- `comparia_reactions.parquet` — réactions message par message (granularité fine)

### Champs critiques pour Sobr.ia

- Modèle utilisé (provider + nom + version)
- Tokens entrée / sortie réels (mesurés, pas estimés)
- Impact environnemental calculé par EcoLogits (CO₂eq, énergie, eau si présent)
- Localisation serveur (datacenter)
- Type de question (catégorie thématique)
- Préférences utilisateurs (signal qualité)

### Pourquoi c'est crucial

- **Données réelles d'usage** en français (~250 000 questions, ~150 000 votes).
- **Méthodologie environnementale officielle** (EcoLogits + ISO 14044) déjà appliquée.
- **Croisable** avec ComparIA est mention culturelle française → cohérence territoriale.
- **Licence Etalab 2.0** → republication possible.

### Risques

- Volume important → DVC obligatoire, pas de check-in Git.
- Données utilisateur potentiellement sensibles (questions formulées en clair) → pseudonymisation à vérifier dans la doc ComparIA.
- Couverture modèles fermés/ouverts inégale.

### Tests / validation

- Cohérence : somme(impact par message) = impact par conversation.
- Présence des champs obligatoires sur ≥ 99 % des lignes.
- Distribution des modèles attendue (≥ 30 modèles couverts).

---

## S02 — Consommation annuelle IRIS sites industriels (RTE / NaTran / Teréga)

| Champ | Valeur |
|-------|--------|
| **URL data.gouv.fr** | https://www.data.gouv.fr/datasets/consommation-annuelle-definitive-delectricite-et-de-gaz-par-iris-des-sites-industriels-raccordes-aux-reseaux-de-transport |
| **Source originale** | https://odre.opendatasoft.com/explore/dataset/consommation-annuelle-par-iris/ |
| **Producteurs** | NaTran, RTE, Teréga (gestionnaires des réseaux de transport élec + gaz) |
| **Diffuseur** | Open Data Réseaux Énergies (ODRÉ) |
| **Licence** | **Licence Ouverte / Open Licence v2.0 (Etalab 2.0)** |
| **Compatibilité Sobr.ia** | ✅ Parfaite |
| **Fréquence MAJ** | Annuelle (dernière : 6 décembre 2024, référentiel IRIS 2023) |
| **Politique fetch** | Annuelle (déclenchée manuellement à chaque MAJ ODRÉ) |
| **Authentification** | ❌ Aucune |
| **Status défi** | 🎯 Dataset officiel — angle territorial |

### Fichiers à ingérer

| Fichier | Format | Taille | URL téléchargement |
|---------|--------|--------|---------------------|
| Données tabulaires | CSV | 90,2 MB | https://www.data.gouv.fr/api/1/datasets/r/631d6ec4-74c5-4f0f-9187-442cf9d1f0bc |
| Données + coordonnées (JSON) | JSON | 97,8 MB | https://www.data.gouv.fr/api/1/datasets/r/67a173e0-30c6-461c-aed5-3f6a8ab15a09 |
| GeoJSON | JSON | 91,0 MB | https://www.data.gouv.fr/api/1/datasets/r/2b584d52-f4e0-4232-87df-c456e715f334 |
| Shapefile (ZIP) | ZIP | 38,9 MB | https://www.data.gouv.fr/api/1/datasets/r/95fb8941-47a9-4d15-8c7a-3320fa0e0319 |

### Entités Silver produites

- `iris_consommation_annuelle.parquet` — consommation MWh élec + gaz par IRIS + nombre de sites
- `iris_geometries.parquet` (depuis le GeoJSON / shapefile) — polygones IRIS pour rendu carto

### Champs critiques

- Code IRIS (référentiel INSEE 2023)
- Consommation électrique annuelle (MWh)
- Consommation gaz annuelle (MWh)
- Nombre de sites industriels raccordés transport
- Année de référence
- Géométrie du polygone IRIS

### Usage dans Sobr.ia

- **Module M12 — Territoire français** : visualisation choroplèthe des concentrations industrielles.
- **Heuristique datacenter** : un IRIS avec > N MWh élec + faible gaz est candidat à héberger un datacenter (à valider).
- **Scénarios régionaux** : projeter l'impact d'un déploiement LLM par région française.
- **Storytelling** : "voici les 10 IRIS qui concentrent X % de la consommation industrielle élec — combien d'usage IA correspond à un IRIS moyen ?"

### Risques

- Données annuelles → granularité temporelle grossière (compensée par eco2mix pour live).
- Anonymisation par seuils : certains IRIS peuvent être en "secret statistique" → traitement spécial.
- Évolution annuelle des polygones IRIS (INSEE) → versionner le référentiel.

### Tests / validation

- Somme des consommations par région cohérente avec les bilans RTE/GRDF nationaux (±5 %).
- Géométries valides (GeoJSON RFC 7946).
- Nombre d'IRIS couverts (~50 000 IRIS en France) → check ratio de couverture.

---

# 🥈 Tier 2 — Sources complémentaires (sans authentification)

## S03 — ADEME Base Empreinte

| Champ | Valeur |
|-------|--------|
| **URL** | https://base-empreinte.ademe.fr/ |
| **API** | https://data.ademe.fr/datasets/base-empreinte |
| **Format Copper** | CSV + JSON |
| **Licence** | Etalab 2.0 |
| **Fréquence MAJ** | Trimestrielle |
| **Politique fetch** | Hebdomadaire |
| **Authentification** | ❌ Aucune |
| **Entités Silver** | `electricity_factors`, `hardware_factors` |

**Usage** : facteurs d'émission de référence (gCO₂eq/kWh par pays, par source, par année). Indispensable pour normaliser ComparIA → CO₂eq quand le champ n'est pas pré-rempli.

---

## S04 — GenAI Impact / EcoLogits

| Champ | Valeur |
|-------|--------|
| **URL** | https://github.com/genai-impact/ecologits |
| **Format Copper** | JSON + Python (catalogue modèles) |
| **Licence** | MIT |
| **Fréquence MAJ** | Continue (releases GitHub) |
| **Politique fetch** | Hebdomadaire (poll releases) |
| **Authentification** | ❌ Aucune |
| **Entités Silver** | `ecologits_models`, `ecologits_assumptions` |

**Usage** : **la méthodologie officielle** déjà adoptée par ComparIA. Sobr.ia doit l'appliquer fidèlement (et la valider sur les 3 études de référence). Importer le catalogue de modèles + paramètres d'incertitude permet d'estimer aussi des modèles non couverts par ComparIA.

---

## S05 — Hugging Face AI Energy Score

| Champ | Valeur |
|-------|--------|
| **URL** | https://huggingface.co/AIEnergyScore |
| **Format Copper** | JSON / Parquet via HF Hub |
| **Licence** | Apache 2.0 (modèles) / CC-BY (scores) |
| **Fréquence MAJ** | Ad hoc (nouveaux modèles) |
| **Politique fetch** | Hebdomadaire |
| **Authentification** | ❌ Aucune (lecture publique HF Hub) |
| **Entités Silver** | `model_energy_score` |

**Usage** : complète ComparIA pour les modèles non couverts. Score standardisé permettant des comparaisons.

---

## S06 — CodeCarbon Hub

| Champ | Valeur |
|-------|--------|
| **URL** | https://github.com/mlco2/codecarbon |
| **Format Copper** | CSV / JSON via GitHub releases |
| **Licence** | MIT |
| **Fréquence MAJ** | Continue |
| **Politique fetch** | Mensuelle |
| **Authentification** | ❌ Aucune |
| **Entités Silver** | `training_runs`, `inference_runs` |

**Usage** : mesures réelles d'entraînement et d'inférence soumises par la communauté. Élément de validation croisée.

---

## S07 — ML.Energy Leaderboard

| Champ | Valeur |
|-------|--------|
| **URL** | https://ml.energy/leaderboard |
| **Format Copper** | CSV publié (+ scraping fallback) |
| **Licence** | CC-BY 4.0 |
| **Fréquence MAJ** | Mensuelle |
| **Politique fetch** | Mensuelle |
| **Authentification** | ❌ Aucune |
| **Entités Silver** | `inference_benchmarks` |

**Usage** : benchmarks contrôlés sur hardware standardisé (NVIDIA A100/H100). Référence pour validation des distributions Sobr.ia.

---

## S08 — Papers académiques

| Champ | Valeur |
|-------|--------|
| **Sources** | arXiv, ACL Anthology, NeurIPS, Nature, etc. |
| **Format Copper** | PDF + extracts manuels |
| **Licence** | Variable (arXiv libre, journaux à vérifier) |
| **Fréquence MAJ** | Ad hoc |
| **Politique fetch** | Manuelle curatée |
| **Authentification** | Selon source (la plupart libre) |
| **Entités Silver** | `extracted_measures` |

**Usage** : validation croisée méthodologique (Luccioni 2023, Patterson 2021, Faiz 2024, etc.).

---

# 🥉 Tier 3 — Source optionnelle (compte gratuit)

## S09 — RTE eco2mix (mix électrique français live)

| Champ | Valeur |
|-------|--------|
| **URL** | https://www.rte-france.com/eco2mix |
| **API** | https://digital.iservices.rte-france.com/ (OAuth2) |
| **Format Copper** | JSON |
| **Licence** | Etalab 2.0 |
| **Fréquence MAJ amont** | Temps réel (15 min) |
| **Politique fetch** | Horaire |
| **Authentification** | OAuth2 client credentials — **clé gratuite après inscription** |
| **Entités Silver** | `mix_hourly_fr`, `mix_daily_fr` |
| **Optionnalité** | ✅ Optionnel pour v1.0 — recommandé pour storytelling fin |

**Usage** : pour les estimations temps réel et le module M4 simulateur avec trajectoire RTE 2030. Si la clé n'est pas obtenue en temps voulu, fallback sur valeur annuelle moyenne ADEME (≈ 56 gCO₂eq/kWh pour la France).

**Fallback sans clé** : RTE publie aussi des **CSV historiques téléchargeables sans authentification** sur leur portail open data (https://opendata.reseaux-energies.fr/). Ces CSV suffisent pour les analyses retrospectives.

---

# ❌ Sources supprimées du périmètre v1.0

| Source | Raison de suppression | Replacement |
|--------|----------------------|-------------|
| Electricity Maps | Plan gratuit limité, licence CC-BY-SA virale | RTE eco2mix pour FR, ADEME pour autres pays |
| MaxMind GeoLite2 | Compte gratuit requis, CC-BY-SA virale | Pour M9 : mapping provider → zone (heuristique) ; pour M12 : pas besoin (IRIS = géocodage natif) |

---

# Vue d'ensemble — matrice rapide

| ID | Source | Tier | Bloquant ? | Sprint cible |
|----|--------|------|------------|--------------|
| S01 | **ComparIA** | 1 🎯 | **Oui** | **S2** |
| S02 | **RTE IRIS** | 1 🎯 | **Oui** | **S2** |
| S03 | ADEME Base Empreinte | 2 | Oui | S2 |
| S04 | EcoLogits | 2 | Oui (méthodo) | S0 + S3 |
| S05 | HF AI Energy Score | 2 | Recommandé | S3 |
| S06 | CodeCarbon | 2 | Recommandé | S3 |
| S07 | ML.Energy | 2 | Recommandé | S3 |
| S08 | Papers | 2 | Oui (validation) | S0 + S3 |
| S09 | RTE eco2mix | 3 | Non (optionnel) | S3 |

**8 sources actives au lieu de 10**, **0 clé bloquante** pour v1.0, **2 datasets officiels du défi** au cœur.

---

# Schémas Silver (vue d'ensemble)

```
schemas/silver/
├── comparia_conversations-v1.json   (S01)
├── comparia_votes-v1.json           (S01)
├── comparia_reactions-v1.json       (S01)
├── iris_consommation-v1.json        (S02)
├── iris_geometries-v1.json          (S02)
├── electricity_factors-v1.json      (S03)
├── hardware_factors-v1.json         (S03)
├── ecologits_models-v1.json         (S04)
├── ecologits_assumptions-v1.json    (S04)
├── model_energy_score-v1.json       (S05)
├── training_runs-v1.json            (S06)
├── inference_runs-v1.json           (S06)
├── inference_benchmarks-v1.json     (S07)
├── extracted_measures-v1.json       (S08)
└── mix_hourly_fr-v1.json            (S09, optionnel)
```

---

# Pré-requis avant ingestion (S2)

- [x] Stratégie de sources figée (ce document)
- [ ] Stockage DVC remote configuré (R2 ou S3 — ComparIA pèse 5 GB)
- [ ] Schémas Silver v1 figés et committés
- [ ] Tests `proptest` template prêts
- [ ] (Optionnel) Clé API RTE eco2mix obtenue (Thibault)

---

# Politique de rétention par source

| Source | Politique Copper |
|--------|-------------------|
| ComparIA | 1 snapshot par MAJ trimestrielle (4/an), conservé indéfiniment (lineage scientifique) |
| RTE IRIS | 1 snapshot par MAJ annuelle, conservé indéfiniment |
| ADEME | 30 derniers snapshots, puis mensuels 2 ans, puis annuels |
| EcoLogits | Tous les snapshots de release GitHub (versionnement strict) |
| Autres | 30 derniers snapshots, puis mensuels 1 an |

---

*Ce catalogue est vivant. Toute nouvelle source = nouvelle fiche + nouvelle implémentation `DataLayer`. Si le défi ajoute un dataset officiel, il passe directement en Tier 1.*
