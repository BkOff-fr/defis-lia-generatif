# Dossier candidature — Défi data.gouv.fr

## « L'impact environnemental de l'IA générative »

---

**Projet** : Sobr.ia
**Auteur** : Thibault (étudiant, contributeur Sobr.ia)
**Date** : Mai 2026
**Lien dépôt** : <https://github.com/BkOff-fr/defis-lia-generatif>
**Licence code** : MIT
**Statut** : v0.3 — 13 modules livrés et testés

---

## 1. Résumé exécutif (1 page)

**Sobr.ia** est une application native multi-plateforme (Windows / macOS /
Linux / Web) qui mesure l'empreinte environnementale de l'usage d'IA
générative (LLM) avec **rigueur scientifique**, **transparence
méthodologique** et **un angle territorial français unique**.

L'application répond aux trois axes du défi data.gouv.fr :

1. **Comprendre l'impact** : moteur Monte-Carlo N=10⁴ tirages, méthodologie
   AFNOR SPEC 2314, validation croisée Luccioni 2023 / EcoLogits 2024 à ±15%.
2. **Visualiser** : 13 modules dont 7 dataviz (carte Europe Leaflet,
   Sankey énergétique, distributions log-normales bornées, waterfall
   contributions, time series cumulatives).
3. **Sensibiliser & changer** : 5 personas (étudiant·e, pro tech,
   entreprise, collectivité, chercheur·se) avec bundles modulaires,
   eco-budget personnel, simulateur « Et si...? ».

**Trois différenciateurs majeurs** par rapport à l'état de l'art :

- **Audit ledger chaîné SHA-256** : chaque estimation est journalisée
  dans une chaîne cryptographique anti-tampering vérifiable. Aucun
  concurrent (EcoLogits, GreenAlgorithms, AI Energy Score) ne propose
  cette garantie de traçabilité. Critique pour la conformité **CSRD**
  qui exige des preuves auditables.
- **Pivot Territoire FR** : croisement entre les datasets officiels
  **ComparIA** (Beta.gouv / Ministère de la Culture) et **RTE IRIS
  sites industriels** (ODRÉ, Etalab 2.0). Aucun outil existant ne fait
  ce croisement à la maille IRIS française.
- **Datasheet Gebru 2018** : génération automatique du format académique
  standard pour reproductibilité scientifique. Permet la publication
  conjointe d'articles de recherche et de leurs traces d'estimation.

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

Le détail de calibration par modèle (8 modèles : GPT-4o, Claude 3.5
Sonnet, Mistral Large 2, Mistral Medium 3, Llama 3.1 70B, Llama 3.1 8B,
Gemini 2.0 Flash, GPT-4o-mini) est documenté dans
[`docs/methodology/MODEL-PRESETS.md`](methodology/MODEL-PRESETS.md).

### 3.3 Reproductibilité

- **Seed déterministe** : `SOBRIA_SEED=42` (configurable via env var).
  Même seed + mêmes paramètres → résultat identique à la nanoseconde.
- **RNG** : Xoshiro256++ (qualité statistique excellente, vitesse élevée).
- **Histogramme distributionnel** : 50 bins équi-width persistés dans
  chaque entrée d'audit, pour rejouer la distribution complète.

### 3.4 Validation croisée

Trois benchmarks de validation contre la littérature :

| Cas | Notre P50 | Cible | Référence |
|---|---|---|---|
| GPT-3 1024 tokens | 2.95 g | 2.8-3.4 g | Luccioni 2023 |
| LLaMA 7B inference | 0.34 g | 0.28-0.42 g | EcoLogits 2024 |
| Datacenter FR baseline | 56 g/kWh | 56 g/kWh | RTE Bilan 2023 |

Tolérance : **±15%**. Tous les cas passent en CI (`cargo test`).
Cf. [`docs/methodology/VALIDATION-CROISEE.md`](methodology/VALIDATION-CROISEE.md).

---

## 4. Différenciateurs et innovations

### 4.1 Audit ledger SHA-256 chaîné

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

### 4.2 Croisement ComparIA × RTE IRIS

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

### 4.3 Datasheet Gebru 2018

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

### 4.4 Rapport CSRD/AGEC

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

- **9 crates Rust** : ~15 000 lignes de code, ~250 tests unitaires.
- **Frontend SvelteKit 2 + Svelte 5 runes + TypeScript strict** :
  13 routes, design system custom (ink/lime/ivory + Instrument Serif +
  Geist + JetBrains Mono).
