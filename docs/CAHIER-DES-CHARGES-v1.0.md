# Cahier des charges — Sobr.ia

> **Version** : 1.4 (figée)
> **Date** : 13 mai 2026
> **Auteur** : Thibault (étudiant, candidat au défi data.gouv.fr)
> **Défi** : « L'impact environnemental de l'IA générative » — defis.data.gouv.fr
> **Statut** : Référence projet. Toute modification = bump version + ADR associé.
>
> **Changelog v1.4** : réduction du périmètre v1.0 à 13 modules essentiels (ADR-0011).
> - **13 modules en cible v1.0** : M1, M3, M7, M8, M9, M12, M13, M14, M15, M17, M20, M22, M25.
> - **11 modules différés v1.1+** : M2, M5, M6, M10, M11, M16, M18, M19, M21, M23, M24.
> - **Bundles persona recomposés** sur les 13 modules retenus (M1 reste dans tous).
> - **Justification** : focus sur la rigueur méthodologique et l'angle territorial FR, suppression de modules redondants (M2 ⊂ M3+M18, M5 ⊂ M22+M17+M18, M6 ⊂ M12, M10 ⊂ M18) et hors-scope app (M11/M19/M21/M23).
> - Le code backend des modules différés **reste compilé et testé** dans la crate, activable en v1.1 sans refonte.
>
> **Changelog v1.3** : élargissement public + gating modulaire (ADR-0010).
> - **5 personas v2** : Étudiant·e, Pro tech, Entreprise, Collectivité, Chercheur·se — chacun avec un bundle de modules par défaut.
> - **+13 nouveaux modules** (M13 à M25) couvrant simulation, dataviz cartographique, forecast, reporting CSRD, pédagogie, équipes, alertes, marchés publics.
> - **Onboarding** au premier lancement avec persona picker + bundle customizable, persistance dans SQLite référentiel.
> - **Gating frontend** : un même binaire sert tous les publics, l'expérience est composée par préférences utilisateur (cf. ADR-0010).
> - **25 modules au total** (M1-M25). Les 12 modules v1.2 sont conservés.
>
> **Changelog v1.2** : pivot sur les datasets officiels du défi data.gouv.fr.
> - **ComparIA** (Beta.gouv / Ministère de la Culture) devient le dataset central : 5 GB de conversations + votes + réactions sur LLMs, méthodologie EcoLogits intégrée.
> - **Consommation IRIS sites industriels** (RTE/NaTran/Teréga via ODRÉ) ajouté pour la dimension territoriale française.
> - Nouveau module **M12 — Territoire français** (cartographie IRIS, scénarios régionaux).
> - Suppression d'Electricity Maps et MaxMind GeoLite2 (paywalls / licences virales).
> - 0 clé API bloquante pour v1.0.
>
> **Changelog v1.1** : ajout de l'architecture médaillon (ADR-0009) — pipeline Copper/Silver/Gold automatique pour toutes les sources.

---

## 0. Identité du projet

| Élément | Valeur |
|---------|--------|
| **Nom** | Sobr.ia |
| **Étymologie** | *Sobriété* + *IA* — l'outil incarne son sujet |
| **Tagline (FR)** | « Mesurez la sobriété de votre IA générative » |
| **Tagline (EN)** | « Make generative AI's footprint visible » |
| **Modèle** | Associatif (association loi 1901 à terme) + open-source |
| **Licences** | MIT (code), Etalab 2.0 (données), CC-BY 4.0 (docs) |
| **Domaine** | sobr.ia (à réserver) / sobria.fr (fallback) |

---

## 1. Contexte et enjeux

L'IA générative connaît une croissance exponentielle depuis 2022, avec une explosion concomitante de la demande en énergie, en eau de refroidissement et en hardware spécialisé (GPU, ASIC). Les ordres de grandeur publics restent flous, fragmentés, parfois contradictoires :

- L'ADEME a publié plusieurs études sur l'empreinte du numérique (2,5 % des émissions françaises en 2020) sans intégrer pleinement la rupture GenAI.
- Le **référentiel général pour l'IA frugale (AFNOR SPEC 2314)**, piloté par l'Ecolab du CGDD, fournit la méthodologie cadre mais reste peu appliqué publiquement.
- **GenAI Impact (Data for Good)** maintient `EcoLogits`, une bibliothèque Python pour estimer les requêtes LLM, mais l'outil reste cantonné aux développeurs.
- **Hugging Face / Salesforce** publient l'AI Energy Score, **CodeCarbon** trace les entraînements, **ML.Energy** benchmarke les inférences — chacun avec ses biais et hypothèses.
- La **feuille de route Numérique & IA** (ministères Transition écologique, septembre 2025) appelle à une transparence accrue et à des indicateurs comparables.

Sobr.ia comble le vide entre cadres méthodologiques et outils accessibles : **une application native, frugale, méthodologiquement rigoureuse, et qui ouvre ses données.**

---

## 2. Objectifs

### 2.1 Objectif général

Produire **une stack complète et open-source** (dataset + application + extension + méthodologie) permettant à un public non-expert d'évaluer, comparer et communiquer l'impact environnemental d'usages réels d'IA générative à base de LLMs, conformément au référentiel AFNOR SPEC 2314 et aux facteurs d'émission ADEME.

### 2.2 Objectifs spécifiques

