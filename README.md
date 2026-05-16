# Sobr.ia

> **Sobr.ia mesure l'empreinte de vos prompts IA en local, agrège les
> chiffres officiels des fabricants (Mistral × ADEME, Google, Meta) et
> vous donne un journal scientifique reproductible — pour particulier,
> équipe ou administration, sans cloud Sobr.ia.**
>
> *Candidat au défi data.gouv.fr — « L'impact environnemental de l'IA générative »*

---

## Sobr.ia, c'est quoi ?

Sobr.ia est une **application qui vous dit combien chaque prompt IA
coûte vraiment** — en grammes de CO₂, en watts-heures, en gouttes
d'eau, et en équivalents concrets (km de voiture, douches, minutes
de streaming).

Elle tourne **en 100 % local** sur votre ordinateur (Windows, macOS,
Linux). **Aucune inscription, aucun compte, aucun envoi de prompt
vers un serveur tiers.**

Sous le capot, **deux méthodologies scientifiques** (AFNOR SPEC 2314
+ EcoLogits peer-reviewed) calculent vos chiffres, et **les
disclosures officielles des fabricants** (Mistral × ADEME, Google
Gemini, Meta Llama) viennent enrichir le référentiel.

---

## Pour qui ?

Sobr.ia sert **cinq publics** distincts, avec un bundle de modules
adapté à chacun. Vous choisissez votre profil au premier lancement
et personnalisez ensuite librement.

### 🎓 Étudiant·e / Curieux·se

Comprendre l'empreinte de vos usages IA, apprendre les bons
réflexes, suivre votre semaine.

→ [Guide Étudiant·e](docs/personas/student.md)

### 💻 Professionnel·le tech (dev, ML eng)

Estimer, comparer, journaliser pour vos intégrations API. Audit
SHA-256, exports JSON-LD PROV-O.

→ [Guide Pro Tech](docs/personas/pro-tech.md)

### 🏢 Entreprise (DSI, RSE)

Piloter votre scope 3 IA, sortir un rapport CSRD, forecast budget
carbone, agrégation équipe self-hosted.

→ [Guide Entreprise](docs/personas/enterprise.md)

### 🏛️ Collectivité / Service public

Empreinte territoriale (IRIS RTE × ComparIA), critères de marchés
publics frugaux, rapport AGEC.

→ [Guide Service public](docs/personas/public-sector.md)

### 🔬 Chercheur·se / Journaliste

Datasheet Gebru, multi-méthodologie, datasets publiables,
reproductibilité scientifique, citation DOI.

→ [Guide Chercheur·se](docs/personas/researcher.md)

---

## Pourquoi Sobr.ia ?

| 🎯 | Détail |
|---|---|
| **Catalogue souverain de méthodologies** | Sobr.ia v1.0 embarque **2 méthodologies scientifiques** d'estimation d'empreinte LLM (AFNOR SPEC 2314 française + EcoLogits 2026-01 peer-reviewed). L'utilisateur choisit la sienne par défaut, active les autres en référence pour comparer les résultats côté Atelier. **Aucun concurrent ne fait ça** : EcoLogits / BoaVizta / AI Energy Score sont mono-méthodologie. Cf. [ADR-0012](docs/adr/ADR-0012-multi-methodology-engine.md). |
| **Tiers de confiance vendor disclosure** | Sobr.ia agrège les disclosures environnementales officielles publiées par les fabricants : **Mistral × ADEME** (Large 2, 1.14 gCO₂eq pour 400 tokens), **Google Gemini** (0.03 gCO₂eq prompt médian, août 2025), **Meta Llama** (training location-based / market-based). Encadrés explicites par modèle dans la Bibliothèque. |
| **Audit chaîné SHA-256 avec méthodologie tracée** | Chaque estimation est journalisée dans un ledger ACID SQLite avec chaînage cryptographique + **méthodologie utilisée** (colonne `method`). Anti-tampering vérifiable, reproductible à la nanoseconde, filtrable par méthodologie pour reporting CSRD historique. |
| **Angle territorial FR unique** | Cartographie des sites industriels par IRIS (RTE/NaTran/Teréga) croisée avec ComparIA. Sankey énergétique national. Différenciateur unique du défi data.gouv. |
| **Datasheet Gebru** | Génération automatique du format académique standard (Gebru et al. 2018) pour reproductibilité scientifique. Adopté par NeurIPS, ICML, FAccT. |
| **Rapport CSRD/AGEC** | Export PDF officiel + JSON-LD PROV-O signé SHA-256, prêt pour reporting réglementaire UE. |
| **Privacy by design** | Tout en local. Zéro télémétrie, zéro tracking, zéro appel réseau au runtime. RGPD : droit à l'oubli implémenté avec préservation de la chaîne d'audit. |
| **Frugalité incarnée** | Binaire ≈ 15 MB optimisé (LTO, opt-level=z, strip). Méta-cohérent : l'outil de mesure consomme peu. |

