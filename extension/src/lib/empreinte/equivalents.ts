// Sobr.ia extension — équivalents parlants (C27.3 itération design 38).
//
// 4 équivalents alignés sur `sobr-ia-design-system/project/preview/38-extension-chatgpt.html`
// §"Equivalents grid" :
//   - en voiture thermique (icon car) → m ou km
//   - de douche chaude (icon shower) → sec ou min
//   - ampoule LED 9W (icon led) → min ou h
//   - recharges smartphone (icon phone) → fraction « ~ 1/N » ou nombre
//
// Constantes ADEME 2023-2024 (mirror de `crates/sobria-estimator/src/equivalents.rs`).
// Les labels **n'ont jamais l'unité** (le design place l'unité sur la valeur,
// cf. preview/27-equivalent-badge.html : « 13 m » + « en voiture thermique »).

import type { Equivalent } from '../types.js';

/** Facteur d'émission moyen d'une voiture thermique européenne (gCO₂eq/km). */
const CAR_G_CO2EQ_PER_KM = 192.0;

/** Énergie d'une douche chaude 5 min (8 L/min, ΔT=30°C, chauffe-eau élec.). */
const SHOWER_WH = 1_750.0;
/** Une douche standard dure 5 min = 300 s. */
const SHOWER_DURATION_SEC = 300.0;

/** Puissance d'une ampoule LED 9W (Wh consommé par heure). */
const LED_W = 9.0;

/** Énergie d'une charge complète de smartphone (Wh, mid-2020s smartphone). */
const PHONE_CHARGE_WH = 15.0;

/** Formate un nombre en FR à `digits` chiffres significatifs. */
function fmt(n: number, digits = 2): string {
  return new Intl.NumberFormat('fr-FR', { maximumSignificantDigits: digits }).format(n);
}

/** km en voiture thermique équivalent à un poids de CO₂eq. */
export function co2eqToCar(gco2eq: number): Equivalent {
  const km = gco2eq / CAR_G_CO2EQ_PER_KM;
  // Auto-rescale m / km selon ordre de grandeur (cf. design 38 affiche « 13 m »).
  if (km < 1.0) {
    return {
      label: 'en voiture thermique',
      value: Math.round(km * 1000),
      unit: 'm',
      icon: 'car',
      source: 'ADEME Base Empreinte 2024 — voiture essence moyenne (192 gCO₂eq/km)'
    };
  }
  return {
    label: 'en voiture thermique',
    value: Number(fmt(km).replace(',', '.')),
    unit: 'km',
    icon: 'car',
    source: 'ADEME Base Empreinte 2024 — voiture essence moyenne (192 gCO₂eq/km)'
  };
}

/** Durée de douche chaude équivalente à une énergie en Wh. */
export function energyWhToShower(wh: number): Equivalent {
  const sec = (wh / SHOWER_WH) * SHOWER_DURATION_SEC;
  if (sec >= 60) {
    return {
      label: 'de douche chaude',
      value: Number(fmt(sec / 60).replace(',', '.')),
      unit: 'min',
      icon: 'shower',
      source: 'ADEME — guide ECS 2023, douche 5 min standard (1 750 Wh)'
    };
  }
  return {
    label: 'de douche chaude',
    value: Number(fmt(sec).replace(',', '.')),
    unit: 'sec',
    icon: 'shower',
    source: 'ADEME — guide ECS 2023, douche 5 min standard (1 750 Wh)'
  };
}

/** Durée d'une ampoule LED 9W équivalente à une énergie en Wh. */
export function energyWhToLed(wh: number): Equivalent {
  const hours = wh / LED_W;
  if (hours < 1.0) {
    return {
      label: 'ampoule LED 9W',
      value: Number(fmt(hours * 60).replace(',', '.')),
      unit: 'min',
      icon: 'led',
      source: 'Ampoule LED 9W ≈ 9 Wh / h (calcul direct)'
    };
  }
  return {
    label: 'ampoule LED 9W',
    value: Number(fmt(hours).replace(',', '.')),
    unit: 'h',
    icon: 'led',
    source: 'Ampoule LED 9W ≈ 9 Wh / h (calcul direct)'
  };
}

/** Nombre de charges smartphone équivalentes à une énergie en Wh. */
export function energyWhToPhone(wh: number): Equivalent {
  const charges = wh / PHONE_CHARGE_WH;
  if (charges < 1.0) {
    // Affiche en fraction lisible « ~ 1/6 » (cf. design 38).
    const denom = Math.max(2, Math.round(1 / charges));
    return {
      label: 'recharge smartphone',
      value: 0,
      unit: 'charge',
      display: `~ 1/${denom}`,
      icon: 'phone',
      source: 'Smartphone mid-2020s ≈ 15 Wh / charge (4000 mAh × 3.7 V × 1.1 PUE)'
    };
  }
  return {
    label: 'recharges smartphone',
    value: Number(fmt(charges).replace(',', '.')),
    unit: 'charges',
    icon: 'phone',
    source: 'Smartphone mid-2020s ≈ 15 Wh / charge (4000 mAh × 3.7 V × 1.1 PUE)'
  };
}

/**
 * Calcule les 4 équivalents design 38 à partir d'un résultat (gco2eq, energyWh).
 *
 * Retourne toujours 4 entrées, dans l'ordre voiture / douche / LED / smartphone.
 */
export function computeEquivalents(args: {
  gco2eq: number;
  energyWh: number;
}): readonly Equivalent[] {
  return [
    co2eqToCar(args.gco2eq),
    energyWhToShower(args.energyWh),
    energyWhToLed(args.energyWh),
    energyWhToPhone(args.energyWh)
  ];
}
