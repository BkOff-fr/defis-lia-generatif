// Sobr.ia extension — tests parité moteur JS vs Rust (C27.2).
//
// Mirror des `ReproductionCase` Rust dans
// `crates/sobria-estimator/src/validation/cases.rs::REPRODUCTION_CASES`.
//
// Les valeurs attendues sont les **cibles Python** (notebook/validation.qmd),
// reproduites par le port Rust à ≤ 1 %. Le port JS doit reproduire les mêmes
// cibles à ≤ 2 % (brief C27.2 §DoD).
//
// Si ces tests échouent, deux raisons probables :
// 1. Une constante diverge entre src/lib/empreinte/ecologits.ts et
//    crates/sobria-estimator/src/engines/ecologits.rs.
// 2. Un preset (`paramsBillion`) a été modifié dans src/lib/presets.ts
//    sans le répercuter dans le registry Rust.
//
// Avant de relâcher la tolérance, **toujours** vérifier la cohérence avec
// le code Rust commit-hash documenté ci-dessous.

import { describe, it, expect } from 'vitest';

import { estimate, estimateAfnor, estimateEcoLogits } from '../../src/lib/empreinte/index.js';
import {
  nGpu,
  fEnergyPerTokenWh,
  fLatencyPerTokenSec,
  requestEnergyKwh
} from '../../src/lib/empreinte/ecologits.js';
import { estimateAfnor as _afnor } from '../../src/lib/empreinte/afnor.js';
import { findPreset, MODEL_PRESETS } from '../../src/lib/presets.js';

// ───────────────────────────────────────────────────────────────────────────────
// Cibles Python (notebook/validation.qmd, accédé 2026-05-13).
// Identiques aux constantes embedded en commentaire dans
// crates/sobria-estimator/src/validation/cases.rs (ligne 154-203).
// ───────────────────────────────────────────────────────────────────────────────

const REPRODUCTION_CASES = [
  {
    id: 'ecologits-llama-70b-fr-short',
    modelId: 'llama-3-1-70b',
    tokensIn: 100,
    tokensOut: 500,
    ifGramPerKwh: 56.0, // FR (ADEME 2024)
    pue: 1.2,
    expectedP50G: 0.01843
  },
  {
    id: 'ecologits-llama-70b-us-long',
    modelId: 'llama-3-1-70b',
    tokensIn: 100,
    tokensOut: 2000,
    ifGramPerKwh: 412.0, // US-VA
    pue: 1.2,
    expectedP50G: 0.542
  },
  {
    id: 'ecologits-mistral-large-us-medium',
    modelId: 'mistral-large-2',
    tokensIn: 100,
    tokensOut: 1000,
    ifGramPerKwh: 412.0,
    pue: 1.2,
    expectedP50G: 0.378
  }
] as const;

// Tolérance brief C27.2 : ≤ 2 % entre JS et Rust. Les cibles Python sont
// matchées par Rust à ≤ 1 %, donc JS vs Python à ≤ 2 % implique JS vs Rust
// à ≤ 2 % par construction (port direct + même flottants).
const PARITY_TOLERANCE = 0.02;

// ───────────────────────────────────────────────────────────────────────────────

describe('EcoLogits — fonctions intermédiaires', () => {
  it('n_gpu(70) = 4 (Llama 3.1 70B : 168 GB → ceil(168/80)=3 → next_pow2=4)', () => {
    expect(nGpu(70.0)).toBe(4);
  });

  it('n_gpu(8) = 1 (Llama 3.1 8B : 19.2 GB)', () => {
    expect(nGpu(8.0)).toBe(1);
  });

  it('n_gpu(123) = 4 (Mistral Large 2 : 295.2 GB → ceil=4 → next_pow2=4)', () => {
    expect(nGpu(123.0)).toBe(4);
  });

  it('n_gpu(200) = 8 (GPT-4o : 480 GB → ceil=6 → next_pow2=8)', () => {
    expect(nGpu(200.0)).toBe(8);
  });

  it('f_energy_per_token(70) ≈ 8.05e-5 Wh', () => {
    const v = fEnergyPerTokenWh(70.0);
    expect(Math.abs(v - 8.05e-5)).toBeLessThan(1e-7);
  });

  it('f_latency_per_token(70) ≈ 0.0869 s', () => {
    const v = fLatencyPerTokenSec(70.0);
    expect(Math.abs(v - 0.0869)).toBeLessThan(1e-3);
  });

  it('request_energy_kwh(70, 500, 1.2) ≈ 3.29e-4 kWh (Llama 70B / 500 tok / PUE 1.2)', () => {
    const v = requestEnergyKwh(70.0, 500, 1.2);
    // Cible notebook Python : ~3.29e-4 kWh (0.329 Wh).
    expect(Math.abs(v - 3.29e-4)).toBeLessThan(3.29e-4 * PARITY_TOLERANCE);
  });
});

