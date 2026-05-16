# Chantier C30 — Audit datasets gouvernementaux + externes

> **Type** : chantier de **recherche** (pas d'implémentation code)
> **Version cible** : sortie attendue v1.0 candidature data.gouv.fr (= argument supplémentaire)
> **Sprint** : 3-4 jours, parallélisable à C29 (v0.7.1) ou en juste-après
> **Output** : `docs/sources/AUDIT-2026-Q3.md` (rapport d'audit ~30-50 sources évaluées) + `docs/sources/CATALOGUE-SOURCES.md` mis à jour v3.0 + roadmap d'intégration C31
> **Pré-requis** : v0.7.0 shippée, `docs/sources/CATALOGUE-SOURCES.md` v2.0 connu

---

## 0. Pourquoi cet audit

Sobr.ia v0.7.0 ingère 2 datasets Tier 1 (ComparIA + RTE-IRIS). Le CDC §8 mentionne ADEME, RTE eco2mix, HuggingFace, EcoLogits, CodeCarbon, ML.Energy, Papers, GeoLite2 — mais ces sources Tier 2/3 ne sont **pas** implémentées dans `sobria-ingest` à ce jour (squelettes only).

Avant la candidature data.gouv.fr v1.0 il est stratégique de :

1. **Cartographier exhaustivement** les sources disponibles (FR gouv + EU + monde open + académique + industrie),
2. **Évaluer chacune** sur des critères harmonisés (licence, fraîcheur, volume, valeur pour Sobr.ia),
3. **Prioriser** les 4-8 prochaines sources à intégrer (sprint C31 post-v1.0),
4. **Documenter** dans le dossier candidature comme preuve de sérieux méthodologique.

C'est un travail de **veille + classification**, pas de code. Le résultat alimente directement le pitch défi data.gouv.fr.

---

## 1. Périmètre

### En périmètre

- Audit exhaustif de **5 catégories** de sources (cf. §3).
- Application d'une **grille de scoring** harmonisée par source (cf. §4).
- Production d'un **rapport audit** : `docs/sources/AUDIT-2026-Q3.md` (~40 pages markdown).
- Mise à jour du **catalogue** : `docs/sources/CATALOGUE-SOURCES.md` v3.0 (ajouts Tier 2/3 + Tier 4 « watchlist »).
- Production d'une **roadmap d'intégration** : `briefs/chantiers/C31-integration-tier2-datasets.md` (skeleton du sprint suivant).

### Hors périmètre

- Toute **implémentation de code** (sobria-ingest, schémas Silver, etc.). C'est C31 plus tard.
- Téléchargement / parsing effectif des datasets identifiés (sauf échantillons pour valider l'accessibilité).
- Décisions d'architecture (gardent ADR-0009 inchangé).
- Datasets payants ou nécessitant compte d'entreprise/API key restrictive (rejet automatique cf. CATALOGUE-SOURCES.md v2.0 §"Engagement v1.0 zéro paywall").

---

## 2. Categories à auditer (5 verticales)

### Cat. A — Données gouvernementales françaises

Sources étatiques / publiques FR, généralement Etalab 2.0.

Liste indicative non exhaustive :
- **data.gouv.fr** (search par mots-clés : "IA", "intelligence artificielle", "datacenter", "consommation énergie", "modèles", "CO2", "empreinte", "GES", "numérique responsable")
- **ADEME Base Empreinte** ([base-empreinte.ademe.fr](https://base-empreinte.ademe.fr/))
- **ADEME Base Carbone** (équiv Bilan GES)
- **ARCEP** — observatoire numérique, baromètre du numérique, "Pour un numérique soutenable"
- **ODRE — Open Data Réseaux Énergies** (RTE, GRTgaz, NaTran, Teréga, Enedis, GRDF)
- **INSEE** — démographie entreprises, BSI (Base Sirene Internet), nomenclatures NAF
- **IGN** — BD TOPO, BAN (Base Adresse Nationale)
- **DataESR** (Ministère Enseignement Supérieur Recherche, IA dans la recherche FR)
- **BPI** — données ESG entreprises
- **Bpifrance Le Hub** — startups, IA française
- **Banque de France** — données macro
- **CRE** — Commission de Régulation de l'Énergie (référentiels électricité/gaz)
- **CNIL** — décisions et études IA + données personnelles
- **DGCCRF / DGE** — études impact numérique

### Cat. B — Données UE et internationales open

Sources européennes et globales en open data.

Liste indicative :
- **Eurostat** — énergie, environnement, technologie
- **EEA — European Environment Agency** — facteurs émission EU
- **JRC — Joint Research Centre** (Commission Européenne) — études cycle de vie numérique
- **ENTSO-E Transparency Platform** — mix électrique EU horaire
- **IEA — International Energy Agency** — données globales énergie
- **IRENA — International Renewable Energy Agency**
- **OECD.Stat** — économie numérique, IA
- **World Bank Open Data** — indicateurs développement
- **UN Statistics Division (SDG indicators)**
- **Our World in Data** — datasets curated, MIT (OWID)
- **Climate TRACE** — émissions globales par installation

### Cat. C — Académique + benchmarks IA

Sources de recherche, benchmarks, leaderboards.

Liste indicative :
- **Hugging Face Open LLM Leaderboard** + **HF AI Energy Score**
- **MLPerf Inference + Training Benchmarks** (MLCommons)
- **ML.Energy Leaderboard** (Univ. Michigan)
- **HELM** (Stanford CRFM, holistic evaluation)
- **EpochAI** (compute & training cost / data parameters estimates)
- **LMSYS Chatbot Arena** (rankings et votes humains)
- **Artificial Analysis** (benchmarks de latence + coût + qualité)
- **Papers With Code** — papers + leaderboards
- **arXiv + Semantic Scholar** — accès papers + abstracts
- **OpenReview** — peer review traces
- **MLCO2** — Lacoste et al. carbon calculator

### Cat. D — Cartes modèles industriels

Fiches techniques publiées par les éditeurs (Model Cards Gebru 2018).

Liste indicative :
- **OpenAI** — model cards GPT-4o, GPT-4, GPT-3.5, etc.
- **Anthropic** — Claude 3.5 Sonnet, Opus, Haiku
- **Google DeepMind** — Gemini 1.5, 2.0, Gemma
- **Meta AI** — Llama 3.1, 3.2, 3.3, 4 (si dispo), Code Llama
- **Mistral AI** — Mistral Large 2, Small 3, Codestral, Pixtral
- **Cohere** — Command R+, Aya
- **xAI** — Grok-2, Grok-3
- **Databricks** — DBRX
- **AI21** — Jamba
- **DeepSeek** — V3, R1
- **Alibaba** — Qwen 2.5, 3
- **01.AI** — Yi
- **Microsoft** — Phi-3, Phi-4

### Cat. E — Carbon-specific et géoloc

Sources spécialisées impact carbone et géolocalisation des infrastructures.

Liste indicative :
- **ElectricityMaps** — mix électrique horaire global (gratuit pour usage non-commercial uniquement, **à valider**)
- **WattTime** — signal carbon API (compte gratuit limité)
- **CDP — Carbon Disclosure Project** — émissions entreprises
- **Cloud Carbon Footprint** (open source, Thoughtworks)
- **Boavizta** — référentiel ACV équipement IT (FR, BSD-3)
- **Negaoctet** — base ACV numérique FR
- **GreenIT.fr** — référentiels et études
- **The Shift Project** — études "Numérique et environnement"
- **NegaWatt** — scénarios énergétiques FR
- **Open Compute Project (OCP)** — specs datacenters open

---

## 3. Grille de scoring (par source)

Chaque source identifiée est évaluée avec ce template uniforme :

```markdown
### S<NN> — <Nom court>

| Champ | Valeur |
|---|---|
| **Producteur** | (ex: Ministère de la Culture / DINUM) |
| **URL canonique** | (lien) |
| **Catégorie** | A / B / C / D / E |
| **Licence** | (Etalab 2.0 / CC-BY 4.0 / CC0 / MIT / Apache 2.0 / proprio / autre) |
| **Compatibilité licence Sobr.ia** | ✅ / ⚠️ / ❌ + justification |
| **Volume estimé** | (Mo, Go) |
| **Format** | CSV / JSON / Parquet / API / PDF / scraping |
| **Authentification** | ❌ Aucune / 🔑 API key gratuite / 💰 Payant |
| **Fréquence MAJ** | (live / quotidien / hebdo / mensuel / trimestriel / annuel / one-shot) |
| **Fraîcheur** | (date dernière MAJ amont si connue) |
| **Accessibilité tech** | API REST / dump direct / scraping / manuel |
| **Score valeur pour Sobr.ia** | /10 |
| **Score effort intégration** | /10 (1 = trivial, 10 = sprint complet) |
| **Tier proposé** | 1 / 2 / 3 / 4 (watchlist) |
| **Risques** | (ex: stabilité URL, changement format, GDPR) |
| **Note méthodologique** | 2-4 phrases |
| **Décision** | ✅ Intégrer C31 / 📋 Backlog v1.2+ / ⏸ Surveiller / ❌ Rejeté |
```

### Critères d'évaluation des deux scores

**Score valeur pour Sobr.ia (/10)** combine :

- Adéquation avec moteur AFNOR/EcoLogits (modèles, datacenters, mix électrique, facteurs émission) → 0–4 pts.
- Granularité géographique FR/EU (IRIS, régions, datacenters spécifiques) → 0–2 pts.
- Différenciateur pitch défi data.gouv.fr (souveraineté, qualité officielle) → 0–2 pts.
- Fraîcheur + fréquence de MAJ → 0–2 pts.

**Score effort intégration (/10)** combine :

- Volume + format (Parquet/CSV facile, PDF/scraping difficile) → 1–4 pts.
- Licence claire vs zone grise → 1–2 pts.
- Stabilité URL et schéma vs susceptibilité de breaks → 1–2 pts.
- Volume de transformations nécessaires en Silver/Gold → 1–2 pts.

### Quadrant de priorisation

```
            Effort faible          Effort élevé
          ┌──────────────────┬──────────────────┐
Valeur    │   QUICK WINS     │  STRATEGIC BETS  │
élevée    │   → C31 court    │  → C31 long /    │
          │                  │    v1.2+         │
          ├──────────────────┼──────────────────┤
Valeur    │   FILL-IN /      │     SKIP /       │
faible    │   BACKLOG        │    ⏸ Surveiller │
          │                  │    ou ❌         │
          └──────────────────┴──────────────────┘
```

---

## 4. Sous-chantiers

### C30.1 — Audit Cat. A (Gouv FR) — 1 jour

- Tour exhaustif de data.gouv.fr par 10+ mots-clés.
- Évaluation de chaque source ADEME, ARCEP, ODRE, INSEE, IGN, CNIL, etc.
- ~15-25 sources évaluées, ~5-10 retenues.

### C30.2 — Audit Cat. B + C (EU/global + académique) — 0.75 jour

- Survol Eurostat, EEA, IEA, OWID, ENTSO-E.
- Survol HF leaderboards, MLPerf, ML.Energy, HELM, EpochAI, LMSYS Arena.
- ~15-20 sources évaluées, ~5-8 retenues.

### C30.3 — Audit Cat. D (modèles industriels) — 0.5 jour

- Tour des model cards OpenAI / Anthropic / Google / Meta / Mistral / Cohere / xAI / Databricks / AI21 / DeepSeek / Alibaba / 01.AI / Microsoft.
- Vérifier disponibilité champs : nb paramètres, training compute (FLOPs), training energy (kWh), training CO₂eq, hardware, durée training, dataset volume.
- ~10-15 model cards évaluées, objectif : enrichir presets modèles de Sobr.ia.

### C30.4 — Audit Cat. E (carbon-specific) — 0.5 jour

- ElectricityMaps : revérifier conditions licence non-commerciale.
- Boavizta : licence BSD-3, comparer avec ADEME.
- The Shift Project / NegaWatt / Cloud Carbon Footprint / Negaoctet.
- ~10 sources évaluées.

### C30.5 — Rapport + catalogue + roadmap — 1 jour

- Production de `docs/sources/AUDIT-2026-Q3.md` (~40 pages markdown : intro + méthodologie + 50 sources évaluées + matrice de priorisation + conclusion).
- Mise à jour `docs/sources/CATALOGUE-SOURCES.md` v3.0 : ajouter Tier 2/3 nouvelles retenues + section Tier 4 « watchlist » pour celles à surveiller.
- Skeleton `briefs/chantiers/C31-integration-tier2-datasets.md` : liste des 4-8 sources prioritaires à intégrer dans C31 + estimation effort par source.

---

## 5. Definition of Done

- [ ] `docs/sources/AUDIT-2026-Q3.md` produit, ≥ 40 sources évaluées avec template harmonisé.
- [ ] Matrice quadrant complétée (4 zones).
- [ ] ≥ 4 sources « quick wins » identifiées pour C31.
- [ ] ≥ 2 sources « strategic bets » identifiées pour v1.2+.
- [ ] `docs/sources/CATALOGUE-SOURCES.md` v3.0 mis à jour.
- [ ] `briefs/chantiers/C31-integration-tier2-datasets.md` skeleton créé.
- [ ] 1 page de synthèse exécutive en tête du rapport audit (pitch defi data.gouv.fr ready).
- [ ] Pas de modification de code Rust/TS (pure documentation).
- [ ] Commit Conventional : `docs(sources): C30 audit datasets gouv + externes (40 sources)`.
- [ ] Pas de tag versionné (c'est de la doc, pas un release).

---

## 6. Anti-périmètre

- Téléchargement effectif des datasets (sauf échantillons).
- Implémentation de nouvelles sources dans `sobria-ingest`.
- Modification des sources existantes (ComparIA, RTE-IRIS).
- Pricing ou business model (ADR-0014 séparé).
- Décisions juridiques GDPR définitives (à valider avec DPO en C-cloud-beta).

---

## 7. Timing recommandé

Trois options :

| Option | Description | Pour | Contre |
|---|---|---|---|
| **A** | C30 **avant** v1.0 candidature (en parallèle de C29) | Pitch candidature renforcé ("30+ sources cataloguées, roadmap d'intégration publiée") | Délais de 3-4 jours sur la candidature |
| **B** | C30 **après** v1.0 candidature | Candidature sur le périmètre actuel sans délai | Manque un argument à la candidature |
| **C** | C30 **en parallèle** de v1.0 candidature (Cowork audit pendant que Claude Code finalise la candidature) | Pas de délai + argument enrichi | Cognitive load double |

**Recommandation** : option **A** ou **C**. C30 est pure recherche, ça ne bloque pas le code v1.0. Si je (Cowork) prends C30 pendant que Claude Code prend la candidature, on parallélise sainement.

---

## 8. Output attendu

Le rapport `docs/sources/AUDIT-2026-Q3.md` doit avoir cette structure :

```markdown
# Audit datasets Sobr.ia — Q3 2026

## Synthèse exécutive (1 page)
- 50 sources évaluées en 5 catégories.
- 8 sources retenues pour intégration C31 (5 gouv FR, 2 académique, 1 industrie).
- 12 sources backlog v1.2+.
- 30 sources rejetées (paywall, licence non compatible, hors périmètre).

## Méthodologie
- Critères de scoring (cf. brief C30).
- Catégories.
- Grille uniforme.

## Catégorie A — Gouv FR (15 sources)
### S01 — ADEME Base Empreinte
...
### S15 — DataESR
...

## Catégorie B — EU/Global (10 sources)
...

## Catégorie C — Académique + benchmarks (10 sources)
...

## Catégorie D — Cartes modèles industriels (10 sources)
...

## Catégorie E — Carbon-specific + géoloc (5 sources)
...

## Matrice de priorisation
[Quadrant valeur × effort]

## Roadmap d'intégration recommandée
- C31 (v1.1) : 8 sources quick wins.
- v1.2+ : 12 sources strategic bets.

## Annexes
- Sources rejetées + motif.
- Veille à mener (sources émergentes).
```

Le rapport doit être **lisible par un jury data.gouv.fr** (pas du jargon ingénieur pur) — c'est un livrable pitch en plus d'un livrable technique.
