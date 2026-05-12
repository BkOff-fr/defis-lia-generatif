# Chantier #6 — Calibrage distributionnel par modèle

> **Pré-requis** : v0.1.0-foundation taggé.
> **Crate touchée** : `sobria-estimator`.
> **Durée cible** : 1 jour.

---

## 0. Objectif

Permettre à l'app et au notebook d'écrire :

```rust
let params = EstimationParams::for_model("gpt-4o-mini")?;
let result = MonteCarloEngine::default().estimate(&request, &params)?;
```

… sans avoir à connaître soi-même les valeurs distributionnelles de
ε_prefill, ε_decode, embodied par modèle. Le registry interne sait.

## 1. Garde-fou méthodologique (lire avant tout)

**Les chiffres fournis dans ce chantier sont *indicatifs*** :

- Calibrés par ordre de grandeur depuis HF AI Energy Score, EcoLogits,
  et les papers Luccioni 2023 / Patterson 2021 pour les modèles publiés.
- **Pas encore validés contre les 3 études de référence à ±15 %** —
  ce sera l'objet du chantier #7 (validation croisée).
- Documentés explicitement dans `docs/methodology/MODEL-PRESETS.md`
  avec colonne "statut de calibration" : `indicatif` | `validé` | `extrapolé`.

Pour les **modèles fermés** (GPT, Claude, Gemini), les chiffres sont
nécessairement **extrapolés** depuis des modèles ouverts comparables —
on l'assume publiquement.

## 2. Architecture

```
crates/sobria-estimator/src/
├── model_presets.rs       ← registry statique + API for_model()
└── lib.rs                 ← + pub use model_presets::*
```

Structure du preset :

```rust
pub struct ModelPreset {
    pub id: &'static str,            // "gpt-4o-mini"
    pub display_name: &'static str,  // "GPT-4o mini"
    pub provider: &'static str,      // "OpenAI"
    pub family: &'static str,        // "gpt-4"
    pub approx_params_billions: f64, // 8.0 (estimation publique)
    pub openness: Openness,          // Open | OpenWeights | Closed
    pub epsilon_prefill_mj: (f64, f64, f64), // (p5, p50, p95)
    pub epsilon_decode_mj:  (f64, f64, f64),
    pub embodied_g_per_req: (f64, f64, f64),
    pub calibration: CalibrationStatus,
    pub sources: &'static [&'static str],
}
```

## 3. Modèles couverts en v1

8 modèles populaires, couvrant les principaux providers :

| ID | Provider | Famille | Ouverture | Calibration v1 |
|----|----------|---------|-----------|----------------|
| `gpt-4o` | OpenAI | gpt-4 | Closed | extrapolé |
| `gpt-4o-mini` | OpenAI | gpt-4 | Closed | extrapolé |
| `claude-3-5-sonnet` | Anthropic | claude-3 | Closed | extrapolé |
| `mistral-large-2` | Mistral | mistral-large | OpenWeights | indicatif |
| `mistral-medium-3` | Mistral | mistral-medium | OpenWeights | indicatif |
| `llama-3-1-70b` | Meta | llama-3 | OpenWeights | indicatif |
| `llama-3-1-8b` | Meta | llama-3 | OpenWeights | indicatif |
| `gemini-2-0-flash` | Google | gemini-2 | Closed | extrapolé |

## 4. Méthodologie d'extrapolation

Pour un modèle fermé de taille `N_b` milliards de paramètres :

- **ε_decode ≈ N_b × k_decode** avec `k_decode ≈ 0.025 mJ/token/B`
  (extrapolation linéaire des mesures HF/EcoLogits sur modèles ouverts).
- **ε_prefill ≈ ε_decode × 0.4** (le prefill est typiquement moins coûteux,
  car batché sur GPU avec un haut throughput).
- **embodied ≈ 0.00025 × N_b g/req** (Gupta 2022 amorti sur 10⁹ req).

Largeur d'incertitude (CV = σ/μ ≈ 0.3 sur la log) :

- p5 = p50 / 1.65
- p95 = p50 × 1.65

(soit ratio P95/P5 ≈ 2.7, ce qui correspond à un σ_log ≈ 0.30)

Ces formules sont **codées comme helpers privés** dans `model_presets.rs`,
ce qui permet de générer le registry de manière transparente et auditable.

## 5. API publique

```rust
impl EstimationParams {
    /// Construit des paramètres calibrés depuis le registry interne.
    pub fn for_model(model_id: &str) -> EstimatorResult<Self>;
}

/// Liste tous les modèles connus.
pub fn available_models() -> Vec<&'static ModelPreset>;

/// Trouve un preset par son ID exact.
pub fn find_preset(model_id: &str) -> Option<&'static ModelPreset>;
```

## 6. Tests requis

- `for_model("gpt-4o-mini")` retourne un `EstimationParams` valide.
- Tous les modèles du registry produisent un `EstimationParams` valide.
- `for_model("modele-inconnu")` retourne `Err`.
- Property test : pour chaque modèle, une estimation typique
  (100/500 tokens) produit P50 entre 0.1 et 100 g CO₂eq (sanity).
- L'ordre P5 ≤ P50 ≤ P95 reste vrai pour toutes les distributions générées.
- `available_models()` retourne au moins 8 entrées.

## 7. Non-objectifs (v2)

- Calibration validée contre Luccioni / Patterson (→ chantier C07).
- Variation par datacenter (PUE/WUE/IF par provider).
- Modèles multimodaux (vision, audio).
- Catalogue auto-généré depuis HF Hub.

## 8. Definition of Done

- [ ] `cargo build` + `cargo test -p sobria-estimator` verts.
- [ ] `cargo clippy -- -D warnings` passe.
- [ ] ≥ 8 modèles dans le registry.
- [ ] `docs/methodology/MODEL-PRESETS.md` documente chaque chiffre.
- [ ] Tests de sanity (ordre de grandeur) passent.
