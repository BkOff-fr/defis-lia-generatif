# Dossier candidature — Défi data.gouv.fr

## « L'impact environnemental de l'IA générative »

---

**Projet** : Sobr.ia
**Auteur** : Thibault (étudiant, contributeur Sobr.ia)
**Date** : Mai 2026 — **dernière mise à jour : 2026-06-12**
**Lien dépôt** : <https://github.com/BkOff-fr/defis-lia-generatif>
**Licence code** : MIT
**Statut** : v0.9.0 (taguée 2026-05-20) — 13 modules app livrés et testés,
plus extension navigateur MV3 (Chrome + Firefox), Mode Équipe self-hosted,
site vitrine et mode démo web (cf. §6 bis)

---

## 1. Résumé exécutif (1 page)

**Sobr.ia** est une application native multi-plateforme (Windows / macOS /
Linux / Web) qui mesure l'empreinte environnementale de l'usage d'IA
générative (LLM) avec **rigueur scientifique**, **transparence
méthodologique** et **un angle territorial français unique**.

L'application répond aux trois axes du défi data.gouv.fr :

1. **Comprendre l'impact** : **catalogue de méthodologies** (AFNOR SPEC
   2314 par défaut + EcoLogits 2026-01 peer-reviewed). Monte-Carlo
   N=10⁴ tirages sur l'engine AFNOR, port direct des formules
   EcoLogits reproduit à ≤ 1 %. Calculs reproductibles en Python dans
   [`notebook/validation.qmd`](../notebook/validation.qmd).
2. **Visualiser** : 13 modules dont 7 dataviz (carte Europe Leaflet,
   Sankey énergétique, distributions log-normales bornées, waterfall
   contributions, time series cumulatives) + nouvelle page **catalogue
   méthodologies** + panneau **« Voir aussi »** comparatif côté Atelier.
3. **Sensibiliser & changer** : 5 personas (étudiant·e, pro tech,
   entreprise, collectivité, chercheur·se) avec bundles modulaires,
   eco-budget personnel, simulateur « Et si...? ».

**Quatre différenciateurs majeurs** par rapport à l'état de l'art :

