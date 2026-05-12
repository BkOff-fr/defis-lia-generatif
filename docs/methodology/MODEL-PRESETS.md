# Presets distributionnels par modèle — méthodologie

> **Version** : v1 — calibrage par ordre de grandeur.
> **Crate** : `sobria-estimator`, module `model_presets`.
> **Statut** : *indicatif* / *extrapolé* selon modèle. Validation Luccioni 2023
> + EcoLogits 2024 à venir en chantier C07.

---

## 1. Pourquoi ces chiffres et pas d'autres

Le moteur Monte-Carlo a besoin de **distributions** pour chacun de ces
paramètres :

- `ε_prefill_mj_per_token` — énergie pour le prefill (lecture du prompt).
- `ε_decode_mj_per_token` — énergie pour le decode (génération).
- `embodied_g_per_request` — embodied carbon hardware amorti par requête.

Pour les fournir, on dispose de trois types de sources :

| Source | Couverture | Fiabilité |
|--------|-----------|-----------|
| HF AI Energy Score | Modèles ouverts mesurés sur hardware standardisé | Élevée |
| EcoLogits (Data for Good) | Catalogue modèles fermés + ouverts, méthodologie ISO 14044 | Élevée pour ouverts, indicative pour fermés |
| Papers (Luccioni 2023, Patterson 2021) | Quelques modèles spécifiques | Très élevée mais ponctuelle |

Pour la **v1 des presets**, on adopte une **extrapolation linéaire** depuis
les modèles ouverts mesurés, en assumant la transparence sur le caractère
indicatif de la démarche.

## 2. Formules d'extrapolation

```
ε_decode   ≈ k_decode × N_b                        avec k_decode = 0.025 mJ/token/B
ε_prefill  ≈ ε_decode × 0.4                        (le prefill est plus efficient)
embodied   ≈ k_embodied × N_b g/req                avec k_embodied = 0.00025
```

où `N_b` = nombre de paramètres en milliards.

**Largeur d'incertitude** :
- `P5 = P50 / 1.65`
- `P95 = P50 × 1.65`

Soit un ratio `P95/P5 ≈ 2.7`, correspondant à `σ_log ≈ 0.30`. Cette valeur
est cohérente avec la dispersion observée sur HF AI Energy Score 2026 pour
des modèles de même famille.

### Justification des coefficients

| Coefficient | Valeur | Justification |
|-------------|--------|---------------|
| `k_decode` | 0.025 mJ/token/B | Llama 3.1 70B mesuré ≈ 1.75 mJ/token → 1.75 / 70 = 0.025 |
| Ratio prefill/decode | 0.4 | Le prefill est batché sur GPU à haut throughput, le decode est autoregressif |
| `k_embodied` | 0.00025 g/req/B | Gupta 2022 amorti sur 10⁹ requêtes/an |
| `σ_log` | 0.30 | CV observé sur mesures HF/EcoLogits |

## 3. Statuts de calibration

Trois niveaux possibles :

- **`Validated`** — comparé à une mesure publiée d'une étude à ±15 %. **0 modèle
  à ce statut en v1.** Le chantier C07 calibrera Llama 3.1 et Mistral Large
  contre les papers.
- **`Indicative`** — extrapolé pour modèle ouvert depuis HF AI Energy Score
  par ordre de grandeur.
- **`Extrapolated`** — modèle fermé (GPT, Claude, Gemini) : la taille `N_b`
  est elle-même une estimation publique (non confirmée par le provider).

## 4. Table v1 — 8 modèles

