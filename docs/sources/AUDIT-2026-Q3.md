# Audit datasets Sobr.ia — Q3 2026

> **Statut** : en cours (1ère passe, batch 1 livré)
> **Auteur** : Cowork
> **Date** : 2026-05-16
> **Périmètre** : audit exhaustif des sources de données disponibles pour enrichir le référentiel Sobr.ia (modèles, datacenters, mix électrique, facteurs émission, benchmarks IA).
> **Méthodologie** : voir `briefs/chantiers/C30-audit-datasets.md`.
> **Output cible** : ~40-50 sources évaluées en 5 catégories + matrice de priorisation + roadmap C31.

---

## Synthèse exécutive (preview, à finaliser)

> Cette synthèse sera complétée une fois les 5 catégories auditées. Objectif : 1 page lisible jury data.gouv.fr.

**État au 2026-05-16** : **19 sources évaluées** sur ~40-50 attendues (batches 1+2+3+4 livrés). Le périmètre couvert est désormais **suffisant pour le pitch v1.0** ; les sources restantes seront ajoutées en v1.1+.

**Quick wins identifiés (à intégrer dans C31)** :
- ⭐⭐⭐ **Mistral AI Environmental Footprint Large 2** (Cat. D, premier vendor mondial à publier ACV complet, partenariat ADEME — différenciateur pitch absolu)
- ⭐⭐ **Google Gemini Environmental Disclosure** (Cat. D, 2ème vendor à publier chiffres prompt-level, août 2025)
- ⭐ **Meta Llama 3.x model cards** (Cat. D, 3ème vendor disclosure — training officiel + distinction location/market-based)
- ⭐ **ML.ENERGY Leaderboard v3.0** (Cat. C, 46 modèles × 7 tasks H100+B200 = 1858 configs empiriques, déc 2025)
- ⭐ ADEME Base Empreinte (Cat. A, facteurs émission numérique officiels FR)
- ⭐ HuggingFace AI Energy Score (Cat. C, ratings 1-5 étoiles, mapping naturel score Sobr.ia A-F)
- EpochAI AI Models dataset (Cat. C, CC-BY, ~1500 modèles + trends compute)
- ODRE complémentaire (Cat. A, 4 datasets supplémentaires : registre installations + eco2mix horaire + régional + EPCI)
- ARCEP "Pour un numérique soutenable" édition 2025 (Cat. A, parse PDF), édition 2026 à surveiller
- The Shift Project — projections 2030 (Cat. E, citation + valeurs clés M16 Forecaster)
- ⭐⭐ **IEA Energy and AI 2025** (Cat. B, projections internationales référence — fourchette monde avec Shift Project)
- Cloud Carbon Footprint Thoughtworks (Cat. E, Apache 2.0, facteurs PUE + carbon intensity cloud par région)

**Strategic bets (v1.2+)** :
- ENTSO-E Transparency Platform (Cat. B, mix élec horaire EU — internationaliser Sobr.ia au-delà FR)
- Boavizta API (Cat. E, ACV multi-critères CC BY-SA)
- MLPerf Power benchmarks (Cat. C, energy efficiency ML systems)

