# sobria-core

Crate fondation du projet Sobr.ia : types, traits, erreurs.

## Rôle

- Types métier partagés (`Model`, `Datacenter`, `EmissionFactor`, `Estimation`, etc.)
- Traits transverses
- Erreurs publiques (`thiserror`)
- Pas de logique métier — uniquement des contrats.

## Règles

- Aucune dépendance lourde (pas de tokio, pas de reqwest, pas de duckdb).
- Tous les types publics sont `Serialize + Deserialize + JsonSchema`.
- Documenter chaque type avec un exemple `///` runnable.

Voir [`CLAUDE.md`](../../CLAUDE.md) et [ADR-0001](../../docs/adr/ADR-0001-rust-tauri.md).