| ID | Provider | Famille | N_b (G) | Ouverture | Statut | ε_decode P50 (mJ/tok) | Embodied P50 (g/req) |
|----|----------|---------|---------|-----------|--------|-----------------------|----------------------|
| `gpt-4o` | OpenAI | gpt-4 | 200 | Closed | Extrapolated | 5.0 | 0.050 |
| `gpt-4o-mini` | OpenAI | gpt-4 | 8 | Closed | Extrapolated | 0.20 | 0.0020 |
| `claude-3-5-sonnet` | Anthropic | claude-3 | 200 | Closed | Extrapolated | 5.0 | 0.050 |
| `mistral-large-2` | Mistral AI | mistral-large | 123 | OpenWeights | Indicative | 3.075 | 0.03075 |
| `mistral-medium-3` | Mistral AI | mistral-medium | 30 | OpenWeights | Indicative | 0.75 | 0.0075 |
| `llama-3-1-70b` | Meta | llama-3 | 70 | OpenWeights | Indicative | 1.75 | 0.0175 |
| `llama-3-1-8b` | Meta | llama-3 | 8 | OpenWeights | Indicative | 0.20 | 0.0020 |
| `gemini-2-0-flash` | Google | gemini-2 | 32 | Closed | Extrapolated | 0.80 | 0.0080 |

(Les triplets complets `(P5, P50, P95)` sont dans le code Rust, `MODEL_REGISTRY`.)

## 5. Limites assumées

1. **Taille des modèles fermés** : les valeurs `N_b` proviennent d'analyses
   publiques (papers de chercheurs externes, fuites, benchmarks de coût) et
   peuvent être fausses d'un facteur 2-3.

2. **Extrapolation linéaire** : suppose que l'efficience énergétique par
   token est constante avec la taille. C'est faux à grande échelle (les modèles
   très gros bénéficient d'économies d'échelle batching) mais raisonnable
   pour des comparaisons inter-modèles à granularité utilisateur.

3. **Quantization, MoE, distillation** : non modélisés. Un modèle quantizé
   8-bit consomme moins, un MoE active seulement une fraction des paramètres.
   Les chiffres ici représentent un usage **standard sans optimisation**.

4. **PUE et mix électrique** : laissés en paramètre du `EstimationParams`
   (par défaut PUE Uniform [1.1, 1.4], mix France 56 gCO₂eq/kWh). À surcharger
   selon le datacenter réel.

## 6. Évolution prévue

| Chantier | Apport |
|----------|--------|
| C07 | Validation Luccioni 2023 + EcoLogits 2024 → certains statuts passent à `Validated` |
| C08 | Calibrage spécifique par datacenter (PUE/WUE/IF par provider) |
| C09 | Ajout des modèles multimodaux (vision, audio) avec leurs propres distributions |
| C10 | Auto-import depuis HF AI Energy Score (script Python S0 → Rust constant) |

## 7. Comment ajouter ou modifier un preset

1. Modifier la doc ici (table §4 + rationale).
2. Ajouter / modifier la ligne dans `MODEL_REGISTRY` (`model_presets.rs`).
3. Ajouter au moins une source dans le champ `sources`.
4. Si validation paper-based : passer `calibration` à `Validated` et ajouter
   un test de reproduction dans `validation::` (chantier C07).
5. CHANGELOG.md.

## 8. FAQ

**Pourquoi pas de coefficient par token type (input vs output) sur le
prefill ?** Le prefill traite l'ensemble du contexte en une seule passe
batchée, le coût marginal par token est très faible. Le decode est
autoregressif, chaque token nécessite une passe complète. Le ratio 0.4
absorbe cette asymétrie.

**Pourquoi pas de coefficient par contexte (8k vs 128k) ?** Le coût du
prefill augmente quadratiquement avec la longueur du contexte sur les
architectures classiques, mais les implémentations modernes (Flash Attention,
KV cache) ramènent cela à du linéaire. À calibrer en v2.

**Pourquoi PUE et IF séparés du modèle ?** Le PUE dépend du datacenter, pas
du modèle. Idem pour le mix électrique : un GPT-4o tourné dans un datacenter
Azure FR n'a pas le même IF qu'en Virginie. Ces deux paramètres restent dans
`EstimationParams` indépendamment du preset modèle.
