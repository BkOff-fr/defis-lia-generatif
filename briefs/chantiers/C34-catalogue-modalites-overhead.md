# Chantier C34 — Catalogue 2026 + modalités + overhead (v0.9.0)

> **Version cible** : v0.9.0
> **Sprint** : ~6 jours focalisés méthodologie moteur + UI
> **Pré-requis** : v0.8.0 shippée (C32 Clarté produit), ComparIA Gold ingéré v0.5.0, vendors disclosure intégrés v0.8.0
> **Lien** : ADR-0012 (multi-méthodologie), brief C30 audit datasets
> **Décision** : précède C33 site internet — on doit présenter un moteur 2026-credible avant de marketing.

---

## 0. Pourquoi ce chantier maintenant

Le moteur Sobr.ia v0.8.0 a **3 trous critiques** qui invalident sa crédibilité scientifique pour la candidature data.gouv.fr :

### Trou 1 — Catalogue obsolète (8 modèles 2024)

Presets actuels dans `crates/sobria-estimator/src/model_presets.rs` :
- gpt-4o, gpt-4o-mini (mai 2024)
- claude-3-5-sonnet (juin 2024) — Claude 3.7 (févr 2025) et Claude 4 sortis depuis
- mistral-large-2 (juil 2024), mistral-medium-3 (2025)
- llama-3-1-70b, llama-3-1-8b (juil 2024) — Llama 3.3 (déc 2024), Llama 4 sortis
- gemini-2-0-flash (déc 2024) — Gemini 2.5 (mars 2025) sorti

**Manquent** (au moins) : Claude 4, GPT-5/o3, Gemini 2.5 Pro, Llama 4, DeepSeek V3+R1, Qwen 2.5/3, Grok 3, Phi-4, Mistral Large 3, Mistral Small 3, Codestral 2, Pixtral, et tous les reasoning models.

### Trou 2 — Pas d'overhead contextualisation

Un user qui tape un prompt de 50 tokens sur ChatGPT envoie en réalité ~800-2000 tokens facturés :
- **System prompt** caché : Claude 3.5 ~1500 tokens, GPT-4o ~600-1500 tokens, ChatGPT app ~800-1200 tokens (source : leaks Anthropic + reverse-engineering publics).
- **Tools définitions** : ~200-1500 tokens si l'utilisateur a activé code interpreter / web search / etc.
- **Memory / conversation context** : tokens accumulés des tours précédents.

Sans modélisation de cet overhead, **notre P50 sous-estime d'un facteur 5-30×** la réalité opérationnelle. C'est catastrophique pour le pitch.

### Trou 3 — Aveugle aux modalités non-texte

En 2026 ~30 % des requêtes sont multimodales (image, document, audio). Notre moteur estime ZÉRO impact pour :
- 1 image GPT-4o haute résolution = 85 + 170 × tiles ≈ 765 à 2125 tokens (source: OpenAI Vision pricing doc).
- 1 image Claude 3.5 = ~1568 tokens (source: Anthropic vision doc).
- 1 PDF 5 pages dans la conversation = ~15 k tokens.
- 1 minute d'audio Whisper-style = ~600 tokens.
- **Thinking tokens** d'un reasoning model (o1, DeepSeek R1, Claude 4 extended thinking) = 5×–100× les tokens output visibles.

---

## 1. Périmètre

### En périmètre