1. **Consolider** un dataset ouvert publiable sur data.gouv.fr (modèles LLM, consommations mesurées, mix électriques, facteurs ADEME, eau, hardware).
2. **Outiller** les décideurs et le grand public avec une application desktop/web/mobile (Tauri 2) chiffrant l'impact d'un usage ou d'un scénario.
3. **Capturer** l'usage réel via une extension navigateur (Chrome/Firefox MV3) qui mesure en vie réelle.
4. **Documenter** la méthodologie avec un notebook reproductible et un rapport scientifique défendable.
5. **Démontrer** par l'exemple la cohérence d'une stack frugale (Rust + Tauri + SvelteKit) face aux outils SaaS classiques.

### 2.3 Non-objectifs (hors périmètre v1.0)

- Génération d'images, vidéo, audio (à traiter en v2.0).
- Mesure physique en temps réel sur des GPU (instrumentation hardware, RAPL).
- Optimisation des modèles (compression, quantization).
- Recommandations juridiques (RGPD, AI Act) hors angle environnemental.

---

## 3. Personas et cas d'usage

> Voir [ADR-0010](adr/ADR-0010-personas-and-module-gating.md) pour la décision
> architecturale qui ancre cette segmentation dans l'app (onboarding + module
> gating). Les 5 personas P1-P5 v1.2 ont été refondus en 5 personas v2 :
> Étudiant·e, Pro tech, Entreprise, Collectivité, Chercheur·se. Chaque persona
> est associé à un **bundle de modules par défaut** que l'utilisateur peut
> ensuite ajuster librement.

### 3.1 P1 — Étudiant·e / Curieux·se (`student`)

> *Léa, 21 ans, en M1 sciences sociales* : « J'ai vu un article sur l'empreinte
> de ChatGPT, je veux comprendre l'impact de mes propres prompts et savoir
> comment prompter plus efficacement. »

**Bundle par défaut** : M1 Estimer · M8 Méthodologie · M11 Extension navigateur ·
M13 Simulateur « Et si…? » · M14 À propos · M15 Dashboard personnel ·
M24 Apprendre · M25 Objectifs & habitudes.

**Parcours type** : Onboarding → premier prompt sur M1 → installe l'extension
M11 → vérifie son dashboard hebdomadaire M15 → ajuste un levier dans
M13 pour comprendre l'impact d'un changement de modèle → suit un mini-cours
M24 sur le prompting frugal.

### 3.2 P2 — Professionnel·le tech (`pro_tech`)

> *Thomas, 29 ans, dev backend SaaS* : « Je dois choisir entre intégrer Mistral
> Large ou GPT-4o-mini pour notre feature de résumé. Je veux comparer les
> coûts carbone, faire tourner mes prompts réels et exporter pour mon
> tech lead. »

**Bundle par défaut** : M1 Estimer · M2 Workbench · M3 Comparer modèles ·
M5 Rapports & exports · M7 Journal d'audit · M9 Géolocalisation datacenter ·
M11 Extension navigateur · M18 Batch CSV → rapport · M21 Alertes & seuils.

**Parcours type** : Onboarding → workbench M2 sur 200 prompts internes →
comparateur M3 sur 4 modèles candidats → export PDF M5 pour revue tech →
seuil d'alerte M21 si usage hebdo dépasse X g CO₂eq.

### 3.3 P3 — Entreprise (`enterprise`)

> *Claire, 35 ans, directrice RSE d'un groupe industriel 8 000 personnes* :
> « Notre comex me demande comment intégrer l'IA générative dans notre rapport
> CSRD. J'ai besoin de chiffres défendables, d'un export PDF signé, et d'un
> forecast à 12 mois sur l'adoption interne. »

**Bundle par défaut** : M1 Estimer · M2 Workbench · M5 Rapports & exports ·
M6 Géolocalisation datacenter · M7 Journal d'audit · M10 Import logs ·
M12 Datacenters Europe · M16 Forecaster 12 mois · M19 Équipe ·
M21 Alertes & seuils · M22 Rapport CSRD/AGEC.

**Parcours type** : Onboarding → import CSV M10 du SI → forecast M16 sur
l'adoption interne → carte M12 pour repérer les datacenters utilisés →
rapport CSRD signé M22 → suivi annuel via M19 multi-utilisateurs.

### 3.4 P4 — Collectivité / Service public (`public_sector`)

> *Marc, 42 ans, agent à la direction du numérique d'une métropole* :
> « Je dois orienter notre achat IA vers du frugal, suivre l'empreinte sur
> notre territoire et préparer un marché public exemplaire. »

**Bundle par défaut** : M1 Estimer · M5 Rapports & exports · M6 Géolocalisation
datacenter · M7 Journal d'audit · M8 Méthodologie · M12 Datacenters Europe ·
M16 Forecaster 12 mois · M20 Territoire FR · M22 Rapport CSRD/AGEC ·
M23 Marchés publics IA frugale.

**Parcours type** : Onboarding → M20 Territoire FR pour cartographier
l'empreinte régionale → M12 Datacenters Europe pour sourcer hors-territoire →
M16 forecast 12 mois sur la consommation projetée → M23 export d'un
cahier des charges type pour appel d'offres IA frugale.

### 3.5 P5 — Chercheur·se / Journaliste (`researcher`)

> *Sami, 33 ans, doctorant en sciences de l'information* : « Je publie un
> papier sur l'empreinte des LLMs francophones. Il me faut reproduire
> Luccioni 2023, comparer 8 modèles sur mes propres prompts, et publier
> mes datasets bruts pour reproductibilité. »

**Bundle par défaut** : M1 Estimer · M3 Comparer modèles · M5 Rapports
& exports · M8 Méthodologie · M14 À propos · M17 Empreinte projet ·
M18 Batch CSV → rapport.

