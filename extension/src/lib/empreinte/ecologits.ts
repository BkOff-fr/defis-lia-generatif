// Sobr.ia extension — moteur EcoLogits (port JS direct).
//
// Port direct des formules publiées par GenAI Impact :
//   Rincé S., Banse A., *EcoLogits: Evaluating the Environmental Impacts
//   of Generative AI*, JOSS 10(111):7471, 2025.
//   DOI: 10.21105/joss.07471
//   Documentation : https://ecologits.ai/latest/methodology/llm_inference/
//
// License des formules : CC BY-SA 4.0 (cf. attribution dans
// docs/methodology/ECOLOGITS-PORT.md côté projet).
//
// Ce port est la jumelle JS de `crates/sobria-estimator/src/engines/ecologits.rs` :
// mêmes constantes, même algorithmique, point-estimate déterministe. Parité
// vérifiée à ≤ 2 % dans tests/unit/empreinte.spec.ts (3 ReproductionCase).

import type { Estimate, EstimateInput } from '../types.js';
import { REGION_DEFAULTS } from '../types.js';
import { findPreset } from '../presets.js';
import { computeEquivalents } from './equivalents.js';

// ─── Constantes EcoLogits 2026-01 ────────────────────────────────────────────
// Source : https://ecologits.ai/latest/methodology/llm_inference/ (2026-01)
// Identiques à crates/sobria-estimator/src/engines/ecologits.rs constants.

/** α du fit f_E (énergie GPU par token de sortie, Wh). */
const ALPHA_E = 1.17e-6;
/** β du fit f_E. */
const BETA_E = -1.12e-2;
/** γ du fit f_E. */
const GAMMA_E = 4.05e-5;

/** α du fit f_L (latence par token, s). */
const ALPHA_L = 6.78e-4;
/** β du fit f_L. */
const BETA_L = 3.12e-4;
/** γ du fit f_L. */
const GAMMA_L = 1.94e-2;

/** Batch size de référence (vLLM continuous batching). */
const BATCH_SIZE = 64.0;

/** Puissance électrique du serveur de référence hors GPUs (p5.48xlarge). */
const P_SERVER_W = 1200.0;

/** Nombre de GPUs installés sur le serveur de référence. */
const N_GPU_INSTALLED = 8.0;

/** Mémoire VRAM par GPU (H100 80 Go). */
const MEM_GPU_GB = 80.0;

/** Bits par poids du modèle (FP16). */
const Q_BITS = 16.0;

/** Overhead mémoire d'inférence (KV cache, activations, marges). */
const MEM_OVERHEAD = 1.2;

/** Embodied carbon du serveur hors GPUs (Boavizta amorti sur 3 ans). */
const I_SERVER_NO_GPU_KG = 5700.0;

/** Embodied carbon d'un GPU H100 (Boavizta). */
const I_GPU_KG = 273.0;

/** Durée de vie hardware (3 ans, en secondes). */
const HW_LIFETIME_SEC = 3.0 * 365.25 * 86_400.0;

// ─── Formules EcoLogits ──────────────────────────────────────────────────────

/**
 * Plus petite puissance de 2 supérieure ou égale à `n`.
 *
 * Équivalent JS de `u32::next_power_of_two()` côté Rust.
 */
function nextPowerOfTwo(n: number): number {
  if (n <= 1) return 1;
  return 2 ** Math.ceil(Math.log2(n));
}

/**
 * Nombre de GPUs requis pour servir un modèle de `pBillions` milliards
 * de paramètres en FP16, arrondi à la puissance de 2 supérieure.
 *
 * Formule : `n_GPU = next_pow2(ceil(1.2 × P × 16/8 / 80))`.
 */
export function nGpu(pBillions: number): number {
  const mModelGb = (MEM_OVERHEAD * pBillions * Q_BITS) / 8.0;
  const raw = Math.ceil(mModelGb / MEM_GPU_GB);
  return raw <= 1 ? 1 : nextPowerOfTwo(raw);
}

/**
 * Énergie GPU par token de sortie, Wh, à batch B=64.
 *
 * `f_E(P, 64) = α × exp(β × 64) × P + γ`
 */
export function fEnergyPerTokenWh(pBillions: number): number {
  return ALPHA_E * Math.exp(BETA_E * BATCH_SIZE) * pBillions + GAMMA_E;
}

/**
 * Latence par token de sortie, secondes, à batch B=64.
 *
 * `f_L(P, 64) = α' × P + β' × 64 + γ'`
 */