**Bloc A — Catalogue moderne** (C34.1 + C34.2) :
- Extraction exhaustive des modèles présents dans ComparIA Gold.
- Enrichissement `model_presets.rs` avec ~20 nouveaux presets 2025-2026.
- Champs structurés ajoutés à chaque preset :
  - `release_date: chrono::NaiveDate` (date de sortie publique).
  - `model_family: ModelFamily` enum (`OpenAi`, `Anthropic`, `GoogleDeepMind`, `MetaAi`, `MistralAi`, `DeepSeek`, `Alibaba`, `Xai`, `Microsoft`, `Other`).
  - `architecture: ArchitectureKind` enum (`DenseTransformer`, `MoE { experts: u32, active_experts: u32 }`, `Mamba`, `Hybrid`).
  - `vision_capable: bool`.
  - `audio_capable: bool` (input audio).
  - `reasoning_capable: bool` (modèles à chain-of-thought intégré : o1, o3, R1, Claude 4 thinking, Gemini 2.5 Pro thinking).
  - `default_context_overhead_tokens: u32` (system prompt typique pour ce modèle).
  - `vision_pricing: Option<VisionPricing>` (formule de tokens vision).
  - `thinking_token_multiplier: Option<(f64, f64)>` (P5, P95 du ratio thinking/output pour reasoning).
  - `deprecated: bool` (modèles 2024 marqués deprecated mais conservés pour historique).
  - `source_url: &'static str` (model card officielle).

**Bloc B — Type Modality + overhead** (C34.3) :
- Nouveau type `Modality` enum dans `sobria-core` :
  ```rust
  pub enum Modality {
      Text,
      VisionLow { image_count: u32 },
      VisionHigh { image_count: u32, avg_width: u32, avg_height: u32 },
      Document { page_count: u32 },
      AudioInput { duration_seconds: u32 },
  }
  ```
- Nouveau struct `ContextOverhead` :
  ```rust
  pub struct ContextOverhead {
      pub system_prompt_tokens: u32,
      pub tools_definition_tokens: u32,
      pub memory_tokens: u32,
      pub thinking_tokens_p50: u32,  // pour reasoning models
  }
  ```
- `EstimationRequest` étendu avec `modalities: Vec<Modality>` et `overhead: ContextOverhead`.
- Calcul `effective_input_tokens = user_visible_tokens + overhead.total() + modalities.sum_tokens(preset)`.
- Formule tokens vision par modèle (extraite des docs officielles, cf. §3 sources).
- Formule tokens document : `OCR_tokens_per_page = 850 * page_count` (PDF moyen 850 mots × 1.3 token/mot).
- Formule tokens audio : `audio_tokens = duration_seconds * 10` (Whisper rate, à valider).
- `thinking_tokens_p50` calculé via `output_tokens * thinking_multiplier` quand `preset.reasoning_capable`.

**Bloc C — UI Atelier M1 + Fiche M9** (C34.4 + C34.5) :
- M1 Atelier (`web/src/routes/+page.svelte`) :
  - Sélecteur de modalités (toggles Text / Image / Document / Audio).
  - Section repliable "Détails techniques" avec :
    - Champ `system_prompt_tokens` (défaut auto-rempli par preset.default_context_overhead_tokens).
    - Champ `tools_tokens` (défaut 0).
    - Si modalité Image : champ `image_count`, sélecteur "Basse / Haute résolution".
    - Si modalité Document : champ `page_count`.
    - Si modalité Audio : champ `duration_seconds`.
    - Si modèle reasoning : info-tooltip "ce modèle ajoute X-Y thinking tokens automatiquement".
  - Tooltip pédagogique sur chaque champ avec source (lien OpenAI Vision pricing / Anthropic vision doc / etc.).
  - Mode "Simple" (cache les détails techniques par défaut) vs "Expert" (tout déplié).
- M9 Fiche modèle (`web/src/routes/m9/[id]/+page.svelte` ou équivalent) :
  - Section "Capacités" : badges Vision / Audio / Reasoning / MoE / etc.
  - Section "Overhead par défaut" : table avec system / tools / memory + total.
  - Si vision : viz "1 image basse rés = X tokens · haute rés 512×512 = Y tokens · haute rés 1024×1024 = Z tokens".
  - Si reasoning : viz "ratio thinking/output P5-P50-P95 = N×-M×-K×".