**Parcours type** : Onboarding → M18 batch sur 1 000 prompts annotés →
M3 comparaison de 8 modèles → M17 page projet publique avec datasheet
Gebru et hash référentiel utilisé → M5 export Parquet + JSON-LD pour
référencement dans le papier.

---

## 4. Périmètre fonctionnel — 13 modules v1.0 + 11 différés v1.1+

> Nomenclature : `Mxx` = identifiant stable du module. Les IDs sont gelés
> à compter de la v1.3 (cf. ADR-0010). La v1.4 (cf. ADR-0011) **réduit le
> périmètre v1.0 à 13 modules essentiels** et **différe 11 modules en
> v1.1+**. Le backend Rust des modules différés reste compilé et
> testé — leur activation v1.1 ne nécessite qu'un ajout dans le rail
> frontend + un toggle dans Paramètres.

### 4.1 Tableau des 24 modules (v1.0 + v1.1+)

| ID | Module | Description | Cible |
|----|--------|-------------|-------|
| **M1** | **Estimer un prompt** | Moteur Monte-Carlo + UI d'entrée unitaire | 🏆 v1.0 |
| **M2** | Workbench | Exploration interactive multi-prompts | 🪦 v1.1+ (chevauche M3+M18) |
| **M3** | **Comparer modèles** | Matrice N modèles × indicateurs, benchmark frugalité | 🏆 v1.0 |
| **M4** | *(réservé — retiré v1.3)* | — | retiré |
| **M5** | Rapports & exports génériques | PDF, CSV/Parquet, JSON-LD, Quarto | 🪦 v1.1+ (chevauche M22+M17+M18) |
| **M6** | Géolocalisation datacenter unitaire | Détection IP/zone → datacenter probable | 🪦 v1.1+ (chevauche M12) |
| **M7** | **Journal d'audit** | Ledger ACID immuable des estimations (chaîne SHA-256) | 🏆 v1.0 |
| **M8** | **Méthodologie interactive** | Documentation embarquée, glossaire, références | 🏆 v1.0 |
| **M9** | **Référentiel modèles** | Catalogue + fiches modèles avec calibration & sources | 🏆 v1.0 |
| **M10** | Import logs | Import CSV/JSONL/Parquet d'historiques | 🪦 v1.1+ (chevauche M18) |
| **M11** | Extension navigateur | Chrome/Firefox MV3, capture vie réelle | 🪦 v1.1+ (chantier MV3 séparé) |
| **M12** | **Datacenters Europe** | Carte Leaflet + 28 datacenters, drill-down donut + barres + 24h | 🏆 v1.0 |
| **M13** | **Simulateur « Et si...? »** | 7 leviers temps réel + verdict CO₂, waterfall, projection 12 mois | 🏆 v1.0 |
| **M14** | **À propos / Crédits** | Mentions, licences, contributeurs, sources de financement | 🏆 v1.0 |
| **M15** | **Dashboard personnel** | Vue jour/semaine/mois de ses propres prompts | 🏆 v1.0 |
| **M16** | Forecaster 12 mois | Projection budget carbone IA avec bande d'incertitude | 🪦 v1.1+ (backend prêt, UI v1.1) |
| **M17** | **Empreinte projet** | Page projet publique avec datasheet Gebru | 🏆 v1.0 |
| **M18** | Batch CSV → rapport | Upload N prompts, rapport agrégé exportable | 🪦 v1.1+ (backend prêt, UI v1.1) |
| **M19** | Équipe / multi-utilisateurs | Rollup, rôles (manager/IC), exports par cost-center | 🪦 v1.1+ (auth back-end) |
| **M20** | **Territoire FR (IRIS)** | Cartographie IRIS + Sankey énergétique, croisement ComparIA × RTE IRIS | 🏆 v1.0 (différenciateur) |
| **M21** | Alertes & seuils | Notifications locales si dépassement de seuils | 🪦 v1.1+ (notifications OS) |
| **M22** | **Rapport CSRD/AGEC** | Export PDF signé + JSON-LD PROV-O, conforme SPEC 2314 | 🏆 v1.0 |
| **M23** | Marchés publics IA frugale | Cahier des charges type + critères d'évaluation pour AO | 🪦 v1.1+ (partenariat institutionnel) |
| **M24** | Apprendre | Mini-cours markdown sur prompting frugal | 🪦 v1.1+ (volume sans différenciateur) |
| **M25** | **Objectifs & habitudes** | Eco-budget personnel, suivi mensuel | 🏆 v1.0 |

> **🏆 v1.0 (13 modules)** : M1, M3, M7, M8, M9, M12, M13, M14, M15, M17, M20, M22, M25.
> **🪦 v1.1+ (11 modules)** : M2, M5, M6, M10, M11, M16, M18, M19, M21, M23, M24.

> *Note historique* : `M4` était le « Simulateur de scénarios » en v1.2 ; il
> est remplacé par `M13 Simulateur « Et si…? »` dans la v1.3 (refonte UX +
> dataviz). L'ID `M4` est laissé réservé pour ne pas casser les références
> antérieures.

### 4.2 Grille persona × module (par défaut) — v1.4

Recomposée sur les **13 modules v1.0** (cf. ADR-0011). Les modules différés
v1.1+ ne sont dans aucun bundle par défaut, mais leur backend est testé
et activable manuellement dans Paramètres si le frontend devient
disponible.

