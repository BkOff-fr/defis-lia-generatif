# Sobr.ia pour Chercheur·se / Journaliste

> **Reproductibilité, comparaisons inter-modèles, datasets
> publiables, Datasheet Gebru, multi-méthodologie, DOI citable.**

---

## Qui c'est ?

Vous êtes chercheur·se (LIG, LIP6, Inria, ADEME, Carbone 4…),
data-journaliste, ou expert·e indépendant·e travaillant sur
l'empreinte environnementale de l'IA. Vous publiez des papiers,
des enquêtes, ou des rapports qui doivent être **reproductibles**,
**sourcés**, **comparables** entre modèles et entre méthodologies.

Sobr.ia est conçu pour vous : c'est probablement votre persona le
mieux servi du produit, et c'est volontaire (le pitch
data.gouv.fr l'exige).

## Ce que Sobr.ia résout pour vous

| Question | Réponse Sobr.ia |
|---|---|
| « Comment reproduire les chiffres Sobr.ia pour mon papier ? » | **Notebook Quarto** (`notebook/validation.qmd`) + seed déterministe `SOBRIA_SEED=42` |
| « Comment comparer N modèles selon 2 méthodologies ? » | **Comparer modèles** + catalogue multi-méthodos (AFNOR + EcoLogits, plus à venir) |
| « Comment citer Sobr.ia avec un DOI ? » | DOI Zenodo publié avec v0.8.0 — voir section Citation du README |
| « Comment exporter mon dataset d'estimations pour analyse externe ? » | **Rapport réglementaire** + JSON-LD PROV-O |

## Top 3 use cases

1. **Reproduire un chiffre publié** — relancer le notebook Quarto
   `notebook/validation.qmd` avec le seed 42, vérifier l'écart ≤ 1 %
   vs port Rust.
2. **Comparer 5 modèles selon 2 méthodologies** — Comparer modèles
   + activer EcoLogits en méthode "Voir aussi" → 2 colonnes par
   modèle, écart visible.
3. **Publier un dataset d'estimations** — Datasheet scientifique
   (Gebru 2018) auto-générée + JSON-LD PROV-O + ledger SHA-256 →
   datasheet conforme NeurIPS, ICML, FAccT.

## Modules pertinents

- **Estimer un prompt** — atelier reproductible avec seed
- **Comparer modèles** — benchmark multi-méthodo
- **Journal d'audit** — ledger chaîné SHA-256 (preuve de
  non-altération pour reviewers)
- **Comment ça marche** — méthodologie expliquée + sources DOI
- **Bibliothèque de modèles** — catalogue P5/P50/P95 + sources
  vendor (Mistral × ADEME, Google, Meta)
- **Datasheet scientifique** — format Gebru 2018 auto-généré

## Quickstart 5 minutes

```bash
# 1. Cloner le repo (la reproductibilité scientifique exige le code)
git clone https://github.com/BkOff-fr/defis-lia-generatif.git
cd defis-lia-generatif

# 2. Lancer la validation croisée (port Rust vs Python EcoLogits)
cargo test -p sobria-estimator validation
quarto render notebook/validation.qmd

# 3. Lancer l'app pour explorer la Bibliothèque
cargo tauri dev
# → choisir persona "Chercheur·se / Journaliste"

# 4. Pour citer dans un papier — DOI Zenodo dans README §Citation
```

**Seed déterministe** : toutes les estimations Monte-Carlo sont
reproductibles à la nanoseconde via la variable
d'environnement `SOBRIA_SEED` (défaut 42).

## Pour aller plus loin

- [Notebook Quarto de validation croisée](../../notebook/validation.qmd)
- [ADR-0012 — Catalogue multi-méthodologie](../adr/ADR-0012-multi-methodology-engine.md)
- [Synthèse AFNOR SPEC 2314](../methodology/AFNOR-SPEC-2314-synthese.md)
- [Catalogue sources](../sources/CATALOGUE-SOURCES.md)
- [Audit datasets Q3 2026](../sources/AUDIT-2026-Q3.md)
- Citation : voir README §Citation (DOI Zenodo publié avec v0.8.0)
