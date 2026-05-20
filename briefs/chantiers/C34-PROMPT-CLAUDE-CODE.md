# C34 — Prompt Claude Code (v0.9.0 — Catalogue 2026 + modalités + overhead)

> **Mode d'emploi** : copier-coller le bloc ci-dessous dans une nouvelle session Claude Code (CLI) à la racine du repo. Le prompt démarre par `/using-superpower`.

---

```
/using-superpower

# Mission : C34 — Catalogue 2026 + modalités + overhead (v0.9.0)

Tu vas livrer la version 0.9.0 de Sobr.ia, centrée sur la crédibilité
scientifique du moteur. 3 trous critiques à combler :

1. Catalogue obsolète (presets actuels = 8 modèles 2024) — il manque
   tous les modèles 2025-2026 dont Claude 4.7 Opus/Sonnet, GPT-5/5.5,
   Gemini 2.5, Llama 4, DeepSeek V3/R1, etc.
2. Pas d'overhead contextualisation (system prompt + tools + memory)
   → notre P50 sous-estime de 5× à 30× la réalité.
3. Aveugle aux modalités non-texte (images, documents, audio,
   reasoning thinking tokens).

Sprint ~6 jours. Ce chantier PRÉCÈDE C33 site internet — pas de
marketing avec un moteur obsolète.

## Contexte à charger AVANT toute action

Lis ces fichiers dans l'ordre :

1. `CLAUDE.md` — règles, anti-patterns, méthodologie scientifique §6.
2. **`briefs/chantiers/C34-catalogue-modalites-overhead.md`** — brief
   complet, source de vérité pour périmètre + DoD + sources.
3. `crates/sobria-estimator/src/model_presets.rs` — état actuel des
   8 presets 2024 à enrichir.
4. `crates/sobria-estimator/src/engines/mod.rs` + `engine_trait.rs` —
   trait EmpreinteEngine actuel.
5. `crates/sobria-core/src/` — types core (EstimationRequest, etc.).
6. `crates/sobria-core/src/vendor_disclosure.rs` (si existe, sinon
   regarder où la structure VendorDisclosure est définie suite à C32).
7. `docs/sources/AUDIT-2026-Q3.md` — audit sources (notamment §D
   vendors disclosure pour Mistral × ADEME, Google Gemini, Meta Llama).
8. `crates/sobria-ingest/src/sources/comparia.rs` — pipeline médaillon
   ComparIA qui sert les données Gold (à requêter pour catalogue).
9. `crates/sobria-estimator/src/validation/cases.rs` — exemples
   ReproductionCase à étendre pour les 5 nouvelles modalités.

## Stratégie + garde-fous

- **Catalogue scientifique sourcé** : chaque preset cite son `source_url`
  (model card officielle vendor). Zéro chiffre inventé.
- **Cutoff Cowork = fin mai 2025** : pour les modèles 2026 (Claude 4.7,
  GPT-5.5, etc.), DOIS lancer une recherche web actualisée au début de
  C34.1 pour valider versions et dates. Voir §3 du brief pour la liste
  des recherches à lancer.
- **Zéro preset fantôme** : si un modèle annoncé n'est PAS effectivement
  sorti à la date d'exécution, ne PAS l'inclure.
- **Disclaimer transparent** : pour overhead système et formules
  vision, afficher explicitement "estimation ± 50 %" + sources URLs
  dans tooltips UI.
- **Modèles 2024 deprecated** : marqués `deprecated: true` mais
  conservés dans presets (le ledger d'audit historique pointe dessus).
- **DEMANDER** si tu hésites sur :
  - L'inclusion d'un modèle dont la sortie est incertaine.
  - Formule vision exacte pour un vendor sans doc claire.
  - UI design "Simple vs Expert" mode.
  - Wording du disclaimer overhead.

## Plan d'exécution

### C34.1 — Audit modèles ComparIA Gold + recherche web actualisée (1 j)

**Étape 1 : Recherche web actualisée** (priorité absolue, à faire EN
PREMIER) :

Lance WebSearch pour valider la shortlist 2026 réelle :
1. `"Claude 4.7 Opus Sonnet release date 2026 system card"`
2. `"GPT-5 GPT-5.5 OpenAI release 2026 model card"`
3. `"latest LLM releases 2026 Q2 catalog comprehensive"`
4. `"Llama 4 Maverick Scout Meta release 2025 2026"`
5. `"DeepSeek 2026 latest model R2 V4 reasoning"`
6. `"Gemini 2.5 3.0 Google DeepMind release 2026"`
7. `"Mistral Large 3 Magistral 2026 release"`

Synthétise les résultats dans une table de référence locale (markdown
temporaire dans `briefs/chantiers/C34-shortlist-models-validated.md`)
avec colonnes : Modèle / Date sortie effective / URL source officielle
/ Active params B / Total params B / Vision / Reasoning / Notes.

**Étape 2 : Query ComparIA Gold** :

```bash
# Le pipeline médaillon a produit data/gold/referentiel.sqlite.
# Requête pour lister les modèles présents :
sqlite3 data/gold/referentiel.sqlite \
  "SELECT DISTINCT model_id, COUNT(*) AS nb_calls FROM model_overview GROUP BY model_id ORDER BY nb_calls DESC;"
