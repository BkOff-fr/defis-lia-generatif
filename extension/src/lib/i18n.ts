// Sobr.ia extension — i18n minimaliste (C27.2).
//
// Pas de dépendance externe (svelte-i18n côté web/, surdimensionné ici).
// Catalogue plat FR + EN, démarrage en FR par défaut. À étendre en C27.4
// pour les chaînes popup/options réelles.

export type Locale = 'fr' | 'en';

export const DEFAULT_LOCALE: Locale = 'fr';

type Catalog = Record<string, { fr: string; en: string }>;

/**
 * Catalogue des chaînes traduites. Garder les clés en `kebab-case` groupé
 * par contexte (popup, options, methodology, errors).
 */
const CATALOG: Catalog = {
  // Popup
  'popup.tagline': {
    fr: 'Mesurer la sobriété de votre IA générative.',
    en: 'Measure the sobriety of your generative AI.'
  },
  'popup.last-prompt': { fr: 'Dernier prompt', en: 'Last prompt' },
  'popup.daily-total': { fr: 'Aujourd’hui', en: 'Today' },
  'popup.open-app': { fr: 'Ouvrir Sobr.ia', en: 'Open Sobr.ia' },
  'popup.settings': { fr: 'Réglages', en: 'Settings' },

  // Options
  'options.pairing': {
    fr: 'Pairing avec l’app Sobr.ia',
    en: 'Pairing with Sobr.ia app'
  },
  'options.sites': { fr: 'Sites surveillés', en: 'Watched sites' },
  'options.privacy': { fr: 'Confidentialité', en: 'Privacy' },
  'options.methodology': { fr: 'Méthodologie', en: 'Methodology' },

  // Méthodologies (mirror METHOD_INFO)
  'methodology.afnor-sobria': {
    fr: 'AFNOR SPEC 2314 — Sobr.ia',
    en: 'AFNOR SPEC 2314 — Sobr.ia'
  },
  'methodology.ecologits': {
    fr: 'EcoLogits 2026-01',
    en: 'EcoLogits 2026-01'
  },

  // Erreurs
  'error.unknown-model': {
    fr: 'Modèle inconnu (registry Sobr.ia)',
    en: 'Unknown model (Sobr.ia registry)'
  },
  'error.invalid-tokens': {
    fr: 'tokensIn et tokensOut doivent être ≥ 0',
    en: 'tokensIn and tokensOut must be ≥ 0'
  }
};

/**
 * Traduit une clé dans la locale donnée (défaut : `fr`).
 *
 * Renvoie la clé brute si elle n'existe pas, pour faciliter le debug en
 * développement (visible en surface au lieu d'une chaîne vide).
 */
export function t(key: string, locale: Locale = DEFAULT_LOCALE): string {
  const entry = CATALOG[key];
  if (!entry) return key;
  return entry[locale];
}

/** Toutes les locales supportées (pour le sélecteur options). */
export const AVAILABLE_LOCALES: readonly Locale[] = ['fr', 'en'];