- **🏛️ Catalogue souverain de méthodologies** *(différenciateur C24,
  unique au monde)* : Sobr.ia embarque **deux méthodologies
  scientifiques** d'estimation au choix de l'utilisateur — AFNOR SPEC
  2314 (référentiel français) et EcoLogits 2026-01 (peer-reviewed JOSS,
  [doi:10.21105/joss.07471](https://doi.org/10.21105/joss.07471), port
  direct CC BY-SA 4.0). L'utilisateur peut activer la seconde *« en
  référence »* pour comparer les écarts dans l'Atelier. **Aucun
  concurrent ne fait ça** : EcoLogits, BoaVizta, AI Energy Score,
  GreenAlgorithms sont tous mono-méthodologie. Architecture extensible
  v1.1+ (BoaVizta, Custom user CSV, etc.). Cf. [ADR-0012](adr/ADR-0012-multi-methodology-engine.md).
- **🔐 Audit ledger chaîné SHA-256 avec méthodologie tracée** : chaque
  estimation est journalisée dans une chaîne cryptographique
  anti-tampering vérifiable, **avec sa méthodologie** (colonne `method`
  du ledger v2). Un rapport CSRD régénéré 2 ans plus tard utilise
  exactement la méthodologie qui était active au moment des calculs.
  Aucun concurrent ne propose cette garantie de traçabilité
  méthodologique historique.
- **🇫🇷 Pivot Territoire FR** : croisement entre les datasets officiels
  **ComparIA** (Beta.gouv / Ministère de la Culture) et **RTE IRIS
  sites industriels** (ODRÉ, Etalab 2.0). Aucun outil existant ne fait
  ce croisement à la maille IRIS française.
- **📑 Datasheet Gebru 2018** : génération automatique du format
  académique standard pour reproductibilité scientifique. Permet la
  publication conjointe d'articles de recherche et de leurs traces
  d'estimation.

**Frugalité incarnée** : le code applicatif est en Rust + Tauri 2 (pas
d'Electron), binaire ≈ 15 MB optimisé, zéro télémétrie, zéro appel
réseau au runtime, données stockées en SQLite local avec chiffrement WAL.

---

## 2. Contexte du défi

Le défi data.gouv.fr 2026 invite à proposer des outils d'**accélération
de la connaissance et de la transparence** sur l'impact environnemental
de l'IA générative. Les axes prioritaires sont :

- Exploitation des **datasets officiels** : ComparIA (5 GB de
  conversations + votes sur LLMs, méthodologie EcoLogits intégrée),
  RTE IRIS (consommation électrique et gaz par maille IRIS, Etalab 2.0).
- **Angle territorial français** : maille IRIS, croisement avec les
  données de consommation industrielle.
- **Méthodologie scientifique** : référentiel AFNOR SPEC 2314, validation
  croisée contre la littérature.
- **Ouverture** : licences ouvertes (MIT / Etalab 2.0 / CC-BY).

Sobr.ia répond à ces 4 axes simultanément.

---

## 3. Méthodologie scientifique

### 3.1 Formule de référence

Conformément à AFNOR SPEC 2314 (§ 7.2), l'empreinte d'une requête
d'inférence LLM est calculée par tirage Monte-Carlo :

```
Pour chaque tirage k ∈ [1, N=10⁴] :
  E_compute_k = T_in × ε_prefill_k + T_out × ε_decode_k                 (mJ)
  E_total_k   = (E_compute_k × PUE_k) / 3 600 000                       (Wh)
  CO2eq_k     = (E_total_k / 1000) × IF_k + embodied_k                  (g)
  Water_k     = (E_total_k / 1000) × WUE_k                              (L)

Agrégation :
  P5  = quantile(values, 0.05)
  P50 = quantile(values, 0.50)
  P95 = quantile(values, 0.95)
```

### 3.2 Distributions paramétriques

| Paramètre | Distribution | Source |
|---|---|---|
| ε_prefill_mj_per_token | LogNormal | HF AI Energy Score 2026 |
| ε_decode_mj_per_token | LogNormal | HF AI Energy Score 2026 |
| PUE (datacenter) | Uniform [1.05, 1.6] | Rapports sustainability operators |
| IF_electrical_g_per_kwh | Point (mix horaire) | RTE / ADEME / Electricity Maps |
| embodied_g_per_request | LogNormal | Gupta 2022, amorti |
| WUE_l_per_kwh | Uniform [0.0, 5.0] | Mytton 2021 |

Le détail de calibration par modèle (registre v0.9.0 : **34 presets, dont
26 actifs** — catalogue 2026 : Claude 4.x, GPT-5.5, Gemini 3.x, Llama 4,
Mistral Large 3, DeepSeek V4, Grok 4, Qwen 3.6, Phi-4 reasoning — et
**8 presets dépréciés** — les anciens modèles de la candidature initiale
(GPT-4o, Claude 3.5 Sonnet, Llama 3.1…), conservés pour la
reproductibilité de l'audit ledger) est documenté dans
[`docs/methodology/MODEL-PRESETS.md`](methodology/MODEL-PRESETS.md), la
source de vérité étant le code
([`crates/sobria-estimator/src/model_presets.rs`](../crates/sobria-estimator/src/model_presets.rs)).

### 3.3 Reproductibilité

- **Seed déterministe** : `SOBRIA_SEED=42` (configurable via env var).
  Même seed + mêmes paramètres → résultat identique à la nanoseconde.
- **RNG** : Xoshiro256++ (qualité statistique excellente, vitesse élevée).
- **Histogramme distributionnel** : 50 bins équi-width persistés dans
  chaque entrée d'audit, pour rejouer la distribution complète.

### 3.4 Validation croisée — port direct EcoLogits

L'audit B (mai 2026) a établi qu'aucune calibration unique d'une formule
linéaire-par-token ne peut reproduire EcoLogits à ±15 % sur toute la
gamme de modèles (8B → 200B), à cause de la non-linéarité d'EcoLogits
(terme γ + facteur `n_GPU` discret + server overhead non-GPU). Plutôt
que de bricoler un coefficient, **Sobr.ia v1.0 embarque les formules
EcoLogits intégralement** dans un moteur dédié, en plus du moteur AFNOR
SPEC 2314 (cf. ADR-0012).

**Trois `ReproductionCase` ciblent l'`EcoLogitsEngine`** — port direct
de leurs formules (`f_E`, `f_L`, `n_GPU`, `E_server_noGPU`, embodied)
depuis la documentation publique (Rincé & Banse 2025, JOSS 2025,
[doi:10.21105/joss.07471](https://doi.org/10.21105/joss.07471), CC BY-SA
4.0). Les cibles sont recalculées en Python *de zéro* dans
[`notebook/validation.qmd`](../notebook/validation.qmd), puis comparées
au port Rust à tolérance **1 %** (port direct → seule l'arithmétique
float64 introduit du bruit) :

| ID | Modèle | tokens (in/out) | Mix élec | Cible Python g | Rust port g | Écart |
|---|---|---:|---:|---:|---:|---:|
| A | Llama 3.1 70B | 100/500 | FR 56 g/kWh | 0.01843 | 0.01842 | **-0.08 %** ✓ |
| B | Llama 3.1 70B | 100/2000 | US-VA 412 g/kWh | 0.542 | 0.54195 | **-0.01 %** ✓ |
| C | Mistral Large 2 | 100/1000 | US-VA 412 g/kWh | 0.378 | 0.37712 | **-0.23 %** ✓ |

**L'engine AFNOR Sobr.ia** est validé séparément par 6
`PlausibilityCase` couvrant petits/gros modèles × mix FR/US-VA ×
prompts courts/longs (plages 3-5 ordres de grandeur, garde-fou contre
les bugs catastrophiques). Sa calibration `K_DECODE_MJ_PER_TOKEN_PER_B`
a été corrigée d'un facteur 1000 en mai 2026 (audit B → C24) pour
s'aligner sur les mesures HF AI Energy Score et ML.ENERGY.

**Au runtime**, l'utilisateur compare lui-même les deux méthodologies
côte-à-côte via le panneau « Voir aussi » de l'Atelier — pas besoin de
nous croire sur parole.

Tous les cas tournent en CI :
```bash
cargo test -p sobria-estimator validation
quarto render notebook/validation.qmd
```

Cf. [`docs/methodology/VALIDATION-CROISEE.md`](methodology/VALIDATION-CROISEE.md),
[ADR-0012 multi-méthodologie](adr/ADR-0012-multi-methodology-engine.md),
et [`crates/sobria-estimator/src/validation/cases.rs`](../crates/sobria-estimator/src/validation/cases.rs).

---

## 4. Différenciateurs et innovations

### 4.1 Catalogue souverain de méthodologies (C24)

**Le différenciateur central de Sobr.ia v1.0.** Le constat à l'origine
du chantier C24 (mai 2026) : tous les outils d'estimation d'empreinte
LLM existants sont **mono-méthodologie**. EcoLogits propose sa méthode,
BoaVizta la sienne, AI Energy Score la sienne, GreenAlgorithms la
sienne. L'utilisateur final ne peut pas comparer, est obligé de croire
sur parole, et perd toute traçabilité méthodologique si l'outil change
ses formules.

**Sobr.ia inverse la logique** : la méthodologie devient un **objet de
premier ordre** que l'utilisateur sélectionne, et dont la trace est
journalisée dans l'audit ledger. Concrètement :

1. **Catalogue exposé via la page `/methodologies`** : chaque méthodo
   disponible est présentée avec son DOI, sa licence, son année de
   publication et son statut de calibration (peer-reviewed reproduit /
   méthode publique en cours de calibration / indicative).
2. **Sélection par l'utilisateur** : une méthodologie par défaut + une
   ou plusieurs méthodologies activables en référence (panneau « Voir
   aussi » à côté du résultat principal côté Atelier).
3. **Traçabilité historique** : chaque entrée du ledger d'audit
   porte sa méthodo dans la colonne `method` (migration v2 idempotente
   pour les ledgers préexistants).
4. **Architecture extensible** : un trait Rust `EmpreinteEngine`
   commun, factory `engine_for(method)`. Ajouter `BoaViztaEngine` ou
   `AIEnergyScoreEngine` en v1.1+ = implémenter le trait + une entrée
   dans le registry compile-time.

**v1.0 embarque 2 méthodologies** :

| Méthodologie | Description | Validation Sobr.ia |
|---|---|---|
| **AFNOR SPEC 2314 (Sobr.ia)** | Référentiel français officiel, formule linéaire-par-token + Monte-Carlo N=10⁴, intervalles P5/P50/P95 distributionnels. Méthodologie souveraine native. | 6 PlausibilityCase (ordres de grandeur), calibration en cours |
| **EcoLogits 2026-01** | Méthode peer-reviewed JOSS 2025, port direct des formules officielles (`f_E`, `f_L`, `n_GPU`, server overhead, embodied). Référence internationale. | 3 ReproductionCase reproduits **à ≤ 1 %** vs cibles Python |

**Pourquoi c'est unique au monde** :
- Aucun concurrent ne propose plusieurs méthodologies simultanées avec
  switch utilisateur. Sobr.ia est *le premier outil*.
- La **souveraineté méthodologique française** est préservée : AFNOR
  SPEC 2314 reste la méthodo par défaut au premier lancement, pas un
  citoyen de seconde zone vs EcoLogits.
- **Honnêteté radicale** : Sobr.ia ne cache pas les écarts entre
  méthodologies — au contraire, il les *montre* à l'écran pour que
  l'utilisateur prenne sa décision en connaissance de cause.

**Use case démonstratif** : un journaliste compare l'empreinte d'un
article généré par Claude 3.5 Sonnet selon les 2 méthodologies. Si elles
divergent de 200 %, c'est un sujet d'enquête en soi. Si elles convergent
à ≤ 10 %, son lectorat peut prendre l'estimation au sérieux. Dans les
deux cas, Sobr.ia donne *la matière à creuser*, pas un chiffre
imposé.

Cf. [ADR-0012 Multi-méthodologie](adr/ADR-0012-multi-methodology-engine.md),
[`briefs/chantiers/C24-multi-methodologie-ecologits.md`](../briefs/chantiers/C24-multi-methodologie-ecologits.md)
et [`crates/sobria-estimator/src/engine_trait.rs`](../crates/sobria-estimator/src/engine_trait.rs).

### 4.2 Audit ledger SHA-256 chaîné

Chaque estimation produit une entrée dans le ledger SQLite ACID + WAL,
avec :

```
sig_i = SHA256(timestamp_i || estimation_result_json_i || prev_sig_i)
```

Toute modification *a posteriori* (intrusion, manipulation) **casse la
chaîne** et est détectée par `verify_chain()`. Un export NDJSON signé
permet l'audit externe.

**RGPD** : la commande `purge_audit_before` remplace le payload des
entrées par un sentinel `"PURGED"` tout en **conservant la signature
originale**. Le droit à l'oubli est respecté, la chaîne reste vérifiable
cryptographiquement.

Cf. [`crates/sobria-audit/`](../crates/sobria-audit/) — 13 tests dont
2 tests anti-tampering explicites.

### 4.3 Croisement ComparIA × RTE IRIS

Le module **M20 Territoire FR** consomme deux datasets officiels :

- **ComparIA** (5 GB Parquet) — conversations + votes utilisateurs
  français sur LLMs.
- **RTE IRIS sites industriels** (ODRÉ) — consommation électrique et
  gaz par maille IRIS, données mises à jour annuellement.

Le pipeline `sobria-ingest` télécharge ces données via l'API ODRÉ
(`https://odre.opendatasoft.com/api/explore/v2.1/`), filtre les 200
sites industriels les plus consommateurs, et produit un JSON traçable
(SHA-256 + URL source + timestamp UTC).

**Le frontend M20** présente :

- Une carte Leaflet avec markers individuels par IRIS et agrégat
  régional (13 régions ADMIN1).
- Un **Sankey énergétique** national alimenté par le mix RTE eco2mix
  annuel (nucléaire, hydraulique, éolien, solaire, gaz, charbon, fioul,
  bioénergies, pompage, échange net export/import).
- Une projection : « si X% du trafic IA basculait vers la région Y,
  quel serait l'impact CO₂eq ? » (réutilise le moteur Monte-Carlo).

Cf. [`docs/sources/CATALOGUE-TERRITOIRE-FR.md`](sources/CATALOGUE-TERRITOIRE-FR.md).

### 4.4 Datasheet Gebru 2018

Le module **M17 Empreinte projet** génère automatiquement un
**datasheet JSON-LD** selon le standard académique Gebru et al. 2018
(arXiv:1803.09010), adopté par les conférences NeurIPS, ICML, FAccT et
les revues scientifiques majeures.

Les 7 sections sont automatiquement remplies depuis le ledger d'audit :

1. Motivation (texte utilisateur)
2. Composition (agrégat ledger : nb requêtes, modèles, totaux P50)
3. Collection process (paramètres Monte-Carlo, seed, N)
4. Preprocessing (aucun — valeurs Monte-Carlo brutes)
5. Uses (texte utilisateur)
6. Distribution (licences, hash SHA-256, URI canonique)
7. Maintenance (contact, version, date dernière mise à jour)

Le JSON-LD combine **4 vocabulaires standards** : schema.org/Dataset,
W3C PROV-O, DCAT, Sobr.ia. Validable par n'importe quel parseur
JSON-LD standard (jsonld-cli, RDF4J, etc.).

**Use case** : un chercheur écrit un article sur l'empreinte de Claude
Sonnet sur 500 prompts éducatifs collectés en Q1 2026. Il crée un
projet dans Sobr.ia avec ces 500 estimations, génère le datasheet
JSON-LD, le joint à son papier. Tout reproducteur peut télécharger le
JSON-LD, vérifier le SHA-256, rejouer les estimations avec le même
seed → résultats identiques.

### 4.5 Rapport CSRD/AGEC

Le module **M22 Rapport CSRD/AGEC** génère un **PDF officiel** pour le
reporting réglementaire :

- **CSRD** (UE 2024+, directive 2022/2464) : les entreprises >250
  salariés doivent reporter leur scope 3 numérique, IA incluse.
- **AGEC** (loi française 2020) : mesure de l'empreinte numérique pour
  collectivités.
- **AFNOR SPEC 2314** : référentiel français de mesure IA.

Le PDF inclut :
- Page de garde avec organisation + période + version Sobr.ia
- Synthèse exécutive (3 indicateurs avec intervalles d'incertitude)
- Méthodologie complète + sources
- Tableau détaillé par modèle
- Excerpt audit ledger (chaîne SHA-256)
- Provenance (lien vers JSON-LD PROV-O joint)
- Annexe : glossaire + références bibliographiques

**SHA-256 du PDF inscrit dans le PROV-O** → reproductibilité bout en bout.

---

## 5. État technique et qualité

### 5.1 Code

- **11 crates Rust** : ~43 000 lignes de code, 770 tests.
- **Frontend SvelteKit 2 + Svelte 5 runes + TypeScript strict** :
  20 routes, design system custom (ink/lime/ivory + Instrument Serif +
  Geist + JetBrains Mono).
- **clippy `-D warnings`** propre sur tout le workspace.
- **`cargo deny`** + **`cargo audit`** dans la CI : 0 vulnérabilité.

### 5.2 Tests

- **770 tests Rust** (727 `#[test]` + 43 `#[tokio::test]`, comptés sur
  `crates/` au 2026-06-12).
- **Tests property-based** (proptest) sur les invariants Monte-Carlo
  (P5 ≤ P50 ≤ P95, valeurs finies).
- **Tests d'intégration** : 1 test E2E qui exécute le pipeline complet
  Copper → Silver → Gold avec ComparIA + RTE IRIS.
- **Tests Playwright** : 18 fichiers de spec, dont les contrats "no-mock"
  qui valident que sans Tauri runtime, l'erreur typée
  `tauri_unavailable` s'affiche correctement.

### 5.3 Sécurité

- **`rustls`** (pas d'OpenSSL) pour HTTPS.
- **CSP stricte** Tauri : `default-src 'self'; connect-src 'self' ipc:`.
- **Capabilities minimales** : `core:default` + `dialog:default` uniquement.
- **`#![deny(unsafe_code)]`** sur toutes les crates.

### 5.4 Performance

- **Estimation Monte-Carlo** : 5-20 ms à N=10⁴.
- **Batch CSV** : 1000 lignes en ~10 secondes (sequential).
- **Démarrage app** : < 2 secondes (cold start sur SSD).
- **Binaire release** : ≈ 15 MB après LTO + strip + opt-level=z.

---

## 6. Vue d'ensemble des 13 modules

| Module | Persona cible | Statut |
|---|---|:--:|
| M1 Estimer | Tous | ✅ |
| M3 Comparer modèles | Pro tech / Chercheur | ✅ |
| M7 Journal d'audit | Pro tech / Entreprise / Chercheur | ✅ |
| M8 Méthodologie | Étudiant / Pro / Collectivité / Chercheur | ✅ |
| M9 Référentiel modèles | Pro tech / Chercheur | ✅ |
| M12 Datacenters Europe | Entreprise / Collectivité | ✅ |
| M13 Simulateur « Et si...? » | Étudiant / Pro | ✅ |
| M14 À propos | Tous | ✅ |
| M15 Dashboard personnel | Étudiant / Entreprise | ✅ |
| M17 Empreinte projet | Entreprise / Collectivité / Chercheur | ✅ |
| M20 Territoire FR | Entreprise / Collectivité | ✅ |
| M22 Rapport CSRD/AGEC | Entreprise / Collectivité | ✅ |
| M25 Eco-budget | Étudiant / Entreprise | ✅ |

**Modules différés v1.1+** (11) : voir [ADR-0011](adr/ADR-0011-reduction-perimetre-v1-0.md).

---

## 6 bis. Périmètre livré depuis la rédaction initiale (v0.3 → v0.9.0)

Les versions v0.4.0 à v0.9.0 (taguée 2026-05-20, voir
[CHANGELOG.md](../CHANGELOG.md)) ont livré, au-delà des 13 modules app :

- **Extension navigateur MV3 (Chrome + Firefox)** — capture en conditions
  réelles sur les interfaces ChatGPT, Claude et Le Chat, avec pairing à
  l'application (v0.6.0, [ADR-0005](adr/ADR-0005-webextension-mv3.md),
  [ADR-0013](adr/ADR-0013-extension-pairing-team-mode.md)).
- **Mode Équipe self-hosted** (v0.7.0+) — agrégation d'équipe
  auto-hébergée (crate `sobria-team-aggregator`), opt-in et **k-anonymat**
  ([ADR-0015](adr/ADR-0015-privacy-mode-equipe.md)), **politiques de
  visibilité** choisies au déploiement (anonymous / opt_in / identified
  avec attestation, [ADR-0016](adr/ADR-0016-politique-visibilite-deploiement.md)).
- **Site vitrine** (`site/`) — présentation publique du projet.
- **Mode démo web** — l'app SvelteKit est consultable hors Tauri avec
  bannière démo et données d'exemple.
- **Catalogue modèles 2026** (v0.9.0, chantier C34) — registre porté à
  **34 presets (26 actifs + 8 dépréciés)** avec modalités vision / audio /
  reasoning et overhead de contexte.

---

## 7. Roadmap post-candidature (v1.1+)

| Module | Description | Effort estimé |
|---|---|---|
| **M11 Extension navigateur** | Chrome + Firefox MV3, capture vie réelle | ✅ **Livré** (v0.6.0, 2026-05-16) |
| **M16 Forecaster UI** | Backend prêt — manque UI Chart.js sliders live | 1 semaine |
| **M18 Batch CSV UI** | Backend prêt — manque drag-and-drop frontend | 3-5 jours |
| **M21 Alertes système** | Notifications OS (winrt/macOS/Linux) | 1 semaine |
| **M19 Équipe** | Auth backend + partage multi-utilisateurs | ✅ **Livré** (Mode Équipe self-hosted, v0.7.0+, ADR-0015/0016) |
| **M2 Workbench** | Éditeur multi-prompts interactif | 2-3 semaines |
| **M24 Apprendre** | 10 mini-cours markdown sur prompting frugal | 2 semaines |
| **M23 Marchés publics** | Cahier des charges type + critères AO | partenariat institutionnel |
| **M5 / M6 / M10** | Modules redondants — différé sine die | — |

---

## 8. Comparaison avec l'état de l'art

| Outil | Multi-méthodologies | Audit chaîné | Territoire FR | Datasheet Gebru | Privacy local | Méthodo AFNOR FR |
|---|:--:|:--:|:--:|:--:|:--:|:--:|
| **Sobr.ia** | ✅ **AFNOR + EcoLogits** *(C24)* | ✅ | ✅ | ✅ | ✅ | ✅ |
| EcoLogits | ❌ (leur méthodo seule) | ❌ | ❌ | ❌ | ⚠️ (lib) | ❌ |
| AI Energy Score | ❌ | ❌ | ❌ | ❌ | ❌ (web) | ❌ |
| GreenAlgorithms | ❌ | ❌ | ❌ | ❌ | ❌ (web) | ❌ |
| BoaVizta IA | ❌ | ❌ | ❌ | ❌ | ⚠️ (lib) | ❌ |
| CarbonAware SDK | ❌ | ❌ | ❌ | ❌ | ⚠️ | ❌ |

Sobr.ia est, à notre connaissance, **le premier outil au monde à
embarquer plusieurs méthodologies scientifiques d'estimation d'empreinte
LLM avec switch utilisateur et audit cryptographique de la méthodologie
utilisée**. Combinée au territoire FR, à la datasheet Gebru et au
local-first, c'est une combinaison unique.

---

## 9. Démos clés (3 scenarios)

### Scenario 1 — Étudiant·e curieux·se

1. Premier lancement : onboarding, choix « 🎓 Étudiant·e ».
2. Bundle pré-coché : M1, M8, M13, M14, M15, M25.
3. Estime son premier prompt sur `/` (Estimer) → résultat P5/P50/P95
   instantané + équivalents parlants (× km voiture, × douches).
4. Va sur `/m25` (Eco-budget), fixe un objectif mensuel de 100 gCO₂eq.
5. Continue à estimer sur 1 semaine → revient sur `/m15` (Dashboard) :
   voit son évolution journalière, son delta vs semaine précédente, ses
   top modèles utilisés.
6. Va sur `/simuler` (Simulateur Et si) → joue avec les 7 leviers, voit
   le levier dominant pour son profil (souvent embodied carbon sur gpt-mini).

### Scenario 2 — Entreprise (DSI / RSE)

1. Onboarding, choix « 🏢 Entreprise ».
2. Bundle pré-coché : M1, M7, M12, M14, M15, M17, M20, M22, M25.
3. Estime 500 prompts via M22 (Batch CSV en v1.1, ou via API IPC
   directement v1.0).
4. Va sur `/m20` (Territoire FR) → voit la part de consommation
   industrielle française imputable à son usage IA.
5. Va sur `/rapport-csrd` (Rapport CSRD) → entre nom organisation +
   période Q1 2026 → génère un PDF officiel signé SHA-256 + JSON-LD
   PROV-O.
6. Joint le PDF au rapport ESG annuel obligatoire CSRD.

### Scenario 3 — Chercheur·se / Journaliste

1. Onboarding, choix « 🔬 Chercheur·se ».
2. Bundle pré-coché : M1, M3, M7, M8, M9, M14, M17.
3. Va sur `/modeles` (Référentiel modèles) → consulte les fiches
   détaillées des 26 modèles actifs du registre (34 presets au total)
   avec triplets P5/P50/P95 et sources documentaires.
4. Va sur `/comparer` (Comparer) → benchmark 4 modèles sur un prompt
   identique, voit le classement par CO₂eq / énergie / eau.
5. Crée un projet sur `/m17` (Empreinte projet) : « Étude Q1 2026
   Claude Sonnet », période Jan-Mars.
6. Génère le datasheet Gebru JSON-LD → joint au papier de recherche.
   Tout reviewer peut vérifier la reproductibilité.

### Scenario 4 — Journaliste tech / Investigation comparative *(C24)*

1. Onboarding, choix « 🔬 Chercheur·se ».
2. Va sur `/methodologies` (rail Audit) → découvre le catalogue.
   Méthodologie par défaut : AFNOR SPEC 2314 (Sobr.ia). Coche **EcoLogits
   2026-01** dans "Afficher en référence".
3. Retourne sur `/` (Atelier). Estime un prompt typique de son cas
   d'étude (résumé d'article, génération de 800 tokens) sur Llama 3.1
   70B avec mix électrique FR.
4. Le résultat AFNOR Sobr.ia s'affiche en grand. **Juste en dessous, le
   panneau "Voir aussi" affiche automatiquement le résultat EcoLogits**
   avec :
   - CO₂eq P50 EcoLogits
   - Énergie P50 EcoLogits
   - **Écart relatif vs Sobr.ia** (par ex. `+12 %`, lime si > 0)
   - Lien vers l'audit ID correspondant (`#42`)
   - Lien DOI vers la documentation EcoLogits
5. **Cas où les méthodos divergent (>50 %)** : le journaliste a un sujet
   d'enquête — pourquoi les deux outils peer-reviewed ne convergent
   pas ? Quelle hypothèse interne diffère ?
6. **Cas où elles convergent (<10 %)** : son lectorat peut prendre le
   chiffre au sérieux (deux méthodos indépendantes le confirment).
7. Dans les deux cas, l'audit ledger conserve les deux estimations avec
   leur méthodologie tracée (`method = 'afnor_sobria'` et `method =
   'ecologits'`). Reproductible 5 ans plus tard, même si EcoLogits ou
   AFNOR évoluent dans l'intervalle.

---

## 10. Conclusion

Sobr.ia répond aux 4 axes du défi data.gouv.fr :

1. **Comprendre** : **catalogue de méthodologies scientifiques** (AFNOR
   SPEC 2314 par défaut + EcoLogits 2026-01 peer-reviewed, port direct
   reproduit à ≤ 1 %). Monte-Carlo N=10⁴ sur l'engine AFNOR,
   déterministe sur l'engine EcoLogits, transparence complète des
   paramètres et des hypothèses (cliquables vers les sources).
2. **Visualiser** : 7 dataviz interactives (carte EU, Sankey, histogrammes,
   waterfall, time series, donut, barres).
3. **Sensibiliser & transformer** : 5 personas, simulateur, eco-budget,
   dashboard.
4. **Exploiter les datasets officiels** : ComparIA + RTE IRIS croisés,
   pipeline médaillon Copper/Silver/Gold reproductible.

L'app est **livrée, testée, documentée**. Le code est ouvert sous MIT.
Les données embarquées sont sous Etalab 2.0. La méthodologie est
**reproductible** par n'importe qui via le seed Monte-Carlo déterministe.

**Sobr.ia n'est pas juste un calculateur d'empreinte. C'est un outil de
gouvernance environnementale auditable de l'IA générative — et le
premier outil au monde à donner à l'utilisateur le choix
souverain de sa méthodologie scientifique d'estimation, avec
traçabilité cryptographique de ce choix.**

---

## Annexes

- [Cahier des charges v1.4](CAHIER-DES-CHARGES-v1.0.md)
- [16 ADR architecturaux](adr/) (dont [ADR-0012 multi-méthodologie](adr/ADR-0012-multi-methodology-engine.md),
  [ADR-0015 privacy Mode Équipe](adr/ADR-0015-privacy-mode-equipe.md) et
  [ADR-0016 politiques de visibilité](adr/ADR-0016-politique-visibilite-deploiement.md))
- [Méthodologie complète](methodology/)
- [Notebook de validation Quarto](../notebook/validation.qmd)
- [Catalogue des sources](sources/)
- [Sources de données embarquées](sources/CATALOGUE-DATACENTERS.md)
- [Brief chantier C24 multi-méthodologie](../briefs/chantiers/C24-multi-methodologie-ecologits.md)

---

*Sobr.ia — Empreinte environnementale auditable des LLMs · Made in France · MIT licence*
