// Sobr.ia extension — registry de presets modèles (C27.2, étendu C34.2).
//
// Mirror du registry Rust `crates/sobria-estimator/src/model_presets.rs::MODEL_REGISTRY`.
// Version affichée UI : `registry-meta.ts` (= `package.json`). Cohorte actuelle :
// 33 presets : 25 actifs (2025-2026) + 8 conservés
// `deprecated` (2024) pour parité tests historiques (cf.
// `tests/unit/empreinte.spec.ts`) et reproductibilité de l'audit ledger.
//
// Les valeurs P50 (ε_prefill, ε_decode, embodied) sont copiées 1-pour-1 du
// triplet `(P5, P50, P95)` Rust : `epsilon_*_mj.1` et `embodied_g_per_req.1`.
// La calibration suit AFNOR SPEC 2314 — `epsilon_decode = K_DECODE × paramsB`
// avec `K_DECODE = 25 mJ/token/B` et `epsilon_prefill ≈ 0.4 × epsilon_decode`.
// Pour les MoE, `paramsBillion` = paramètres ACTIFS (= ce qui domine le coût
// d'inférence ; cf. notes Rust `llama-4-maverick`, `mistral-large-3`).
//
// **Toute évolution doit rester synchronisée avec le registry Rust** pour
// préserver la parité ≤ 2 % vérifiée en C27.2 (`empreinte.spec.ts`).

import type { ModelPreset } from './types.js';

