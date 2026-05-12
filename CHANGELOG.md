# Changelog Sobr.ia

Toutes les modifications notables sont documentées ici, conformément à [Keep a Changelog 1.1.0](https://keepachangelog.com/fr/1.1.0/) et [SemVer](https://semver.org/).

Format : `[X.Y.Z] - YYYY-MM-DD`
Types : `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

## [Unreleased]

### Added
- Pack de cadrage initial : CDC v1.2, 9 ADR, roadmap 12 semaines, brief S0, catalogue sources, maquette UI textuelle.
- Architecture médaillon Copper / Silver / Gold (ADR-0009) implémentée comme trait Rust unique.
- Module M12 — Territoire français (cartographie IRIS, croisement ComparIA × RTE IRIS).
- Bootstrap technique : workspace Cargo, CI GitHub Actions, DVC pipeline, scripts/bootstrap.sh.

### Changed
- Pivot stratégique sur les datasets officiels du défi data.gouv.fr (ComparIA + RTE IRIS).
- 0 clé API bloquante en v1.0 (RTE eco2mix reste optionnel pour le live FR).

### Removed
- Sources Electricity Maps et MaxMind GeoLite2 (paywalls / licences virales).

---

## [0.1.0] - À venir

Première release publique : cadrage + S0 (revue biblio) terminés.