describe('EcoLogits — ReproductionCase parité JS vs Python/Rust (≤ 2 %)', () => {
  for (const tc of REPRODUCTION_CASES) {
    it(`${tc.id} : usage P50 = ${tc.expectedP50G} gCO₂eq ± 2 %`, () => {
      const result = estimateEcoLogits({
        method: 'ecologits',
        modelId: tc.modelId,
        tokensIn: tc.tokensIn,
        tokensOut: tc.tokensOut,
        pue: tc.pue,
        ifGramPerKwh: tc.ifGramPerKwh,
        disableEmbodied: true // usage-only, conforme au champ Rust
      });
      const usage = result.gco2eqUsage;
      const delta = Math.abs(usage - tc.expectedP50G) / tc.expectedP50G;
      expect(delta).toBeLessThan(PARITY_TOLERANCE);
      // Sanity sur les autres indicateurs
      expect(result.energyWh).toBeGreaterThan(0);
      expect(result.waterMl).toBeGreaterThan(0);
      expect(result.gco2eqEmbodied).toBe(0); // disableEmbodied=true
      expect(result.gco2eq).toBe(usage); // total = usage si embodied=0
    });
  }
});

describe('EcoLogits — intégration facade', () => {
  it('estimate({method:"ecologits"}) délègue à estimateEcoLogits', () => {
    const viaFacade = estimate({
      method: 'ecologits',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      pue: 1.2,
      ifGramPerKwh: 56,
      disableEmbodied: true
    });
    const viaEngine = estimateEcoLogits({
      method: 'ecologits',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      pue: 1.2,
      ifGramPerKwh: 56,
      disableEmbodied: true
    });
    expect(viaFacade.gco2eqUsage).toBe(viaEngine.gco2eqUsage);
    expect(viaFacade.method).toBe('ecologits');
  });

  it('modèle inconnu lève une erreur explicite', () => {
    expect(() =>
      estimateEcoLogits({
        method: 'ecologits',
        modelId: 'modele-bidon',
        tokensIn: 10,
        tokensOut: 50
      })
    ).toThrow(/modèle inconnu/);
  });

  it('embodied > 0 par défaut (mode normal)', () => {
    const r = estimateEcoLogits({
      method: 'ecologits',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500
    });
    expect(r.gco2eqEmbodied).toBeGreaterThan(0);
    expect(r.gco2eq).toBeCloseTo(r.gco2eqUsage + r.gco2eqEmbodied, 12);
  });

  it("doubler les tokens d'output ≈ double le CO₂eq usage (à γ près)", () => {
    const r1 = estimateEcoLogits({
      method: 'ecologits',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      pue: 1.2,
      ifGramPerKwh: 56,
      disableEmbodied: true
    });
    const r2 = estimateEcoLogits({
      method: 'ecologits',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 1000,
      pue: 1.2,
      ifGramPerKwh: 56,
      disableEmbodied: true
    });
    const ratio = r2.gco2eqUsage / r1.gco2eqUsage;
    // Pas exactement 2× à cause du terme constant γ par token (overhead).
    expect(ratio).toBeGreaterThan(1.85);
    expect(ratio).toBeLessThan(2.15);
  });

  it('mix US-VA produit > FR à modèle/prompt égaux', () => {
    const rFr = estimateEcoLogits({
      method: 'ecologits',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      pue: 1.2,
      ifGramPerKwh: 56,
      disableEmbodied: true
    });
    const rVa = estimateEcoLogits({
      method: 'ecologits',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      pue: 1.2,
      ifGramPerKwh: 412,
      disableEmbodied: true
    });
    expect(rVa.gco2eqUsage).toBeGreaterThan(rFr.gco2eqUsage * 5);
  });
});

// ───────────────────────────────────────────────────────────────────────────────

