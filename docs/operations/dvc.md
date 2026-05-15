# Operations — DVC (versionnage et reproductibilité du pipeline médaillon)

> **Public** : opérateurs Sobr.ia, mainteneurs, contributeurs qui veulent
> rejouer le pipeline localement.
> **Référence amont** : [`ADR-0009`](../adr/ADR-0009-medallion-architecture.md)
> + chantier C26 (`briefs/chantiers/C26-pipeline-medaillon-activation.md`).

DVC ([dvc.org](https://dvc.org/)) est notre versionneur de **données**. Il
remplace Git pour les artefacts trop gros pour le repo (snapshots Copper de
plusieurs Go, Parquet Silver de centaines de Mo, `referentiel.sqlite`
final). Trois fichiers définissent l'orchestration :

- [`dvc.yaml`](../../dvc.yaml) — pipeline en 3 stages (`copper`, `silver`,
  `gold`) + un stage `validate`. Chaque stage déclare ses **deps** (sources
  Rust, schémas) et **outs** (fichiers / dossiers produits).
- [`.dvc/config`](../../.dvc/config) — remote par défaut + remotes
  alternatifs (S3, HTTP).
- [`.dvcignore`](../../.dvcignore) — fichiers à ne pas hasher.

---

## Quick start (local, sans clone réseau)

```bash
# 1. Installer DVC (Python ≥ 3.9)
pip install dvc dvc-s3   # dvc-s3 si remote S3 prévu

# 2. Cloner le repo Sobr.ia (sans les données)
git clone https://github.com/<TBD>/sobria.git
cd sobria

# 3. Construire les binaires Rust
cargo build -p sobria-ingest --release

# 4. Récupérer les snapshots versionnés (vide la première fois)
dvc pull

# 5. Rejouer le pipeline (uniquement les stages dont les deps ont changé)
dvc repro

# 6. Pousser les nouveaux artefacts dans le remote
dvc push
```

À la sortie, `data/gold/referentiel.sqlite`, `analytics.parquet`,
`datasheet.jsonld` et `MANIFEST.sha256` (+ `.asc` si GPG actif) sont
disponibles localement et l'app Tauri peut les consommer.

---

## Stages du pipeline

| Stage | Commande | Inputs | Outputs |
|-------|----------|--------|---------|
| `copper` | `cargo run -p sobria-ingest --release -- copper --all` | code `sources/` + `schemas/copper/` | `data/copper/<source>/<YYYY-MM-DD>/` |
| `silver` | `cargo run -p sobria-ingest --release -- silver --all` | `data/copper/` + `schemas/silver/` | `data/silver/<source>/*.parquet` |
| `gold`   | `cargo run -p sobria-ingest --release -- gold` | `data/silver/` + `schemas/gold/` | `data/gold/{referentiel.sqlite, analytics.parquet, datasheet.jsonld, MANIFEST.sha256}` |
| `validate` | `cargo run -p sobria-ingest --release -- validate` | `data/gold/` | (rapport stdout) |

`dvc repro` exécute uniquement les stages dont les inputs ont changé.
`dvc repro --force` force tout. `dvc dag` affiche le graphe.

---

## Politique de rétention Copper

Voir [ADR-0009 §"Politique de rétention Copper"](../adr/ADR-0009-medallion-architecture.md).
Résumé :

| Âge du snapshot | Conservation |
|-----------------|--------------|
| ≤ 30 jours | **Tous** les snapshots |
| 30 j → 2 ans | 1 snapshot **mensuel** (premier jour du mois) |
| > 2 ans | 1 snapshot **annuel** indéfiniment |

Le ménage est automatisé via `dvc gc` planifié hebdomadaire (cf.
`.github/workflows/dvc-nightly.yml`).

---

## Configurer un remote alternatif (S3 / HTTP)

Le remote par défaut est local (`.dvc-cache/` à la racine du repo).
C'est suffisant pour le dev solo. En équipe ou en CI, basculer vers un
remote partagé :

### Remote S3 (production)

```bash
# 1. Configurer le remote
dvc remote modify s3-prod url s3://my-bucket/sobria/dvc

# 2. Authentification : préférer un IAM Role en CI ; en local, env vars
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
# ou bien :
dvc remote modify --local s3-prod access_key_id $AWS_ACCESS_KEY_ID
dvc remote modify --local s3-prod secret_access_key $AWS_SECRET_ACCESS_KEY

# 3. Définir le remote par défaut
dvc remote default s3-prod

# 4. Push
dvc push
```

Coûts à anticiper : ComparIA fait ~ 5 GB par snapshot trimestriel ; RTE
IRIS fait ~ 180 MB par snapshot annuel. Avec la rétention ci-dessus on
plafonne autour de **40 GB** sur 5 ans (∼ 1 €/mois en S3 standard).

### Remote HTTP (read-only public)

```bash
dvc remote add -d http-public https://snapshots.sobr.ia/dvc
```

Pas de push possible (HTTP read-only). Utile pour distribution publique.

---

## FAQ

### Pourquoi DVC plutôt que Git LFS ?

- DVC sépare **code** (Git) et **données** (DVC remote) ; LFS les mélange.
- DVC pipeline (`dvc repro`) gère le DAG des étages, pas LFS.
- DVC supporte **plusieurs remotes** simultanés (local + S3 + HTTP).
- DVC est **language-agnostic** (Rust + Python + Notebook Quarto).

Voir aussi [`ADR-0007`](../adr/ADR-0007-dvc.md).

### Que fait exactement `dvc repro` ?

Pour chaque stage de `dvc.yaml`, DVC compare les hashes des `deps` à ceux
de la dernière exécution (stockés dans `dvc.lock`). Si un dep a changé,
le stage est **ré-exécuté** ; ses outs sont rehashés et `dvc.lock` est
mis à jour. Les stages **avals** dépendant des outs modifiés sont aussi
rejoués (cascade).

### `dvc.lock` doit-il être versionné dans Git ?

**Oui**. Sans `dvc.lock`, DVC ne peut pas garantir la reproductibilité
entre machines. C'est l'équivalent de `Cargo.lock` ou `package-lock.json`.

### Comment ré-ingérer une source spécifique ?

```bash
# Via DVC (force) ─────────────────────────────────────────────────
dvc repro --force-downstream copper

# Via la CLI sans DVC ─────────────────────────────────────────────
cargo run -p sobria-ingest --release -- copper --source comparia
```

### Comment vérifier l'intégrité des snapshots Copper ?

```bash
cargo run -p sobria-ingest --release -- validate
```

Recalcule les SHA-256 de chaque fichier référencé dans
`data/copper/<source>/<date>/manifest.json` et compare au hash enregistré.
Code de sortie ≠ 0 si au moins un manifest est corrompu.

### J'obtiens « `dvc: command not found` »

DVC est livré séparément. Installer avec `pip install dvc` (≥ 3.x) ou
`brew install dvc` (macOS).

### Reproduction bit-à-bit (hash Gold stable)

Le pipeline est **déterministe** sous deux conditions :

1. La variable `SOBRIA_SEED` (défaut `42`) est fixée — déjà honoré par
   `cargo run -p sobria-ingest`.
2. Les snapshots Copper inputs ont un hash stable — garanti par le
   `manifest.json` immuable + `verify_files` à la lecture.

Vérifier :

```bash
sha256sum data/gold/referentiel.sqlite
dvc repro
sha256sum data/gold/referentiel.sqlite
# → mêmes 64 caractères hex.
```

Si divergence, ouvrir une issue avec le diff `MANIFEST.sha256` avant/après.

---

## Voir aussi

- [ADR-0007 — Versionnage des données via DVC](../adr/ADR-0007-dvc.md)
- [ADR-0009 — Architecture médaillon](../adr/ADR-0009-medallion-architecture.md)
- [Brief C26 — Activation du pipeline](../../briefs/chantiers/C26-pipeline-medaillon-activation.md)
- [Documentation DVC officielle](https://dvc.org/doc)