**Bloc D — Tests et ship** (C34.6) :
- Tests reproductibilité : 5 nouveaux `ReproductionCase` couvrant chaque modalité (vision + document + audio + reasoning + overhead).
- Smoke test E2E : prompt avec image sur M1 → estimation cohérente.
- CHANGELOG `[0.9.0]` + bump versions partout + tag.

### Hors périmètre

- Refonte profonde moteur Monte-Carlo (rester sur calibration légère).
- Nouveau crate.
- Output audio (TTS) — différé v1.x.
- Vidéo en input ou output — différé v1.x.
- Tokens caching côté OpenAI (-50% prix sur context répété) — différé v1.x.
- Predictive outputs (OpenAI feature) — différé v1.x.
- Fine-tune custom : on documente comment l'estimer mais on ne crée pas d'UI dédiée v0.9.0.

---

## 2. Architecture types

### Modality

```rust
// crates/sobria-core/src/modality.rs

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Modality {
    Text,
    VisionLow { image_count: u32 },
    VisionHigh { image_count: u32, avg_width: u32, avg_height: u32 },
    Document { page_count: u32 },
    AudioInput { duration_seconds: u32 },
}

impl Modality {
    /// Calcule les tokens supplémentaires pour cette modalité selon le preset.
    /// Renvoie 0 si le modèle ne supporte pas la modalité (warning loggé).
    pub fn tokens_overhead(&self, preset: &ModelPreset) -> u32 { ... }
}
```

### ContextOverhead

```rust
// crates/sobria-core/src/context_overhead.rs

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ContextOverhead {
    pub system_prompt_tokens: u32,
    pub tools_definition_tokens: u32,
    pub memory_tokens: u32,
    pub thinking_tokens_p50: u32,
}

impl ContextOverhead {
    pub fn from_preset(preset: &ModelPreset, has_reasoning: bool) -> Self { ... }
    pub fn total(&self) -> u32 { ... }
}
```

### VisionPricing

```rust
// crates/sobria-estimator/src/model_presets.rs

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VisionPricing {
    /// Formule OpenAI : 85 + 170 × ⌈W/512⌉ × ⌈H/512⌉ tokens.
    OpenAiTiles { base: u32, per_tile: u32, tile_size: u32 },
    /// Formule Anthropic Claude vision : tokens = (W × H) / 750, max 1568.
    AnthropicArea { tokens_per_pixel_750: f64, max_tokens: u32 },
    /// Gemini multimodal natif : 258 tokens par image (≤ 384×384) ou 258 × tiles (768×768 chacun).
    GeminiNative { base: u32, tile_size: u32 },
    /// Llama 3.2 Vision : 1601 tokens par image (560×560 patches).
    LlamaPatches { tokens_per_image: u32 },
}

impl VisionPricing {
    pub fn tokens_for(&self, count: u32, width: u32, height: u32, high_detail: bool) -> u32 { ... }
}
```