describe('AFNOR point-estimate — sanity & comportement', () => {
  it('llama-3-1-70b 100/500 FR PUE=1.2 produit un résultat ordonné > 0', () => {
    const r = estimateAfnor({
      method: 'afnor_sobria',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      region: 'FR',
      pue: 1.2
    });
    expect(r.gco2eq).toBeGreaterThan(0);
    expect(r.gco2eqUsage).toBeGreaterThan(0);
    expect(r.gco2eqEmbodied).toBeGreaterThan(0);
    expect(r.energyWh).toBeGreaterThan(0);
    expect(r.waterMl).toBeGreaterThan(0);
    expect(r.method).toBe('afnor_sobria');
  });

  it('formule manuelle correspond au moteur (parité interne)', () => {
    // Llama 3.1 70B : ε_prefill = 700, ε_decode = 1750, embodied = 0.0175
    // Input : 100 in, 500 out, FR mix 56, PUE 1.2, WUE 1.5
    const eComputeMj = 100 * 700 + 500 * 1750; // 945 000 mJ
    const eTotalWh = (eComputeMj * 1.2) / 3_600_000.0; // 0.315 Wh
    const usageG = (eTotalWh / 1000.0) * 56.0; // 0.01764 g
    const embodied = 0.0175;
    const totalG = usageG + embodied;
    const waterMl = (eTotalWh / 1000.0) * 1.5 * 1000.0;

    const r = estimateAfnor({
      method: 'afnor_sobria',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      region: 'FR',
      pue: 1.2
    });

    expect(r.energyWh).toBeCloseTo(eTotalWh, 12);
    expect(r.gco2eqUsage).toBeCloseTo(usageG, 12);
    expect(r.gco2eqEmbodied).toBeCloseTo(embodied, 12);
    expect(r.gco2eq).toBeCloseTo(totalG, 12);
    expect(r.waterMl).toBeCloseTo(waterMl, 9);
  });

  it("doubler les tokens d'output double le compute (linéaire AFNOR)", () => {
    const r1 = estimateAfnor({
      method: 'afnor_sobria',
      modelId: 'llama-3-1-70b',
      tokensIn: 0,
      tokensOut: 500,
      region: 'FR',
      pue: 1.2,
      disableEmbodied: true
    });
    const r2 = estimateAfnor({
      method: 'afnor_sobria',
      modelId: 'llama-3-1-70b',
      tokensIn: 0,
      tokensOut: 1000,
      region: 'FR',
      pue: 1.2,
      disableEmbodied: true
    });
    expect(r2.gco2eqUsage / r1.gco2eqUsage).toBeCloseTo(2.0, 9);
  });

  it('modèle inconnu lève une erreur', () => {
    expect(() =>
      estimateAfnor({
        method: 'afnor_sobria',
        modelId: 'modele-zzz',
        tokensIn: 10,
        tokensOut: 50
      })
    ).toThrow(/modèle inconnu/);
  });

  it('mix US-VA produit > FR à modèle/prompt égaux', () => {
    const rFr = estimateAfnor({
      method: 'afnor_sobria',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      region: 'FR'
    });
    const rVa = estimateAfnor({
      method: 'afnor_sobria',
      modelId: 'llama-3-1-70b',
      tokensIn: 100,
      tokensOut: 500,
      region: 'US-VA'
    });
    // Avec PUE/WUE identiques (1.2/1.5), seul IF change. Usage seul devient
    // ~7.4× plus carboné (412/56). Mais le total est dilué par embodied
    // qui est constant — on garde une borne très permissive.
    expect(rVa.gco2eqUsage).toBeGreaterThan(rFr.gco2eqUsage * 5);
  });
});

// ───────────────────────────────────────────────────────────────────────────────

describe('Registry presets', () => {
  it('contient au moins le catalogue C34 (mirror Rust) + les 8 modèles historiques 2024', () => {
    // ≥ 25 actifs (C34.2) + 8 deprecated conservés pour reproductibilité audit.
    expect(MODEL_PRESETS.length).toBeGreaterThanOrEqual(25);
    const ids = MODEL_PRESETS.map((p) => p.id);
    // Cœur 2025-2026 (sous-ensemble — la liste exhaustive est testée Rust).
    expect(ids).toEqual(
      expect.arrayContaining([
        'claude-opus-4-8',
        'claude-opus-4-7',
        'claude-sonnet-4-6',
        'gpt-5-5',
        'o3',
        'gemini-3-1-pro',
        'llama-4-maverick',
        'mistral-large-3'
      ])
    );
    // 8 modèles 2024 conservés pour parité tests historiques.
    expect(ids).toEqual(
      expect.arrayContaining([
        'gpt-4o',
        'gpt-4o-mini',
        'claude-3-5-sonnet',
        'mistral-large-2',
        'mistral-medium-3',
        'llama-3-1-70b',
        'llama-3-1-8b',
        'gemini-2-0-flash'
      ])
    );
  });

  it('aucun doublon d’id', () => {
    const ids = MODEL_PRESETS.map((p) => p.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('chaque preset respecte la formule K_DECODE × paramsBillion (recalibration C24)', () => {
    const K_DECODE = 25.0; // mJ/token/B, cf. crates/sobria-estimator/src/model_presets.rs
    const PREFILL_RATIO = 0.4;
    for (const preset of MODEL_PRESETS) {
      const expectedDecode = K_DECODE * preset.paramsBillion;
      const expectedPrefill = expectedDecode * PREFILL_RATIO;
      expect(preset.epsilonDecodeMjPerToken).toBeCloseTo(expectedDecode, 1);
      expect(preset.epsilonPrefillMjPerToken).toBeCloseTo(expectedPrefill, 1);
    }
  });

  it('findPreset() trouve les modèles existants et retourne undefined sinon', () => {
    expect(findPreset('llama-3-1-70b')?.paramsBillion).toBe(70.0);
    expect(findPreset('modele-bidon')).toBeUndefined();
  });
});

// Empêche le linter de râler sur les imports non-utilisés (tests par bloc).
void _afnor;
