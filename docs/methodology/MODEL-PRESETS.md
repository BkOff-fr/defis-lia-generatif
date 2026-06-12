# Presets distributionnels par modèle — méthodologie

> **Version** : v2 — coefficient `k_decode` recalibré ×1000 (chantier C24,
> mai 2026) ; registre étendu au catalogue 2026 (chantier C34, v0.9.0).
> **Mis à jour** : 2026-06-12.
> **Crate** : `sobria-estimator`, module `model_presets`.
> **Statut** : *indicatif* / *extrapolé* selon modèle (aucun preset
> `Validated` à ce jour). La validation croisée est portée par le catalogue
> multi-méthodologies (ADR-0012) : port direct EcoLogits reproduit à ≤ 1 %,
> engine AFNOR borné par des `PlausibilityCase` — voir
> [`VALIDATION-CROISEE.md`](VALIDATION-CROISEE.md).

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

Pour ces presets, on adopte une **extrapolation linéaire** depuis
les modèles ouverts mesurés, en assumant la transparence sur le caractère
indicatif de la démarche.

## 2. Formules d'extrapolation

```
ε_decode   ≈ k_decode × N_b                        avec k_decode = 25.0 mJ/token/B
ε_prefill  ≈ ε_decode × 0.4                        (le prefill est plus efficient)
embodied   ≈ k_embodied × N_b g/req                avec k_embodied = 0.00025
```

où `N_b` = nombre de paramètres **actifs** en milliards (depuis C34.2, les
modèles MoE comptent leurs seuls paramètres actifs ; pour les modèles
denses, actifs = totaux).

### Recalibration C24 (mai 2026) — facteur ×1000 sur `k_decode`

La v1 de ce document publiait `k_decode = 0.025 mJ/token/B`, issu d'une
erreur d'unité : la mesure de référence Llama 3.1 70B en decode est
≈ 1.75 **J**/token (HF AI Energy Score, ML.ENERGY), et non 1.75 mJ/token.
Le coefficient v1 **sous-évaluait donc l'énergie d'un facteur ~1000**.

Le chantier C24 (audit B, mai 2026) a recalibré la constante
`K_DECODE_MJ_PER_TOKEN_PER_B` à **25.0 mJ/token/B** pour l'aligner sur les
mesures HF AI Energy Score et ML.ENERGY : 1750 mJ/token ÷ 70 B =
25.0 mJ/token/B. Cf. le commentaire sourcé sur la constante dans
`model_presets.rs`, l'[ADR-0012](../adr/ADR-0012-multi-methodology-engine.md)
et `briefs/chantiers/C24-multi-methodologie-ecologits.md`.

**Largeur d'incertitude** :
- `P5 = P50 / 1.65`
- `P95 = P50 × 1.65`

Soit un ratio `P95/P5 ≈ 2.7`, correspondant à `σ_log ≈ 0.30`. Cette valeur
est cohérente avec la dispersion observée sur HF AI Energy Score 2026 pour
des modèles de même famille.

### Justification des coefficients

| Coefficient | Valeur | Justification |
|-------------|--------|---------------|
| `k_decode` | 25.0 mJ/token/B | Llama 3.1 70B mesuré ≈ 1.75 J/token decode (HF AI Energy Score, ML.ENERGY) → 1750 / 70 = 25.0. Recalibré ×1000 en C24 (erreur d'unité de la v1, cf. §2) |
| Ratio prefill/decode | 0.4 | Le prefill est batché sur GPU à haut throughput, le decode est autoregressif |
| `k_embodied` | 0.00025 g/req/B | Gupta 2022 amorti sur 10⁹ requêtes/an |
| `σ_log` | 0.30 | CV observé sur mesures HF/EcoLogits |

## 3. Statuts de calibration

Trois niveaux possibles :

- **`Validated`** — comparé à une mesure publiée d'une étude à ±15 %.
  **0 preset à ce statut à ce jour** (registre v0.9.0 : 20 `Extrapolated`,
  14 `Indicative`). L'audit B (mai 2026) a montré qu'aucune calibration
  unique d'une formule linéaire-par-token ne reproduit EcoLogits à ±15 %
  sur toute la gamme de modèles ; la validation croisée passe donc par le
  catalogue multi-méthodologies (C24, ADR-0012) : port direct EcoLogits
  reproduit à ≤ 1 %, engine AFNOR borné par des `PlausibilityCase`.
- **`Indicative`** — extrapolé pour modèle ouvert depuis HF AI Energy Score
  par ordre de grandeur.
- **`Extrapolated`** — modèle fermé (GPT, Claude, Gemini) : la taille `N_b`
  est elle-même une estimation publique (non confirmée par le provider).

## 4. Registre des modèles — le code est la source de vérité

**La source de vérité est le code** : le registre `MODEL_REGISTRY` dans
[`crates/sobria-estimator/src/model_presets.rs`](../../crates/sobria-estimator/src/model_presets.rs).
La table de 8 modèles publiée dans la v1 de ce document a divergé du code
dès le catalogue 2026 (C34) ; elle n'est plus dupliquée ici.

État du registre (v0.9.0, vérifié 2026-06-12) :

- **34 presets** au total ;
- **26 actifs** — catalogue 2026 (C34) : Claude 4.x (Anthropic),
  GPT-5.5 / o3 (OpenAI), Gemini 3.x / 2.5 Pro (Google), Llama 4 / 3.3
  (Meta), Mistral Large 3 / Medium 3.5 / Small 4 (Mistral AI),
  DeepSeek V4 / R1, Grok 4 (xAI), Qwen 3.6 (Alibaba), Phi-4 reasoning
  (Microsoft) ;
- **8 dépréciés** (`deprecated: true`) — exactement les 8 modèles de
  l'ancienne table v1 de ce document (GPT-4o, GPT-4o-mini, Claude 3.5
  Sonnet, Mistral Large 2, Mistral Medium 3, Llama 3.1 70B/8B,
  Gemini 2.0 Flash), conservés uniquement pour la reproductibilité de
  l'audit ledger et exclus de la liste UI par défaut ;
