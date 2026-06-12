// Sobr.ia extension — indicateur circulaire compact à côté du composer.
//
// Design « less is more » :
//   - Petit cercle 32 px avec progress ring SVG autour
//   - Lettre de grade (A-F) au centre, en Instrument Serif italic
//   - Couleur ring/lettre = tone du grade (lime / amber / coral)
//   - Animation scale + opacity à l'apparition (start typing)
//   - Disparaît quand le textarea redevient vide (après envoi notamment)
//
// Si le modèle n'est pas dans le registry Sobr.ia, le cercle affiche « ? »
// désactivé (pas d'estimation trompeuse).

import { estimate } from '../../lib/empreinte/index.js';
import { gaugeRatio, pickGrade } from '../../lib/empreinte/grade.js';
import { estimateTokens, estimateOutputTokens } from './tokens.js';
import { getMethod } from './storage.js';
import { findPreset } from '../../lib/presets.js';
import type { EmpreinteMethod } from '../../lib/types.js';
import {
  FONT_UI,
  FONT_DISPLAY,
  FONT_MONO,
  renderShadowFontFaces,
  SHADOW_HOST_TYPO
} from '../../lib/design-fonts.js';

const HOST_ATTR = 'data-sobria-typing';
const THROTTLE_MS = 80;

export type ComposerIndicatorConfig = {
  /** Textarea / contenteditable du composer. */
  readonly textarea: Element;
  /** Sélecteur du conteneur composer (form, footer, etc.). */
  readonly composerSelector: string;
  /**
   * Sélecteur du bouton d'envoi : le cercle est inséré juste avant lui pour
   * apparaître à droite de la zone de saisie.
   */
  readonly sendButtonSelector?: string;
  /**
   * Extracteur du modèle courant. Doit retourner `null` si le modèle n'est
   * **pas** dans le registry Sobr.ia. Dans ce cas, l'indicateur affiche un
   * état désactivé « ? » plutôt qu'une estimation fausse.
   */
  readonly extractModelId: () => string | null;
};

function fmtFr(n: number, digits = 2): string {
  return new Intl.NumberFormat('fr-FR', { maximumSignificantDigits: digits }).format(n);
}

function toneOf(gco2eq: number): 'lime' | 'amber' | 'coral' {
  const g = pickGrade(gco2eq);
  if (g === 'A' || g === 'B') return 'lime';
  if (g === 'C' || g === 'D') return 'amber';
  return 'coral';
}

function readText(el: Element): string {
  if (el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement) {
    return el.value;
  }
  return el.textContent ?? '';
}

// Circonférence d'un cercle r=14 : C = 2π × 14 ≈ 87.96.
const RING_CIRCUMFERENCE = 87.96;

function renderMarkup(): string {
  return `
<style>
${renderShadowFontFaces()}
:host {
  all: initial;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  /* Anim entrée/sortie : scale + opacity. */
  opacity: 0;
  transform: scale(0.6);
  transition:
    opacity 220ms cubic-bezier(0.2, 0.8, 0.2, 1),
    transform 220ms cubic-bezier(0.34, 1.56, 0.64, 1);
  pointer-events: none;
  ${SHADOW_HOST_TYPO}
}
:host([data-active='1']) {
  opacity: 1;
  transform: scale(1);
  pointer-events: auto;
}
.bubble {
  position: relative;
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  cursor: default;
}
.ring {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.ring circle {
  transition:
    stroke-dashoffset 280ms cubic-bezier(0.2, 0.8, 0.2, 1),
    stroke 220ms ease-out;
}
.ring .track {
  stroke: rgba(255, 255, 255, 0.08);
}
.ring .arc {
  stroke: #c5f04a;
  stroke-dasharray: ${RING_CIRCUMFERENCE};
  stroke-dashoffset: ${RING_CIRCUMFERENCE};
  filter: drop-shadow(0 0 4px rgba(197, 240, 74, 0.4));
}
:host([data-tone='amber']) .ring .arc {
  stroke: #f5b769;
  filter: drop-shadow(0 0 4px rgba(245, 183, 105, 0.4));
}
:host([data-tone='coral']) .ring .arc {
  stroke: #f06c5a;
  filter: drop-shadow(0 0 4px rgba(240, 108, 90, 0.4));
}
:host([data-unknown='1']) .ring .arc {
  stroke: rgba(255, 255, 255, 0.18);
  filter: none;
}
.grade {
  font: 400 14px/1 ${FONT_DISPLAY};
  font-style: italic;
  color: #c5f04a;
  position: relative;
  z-index: 1;
  transition: color 220ms ease-out;
}
:host([data-tone='amber']) .grade { color: #f5b769; }
:host([data-tone='coral']) .grade { color: #f06c5a; }
:host([data-unknown='1']) .grade {
  color: rgba(255, 255, 255, 0.5);
  font-style: normal;
  font-family: ${FONT_UI};
  font-weight: 600;
  font-size: 13px;
}
.tooltip {
  position: absolute;
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  background: #0a0d0b;
  color: #f0ece3;
  font: 500 12px ${FONT_UI};
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 6px 14px rgba(0, 0, 0, 0.4);
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 180ms ease-out;
  z-index: 10;
}
.bubble:hover .tooltip,
.bubble:focus-within .tooltip { opacity: 1; }
.tooltip .v {
  font-family: ${FONT_MONO};
  color: #c5f04a;
}
:host([data-tone='amber']) .tooltip .v { color: #f5b769; }
:host([data-tone='coral']) .tooltip .v { color: #f06c5a; }
@media (prefers-reduced-motion: reduce) {
  :host { transition: none; }
  .ring circle { transition: none; }
}
</style>
<div class="bubble" role="status" aria-live="polite">
  <svg class="ring" viewBox="0 0 32 32" aria-hidden="true">
    <circle class="track" cx="16" cy="16" r="14" fill="none" stroke-width="2"/>
    <circle class="arc"   cx="16" cy="16" r="14" fill="none" stroke-width="2"
            stroke-linecap="round" transform="rotate(-90 16 16)"/>
  </svg>
  <span class="grade" aria-label="Empreinte Sobr.ia">—</span>
  <span class="tooltip"><span class="v">0 g CO₂eq</span></span>
</div>
`.trim();
}