| Module v1.0 | Étudiant | Pro tech | Entreprise | Collectivité | Chercheur |
|-------------|:-:|:-:|:-:|:-:|:-:|
| M1 Estimer | **★** | **★** | **★** | **★** | **★** |
| M3 Comparer modèles | | **★** | | | **★** |
| M7 Journal d'audit | | **★** | **★** | | **★** |
| M8 Méthodologie | **★** | **★** | | **★** | **★** |
| M9 Référentiel modèles | | **★** | | | **★** |
| M12 Datacenters Europe | | | **★** | **★** | |
| M13 Simulateur « Et si...? » | **★** | **★** | | | |
| M14 À propos | **★** | **★** | **★** | **★** | **★** |
| M15 Dashboard personnel | **★** | | **★** | | |
| M17 Empreinte projet | | | **★** | **★** | **★** |
| M20 Territoire FR | | | **★** | **★** | |
| M22 Rapport CSRD/AGEC | | | **★** | **★** | |
| M25 Eco-budget | **★** | | **★** | | |

**Tailles de bundle** : Étudiant 6 · Pro tech 7 · Entreprise 9 · Collectivité 7 · Chercheur 7.

(Tout module non coché reste **activable à la carte** par l'utilisateur après
l'onboarding via Paramètres.)

### 4.3 Composants dataviz transverses

Trois composants techniques réutilisés à travers plusieurs modules :

- **Bande d'incertitude** (P5-P50-P95 + sliders live) → utilisée dans M13, M16, M22.
- **Sankey énergétique** (flux par datacenter ou par modèle) → utilisée dans M12, M20, M3.
- **Histogramme distributionnel** (50 bins Monte-Carlo) → utilisé dans M1, M3, M13.

Ces composants Svelte vivent dans `web/src/lib/components/dataviz/` et sont
versionnés avec les écrans qui les consomment.

### 4.4 Indicateurs calculés

Pour chaque estimation, l'outil produit (avec intervalles d'incertitude propagés par Monte-Carlo) :

- **CO₂eq** (gCO₂eq) — émissions opérationnelles + embarquées (amorties)
- **Énergie** (Wh) — décomposée en compute, idle, networking, cooling (PUE)
- **Eau** (L) — WUE direct (refroidissement) + indirect (production électrique)
- **Métaux critiques** (mg équivalent terre rare) — proxy à partir de l'embodied hardware
- **Coût** (€) — coût utilisateur facturé, pour mise en perspective économique
- **Équivalents parlants** — km voiture, douches, écrans-heures, etc.

---

## 5. Exigences fonctionnelles détaillées

### 5.1 Module M1 — Référentiel