export function fLatencyPerTokenSec(pBillions: number): number {
  return ALPHA_L * pBillions + BETA_L * BATCH_SIZE + GAMMA_L;
}

/** Énergie totale d'une requête en kWh (usage seul, hors embodied). */
export function requestEnergyKwh(pBillions: number, tokensOut: number, pue: number): number {
  const nGpuF = nGpu(pBillions);
  const dtSec = tokensOut * fLatencyPerTokenSec(pBillions);
  const eGpuWh = nGpuF * tokensOut * fEnergyPerTokenWh(pBillions);
  const eServerNoGpuWh = (dtSec * P_SERVER_W * (nGpuF / N_GPU_INSTALLED)) / BATCH_SIZE / 3600.0;
  const eServerWh = eGpuWh + eServerNoGpuWh;
  const eRequestWh = pue * eServerWh;
  return eRequestWh / 1000.0;
}

/**
 * Embodied carbon amorti par requête (g CO₂eq).
 *
 * `I_request_emb = (ΔT / (B × ΔL)) × I_server`
 * avec `I_server = (n_GPU/N_installed) × I_server_noGPU + n_GPU × I_GPU`.
 */
export function requestEmbodiedCo2eqG(pBillions: number, tokensOut: number): number {
  const nGpuF = nGpu(pBillions);
  const dtSec = tokensOut * fLatencyPerTokenSec(pBillions);
  const iServerKg = (nGpuF / N_GPU_INSTALLED) * I_SERVER_NO_GPU_KG + nGpuF * I_GPU_KG;
  const allocRatio = dtSec / (BATCH_SIZE * HW_LIFETIME_SEC);
  return allocRatio * iServerKg * 1000.0;
}

// ─── Moteur ──────────────────────────────────────────────────────────────────

/**
 * Lance l'estimation EcoLogits pour une entrée extension.
 *
 * Lève une erreur si `input.modelId` n'est pas dans le registry (cf.
 * `presets.ts`). Mirror du comportement `EcoLogitsEngine::estimate` côté Rust.
 */
export function estimateEcoLogits(input: EstimateInput): Estimate {
  const preset = findPreset(input.modelId);
  if (!preset) {
    throw new Error(`EcoLogits: modèle inconnu "${input.modelId}" — registry Sobr.ia`);
  }
  if (input.tokensIn < 0 || input.tokensOut < 0) {
    throw new Error('EcoLogits: tokensIn et tokensOut doivent être ≥ 0');
  }

  const region = input.region ?? preset.defaultRegion;
  const defaults = REGION_DEFAULTS[region];
  const pue = input.pue ?? defaults.pue;
  const ifElec = input.ifGramPerKwh ?? defaults.ifGramPerKwh;
  const wue = input.wueLPerKwh ?? defaults.wueLPerKwh;

  const pBillions = preset.paramsBillion;

  // Usage : énergie (kWh) × facteur d'émission (gCO₂eq/kWh) = g CO₂eq.
  const energyKwh = requestEnergyKwh(pBillions, input.tokensOut, pue);
  const energyWh = energyKwh * 1000.0;
  const usageCo2G = energyKwh * ifElec;

  // Embodied : formule EcoLogits interne, sauf si l'utilisateur force usage-only.
  const embodiedCo2G = input.disableEmbodied
    ? 0
    : requestEmbodiedCo2eqG(pBillions, input.tokensOut);

  const totalCo2G = usageCo2G + embodiedCo2G;

  // Eau : formule équivalente Sobr.ia (E_kWh × WUE en L → ×1000 pour mL).
  const waterMl = energyKwh * wue * 1000.0;

  const notes: string[] = [
    'EcoLogits 2026-01 — doi:10.21105/joss.07471 (CC BY-SA 4.0)',
    `Modèle ${preset.displayName} ~${pBillions} G paramètres, ${nGpu(pBillions)} GPUs H100 (next_pow2)`,
    `PUE=${pue}, IF=${ifElec} gCO₂eq/kWh (${region}), WUE=${wue} L/kWh`
  ];
  if (input.disableEmbodied) {
    notes.push('Embodied désactivé (mode usage-only)');
  }

  return {
    method: 'ecologits',
    modelId: input.modelId,
    tokensIn: input.tokensIn,
    tokensOut: input.tokensOut,
    gco2eq: totalCo2G,
    gco2eqUsage: usageCo2G,
    gco2eqEmbodied: embodiedCo2G,
    waterMl,
    energyWh,
    equivalents: computeEquivalents({ gco2eq: totalCo2G, energyWh }),
    notes
  };
}