```

Si Gold pas dispo, fallback : lire directement les Parquet Silver
ComparIA via `polars` ou `duckdb` Rust.

**Étape 3 : Croisement et shortlist finale** :

Croise (a) modèles ComparIA effectivement utilisés + (b) modèles
identifiés dans Étape 1 web search. Cible : 20-25 presets dont 12-15
sortis en 2025-2026.

**DoD C34.1** : `briefs/chantiers/C34-shortlist-models-validated.md`
livré avec liste exhaustive + sources URLs vérifiées.

### C34.2 — Enrichir model_presets.rs (1.5 j)

Voir brief §1 (Bloc A) et §2 (architecture types).

Étapes :

1. Étendre struct `ModelPreset` avec nouveaux champs :
   - `release_date: chrono::NaiveDate`
   - `model_family: ModelFamily` (enum à créer)
   - `architecture: ArchitectureKind` (enum à créer : DenseTransformer,
     MoE { experts, active_experts }, Mamba, Hybrid)
   - `vision_capable: bool`
   - `vision_pricing: Option<VisionPricing>`
   - `audio_capable: bool`
   - `reasoning_capable: bool`
   - `thinking_token_multiplier: Option<(f64, f64)>` (P5, P95)
   - `default_context_overhead_tokens: u32`
   - `deprecated: bool`
   - `source_url: &'static str`

2. Créer enums associés dans `model_presets.rs` ou `model_meta.rs`.

3. Marquer les 8 presets actuels avec `deprecated: true`.

4. Ajouter 15-20 nouveaux presets validés en C34.1.

5. Mettre à jour `find_preset` et `available_models` pour exposer un
   filter `include_deprecated: bool`.

6. Tests round-trip serde + tests `find_preset("claude-4-7-sonnet")`.

**DoD C34.2** : `cargo test -p sobria-estimator presets` vert, ≥ 20
presets totaux dont ≥ 12 sortis en 2025-2026.

### C34.3 — Type Modality + ContextOverhead (1.5 j)

Voir brief §2.

Étapes :

1. Créer `crates/sobria-core/src/modality.rs` avec enum `Modality` (Text,
   VisionLow, VisionHigh, Document, AudioInput).

2. Créer `crates/sobria-core/src/context_overhead.rs` avec struct
   `ContextOverhead` + helper `from_preset()`.

3. Créer enum `VisionPricing` dans `model_presets.rs` avec 4 variants
   (OpenAiTiles, AnthropicArea, GeminiNative, LlamaPatches) +
   méthode `tokens_for(count, width, height, high_detail) -> u32`.

4. Étendre `EstimationRequest` (dans sobria-core/dto.rs ou équivalent)
   avec :
   - `modalities: Vec<Modality>` (default empty = juste Text).
   - `overhead: ContextOverhead` (default = zéros, mais auto-rempli
     par preset si user ne touche pas).

5. Modifier `EmpreinteEngine::estimate` (les 2 engines AFNOR + EcoLogits)
   pour calculer `effective_input_tokens = user_tokens + overhead.total()
   + modalities.tokens_overhead(preset)`.

6. Pour reasoning models : ajouter automatiquement
   `thinking_tokens_p50 = output_tokens * (P50 du thinking_multiplier)`
   à `overhead.thinking_tokens_p50` avant calcul.