**EF-M1-01** : Ingestion ADEME Base Empreinte (facteurs d'émission électricité, hardware)
**EF-M1-02** : Ingestion RTE eco2mix (mix électrique français temps réel + historique)
**EF-M1-03** : Ingestion Electricity Maps (mix mondial)
**EF-M1-04** : Ingestion GenAI Impact / EcoLogits (caractéristiques modèles)
**EF-M1-05** : Ingestion Hugging Face AI Energy Score
**EF-M1-06** : Ingestion CodeCarbon (mesures d'entraînement)
**EF-M1-07** : Ingestion ML.Energy Leaderboard
**EF-M1-08** : Ingestion datasheets GPU (TDP, embodied carbon)
**EF-M1-09** : Versionnage du référentiel (CalVer YYYY.MM.DD, DVC)
**EF-M1-10** : Validation de schéma à l'ingestion (JSON Schema + Rust types)

### 5.2 Module M2 — Estimateur

**EF-M2-01** : Calcul CO₂eq pour un prompt (T_in, T_out, modèle, datacenter) < 50 ms
**EF-M2-02** : Propagation d'incertitude Monte-Carlo (N=10⁴) < 200 ms
**EF-M2-03** : Application de la formule AFNOR SPEC 2314
**EF-M2-04** : Affichage des hypothèses utilisées (sources clic-cliquables)
**EF-M2-05** : Journalisation SQLite signée SHA-256 (audit ledger M7)
**EF-M2-06** : Mode "batch" pour estimation par lot (entrée CSV)
**EF-M2-07** : Validation contre 3 études de référence (Luccioni, Patterson, EcoLogits)

### 5.3 Module M3 — Workbench

**EF-M3-01** : Exploration du référentiel avec filtres (provider, taille, modalité, licence)
**EF-M3-02** : Recherche full-text (FTS5 SQLite)
**EF-M3-03** : Tri multi-colonnes
**EF-M3-04** : Détail d'un modèle (fiche complète avec sources et hypothèses)
**EF-M3-05** : Export sélection (CSV, Parquet, JSON)

### 5.4 Module M4 — Simulateur de scénarios

**EF-M4-01** : Création d'un scénario (population, taux d'adoption, fréquence, modèle, période)
**EF-M4-02** : Projection temporelle (mois ou années jusqu'à 2030)
**EF-M4-03** : Visualisation aire/courbe avec bande d'incertitude
**EF-M4-04** : Comparaison de plusieurs scénarios côte à côte
**EF-M4-05** : Sauvegarde / chargement de scénarios (JSON)
**EF-M4-06** : Mode "what-if" : variation paramétrique sur un slider

### 5.5 Module M5 — Comparateur

**EF-M5-01** : Sélection de 2 à 8 modèles
**EF-M5-02** : Matrice indicateurs triable et normalisable
**EF-M5-03** : Score composite paramétrable (poids ajustables)
**EF-M5-04** : Indication des données manquantes ou estimées par proxy
**EF-M5-05** : Export matrice (CSV, PDF)

### 5.6 Module M6 — Rapports & exports

**EF-M6-01** : Génération rapport PDF synthétique (4-8 p., template Quarto)
**EF-M6-02** : Génération notebook Quarto reproductible
**EF-M6-03** : Export dataset (CSV, Parquet) avec datasheet (Gebru et al.)
**EF-M6-04** : Export JSON-LD compatible audit réglementaire (CSRD)
**EF-M6-05** : Export Observable Notebook embeddable

### 5.7 Module M7 — Audit ledger

**EF-M7-01** : Chaque estimation est journalisée (timestamp, paramètres, résultat, hash)
**EF-M7-02** : Mode WAL SQLite + signatures SHA-256 chaînées
**EF-M7-03** : Export ledger complet (NDJSON signé)
**EF-M7-04** : Vérification d'intégrité du ledger
**EF-M7-05** : Purge configurable (RGPD)

### 5.8 Module M8 — Aide & méthodologie

**EF-M8-01** : Doc embarquée (mdBook compilé en HTML statique)
**EF-M8-02** : Glossaire (CO₂eq, PUE, WUE, embodied, etc.)
**EF-M8-03** : Liens vers AFNOR SPEC 2314, ISO/IEC 21031, ITU-T L.1410
**EF-M8-04** : Tour guidé pour nouveaux utilisateurs (onboarding)

### 5.9 Module M9 — Géolocalisation datacenter

**EF-M9-01** : Détection zone via IP (offline, base GeoLite2 embarquée)
**EF-M9-02** : Mapping provider → datacenter probable (heuristique documentée)
**EF-M9-03** : Récupération du mix électrique local (RTE pour FR, Electricity Maps sinon)
**EF-M9-04** : Choix manuel possible (override utilisateur)
**EF-M9-05** : Indicateur de confiance sur la détection

### 5.10 Module M10 — Import logs entreprise

**EF-M10-01** : Import CSV avec mappeur de colonnes interactif
**EF-M10-02** : Import JSONL (formats OpenAI, Anthropic, Mistral)
**EF-M10-03** : Anonymisation locale (pas d'envoi externe)
**EF-M10-04** : Agrégation par utilisateur / équipe / projet
**EF-M10-05** : Génération rapport RSE prêt à intégrer dans bilan CSRD

### 5.11 Module M11 — Extension navigateur

**EF-M11-01** : Manifest V3 Chrome / Firefox (WebExtension cross-browser)
**EF-M11-02** : Détection automatique des interfaces (ChatGPT, Claude.ai, Mistral, Gemini, Le Chat)
**EF-M11-03** : Comptage local des tokens (tiktoken-wasm ou Tokenizers-rs via WASM)
**EF-M11-04** : Badge visuel à côté du prompt (CO₂eq cumulé du jour)
**EF-M11-05** : Communication avec l'app desktop via localhost (HTTPS + token) ou stockage local
**EF-M11-06** : Bilan hebdomadaire automatique (notification)
**EF-M11-07** : Pas de tracking : tout reste local sauf opt-in explicite

### 5.12 Module M12 — Territoire français (NOUVEAU v1.2)

**Données primaires** : ComparIA (5 GB Parquet) + RTE/NaTran/Teréga IRIS (90 MB CSV + GeoJSON).

**EF-M12-01** : Carte choroplèthe IRIS de la consommation industrielle électrique (90 MB CSV ingéré, ~50 000 IRIS)
**EF-M12-02** : Détection des IRIS candidats à héberger un datacenter (heuristique : élec ≫ gaz, > seuil MWh)
**EF-M12-03** : Croisement ComparIA × IRIS : volume de requêtes LLMs estimé par bassin de population
**EF-M12-04** : Scénarios régionaux (Île-de-France, AURA, Occitanie…) avec projection à 5 ans
**EF-M12-05** : Comparaison régions FR vs benchmarks internationaux (Virginie, Oregon, Islande…)
**EF-M12-06** : Export carte PNG/SVG haute résolution + GeoJSON enrichi
**EF-M12-07** : Storytelling intégré : « top 10 IRIS qui pourraient absorber un déploiement national d'IA »

---

## 6. Exigences non-fonctionnelles

| ID | Catégorie | Exigence | Cible |
|----|-----------|----------|-------|
| NF-01 | Performance | Temps de lancement à froid | < 800 ms |
| NF-02 | Performance | Empreinte RAM moyenne | < 100 Mo |
| NF-03 | Performance | Taille binaire desktop | < 20 Mo |
| NF-04 | Performance | Taille extension navigateur | < 500 Ko |
| NF-05 | Frugalité | Empreinte CO₂eq par session de 30 min | mesurée et publiée |
| NF-06 | Robustesse | Couverture de tests Rust | ≥ 80 % |
| NF-07 | Robustesse | Audit ledger ACID intègre | 100 % (SQLite WAL) |
| NF-08 | Sécurité | Pas de télémétrie sans opt-in | obligatoire |
| NF-09 | Sécurité | Communications HTTPS uniquement, sources signées | obligatoire |
| NF-10 | Sécurité | Extension : pas d'accès aux pages hors LLM connus | obligatoire |
| NF-11 | Privacy | Tout traitement local par défaut | obligatoire |
| NF-12 | Accessibilité | Conformité RGAA 4.1 niveau AA | obligatoire |
| NF-13 | i18n | Langues | FR + EN |
| NF-14 | Reproductibilité | Builds déterministes (Nix/Earthly) | obligatoire |
| NF-15 | Open source | Licences | MIT + Etalab 2.0 + CC-BY |
| NF-16 | Documentation | Doc utilisateur + dev complète FR/EN | obligatoire |
| NF-17 | Multi-plateforme | Cibles | Win / macOS / Linux / Web (Wasm) / Android / iOS |

---

## 7. Architecture technique

### 7.1 Vue d'ensemble

```
┌────────────────────────────────────────────────────────────┐
│  Tauri 2.x — wrapper natif (desktop/mobile/web)            │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Frontend : SvelteKit + TypeScript                   │  │
│  │  ├─ stores réactifs (estimations, scénarios)         │  │
│  │  ├─ dataviz : Observable Plot + D3 (Sankey, etc.)    │  │
│  │  ├─ UI : skeleton CSS custom (frugalité visuelle)    │  │
│  │  └─ a11y : RGAA AA + lecteurs d'écran                │  │
│  └────────────────┬─────────────────────────────────────┘  │
│                   │ IPC Tauri (commandes typées)            │
│  ┌────────────────▼─────────────────────────────────────┐  │
│  │  Cœur Rust (workspace cargo, 9 crates)               │  │
│  │  ├─ sobria-core         : types, traits, errors      │  │
│  │  ├─ sobria-referentiel  : accès SQLite + cache       │  │
│  │  ├─ sobria-estimator    : moteur AFNOR SPEC 2314     │  │
│  │  ├─ sobria-ingest       : pipeline médaillon         │  │
│  │  │                        (trait DataLayer, ADR-0009) │  │
│  │  ├─ sobria-geoloc       : détection datacenter (M9)  │  │
│  │  ├─ sobria-import       : parsers logs entreprise(M10)│ │
│  │  ├─ sobria-export       : PDF, Quarto, JSON-LD       │  │
│  │  ├─ sobria-audit        : ledger immuable signé      │  │
│  │  └─ sobria-app          : commandes Tauri            │  │
│  └─────────┬────────────────────────────────────────────┘  │
│            │                                                │
│   ┌────────┴────────────┐                                  │
│   ▼                     ▼                                  │
│ SQLite (rusqlite)   DuckDB (duckdb-rs)                     │
│ • référentiel       • requêtes analytiques                 │
│ • audit ledger      • agrégations scénarios                │
│ • WAL + signatures  • lecture parquet                      │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│  Extension navigateur Sobr.ia (M11)                         │
│  Manifest V3 — Chrome + Firefox                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Content scripts (TypeScript)                      │    │
│  │  • ChatGPT / Claude / Mistral / Gemini detectors   │    │
│  │  • Token counting (tiktoken-wasm)                  │    │
│  │  • Badge UI overlay                                │    │
│  └─────────────┬──────────────────────────────────────┘    │
│                │                                            │
│  ┌─────────────▼──────────────────────────────────────┐    │
│  │  Background service worker                         │    │
│  │  • Aggregation locale (IndexedDB)                  │    │
│  │  • Bridge vers Sobr.ia desktop (localhost:port)    │    │
│  │  • Notifications hebdo                             │    │
│  └────────────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────────┘
                          │
                          ▼ (alimentés par le pipeline médaillon — ADR-0009)
┌────────────────────────────────────────────────────────────┐
│ Pipeline médaillon (CI nocturne via GitHub Actions + DVC)  │
│                                                             │
│   Sources externes (ADEME, RTE, HF, EcoLogits, papers…)    │
│                       │                                     │
│           ingest_copper()                                   │
│                       ▼                                     │
│   🟫 Copper  data/copper/<source>/<YYYY-MM-DD>/...         │
│   (raw, immutable, hashé, daté, manifest.json)             │
│                       │                                     │
│           promote_silver()                                  │
│                       ▼                                     │
│   🥈 Silver  data/silver/<source>/*.parquet                │
│   (typé, validé, normalisé SI, lineage vers Copper)        │
│                       │                                     │
│           contribute_gold()                                 │
│                       ▼                                     │
│   🥇 Gold    referentiel.sqlite  +  analytics.parquet      │
│              datasheet.jsonld    +  MANIFEST.sha256 (GPG)  │
│                                                             │
│ Orchestration : dvc.yaml — `dvc repro` reproduit à l'iden- │
│ tique. Publication release GitHub + dataset data.gouv.fr.  │
└────────────────────────────────────────────────────────────┘
```

### 7.2 Choix techniques justifiés

Voir `docs/adr/` pour chaque ADR détaillé.

| Choix | ADR | Justification courte |
|-------|-----|---------------------|
| Rust + Tauri 2 | ADR-0001 | Performance, frugalité, multi-cibles |
| SvelteKit + TS | ADR-0002 | Légèreté runtime, dataviz fluide |
| SQLite (WAL) + DuckDB | ADR-0003 | ACID + OLAP embarqués |
| Monte-Carlo N=10⁴ | ADR-0004 | Propagation d'incertitude AFNOR SPEC 2314 |
| Extension WebExtension MV3 | ADR-0005 | Standard cross-browser long terme |
| Licences MIT + Etalab 2.0 + CC-BY | ADR-0006 | Cohérence écosystème français |
| DVC pour le référentiel | ADR-0007 | Versionnage de données massives |
| Observable Plot + D3 | ADR-0008 | Grammar of graphics + customisation |
| Architecture médaillon (Copper/Silver/Gold) | ADR-0009 | Pipeline data discipliné et reproductible |

### 7.3 Architecture médaillon (résumé — détail dans ADR-0009)

Toutes les sources externes traversent un **pipeline à 3 couches** implémenté automatiquement via un trait Rust unique :

```
Sources externes (ADEME, RTE, HF, EcoLogits, papers…)
       │
       ▼  ingest_copper()        — récupération brute, immutable, datée, hashée
┌──────────────────┐
│  🟫 COPPER       │  data/copper/<source>/<YYYY-MM-DD>/...
│  (raw, immutable)│  format d'origine + manifest.json (URL, hash, licence)
└──────┬───────────┘
       │
       ▼  promote_silver()       — schéma figé, validation, normalisation SI
┌──────────────────┐
│  🥈 SILVER       │  data/silver/<source>/*.parquet
│  (clean, typed)  │  schémas versionnés, lineage vers Copper
└──────┬───────────┘
       │
       ▼  contribute_gold()      — jointures, dédup inter-source, matérialisations
┌──────────────────┐
│  🥇 GOLD         │  data/gold/referentiel.sqlite  (lu par l'app)
│  (consumer-ready)│  data/gold/analytics.parquet   (lu par DuckDB)
└──────────────────┘   data/gold/datasheet.jsonld   (Gebru et al.)
                       data/gold/MANIFEST.sha256    (signé GPG)
```

**Garanties produites** : traçabilité scientifique de bout en bout (lineage des hashes Copper), reproductibilité totale (DVC + seeds), idempotence à l'ingestion, validation de schéma à chaque écriture Silver, lecture rapide en production. Une commande unique (`cargo run -p sobria-ingest -- pipeline run`) exécute tout le pipeline et le CI nocturne le rejoue automatiquement via DVC.

**Onboarding d'une nouvelle source** = implémenter un seul trait `DataLayer` (méthodes `ingest_copper`, `promote_silver`, `contribute_gold`). Aucune intervention manuelle ailleurs.

---

## 8. Sources de données

Voir `docs/sources/CATALOGUE-SOURCES.md` pour la fiche détaillée de chaque source. **Stratégie en 3 tiers**, alignée sur les datasets officiels du défi data.gouv.fr.

### 8.1 Tier 1 — Datasets officiels du défi 🎯

| Source | Données | Format | Volume | Licence |
|--------|---------|--------|--------|---------|
| **ComparIA** (Beta.gouv) | Conversations + votes + réactions LLMs | Parquet | 5 GB | Etalab 2.0 |
| **RTE/NaTran/Teréga IRIS** (ODRÉ) | Consommation industrielle élec + gaz par IRIS | CSV + GeoJSON | ~200 MB | Etalab 2.0 |

### 8.2 Tier 2 — Complémentaires (sans authentification)

| Source | Données | Format | Licence |
|--------|---------|--------|---------|
| ADEME Base Empreinte | Facteurs d'émission | API + CSV | Etalab 2.0 |
| GenAI Impact / EcoLogits | Modèles + méthodologie officielle | Python + JSON | MIT |
| Hugging Face AI Energy Score | Score énergétique | HF Hub | Apache 2.0 / CC-BY |
| CodeCarbon | Mesures d'entraînement | GitHub | MIT |
| ML.Energy Leaderboard | Benchmarks inférence | CSV | CC-BY |
| Papers académiques | Validations croisées | PDF | varies |

### 8.3 Tier 3 — Optionnelle (compte gratuit)

| Source | Données | Format | Licence | Statut |
|--------|---------|--------|---------|--------|
| RTE eco2mix | Mix élec FR live | API OAuth2 | Etalab 2.0 | Optionnel v1.0, fallback CSV historiques |

### 8.4 Supprimées du périmètre v1.0

- ❌ Electricity Maps (plan gratuit limité, CC-BY-SA viral)
- ❌ MaxMind GeoLite2 (compte requis, CC-BY-SA, redondant avec IRIS pour la France)

**Bilan : 0 clé bloquante pour v1.0.**

---

## 9. Méthodologie de calcul

### 9.1 Formule de référence (inférence)

```
CO₂eq(prompt) =
  [ E_compute × PUE × IF_électrique
  + E_embodied / N_amortissement ]
  + propagation d'incertitude (Monte-Carlo, N=10⁴)

avec :
  E_compute     = (T_in × ε_prefill + T_out × ε_decode) × η_modèle
  PUE           = ratio datacenter (1.1-1.6 selon zone, source datacenter)
  IF_électrique = facteur émission temps réel (RTE, Electricity Maps)
  E_embodied    = embodied carbon hardware / nb requêtes amorties
```

### 9.2 Propagation d'incertitude

Chaque paramètre est représenté par une distribution (uniforme, normale, log-normale selon nature). Le moteur fait tourner **10 000 simulations Monte-Carlo** par estimation et restitue P5, P50, P95.

### 9.3 Validation croisée

Le moteur est validé par :
- Reproduction de **3 études de référence** : Luccioni 2023 (BLOOM), Patterson 2021 (Google), EcoLogits 2024.
- Comparaison aux mesures CodeCarbon publiées (≥ 30 cas de test).
- Revue par notre mentor Ecolab/ADEME.

---

## 10. Livrables

| ID | Livrable | Format | Public |
|----|----------|--------|--------|
| L1 | Application Sobr.ia | Binaires Win/Mac/Linux + Android/iOS + Wasm | Grand public |
| L2 | Extension navigateur | .crx (Chrome) + .xpi (Firefox) | Grand public |
| L3 | Dataset consolidé | SQLite + Parquet + CSV | data.gouv.fr |
| L4 | Notebook de validation | Quarto (.qmd → HTML + PDF) | Communauté scientifique |
| L5 | Rapport méthodologique | PDF 30-40 p. FR + EN | ADEME / Ecolab / jury |
| L6 | Policy brief | PDF 4 p. FR | Décideurs publics |
| L7 | Code source | GitHub (workspace Cargo + SvelteKit + extension) | Développeurs |
| L8 | Documentation | mdBook + site statique | Tous |
| L9 | Vidéo démo | MP4 3-5 min sous-titrée FR/EN | Jury, communication |
| L10 | Datasheet (Gebru et al.) | PDF | Communauté ML |

---

## 11. Roadmap 12 semaines

Voir `docs/ROADMAP.md` pour le détail sprint par sprint avec définitions of done.

| Sem. | Phase | Livrables intermédiaires clés |
|------|-------|-------------------------------|
| S0 | Revue biblio | Synthèse 10-15 p. + bibliographie BibTeX |
| S1 | Cadrage technique | Repo init, ADR, schéma DB, CI/CD |
| S2 | Référentiel pt.1 | Ingest ADEME + RTE |
| S3 | Référentiel pt.2 | Ingest HF + EcoLogits + papers |
| S4 | Estimateur pt.1 | Moteur Rust + Monte-Carlo |
| S5 | Estimateur pt.2 + audit | Validation papers + ledger ACID |
| S6 | UI MVP pt.1 + géoloc M9 | Shell Tauri + Svelte + détection DC |
| S7 | UI MVP pt.2 + import M10 | Workbench + comparateur + import CSV |
| S8 | Simulateur M4 + extension MV3 | Scénarios + extension navigateur |
| S9 | Méthodologie + Quarto | Notebook reproductible, rapport rédigé |
| S10 | Exports M6 + multi-cibles | PDF, Parquet, mobile, Wasm, a11y |
| S11 | Tests utilisateurs | 5 entretiens, itération UX |
| S12 | Soumission | Vidéo, dépôt data.gouv.fr, communication |

---

## 12. Gouvernance, licences, open source

- **Structure cible** : association loi 1901 (à constituer post-livraison)
- **Code** : MIT
- **Données** : Etalab 2.0
- **Documentation** : CC-BY 4.0
- **Modèle de contribution** : DCO (Developer Certificate of Origin)
- **Code de conduite** : Contributor Covenant 2.1
- **Versionnage** : SemVer pour l'app, CalVer (YYYY.MM.DD) pour le référentiel
- **Sécurité** : SECURITY.md, GPG signing des releases, SBOM (CycloneDX)

---

## 13. Critères de succès (KPI)

### 13.1 KPI techniques (avant jury)
- Couverture tests Rust ≥ 80 %
- Temps de calcul moyen estimation unitaire < 50 ms
- Empreinte binaire desktop < 20 Mo
- Extension < 500 Ko
- 0 vulnérabilité critique (cargo audit / npm audit / Snyk)
- Conformité RGAA AA validée

### 13.2 KPI projet (post-jury, 6 mois)
- Dataset téléchargé ≥ 200 fois sur data.gouv.fr
- App téléchargée ≥ 500 fois cumulé toutes plateformes
- Extension installée ≥ 200 fois
- ≥ 1 mention dans la presse spécialisée
- ≥ 1 retour officiel ADEME ou Ecolab

### 13.3 KPI candidature
- Cohérence du dossier (technique, méthodologie, frugalité)
- Reproductibilité de bout en bout (`./build.sh` produit tout)
- Qualité visuelle et accessibilité de l'app
- Solidité de la méthodologie scientifique (validation croisée)

---

## 14. Risques et mitigations

| Risque | P | I | Mitigation |
|--------|---|---|-----------|
| Courbe d'apprentissage Rust + Tauri 2 + SvelteKit | H | H | S0 inclut tutoriels, prototype S6 sans optimisation |
| Sources de données indisponibles ou changeantes | M | H | DVC + mocks + fallback CSV statiques |
| Méthodologie contestable scientifiquement | M | H | Validation croisée + relecture mentor |
| Scope creep | H | M | CDC strict, backlog v2.0 |
| Tauri mobile encore jeune (Tauri 2 OK mais peu de retours) | M | M | Prioriser desktop, mobile en bonus S10 |
| Solo : surcharge / abandon | M | H | Sprints hebdo, journal public, mentor |
| Validation jury : critères non publics | M | M | Lire archives, contacter Ecolab via mentor |
| Extension MV3 : politiques navigateurs strictes | M | M | Pas de code remote, manifest minimal, audit avant soumission stores |
| Conflit licence dataset (sources hétérogènes) | M | H | Validation juridique par source, attribution propre |

---

## 15. Questions résolues (référence)

Toutes les questions ouvertes du v0.1 ont été tranchées :

1. **Géolocalisation datacenter** → ✅ Intégrée (M9)
2. **Import logs entreprise** → ✅ Intégré (M10)
3. **Plug-in navigateur** → ✅ Intégré (M11)
4. **Modèle économique** → ✅ Associatif + open-source
5. **Nom** → ✅ **Sobr.ia**
6. **Mentor Ecolab/ADEME** → ✅ Acquis
7. **Testeurs utilisateurs** → ✅ Acquis
8. **Distribution stores** → ⏳ Décision reportée S10-S11

---

## 16. Glossaire express

| Terme | Définition |
|-------|------------|
| **CO₂eq** | Équivalent CO₂ — métrique unifiée des gaz à effet de serre (GWP100) |
| **PUE** | Power Usage Effectiveness — ratio énergie totale / énergie IT d'un datacenter |
| **WUE** | Water Usage Effectiveness — litres d'eau par kWh IT |
| **IF** | Facteur d'émission (Impact Factor) — gCO₂eq par kWh selon mix élec |
| **Embodied** | Émissions incorporées (fabrication hardware) |
| **AFNOR SPEC 2314** | Référentiel français IA frugale |
| **CSRD** | Corporate Sustainability Reporting Directive (UE) |
| **MV3** | Manifest V3, format des extensions WebExtension modernes |
| **DCO** | Developer Certificate of Origin (contribution open-source) |
| **DVC** | Data Version Control |

---

*Document figé v1.0 — toute modification = ADR + bump version.*
