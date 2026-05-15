# Chantier C26 — Activation du pipeline médaillon (Copper → Silver → Gold)

> **Version cible** : v0.5.0
> **Sprint** : S12 (post-v0.4.0 ship)
> **Approche** : incrémental, 5 sous-chantiers livrables séparément
> **Crates impactées** : `sobria-ingest`, `sobria-referentiel`, `sobria-app`
> **Pré-requis** : ADR-0009 (architecture médaillon), C01 (foundation), C02 (ComparIA), C03 (RTE IRIS)
> **Statut amont** : trait `DataLayer` complet, `LayerRegistry::run_full_pipeline` orchestré, `ComparIASource` + `RteIrisSource` implémentent le trait, `assemble_gold` produit déjà 4 artefacts. **Ce qui manque** : câblage CLI (`main.rs` 100 % stubs), `LayerRegistry::standard()` vide, `dvc.yaml` absent, app pas reconnectée au Gold.

---

## 0. Pourquoi ce chantier maintenant ?

La v0.4.0 vient de shipper le catalogue multi-méthodologie (C24). Le différentiateur produit côté défi data.gouv.fr est désormais l'angle territorial **IRIS** et l'ingestion **officielle** des datasets ComparIA + RTE. Or actuellement aucun de ces deux datasets n'est réellement consommé par l'app : on a du Rust qui compile, mais la commande `cargo run -p sobria-ingest -- pipeline run` ne fait rien d'autre que tracer un message.

C26 active le pipeline déjà construit pour qu'il produise un vrai `referentiel.sqlite` consommable par l'app Tauri. C'est le différentiateur côté défi.

---

## 1. Périmètre (5 sous-chantiers)

### C26.1 — Câblage CLI + registry standard (1-2 jours)

**Sortie** : `cargo run -p sobria-ingest -- pipeline run` produit un Gold réel.

- `LayerRegistry::standard()` instancie `ComparIASource` + `RteIrisSource`.
- `main.rs` câble chaque sous-commande (`pipeline run`, `copper`, `silver`, `gold`, `validate`) à `LayerRegistry`. Plus de stubs.
- `Context::from_env()` lit `SOBRIA_DATA_ROOT` (défaut `./data`).
- Variable `SOBRIA_USE_REAL_DATA=1` (sinon `--limit-rows 10000` par défaut).
- Sous-commande `validate` : recharge `MANIFEST.sha256` + recalcule les hashes Copper → KO si divergence.
- Tests d'intégration `tests/cli_pipeline.rs` avec wiremock + tempdir.

### C26.2 — Schémas Silver versionnés + validation (2 jours)

**Sortie** : chaque écriture Silver est validée par schéma JSON.

- `schemas/silver/comparia_conversations-v1.json`, `comparia_votes-v1.json`, `comparia_reactions-v1.json`, `rte_iris-v1.json`.
- Module `sobria-ingest::silver_validate` (arrow-schema + jsonschema).
- `promote_silver` appelle `silver_validate::check(&entity)` avant retour.
- `proptest` sur les transformations (round-trip Copper → Silver → schema).
- `insta` golden files pour 1 mini-fixture par source.

### C26.3 — Gold complet (jointures + datasheet) (2 jours)

**Sortie** : `data/gold/referentiel.sqlite` + `analytics.parquet` + `datasheet.jsonld` + `MANIFEST.sha256` consommables.

- `assemble_gold` enrichi : jointures ComparIA × RTE IRIS sur la maille géographique (datacenter → région IRIS → mix électrique).
- Vues matérialisées SQLite : `model_overview`, `scenario_inputs`, `time_series_mix`, `comparison_matrix`.
- Index FTS5 sur `model_overview` (recherche M9).
- `datasheet.jsonld` formaté Gebru et al. 2018 + schema.org/Dataset.
- `MANIFEST.sha256` signé GPG (clé fournie en CI via `GPG_PRIVATE_KEY`).
- Tests d'intégration : `tests/gold_pipeline.rs` étendu pour vérifier les tables Gold attendues.

### C26.4 — Orchestration DVC (1 jour)

**Sortie** : `dvc repro` rejoue l'ensemble du pipeline.

