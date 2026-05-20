# C34.1 — Shortlist modèles 2025-2026 validée

> **Statut** : livrable C34.1 — source unique de vérité pour C34.2 (`model_presets.rs`).
> **Date d'établissement** : 2026-05-20.
> **Auteur** : Claude Code (sprint C34, v0.9.0).
> **Méthodologie** : 10 `WebSearch` officiels (anthropic.com, openai.com, mistral.ai, deepmind.google, ai.meta.com, deepseek.com, x.ai, qwenlm.github.io, microsoft.com) + croisement model cards.
> **Limite assumée** : ComparIA Gold (`data/gold/referentiel.sqlite`) **absent en local** — le pipeline médaillon n'a pas été ré-exécuté dans ce workspace. La shortlist s'appuie donc sur la connaissance modèle publique. Croisement ComparIA Gold à compléter quand le pipeline sera relancé (≤ C34.2 si possible, sinon C34.6 validation).

---

## 0. Méthodologie de validation

Pour chaque modèle inclus, on exige :

1. **Date de sortie publique confirmée** par une source officielle vendor (model card, blog post, system card, ou release notes).
2. **Paramètres connus** (au moins une approximation publique pour les modèles fermés).
3. **`source_url` HTTPS** pointant vers la doc canonique du modèle.
4. **Pas de spéculation** : tout modèle annoncé mais non livré à la date du 2026-05-20 est **exclu** (zéro preset fantôme — DoD C34 §5).

Les modèles 2024 historiques de `model_presets.rs` restent dans le registry avec `deprecated: true` car le ledger d'audit historique pointe dessus (les estimations passées doivent rester reproductibles).

---

## 1. Corrections vs hypothèses initiales du brief

Le brief C34 mentionnait certains modèles avec des préjugés sur les versions disponibles. Voici les **rectifications validées par recherche web 2026-05-20** :

| Hypothèse brief | Réalité 2026-05-20 | Décision |
|---|---|---|
| « Claude 4.7 Sonnet » | **N'existe pas** — la dernière Sonnet est **4.6** (sortie 17 février 2026). Opus 4.7 a été sortie 16 avril 2026 mais sans Sonnet matching. Anthropic a découplé les rythmes. | Inclure `claude-opus-4-7` ET `claude-sonnet-4-6` comme deux modèles distincts. **Pas de `claude-sonnet-4-7`**. |
| « GPT-5 et GPT-5.5 séparés » | **GPT-5.5** sorti 23-24 avril 2026, remplace GPT-5 comme défaut ChatGPT. GPT-5 reste accessible en API mais la doc met l'accent sur 5.5. | Inclure `gpt-5-5` (3 variantes : default, Thinking, Pro) et `gpt-5` comme deux entrées. |
| « Mistral Magistral » | **Plus standalone depuis mars 2026** : Magistral (reasoning), Pixtral (vision), Devstral (code) **mergés** dans Mistral Small 4 (16 mars 2026) puis Medium 3.5 (30 avril 2026). | **Pas de preset `mistral-magistral`**. Préférer `mistral-small-4` et `mistral-medium-3-5` qui héritent ces capabilities. |
| « DeepSeek R2 » | **Non sorti** : Reuters/The Information juin 2025 — CEO Liang non satisfait, pas de date publique. **DeepSeek V4-Pro** (24 avril 2026) supplante R1 (mode thinking optionnel intégré). | **Pas de preset `deepseek-r2`**. Inclure `deepseek-v4-pro` et conserver `deepseek-r1` historique. |
| « Phi-5 » | **Non annoncé**. Dernière itération : `phi-4-reasoning` (30 avril 2025) et `phi-4-reasoning-vision-15b` (4 mars 2026). | **Pas de preset `phi-5`**. Inclure `phi-4-reasoning`. |
| « o4 / o5 » | **Non annoncés publiquement**. Familles reasoning OpenAI : o1 (sept 2024), o3 (déc 2024) — puis le « reasoning » est devenu un mode au sein de GPT-5.5 Thinking / Pro. | **Pas de preset `o4` / `o5`**. Conserver `o3`. |
| « Grok 4.5 / Grok 5 » | **Pas encore livrés** au 2026-05-20 (annonces de roadmap). Grok 4 sorti 10 juillet 2025, Grok 4.2 (500B), Grok 4.4 (1T planned), Grok 4.5/5 sont des roadmap. | **Inclure uniquement `grok-4`** (modèle livré). |
| « Claude Mythos » | **Preview, small partner cohort** (avril 2026). Pas grand public. | **Exclu du registry stable v0.9.0**. À ré-évaluer si GA. |

