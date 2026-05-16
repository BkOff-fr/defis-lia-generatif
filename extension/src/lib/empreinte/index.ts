// Sobr.ia extension — façade EmpreinteEngine (C27.2).
//
// Point d'entrée unique pour les content scripts + popup. Dispatch vers le
// moteur AFNOR ou EcoLogits selon `input.method`. Mirror simplifié de
// `crates/sobria-estimator/src/engines/factory.rs::engine_for`.

import type { Estimate, EstimateInput, EmpreinteMethod } from '../types.js';
import { estimateAfnor } from './afnor.js';
import { estimateEcoLogits } from './ecologits.js';

export { estimateAfnor } from './afnor.js';
export { estimateEcoLogits } from './ecologits.js';
export type { Estimate, EstimateInput, EmpreinteMethod } from '../types.js';

/**
 * Lance une estimation point-estimate selon la méthodologie demandée.
 *
 * Lève une erreur si :
 * - `input.modelId` n'est pas dans `presets.ts`
 * - `tokensIn` ou `tokensOut` est négatif
 * - `input.method` n'est pas reconnue (typage TypeScript empêche ce cas
 *   en pratique — utile pour les appelants JS pur)
 */
export function estimate(input: EstimateInput): Estimate {
  switch (input.method) {
    case 'afnor_sobria':
      return estimateAfnor(input);
    case 'ecologits':
      return estimateEcoLogits(input);
    default: {
      // Garde-fou exhaustif TypeScript.
      const exhaustive: never = input.method;
      throw new Error(`Méthodologie inconnue : ${String(exhaustive)}`);
    }
  }
}

/**
 * Liste des méthodologies disponibles (utilisé par le toggle popup C27.4).
 */
export const AVAILABLE_METHODS: readonly EmpreinteMethod[] = ['afnor_sobria', 'ecologits'];

/** Métadonnées d'affichage pour chaque méthodologie (mirror MethodologyInfo Rust). */
export const METHOD_INFO: Record<
  EmpreinteMethod,
  { displayName: string; shortDescription: string; license: string; doi?: string }
> = {
  afnor_sobria: {
    displayName: 'AFNOR SPEC 2314 — Sobr.ia',
    shortDescription:
      "Référentiel français de mesure d'empreinte LLM. Point-estimate sur " +
      'les P50 des coefficients distributionnels (Monte-Carlo côté app).',
    license: 'AFNOR SPEC publique ; code Sobr.ia MIT'
  },
  ecologits: {
    displayName: 'EcoLogits 2026-01',
    shortDescription:
      'Méthodologie open peer-reviewed (Rincé & Banse, JOSS 2025). Port ' +
      'direct des formules f_E, f_L, n_GPU.',
    license: 'CC BY-SA 4.0 (méthodologie) ; code Sobr.ia MIT',
    doi: '10.21105/joss.07471'
  }
};
