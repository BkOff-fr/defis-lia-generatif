# Sobr.ia

> *Mesurez la sobriété de votre IA générative.*
> *Make generative AI's footprint visible.*

**Sobr.ia** est une application native multi-plateforme (Rust + Tauri 2 + SvelteKit), accompagnée d'une extension navigateur et d'un dataset ouvert, qui mesure et visualise l'impact environnemental de l'usage des LLMs — conformément au référentiel **AFNOR SPEC 2314** sur l'IA frugale, et publiée sur **data.gouv.fr** dans le cadre du défi *« L'impact environnemental de l'IA générative »*.

> ⚠️ Le projet est en phase de cadrage (S0) — aucun code applicatif n'est encore écrit.

---

## Navigation rapide

| Si vous êtes… | Commencez par… |
|---------------|----------------|
| Curieux du projet | Ce README + [CDC v1.1](docs/CAHIER-DES-CHARGES-v1.0.md) |
| Contributeur dev / Claude Code | [`CLAUDE.md`](CLAUDE.md) (impératif) puis [`ROADMAP.md`](docs/ROADMAP.md) |
| Décideur / jury | [CDC v1.1](docs/CAHIER-DES-CHARGES-v1.0.md) §0-3 puis [Roadmap](docs/ROADMAP.md) |
| Architecte / méthodologue | [ADR](docs/adr/) (9 décisions) + [Catalogue sources](docs/sources/CATALOGUE-SOURCES.md) |
| Scientifique | [Brief S0](briefs/sprints/S0-revue-bibliographique.md) (à venir : synthèse biblio + méthodologie) |

---

## Arborescence du dépôt

```
defis-lia-generatif/
├── README.md                                 ← vous êtes ici
├── CLAUDE.md                                 ← contexte maître pour Claude Code
│
├── docs/
│   ├── CAHIER-DES-CHARGES-v1.0.md            ← spec figée (interne v1.1)
│   ├── ROADMAP.md                            ← plan 12 semaines
│   ├── adr/                                  ← 9 ADR (décisions archi)
│   │   ├── README.md                         ← index des ADR
│   │   ├── ADR-0001-rust-tauri.md
│   │   ├── ADR-0002-sveltekit.md
│   │   ├── ADR-0003-sqlite-duckdb.md
│   │   ├── ADR-0004-monte-carlo.md
│   │   ├── ADR-0005-webextension-mv3.md
│   │   ├── ADR-0006-licences.md
│   │   ├── ADR-0007-dvc.md
│   │   ├── ADR-0008-observable-plot.md
│   │   └── ADR-0009-medallion-architecture.md   ← pipeline Copper/Silver/Gold
│   ├── sources/
│   │   └── CATALOGUE-SOURCES.md              ← 10 sources de données documentées
│   ├── methodology/                          ← à remplir en S0
│   │   └── (AFNOR-SPEC-2314-synthese.md à venir)
│   ├── personas/                             ← à enrichir si besoin
│   └── archive/
│       └── CAHIER-DES-CHARGES-v0.1.md
│
├── briefs/
│   └── sprints/
│       └── S0-revue-bibliographique.md       ← le sprint actif
│
├── research/
│   ├── biblio/                               ← à remplir en S0
│   └── papers/                               ← PDFs à archiver (Copper-like)
│
└── .github/
    └── workflows/                            ← à remplir en S1
```

---

## En une phrase, qu'est-ce qu'on construit ?

**Une stack open-source en 11 modules** (référentiel, estimateur, workbench, simulateur, comparateur, exports, audit ledger, aide, géolocalisation datacenter, import logs entreprise, extension navigateur), packagée en **app native multi-plateforme (Windows, macOS, Linux, Android, iOS, Web/Wasm) + extension Chrome/Firefox + dataset publié sur data.gouv.fr + notebook scientifique reproductible**, le tout sous licences ouvertes (MIT, Etalab 2.0, CC-BY).

---

## L'architecture en une image

```
   Sources ouvertes              Pipeline médaillon            Consommation
   (ADEME, RTE, HF,         ┌───────────────────────────┐     (App + Notebook
    EcoLogits, papers…)     │ 🟫 Copper  → raw immutable │      + dataset publié
           ───────────►     │ 🥈 Silver → typed Parquet  │ ──►  + extension MV3
                            │ 🥇 Gold   → SQLite+Parquet │      + rapport PDF)
                            └───────────────────────────┘
                              (trait Rust DataLayer,
                               orchestré par DVC)
```

Détails : [ADR-0009](docs/adr/ADR-0009-medallion-architecture.md), [CDC §7.3](docs/CAHIER-DES-CHARGES-v1.0.md).

---

## État d'avancement

| Phase | Statut |
|-------|--------|
| Cadrage (CDC + ADR + roadmap) | ✅ Terminé |
| S0 — Revue biblio + méthodologie | 🔜 À démarrer |
| S1 — Bootstrap technique | ⏳ |
| S2-S3 — Pipeline médaillon (ingestion) | ⏳ |
| S4-S5 — Estimateur scientifique | ⏳ |
| S6-S8 — UI + extensions modules | ⏳ |
| S9 — Notebook + rapport | ⏳ |
| S10-S11 — Polish + tests utilisateurs | ⏳ |
| S12 — Soumission data.gouv.fr | ⏳ |

---

## Licences

- **Code** : MIT — voir `LICENSE` (à créer en S1)
- **Données publiées** : Etalab 2.0 (compatible CC-BY)
- **Documentation** : CC-BY 4.0
- **Logo / identité** : CC-BY-SA 4.0

---

## Contributeurs et rôles

| Rôle | Personne |
|------|----------|
| Porteur du projet, étudiant candidat | Thibault |
| Chef de projet, architecte, méthodologue | Claude Cowork |
| Réalisation code | Claude Code |
| Mentor scientifique | (Ecolab/ADEME) |
| Testeurs utilisateurs | 5 personnes (1 par persona) |

---

## Pour démarrer (à venir en S1)

```bash
git clone <url-repo>
cd defis-lia-generatif
./scripts/bootstrap.sh        # installe Rust, Node, Tauri CLI, Quarto, DVC

# Lancer le pipeline d'ingestion (médaillon)
cargo run -p sobria-ingest -- pipeline run

# Lancer l'app Tauri en dev
cargo run -p sobria-app
```

---

*« Le médium est le message. » — McLuhan, appliqué ici à un outil qui mesure la sobriété et qui est lui-même frugal.*