/**
 * Injecte l'indicateur circulaire à droite du composer.
 *
 * - Caché par défaut (opacity 0, scale 0.6).
 * - Animation scale-in à la première saisie.
 * - Animation scale-out quand le textarea redevient vide (après envoi).
 * - Affiche « ? » désactivé si `extractModelId()` retourne `null`.
 *
 * Retourne une fonction de cleanup.
 */
export function injectComposerIndicator(config: ComposerIndicatorConfig): () => void {
  const composer =
    (config.textarea.closest(config.composerSelector) as HTMLElement | null) ??
    config.textarea.parentElement;
  if (!composer) return () => undefined;

  document.querySelectorAll(`[${HOST_ATTR}]`).forEach((el) => el.remove());

  const host = document.createElement('span');
  host.setAttribute(HOST_ATTR, '1');
  host.style.display = 'inline-flex';
  host.style.alignItems = 'center';
  host.style.marginRight = '4px';

  const shadow = host.attachShadow({ mode: 'open' });
  shadow.innerHTML = renderMarkup();

  // Insertion : juste avant le bouton d'envoi pour apparaître à droite de
  // la zone de saisie. Fallback : append en fin de composer.
  const sendBtn = config.sendButtonSelector
    ? composer.querySelector(config.sendButtonSelector)
    : null;
  if (sendBtn?.parentElement === composer) {
    composer.insertBefore(host, sendBtn);
  } else if (sendBtn?.parentElement) {
    sendBtn.parentElement.insertBefore(host, sendBtn);
  } else {
    composer.appendChild(host);
  }

  let timer: ReturnType<typeof setTimeout> | null = null;
  let currentMethod: EmpreinteMethod = 'afnor_sobria';
  getMethod()
    .then((m) => {
      currentMethod = m;
    })
    .catch(() => undefined);

  const gradeEl = shadow.querySelector<HTMLElement>('.grade');
  const arcEl = shadow.querySelector<SVGCircleElement>('.ring .arc');
  const tooltipValEl = shadow.querySelector<HTMLElement>('.tooltip .v');

  function setVisible(visible: boolean): void {
    if (visible) host.setAttribute('data-active', '1');
    else host.removeAttribute('data-active');
  }
  function setUnknown(unknown: boolean): void {
    if (unknown) host.setAttribute('data-unknown', '1');
    else host.removeAttribute('data-unknown');
  }
  function setTone(tone: 'lime' | 'amber' | 'coral' | null): void {
    if (!tone || tone === 'lime') host.removeAttribute('data-tone');
    else host.setAttribute('data-tone', tone);
  }

  function update(): void {
    const text = readText(config.textarea).trim();
    if (text.length === 0) {
      setVisible(false);
      setUnknown(false);
      setTone(null);
      if (gradeEl) gradeEl.textContent = '—';
      if (arcEl) arcEl.setAttribute('stroke-dashoffset', `${RING_CIRCUMFERENCE}`);
      if (tooltipValEl) tooltipValEl.textContent = '0 g CO₂eq';
      return;
    }

    const modelId = config.extractModelId();
    if (!modelId || !findPreset(modelId)) {
      // Modèle inconnu / non géré → état désactivé.
      setVisible(true);
      setUnknown(true);
      setTone(null);
      if (gradeEl) gradeEl.textContent = '?';
      if (arcEl) arcEl.setAttribute('stroke-dashoffset', '0'); // ring complet en gris
      if (tooltipValEl) tooltipValEl.textContent = 'Modèle non pris en charge';
      return;
    }

    setUnknown(false);
    const tokensIn = estimateTokens(text);
    const tokensOut = estimateOutputTokens(tokensIn);
    try {
      const result = estimate({
        method: currentMethod,
        modelId,
        tokensIn,
        tokensOut
      });
      const ratio = gaugeRatio(result.gco2eq);
      const grade = pickGrade(result.gco2eq);
      setVisible(true);
      setTone(toneOf(result.gco2eq));
      if (gradeEl) gradeEl.textContent = grade;
      if (arcEl) {
        const offset = RING_CIRCUMFERENCE * (1 - Math.min(1, Math.max(0, ratio)));
        arcEl.setAttribute('stroke-dashoffset', `${offset}`);
      }
      if (tooltipValEl) tooltipValEl.textContent = `${fmtFr(result.gco2eq)} g CO₂eq`;
    } catch {
      // Erreur silencieuse — affichage en état désactivé.
      setUnknown(true);
      if (gradeEl) gradeEl.textContent = '?';
    }
  }

  function onInput(): void {
    if (timer) clearTimeout(timer);
    timer = setTimeout(update, THROTTLE_MS);
  }

  config.textarea.addEventListener('input', onInput);
  config.textarea.addEventListener('keyup', onInput);
  update();

  return function dispose(): void {
    if (timer) clearTimeout(timer);
    config.textarea.removeEventListener('input', onInput);
    config.textarea.removeEventListener('keyup', onInput);
    host.remove();
  };
}
