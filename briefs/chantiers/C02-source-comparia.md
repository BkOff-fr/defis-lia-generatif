# Chantier #2 — Source ComparIA (première source concrète)

> **Pré-requis** : chantier #1 (foundation) mergé sur main.
> **Crates touchées** : `sobria-ingest` uniquement (+ schémas JSON).
> **Durée cible** : 2-3 jours.
> **Approche** : on exploite la foundation. **Zéro réinvention.**

---

## 0. Pourquoi ComparIA en premier

C'est le **dataset central du défi data.gouv.fr** (voir CDC §0). En l'implémentant
en premier :

- On valide que la foundation tient sur un cas réel (pas juste sur des mocks).
- On démontre la promesse d'ADR-0009 : *« une nouvelle source = un seul trait à implémenter »*.
- On constitue le socle data sur lequel les autres modules s'appuieront (M2 estimator, M3 workbench, M5 comparateur, M12 territoire).

---

## 1. Données à ingérer

Voir `docs/sources/CATALOGUE-SOURCES.md` S01.

| Fichier | Format | Taille | URL data.gouv.fr |
|---------|--------|--------|------------------|
| `conversations.parquet` | Parquet | 682 MB | `r/7651fd0b-f222-43b3-8db8-ed6ae660d313` |
| `votes.parquet` | Parquet | 733 MB | `r/4ffc86e1-84a4-4fdc-9726-66408e596fef` |
| `reactions.parquet` | Parquet | 3,4 GB | `r/9dd3d51f-4299-4193-ab46-81ae039fe1be` |

**Licence** : Licence Ouverte Etalab 2.0. **Authentification** : aucune.

---

## 2. Conception

### 2.1 Hypothèse explicite

Je n'ai pas la spec exacte des colonnes des 3 Parquet ComparIA en main au moment
de la conception. **Stratégie défensive** :

- Première version (v1) : **schéma Silver = passthrough enrichi**. On lit le
  Parquet Copper avec `polars`, on inspecte le schéma dynamiquement, on écrit
  un Parquet Silver qui ajoute deux colonnes systématiques : `_copper_sha256`
  (lineage) et `_ingested_at` (timestamp UTC).
- Toutes les colonnes ComparIA d'origine sont conservées telles quelles.
- Le mapping métier précis (ex: extraire `co2_eq_g`, `tokens_in`, `model_id`)
  viendra dans une v2 du schéma Silver une fois la doc ComparIA validée.

Ce choix permet d'**avancer maintenant sur le pipeline** sans bloquer sur la
spec, tout en garantissant que les données brutes restent accessibles côté Silver.

### 2.2 Architecture du module

```
crates/sobria-ingest/src/sources/
├── mod.rs              ← liste publique des sources
└── comparia.rs         ← struct ComparIASource + impl DataLayer
```

### 2.3 Flux

```
ingest_copper
  ├─ pour chaque fichier (conversations, votes, reactions) :
  │   ├─ télécharger via Downloader::fetch_to_file (hash + retry)
  │   └─ ajouter au CopperManifest
  ├─ écrire manifest.json
  └─ retourner CopperSnapshot

promote_silver
  └─ pour chaque CopperRef :
      ├─ tokio::task::spawn_blocking { polars lazy scan }
      ├─ enrichir avec _copper_sha256 + _ingested_at
      ├─ écrire Parquet Silver
      └─ retourner SilverEntity (avec lineage)

contribute_gold
  └─ déclarer tables touchées + notes méthodologiques
     (la consolidation finale Gold est faite par le registry)
```

### 2.4 Polars en contexte async

Polars est **synchrone bloquant**. On l'enveloppe systématiquement dans
`tokio::task::spawn_blocking` pour ne pas bloquer le runtime tokio.

---

## 3. Schémas Silver v1

Voir `schemas/silver/comparia_*-v1.json`. Trois schémas, tous suivent le même
modèle minimal :

- `_copper_sha256` (string, 64 hex) — lineage.
- `_ingested_at` (datetime ISO 8601) — horodatage Silver.
- Autres colonnes : inconnues à ce stade, validation laxe (additionalProperties: true).

Le bump v1 → v2 interviendra quand on aura le mapping métier précis.

---

## 4. Definition of Done

- [ ] `cargo build --workspace` passe.
- [ ] `cargo clippy --workspace -- -D warnings` passe.
- [ ] `cargo test --workspace --all-features` passe.
- [ ] `ComparIASource` enregistrable dans le registry via `LayerRegistry::register`.
- [ ] 3 entités Silver produites avec lignage propagé.
- [ ] Tests : ingest_copper (wiremock), promote_silver (Parquet synthétique), bout en bout.
- [ ] Documentation `///` complète + brief mis à jour avec notes post-implémentation.

---

## 5. Non-objectifs (reportés à v2 ou ultérieur)

- Mapping métier précis (extraction de `co2_eq`, `tokens`, `model_id` typés).
- Téléchargement incrémental (eTag / If-Modified-Since) — ajouté plus tard.
- Téléchargement parallèle des 3 fichiers — gain marginal vs complexité.
- Validation cross-fichier (cohérence conversation_id entre conversations/votes/reactions).

---

## 6. Risques et mitigations

| Risque | Mitigation |
|--------|-----------|
| Polars 0.46 introduit un breaking change vs notre code | Tests d'intégration sur Parquet synthétique en CI |
| Volume 5 GB → CI ralentie | Téléchargement réel testé seulement en CI nocturne, tests unitaires sur fixtures |
| Schéma ComparIA évolue | Schéma Silver volontairement laxe en v1 |
| Memory pressure sur 3.4 GB | LazyFrame + streaming (jamais collect() complet) |