- **clippy `-D warnings`** propre sur tout le workspace.
- **`cargo deny`** + **`cargo audit`** dans la CI : 0 vulnérabilité.

### 5.2 Tests

- **250+ tests unitaires** Rust.
- **Tests property-based** (proptest) sur les invariants Monte-Carlo
  (P5 ≤ P50 ≤ P95, valeurs finies).
- **Tests d'intégration** : 1 test E2E qui exécute le pipeline complet
  Copper → Silver → Gold avec ComparIA + RTE IRIS.
- **Tests Playwright** : 13 tests "no-mock contract" (un par module
  frontend) qui valident que sans Tauri runtime, l'erreur typée
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

## 7. Roadmap post-candidature (v1.1+)

| Module | Description | Effort estimé |
|---|---|---|
| **M11 Extension navigateur** | Chrome + Firefox MV3, capture vie réelle | 3-4 semaines |
| **M16 Forecaster UI** | Backend prêt — manque UI Chart.js sliders live | 1 semaine |
| **M18 Batch CSV UI** | Backend prêt — manque drag-and-drop frontend | 3-5 jours |
| **M21 Alertes système** | Notifications OS (winrt/macOS/Linux) | 1 semaine |
| **M19 Équipe** | Auth backend + partage multi-utilisateurs | 4-6 semaines |
| **M2 Workbench** | Éditeur multi-prompts interactif | 2-3 semaines |
| **M24 Apprendre** | 10 mini-cours markdown sur prompting frugal | 2 semaines |
| **M23 Marchés publics** | Cahier des charges type + critères AO | partenariat institutionnel |
| **M5 / M6 / M10** | Modules redondants — différé sine die | — |

---

## 8. Comparaison avec l'état de l'art

| Outil | Domaine | Audit chaîné | Territoire FR | Datasheet Gebru | Privacy local | Méthodo AFNOR |
|---|---|:--:|:--:|:--:|:--:|:--:|
| **Sobr.ia** | Estimation + audit + reporting | ✅ | ✅ | ✅ | ✅ | ✅ |
| EcoLogits | Estimation Python lib | ❌ | ❌ | ❌ | ⚠️ (lib) | ⚠️ partiel |
| AI Energy Score | Benchmark plateforme | ❌ | ❌ | ❌ | ❌ (web) | ⚠️ partiel |
| GreenAlgorithms | Calculator simple | ❌ | ❌ | ❌ | ❌ (web) | ❌ |
| BoaVizta IA | Estimation hardware | ❌ | ❌ | ❌ | ⚠️ (lib) | ⚠️ partiel |
| CarbonAware SDK | Optimisation ordonnancement | ❌ | ❌ | ❌ | ⚠️ | ❌ |

Sobr.ia est, à notre connaissance, **le seul outil simultanément
auditable cryptographiquement, ciblé territoire FR, conforme datasheet
Gebru, et local-first**.

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
3. Va sur `/m9` (Référentiel modèles) → consulte les fiches détaillées
   des 8 modèles avec triplets P5/P50/P95 et sources documentaires.
4. Va sur `/comparer` (Comparer) → benchmark 4 modèles sur un prompt
   identique, voit le classement par CO₂eq / énergie / eau.
5. Crée un projet sur `/m17` (Empreinte projet) : « Étude Q1 2026
   Claude Sonnet », période Jan-Mars.
6. Génère le datasheet Gebru JSON-LD → joint au papier de recherche.
   Tout reviewer peut vérifier la reproductibilité.

---

## 10. Conclusion

Sobr.ia répond aux 4 axes du défi data.gouv.fr :

1. **Comprendre** : méthodologie AFNOR SPEC 2314 + Monte-Carlo, validation
   croisée à ±15%, transparence des paramètres.
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
gouvernance environnementale auditable de l'IA générative.**

---

## Annexes

- [Cahier des charges v1.4](CAHIER-DES-CHARGES-v1.0.md)
- [11 ADR architecturaux](adr/)
- [Méthodologie complète](methodology/)
- [Catalogue des sources](sources/)
- [Sources de données embarquées](sources/CATALOGUE-DATACENTERS.md)

---

*Sobr.ia — Empreinte environnementale auditable des LLMs · Made in France · MIT licence*