7. Tests : `Modality::tokens_overhead(preset)` pour chaque variant.

**DoD C34.3** : `cargo test -p sobria-estimator` + `cargo test -p sobria-core`
verts. Les estimations modifient effective_input_tokens correctement.

### C34.4 — UI M1 Atelier modalités + détails techniques (1 j)

Voir brief §1 Bloc C.

Étapes :

1. `web/src/lib/api.ts` : étendre les types `EstimateRequest` côté TS
   avec `modalities` + `overhead`.

2. `web/src/routes/+page.svelte` (M1 Atelier) :
   - Toggles modalités : Text (toujours) + Image + Document + Audio.
   - Si Image : champ `image_count` + radio "Basse / Haute résolution"
     + (si Haute) champ avg_width × avg_height.
   - Si Document : champ `page_count`.
   - Si Audio : champ `duration_seconds`.
   - Section repliable "Détails techniques" (`<details>` ou collapsible):
     - `system_prompt_tokens` (default = preset.default_context_overhead_tokens).
     - `tools_definition_tokens` (default 0).
     - `memory_tokens` (default 0, info-tooltip "tokens accumulés des
       tours précédents").
   - Si modèle `reasoning_capable` : badge info "Ce modèle ajoute
     automatiquement X-Y thinking tokens (P5-P95) à votre estimation".
   - Mode "Simple" par défaut (cache détails techniques) vs Mode "Expert"
     (déplie tout). Toggle dans header. Stocké en préférence utilisateur.

3. Tooltips pédagogiques sur CHAQUE champ avec lien vers source
   (OpenAI Vision pricing / Anthropic vision doc / etc.).

**DoD C34.4** : sur M1 Atelier, un utilisateur peut estimer un prompt
avec 2 images + system prompt 1500 tokens et obtenir une estimation
cohérente.

### C34.5 — UI M9 Fiche modèle capacités + viz (0.5 j)

Voir brief §1 Bloc C.

Étapes :

1. Sur fiche M9 (`web/src/routes/m9/[id]/+page.svelte` ou équivalent) :
   - Section "Capacités" : badges Vision / Audio / Reasoning / MoE,
     selon les flags du preset.
   - Section "Overhead par défaut" : table avec system, tools, memory.
   - Si `vision_capable` : viz "1 image 512×512 low = X tokens · high =
     Y tokens · 1024×1024 high = Z tokens" calculé via preset.vision_pricing.
   - Si `reasoning_capable` : viz "ratio thinking/output P5-P50-P95
     = N×-M×-K×" + cite source.
   - Toutes les valeurs cliquables ouvrent la source URL dans nouvel onglet.

**DoD C34.5** : M9 fiche pour `claude-4-7-sonnet` (ou équivalent) affiche
toutes ses capacités avec sources.

### C34.6 — 5 ReproductionCase + smoke E2E + ship v0.9.0 (0.5 j)

Voir brief §1 Bloc D.

Étapes :

1. Étendre `crates/sobria-estimator/src/validation/cases.rs` avec 5
   nouveaux cas :
   - `vision_gpt4o_high_2_images` : prompt + 2 images 1024×1024 high
     → tokens attendus + estimation P50/P5/P95 documentée.
   - `document_5_pages` : prompt + PDF 5 pages → tokens attendus.
   - `audio_30_seconds` : prompt + 30 s audio → tokens attendus.
   - `reasoning_o3_complex` : prompt complexe sur reasoning model
     → tokens output + thinking attendus.
   - `overhead_claude_default` : prompt simple sur Claude 3.7 avec
     overhead par défaut → vérifier que les ~1500 tokens system sont
     bien comptés.

2. Smoke test E2E manuel : ouvrir M1 Atelier, essayer chaque modalité,
   vérifier que les résultats sont cohérents.

3. CHANGELOG entrée `[0.9.0] — YYYY-MM-DD — Catalogue 2026 +
   modalités + overhead (C34)`.

4. Bump versions Cargo workspace + tauri.conf + web/package +
   extension/package + manifest + web-team/package : tous 0.8.0 → 0.9.0.

5. Tag `v0.9.0` :

```bash
git tag -a v0.9.0 -m "v0.9.0 — Catalogue 2026 + modalités + overhead (C34)

Catalogue moteur enrichi avec les modèles 2025-2026 (Claude 4.7
Opus/Sonnet, GPT-5/5.5, Gemini 2.5, Llama 4, DeepSeek V3/R1,
Qwen 3, Mistral Large 3, etc.) — au total ≥ 20 presets dont ≥ 12
sortis en 2025-2026. Anciens modèles 2024 marqués deprecated.

5 modalités supportées : Text, VisionLow, VisionHigh, Document,
AudioInput. Formules tokens vision sourcées OpenAI / Anthropic /
Google / Meta.

ContextOverhead modélise system prompt + tools + memory + thinking
tokens (pour reasoning models). Disclaimer transparent ± 50 % avec
sources publiques.

UI M1 enrichie (modalités + détails techniques mode Simple/Expert).
UI M9 affiche capacités + viz vision + viz reasoning.

5 nouveaux ReproductionCase. Tag précède C33 site internet."
```

## DoD globale

- [ ] `cargo test --workspace` 100 % vert (5 nouveaux cases inclus).
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cd web && npm run check && npm run lint && npm run test` propre.
- [ ] ≥ 20 presets totaux dont ≥ 12 sortis 2025-2026.
- [ ] Zéro preset fantôme (modèles non sortis exclus).
- [ ] 5 modalités supportées (Text + Vision Low + Vision High + Document + Audio).
- [ ] ContextOverhead appliqué à toutes les estimations.
- [ ] Thinking tokens auto-ajoutés pour reasoning models.
- [ ] UI M1 expose modalités + détails techniques (Simple/Expert).
- [ ] UI M9 affiche capacités + sources.
- [ ] Chaque preset cite `source_url` officielle.
- [ ] Disclaimer "± 50 %" visible dans tooltips + doc.
- [ ] CHANGELOG `[0.9.0]` complet.
- [ ] Bump versions partout (Cargo + tauri.conf + web + extension + web-team).
- [ ] Tag `v0.9.0`.

## Convention de commit

```
feat(estimator): C34.1 audit modèles ComparIA + recherche web shortlist 2026 validée
feat(estimator): C34.2 enrichir model_presets.rs (≥ 20 presets, dont 12+ 2025-2026)
feat(core,estimator): C34.3 type Modality + ContextOverhead + VisionPricing
feat(web): C34.4 UI M1 modalités + détails techniques (mode Simple/Expert)
feat(web): C34.5 UI M9 capacités + viz vision + viz reasoning
test(estimator): C34.6 5 ReproductionCase modalités + ship v0.9.0
chore(release): bump v0.9.0
```

## Garde-fous

- **JAMAIS** d'invention de chiffre sans source publique citée.
- **JAMAIS** de preset fantôme (modèle non sorti).
- **JAMAIS** retirer les anciens presets 2024 (marquer deprecated mais
  conserver, le ledger audit historique en dépend).
- **TOUJOURS** disclaimer "± 50 %" sur overhead estimé.
- **TOUJOURS** lien source dans tooltip UI + commentaire code.
- **TOUJOURS** vérifier via WebSearch les versions effectives 2026
  AVANT de coder les presets.
- **DEMANDER** si :
  - Un modèle a une sortie incertaine.
  - Une formule vision n'a pas de source publique claire.
  - Le wording du disclaimer overhead t'inspire pas.

Bonne mission. Commence ABSOLUMENT par C34.1 (recherche web actualisée
+ audit ComparIA Gold) — c'est la base de tout le reste.
```

---

## Notes pour Thibault

- Sprint ~6 jours. Phase critique pour la crédibilité scientifique.
- Au retour : `git log --oneline -10` + on review avant tag v0.9.0.
- Smoke test critique : ouvre M1 Atelier sur `claude-4-7-sonnet`
  (ou la version 2026 la plus récente), envoie un prompt avec 1 image
  haute résolution, vérifie que l'estimation prend bien en compte les
  ~1500 tokens system + ~1500 tokens image + tokens reasoning si modèle
  reasoning.
- Pour la **validation manuelle** : compare l'estimation Sobr.ia à
  la facturation API réelle d'OpenAI / Anthropic / Google sur ton
  compte pro pour 2-3 cas. Si écart > 30 %, alerte avant de tag.
- Une fois v0.9.0 shippée → **C33 site internet** avec un produit
  enfin crédible scientifiquement.