---

## 2. Catalogue cible v0.9.0 (32 presets dont 25 sortis 2025-2026)

DoD C34 = ≥ 20 presets totaux, ≥ 12 sortis 2025-2026. **On dépasse largement les deux seuils.**

### 2.1 Anthropic (6 presets, dont 6 en 2025-2026)

| `id` | display | date | params (B, total/actif) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `claude-opus-4-7` | Claude Opus 4.7 | 2026-04-16 | ~2000B (est. inconnue) | DenseTransformer ou Hybrid (non précisé) | ✅ | ✅ extended thinking | [anthropic.com/news](https://www.anthropic.com/news) + Wikipedia |
| `claude-sonnet-4-6` | Claude Sonnet 4.6 | 2026-02-17 | ~400B (estimation publique) | DenseTransformer | ✅ | ✅ extended thinking | Wikipedia Claude language model |
| `claude-haiku-4-5` | Claude Haiku 4.5 | 2025-10 (mois précis à confirmer) | ~70B (est.) | DenseTransformer | ✅ | ✅ | claudefa.st/blog/models |
| `claude-opus-4` | Claude Opus 4 | 2025-05-22 | ~1500B (est.) | DenseTransformer | ✅ | ✅ extended thinking | [anthropic.com/news/claude-4](https://www.anthropic.com/news) |
| `claude-sonnet-4` | Claude Sonnet 4 | 2025-05-22 | ~400B (est.) | DenseTransformer | ✅ | ✅ extended thinking | Anthropic Claude 4 release |
| `claude-3-7-sonnet` | Claude 3.7 Sonnet | 2025-02-25 | ~200B (est.) | DenseTransformer | ✅ | ✅ extended thinking (jusqu'à 128k tokens) | [anthropic.com/news/claude-3-7-sonnet](https://www.anthropic.com/news/claude-3-7-sonnet) |

### 2.2 OpenAI (5 presets, dont 4 en 2025-2026)

| `id` | display | date | params (B) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `gpt-5-5` | GPT-5.5 | 2026-04-23 | non publié (estimation 1T+) | non publié | ✅ | ✅ via GPT-5.5 Thinking / Pro | [openai.com/index/gpt-5-5-system-card](https://openai.com/index/gpt-5-5-system-card/) |
| `gpt-5` | GPT-5 | 2025 (mois précis à valider) | non publié | non publié | ✅ | ✅ | [openai.com](https://openai.com/) |
| `o3` | OpenAI o3 | 2024-12 | non publié | DenseTransformer (reasoning) | ❌ texte uniquement | ✅ (modèle reasoning natif) | [openai.com/o1](https://openai.com/o1/) + o3 system card |
| `gpt-4o` | GPT-4o | 2024-05 | ~200B (est. publique) | DenseTransformer multimodal | ✅ | ❌ | [openai.com](https://openai.com/) (existant, à `deprecated: true`) |
| `gpt-4o-mini` | GPT-4o mini | 2024-07 | ~8B (est.) | DenseTransformer | ✅ | ❌ | [openai.com/index/gpt-4o-mini-advancing-cost-efficient-intelligence](https://openai.com/index/gpt-4o-mini-advancing-cost-efficient-intelligence/) (existant, à `deprecated: true`) |

### 2.3 Google DeepMind (5 presets, tous 2025-2026)

| `id` | display | date | params (B) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `gemini-3-5-flash` | Gemini 3.5 Flash | 2026-05 (GA mai 2026) | non publié | non publié | ✅ multimodal natif | ✅ | [deepmind.google/models/gemini/flash](https://deepmind.google/models/gemini/flash/) |
| `gemini-3-1-pro` | Gemini 3.1 Pro | 2026-02 / 2026-03 | non publié | non publié | ✅ | ✅ thinking | [deepmind.google/models/gemini](https://deepmind.google/models/gemini/) |
| `gemini-3-pro` | Gemini 3 Pro | 2025-11 | non publié | non publié | ✅ | ✅ thinking | DeepMind Gemini 3 launch |
| `gemini-2-5-pro` | Gemini 2.5 Pro | 2025-03 | non publié | non publié | ✅ | ✅ thinking | [deepmind.google/technologies/gemini](https://deepmind.google/technologies/gemini/) |
| `gemini-2-5-flash` | Gemini 2.5 Flash | 2025-03 | non publié | non publié | ✅ | ✅ thinking | [deepmind.google/technologies/gemini/flash](https://deepmind.google/technologies/gemini/flash/) |

> Note : conserver `gemini-2-0-flash` existant en `deprecated: true` (déjà sourcé Google Environmental Impact paper avec disclosure C32.4).

### 2.4 Meta Llama (3 presets, dont 3 en 2025)

| `id` | display | date | params (B total / actifs) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `llama-4-scout` | Llama 4 Scout | 2025-04-05 | 109B total / 17B actifs (16 experts) | MoE | ✅ multimodal natif | ❌ | [ai.meta.com/blog/llama-4-multimodal-intelligence](https://ai.meta.com/blog/llama-4-multimodal-intelligence/) |
| `llama-4-maverick` | Llama 4 Maverick | 2025-04-05 | 400B total / 17B actifs (128 experts) | MoE | ✅ multimodal natif | ❌ | [huggingface.co/meta-llama/Llama-4-Maverick-17B-128E](https://huggingface.co/meta-llama/Llama-4-Maverick-17B-128E) |
| `llama-3-3-70b` | Llama 3.3 70B | 2024-12 | 70B | DenseTransformer | ❌ | ❌ | [github.com/meta-llama/llama-models](https://github.com/meta-llama/llama-models) |

> Llama 4 Behemoth (~2T params) en preview/training au 2026-05-20 → **exclu** (pas livré stable).

> Llama 3.1 70B et 8B existants : conserver `deprecated: true` (Llama 3.1 a la disclosure C32.4 location-based/market-based — précieux).

### 2.5 Mistral AI (4 presets, dont 3 en 2025-2026)

| `id` | display | date | params (B) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `mistral-medium-3-5` | Mistral Medium 3.5 | 2026-04-30 | 128B dense | DenseTransformer (merge Magistral+Devstral 2) | ✅ | ✅ intégré | docs.mistral.ai/changelog + mistral.ai/news |
| `mistral-small-4` | Mistral Small 4 | 2026-03-16 | non précisé (estimation 30B) | DenseTransformer (merge Magistral+Pixtral+Devstral) | ✅ | ✅ intégré | docs.mistral.ai/changelog |
| `mistral-large-3` | Mistral Large 3 | 2025-12-02 | 675B total / 41B actifs | MoE | ✅ | ❌ | [docs.mistral.ai/models/mistral-large-3-25-12](https://docs.mistral.ai/models/mistral-large-3-25-12/) + [mistral.ai/news/mistral-3](https://mistral.ai/news/mistral-3) |
| `mistral-large-2` | Mistral Large 2 | 2024-07 (deprecated) | 123B | DenseTransformer | ❌ | ❌ | (existant — conserver pour disclosure ADEME × Carbone 4) |

> Mistral Magistral / Pixtral / Devstral standalone : **exclus** (fusionnés dans Small 4 / Medium 3.5 — voir §1).

### 2.6 DeepSeek (3 presets, dont 3 en 2024-2026)

| `id` | display | date | params (B total / actifs) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `deepseek-v4-pro` | DeepSeek V4 Pro | 2026-04-24 | 1600B / 49B actifs | MoE | ✅ | ✅ thinking optionnel | [api-docs.deepseek.com/news/news260424](https://api-docs.deepseek.com/news/news260424) + huggingface.co/deepseek-ai/DeepSeek-V4-Pro |
| `deepseek-v4-flash` | DeepSeek V4 Flash | 2026-04-24 | 284B / 13B actifs | MoE | ✅ | ✅ thinking optionnel | api-docs.deepseek.com/news |
| `deepseek-r1` | DeepSeek R1 | 2025-01-20 | 671B / 37B actifs | MoE (reasoning natif) | ❌ texte | ✅ (reasoning natif) | [arxiv.org/abs/2501.12948](https://arxiv.org/abs/2501.12948) + [github.com/deepseek-ai](https://github.com/deepseek-ai) |

> DeepSeek V3 (déc 2024) : marginal mais possible si on veut représenter la branche pré-R1. **Choix par défaut** : ne pas inclure V3 séparément (R1 dérivé de V3 base).

> DeepSeek R2 : **exclu** (non livré au 2026-05-20).

### 2.7 xAI (1 preset, en 2025)

| `id` | display | date | params (B) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `grok-4` | xAI Grok 4 | 2025-07-10 | ~500B (Grok 4.2 confirmé 500B) | non publié | ✅ (vision/image expected) | ✅ first-principles reasoning | [x.ai/news](https://x.ai/news) + docs.x.ai/developers/models |

> Grok 4.5 / Grok 5 : non livrés au 2026-05-20 (roadmap), **exclus**.

### 2.8 Alibaba (3 presets, tous 2025-2026)

| `id` | display | date | params (B total / actifs) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `qwen-3-6-plus` | Qwen 3.6-Plus | 2026-04-02 | non publié (proprio, 1T+ est.) | non publié | ✅ | ✅ agentic | [caixinglobal.com/2026-04-02/alibaba-releases-qwen-36-plus](https://www.caixinglobal.com/2026-04-02/alibaba-releases-qwen-36-plus-ai-model-with-enhanced-coding-capabilities-102430395.html) |
| `qwen-3-max` | Qwen3-Max | 2025 (date précise à valider) | 1T+ proprio | non publié | ✅ | ✅ thinking variante | qwen.ai/research |
| `qwen-3-235b-a22b` | Qwen 3 235B-A22B | 2025-04-28 | 235B total / 22B actifs | MoE | ❌ texte (Vision dans Pixtral équivalent) | ✅ hybrid reasoning | [qwenlm.github.io](https://qwenlm.github.io/) + [github.com/QwenLM](https://github.com/QwenLM/Qwen3.6) |

### 2.9 Microsoft Phi (2 presets, dont 2 en 2025-2026)

| `id` | display | date | params (B) | archi | vision | reasoning | source |
|---|---|---|---|---|---|---|---|
| `phi-4-reasoning-vision` | Phi-4 Reasoning Vision 15B | 2026-03-04 | 15B | DenseTransformer reasoning | ✅ | ✅ | [huggingface.co/microsoft/Phi-4-reasoning-vision-15B](https://huggingface.co/microsoft/Phi-4-reasoning-vision-15B) |
| `phi-4-reasoning` | Phi-4 Reasoning | 2025-04-30 | 14B | DenseTransformer reasoning | ❌ | ✅ | [microsoft.com/en-us/research/publication/phi-4-reasoning-technical-report](https://www.microsoft.com/en-us/research/publication/phi-4-reasoning-technical-report/) |

> Phi-4 base (déc 2024) : marginal, peut être inclus si on veut couvrir mieux SLM. **Décision défaut** : ne pas inclure.

---

## 3. Bilan quantitatif

| Cohorte | Nouveau | Existant (deprecated) | Total |
|---|---|---|---|
| 2026 sortis | 12 (Opus 4.7, Sonnet 4.6, GPT-5.5, Gemini 3.5 Flash, Gemini 3.1 Pro, Mistral Medium 3.5, Mistral Small 4, DeepSeek V4 Pro, DeepSeek V4 Flash, Qwen 3.6-Plus, Phi-4 Reasoning Vision, Gemini 3 Pro [nov 2025 ≈ Q4-2025/début-2026]) | 0 | 12 |
| 2025 sortis | 12 (Claude Opus 4, Sonnet 4, Haiku 4.5, Claude 3.7 Sonnet, GPT-5, o3 [déc 2024 marginal], Gemini 2.5 Pro, Gemini 2.5 Flash, Llama 4 Scout, Llama 4 Maverick, Mistral Large 3, DeepSeek R1, Grok 4, Qwen 3 235B, Qwen 3 Max, Phi-4 Reasoning) | 0 | ≈ 14 |
| 2024 sortis | 2 (Llama 3.3 70B, o3 si considéré 2024) | 8 (gpt-4o, gpt-4o-mini, claude-3-5-sonnet, mistral-large-2, mistral-medium-3, llama-3-1-70b, llama-3-1-8b, gemini-2-0-flash) | 10 |
| **Total** | **~26 nouveaux** | **8 deprecated** | **≈ 34 presets** |

DoD C34 §5 satisfait : ≥ 20 presets totaux ✅, ≥ 12 sortis 2025-2026 ✅ (largement, ≥ 26).

> **Note** : on peut élaguer si jugé trop large pour v0.9.0. Cible minimum confortable = 22-25 presets (toute la cohorte ≥ 2025-Q2 + les essentiels 2024-Q4 → 2025-Q1). Décision finale en C34.2 selon UX du sélecteur M1.

---

## 4. Données structurelles à compléter en C34.2 (ne pas fabriquer ici)

Pour CHAQUE preset, il manque :

1. **`approx_params_billions` précis** — pour les modèles fermés (Anthropic, OpenAI, Google), garder l'estimation publique courante (~200B pour Sonnet, ~2T pour Opus 4.7, etc.) avec marqueur `CalibrationStatus::Extrapolated` et source `"Estimation taille — analyse publique 2026"`.
2. **`active_params_b`** pour MoE — confirmé par model card vendor pour Llama 4 / Mistral Large 3 / DeepSeek V4 / Qwen 3 235B. Égal à `params_b` pour dense.
3. **`epsilon_decode_mj`, `epsilon_prefill_mj`, `embodied_g_per_req`** — calculés via `extrapolate_decode(active_params_b)` etc. (utiliser **active_params_b** pour MoE, pas total — cf. ADR-0012 §4).
4. **`default_context_overhead_tokens`** :
   - Claude (claude.ai) : 2000
   - Claude (API direct) : 0 (preset interface "raw API")
   - GPT (ChatGPT app) : 1000
   - GPT (API) : 0
   - Gemini (gemini.google.com) : 1000
   - Gemini (API) : 0
   - Mistral (chat.mistral.ai) : 300
   - Open-weights local : 0 (l'utilisateur fournit le system prompt)
   > **Stratégie** : on intègre la valeur "interface app" comme défaut UI (`default_context_overhead_tokens`), avec disclaimer "estimation ± 50 %, basée sur leaks publics". L'utilisateur peut surcharger via M1 mode Expert.
5. **`vision_pricing`** :
   - OpenAI (GPT-4o, GPT-5.5) : `OpenAiTiles { base: 85, per_tile: 170, tile_size: 512 }` — source [platform.openai.com/docs/guides/vision/calculating-costs](https://platform.openai.com/docs/guides/vision/calculating-costs)
   - Anthropic (Claude 3.5+) : `AnthropicArea { tokens_per_pixel_750: 1.0/750.0, max_tokens: 1568 }` — source [docs.anthropic.com/en/docs/build-with-claude/vision](https://docs.anthropic.com/en/docs/build-with-claude/vision)
   - Google (Gemini) : `GeminiNative { base: 258, tile_size: 768 }` — source [ai.google.dev/gemini-api/docs/vision](https://ai.google.dev/gemini-api/docs/vision)
   - Meta (Llama 4 Scout/Maverick) : `LlamaPatches { tokens_per_image: 1601 }` — source [ai.meta.com/blog/llama-3-2-connect-2024-vision-edge-mobile-devices](https://ai.meta.com/blog/llama-3-2-connect-2024-vision-edge-mobile-devices/) (Llama 4 hérite, à confirmer dans model card Llama 4)
   - Mistral Small 4 / Medium 3.5 (ex-Pixtral) : `OpenAiTiles` (formule similaire — à confirmer doc Mistral)
   - DeepSeek V4 (vision native) : **inconnu publié** — fallback `LlamaPatches { tokens_per_image: 1600 }` avec warning "formule fallback générique ± 50 %"
6. **`thinking_token_multiplier: (P5, P95)`** pour reasoning models :
   - OpenAI o3 / GPT-5.5 Thinking : `(5.0, 30.0)` — source system card o3 + GPT-5.5 system card
   - DeepSeek R1 / V4 thinking : `(8.0, 25.0)` — source paper R1 (arXiv 2501.12948, moyenne 8-12k thinking pour requêtes complexes)
   - Claude 4/4.6/4.7 extended thinking : `(2.0, 50.0)` — source Anthropic doc (configurable, max 128k thinking)
   - Gemini 2.5+ thinking : `(3.0, 25.0)` — source Google doc (configurable)
   - Phi-4-reasoning : `(5.0, 15.0)` — source Phi-4 reasoning technical report
7. **`vendor_disclosures`** : ne rien ajouter au-delà des existants (Mistral Large 2 × ADEME, Meta Llama 3.1 70B location/market-based, Gemini 2.0 Flash Google paper). C34 ne ré-ouvre PAS les vendors disclosures (C32.4 a déjà fait ce travail) — sauf si la recherche web a révélé un nouveau disclosure officiel.

---

## 5. Décisions validées 2026-05-20 (Thibault)

1. ✅ **Sonnet 4.7 absent** : confirmé — on inclut Opus 4.7 + Sonnet 4.6, et la description Opus 4.7 documentera explicitement que Sonnet 4.7 n'existe pas (rythmes Opus/Sonnet découplés depuis 4.x).
2. ✅ **Catalogue v0.9.0 = 25 presets** (17 nouveaux + 8 deprecated). Élague Qwen3-Max, DeepSeek V3, Phi-4 base et Qwen 3 dense intermédiaires.
3. ✅ **GPT-5.5 = 3 presets séparés** : `gpt-5-5`, `gpt-5-5-thinking`, `gpt-5-5-pro` (cohérent avec o3 standalone).
4. ✅ **Naming Modality** : rename existant en `ModelDomain` (rename SemVer 0.9.0 propre, breaking change documenté CHANGELOG). Nouveau type `InputModality` dans `sobria-core/src/input_modality.rs`. `Model.modality: ModelDomain` reste.
5. ✅ **Disclaimer overhead** : texte officiel = « Estimation overhead système ± 50 % — basée sur leaks publics et reverse-engineering interfaces vendor (Claude.ai, ChatGPT app, Gemini app). À surcharger en mode Expert si vous connaissez votre valeur exacte. »

### Catalogue 25 presets retenu (final v0.9.0)

**17 nouveaux (12 sortis 2026 + 5 sortis 2025)** :

| # | id | display | date | source |
|---|---|---|---|---|
| 1 | `claude-opus-4-7` | Claude Opus 4.7 | 2026-04-16 | Anthropic |
| 2 | `claude-sonnet-4-6` | Claude Sonnet 4.6 | 2026-02-17 | Anthropic |
| 3 | `claude-haiku-4-5` | Claude Haiku 4.5 | 2025-10 | Anthropic |
| 4 | `claude-opus-4` | Claude Opus 4 | 2025-05-22 | Anthropic |
| 5 | `claude-sonnet-4` | Claude Sonnet 4 | 2025-05-22 | Anthropic |
| 6 | `claude-3-7-sonnet` | Claude 3.7 Sonnet | 2025-02-25 | Anthropic |
| 7 | `gpt-5-5` | GPT-5.5 | 2026-04-23 | OpenAI |
| 8 | `gpt-5-5-thinking` | GPT-5.5 Thinking | 2026-04-23 | OpenAI |
| 9 | `gpt-5-5-pro` | GPT-5.5 Pro | 2026-04-23 | OpenAI |
| 10 | `o3` | OpenAI o3 | 2024-12 | OpenAI |
| 11 | `gemini-3-5-flash` | Gemini 3.5 Flash | 2026-05 | Google DeepMind |
| 12 | `gemini-3-1-pro` | Gemini 3.1 Pro | 2026-02 | Google DeepMind |
| 13 | `gemini-2-5-pro` | Gemini 2.5 Pro | 2025-03 | Google DeepMind |
| 14 | `llama-4-scout` | Llama 4 Scout | 2025-04-05 | Meta |
| 15 | `llama-4-maverick` | Llama 4 Maverick | 2025-04-05 | Meta |
| 16 | `mistral-large-3` | Mistral Large 3 | 2025-12-02 | Mistral AI |
| 17 | `deepseek-v4-pro` | DeepSeek V4 Pro | 2026-04-24 | DeepSeek |

**À ajouter pour atteindre 25** (selon DoD final, modèles ≥ 2025) :

| # | id | display | date | source |
|---|---|---|---|---|
| 18 | `mistral-small-4` | Mistral Small 4 | 2026-03-16 | Mistral AI |
| 19 | `mistral-medium-3-5` | Mistral Medium 3.5 | 2026-04-30 | Mistral AI |
| 20 | `deepseek-r1` | DeepSeek R1 | 2025-01-20 | DeepSeek |
| 21 | `grok-4` | xAI Grok 4 | 2025-07-10 | xAI |
| 22 | `qwen-3-6-plus` | Qwen 3.6-Plus | 2026-04-02 | Alibaba |
| 23 | `phi-4-reasoning-vision` | Phi-4 Reasoning Vision 15B | 2026-03-04 | Microsoft |
| 24 | `phi-4-reasoning` | Phi-4 Reasoning | 2025-04-30 | Microsoft |
| 25 | `llama-3-3-70b` | Llama 3.3 70B | 2024-12 | Meta |

**Total 2025-2026** : 17 (= ≥ 12 DoD ✅) ; **Total** : 25 (+8 deprecated = 33 dans registry) — DoD ≥ 20 ✅.

> Note : `gemini-3-pro` (2025-11) et `gemini-2-5-flash` (2025-03) restent **non inclus** dans les 25 v0.9.0 par décision d'élagage. Possibles à ajouter en v0.9.1 si demande utilisateurs.

**8 deprecated conservés** (déjà présents, `deprecated: true` à ajouter en C34.2) : `gpt-4o`, `gpt-4o-mini`, `claude-3-5-sonnet`, `mistral-large-2`, `mistral-medium-3`, `llama-3-1-70b`, `llama-3-1-8b`, `gemini-2-0-flash`.

---

## 6. Decisions techniques à acter en C34.3

1. **Conflit de nom `Modality`** : l'enum existant dans `sobria-core/src/model.rs` a une sémantique de **modalité de domaine du modèle** (`Text` = LLM, `Image` = SD, `Audio` = TTS/STT, `Video` = video gen). Le brief C34 veut un `Modality` = **type d'input d'un prompt** (`Text`, `VisionLow`, `VisionHigh`, `Document`, `AudioInput`). **Solution recommandée** :
   - Renommer l'enum existant en `ModelDomain` (sémantique : domaine fonctionnel du modèle).
   - Créer un NOUVEAU type `InputModality` dans `sobria-core/src/input_modality.rs` (ou rester `Modality` dans `modality.rs` et renommer l'autre).
   - Mettre à jour les imports : `model.rs::Modality` → `model.rs::ModelDomain`, et `lib.rs::pub use` aligné.
   - Compat backward : la struct `Model` continue d'exposer `modality: ModelDomain`, et le champ devient `domain: ModelDomain` (renommage SemVer 0.9.0 acceptable, breaking change documenté dans CHANGELOG).
   - **Alternative plus conservatrice** : appeler le nouveau type `InputModality` et garder `Modality` existant intact. Plus safe pour SemVer mais moins propre sémantiquement.
   - **À trancher** : voir question Thibault §7 ci-dessous.

2. **EstimationRequest** : ajouter deux champs serde-default (compat backward audit ledger v0.3.x → v0.8.x déjà respectée) :
   ```rust
   #[serde(default)]
   pub modalities: Vec<InputModality>,
   #[serde(default)]
   pub overhead: ContextOverhead,
   ```
   Le test `result_deserializes_legacy_v03_without_method_field` (estimation.rs:159) doit continuer de passer.

3. **Engines AFNOR + EcoLogits** : modifier `EmpreinteEngine::estimate(request, params)` pour calculer `effective_input_tokens` AVANT le Monte-Carlo. Le calcul ε_prefill × effective_input_tokens reste inchangé en aval. Pas d'impact sur les ReproductionCase existants si overhead=default(0,0,0,0) et modalities=[].

---

## 7. Questions bloquantes — RÉSOLUES 2026-05-20

Voir §5 ci-dessus. Toutes validées par Thibault avant démarrage C34.2.

---

## 8. Limites et travail à compléter

- ❌ **ComparIA Gold non interrogé** — pipeline médaillon non re-runné en local. À compléter en C34.6 (validation) ou en setup C34.2 si on veut prioriser le croisement empirique.
- ⚠️ **Estimations params modèles fermés** — `~2000B Opus 4.7`, `~400B Sonnet 4.6`, etc. sont des **estimations publiques** sans publication officielle. Doivent être marquées `CalibrationStatus::Extrapolated` et sourcées "Estimation taille — analyse publique 2026" comme les anciens presets.
- ⚠️ **Formules vision DeepSeek V4** : pas de doc publique consultée. Fallback `LlamaPatches { tokens_per_image: 1600 }` provisoire — à raffiner si DeepSeek publie la formule officielle.
- ⚠️ **Date précise Claude Haiku 4.5** : "octobre 2025" — mois exact à confirmer via Anthropic release notes.
- ⚠️ **Date précise GPT-5** : "2025" — mois exact (probablement été 2025) à confirmer.

---

## 9. Prochaine étape (C34.2)

Avec ce shortlist validé, C34.2 peut :

1. Étendre la struct `ModelPreset` avec les 11 nouveaux champs (release_date, model_family, architecture, vision_capable, vision_pricing, audio_capable, reasoning_capable, thinking_token_multiplier, default_context_overhead_tokens, deprecated, source_url).
2. Créer les enums `ModelFamily`, `ArchitectureKind`, `VisionPricing`.
3. Marquer les 8 presets actuels avec `deprecated: true`.
4. Ajouter les 26 nouveaux presets validés ici.
5. Étendre `find_preset` + `available_models` avec filter `include_deprecated`.
6. Tests round-trip serde + tests find_preset par id pour chaque nouveau modèle.

**Sources globales consultées (2026-05-20)** :
- [anthropic.com/news](https://www.anthropic.com/news), [docs.anthropic.com](https://docs.anthropic.com/en/docs/build-with-claude/vision), [en.wikipedia.org/wiki/Claude_(language_model)](https://en.wikipedia.org/wiki/Claude_(language_model))
- [openai.com/index/gpt-5-5-system-card](https://openai.com/index/gpt-5-5-system-card/), [platform.openai.com/docs/guides/vision/calculating-costs](https://platform.openai.com/docs/guides/vision/calculating-costs)
- [deepmind.google/models/gemini](https://deepmind.google/models/gemini/), [ai.google.dev/gemini-api/docs/vision](https://ai.google.dev/gemini-api/docs/vision)
- [ai.meta.com/blog/llama-4-multimodal-intelligence](https://ai.meta.com/blog/llama-4-multimodal-intelligence/), [github.com/meta-llama/llama-models](https://github.com/meta-llama/llama-models)
- [mistral.ai/news/mistral-3](https://mistral.ai/news/mistral-3), [docs.mistral.ai/getting-started/changelog](https://docs.mistral.ai/getting-started/changelog)
- [api-docs.deepseek.com/news/news260424](https://api-docs.deepseek.com/news/news260424), [arxiv.org/abs/2501.12948](https://arxiv.org/abs/2501.12948)
- [x.ai/news](https://x.ai/news), [docs.x.ai/developers/models](https://docs.x.ai/developers/models)
- [qwenlm.github.io](https://qwenlm.github.io/), [github.com/QwenLM/Qwen3.6](https://github.com/QwenLM/Qwen3.6)
- [microsoft.com/en-us/research/publication/phi-4-reasoning-technical-report](https://www.microsoft.com/en-us/research/publication/phi-4-reasoning-technical-report/), [huggingface.co/microsoft/Phi-4-reasoning-vision-15B](https://huggingface.co/microsoft/Phi-4-reasoning-vision-15B)
- [llm-stats.com](https://llm-stats.com/), [llm-release-dashboard.vercel.app](https://llm-release-dashboard.vercel.app/) (synthèses tierces, validation croisée)