### ModelPreset étendu

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreset {
    pub id: &'static str,
    pub display_name: &'static str,
    pub release_date: NaiveDate,
    pub model_family: ModelFamily,
    pub architecture: ArchitectureKind,
    pub params_b: f64,
    pub active_params_b: f64,  // ≠ params_b si MoE
    pub vision_capable: bool,
    pub vision_pricing: Option<VisionPricing>,
    pub audio_capable: bool,
    pub reasoning_capable: bool,
    pub thinking_token_multiplier: Option<(f64, f64)>,  // P5, P95
    pub default_context_overhead_tokens: u32,
    pub deprecated: bool,
    pub source_url: &'static str,
    pub vendor_disclosure: Option<VendorDisclosure>,  // déjà ajouté v0.8.0
    // champs existants : k_decode, k_prefill, k_embodied, etc.
    ...
}
```

---

## 3. Sources autoritatives par information

### Modèles modernes (release dates + caractéristiques)

⚠️ **À FAIRE EN C34.1** : Claude Code DOIT lancer une recherche web actualisée au début du chantier pour valider la shortlist 2026 réelle. Cowork a un cutoff connaissance fin mai 2025, donc le tableau ci-dessous est probablement incomplet. Notamment vérifier l'existence et la version exacte de :
- **Claude 4.7 Opus + 4.7 Sonnet** (mentionnés par Thibault — confirmer release date + caractéristiques)
- **GPT-5 / GPT-5.5** (mentionnés par Thibault — confirmer version actuelle)
- Toutes les versions Gemini ≥ 2.5 (Pro / Flash / Nano).
- Toutes les versions Llama ≥ 3.3 (incluant Llama 4 si sorti).
- Tous les modèles reasoning successeurs de o1/o3.

| Modèle (catalogue cible 2026-Q3) | Sortie estimée | Source canonique |
|---|---|---|
| **Anthropic Claude 4.7 Opus + 4.7 Sonnet** ⭐ | 2026 (confirmer) | [anthropic.com/news](https://www.anthropic.com/news) |
| **Anthropic Claude 4 Opus + Sonnet + Haiku** | 2025 | Anthropic releases |
| Claude 3.7 Sonnet | févr 2025 | [anthropic.com/news/claude-3-7-sonnet](https://www.anthropic.com/news/claude-3-7-sonnet) |
| **OpenAI GPT-5 + GPT-5.5** ⭐ | 2025-2026 (confirmer) | [openai.com](https://openai.com/) |
| OpenAI o1 / o1-pro / o3 / o3-mini | sept 2024 / déc 2024 | [openai.com/o1](https://openai.com/o1/) + system cards |
| **OpenAI o4 / o5** (si dispo) | 2026 (confirmer) | OpenAI roadmap |
| **Google Gemini 2.5 Pro + Flash + Nano** | mars 2025 | [deepmind.google/technologies/gemini/](https://deepmind.google/technologies/gemini/) |
| Google Gemini 3.x (si dispo) | 2026 (confirmer) | DeepMind |
| Llama 3.3 70B | déc 2024 | [github.com/meta-llama/llama-models](https://github.com/meta-llama/llama-models) |
| **Llama 4 + Llama 4 Maverick + Scout** | 2025 (confirmer) | GitHub model cards |
| Mistral Large 3 / Small 3 / Codestral 2 / Pixtral / Pixtral Large | 2025 | [mistral.ai/news](https://mistral.ai/news/) |
| **Mistral Magistral** (reasoning, si dispo) | 2026 (confirmer) | Mistral releases |
| DeepSeek V3 / R1 | déc 2024 / janv 2025 | [github.com/deepseek-ai](https://github.com/deepseek-ai) + arXiv [2501.12948](https://arxiv.org/abs/2501.12948) |
| **DeepSeek V4 / R2** (si dispo) | 2026 (confirmer) | DeepSeek releases |
| Qwen 2.5 / **Qwen 3** | sept 2024 / 2025 (confirmer) | [qwenlm.github.io](https://qwenlm.github.io/) |
| Grok 3 + **Grok 4** (si dispo) | févr 2025 / 2026 (confirmer) | [x.ai/news](https://x.ai/news) |
| Phi-3.5 / Phi-4 / **Phi-5** (si dispo) | août 2024 / déc 2024 / 2026 (confirmer) | [microsoft.com/en-us/research/](https://www.microsoft.com/en-us/research/) |

**Instruction Claude Code** : au début de C34.1, lancer des `WebSearch` pour :

1. `"Claude 4.7 Opus Sonnet release date 2026 system card"`
2. `"GPT-5 GPT-5.5 OpenAI release 2026 model card"`
3. `"latest LLM releases 2026 Q2 catalog"`
4. `"Llama 4 Meta release 2025 2026"`
5. `"DeepSeek 2026 latest model R2 V4"`

Compléter ensuite le tableau ci-dessus avec dates et URLs exactes. **Si un modèle annoncé n'est pas effectivement sorti à la date d'exécution, ne PAS l'inclure dans les presets** (zéro preset fantôme).

**Cible minimale** : 20 modèles dans presets, dont 12+ sortis en 2025-2026. Anciens 2024 deprecated mais conservés.

### Vision pricing

- **OpenAI GPT-4o / GPT-4 Vision** : [platform.openai.com/docs/guides/vision/calculating-costs](https://platform.openai.com/docs/guides/vision/calculating-costs) — formule `85 + 170 × ⌈W/512⌉ × ⌈H/512⌉` pour high detail, fixe `85` pour low detail.
- **Anthropic Claude Vision** : [docs.anthropic.com/en/docs/build-with-claude/vision](https://docs.anthropic.com/en/docs/build-with-claude/vision) — formule `tokens = (W × H) / 750`, max 1568.
- **Google Gemini Vision** : [ai.google.dev/gemini-api/docs/vision](https://ai.google.dev/gemini-api/docs/vision) — 258 tokens fixes pour images ≤ 384×384, sinon 258 × ⌈W/768⌉ × ⌈H/768⌉.
- **Llama 3.2 Vision** : [ai.meta.com/blog/llama-3-2-connect-2024-vision-edge-mobile-devices/](https://ai.meta.com/blog/llama-3-2-connect-2024-vision-edge-mobile-devices/) — 1601 tokens par image traitée (patches 14×14 sur 560×560).
- **Mistral Pixtral** : doc Mistral, formule similaire à GPT-4o.

### Overhead système (system prompt typique)

- **Anthropic Claude (claude.ai)** : [evanzhoudev.github.io/claude-system-prompts/](https://evanzhoudev.github.io/claude-system-prompts/) (community leaks) — ~1500-2500 tokens selon version.
- **OpenAI ChatGPT (app)** : reverse engineering publics + plusieurs leaks reddit r/ChatGPT — ~600-1200 tokens.
- **OpenAI API direct** : 0 system prompt par défaut (user fournit).
- **Mistral Chat (chat.mistral.ai)** : ~200-400 tokens.
- **Google Gemini (gemini.google.com)** : ~800-1200 tokens.

**Pour Sobr.ia** : on intègre une valeur P50 par modèle dans `default_context_overhead_tokens` avec disclaimer "estimation basée sur leaks publics, peut varier ± 50 %".

### Reasoning thinking tokens

- **OpenAI o1 / o3** : system card mentionne ratio 5×-30× output tokens, P50 ≈ 15× pour problèmes math/code.
- **DeepSeek R1** : paper arXiv [2501.12948](https://arxiv.org/abs/2501.12948) — moyenne 8-12k thinking tokens par requête complexe.
- **Claude 4 Extended Thinking** : Anthropic doc — configurable, défaut ~16k max thinking tokens.
- **Gemini 2.5 Pro Thinking** : doc Google — configurable similaire.

### Document / audio

- **Document PDF** : conversion OCR + tokenisation, estimation 850 mots × 1.3 token/mot ≈ 1100 tokens/page. Source : analyse empirique LMSYS arena uploads.
- **Audio input** : Whisper rate ~10 tokens/seconde (source : OpenAI Whisper paper + Realtime API doc).

---

## 4. Découpage temporel

| Jour | Sous-chantier | Livrable |
|---|---|---|
| J1 | C34.1 Audit modèles ComparIA Gold | Query SQL Gold + liste exhaustive + shortlist 20 nouveaux presets |
| J2 | C34.2 Enrichir model_presets.rs | 20 nouveaux presets, champs étendus, tests round-trip |
| J3 | C34.3 Type Modality + ContextOverhead | Nouveaux types core + intégration EstimationRequest |
| J4 | C34.3 fin + C34.4 UI M1 | Calcul effective_tokens + UI Atelier modalités |
| J5 | C34.4 fin + C34.5 UI M9 | Tooltip pédagogique + fiche M9 capacités |
| J6 | C34.6 Tests + ship | 5 ReproductionCase + smoke E2E + tag v0.9.0 |

Total : **~6 jours** focalisés.

---

## 5. Definition of Done v0.9.0

- [ ] `cargo test --workspace` 100 % vert (avec 5 nouveaux ReproductionCase).
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cd web && npm run check && npm run lint && npm run test` propre.
- [ ] **≥ 20 modèles** dans `model_presets.rs` dont **≥ 12 sortis en 2025-2026**.
  - **Obligatoires si effectivement sortis à la date d'exécution** : Claude 4.7 Opus + 4.7 Sonnet, Claude 4 Opus/Sonnet/Haiku, Claude 3.7 Sonnet, GPT-5 + GPT-5.5, o1/o3 (+ o4/o5 si dispo), Gemini 2.5 Pro + Flash + Nano, Llama 3.3 + Llama 4, DeepSeek V3 + R1 (+ V4/R2 si dispo), Qwen 2.5 + Qwen 3, Grok 3 + Grok 4 (si dispo), Phi-4 (+ Phi-5 si dispo), Mistral Large 3 + Small 3 + Codestral 2 + Pixtral.
  - Vérification web faite en C34.1 pour confirmer dates réelles.
  - **Zéro preset fantôme** : un modèle non encore sorti n'est PAS inclus.