export const MODEL_PRESETS: readonly ModelPreset[] = [
  // ════════════════════════════════════════════════════════════════════════
  // C34.2 — Catalogue 2025-2026 (actif)
  // ════════════════════════════════════════════════════════════════════════

  // ─── Anthropic ────────────────────────────────────────────────────────
  {
    id: 'claude-opus-4-8',
    displayName: 'Claude Opus 4.8',
    vendor: 'Anthropic',
    family: 'claude-4',
    paramsBillion: 2000.0,
    epsilonPrefillMjPerToken: 20000.0,
    epsilonDecodeMjPerToken: 50000.0,
    embodiedGPerRequest: 0.5,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Anthropic Claude Opus 4.8 release (2026-05)', 'Estimation taille publique 2026']
  },
  {
    id: 'claude-opus-4-7',
    displayName: 'Claude Opus 4.7',
    vendor: 'Anthropic',
    family: 'claude-4',
    paramsBillion: 2000.0,
    epsilonPrefillMjPerToken: 20000.0,
    epsilonDecodeMjPerToken: 50000.0,
    embodiedGPerRequest: 0.5,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Anthropic Claude Opus 4.7 release (2026-04-16)', 'Estimation taille publique 2026']
  },
  {
    id: 'claude-sonnet-4-6',
    displayName: 'Claude Sonnet 4.6',
    vendor: 'Anthropic',
    family: 'claude-4',
    paramsBillion: 400.0,
    epsilonPrefillMjPerToken: 4000.0,
    epsilonDecodeMjPerToken: 10000.0,
    embodiedGPerRequest: 0.1,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Anthropic Claude Sonnet 4.6 release (2026-02-17)', 'Estimation taille publique 2026']
  },
  {
    id: 'claude-haiku-4-5',
    displayName: 'Claude Haiku 4.5',
    vendor: 'Anthropic',
    family: 'claude-4',
    paramsBillion: 70.0,
    epsilonPrefillMjPerToken: 700.0,
    epsilonDecodeMjPerToken: 1750.0,
    embodiedGPerRequest: 0.0175,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Anthropic Claude Haiku 4.5 release (2025-10)', 'Estimation taille publique 2025']
  },
  {
    id: 'claude-opus-4',
    displayName: 'Claude Opus 4',
    vendor: 'Anthropic',
    family: 'claude-4',
    paramsBillion: 1500.0,
    epsilonPrefillMjPerToken: 15000.0,
    epsilonDecodeMjPerToken: 37500.0,
    embodiedGPerRequest: 0.375,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Anthropic Claude Opus 4 release (2025-05)', 'Estimation taille publique 2025']
  },
  {
    id: 'claude-sonnet-4',
    displayName: 'Claude Sonnet 4',
    vendor: 'Anthropic',
    family: 'claude-4',
    paramsBillion: 400.0,
    epsilonPrefillMjPerToken: 4000.0,
    epsilonDecodeMjPerToken: 10000.0,
    embodiedGPerRequest: 0.1,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Anthropic Claude Sonnet 4 release (2025-05)', 'Estimation taille publique 2025']
  },
  {
    id: 'claude-3-7-sonnet',
    displayName: 'Claude 3.7 Sonnet',
    vendor: 'Anthropic',
    family: 'claude-3',
    paramsBillion: 200.0,
    epsilonPrefillMjPerToken: 2000.0,
    epsilonDecodeMjPerToken: 5000.0,
    embodiedGPerRequest: 0.05,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Anthropic Claude 3.7 Sonnet release (2025-02)', 'EcoLogits 2026-01']
  },

  // ─── OpenAI ───────────────────────────────────────────────────────────
  {
    id: 'gpt-5-5',
    displayName: 'GPT-5.5',
    vendor: 'OpenAI',
    family: 'gpt-5',
    paramsBillion: 1000.0,
    epsilonPrefillMjPerToken: 10000.0,
    epsilonDecodeMjPerToken: 25000.0,
    embodiedGPerRequest: 0.25,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['OpenAI GPT-5.5 release (2026-04-23)', 'Estimation taille publique 2026']
  },
  {
    id: 'gpt-5-5-thinking',
    displayName: 'GPT-5.5 Thinking',
    vendor: 'OpenAI',
    family: 'gpt-5',
    paramsBillion: 1000.0,
    epsilonPrefillMjPerToken: 10000.0,
    epsilonDecodeMjPerToken: 25000.0,
    embodiedGPerRequest: 0.25,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['OpenAI GPT-5.5 Thinking release (2026-04-23)', 'Estimation taille publique 2026']
  },
  {
    id: 'gpt-5-5-pro',
    displayName: 'GPT-5.5 Pro',
    vendor: 'OpenAI',
    family: 'gpt-5',
    paramsBillion: 1000.0,
    epsilonPrefillMjPerToken: 10000.0,
    epsilonDecodeMjPerToken: 25000.0,
    embodiedGPerRequest: 0.25,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['OpenAI GPT-5.5 Pro release (2026-04-23)', 'Estimation taille publique 2026']
  },
  {
    id: 'o3',
    displayName: 'o3',
    vendor: 'OpenAI',
    family: 'o-series',
    paramsBillion: 400.0,
    epsilonPrefillMjPerToken: 4000.0,
    epsilonDecodeMjPerToken: 10000.0,
    embodiedGPerRequest: 0.1,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['OpenAI o3 release (2025-04)', 'Estimation taille publique 2025']
  },

  // ─── Google DeepMind ─────────────────────────────────────────────────
  {
    id: 'gemini-3-5-flash',
    displayName: 'Gemini 3.5 Flash',
    vendor: 'Google',
    family: 'gemini-3',
    paramsBillion: 32.0,
    epsilonPrefillMjPerToken: 320.0,
    epsilonDecodeMjPerToken: 800.0,
    embodiedGPerRequest: 0.008,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Google Gemini 3.5 Flash release (2026-05)', 'Estimation taille publique 2026']
  },
  {
    id: 'gemini-3-1-pro',
    displayName: 'Gemini 3.1 Pro',
    vendor: 'Google',
    family: 'gemini-3',
    paramsBillion: 400.0,
    epsilonPrefillMjPerToken: 4000.0,
    epsilonDecodeMjPerToken: 10000.0,
    embodiedGPerRequest: 0.1,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Google Gemini 3.1 Pro release (2026-03)', 'Estimation taille publique 2026']
  },
  {
    id: 'gemini-2-5-pro',
    displayName: 'Gemini 2.5 Pro',
    vendor: 'Google',
    family: 'gemini-2',
    paramsBillion: 400.0,
    epsilonPrefillMjPerToken: 4000.0,
    epsilonDecodeMjPerToken: 10000.0,
    embodiedGPerRequest: 0.1,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Google Gemini 2.5 Pro release (2025-06)', 'Estimation taille publique 2025']
  },

  // ─── Meta AI ─────────────────────────────────────────────────────────
  {
    id: 'llama-4-scout',
    displayName: 'Llama 4 Scout',
    vendor: 'Meta',
    family: 'llama-4',
    paramsBillion: 17.0,
    epsilonPrefillMjPerToken: 170.0,
    epsilonDecodeMjPerToken: 425.0,
    embodiedGPerRequest: 0.00425,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'moe',
    sources: ['Meta Llama 4 Scout (MoE 109B / 17B actifs)', 'Meta Llama 4 release (2025-04-05)']
  },
  {
    id: 'llama-4-maverick',
    displayName: 'Llama 4 Maverick',
    vendor: 'Meta',
    family: 'llama-4',
    paramsBillion: 17.0,
    epsilonPrefillMjPerToken: 170.0,
    epsilonDecodeMjPerToken: 425.0,
    embodiedGPerRequest: 0.00425,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'moe',
    sources: ['Meta Llama 4 Maverick (MoE 400B / 17B actifs)', 'Meta Llama 4 release (2025-04-05)']
  },
  {
    id: 'llama-3-3-70b',
    displayName: 'Llama 3.3 70B',
    vendor: 'Meta',
    family: 'llama-3',
    paramsBillion: 70.0,
    epsilonPrefillMjPerToken: 700.0,
    epsilonDecodeMjPerToken: 1750.0,
    embodiedGPerRequest: 0.0175,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Meta Llama 3.3 70B release (2024-12)', 'HF AI Energy Score 2026']
  },

  // ─── Mistral AI ──────────────────────────────────────────────────────
  {
    id: 'mistral-medium-3-5',
    displayName: 'Mistral Medium 3.5',
    vendor: 'Mistral AI',
    family: 'mistral-medium',
    paramsBillion: 128.0,
    epsilonPrefillMjPerToken: 1280.0,
    epsilonDecodeMjPerToken: 3200.0,
    embodiedGPerRequest: 0.032,
    defaultRegion: 'FR',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Mistral Medium 3.5 release (2025)', 'Mistral × ADEME 2024-2025']
  },
  {
    id: 'mistral-small-4',
    displayName: 'Mistral Small 4',
    vendor: 'Mistral AI',
    family: 'mistral-small',
    paramsBillion: 30.0,
    epsilonPrefillMjPerToken: 300.0,
    epsilonDecodeMjPerToken: 750.0,
    embodiedGPerRequest: 0.0075,
    defaultRegion: 'FR',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Mistral Small 4 release (2025)', 'Mistral × ADEME 2024-2025']
  },
  {
    id: 'mistral-large-3',
    displayName: 'Mistral Large 3',
    vendor: 'Mistral AI',
    family: 'mistral-large',
    paramsBillion: 41.0,
    epsilonPrefillMjPerToken: 410.0,
    epsilonDecodeMjPerToken: 1025.0,
    embodiedGPerRequest: 0.01025,
    defaultRegion: 'FR',
    defaultPue: 1.2,
    architectureFamily: 'moe',
    sources: ['Mistral Large 3 (MoE 675B / 41B actifs)', 'Mistral × ADEME 2024-2025']
  },

  // ─── DeepSeek ────────────────────────────────────────────────────────
  {
    id: 'deepseek-v4-pro',
    displayName: 'DeepSeek V4 Pro',
    vendor: 'DeepSeek',
    family: 'deepseek-v4',
    paramsBillion: 49.0,
    epsilonPrefillMjPerToken: 490.0,
    epsilonDecodeMjPerToken: 1225.0,
    embodiedGPerRequest: 0.01225,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'moe',
    sources: ['DeepSeek V4 Pro (MoE 1600B / 49B actifs)', 'DeepSeek tech report 2025']
  },
  {
    id: 'deepseek-r1',
    displayName: 'DeepSeek R1',
    vendor: 'DeepSeek',
    family: 'deepseek-r1',
    paramsBillion: 37.0,
    epsilonPrefillMjPerToken: 370.0,
    epsilonDecodeMjPerToken: 925.0,
    embodiedGPerRequest: 0.00925,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'moe',
    sources: ['DeepSeek R1 (MoE 671B / 37B actifs)', 'DeepSeek tech report 2025']
  },

  // ─── xAI ─────────────────────────────────────────────────────────────
  {
    id: 'grok-4',
    displayName: 'Grok 4',
    vendor: 'xAI',
    family: 'grok',
    paramsBillion: 500.0,
    epsilonPrefillMjPerToken: 5000.0,
    epsilonDecodeMjPerToken: 12500.0,
    embodiedGPerRequest: 0.125,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['xAI Grok 4 release (2025)', 'Estimation taille publique 2025']
  },

  // ─── Alibaba ─────────────────────────────────────────────────────────
  {
    id: 'qwen-3-6-plus',
    displayName: 'Qwen 3.6 Plus',
    vendor: 'Alibaba',
    family: 'qwen-3',
    paramsBillion: 1000.0,
    epsilonPrefillMjPerToken: 10000.0,
    epsilonDecodeMjPerToken: 25000.0,
    embodiedGPerRequest: 0.25,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Alibaba Qwen 3.6 Plus release (2025)', 'Estimation taille publique 2025']
  },

  // ─── Microsoft ───────────────────────────────────────────────────────
  {
    id: 'phi-4-reasoning-vision',
    displayName: 'Phi-4 Reasoning Vision',
    vendor: 'Microsoft',
    family: 'phi-4',
    paramsBillion: 15.0,
    epsilonPrefillMjPerToken: 150.0,
    epsilonDecodeMjPerToken: 375.0,
    embodiedGPerRequest: 0.00375,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Microsoft Phi-4 Reasoning Vision release (2025)', 'Microsoft Phi tech report']
  },
  {
    id: 'phi-4-reasoning',
    displayName: 'Phi-4 Reasoning',
    vendor: 'Microsoft',
    family: 'phi-4',
    paramsBillion: 14.0,
    epsilonPrefillMjPerToken: 140.0,
    epsilonDecodeMjPerToken: 350.0,
    embodiedGPerRequest: 0.0035,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Microsoft Phi-4 Reasoning release (2025)', 'Microsoft Phi tech report']
  },

  // ════════════════════════════════════════════════════════════════════════
  // Presets 2024 — conservés pour parité tests C27.2 + reproductibilité
  // audit ledger historique. Côté Rust : `deprecated: true`.
  // ════════════════════════════════════════════════════════════════════════
  {
    id: 'gpt-4o',
    displayName: 'GPT-4o',
    vendor: 'OpenAI',
    family: 'gpt-4',
    paramsBillion: 200.0,
    // 200 × 25 = 5000 mJ/tok decode, 200 × 25 × 0.4 = 2000 mJ/tok prefill
    epsilonPrefillMjPerToken: 2000.0,
    epsilonDecodeMjPerToken: 5000.0,
    embodiedGPerRequest: 0.05,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: [
      'EcoLogits 2026-01',
      'HF AI Energy Score 2026',
      'Estimation taille — analyse publique 2024'
    ]
  },
  {
    id: 'gpt-4o-mini',
    displayName: 'GPT-4o mini',
    vendor: 'OpenAI',
    family: 'gpt-4',
    paramsBillion: 8.0,
    // 8 × 25 = 200 mJ/tok decode
    epsilonPrefillMjPerToken: 80.0,
    epsilonDecodeMjPerToken: 200.0,
    embodiedGPerRequest: 0.002,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['EcoLogits 2026-01', 'Estimation taille — analyse publique 2024']
  },
  {
    id: 'claude-3-5-sonnet',
    displayName: 'Claude 3.5 Sonnet',
    vendor: 'Anthropic',
    family: 'claude-3',
    paramsBillion: 200.0,
    epsilonPrefillMjPerToken: 2000.0,
    epsilonDecodeMjPerToken: 5000.0,
    embodiedGPerRequest: 0.05,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['EcoLogits 2026-01 (analogie modèles dense ~200B)']
  },
  {
    id: 'mistral-large-2',
    displayName: 'Mistral Large 2',
    vendor: 'Mistral AI',
    family: 'mistral-large',
    paramsBillion: 123.0,
    // 123 × 25 = 3075
    epsilonPrefillMjPerToken: 1230.0,
    epsilonDecodeMjPerToken: 3075.0,
    embodiedGPerRequest: 0.03075,
    defaultRegion: 'FR',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Mistral AI tech report 2024', 'HF AI Energy Score 2026', 'EcoLogits 2026-01']
  },
  {
    id: 'mistral-medium-3',
    displayName: 'Mistral Medium 3',
    vendor: 'Mistral AI',
    family: 'mistral-medium',
    paramsBillion: 30.0,
    // 30 × 25 = 750
    epsilonPrefillMjPerToken: 300.0,
    epsilonDecodeMjPerToken: 750.0,
    embodiedGPerRequest: 0.0075,
    defaultRegion: 'FR',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Mistral AI 2024', 'EcoLogits 2026-01']
  },
  {
    id: 'llama-3-1-70b',
    displayName: 'Llama 3.1 70B',
    vendor: 'Meta',
    family: 'llama-3',
    paramsBillion: 70.0,
    // 70 × 25 = 1750
    epsilonPrefillMjPerToken: 700.0,
    epsilonDecodeMjPerToken: 1750.0,
    embodiedGPerRequest: 0.0175,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Meta Llama 3.1 paper (Touvron et al. 2024)', 'HF AI Energy Score 2026']
  },
  {
    id: 'llama-3-1-8b',
    displayName: 'Llama 3.1 8B',
    vendor: 'Meta',
    family: 'llama-3',
    paramsBillion: 8.0,
    // 8 × 25 = 200
    epsilonPrefillMjPerToken: 80.0,
    epsilonDecodeMjPerToken: 200.0,
    embodiedGPerRequest: 0.002,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Meta Llama 3.1 paper (Touvron et al. 2024)', 'HF AI Energy Score 2026']
  },
  {
    id: 'gemini-2-0-flash',
    displayName: 'Gemini 2.0 Flash',
    vendor: 'Google',
    family: 'gemini-2',
    paramsBillion: 32.0,
    // 32 × 25 = 800
    epsilonPrefillMjPerToken: 320.0,
    epsilonDecodeMjPerToken: 800.0,
    embodiedGPerRequest: 0.008,
    defaultRegion: 'US-VA',
    defaultPue: 1.2,
    architectureFamily: 'dense',
    sources: ['Google DeepMind 2025 (annonce publique)', 'Analogie Mistral Medium']
  }
];

/**
 * Cherche un preset par identifiant exact.
 *
 * Retourne `undefined` si le modèle n'est pas dans le registry (l'appelant
 * doit décider du fallback — défaut UI ou rejet de l'estimation).
 */
export function findPreset(modelId: string): ModelPreset | undefined {
  return MODEL_PRESETS.find((p) => p.id === modelId);
}

/** Liste tous les presets disponibles (pour sélecteur popup). */
export function availablePresets(): readonly ModelPreset[] {
  return MODEL_PRESETS;
}
