// Sobr.ia extension — moteur AFNOR SPEC 2314 (Sobr.ia), point-estimate (C27.2).
//
// Port JS simplifié de `crates/sobria-estimator/src/engine.rs::MonteCarloEngine` :
// l'extension fonctionne en point-estimate sur les P50 des distributions
// (epsilon_prefill, epsilon_decode, embodied) au lieu du Monte-Carlo N=10⁴.
// Le moteur Monte-Carlo complet reste côté app desktop ; l'extension privilégie
// la légèreté (cf. brief C27.2 §"Port JS du moteur").
//
// **Sources scientifiques** (cf. registry Rust, code identique) :
// - Formule : AFNOR SPEC 2314 (référentiel français mesure empreinte LLM)
// - Constante K_DECODE = 25 mJ/token/B recalibrée audit B (ADR-0012 §Recalibration)
// - Sources HF AI Energy Score 2026, EcoLogits 2026-01, ML.ENERGY (alignement
//   1.75 J/token decode pour Llama 3.1 70B ≈ 25 × 70 mJ/token)
//
// Parité ≤ 2 % vs Rust vérifiée dans tests/unit/empreinte.spec.ts.

import type { Estimate, EstimateInput } from '../types.js';
import { REGION_DEFAULTS } from '../types.js';
import { findPreset } from '../presets.js';
import { computeEquivalents } from './equivalents.js';

/**
 * Lance l'estimation AFNOR pour une entrée extension.
 *
 * Formule (point-estimate sur les P50 du preset) :
 *
 *   E_compute_mJ = tokensIn × ε_prefill_P50 + tokensOut × ε_decode_P50
 *   E_total_Wh   = E_compute_mJ × PUE / 3_600_000
 *   CO₂eq_g_usage = (E_total_Wh / 1000) × IF_gCO₂eq_per_kWh
 *   CO₂eq_g_total = CO₂eq_g_usage + embodied_g_per_request
 *   water_L       = (E_total_Wh / 1000) × WUE_L_per_kWh
 *
 * Lève une erreur si `input.modelId` n'est pas dans le registry.
 */
export function estimateAfnor(input: EstimateInput): Estimate {
  const preset = findPreset(input.modelId);
  if (!preset) {
    throw new Error(`AFNOR: modèle inconnu "${input.modelId}" — registry Sobr.ia`);
  }
  if (input.tokensIn < 0 || input.tokensOut < 0) {
    throw new Error('AFNOR: tokensIn et tokensOut doivent être ≥ 0');
  }

  const region = input.region ?? preset.defaultRegion;
  const defaults = REGION_DEFAULTS[region];
  const pue = input.pue ?? defaults.pue;
  const ifElec = input.ifGramPerKwh ?? defaults.ifGramPerKwh;
  const wue = input.wueLPerKwh ?? defaults.wueLPerKwh;

  const epsPrefill = preset.epsilonPrefillMjPerToken;
  const epsDecode = preset.epsilonDecodeMjPerToken;

  // E_compute en mJ, puis Wh : 1 Wh = 3 600 J = 3 600 000 mJ.
  // Donc E_total_Wh = (E_compute_mJ × PUE) / 3 600 000.
  const eComputeMj = input.tokensIn * epsPrefill + input.tokensOut * epsDecode;
  const eTotalWh = (eComputeMj * pue) / 3_600_000.0;

  // CO₂eq usage (g) = (E_total_Wh / 1000) × IF (g/kWh)
  const usageCo2G = (eTotalWh / 1000.0) * ifElec;

  const embodiedCo2G = input.disableEmbodied ? 0 : preset.embodiedGPerRequest;
  const totalCo2G = usageCo2G + embodiedCo2G;

  // Eau (mL) = (E_total_Wh / 1000) × WUE (L/kWh) × 1000
  const waterMl = (eTotalWh / 1000.0) * wue * 1000.0;

  const notes: string[] = [
    'AFNOR SPEC 2314 — Sobr.ia (point-estimate P50)',
    `Modèle ${preset.displayName}, ε_decode=${epsDecode} mJ/token (K_DECODE=25 × ${preset.paramsBillion} G)`,
    `PUE=${pue}, IF=${ifElec} gCO₂eq/kWh (${region}), WUE=${wue} L/kWh`
  ];
  if (input.disableEmbodied) {
    notes.push('Embodied désactivé (mode usage-only)');
  } else {
    notes.push(`Embodied amorti = ${preset.embodiedGPerRequest} gCO₂eq/req`);
  }

  return {
    method: 'afnor_sobria',
    modelId: input.modelId,
    tokensIn: input.tokensIn,
    tokensOut: input.tokensOut,
    gco2eq: totalCo2G,
    gco2eqUsage: usageCo2G,
    gco2eqEmbodied: embodiedCo2G,
    waterMl,
    energyWh: eTotalWh,
    equivalents: computeEquivalents({ gco2eq: totalCo2G, energyWh: eTotalWh }),
    notes
  };
}