## Méthodologies disponibles

Sobr.ia propose **un catalogue de méthodologies** sélectionnables par l'utilisateur, exposé via la page `/methodologies`. Ajouter une méthodologie en v1.1+ = implémenter un trait + une entrée dans le registry (cf. [ADR-0012](docs/adr/ADR-0012-multi-methodology-engine.md)).

| Méthodologie | Statut Sobr.ia | Référence | Licence |
|---|---|---|---|
| **AFNOR SPEC 2314 (Sobr.ia)** *(défaut)* | Méthode publique de référence FR, calibration en cours | [AFNOR SPEC 2314](https://norminfo.afnor.org/norme/AFNOR%20SPEC%202314/) | Spec publique ; code MIT |
| **EcoLogits 2026-01** | Peer-reviewed · reproduit à ≤ 1 % | [doi:10.21105/joss.07471](https://doi.org/10.21105/joss.07471) | CC BY-SA 4.0 |
| *BoaVizta · AI Energy Score · GreenAlgorithms* | Prévu v1.1+ | — | — |

### Détails communs

- **Estimation** : Monte-Carlo N=10⁴ tirages (AFNOR), formules déterministes (EcoLogits), seed déterministe (42), reproductible à la nanoseconde.
- **Indicateurs** : CO₂eq (g), Énergie (Wh), Eau (L), avec intervalles P5/P50/P95.
- **Validation EcoLogits port** : 3 `ReproductionCase` cibles recalculées en Python depuis les formules officielles, écart ≤ 1 % vs port Rust. Cf. [`notebook/validation.qmd`](notebook/validation.qmd) (Quarto + Python) et `cargo test -p sobria-estimator validation`.
- **Audit** : chaque estimation est tracée avec sa méthodologie dans le ledger SHA-256. Un rapport CSRD régénéré à partir d'entrées historiques utilise la méthodologie qui était active au moment du calcul (cohérence rétroactive garantie).

Détails complets : [`docs/methodology/`](docs/methodology/), [ADR-0012](docs/adr/ADR-0012-multi-methodology-engine.md) et [CDC v1.4](docs/CAHIER-DES-CHARGES-v1.0.md).

## 13 modules essentiels (v1.0)

### 🏆 Cœur méthodologique & transparence
- **Estimer un prompt** : moteur Monte-Carlo + UI unitaire
- **Journal d'audit** : ledger chaîné SHA-256, anti-tampering
- **Bibliothèque de modèles** : 8 modèles avec triplets P5/P50/P95 + encadrés vendor disclosure (Mistral, Google, Meta)
- **Comment ça marche (méthodologie)** : doc embarquée
- **À propos** : licences, sources, mentions

### 🇫🇷 Angle territorial unique
- **Territoire FR** : IRIS + Sankey énergétique (RTE eco2mix)
- **Datacenters Europe** : 28 DC carte Leaflet + drill-down 24h

### 💼 Use cases pros & chercheurs
- **Rapport réglementaire (CSRD/AGEC)** : PDF + JSON-LD PROV-O conforme SPEC 2314
- **Datasheet scientifique** : datasheet Gebru 2018 (reproductibilité)
- **Comparer modèles** : benchmark côte-à-côte 3 indicateurs

### 🎓 Pédagogie & rétention
- **Simulateur « Et si...? »** : 7 leviers, waterfall, projection 12 mois
- **Tableau de bord** : agrégat jour/semaine/mois
- **Eco-budget** : objectifs personnels + alerte dépassement

**Différé v1.1+** : Workbench multi-prompts · Rapports génériques · Géoloc unitaire · Import logs · Forecaster UI · Batch CSV UI · Alertes · Marchés publics · Apprendre.

**Extension navigateur** : livrée en v0.6.0 (cf. ci-dessous).
**Mode Équipe** : self-hosted livré en v0.7.0, polish v0.7.1 (cf. ci-dessous).

## Extension navigateur (v0.6.0)

Mesurez l'empreinte de vos prompts **directement dans le navigateur**, sans
ouvrir l'app Tauri. Sites supportés : ChatGPT, Claude (claude.ai), Le Chat
(chat.mistral.ai). Gemini reporté v0.7+.

- **WebExtension MV3** (Chrome 120+ et Firefox 120+) — ~207 KB par bundle,
  vanilla DOM + TypeScript strict, zéro dépendance runtime hors
  `webextension-polyfill`.
- **Estimation 100 % locale** : port JS du moteur Sobr.ia (AFNOR + EcoLogits),
  parité < 2 % vs Rust. Aucun prompt n'est envoyé à un serveur distant.
- **Badge circulaire** à côté du composer affichant le score Sobr.ia (A-F)
  + estimation gCO₂eq/Wh/mL après envoi.
- **Pairing perso optionnel** : un code 6 chiffres généré dans
  `/parametres` permet à l'app desktop d'ingérer les estimations dans le
  Journal + Dashboard via un **native messaging bridge** (binaire local
  `sobria-bridge`, pas de port réseau).
- **Privacy by design** : permissions minimales (`activeTab`, `storage`,
  `nativeMessaging` opt-in), CSP stricte, no remote code.

Téléchargement (v0.6.0) — Releases GitHub :

- `sobria-extension-chrome-v0.6.0.zip` (load unpacked depuis
  `chrome://extensions/`)
- `sobria-extension-firefox-v0.6.0.xpi` (drag-drop dans `about:debugging`)

Architecture : [ADR-0013](docs/adr/ADR-0013-extension-pairing-team-mode.md)
· installation manifest natif : [`crates/sobria-bridge/README.md`](crates/sobria-bridge/README.md).

Cf. [ADR-0011](docs/adr/ADR-0011-reduction-perimetre-v1-0.md) pour la justification de la réduction de périmètre.

## Mode Équipe self-hosted (v0.7.0 + polish v0.7.1)

Pour les TPE/PME et DSI qui veulent **agréger les estimations de leurs N
employés sans cloud externe**. Un binaire Rust standalone
`sobria-team-aggregator` (~15 MB) se déploie sur poste admin / NAS /
VPS interne et expose :

- **Dashboard admin** (Svelte embedded, 201 KB) : analytics agrégés
  (séries quotidiennes/hebdo/mensuelles, top modèles, top employés
  anonymisables, breakdown AFNOR vs EcoLogits, 4 cards KPI), gestion
  des **enrollment codes** 12 chiffres (création / révocation),
  liste des employés enrôlés avec leurs totaux, **alertes seuils
  CSRD** (v0.7.1) : plafond gCO₂eq par jour/semaine/mois avec
  notification webhook ou email.
- **Dashboard employé perso** : son usage personnel + transparence
  (« ce qui est partagé / ce qui ne l'est jamais »).
- **API REST** `/api/v1/*` (JWT HS256 24h + refresh 7j Argon2id, TLS
  auto-signé via rcgen + ring, pas d'OpenSSL).
- **Exports** CSRD PDF (réutilise la chaîne `sobria-export`), PROV-O
  JSON-LD avec per-user agents anonymisables, CSV brut RFC 4180.
- **Extension + app desktop** étendues : section « Mode Équipe » dans
  les Options avec dispatch radio (`local | team | both`). **L'app
  Tauri permet désormais (v0.7.1) d'enrôler son device depuis
  `/parametres` sans toucher la SQLite.**
- **Outillage admin v0.7.1** : `sobria-team-aggregator admin
  reset-password <user>` (Argon2id PHC + révocation tokens),
  `sobria-team-aggregator admin list`, et `serve --regen-cert` pour
  rotation TLS sans perdre la base.

**Aucun cloud Sobr.ia n'est impliqué** — votre entreprise contrôle
son serveur et ses données.

Quickstart :

```bash
chmod +x sobria-team-aggregator-linux-x86_64
./sobria-team-aggregator --data-dir ./team-data init \
    --admin-username admin --admin-password 'CHANGE-ME'
./sobria-team-aggregator --data-dir ./team-data serve --port 8443
```

Doc complète : [`docs/operations/team-aggregator.md`](docs/operations/team-aggregator.md)
(quickstart, TPE/PME systemd, DSI reverse proxy + Let's Encrypt,
sauvegardes SQLite, upgrade, troubleshooting). Bonus : Dockerfile
multi-stage dans [`crates/sobria-team-aggregator/Dockerfile`](crates/sobria-team-aggregator/Dockerfile).

Architecture : [ADR-0013 Phase 2](docs/adr/ADR-0013-extension-pairing-team-mode.md)
· brief : [`briefs/chantiers/C28-mode-equipe-self-hosted.md`](briefs/chantiers/C28-mode-equipe-self-hosted.md).

## Stack technique

| Couche | Technologie |
|---|---|
| Wrapper natif | Tauri 2 |
| Backend | Rust stable (≥ 1.79), workspace 9 crates |
| Frontend | SvelteKit 2 + TypeScript strict + Svelte 5 runes |
| DB transactionnelle | SQLite WAL (rusqlite 0.32) |
| DB analytique | DuckDB (duckdb-rs 1.1) |
| ETL Rust | reqwest + serde + polars 0.46 |
| Dataviz | Chart.js + SVG natif (Sankey custom) |
| Notebook | Quarto 1.4+ (validation croisée) |
| Versionnage données | DVC 3.x |
| Génération PDF | printpdf 0.7 (pure Rust) |
| Sérialisation JSON-LD | serde_json + vocabularies schema.org / PROV-O / DCAT |

## Installation

### Pré-requis
- Rust stable (rustup recommandé)
- Node.js 22+
- Tauri prerequisites : voir <https://v2.tauri.app/start/prerequisites/>

### Bootstrap

```bash
./scripts/bootstrap.sh         # Linux / macOS / WSL
# Windows : équivalent manuel (cargo install tauri-cli, npm install, etc.)
```

### Développement

```bash
cargo tauri dev                # lance l'app en dev (hot reload front + back)
cargo test --workspace         # 250+ tests Rust
cd web && npm run check        # type-check SvelteKit
cd web && npm run test         # Playwright e2e
```

### Données officielles (à pull une fois)

```bash
cargo run -p sobria-ingest -- fetch territoire-fr --limit 200
cargo run -p sobria-ingest -- fetch rte-mix --year 2023
```

Datasets ODRÉ Etalab 2.0, traçabilité SHA-256 + URL source dans le JSON produit.

### Build release

```bash
./scripts/build-all.sh         # produit binaires Win / macOS / Linux
```

## Architecture

- **10 crates Rust** : `sobria-core`, `sobria-estimator`, `sobria-audit`, `sobria-referentiel`, `sobria-geoloc`, `sobria-import`, `sobria-export`, `sobria-ingest`, `sobria-app`, `sobria-bridge` (native messaging extension navigateur).
- **Architecture médaillon** Copper/Silver/Gold pour toutes les sources externes ([ADR-0009](docs/adr/ADR-0009-medallion-architecture.md)).
- **Pipeline ingest** unique : `cargo run -p sobria-ingest -- fetch ...` télécharge ODRÉ + RTE en local.
- **IPC Tauri** : 37+ commandes typées DTO ↔ TypeScript (dont 7 pour le pairing extension navigateur v0.6.0).

Cf. [`docs/adr/`](docs/adr/) pour les 13 décisions architecturales (ADR-0012 = catalogue multi-méthodologie, ADR-0013 = WebExtension MV3 + pairing).

## Statut

- **Backend Rust** : ✅ complet, 250+ tests, clippy `-D warnings` clean. **Trait `EmpreinteEngine`** + 2 engines (AFNOR Sobr.ia + EcoLogits port direct).
- **Frontend SvelteKit** : ✅ 13 modules livrés, design system v2 (ink/lime/ivory). Nouvelle page `/methodologies` (catalogue) + panneau "Voir aussi" dans M1.
- **Données réelles** : ✅ fetch automatique ODRÉ + RTE via `sobria-ingest` (200 sites industriels FR + mix élec 2023 validé à <2% du Bilan RTE). Profils horaires 24h des datacenters : forme typique modélisée en v1.0, **pull ENTSO-E live prévu v1.1** — documenté honnêtement.
- **Validation méthodologique** : ✅ port direct EcoLogits 2026-01, écart ≤ 1 % vs formules officielles sur 3 cas (Llama 3.1 70B, Mistral Large 2). 6 cas de plausibilité sur l'engine AFNOR. Notebook Python reproductible (`notebook/validation.qmd`).
- **Audit ledger v2** : ✅ migration v1 → v2 idempotente (colonne `method` ajoutée). Traçabilité méthodologique pour reporting CSRD historique.
- **Documentation** : ✅ 12 ADR + CDC v1.4 + 14 briefs chantiers (dont C24 multi-méthodologie).

## Licences

- **Code** : MIT
- **Données embarquées** (datacenters, IRIS, mix élec) : Etalab 2.0 (sources ODRÉ/RTE/AIB)
- **Polices** : SIL Open Font License 1.1 (Geist, Instrument Serif, JetBrains Mono)
- **Documentation** : CC-BY 4.0

Sources cliquables : voir l'écran **À propos** dans l'app.

## Contributions

Issues + PR bienvenues sur GitHub. Pour proposer un nouveau module ou
modifier un bundle persona, ouvrir d'abord un mini-ADR dans `docs/adr/`.

## Citation

Si vous utilisez Sobr.ia dans un travail académique :

```bibtex
@software{sobria_2026,
  title   = {Sobr.ia: Empreinte environnementale auditable des LLMs},
  author  = {Thibault et contributeurs Sobr.ia},
  year    = {2026},
  url     = {https://github.com/BkOff-fr/defis-lia-generatif},
  license = {MIT}
}
```

*Un DOI Zenodo sera publié avec la release v0.8.0 (cf. C32.5).*

---

*Sobr.ia — Made in France · Privacy by design*