- statuts de calibration : 20 `Extrapolated`, 14 `Indicative`,
  0 `Validated` (cf. §3).

Exemple de lecture avec le coefficient recalibré (cf. §2) : Llama 3.1 70B →
ε_decode P50 = 25.0 × 70 = **1750 mJ/token** (1.75 J/token) ; embodied
P50 = 0.00025 × 70 = 0.0175 g/req.

Les triplets complets `(P5, P50, P95)`, ainsi que les champs de modalités
C34.2 (`vision_pricing`, `audio_capable`, `reasoning_capable`,
`thinking_token_multiplier`, `default_context_overhead_tokens`), sont dans
le code Rust, `MODEL_REGISTRY`.

## 5. Limites assumées

1. **Taille des modèles fermés** : les valeurs `N_b` proviennent d'analyses
   publiques (papers de chercheurs externes, fuites, benchmarks de coût) et
   peuvent être fausses d'un facteur 2-3.

2. **Extrapolation linéaire** : suppose que l'efficience énergétique par
   token est constante avec la taille. C'est faux à grande échelle (les modèles
   très gros bénéficient d'économies d'échelle batching) mais raisonnable
   pour des comparaisons inter-modèles à granularité utilisateur.

3. **Quantization, distillation** : non modélisées. Un modèle quantizé
   8-bit consomme moins. Les chiffres représentent un usage **standard
   sans optimisation**. Les **MoE sont modélisés depuis C34.2** via le
   champ `active_params_b` : seuls les paramètres actifs entrent dans
   l'extrapolation (cf. §2).

4. **PUE et mix électrique** : laissés en paramètre du `EstimationParams`
   (par défaut PUE Uniform [1.1, 1.4], mix France 56 gCO₂eq/kWh). À surcharger
   selon le datacenter réel.

## 6. Évolution

| Chantier | Apport | Statut (2026-06-12) |
|----------|--------|---------------------|
| C07 | Validation Luccioni 2023 + EcoLogits 2024 → statuts `Validated` | Remplacé par C24 : validation croisée via catalogue multi-méthodologies (ADR-0012), port direct EcoLogits ≤ 1 % |
| C08 | Calibrage spécifique par datacenter (PUE/WUE/IF par provider) | À venir |
| C09 | Ajout des modèles multimodaux (vision, audio) avec leurs propres distributions | Réalisé en C34.2 (v0.9.0) : `vision_pricing`, `audio_capable`, `reasoning_capable`, overhead de contexte |
| C10 | Auto-import depuis HF AI Energy Score (script Python S0 → Rust constant) | À venir |

## 7. Comment ajouter ou modifier un preset

1. Ajouter / modifier la ligne dans `MODEL_REGISTRY` (`model_presets.rs`)
   — **c'est la source de vérité** ; la liste des modèles n'est plus
   dupliquée dans ce document (cf. §4).
2. Ajouter au moins une source dans le champ `sources` (+ `source_url`).
3. Mettre à jour ce document **uniquement** si la méthodologie change
   (coefficients §2, statuts §3, limites §5).
4. Si validation paper-based : passer `calibration` à `Validated` et ajouter
   un test de reproduction dans `validation::`.
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
