// Sobr.ia extension — métadonnées du registry modèles (cohorte = version extension).
//
// Le « registry » affiché à l'utilisateur est la version du catalogue
// `MODEL_PRESETS` (miroir de `crates/sobria-estimator/.../MODEL_REGISTRY`).
// Une seule source de vérité pour les libellés UI : `package.json` → version.

import pkg from '../../package.json' with { type: 'json' };
import { MODEL_PRESETS } from './presets.js';

/** Version extension / cohorte registry (ex. 0.9.0). */
export const EXTENSION_VERSION: string = pkg.version;

/** Alias explicite : le registry embarqué suit la version extension. */
export const REGISTRY_VERSION: string = EXTENSION_VERSION;

/** Nombre de presets dans le registry embarqué. */
export const REGISTRY_MODEL_COUNT: number = MODEL_PRESETS.length;

/** Libellé court pour tooltips et logs (ex. « registry Sobr.ia v0.9.0 »). */
export function registryLabel(): string {
  return `registry Sobr.ia v${REGISTRY_VERSION}`;
}

/**
 * Info-bulle du badge « modèle non pris en charge ».
 * `modelLabel` = libellé brut lu dans l'UI du site (si disponible).
 */
export function unsupportedModelTooltip(modelLabel?: string): string {
  const meta = `${registryLabel()} · ${REGISTRY_MODEL_COUNT} modèles`;
  if (modelLabel?.trim()) {
    return `Modèle « ${modelLabel.trim()} » pas encore dans le ${meta}`;
  }
  return `Ce modèle n’est pas encore dans le ${meta}`;
}

/** Applique version + compteur registry aux emplacements marqués dans le DOM. */
export function applyVersionLabels(root: ParentNode = document): void {
  const v = EXTENSION_VERSION;
  const count = String(REGISTRY_MODEL_COUNT);

  root.querySelectorAll<HTMLElement>('[data-sobria-version]').forEach((el) => {
    el.dataset.sobriaVersion = v;
  });

  root.querySelectorAll<HTMLElement>('[data-sobria-version-display]').forEach((el) => {
    el.textContent = `v${v}`;
  });

  const aboutVersion = root.querySelector<HTMLElement>('[data-sobria-about-version]');
  if (aboutVersion) {
    aboutVersion.textContent = `${v} · registry ${count} modèles (WebExtension MV3)`;
  }

  const methodologyIntro = root.querySelector<HTMLElement>('[data-sobria-methodology-intro]');
  if (methodologyIntro) {
    methodologyIntro.innerHTML = [
      `Deux méthodologies embarquées (<strong>${registryLabel()}</strong>, `,
      `<strong>${count} modèles</strong>) : `,
      `<strong>AFNOR SPEC 2314</strong> (référentiel français) et `,
      `<strong>EcoLogits 2026-01</strong> (peer-reviewed JOSS 2025). `,
      `Le choix se fait depuis la popup. Sources et formules détaillées dans l'app desktop /methodologies.`
    ].join('');
  }
}
