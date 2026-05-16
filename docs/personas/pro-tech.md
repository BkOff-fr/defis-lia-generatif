# Sobr.ia pour Professionnel·le tech (dev, ML eng, intégrateur)

> **Estimer, comparer, journaliser pour vos intégrations API — avec
> audit chaîné SHA-256 et exports JSON-LD PROV-O.**

---

## Qui c'est ?

Vous êtes dev backend, ML engineer, intégrateur·rice. Vous appelez
les APIs OpenAI / Anthropic / Mistral / Google / Meta dans une app
en prod ou en POC. Vous voulez **mesurer l'empreinte par modèle**,
**comparer les choix techniques** et **garder une trace
auditable** pour le reporting interne.

Vous connaissez probablement EcoLogits, BoaVizta, ou AI Energy
Score. Sobr.ia parle votre langue (Monte-Carlo, P5/P95, JSON-LD,
PROV-O) mais ajoute deux choses qu'aucun ne fait : **multi-méthodo
en parallèle** (vous pouvez croiser AFNOR + EcoLogits sur le même
prompt) et **agrégation des vendor disclosure** (Mistral × ADEME,
Google Gemini, Meta Llama 3.x).

## Ce que Sobr.ia résout pour vous

| Question | Réponse Sobr.ia |
|---|---|
| « Quel modèle minimise l'empreinte pour mon use case ? » | **Comparer modèles** côte-à-côte sur le même prompt, 3 indicateurs (CO₂, énergie, eau) |
| « Comment justifier mon choix de Mistral vs GPT-4o ? » | Bibliothèque modèles + **encadrés vendor disclosure** sourcés (Mistral × ADEME, Google) |
| « Comment logger toutes mes estimations pour reporting trimestriel ? » | **Journal d'audit** ledger SHA-256 chaîné + export JSON-LD PROV-O |

## Top 3 use cases

1. **Comparer 3 candidats LLM pour une intégration** — un prompt
   représentatif, 3 modèles, lecture P5/P50/P95 sur les 3
   indicateurs.
2. **Logger toutes les estimations en lot** — soit via l'extension
   navigateur (Chrome/Firefox) qui remonte automatiquement vers
   l'app, soit en batch CSV pour analyse a posteriori.
3. **Croiser méthodologies** — un même prompt évalué par AFNOR SPEC
   2314 + EcoLogits 2026-01 → écart visible, transparent pour les
   reviewers.

## Modules pertinents

- **Estimer un prompt** — atelier de mesure unitaire
- **Comparer modèles** — benchmark N modèles
- **Journal d'audit** — ledger chaîné SHA-256
- **Comment ça marche** — méthodologie expliquée
- **Bibliothèque de modèles** — catalogue + encadrés vendor (Mistral,
  Google, Meta)
- **Simulateur « Et si...? »** — explorer leviers d'optimisation

## Quickstart 5 minutes

```bash
# Option 1 — App desktop Tauri
./sobria-app

# Option 2 — Extension navigateur (capture en vie réelle)
# Téléchargez sobria-extension-chrome-v0.6.0.zip depuis Releases
# Load unpacked dans chrome://extensions/

# Option 3 — En attendant la CLI (v1.1), utilisez le crate Rust
#           directement dans vos tests d'intégration
cargo add sobria-estimator        # quand publié sur crates.io
```

**Pairing extension ↔ app** : générez un code 6 chiffres dans
`/parametres`, collez-le dans l'extension, vos estimations sont
remontées automatiquement dans le Journal et le Tableau de bord.

## Pour aller plus loin

- [ADR-0012 — Multi-méthodologie](../adr/ADR-0012-multi-methodology-engine.md)
- [ADR-0013 — Extension + pairing + Mode Équipe](../adr/ADR-0013-extension-pairing-team-mode.md)
- [Catalogue sources](../sources/CATALOGUE-SOURCES.md)
- [Reproductibilité scientifique — notebook Quarto](../../notebook/validation.qmd)