- `dvc.yaml` à la racine avec 3 stages (`copper`, `silver`, `gold`) — déps + outs corrects.
- `.dvcignore` pour ignorer `target/`, fixtures de test, etc.
- `data/copper`, `data/silver`, `data/gold` ajoutés à `.gitignore` (DVC les gère).
- `docs/operations/dvc.md` : doc opérateur (clone, `dvc pull`, `dvc repro`).
- CI nocturne : workflow `.github/workflows/dvc-nightly.yml` exécute `dvc repro && dvc push` (avec creds DVC remote en secret).

### C26.5 — Reconnexion app au Gold (1 jour)

**Sortie** : l'app Tauri lit le `referentiel.sqlite` produit par C26.3, plus de référentiel embedded.

- `sobria-referentiel::load()` charge le SQLite depuis `data/gold/referentiel.sqlite` (path configurable via `SOBRIA_REFERENTIEL_PATH`).
- Bootstrap utilisateur : à la première ouverture, l'app télécharge le snapshot Gold via DVC (ou copie depuis bundle si offline).
- IPC `referentiel_status` : retourne version, date snapshot, hash, source.
- Bandeau dans `/parametres` : « Référentiel : v0.5.0-fr (2026-05-XX, SHA-256: ...) ».
- Migration : si l'utilisateur a un référentiel embedded v0.4.0, on le remplace avec confirmation.

---

## 2. Definition of Done v0.5.0

- [ ] `cargo run -p sobria-ingest -- pipeline run` produit `data/gold/referentiel.sqlite` (≥ 10 modèles, ≥ 50 datacenters, ≥ 1000 lignes RTE IRIS).
- [ ] `dvc repro` rejoue à l'identique (hash stable du `referentiel.sqlite`).
- [ ] `cargo test --workspace` passe (incl. proptest + golden files).
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] L'app Tauri démarre avec le nouveau référentiel sans bug visible (smoke test M1, M9, M12, M15, M20).
- [ ] `docs/operations/dvc.md` rédigé + lien dans README.
- [ ] CHANGELOG entrée `[0.5.0]` complète.
- [ ] ADR-0009 mis à jour (statut « Accepted → Implemented »).
- [ ] Tag `v0.5.0` poussé + release notes auto-générées.

---

## 3. Anti-périmètre (différé v0.6+)

- Couche **Platinum** (résultats Monte-Carlo agrégés persistés).
- Materialized views incrémentales.
- Plugin Quarto pour citer les sources Copper.
- Sources Tier 2/3 (ADEME, HuggingFace, CodeCarbon, ML.Energy) — restent stubs.
- Mode multi-utilisateurs / référentiel partagé.

---

## 4. Risques + mitigations

| Risque | Probabilité | Mitigation |
|--------|-------------|------------|
| ComparIA 5 GB long à télécharger en CI | Haute | Fixtures 10k lignes par défaut, vrais datasets en CI nocturne uniquement |
| Schémas Silver divergent de la spec ComparIA réelle | Moyenne | v1 = passthrough enrichi (lineage). Mapping métier en v2. |
| `assemble_gold` jointures FAIL si IRIS↔datacenter pas mappable | Haute | Géolocalisation par IP via GeoLite2 + fallback maille région NUTS-2 |
| DVC remote coûteux (5 GB × N snapshots) | Moyenne | Rétention Copper 30j complet / 2 ans mensuel / annuel ∞ (cf. ADR-0009 §"Politique de rétention") |
| App pas rétro-compatible avec ancien référentiel embedded | Faible | Migration explicite avec dialog de confirmation utilisateur |

---

## 5. Livrables annexes

- ADR-0009 → statut "Implemented", section "Conséquences observées" ajoutée.
- README : section "Pipeline de données" mise à jour avec commandes DVC + exemple sortie.
- Dossier candidature data.gouv.fr : ajout sortie pipeline réelle (capture `pipeline run` + SHA Gold).

---

## 6. Découpage temporel suggéré

| Jour | Sous-chantier | Livrable |
|------|--------------|----------|
| J1 | C26.1 | CLI utilisable, registry câblé |
| J2 | C26.1 fin + tests | `pipeline run` reproductible sur fixtures |
| J3 | C26.2 | Schémas + validation |
| J4 | C26.2 fin + C26.3 début | proptest verts, Gold jointures |
| J5 | C26.3 | datasheet.jsonld + MANIFEST signé |
| J6 | C26.4 + C26.5 début | DVC orchestré, app lit Gold |
| J7 | C26.5 fin + ship | Smoke test, CHANGELOG, tag v0.5.0 |

Total estimé : **5-7 jours** selon densité.
