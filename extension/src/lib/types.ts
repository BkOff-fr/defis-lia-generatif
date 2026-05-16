// Sobr.ia extension — types DTO partagés (C27.2).
//
// Mirror minimal de sobria-core::EstimationRequest + EstimationResult, simplifié
// pour usage extension (point-estimate, pas de Monte-Carlo). Voir
// crates/sobria-estimator/src/engine_trait.rs pour la version Rust complète.

/**
 * Identifiant stable de méthodologie d'empreinte.
 *
 * Mirror de `sobria_core::EmpreinteMethod` (cf. ADR-0012 §"Décision").
 */
export type EmpreinteMethod = 'afnor_sobria' | 'ecologits';

/**
 * Région d'inférence — raccourcis pour PUE / facteur d'émission / WUE.
 *
 * - `FR` : mix France 2024 (ADEME ~56 gCO₂eq/kWh)
 * - `US-VA` : mix US-Virginie (Electricity Maps moyenne ~412 gCO₂eq/kWh)
 *
 * L'utilisateur peut toujours surcharger via `pue`, `ifGramPerKwh`, `wueLPerKwh`.
 */
export type Region = 'FR' | 'US-VA';

/** Defaults régionaux (PUE / IF / WUE). */
export const REGION_DEFAULTS: Record<
  Region,
  { pue: number; ifGramPerKwh: number; wueLPerKwh: number }
> = {
  FR: { pue: 1.2, ifGramPerKwh: 56.0, wueLPerKwh: 1.5 },
  'US-VA': { pue: 1.2, ifGramPerKwh: 412.0, wueLPerKwh: 1.5 }
};

/**
 * Preset distributionnel d'un modèle, simplifié au P50 pour usage extension.
 *
 * Les triplets (P5, P50, P95) de la version Rust sont réduits à leur médiane :
 * l'extension fonctionne en point-estimate déterministe (le Monte-Carlo
 * reste côté app desktop).
 */
export type ModelPreset = {
  /** Identifiant stable (ex: `"llama-3-1-70b"`). */
  readonly id: string;
  /** Nom commercial pour affichage UI. */
  readonly displayName: string;
  /** Fournisseur (Meta, OpenAI, Anthropic, …). */
  readonly vendor: string;
  /** Famille intra-fournisseur (regroupement). */
  readonly family: string;
  /** Nombre de paramètres en milliards (estimation publique). */
  readonly paramsBillion: number;
  /** ε_prefill P50 en mJ/token. Pour AFNOR. */
  readonly epsilonPrefillMjPerToken: number;
  /** ε_decode P50 en mJ/token. Pour AFNOR. */
  readonly epsilonDecodeMjPerToken: number;
  /** Embodied amorti P50 en gCO₂eq/req. Pour AFNOR. */
  readonly embodiedGPerRequest: number;
  /** Région par défaut (cohérence app desktop). */
  readonly defaultRegion: Region;
  /** PUE par défaut (cohérence app desktop). */
  readonly defaultPue: number;
  /** Type d'architecture (dense / mixture-of-experts), pour notes futures. */
  readonly architectureFamily: 'dense' | 'moe';
  /** Sources documentaires (DOI, papers, etc.). */
  readonly sources: readonly string[];
};

/**
 * Entrée d'une estimation.
 *
 * `region` choisit les defaults PUE/IF/WUE. Les surcharges explicites
 * (`pue`, `ifGramPerKwh`, `wueLPerKwh`) ont priorité sur la région.
 *
 * `disableEmbodied=true` force embodied=0 — utilisé par les ReproductionCase
 * usage-only (parité avec `crates/sobria-estimator/tests/`).
 */
export type EstimateInput = {
  /** Méthodologie à appliquer. */
  readonly method: EmpreinteMethod;
  /** Identifiant du modèle (doit exister dans `presets.ts`). */
  readonly modelId: string;
  /** Tokens d'entrée (prompt utilisateur). */
  readonly tokensIn: number;
  /** Tokens de sortie générés. */
  readonly tokensOut: number;
  /** Région d'inférence (défaut : preset.defaultRegion ou `FR`). */
  readonly region?: Region;
  /** Surcharge PUE. */
  readonly pue?: number;
  /** Surcharge facteur d'émission (gCO₂eq/kWh). */
  readonly ifGramPerKwh?: number;
  /** Surcharge WUE (L/kWh). */
  readonly wueLPerKwh?: number;
  /** Force embodied à 0 (pour reproduction cases usage-only). */
  readonly disableEmbodied?: boolean;
};

/**
 * Identifiant d'icône pour un équivalent (mapping → SVG inline dans le badge).
 *
 * Cf. design system `preview/38-extension-chatgpt.html` §"Equivalents grid".
 */
export type EquivalentIcon = 'car' | 'shower' | 'led' | 'phone';

/**
 * Équivalent parlant pour affichage UI (voiture / douche / LED / smartphone).
 * Mirror enrichi de `sobria_core::Equivalent` côté extension.
 *
 * `label` ne contient **jamais** l'unité (le design system place l'unité dans
 * la valeur, pas le label : « 13 m » + « en voiture thermique »). Cf.
 * `preview/17-equivalents.html` et `preview/27-equivalent-badge.html`.
 */
export type Equivalent = {
  /** Descripteur sans unité (ex: "en voiture thermique"). */
  readonly label: string;
  /** Valeur numérique normalisée pour l'unité (ex: 13 pour 13 m). */
  readonly value: number;
  /** Unité d'affichage prête à concaténer (ex: "m", "sec", "min"). */
  readonly unit: string;
  /** Texte de représentation libre (ex: "~ 1/6") quand `value` n'est pas un nombre. */
  readonly display?: string;
  /** Icône à afficher à gauche du tile. */
  readonly icon: EquivalentIcon;
  /** Citation de la source / coefficient utilisé. */
  readonly source: string;
};

/**
 * Résultat point-estimate d'une estimation extension.
 *
 * Mirror simplifié de `sobria_core::EstimationResult` :
 * - `gco2eq` total (usage + embodied) en grammes
 * - `gco2eqUsage` part énergie × IF
 * - `gco2eqEmbodied` part embodied amortie
 * - `waterMl` en millilitres
 * - `energyWh` en watt-heures
 * - `equivalents` : 3 conversions parlantes (km voiture, sec douche, h écran)
 *
 * `notes` contient les hypothèses clés sourcées (DOI, version méthodo,
 * etc.) à afficher dans la popup / drawer hypothèses.
 */
export type Estimate = {
  readonly method: EmpreinteMethod;
  readonly modelId: string;
  readonly tokensIn: number;
  readonly tokensOut: number;
  readonly gco2eq: number;
  readonly gco2eqUsage: number;
  readonly gco2eqEmbodied: number;
  readonly waterMl: number;
  readonly energyWh: number;
  readonly equivalents: readonly Equivalent[];
  readonly notes: readonly string[];
};
