// Sobr.ia extension — registry de presets modèles (C27.2).
//
// Mirror du registry Rust `crates/sobria-estimator/src/model_presets.rs::MODEL_REGISTRY`.
// 8 modèles couvrant les 3 sites surveillés (ChatGPT/Claude/Le Chat) + extra
// pour comparaisons. Les valeurs P50 viennent du registry Rust qui calibre
// epsilon_decode = K_DECODE × paramsBillion avec K_DECODE = 25 mJ/token/B
// (recalibration C24, cf. ADR-0012 §"Recalibration AFNOR").
//
// **Toute évolution doit rester synchronisée avec le registry Rust** pour
// préserver la parité ≤ 2 % vérifiée en C27.2 tests/unit/empreinte.spec.ts.

import type { ModelPreset } from './types.js';

export const MODEL_PRESETS: readonly ModelPreset[] = [
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
