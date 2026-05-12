# Rétrospective globale — Chantiers #1 à #4 (Pipeline médaillon complet)

> **Date** : 12 mai 2026
> **Sprints couverts** : S1 (fin) → S5 (fin)
> **Statut** : jalon stable, prêt à commiter et tagger.

---

## Ce qui a été livré

### Vue d'ensemble

**Le pipeline médaillon complet est opérationnel** sur des données synthétiques en CI, avec deux sources officielles du défi data.gouv.fr et un Gold final consommable par l'app Tauri.

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   ComparIA (3 Parquet)         RTE IRIS (CSV + GeoJSON)              │
│         │                              │                             │
│         ▼ ingest_copper                ▼ ingest_copper               │
│   🟫 Copper                       🟫 Copper                          │
│   manifest.json (SHA-256)         manifest.json                      │
│         │                              │                             │
│         ▼ promote_silver               ▼ promote_silver              │
│   🥈 3 entités Parquet            🥈 1 entité Parquet                │
│   (_copper_sha256, _ingested_at)  (_copper_sha256, _ingested_at)     │
│         │                              │                             │
│         └──────────────┬───────────────┘                             │
│                        ▼ assemble_gold                               │
│   🥇 referentiel.sqlite (3 tables : sources, silver_entities,        │
│       lineage)                                                       │
│   🥇 analytics.parquet (catalogue 4 entités, lisible DuckDB)         │
│   🥇 datasheet.jsonld (PROV-O + schema.org/Dataset)                  │
│   🥇 MANIFEST.sha256 (intégrité)                                     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Chantiers et métriques

| Chantier | Lignes code | Lignes tests | Tests | Statut |
|----------|------------:|-------------:|------:|:------:|
| C01 — Foundation pipeline | ~1 900 | embarqués | 50 | ✅ |
| C02 — ComparIA source | 353 | 243 | 6 | ✅ |
| C03 — RTE IRIS source | 333 | 190 | 6 | ✅ |
| C04 — Gold assembly | 474 | 202 | 5 | ✅ |
| `sobria-core` | 759 | embarqués | 27 | ✅ |
| **Total jalon** | **~4 000** | **~1 350** | **~85** | **✅** |

### Promesse d'ADR-0009 tenue

> *« Onboarding d'une nouvelle source = un seul trait à implémenter. »*

**Vérifié sur la pratique** : ComparIA (Parquet) et RTE IRIS (CSV+GeoJSON) ont chacun nécessité ~330 lignes de code source-spécifique, sans aucune modification de la foundation. La 3ᵉ source (ADEME, Tier 2) prendrait moins de temps encore vu l'expérience accumulée.

---

## Décisions techniques notables

1. **Erreurs collectées, pas propagées** dans le registry — si une source échoue, les autres continuent. Choix défensif pour la CI nocturne.

2. **Polars en `spawn_blocking`** — sync bloquant encapsulé pour ne pas saturer Tokio.

3. **rusqlite WAL** — permet à l'app Tauri de lire pendant qu'un repipeline tourne.

4. **Schémas Silver permissifs en v1** — `additionalProperties: true`, on conserve les colonnes ComparIA/ODRÉ d'origine en ajoutant `_copper_sha256` + `_ingested_at`. Le typage strict viendra en v2 quand on aura les specs précises.

5. **GeoJSON RTE en Copper uniquement** — pas de promotion Silver, sera consommé directement par le module M12 (cartographie). Décision documentée explicitement dans `contribute_gold` notes.

6. **`gold_artifacts: Option<GoldArtifacts>`** — un échec d'assemblage n'invalide pas le pipeline (les Silver restent valides).

7. **Loopback HTTP autorisé** dans le manifest pour les tests wiremock (HTTPS strict en production).

---

## Definition of Done — bilan global

- [x] `cargo fmt --check` passe (rustfmt config stable-only)
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passe (pedantic + allows ciblés)
- [x] `cargo build --workspace` passe
- [x] `cargo test --workspace --all-features` ≥ 85 tests verts
- [x] Aucun `unsafe` (deny au niveau crate)
- [x] Aucun `unwrap`/`expect` en code production non documenté
- [x] Tracing structuré sur chaque point d'entrée publique
- [x] Documentation `///` sur tout item public
- [x] CHANGELOG.md à jour
- [x] Commits Conventional Commits

---

## Frictions et leçons apprises

1. **Truncation des Write tool** sur fichiers > ~800 octets — contournée via bash heredoc (`cat > file <<'EOF' ... EOF`). À garder en tête.

2. **`clippy::pedantic`** très bavard sur Rust 1.95 — il a fallu ajouter 7 allows ciblés (`missing_errors_doc`, `missing_panics_doc`, `doc_markdown`, `duration_suboptimal_units`, `needless_pass_by_value`, `float_cmp`, `module_name_repetitions`). C'est plus une checklist d'idiomes qu'un filet de sécurité — `clippy::all` (le défaut) suffit pour les vrais bugs.

3. **Polars 0.43 ↔ hashbrown 0.15** — un breaking transitive a forcé un bump à 0.46. Anticiper le pinning en production.

4. **`Context::default()` dans les tests** — pollue le cwd avec des fichiers `data/gold/`. Refactorisé en `temp_ctx()` helper via `tempfile::tempdir()`. À garder comme bonne pratique.

5. **`rustfmt.toml`** — les options "nightly" silencieusement ignorées : éviter `imports_granularity`, `wrap_comments`, `group_imports`, etc. sauf si pipeline nightly assumé.

---

## Pré-requis pour la suite

### Chantier #5 (estimateur) prendra en input

- Le `referentiel.sqlite` produit par le pipeline (lecture rapide indexée).
- Le `analytics.parquet` produit (lecture DuckDB pour scénarios).
- Le `gold_lineage` pour annoter chaque estimation de son contexte.

Tout est en place côté data, on peut attaquer la logique scientifique en confiance.

---

## Prochaine étape recommandée

**Commit du jalon** sous le tag `v0.1.0-foundation` ou similaire :

```bash
git add -A
git status                    # vérifier qu'aucun fichier sensible n'est inclus
git commit -m "<voir message dans la doc>"
git tag -a v0.1.0-foundation -m "Pipeline médaillon complet (C01-C04)"
```

Le message de commit complet est rédigé dans la synthèse Cowork de l'option D.

Après commit : choix entre **chantier #5 (estimateur Monte-Carlo)** ou **chantier #6 (audit ledger ACID)**.