- [ ] Anciens modèles 2024 marqués `deprecated: true` (conservés pour historique audit ledger).
- [ ] **5 modalités** supportées : Text + VisionLow + VisionHigh + Document + AudioInput.
- [ ] **ContextOverhead** appliqué à toutes les estimations.
- [ ] **Thinking tokens** automatiquement ajoutés pour reasoning models.
- [ ] M1 Atelier expose modalités + détails techniques (mode Simple / Expert).
- [ ] M9 Fiche modèle affiche capacités + viz overhead + viz vision.
- [ ] Chaque preset cite sa source URL canonique.
- [ ] Disclaimer "estimation overhead ± 50 %" visible dans la doc + tooltip M1.
- [ ] CHANGELOG `[0.9.0] — YYYY-MM-DD — Catalogue 2026 + modalités + overhead (C34)`.
- [ ] Bump versions Cargo workspace + tauri.conf + web/package + extension/package + web-team/package : tous 0.8.0 → 0.9.0.
- [ ] Tag `v0.9.0`.

---

## 6. Anti-périmètre

- Pas de refonte du moteur Monte-Carlo.
- Pas d'output audio / vidéo (modalités output non couvertes).
- Pas de tokens caching (-50 % OpenAI feature).
- Pas de fine-tune custom UI.
- Pas de nouveaux modules (focus polish moteur).
- C33 site internet reste différé après C34.

---

## 7. Risques + mitigations

| Risque | Mitigation |
|---|---|
| Overhead système leakés inexacts → critique d'évaluateur | Disclaimer ± 50 %, valeurs P50 sourcées, choix utilisateur possible (Mode Expert) |
| GPT-5 ou Claude 4 pas encore sortis en mai 2026 → presets fantômes | Vérifier release_date avant d'inclure ; si non sortis, attendre |
| ComparIA n'a pas tous les nouveaux modèles | Compléter via HF Open LLM Leaderboard + EpochAI dataset (déjà identifié C30) |
| Calculs vision compliqués si modèle pas dans nos vendors connus | Fallback : formule générique `~500 tokens/image` avec warning |
| Trop d'UI options sur M1 → charge cognitive Student | Mode "Simple" par défaut (cache détails) + Mode "Expert" opt-in |

---

## 8. Et après v0.9.0 ?

- **C33 site internet (site-v0.1.0)** : sprint maintenant possible avec un moteur crédible.
- **v1.0 candidature data.gouv.fr** : enfin défendable scientifiquement.
- **v1.1 intégration Tier 2 datasets** (C31) : enrichissement automatique futur du catalogue.