**Watchlist / À clarifier** :
- ⚠️ ElectricityMaps API (free tier **non-commercial uniquement** — bloque l'offre cloud managed ADR-0014). Alternative : leurs parsers GitHub open source à réutiliser dans le pipeline médaillon.

**Couvert indirectement via S03** :
- NegaOctet — 50 datasets numériques publiés dans Base Empreinte ADEME (pas besoin de duplication)

**Highlights pitch défi data.gouv.fr** :
- Sobr.ia s'appuie sur **3 sources officielles FR** (ADEME, ARCEP, ODRE) + 1 modèle vendor FR avec ACV publié (Mistral × ADEME) + 2 vendors internationaux transparents (Google Gemini + Meta Llama).
- Cohérence méthodologique avec **4 référentiels indépendants** : AFNOR/Sobr.ia interne + EcoLogits + HF AI Energy Score + ML.ENERGY Benchmark (empirique GPU réel).
- Référence internationale macro avec IEA Energy and AI 2025 (945 TWh datacenters 2030) couplée vision prospective via Shift Project (1 500 TWh max).
- **Table de comparaison vendor disclosure** dans Sobr.ia (M9) :

| Vendor | Disclosure officielle prompt-level ? | Disclosure training ? | Source |
|---|---|---|---|
| **Mistral AI** (FR) | ✅ Oui (août 2025) | ✅ Oui | Vendor + ADEME |
| **Google (Gemini)** | ✅ Oui (août 2025) | ⚠️ Partiel | Vendor |
| **Meta (Llama 3.x)** | ❌ Non | ✅ Oui (location + market-based) | Vendor model cards |
| **Anthropic (Claude)** | ❌ Non | ❌ Non | Estimations tierces |
| **OpenAI (GPT-4o)** | ❌ Non | ❌ Non | Estimations tierces |

**Sobr.ia se positionne comme tiers de confiance** qui agrège, normalise, et présente ces disclosures (et leurs absences) avec leur lineage — cohérent avec mission "tiers de confiance" pitch data.gouv.fr.

---

## Méthodologie de l'audit

### Catégories

| Code | Catégorie | Description |
|---|---|---|
| **A** | Gouvernementales FR | data.gouv.fr, ADEME, ARCEP, ODRE, INSEE, IGN, CNIL, BPI, etc. |
| **B** | EU + globales open | Eurostat, EEA, IEA, OWID, ENTSO-E, JRC, IRENA |
| **C** | Académique + benchmarks IA | HF leaderboards, MLPerf, ML.Energy, HELM, EpochAI, LMSYS Arena |
| **D** | Cartes modèles industriels | OpenAI, Anthropic, Google, Meta, Mistral, Cohere, etc. |
| **E** | Carbon-specific + géoloc | Boavizta, NegaOctet, ElectricityMaps, WattTime, Shift Project |

### Grille de scoring (par source)

Chaque source reçoit :
- **Score valeur Sobr.ia** /10 (adéquation moteur + granularité géo + différenciateur pitch + fraîcheur)
- **Score effort intégration** /10 (volume/format + licence + stabilité + transformations)
- **Décision** : ✅ Intégrer C31 / 📋 Backlog v1.2+ / ⏸ Surveiller / ❌ Rejeté

### Quadrant de priorisation

```
            Effort faible          Effort élevé
          ┌──────────────────┬──────────────────┐
Valeur    │   QUICK WINS     │  STRATEGIC BETS  │
élevée    │   → C31 court    │  → C31 long ou   │
          │                  │    v1.2+         │
          ├──────────────────┼──────────────────┤
Valeur    │   FILL-IN /      │     SKIP /       │
faible    │   BACKLOG        │    ⏸ ou ❌      │
          │                  │                  │
          └──────────────────┴──────────────────┘
```

---

# 🏛️ Catégorie A — Sources gouvernementales françaises

## A.S03 — ADEME Base Empreinte ⭐

| Champ | Valeur |
|---|---|
| **Producteur** | ADEME — Agence de la transition écologique |
| **URL canonique** | [base-empreinte.ademe.fr](https://base-empreinte.ademe.fr/) |
| **URL data.gouv.fr** | [data.gouv.fr/dataservices/api-base-carbone](https://www.data.gouv.fr/dataservices/api-base-carbone) |
| **Portail open data** | [data.ademe.fr](https://data.ademe.fr/datasets) |
| **Catégorie** | A — Gouv FR |
| **Licence** | Etalab 2.0 (open data ADEME) |
| **Compatibilité Sobr.ia** | ✅ Parfaite |
| **Volume** | ~1500 composants/équipements, ~50 facteurs multi-critères numériques (intégrés via NegaOctet) |
| **Format** | API REST + dumps CSV/JSON |
| **Authentification** | ❌ Aucune (Open Data) |
| **Fréquence MAJ** | Régulière, version courante V23.6 (juillet 2025) |
| **Fraîcheur** | V23.6 = juillet 2025, très frais |
| **Accessibilité tech** | API publique documentée |
| **Score valeur Sobr.ia** | **9/10** — facteurs émission ACV numérique = pilier méthodologique ; remplace nos hypothèses internes par des facteurs officiels FR |
| **Score effort intégration** | **3/10** — API REST stable, JSON standard, schéma documenté |
| **Tier proposé** | **Tier 2 forte priorité** |
| **Risques** | Schéma API peut évoluer entre versions majeures (V23 → V24) ; tracer le `version_id` |
| **Décision** | ✅ **Intégrer dans C31** |

**Note méthodologique** : Base de référence officielle française pour la comptabilité carbone et l'écoconception. Intègre les données NegaOctet (1500 composants IT classifiés). Permet à Sobr.ia de citer des facteurs **officiels FR Etalab 2.0** au lieu de constantes calibrées en dur dans le moteur. Renforce massivement le pitch défi data.gouv.fr (méthodologie traçable à une source institutionnelle).

---

## A.S04 — ARCEP « Pour un numérique soutenable » ⭐

| Champ | Valeur |
|---|---|
| **Producteur** | ARCEP — Autorité de régulation des communications électroniques |
| **URL canonique** | [arcep.fr/cartes-et-donnees/.../enquete-annuelle-pour-un-numerique-soutenable](https://www.arcep.fr/cartes-et-donnees/nos-publications-chiffrees/impact-environnemental/enquete-annuelle-pour-un-numerique-soutenable-edition-2025.html) |
| **Catégorie** | A — Gouv FR |
| **Licence** | Données publiques (publication officielle ARCEP, à confirmer Etalab 2.0) |
| **Compatibilité Sobr.ia** | ✅ Compatible |
| **Volume** | ~30 indicateurs annuels (consommation élec, GES, eau, datacenters, terminaux) |
| **Format** | PDF rapport + tableaux Excel + (à venir 2026) données détaillées cloud/IA gé |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Annuelle. Édition 2025 (données 2023) disponible. Édition 2026 (données 2024) en cours, **enrichie d'indicateurs IA générative officiels** (GES, élec, eau, impact IA). |
| **Fraîcheur** | Édition 2025 OK. Édition 2026 publiée fin 2026. |
| **Accessibilité tech** | PDF + Excel téléchargeables ; pas d'API |
| **Score valeur Sobr.ia** | **9/10** — collecte officielle FR sur IA gé = **différenciateur majeur pitch défi data.gouv.fr** |
| **Score effort intégration** | **5/10** — parsing PDF + Excel manuel, schéma stable mais semi-structuré |
| **Tier proposé** | **Tier 2 forte priorité** (édition 2025 disponible immédiatement) |
| **Risques** | Format PDF peut changer ; rythme annuel implique 1 MAJ/an manuelle |
| **Décision** | ✅ **Intégrer édition 2025 dans C31**, surveiller édition 2026 |

**Note méthodologique** : L'enquête ARCEP est la source de référence française pour les indicateurs environnementaux du secteur numérique. La décision homologuée du 21 janvier 2026 enrichit la collecte 2026 avec les fournisseurs cloud et inclut explicitement les indicateurs IA générative (impact sur GES, électricité, eau). C'est la donnée officielle FR la plus alignée avec la mission Sobr.ia. À court terme : extraire les tableaux édition 2025 (datacenters FR) ; à moyen terme : ingérer édition 2026 dès publication.

---

## A.S05 — ODRE — Open Data Réseaux Énergies (catalogue complet)

| Champ | Valeur |
|---|---|
| **Producteur** | RTE + GRTgaz + NaTran + Teréga + Enedis + GRDF (consortium) |
| **URL canonique** | [opendata.reseaux-energies.fr](https://opendata.reseaux-energies.fr/) |
| **URL data.gouv.fr** | [data.gouv.fr/organizations/open-data-reseaux-energies-1](https://www.data.gouv.fr/organizations/open-data-reseaux-energies-1) |
| **Catégorie** | A — Gouv FR |
| **Licence** | Etalab 2.0 |
| **Compatibilité Sobr.ia** | ✅ Parfaite (déjà utilisé partiellement pour S02 RTE-IRIS) |
| **Volume** | 200+ datasets |
| **Format** | CSV + JSON + API REST opendatasoft |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Variable selon dataset (horaire à annuel) |
| **Fraîcheur** | Très bonne, MAJ janvier 2026 sur plusieurs datasets clés |
| **Accessibilité tech** | API opendatasoft standard (excellente) |
| **Score valeur Sobr.ia** | **7/10** — déjà partiellement exploité, mais réservoir de datasets complémentaires |
| **Score effort intégration** | **4/10** — API uniforme, schémas connus |
| **Tier proposé** | **Tier 2** (élargir l'usage existant) |
| **Risques** | Aucun particulier |
| **Décision** | ✅ **Élargir l'usage dans C31** (4 datasets complémentaires identifiés ci-dessous) |

### Datasets ODRE à ingérer en complément (sous-sources)

| Code | Dataset | Valeur ajoutée |
|---|---|---|
| **A.S05.1** | [Registre national installations production+stockage électricité](https://odre.opendatasoft.com/explore/dataset/registre-national-installation-production-stockage-electricite-agrege/) (janv 2026) | Cartographier les sources réelles d'élec consommée par les datacenters FR (nucléaire/renouvelable/fossile) |
| **A.S05.2** | [Eco2mix national consolidé](https://odre.opendatasoft.com/explore/dataset/eco2mix-national-cons-def/) (2012-janv 2026) | Mix élec horaire FR pour calculs intensité carbone temps réel (déjà ingéré dans la v0.5.0 mais juste annuel) |
| **A.S05.3** | [Eco2mix régional consolidé](https://odre.opendatasoft.com/explore/dataset/eco2mix-regional-cons-def/) (2013-janv 2026) | Granularité régionale → datacenters par région NUTS-2 |
| **A.S05.4** | [Consommation EPCI annuelle](https://reseaux-energies-rte.opendatasoft.com/explore/dataset/conso-epci-annuelle/api/) | Maille intercommunalité, complément IRIS |

---

# 🌍 Catégorie B — Sources européennes + globales

## B.S11 — IEA — Energy and AI Report 2025 ⭐⭐

| Champ | Valeur |
|---|---|
| **Producteur** | International Energy Agency (IEA / OCDE) |
| **URL canonique** | [iea.org/reports/energy-and-ai](https://www.iea.org/reports/energy-and-ai) |
| **Catégorie** | B — EU/global |
| **Licence** | Citation libre (publication officielle IEA) |
| **Compatibilité Sobr.ia** | ✅ Citation + chiffres référence |
| **Volume** | Rapport ~400 pages, projections monde 2024 → 2035, scenarios Base/High/Lift-off |
| **Format** | PDF + tableaux Excel + dashboard interactif |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Publication d'avril 2025 + news updates trimestrielles |
| **Fraîcheur** | Avril 2025 + news 2025-11 (data center +17% en 2025) |
| **Accessibilité tech** | PDF + scrapings tables ; dashboard interactif |
| **Score valeur Sobr.ia** | **9/10** — référence INTERNATIONALE pour projections datacenters + IA |
| **Score effort intégration** | **3/10** — extraction de 10-20 chiffres clés, pas un ingest massif |
| **Tier proposé** | **Tier 2 quick win** (citations + valeurs forecaster) |
| **Risques** | Aucun |
| **Décision** | ✅ **Intégrer dans C31** — chiffres pour M16 Forecaster + datasheet |

**Note méthodologique** : L'IEA est THE référence mondiale pour les projections énergétiques. Chiffres clés à intégrer dans Sobr.ia :
- **Datacenters monde 2030** : 945 TWh (≈ 3 % consommation élec globale, équivalent Japon entier).
- **Croissance 2025** : +17 % global datacenters, **+50 % datacenters dédiés IA**.
- **Accelerated servers** (= GPU IA) : +30 % par an dans le scénario Base.
- **US + Chine** = 80 % de la croissance mondiale jusqu'à 2030.
- **Trajectoire** : 460 TWh (2024) → 1 000 TWh (2030) → 1 300 TWh (2035).

À comparer avec Shift Project (1 250 à 1 500 TWh 2030) pour fournir une fourchette dans M16 Forecaster : *« Selon IEA Base 945 TWh, selon Shift jusqu'à 1 500 TWh — voici votre contribution personnelle »*. Citation IEA renforce la crédibilité internationale de Sobr.ia.

---

## B.S10 — ENTSO-E Transparency Platform ⭐

| Champ | Valeur |
|---|---|
| **Producteur** | ENTSO-E — European Network of Transmission System Operators for Electricity |
| **URL canonique** | [transparency.entsoe.eu](https://transparency.entsoe.eu/) |
| **API doc** | [Guide REST API](https://transparency.entsoe.eu/content/static_content/Static%20content/web%20api/Guide.html) |
| **Catégorie** | B — EU/global |
| **Licence** | CC-BY 4.0 (open data, attribution requise) |
| **Compatibilité Sobr.ia** | ✅ Compatible avec attribution |
| **Volume** | Tous les TSO européens, production/consommation horaire, équilibrage, pannes |
| **Format** | API REST + CSV bulk + File Library |
| **Authentification** | 🔑 API key gratuite (email à transparency@entsoe.eu) |
| **Fréquence MAJ** | Horaire |
| **Fraîcheur** | Temps réel (refresh 1 h) |
| **Accessibilité tech** | API standard, Python clients existants |
| **Score valeur Sobr.ia** | **8/10** — couvre TOUS les pays EU (vs RTE/IRIS = FR only), permet datacenters Europe avec mix horaire pays |
| **Score effort intégration** | **5/10** — API key + parsing XML/CSV (moins propre que REST JSON) |
| **Tier proposé** | **Tier 2 strategic bet** |
| **Risques** | API key nécessaire (mais gratuite et permanent) |
| **Décision** | 📋 **Backlog v1.2+** — high value pour étendre M12 datacenters Europe à un calcul carbone précis par pays, mais effort API key + XML parsing |

**Note méthodologique** : Indispensable pour internationaliser Sobr.ia au-delà du périmètre FR. Couplé avec M12 Datacenters Europe (déjà implémenté en v0.4), permettrait de calculer en temps réel l'intensité carbone d'une requête selon le pays du datacenter. ⚠️ L'API key gratuite est un compromis acceptable (pas un paywall, juste une formalité d'enregistrement).

---

# 🎓 Catégorie C — Académique + benchmarks IA

## C.S20 — Hugging Face AI Energy Score ⭐

| Champ | Valeur |
|---|---|
| **Producteur** | Hugging Face (Sasha Luccioni et al.) — Initiative open source |
| **URL canonique** | [huggingface.github.io/AIEnergyScore](https://huggingface.github.io/AIEnergyScore/) |
| **Leaderboard** | [huggingface.co/spaces/AIEnergyScore/Leaderboard](https://huggingface.co/spaces/AIEnergyScore/Leaderboard) |
| **Code source** | [github.com/huggingface/AIEnergyScore](https://github.com/huggingface/AIEnergyScore) |
| **Catégorie** | C — Académique |
| **Licence** | Apache 2.0 / MIT (open source HF), notation 1-5 étoiles dérivée librement |
| **Compatibilité Sobr.ia** | ✅ Parfaite (cohérent multi-méthodologie ADR-0012) |
| **Volume** | 10 tâches × N modèles évalués, score 1-5 étoiles, refresh régulier |
| **Format** | JSON via API HF Spaces + dataset HuggingFace Hub |
| **Authentification** | ❌ Aucune pour lecture publique |
| **Fréquence MAJ** | Continue (modèles ajoutés au fil de l'eau) ; v2 lancée en 2025 avec reasoning |
| **Fraîcheur** | Très frais, v2 incluant reasoning task |
| **Accessibilité tech** | API HF datasets standard + leaderboard space |
| **Score valeur Sobr.ia** | **9/10** — rating normalisé 1-5 étoiles parfaitement alignable avec notre score Sobr.ia A-F |
| **Score effort intégration** | **3/10** — `huggingface_hub` crate ou simple `reqwest` JSON |
| **Tier proposé** | **Tier 2 quick win** |
| **Risques** | Aucun particulier (initiative HF stable) |
| **Décision** | ✅ **Intégrer dans C31** (mapping AI Energy Score ↔ score Sobr.ia A-F) |

**Note méthodologique** : Standardise les évaluations énergétiques sur NVIDIA H100, dataset custom (1000 samples / 3+ datasets par tâche : WikiText, OSCAR, UltraChat-10K). Système de notation 1-5 étoiles inspiré des classes énergétiques appareils électroménagers — exactement la métaphore qu'on veut véhiculer dans Sobr.ia. Permet d'enrichir le M9 Référentiel modèles avec un score externe reconnu.

---

## C.S21 — MLPerf Power (MLCommons)

| Champ | Valeur |
|---|---|
| **Producteur** | MLCommons (consortium industriel) |
| **URL canonique** | [mlcommons.org/benchmarks/inference-datacenter/](https://mlcommons.org/benchmarks/inference-datacenter/) |
| **Code source** | [github.com/mlcommons/inference](https://github.com/mlcommons/inference) |
| **Catégorie** | C — Académique / benchmarks |
| **Licence** | Apache 2.0 (résultats publiés librement accessibles) |
| **Compatibilité Sobr.ia** | ✅ Compatible |
| **Volume** | Benchmarks v6.0 (avril 2026), résultats datacenter + edge, mesures power |
| **Format** | Tables HTML + dumps CSV publiés trimestriellement |
| **Authentification** | ❌ Aucune pour lecture publique |
| **Fréquence MAJ** | Semestrielle (v5.1 sept 2025, v6.0 avril 2026) |
| **Fraîcheur** | v6.0 d'avril 2026, très frais |
| **Accessibilité tech** | Téléchargement direct CSV résultats + scripts Python publics |
| **Score valeur Sobr.ia** | **7/10** — données power vendor-grade (rare en open) mais centrées hardware/datacenter, moins direct utilisateur final |
| **Score effort intégration** | **5/10** — parser des tableaux trimestriels, mapping hardware/modèle parfois indirect |
| **Tier proposé** | **Tier 3 strategic bet** |
| **Risques** | Schéma v5→v6 a évolué, à versionner |
| **Décision** | 📋 **Backlog v1.2+** — pas immédiatement actionnable utilisateur final, mais grosse valeur pour M9 Référentiel modèles vue détaillée datacenter |

---

## C.S23 — LMSYS / LMArena (Chatbot Arena rankings)

| Champ | Valeur |
|---|---|
| **Producteur** | LMSYS Org (UC Berkeley) — rebranded **LMArena** en janv 2026 |
| **URL canonique** | [lmarena.ai](https://lmarena.ai/) (anciennement [lmsys.org](https://www.lmsys.org/)) |
| **Leaderboard HF** | [huggingface.co/spaces/lmarena-ai/arena-leaderboard](https://huggingface.co/spaces/lmarena-ai/arena-leaderboard) |
| **Code source** | [github.com/lm-sys/FastChat](https://github.com/lm-sys/FastChat) (Apache 2.0) |
| **Datasets HF** | [lmsys/lmsys-chat-1m](https://huggingface.co/datasets/lmsys/lmsys-chat-1m), [lmsys/chatbot_arena_conversations](https://huggingface.co/datasets/lmsys/chatbot_arena_conversations) |
| **Catégorie** | C — Académique |
| **Licence** | Code FastChat Apache 2.0 ; datasets sous CC variées (à vérifier par dataset) |
| **Compatibilité Sobr.ia** | ✅ Code Apache 2.0 ; ⚠️ datasets — vérifier au cas par cas |
| **Volume** | 9 leaderboards (Text, Code, Vision, WebDev, Image Edit, Multi-Image, Search, Text-to-Video, Image-to-Video) |
| **Format** | JSON via HF Spaces + GitHub repo |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Continue (votes humains aggregés en quasi-temps réel) |
| **Fraîcheur** | Très frais (janv 2026 rebranding) |
| **Accessibilité tech** | API HF + scraping ; pas d'API officielle d'export structurée |
| **Score valeur Sobr.ia** | **6/10** — rankings populaires mais non-empreinte, complément M9 fiche modèle pour la dimension "qualité perçue" en parallèle de l'empreinte |
| **Score effort intégration** | **6/10** — pas d'API directe pour battle logs ; scraping leaderboard rank |
| **Tier proposé** | **Tier 3 fill-in** |
| **Risques** | API rate-limit ; dataset license fragmentée |
| **Décision** | 📋 **Backlog v1.2+** — utile pour afficher rang qualité côté M9 mais pas critique pour l'empreinte |

**Note méthodologique** : Permet à Sobr.ia d'afficher dans M9 fiche modèle un tradeoff "qualité × empreinte" : *« Mistral Large 2 rank 7 LMArena Text, 2.8 Wh/prompt vs GPT-4o rank 2, ? Wh/prompt »*. C'est ce que les utilisateurs veulent voir pour faire un choix éclairé.

---

## C.S25 — ML.ENERGY Leaderboard v3.0 (Univ. Michigan) ⭐

| Champ | Valeur |
|---|---|
| **Producteur** | University of Michigan — ML.ENERGY Initiative |
| **URL canonique** | [ml.energy/leaderboard](https://ml.energy/leaderboard/) |
| **Code source** | [github.com/ml-energy/leaderboard](https://github.com/ml-energy/leaderboard) |
| **Paper** | [arxiv.org/html/2505.06371v1](https://arxiv.org/html/2505.06371v1) — "ML.ENERGY Benchmark: Toward Automated Inference Energy Measurement" |
| **Catégorie** | C — Académique |
| **Licence** | Open source GitHub (Apache 2.0 probable, à confirmer) |
| **Compatibilité Sobr.ia** | ✅ Citation + utilisation données |
| **Volume** | **v3.0 (déc 2025)** : 46 modèles × 7 tasks × NVIDIA H100 + B200 = **1 858 configurations** |
| **Format** | Leaderboard web + GitHub repo + paper |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Versions majeures (v3.0 déc 2025) |
| **Fraîcheur** | Très frais (déc 2025) |
| **Accessibilité tech** | Open source téléchargeable + scraping leaderboard |
| **Score valeur Sobr.ia** | **9/10** — **mesures empiriques** d'énergie par modèle (vs nos estimations Monte-Carlo). Idéal pour calibration moteur et validation reproductive. |
| **Score effort intégration** | **3/10** — données structurées disponibles |
| **Tier proposé** | **Tier 2 quick win** |
| **Risques** | Modèles testés = LLM open-source uniquement (pas API closed). Pas de Claude, GPT-4o, Gemini. |
| **Décision** | ✅ **Intégrer dans C31** — base de calibration empirique + 4ème source de validation des PlausibilityCase ReproductionCase |

**Note méthodologique** : Données empiriques mesurées sur GPU réels (H100 + B200). Ajoute une 4ème reference au moteur Sobr.ia après EcoLogits, AFNOR/Sobr.ia (interne), et HF AI Energy Score. Finding clé : les **modèles raisonnement (chain-of-thought)** consomment 10-100× plus que les modèles directs. À intégrer dans M16 Forecaster comme variable d'ajustement.

---

## C.S22 — EpochAI — Trends in AI

| Champ | Valeur |
|---|---|
| **Producteur** | Epoch AI (think-tank IA / org de recherche) |
| **URL canonique** | [epoch.ai/](https://epoch.ai/) |
| **Dataset modèles** | [epoch.ai/data/ai-models](https://epoch.ai/data/ai-models) |
| **Catégorie** | C — Académique |
| **Licence** | **CC-BY** (libre usage avec attribution) |
| **Compatibilité Sobr.ia** | ✅ Parfaite |
| **Volume** | ~1500 modèles tracés (notable models), training compute + parameters + cost trends |
| **Format** | CSV + JSON + dashboard interactif |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Continue (analyses publiées régulièrement) |
| **Fraîcheur** | Très frais, publications 2025-2026 |
| **Accessibilité tech** | Dumps CSV téléchargeables + API |
| **Score valeur Sobr.ia** | **8/10** — métadonnées modèles riches (FLOPs training, paramètres, coût) directement utilisables presets |
| **Score effort intégration** | **4/10** — schéma CSV stable, peut servir pour bulk-load presets M9 |
| **Tier proposé** | **Tier 2 quick win** |
| **Risques** | Granularité variable selon modèle ; certains champs vides |
| **Décision** | ✅ **Intégrer dans C31** (enrichir presets `comparia` et `referentiel.sqlite` avec FLOPs training, paramètres, dates) |

**Note méthodologique** : EpochAI est devenu THE référence pour les trends de compute IA (training compute × 5/an, doublement tous les 5.2 mois). Citer leurs chiffres dans la doc Sobr.ia renforce la crédibilité scientifique. Le dataset des modèles permet de pré-remplir massivement notre catalogue M9.

---

# 🏭 Catégorie D — Cartes modèles industriels

## D.S43 — Meta — Llama 3 model cards (training disclosure) ⭐

| Champ | Valeur |
|---|---|
| **Producteur** | Meta AI |
| **URL canonique** | [github.com/meta-llama/llama-models](https://github.com/meta-llama/llama-models) |
| **Model card 3.1** | [github.com/meta-llama/llama-models/blob/main/models/llama3_1/MODEL_CARD.md](https://github.com/meta-llama/llama-models/blob/main/models/llama3_1/MODEL_CARD.md) |
| **Model card 3.3** | [github.com/meta-llama/llama-models/blob/main/models/llama3_3/MODEL_CARD.md](https://github.com/meta-llama/llama-models/blob/main/models/llama3_3/MODEL_CARD.md) |
| **Catégorie** | D — Industriel international |
| **Licence** | Model cards publiques (Llama 3.x Community License) |
| **Compatibilité Sobr.ia** | ✅ Citation + référencement |
| **Volume** | Model cards Llama 3 (8B, 70B), 3.1 (8B, 70B, 405B), 3.3 (70B), Llama 4 |
| **Format** | Markdown sur GitHub, structure stable |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | À chaque release modèle |
| **Fraîcheur** | Llama 3.3 (déc 2024), Llama 4 (2025) |
| **Accessibilité tech** | Markdown parsing direct |
| **Score valeur Sobr.ia** | **8/10** — chiffres training disclosés (rare), partiel inference |
| **Score effort intégration** | **3/10** — markdown parsing + tableau structuré |
| **Tier proposé** | **Tier 2 quick win** |
| **Risques** | Pas de chiffres prompt-level inference (training only) |
| **Décision** | ✅ **Intégrer dans C31** — preset Llama 3.x enrichi avec données training officielles |

**Chiffres clés Llama 3.x à intégrer** :
- **Llama 3 (8B + 70B)** : 2 290 tCO₂eq training, 100 % offset par Meta sustainability program.
- **Llama 3.1** : 39.3M GPU hours H100-80GB (TDP 700 W), 11 390 tCO₂eq **location-based**, **0 tCO₂eq market-based** (Meta matche 100 % renouvelables depuis 2020), 15 trillion tokens training.
- **Llama 3.3 (70B)** : identique 3.1 pour le training (mise à jour fine-tuning).

**Note méthodologique critique** : la différence **location-based vs market-based** est un point pédagogique majeur pour Sobr.ia. Meta affiche "0 tCO₂eq market-based" parce qu'ils achètent des REC (Renewable Energy Certificates) qui matchent leur conso totale annuelle. Mais l'**élec consommée localement** par les datacenters au moment du training est bien 11 390 tCO₂eq location-based. Sobr.ia doit afficher les deux et expliquer la distinction (sinon greenwashing risk). C'est exactement le type de nuance que notre méthodologie AFNOR/Sobr.ia + EcoLogits combinée peut éclairer.

---

## D.S42 — Anthropic (Claude) — Absence de disclosure officielle ⚠️

| Champ | Valeur |
|---|---|
| **Producteur** | Anthropic |
| **URL canonique** | [anthropic.com/transparency/voluntary-commitments](https://www.anthropic.com/transparency/voluntary-commitments) |
| **Catégorie** | D — Industriel international |
| **Licence** | N/A (pas de disclosure officielle) |
| **Compatibilité Sobr.ia** | ❌ Pas de chiffres officiels à citer |
| **Volume** | Aucune publication formelle ACV ou GHG protocol Scope 1/2/3 à date |
| **Fraîcheur** | Statut au 2025-Q4 : pas de disclosure officielle |
| **Score valeur Sobr.ia** | **2/10** — absence de données vendor disclosure |
| **Score effort intégration** | N/A |
| **Tier proposé** | ⏸ **Surveiller** |
| **Décision** | ⏸ **À surveiller** — pression stakeholder croissante, peut publier 2026. Sobr.ia utilise estimations EcoLogits / AI Energy Score à la place. |

**Estimations tierces disponibles** (à utiliser avec disclaimer "estimation tierce") :
- Claude 3 Opus : ~**4.05 Wh/requête, 1.80 gCO₂eq/requête**
- Claude 3 Haiku : ~**0.22 Wh/requête, 0.10 gCO₂eq/requête**

**Note méthodologique** : L'absence de disclosure officielle Anthropic est elle-même une donnée pour le pitch défi data.gouv.fr. Tableau comparatif vendor disclosure dans Sobr.ia (M9) :

| Vendor | Disclosure officielle prompt-level ? | Source |
|---|---|---|
| **Mistral AI** | ✅ Oui (août 2025, partenariat ADEME) | Vendor + ADEME |
| **Google (Gemini)** | ✅ Oui (août 2025) | Vendor |
| **Anthropic (Claude)** | ❌ Non | Estimations tierces |
| **OpenAI (GPT-4o)** | ❌ Non | Estimations tierces |
| **Meta (Llama)** | ⚠️ Partiel (training only) | Model cards |

Sobr.ia se positionne comme **tiers de confiance qui agrège, normalise et présente** ces disclosures (et leurs absences) avec leur lineage. Force du pitch défi data.gouv.fr.

---

## D.S41 — Google Gemini — Environmental Impact Disclosure ⭐⭐

| Champ | Valeur |
|---|---|
| **Producteur** | Google (rapports sustainability + paper technique 2025) |
| **URL canonique** | [services.google.com/fh/files/misc/measuring_the_environmental_impact_of_delivering_ai_at_google_scale.pdf](https://services.google.com/fh/files/misc/measuring_the_environmental_impact_of_delivering_ai_at_google_scale.pdf) |
| **Blog post** | [cloud.google.com/blog/products/infrastructure/measuring-the-environmental-impact-of-ai-inference](https://cloud.google.com/blog/products/infrastructure/measuring-the-environmental-impact-of-ai-inference) |
| **Rapport annuel** | [sustainability.google/reports/google-2025-environmental-report/](https://sustainability.google/reports/google-2025-environmental-report/) |
| **Publication** | Août 2025 (data mai 2025) + rapport sustainability 2025 annuel |
| **Catégorie** | D — Industriel international |
| **Licence** | Publication officielle, citation libre |
| **Compatibilité Sobr.ia** | ✅ Citation + référencement |
| **Volume** | 1 modèle (Gemini App text) + données aggrégées datacenters fleet |
| **Format** | PDF technique + blog + rapport annuel |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Annuelle (rapport sustainability) |
| **Fraîcheur** | Août 2025, très frais |
| **Accessibilité tech** | PDF parsing manuel (chiffres clés concentrés sur 1-2 pages) |
| **Score valeur Sobr.ia** | **9/10** — **2ème vendor mondial** (après Mistral) à publier chiffres prompt-level. Différenciateur pitch crucial. |
| **Score effort intégration** | **2/10** — chiffres clés à transcrire dans preset Gemini |
| **Tier proposé** | **Tier 1 quick win premium** |
| **Risques** | Méthodologie Google contestée par certains (cf. "greenwashing or progress" article TDS) ; à présenter avec esprit critique |
| **Décision** | ✅ **Intégrer dans C31** — preset Gemini enrichi + encadré M9 fiche modèle + transparence sur la controverse méthodologique |

**Note méthodologique** : Chiffres clés à intégrer :
- **Prompt médian Gemini Apps text** : 0.24 Wh + **0.03 gCO₂eq** + 0.26 mL eau (≈ 5 gouttes).
- Amélioration sur 12 mois : énergie ×33 et carbone ×44 en réduction.
- Datacenters Google 2024 : 30.8 TWh élec consommée (×2 vs 2020).
- Scope 1 −8 %, Scope 2 −11 %, Scope 3 **+22 %** (chaîne d'approvisionnement explose, à mentionner pour honnêteté).

**Position critique à conserver** : la méthodologie Google retient le "median prompt" qui sous-estime potentiellement les requêtes complexes (raisonnement, agents). Sobr.ia doit afficher ces chiffres avec un avertissement "valeurs vendor, méthodologie Google — pour cross-validation utilisez méthodo AFNOR/EcoLogits". C'est cohérent avec ADR-0012 multi-méthodologie : *« on présente les chiffres vendor, on les met en perspective »*.

**Comparaison Sobr.ia clé pour pitch** :
| Vendor | Énergie/prompt | gCO₂eq/prompt | Eau/prompt | Source |
|---|---|---|---|---|
| Mistral Large 2 (FR) | ~2.8 Wh (400 tk) | **1.14 g** (400 tk) | — | ADEME × Mistral |
| Google Gemini (median) | **0.24 Wh** | **0.03 g** | 0.26 mL | Google 2025 |

L'écart 40× entre Mistral Large 2 et Gemini App n'est pas comparable directement (modèle vs tâche, 400 tokens vs prompt médian, méthodologies différentes). C'est exactement le **gap pédagogique** que Sobr.ia comble en posant un standard unifié. Argument fort pitch défi data.gouv.fr.

---

## D.S40 — Mistral AI — Environmental Footprint (Large 2) ⭐⭐⭐

| Champ | Valeur |
|---|---|
| **Producteur** | Mistral AI (Paris) — partenariat ADEME + Carbone 4 |
| **URL canonique** | [mistral.ai/news/our-contribution-to-a-global-environmental-standard-for-ai](https://mistral.ai/news/our-contribution-to-a-global-environmental-standard-for-ai) |
| **Publication** | Août 2025 |
| **Catégorie** | D — Industriel FR |
| **Licence** | Publication blog, données dans le post (CC-BY de facto pour citation) |
| **Compatibilité Sobr.ia** | ✅ Parfaite (citation + référencement) |
| **Volume** | 1 modèle (Mistral Large 2) avec ACV complet 18 mois |
| **Format** | Blog post + chiffres détaillés ; possibilité d'extraire en JSON structuré |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Premier modèle 2025 ; à étendre à Medium/Small selon Mistral |
| **Fraîcheur** | Août 2025, données 18 mois jusqu'à janv 2025 |
| **Accessibilité tech** | Manuelle (parsing blog post) ou scraping ciblé |
| **Score valeur Sobr.ia** | **10/10** — **premier vendor mondial à publier ACV complet**, sur un modèle FR, en partenariat ADEME (Etalab compatible) |
| **Score effort intégration** | **2/10** — chiffres déjà publics, ~5 mins de transcription en preset enrichi |
| **Tier proposé** | **Tier 1 quick win premium** |
| **Risques** | Aucun |
| **Décision** | ✅ **Intégrer dans C31 immédiatement** — enrichir le preset `mistral-large-2` avec données ACV réelles (training 20.4 ktCO2, eau 281 000 m³, inference 1.14 gCO2/400 tokens) + citation en M9 fiche modèle |

**Note méthodologique** : Différenciateur **majeur** pour le pitch défi data.gouv.fr.
- Production : 11 % GES + 5 % eau
- Training + inference : 85.5 % GES + 91 % eau
- 1 requête 400 tokens ≈ 1.14 gCO₂eq (10 secondes de streaming vidéo)
- Le modèle est FR, l'analyse co-réalisée avec ADEME — narratif parfaitement aligné avec Sobr.ia.

**Action C31** : remplacer la valeur générique de notre preset Mistral Large 2 par les valeurs Mistral/ADEME, et afficher dans M9 fiche modèle un encadré "Données ACV vendor (vérifiées ADEME)" qui prime sur notre estimation Monte-Carlo.

---

# ♻️ Catégorie E — Carbon-specific + géoloc

## E.S30 — Boavizta API (BoaviztAPI)

| Champ | Valeur |
|---|---|
| **Producteur** | Association Boavizta (FR, open source) |
| **URL canonique** | [boavizta.org](https://boavizta.org/) |
| **API documentation** | [doc.api.boavizta.org](https://doc.api.boavizta.org/) |
| **Code source** | [github.com/Boavizta/boaviztapi](https://github.com/Boavizta/boaviztapi) |
| **Catégorie** | E — Carbon-specific |
| **Licence** | **CC BY-SA** sur les données ; code Apache 2.0 |
| **Compatibilité Sobr.ia** | ⚠️ Compatible avec attribution (CC BY-SA), pas de mélange direct avec données MIT-only |
| **Volume** | Référentiel ACV complet : datacenters, networks, terminals, cloud + services numériques |
| **Format** | REST API (FastAPI), dumps JSON |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Active, releases régulières (PyPI) |
| **Fraîcheur** | Maintenu activement (2.0.3 récent) |
| **Accessibilité tech** | API publique standard + Python SDK |
| **Score valeur Sobr.ia** | **8/10** — ACV multi-critères complet, complète/concurrence ADEME |
| **Score effort intégration** | **4/10** — API standard, mais attention licence CC BY-SA (attribution traçable) |
| **Tier proposé** | **Tier 2 strategic bet** |
| **Risques** | Licence CC BY-SA implique attribution dans tout dérivé. Documenter le lineage dans le Gold. |
| **Décision** | 📋 **Backlog v1.2+** — high value mais nécessite traitement licence soigné, prudent de pas tout intégrer en C31 immédiat |

**Note méthodologique** : Boavizta est l'écosystème FR open source de référence pour l'ACV numérique. La licence CC BY-SA est compatible avec Sobr.ia mais impose un workflow d'attribution (mention explicite dans datasheet Gebru + sidecar PROV-O). On peut l'utiliser pour cross-valider nos chiffres ADEME et fournir une vue alternative dans M9 fiche modèle.

---

## E.S31 — NegaOctet (via Base Empreinte ADEME)

| Champ | Valeur |
|---|---|
| **Producteur** | LCIE Bureau Veritas + APL Data Center + GreenIT.fr + DDemain (consortium projet ADEME) |
| **URL canonique** | [codde.fr/en/our-brands/negaoctet](https://codde.fr/en/our-brands/negaoctet) |
| **URL Base Empreinte (open)** | [base-empreinte.ademe.fr](https://base-empreinte.ademe.fr/) — 50 datasets numériques publiés en open data |
| **Catégorie** | E — Carbon-specific |
| **Licence** | **Commerciale complète** (licence annuelle dégressive 3 ans puis gratuite) **OU 50 datasets en open data ADEME** |
| **Compatibilité Sobr.ia** | ✅ Via Base Empreinte ADEME (open) ; ❌ Via licence commerciale directe |
| **Volume** | 500 datasets × 5 niveaux granularité (DB commerciale 15 000 ACV) ; 50 datasets numériques exposés en open via ADEME |
| **Format** | ILCD EF 3.0 (standard) + CSV (Excel/LibreOffice) |
| **Authentification** | ❌ Pour les 50 open via ADEME |
| **Fréquence MAJ** | Annuelle |
| **Fraîcheur** | OK (DB active 2025) |
| **Accessibilité tech** | API ADEME pour les 50 open ; sinon licence commerciale |
| **Score valeur Sobr.ia** | **7/10** — pour les 50 datasets open (subset suffit pour MVP) ; 9/10 si on accède à la DB complète plus tard |
| **Score effort intégration** | **3/10** — passe par S03 ADEME Base Empreinte déjà identifié |
| **Tier proposé** | **Tier 2** (déjà couvert via S03 ADEME) |
| **Risques** | Aucun sur la portion open |
| **Décision** | ✅ **Intégrer indirectement via S03 ADEME** — pas besoin de duplication |

**Note méthodologique** : NegaOctet est techniquement le référentiel sous-jacent à la portion numérique de Base Empreinte. En intégrant S03 (ADEME Base Empreinte), on capture automatiquement les 50 datasets NegaOctet open. La DB complète reste un strategic bet pour v1.x si on veut une granularité plus fine.

---

## E.S34 — ElectricityMaps (revérification) ⚠️

| Champ | Valeur |
|---|---|
| **Producteur** | Electricity Maps (entreprise privée DK) |
| **URL API** | [electricitymaps.com/free-tier-api](https://www.electricitymaps.com/free-tier-api) |
| **Code open source** | [github.com/electricitymaps/electricitymaps-contrib](https://github.com/electricitymaps/electricitymaps-contrib) (parsers) |
| **Catégorie** | E — Carbon-specific |
| **Licence** | **Free tier non-commercial uniquement** (carbon intensity gCO₂eq/kWh + power breakdown, 200+ zones, live). Parsers GitHub : open source. |
| **Compatibilité Sobr.ia** | ⚠️ **Frontière** : Sobr.ia est open-source mais propose une offre cloud managed payante (ADR-0014). Le free tier non-commercial est ambigu pour la portion managed. À clarifier juridiquement. |
| **Volume** | 200+ zones (grid électrique monde), intensité carbone en temps réel + historique |
| **Format** | API REST JSON |
| **Authentification** | 🔑 API key (free tier disponible non-commercial) |
| **Fréquence MAJ** | Temps réel (refresh ~1h) |
| **Fraîcheur** | Live + 2025 update : retrait du marginal data signal |
| **Accessibilité tech** | API moderne + clients Python |
| **Score valeur Sobr.ia** | **8/10** — couverture mondiale temps réel difficilement remplaçable |
| **Score effort intégration** | **4/10** — API simple |
| **Tier proposé** | ⚠️ **Watchlist / À clarifier** |
| **Risques** | **Licence non-commercial bloque l'offre cloud managed Sobr.ia (ADR-0014 Phase 5+)**. Acceptable pour la version self-hosted gratuite. |
| **Décision** | ⏸ **Surveiller** — utilisable pour version self-hosted seule. Alternative : utiliser leurs **parsers GitHub open source** pour ré-extraire les données depuis les sources amont (RTE, ENTSO-E, EIA US, etc.) sans passer par l'API ElectricityMaps. C'est ce qu'il faut faire à terme. |

**Note méthodologique** : ElectricityMaps a un modèle freemium qui ne s'aligne pas avec ADR-0014 (notre offre managed serait commerciale). MAIS leur repo GitHub `electricitymaps-contrib` est open source et contient les **parsers** vers chaque source amont (RTE en FR, ENTSO-E en EU, EIA en US, etc.). On peut donc soit (a) consommer leur API en self-hosted only, soit (b) **réutiliser leurs parsers open source dans notre pipeline médaillon** pour aller chercher les données amont nous-mêmes. Option (b) est plus élégante et compatible cloud — à explorer en v1.2+.

---

## E.S33 — Cloud Carbon Footprint (Thoughtworks)

| Champ | Valeur |
|---|---|
| **Producteur** | Thoughtworks Inc. (open source) |
| **URL canonique** | [cloudcarbonfootprint.org](https://www.cloudcarbonfootprint.org/) |
| **Code source** | [github.com/cloud-carbon-footprint/cloud-carbon-footprint](https://github.com/cloud-carbon-footprint/cloud-carbon-footprint) |
| **Catégorie** | E — Carbon-specific cloud |
| **Licence** | **Apache 2.0** (open source) |
| **Compatibilité Sobr.ia** | ✅ Parfaite (compatible MIT) |
| **Volume** | Méthodologie + facteurs émission AWS / GCP / Azure (scope 2 et 3) |
| **Format** | Tool TypeScript + méthodologie publique markdown |
| **Authentification** | Nécessaire pour les APIs cloud (compte AWS / GCP / Azure billing) |
| **Fréquence MAJ** | Active, méthodologie versionnée |
| **Fraîcheur** | Maintenu activement 2025 |
| **Accessibilité tech** | Code TS lisible, méthodologie documentée — on n'a PAS besoin de tourner le tool, juste de réutiliser les facteurs PUE + carbon intensity régions cloud |
| **Score valeur Sobr.ia** | **7/10** — facteurs PUE + carbon intensity datacenters cloud par région (utile pour M12 Datacenters Europe et extension internationale) |
| **Score effort intégration** | **3/10** — extraction des constantes depuis méthodologie publique, pas besoin d'intégrer le tool |
| **Tier proposé** | **Tier 2 quick win** |
| **Risques** | Aucun (Apache 2.0) |
| **Décision** | ✅ **Intégrer dans C31** — extraction des facteurs PUE + carbon intensity régions cloud pour enrichir `datacenter_iris_link` Gold |

**Note méthodologique** : CCF est un projet sérieux qui maintient à jour les PUE constatés AWS/GCP/Azure par région, plus les facteurs grid carbon intensity correspondants. Plutôt que d'intégrer le tool entier, on extrait juste les constantes (~50-100 valeurs) dans une table SQLite Sobr.ia. Permet à M12 de calculer correctement l'empreinte d'un datacenter Azure West Europe vs AWS eu-west-1 sans estimation approximative.

---

## E.S32 — The Shift Project — Rapports IA et numérique ⭐

| Champ | Valeur |
|---|---|
| **Producteur** | The Shift Project (think-tank carbone FR) |
| **URL canonique** | [theshiftproject.org/en/thematics/digital/](https://theshiftproject.org/en/thematics/digital/) |
| **Rapport intermédiaire IA 2025** | [theshiftproject.org/.../2025_03_06-TSP-Rapport-intermediaire-IA-quelles-infra-num-monde-decarbone.pdf](https://theshiftproject.org/app/uploads/2025/04/2025_03_06-TSP-Rapport-intermediaire-IA-quelles-infra-num-monde-decarbone.pdf) |
| **Catégorie** | E — Carbon-specific (études et projections) |
| **Licence** | Publication publique, citation autorisée |
| **Compatibilité Sobr.ia** | ✅ Pour citations + projections |
| **Volume** | Rapport intermédiaire 2025 + multiples publications IA depuis |
| **Format** | PDF + tables chiffrées dans les rapports |
| **Authentification** | ❌ Aucune |
| **Fréquence MAJ** | Plusieurs publications par an |
| **Fraîcheur** | Très frais (rapport mars 2025, IA gé focus) |
| **Accessibilité tech** | PDF parsing manuel ou extraction tables |
| **Score valeur Sobr.ia** | **8/10** — chiffres référence pour M16 Forecaster (projection 2030) + narratif pitch |
| **Score effort intégration** | **3/10** — extraction de quelques valeurs clés, pas un ingest massif |
| **Tier proposé** | **Tier 2 quick win** (citation + valeurs clés) |
| **Risques** | Aucun |
| **Décision** | ✅ **Intégrer dans C31** — valeurs clés en `referentiel.sqlite` table `external_projections`, citation dans M16 Forecaster + datasheet |

**Note méthodologique** : Données clés à intégrer dans le Forecaster M16 et le datasheet :
- Datacenters mondiaux : doublement à quadruplement empreinte carbone d'ici 2030 → 920 MtCO₂eq/an (2× émissions FR).
- Consommation élec datacenters 2030 : 1 250 à 1 500 TWh (× 2.3 à 2.8 en 7 ans).
- Part IA dans conso datacenters : 15 % en 2025 → 55 % en 2030.
- Modèles IA générative consomment **50 à 25 000 ×** plus que les modèles classiques.

Ces chiffres positionnent Sobr.ia dans une trajectoire critique et donnent au pitch une dimension prospective. Citer Shift Project dans le M16 ("voici la trajectoire si rien ne change, voici votre contribution personnelle") = impact pédagogique majeur.

---

# 📊 Matrice de priorisation (19 sources évaluées)

```
            Effort faible                  Effort élevé
          ┌──────────────────────────┬──────────────────────────┐
Valeur    │ QUICK WINS               │ STRATEGIC BETS           │
très éle. │ • ⭐⭐⭐ Mistral × ADEME  │                          │
(9-10/10) │ • ⭐⭐ Google Gemini      │                          │
          │ • ⭐⭐ IEA Energy and AI │                          │
          │ • ADEME Base Empreinte   │                          │
          │ • HF AI Energy Score     │                          │
          │ • ML.ENERGY v3.0         │                          │
          │ • Meta Llama 3.x cards   │                          │
          │ • ARCEP édition 2025     │                          │
          ├──────────────────────────┼──────────────────────────┤
Valeur    │ • EpochAI Models         │ • ENTSO-E (API key + XML)│
élevée    │ • ODRE complémentaire    │ • Boavizta API (CC BY-SA)│
(7-8/10)  │ • Shift Project (cit.)   │                          │
          │ • Cloud Carbon Footprint │                          │
          ├──────────────────────────┼──────────────────────────┤
Valeur    │ • LMSYS / LMArena        │ • MLPerf Power v6.0      │
moyenne   │ • NegaOctet (via S03)    │                          │
(5-6/10)  │                          │                          │
          ├──────────────────────────┴──────────────────────────┤
Watchlist │ ⚠️ ElectricityMaps (free tier non-commercial)       │
& Skip    │ ⏸ Anthropic Claude (pas de disclosure officielle)   │
          └─────────────────────────────────────────────────────┘
```

---

# 📋 Roadmap d'intégration recommandée (mise à jour batch 2)

| Priorité | Source | Effort estimé | Valeur attendue |
|---|---|---|---|
| 1 | **Mistral × ADEME ACV Large 2** | 0.5 j | Données vendor réelles dans preset + encadré M9 fiche modèle |
| 2 | **Google Gemini Environmental Disclosure** | 0.5 j | Données vendor Gemini + encadré M9 + table comparaison vendors |
| 3 | **Meta Llama 3.x model cards** | 0.5 j | Training disclosure + distinction location/market-based |
| 4 | ADEME Base Empreinte (API) | 1.5 j | Remplacement constantes facteurs émission par sources officielles FR |
| 5 | **ML.ENERGY Benchmark v3.0** | 1 j | Calibration empirique GPU + 4ème reference moteur |
| 6 | **IEA Energy and AI 2025** | 0.5 j | Citation référence internationale M16 Forecaster + datasheet |
| 7 | HF AI Energy Score | 1 j | Mapping score externe ↔ A-F Sobr.ia |
| 8 | EpochAI Models dataset | 1 j | Bulk-load presets modèles riche (FLOPs, params, dates) |
| 9 | ODRE complémentaire (4 sous-datasets) | 1.5 j | Granularité mix élec FR horaire + régional + EPCI |
| 10 | ARCEP édition 2025 (parse PDF) | 0.5 j | Citation officielle datacenters FR |
| 11 | Shift Project — projections 2030 | 0.5 j | Citation M16 Forecaster + datasheet |
| 12 | Cloud Carbon Footprint (extraction PUE) | 1 j | Facteurs PUE + carbon intensity cloud par région |
| **Total C31 v1.1** | | **~10 j** | |
| 13 | ENTSO-E Transparency Platform | 2 j (API key + XML) | v1.2+ — extension Europe |
| 14 | Boavizta API | 2 j (avec attribution) | v1.2+ — cross-validation ACV |
| 15 | MLPerf Power v6.0 | 2 j | v1.2+ — vue hardware-grade M9 |
| 16 | LMSYS / LMArena rankings | 1 j | v1.2+ — tradeoff qualité × empreinte M9 |
| 17 | ARCEP édition 2026 (dès publication) | 1 j | À surveiller |
| 18 | ElectricityMaps parsers GitHub (alternative) | 2 j | v1.2+ — internationalisation sans dep API freemium |
| 19 | NegaOctet | Couvert via S03 (pas duplication) | — |

---

# 🔜 À suivre (prochain batch)

**Couvert dans cette session (batch 1+2+3 partiel) — 16 sources** :
- ✅ Cat. A : ADEME, ARCEP, ODRE (+ 4 sous-datasets)
- ✅ Cat. B : ENTSO-E, IEA Energy and AI
- ✅ Cat. C : HF AI Energy Score, MLPerf Power, EpochAI
- ✅ Cat. D : Mistral × ADEME, Google Gemini
- ✅ Cat. E : Boavizta, NegaOctet, Shift Project, Cloud Carbon Footprint, ElectricityMaps (watchlist)

**À faire dans une prochaine session (batch 4)** :
- Cat. A complément : INSEE Sirene, IGN BD TOPO, CNIL études IA, ANCT, DataESR, BPI, data.gouv.fr search par mots-clés supplémentaires
- Cat. B complément : Eurostat, EEA, OWID, JRC, IRENA, World Bank, OECD.Stat, Climate TRACE
- Cat. C complément : HELM Stanford, LMSYS Chatbot Arena, ML.Energy leaderboard, MLPerf inference detailed, Papers With Code
- Cat. D restant : Anthropic Claude 3.5 (sustainability section), OpenAI GPT-4o (probable absence model card), Meta Llama 3.3 / 4, Cohere, xAI, DeepSeek, Alibaba Qwen, Microsoft Phi
- Cat. E complément : WattTime, Carbon Disclosure Project, Boavizta côté CC BY-SA détaillé
- **Synthèse finale 1 page jury data.gouv.fr** (à condenser à partir de la synthèse exécutive enrichie)
- **Skeleton brief `C31-integration-tier2-datasets.md`** avec découpage en sous-chantiers concrets

**Estimation reste** : 2-3 heures de search + édition pour 15-20 sources supplémentaires + finalisation. Réalisable en 1-2 sessions.

---

## Annexes

### A. Sources rejetées (préliminaire)

À constituer au fil du batch suivant. Critères de rejet :
- Paywall ou compte payant obligatoire
- Licence non compatible (proprio fermée)
- Hors périmètre Sobr.ia (impact environnemental IA générative)
- Données obsolètes (> 5 ans sans MAJ)

### B. Veille à mener (sources émergentes)

- Sénat — Commission empreinte environnementale IA (lancée 10 déc 2025) — surveiller publications 2026.
- Mistral AI publication empreinte (juillet 2025) — premier vendor à publier, "AI Nutri-Score" — à intégrer comme model card Cat. D.
- DataESR (recherche IA FR) — à explorer.
