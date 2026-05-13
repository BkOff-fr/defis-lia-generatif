# Sobr.ia

> **Mesurez et comprenez l'empreinte environnementale de vos prompts LLM.**
> Native, frugale, scientifique, open source.
>
> *Candidat au défi data.gouv.fr — « L'impact environnemental de l'IA générative »*

---

## En une phrase

Sobr.ia est une application **native multi-plateforme** (Tauri 2 + Rust +
SvelteKit) qui estime, journalise et restitue l'empreinte CO₂eq, énergie
et eau d'un usage IA générative, avec une **rigueur scientifique
auditable** (AFNOR SPEC 2314, Monte-Carlo, audit ledger SHA-256) et un
**angle territorial français unique** (ComparIA × RTE IRIS).

## Différenciateurs

| 🎯 | Détail |
|---|---|
| **Audit chaîné SHA-256** | Chaque estimation est journalisée dans un ledger ACID SQLite avec chaînage cryptographique. Anti-tampering vérifiable. **Aucun concurrent ne propose ça.** |
| **Territoire FR (M20)** | Cartographie des sites industriels par IRIS (RTE/NaTran/Teréga) croisée avec ComparIA. Sankey énergétique national. Différenciateur unique du défi data.gouv. |
| **Datasheet Gebru** | Génération automatique du format académique standard (Gebru et al. 2018) pour reproductibilité scientifique. Adopté par NeurIPS, ICML, FAccT. |
| **Rapport CSRD/AGEC** | Export PDF officiel + JSON-LD PROV-O signé SHA-256, prêt pour reporting réglementaire UE. |
| **Privacy by design** | Tout en local. Zéro télémétrie, zéro tracking, zéro appel réseau au runtime. RGPD : droit à l'oubli implémenté avec préservation de la chaîne d'audit. |
| **Frugalité incarnée** | Binaire ≈ 15 MB optimisé (LTO, opt-level=z, strip). Méta-cohérent : l'outil de mesure consomme peu. |

## Méthodologie

- **Référentiel** : [AFNOR SPEC 2314](https://norminfo.afnor.org/norme/AFNOR%20SPEC%202314/) — référentiel français de mesure de l'empreinte environnementale des LLMs.
- **Estimation** : Monte-Carlo N=10⁴ tirages, seed déterministe (42), reproductible à la nanoseconde.
- **Paramètres distributionnels** : log-normales sourcées sur HF AI Energy Score, RTE eco2mix, ADEME Base Empreinte, Mytton 2021.
- **Indicateurs** : CO₂eq (g), Énergie (Wh), Eau (L), avec intervalles P5/P50/P95.
- **Validation croisée à ±15%** : Luccioni et al. 2023, EcoLogits 2024.

Détails complets : [`docs/methodology/`](docs/methodology/) et [CDC v1.4](docs/CAHIER-DES-CHARGES-v1.0.md).

## 13 modules essentiels (v1.0)

### 🏆 Cœur méthodologique & transparence
- **M1 — Estimer un prompt** : moteur Monte-Carlo + UI unitaire
- **M7 — Journal d'audit** : ledger chaîné SHA-256, anti-tampering
- **M9 — Référentiel modèles** : 8 modèles avec triplets P5/P50/P95
- **M8 — Méthodologie interactive** : doc embarquée
- **M14 — À propos** : licences, sources, mentions

### 🇫🇷 Angle territorial unique
- **M20 — Territoire FR** : IRIS + Sankey énergétique (RTE eco2mix)
- **M12 — Datacenters Europe** : 28 DC carte Leaflet + drill-down 24h

### 💼 Use cases pros & chercheurs
- **M22 — Rapport CSRD/AGEC** : PDF + JSON-LD PROV-O conforme SPEC 2314
- **M17 — Empreinte projet** : datasheet Gebru 2018 (reproductibilité)
- **M3 — Comparer modèles** : benchmark côte-à-côte 3 indicateurs

### 🎓 Pédagogie & rétention
- **M13 — Simulateur « Et si...? »** : 7 leviers, waterfall, projection 12 mois
- **M15 — Dashboard personnel** : agrégat jour/semaine/mois
- **M25 — Eco-budget** : objectifs personnels + alerte dépassement

**Différé v1.1+** : M2 Workbench · M5 Rapports génériques · M6 Géoloc unitaire · M10 Import logs · M11 Extension navigateur · M16 Forecaster UI · M18 Batch CSV UI · M19 Équipe · M21 Alertes · M23 Marchés publics · M24 Apprendre.

Cf. [ADR-0011](docs/adr/ADR-0011-reduction-perimetre-v1-0.md) pour la justification de la réduction de périmètre.

## Personas et bundles

L'app sert **5 publics** aux exigences distinctes. Au premier lancement,
un wizard d'onboarding propose 5 personas avec bundles pré-cochés
(activables/désactivables individuellement) :

| Persona | Bundle par défaut |
|---|---|
| 🎓 Étudiant·e | M1, M8, M13, M14, M15, M25 |
| 🧑‍💻 Pro tech | M1, M3, M7, M8, M9, M13, M14 |
| 🏢 Entreprise | M1, M7, M12, M14, M15, M17, M20, M22, M25 |
| 🏛️ Collectivité | M1, M8, M12, M14, M17, M20, M22 |
| 🔬 Chercheur·se | M1, M3, M7, M8, M9, M14, M17 |

Cf. [ADR-0010](docs/adr/ADR-0010-personas-and-module-gating.md).

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

- **9 crates Rust** : `sobria-core`, `sobria-estimator`, `sobria-audit`, `sobria-referentiel`, `sobria-geoloc`, `sobria-import`, `sobria-export`, `sobria-ingest`, `sobria-app`.
- **Architecture médaillon** Copper/Silver/Gold pour toutes les sources externes ([ADR-0009](docs/adr/ADR-0009-medallion-architecture.md)).
- **Pipeline ingest** unique : `cargo run -p sobria-ingest -- fetch ...` télécharge ODRÉ + RTE en local.
- **IPC Tauri** : 30+ commandes typées DTO ↔ TypeScript.

Cf. [`docs/adr/`](docs/adr/) pour les 11 décisions architecturales.

## Statut

- **Backend Rust** : ✅ complet, 250+ tests, clippy `-D warnings` clean.
- **Frontend SvelteKit** : ✅ 13 modules livrés, design system v2 (ink/lime/ivory).
- **Données réelles** : ✅ fetch automatique ODRÉ + RTE via `sobria-ingest`. **Aucune valeur inventée.**
- **Méthodologie validée** : ✅ croisement Luccioni / EcoLogits dans `notebook/validation.qmd`.
- **Documentation** : ✅ 11 ADR + CDC v1.4 + 13 briefs chantiers.

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

---

*Sobr.ia — Made in France · Privacy by design · v0.3.x*
