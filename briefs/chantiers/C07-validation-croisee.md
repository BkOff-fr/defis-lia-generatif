# Chantier #7 — Validation croisée du moteur

> **Pré-requis** : C05 (engine) + C06 (presets) mergés.
> **Crate touchée** : `sobria-estimator`.
> **Durée cible** : 1 jour pour le framework + cas plausibilité.
> **Compléments S0 biblio** : ajouter les cas `Reproduction` après lecture
> des annexes de Luccioni 2023, Patterson 2021, EcoLogits 2024.

---

## 0. Pourquoi un framework en 2 niveaux

Annoncer « notre moteur reproduit Luccioni 2023 à ±15% » est un engagement
fort qui demande la lecture du paper et la connaissance exacte du setup
(hardware, mix électrique, prompt). Tant qu'on n'a pas ces chiffres
sous la main, on ne peut pas valider strictement.

**Mais** on peut déjà valider qu'on est dans le bon ordre de grandeur :
si notre moteur dit « 5 g CO₂eq pour un prompt GPT-4o » alors qu'EcoLogits
dit « 2 g », on est dans la même décade — c'est déjà précieux.

D'où le **framework à deux niveaux** :

| Niveau | Quoi | Quand |
|--------|------|-------|
| `PlausibilityCase` | Vérifie que le P50 tombe dans une plage large (ordre de grandeur, facteur 10-100) | Toujours actif en CI |
| `ReproductionCase` | Vérifie que le P50 reproduit un chiffre publié à ±15% | Activé cas par cas, après lecture des sources |

## 1. Architecture

```
crates/sobria-estimator/src/validation/
├── mod.rs        ← types ValidationCase, ValidationReport, runner
└── cases.rs      ← cas concrets (PlausibilityCase v1 + ReproductionCase à venir)
```

## 2. Types

```rust
pub struct PlausibilityCase {
    pub id: &'static str,
    pub description: &'static str,
    pub model_id: &'static str,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub if_electrical_g_per_kwh: f64,
    /// Plage attendue (min, max) en gCO₂eq pour le P50.
    pub expected_range_g_co2eq: (f64, f64),
    pub reference: &'static str,
}

pub struct ReproductionCase {
    pub id: &'static str,
    pub source_doi_or_url: &'static str,
    pub model_id: &'static str,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub if_electrical_g_per_kwh: f64,
    pub pue: f64,
    pub expected_p50_g_co2eq: f64,
    pub tolerance: f64,    // 0.15 = ±15%
    pub notes: &'static str,
}

pub struct ValidationReport {
    pub case_id: &'static str,
    pub kind: ValidationKind,
    pub passed: bool,
    pub computed_p50: f64,
    pub expected: Expectation,
    pub message: String,
}
```

## 3. Cas v1 (plausibilité uniquement)

| ID | Modèle | Mix | Tokens | Plage attendue (g CO₂eq) | Justification |
|----|--------|-----|--------|---------------------------|---------------|
| `gpt-4o-mini-fr-short` | gpt-4o-mini | FR (56 g/kWh) | 100/500 | [0.000_01, 0.01] | Petit modèle, mix décarboné, prompt court |
| `llama-70b-fr-medium` | llama-3-1-70b | FR (56 g/kWh) | 200/1000 | [0.000_1, 0.1] | Modèle moyen, mix décarboné |
| `llama-70b-us-medium` | llama-3-1-70b | US-VA (412 g/kWh) | 200/1000 | [0.001, 1.0] | Même modèle, mix carboné — doit être ~7x supérieur à FR |
| `gpt-4o-fr-long` | gpt-4o | FR (56 g/kWh) | 500/2000 | [0.001, 0.1] | Grand modèle, prompt long |
| `mistral-large-fr-short` | mistral-large-2 | FR (56 g/kWh) | 100/500 | [0.000_1, 0.01] | Modèle moyen, prompt court |

Ces plages sont **larges (facteur 10²-10³)**. Elles ne valident pas la
précision du moteur, seulement qu'il ne produit pas de résultats absurdes
(ex: nanogrammes ou tonnes).

## 4. Cas v2 (reproduction stricte, à ajouter après S0 biblio)

À renseigner après lecture des références. Cibles initiales :

| ID | Source | Modèle | Statut |
|----|--------|--------|--------|
| `luccioni-2023-bloom-inference` | Luccioni 2023 §4.2 | BLOOM 176B | À calibrer en S0 |
| `patterson-2021-meena` | Patterson 2021 Table 4 | Meena 2.6B | À calibrer en S0 |
| `ecologits-2024-gpt-4-standard` | EcoLogits docs/cas type | GPT-4o | À calibrer en S0 |

Pour chaque cas, Thibault renseigne :
- le DOI / URL de la source,
- le setup exact (hardware, mix électrique, prompt utilisé),
- le chiffre cible publié,
- la tolérance.

Le test devient alors actif en CI.

## 5. Processus de promotion d'un modèle en `Validated`

Un modèle passe de `Indicative` ou `Extrapolated` à `Validated` quand :

1. **Au moins un `ReproductionCase`** existe pour ce modèle.
2. Le test passe à ±15% en CI.
3. La référence (paper, page, équation) est citée explicitement dans
   `docs/methodology/VALIDATION-CROISEE.md`.

## 6. Definition of Done — chantier #7

- [ ] Module `validation` opérationnel.
- [ ] ≥ 5 `PlausibilityCase` qui passent.
- [ ] `cargo test -p sobria-estimator --all-features` passe.
- [ ] `cargo clippy -- -D warnings` passe.
- [ ] `docs/methodology/VALIDATION-CROISEE.md` documente la méthodologie.
- [ ] Framework prêt à accueillir des `ReproductionCase` (juste à instancier).

## 7. Non-objectifs (autres chantiers)

- Lecture effective et calibration de Luccioni 2023 (→ S0 biblio).
- Tests visuels comparant la distribution complète vs publiée
  (→ chantier futur si pertinent).
- Validation multimodale (→ v2.0).
