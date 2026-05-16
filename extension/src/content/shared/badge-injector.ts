// Sobr.ia extension — badge bouton + popout (design 38, C27.3 v3).
//
// Implémente `sobr-ia-design-system/project/preview/38-extension-chatgpt.html` :
//   - **Button compact** « [leaf] 2,14 g CO₂eq | Sobr.ia ▾ » conçu pour
//     s'intégrer dans la **rangée d'actions du message bot** (à côté des
//     boutons natifs copier/like/dislike/etc.).
//   - **Popout 540 px** sous le bouton avec :
//       · Hero metric (Instrument Serif italic) + jauge score A-F
//       · 4 onglets : Équivalents · Détail · Cumul session · Méthode
//       · Footer « 🔒 100 % local »
//
// Shadow DOM isolé, fontes Sobr.ia self-host, palette ink + lime + ivory.

import type { Estimate, EmpreinteMethod, EquivalentIcon } from '../../lib/types.js';
import type { DailyTotal } from '../../lib/messages.js';

/** Mapping gco2eq → grade score A-F (design 38 §"gauge"). */
function pickGrade(gco2eq: number): { letter: 'A' | 'B' | 'C' | 'D' | 'E' | 'F'; ratio: number } {
  // Ratio = remplissage gauge SVG (0..1). Plus c'est carboné, plus on s'approche de 1.
  if (gco2eq < 1.0) return { letter: 'A', ratio: 0.15 };
  if (gco2eq < 3.0) return { letter: 'B', ratio: 0.3 };
  if (gco2eq < 5.0) return { letter: 'C', ratio: 0.45 };
  if (gco2eq < 10.0) return { letter: 'D', ratio: 0.6 };
  if (gco2eq < 20.0) return { letter: 'E', ratio: 0.78 };
  return { letter: 'F', ratio: 0.92 };
}

/** Formatage FR à `digits` chiffres significatifs (défaut 3 → "2,14"). */
function fmt(n: number, digits = 3): string {
  return new Intl.NumberFormat('fr-FR', { maximumSignificantDigits: digits }).format(n);
}

function shortMethodLabel(method: EmpreinteMethod): string {
  switch (method) {
    case 'afnor_sobria':
      return 'AFNOR';
    case 'ecologits':
      return 'EcoLogits';
    default: {
      const exhaustive: never = method;
      return String(exhaustive);
    }
  }
}

const HOST_ATTR = 'data-sobria-badge';

/** URL d'une ressource extension (fonts/icons), ou '' hors contexte extension. */
function extensionUrl(path: string): string {
  try {
    const api = (globalThis as { chrome?: { runtime?: { getURL?: (p: string) => string } } }).chrome
      ?.runtime?.getURL;
    return api ? api(path) : '';
  } catch {
    return '';
  }
}

/** Mark Sobr.ia simplifié inline (favicon simplifié, currentColor). */
const MARK_SVG = `<svg viewBox="0 0 100 100" fill="none" aria-hidden="true" focusable="false">
  <path d="M 18 78 C 12 50, 32 18, 64 22 C 78 24, 86 36, 84 52 C 81 78, 52 88, 22 84" stroke="currentColor" stroke-width="9" stroke-linecap="round" fill="none"/>
  <circle cx="50" cy="52" r="9" fill="currentColor"/>
</svg>`;

/** Mark Sobr.ia complet (avec nervure + nœuds) pour le header popout. */
const MARK_RICH_SVG = `<svg viewBox="0 0 100 100" fill="none" aria-hidden="true" focusable="false" style="filter: drop-shadow(0 0 8px rgba(197, 240, 74, 0.5))">
  <path d="M 18 78 C 12 50, 32 18, 64 22 C 78 24, 86 36, 84 52 C 81 78, 52 88, 22 84" stroke="currentColor" stroke-width="7" stroke-linecap="round" fill="none"/>
  <path d="M 22 84 C 30 70, 38 56, 48 50 C 60 42, 74 38, 84 52" stroke="currentColor" stroke-width="5" stroke-linecap="round" fill="none" opacity=".7"/>
  <circle cx="32" cy="72" r="7" fill="currentColor"/>
  <circle cx="50" cy="52" r="8" fill="currentColor"/>
  <circle cx="74" cy="32" r="6" fill="currentColor"/>
</svg>`;

/** Mapping icône équivalent → SVG inline (stroke currentColor pour tons). */
const EQ_ICONS: Record<EquivalentIcon, string> = {
  car: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M14 17H6a2 2 0 01-2-2V8a2 2 0 012-2h12a2 2 0 012 2v9h-2"/><circle cx="7" cy="17" r="2"/><circle cx="17" cy="17" r="2"/></svg>`,
  shower: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"><path d="M12 3c0 0-5 5-5 9a5 5 0 0010 0c0-4-5-9-5-9z"/></svg>`,
  led: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M9 18h6M10 22h4M12 2a7 7 0 00-4 12.7l1 .7v1.6h6v-1.6l1-.7A7 7 0 0012 2z"/></svg>`,
  phone: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="7" y="2" width="10" height="20" rx="2"/><path d="M11 18h2"/></svg>`
};

/** Construit le bloc @font-face si on est en contexte extension. */
function renderFontFace(): string {
  const geist = extensionUrl('fonts/geist-latin.woff2');
  const instrument = extensionUrl('fonts/instrument-serif-italic-latin.woff2');
  const mono = extensionUrl('fonts/jetbrains-mono-latin.woff2');
  if (!geist) return '';
  return `
@font-face { font-family: 'Sobria Geist'; font-style: normal; font-weight: 300 700; font-display: swap; src: url('${geist}') format('woff2'); }
@font-face { font-family: 'Sobria Instrument'; font-style: italic; font-weight: 400; font-display: swap; src: url('${instrument}') format('woff2'); }
@font-face { font-family: 'Sobria Mono'; font-style: normal; font-weight: 400 600; font-display: swap; src: url('${mono}') format('woff2'); }
`;
}

/** Construit l'HTML interne du host shadow root selon design 38. */
function renderBadgeMarkup(estimate: Estimate, session: DailyTotal | null): string {
  const methodLabel = shortMethodLabel(estimate.method);
  const gco2 = fmt(estimate.gco2eq);
  const water = fmt(estimate.waterMl);
  const energy = fmt(estimate.energyWh, 3);
  const usage = fmt(estimate.gco2eqUsage);
  const embodied = fmt(estimate.gco2eqEmbodied);
  const tokensIn = estimate.tokensIn;
  const tokensOut = estimate.tokensOut;

  const { letter, ratio } = pickGrade(estimate.gco2eq);
  const gaugeCircumference = 226; // 2 × π × 36 (cf. design 38)
  const gaugeOffset = Math.round(gaugeCircumference * (1 - ratio));

  // Hypothèse IC ±28 % (P5/P95) ~ équivalent log-normal σ ≈ 0.30 — cf. ADR-0004.
  const p5 = fmt(estimate.gco2eq * 0.72);
  const p95 = fmt(estimate.gco2eq * 1.28);

  const equivTiles = estimate.equivalents
    .map((eq) => {
      const valueDisplay = eq.display ?? fmt(eq.value);
      return `<div class="eq">
  <div class="ic">${EQ_ICONS[eq.icon]}</div>
  <div class="body-eq">
    <div class="nm">${eq.label}</div>
    <div class="v">${valueDisplay} <span class="u">${eq.unit}</span></div>
  </div>
</div>`;
    })
    .join('');

  const sessionTotal = session ?? {
    count: 1,
    gco2eq: estimate.gco2eq,
    waterMl: estimate.waterMl,
    energyWh: estimate.energyWh
  };
  const sessionGco2 = fmt(sessionTotal.gco2eq);
  const sessionWater = fmt(sessionTotal.waterMl);
  const sessionEnergy = fmt(sessionTotal.energyWh, 3);
  const sessionTokensIn = tokensIn * sessionTotal.count;
  const sessionTokensOut = tokensOut * sessionTotal.count;
  const sessionKmCar = fmt(sessionTotal.gco2eq / 192);

  return `
<style>
${renderFontFace()}
:host {
  all: initial;
  /* PAS de contain ici : la spec CSS impose qu un element avec containment
     devient le containing block des position:fixed descendants, ce qui empechait
     le popout de s ancrer au viewport (clipping observe dans ChatGPT). */
  display: inline-flex;
  font-family: 'Sobria Geist', system-ui, -apple-system, 'Segoe UI', sans-serif;
  font-variant-numeric: tabular-nums;
}

/* ─── Button (compact, dans la row d'actions) ─── */
.sb-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 11px 5px 8px;
  background: rgba(197, 240, 74, 0.08);
  border: 1px solid rgba(197, 240, 74, 0.2);
  border-radius: 999px;
  color: #c5f04a;
  font: 500 11px/1 'Sobria Geist', system-ui, sans-serif;
  cursor: pointer;
  transition: all 200ms;
  position: relative;
  font-family: inherit;
}
.sb-btn:hover { background: rgba(197, 240, 74, 0.14); border-color: rgba(197, 240, 74, 0.45); }
.sb-btn .leaf-ic { flex-shrink: 0; width: 13px; height: 13px; display: inline-flex; }
.sb-btn .leaf-ic svg { width: 100%; height: 100%; }
.sb-btn .val {
  font-family: 'Sobria Mono', ui-monospace, monospace;
  font-weight: 500;
  font-size: 11px;
}
.sb-btn .by {
  font: 400 9px 'Sobria Geist', sans-serif;
  color: rgba(197, 240, 74, 0.5);
  letter-spacing: 0.04em;
  margin-left: 4px;
  border-left: 1px solid rgba(197, 240, 74, 0.2);
  padding-left: 6px;
  text-transform: uppercase;
}
.sb-btn .chev {
  font-size: 9px;
  color: rgba(197, 240, 74, 0.5);
  transition: transform 200ms;
}
.sb-btn.open .chev { transform: rotate(180deg); }
.sb-btn::before {
  content: '';
  position: absolute;
  inset: -3px;
  border-radius: 999px;
  border: 1px solid rgba(197, 240, 74, 0.35);
  opacity: 0;
  animation: sbPulse 2.4s ease-out infinite;
  pointer-events: none;
}
@keyframes sbPulse {
  0% { transform: scale(0.92); opacity: 0.6; }
  100% { transform: scale(1.12); opacity: 0; }
}

/* ─── Popout (cliquable, 4 onglets) ─── */
/* Flow inline : pas de position:fixed → la conversation s'adapte
   (push down du contenu suivant) et scroll naturellement pour voir
   l'ensemble du popout. Le host est en display:block plein largeur
   (cf. expandHostInline) → le popout prend la largeur de la colonne
   message, plus de width:540 figée. */
.sb-pop {
  display: none;
  width: 100%;
  max-width: 720px;
  background: #0a0d0b;
  border: 1px solid #1f2620;
  border-radius: 14px;
  font-family: 'Sobria Geist', system-ui, sans-serif;
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(197, 240, 74, 0.08);
  color: #f0ece3;
  animation: sbSlide 350ms cubic-bezier(0.2, 0.8, 0.2, 1);
}
:host([data-expanded='1']) .sb-pop { display: block; }
@keyframes sbSlide {
  from { opacity: 0; transform: translateY(-6px); }
  to { opacity: 1; transform: translateY(0); }
}

.sb-pop .head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px;
  border-bottom: 1px solid #1f2620;
  background: linear-gradient(90deg, rgba(197, 240, 74, 0.06), rgba(197, 240, 74, 0.01));
}
.sb-pop .head .rich-mark {
  width: 16px;
  height: 16px;
  color: #c5f04a;
  display: inline-flex;
}
.sb-pop .head .rich-mark svg { width: 100%; height: 100%; }
.sb-pop .head .eye {
  font: 500 9px 'Sobria Geist', sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  color: #a5a39a;
}
.sb-pop .head .live {
  font: 500 10px 'Sobria Geist', sans-serif;
  color: #c5f04a;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
}
.sb-pop .head .close {
  width: 22px;
  height: 22px;
  display: grid;
  place-items: center;
  border-radius: 6px;
  color: #72706a;
  cursor: pointer;
  font-size: 15px;
  line-height: 1;
  background: none;
  border: none;
  font-family: inherit;
}
.sb-pop .head .close:hover { background: rgba(255, 255, 255, 0.04); color: #fff; }

/* Hero metric */
.sb-hero {
  padding: 20px 22px 18px;
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 16px;
  align-items: center;
}
.sb-hero .lab {
  font: 500 9px 'Sobria Geist', sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: #a5a39a;
  margin-bottom: 6px;
}
.sb-hero .big {
  font: 400 44px/1 'Sobria Instrument', 'Cormorant Garamond', Georgia, serif;
  font-style: italic;
  color: #f0ece3;
  letter-spacing: -0.02em;
}
.sb-hero .big .u {
  font: 400 13px 'Sobria Geist', sans-serif;
  font-style: normal;
  color: #a5a39a;
  margin-left: 6px;
}
.sb-hero .ci {
  font: 400 11px/1 'Sobria Mono', ui-monospace, monospace;
  color: #c5f04a;
  margin-top: 6px;
}
.sb-hero .gauge {
  position: relative;
  width: 90px;
  height: 90px;
}
.sb-hero .gauge svg { transform: rotate(-90deg); }
.sb-hero .gauge .gr {
  font: 400 26px 'Sobria Instrument', serif;
  font-style: italic;
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  color: #f0ece3;
}
.sb-hero .gauge .gl {
  position: absolute;
  bottom: -14px;
  left: 0;
  right: 0;
  text-align: center;
  font: 500 9px 'Sobria Geist', sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: #a5a39a;
}

/* Tabs */
.sb-tabs {
  display: flex;
  border-bottom: 1px solid #1f2620;
  padding: 0 18px;
}
.sb-tab {
  padding: 11px 14px;
  font: 500 11px 'Sobria Geist', sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: #a5a39a;
  cursor: pointer;
  border: none;
  background: none;
  border-bottom: 1.5px solid transparent;
  margin-bottom: -1px;
  transition: all 200ms;
  font-family: inherit;
}
.sb-tab:hover { color: #f0ece3; }
.sb-tab.on { color: #c5f04a; border-bottom-color: #c5f04a; }

.sb-body { padding: 18px 22px 20px; display: none; }
.sb-body.on { display: block; }

/* Equivalents grid (4 tiles) */
.eq-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; }
.eq {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px dashed #1f2620;
  border-radius: 10px;
  transition: all 200ms;
}
.eq:hover { border-color: rgba(197, 240, 74, 0.25); background: rgba(197, 240, 74, 0.04); }
.eq .ic {
  width: 34px;
  height: 34px;
  border-radius: 8px;
  background: rgba(197, 240, 74, 0.06);
  display: grid;
  place-items: center;
  color: #c5f04a;
  flex-shrink: 0;
}
.eq .ic svg { width: 18px; height: 18px; }
.eq .body-eq { flex: 1; min-width: 0; }
.eq .nm {
  font: 400 11px 'Sobria Geist', sans-serif;
  color: #a5a39a;
  font-style: italic;
}
.eq .v {
  font: 400 18px/1 'Sobria Instrument', serif;
  font-style: italic;
  color: #f0ece3;
  margin-top: 2px;
  letter-spacing: -0.01em;
}
.eq .v .u {
  font: 400 10px 'Sobria Geist', sans-serif;
  font-style: normal;
  color: #72706a;
  margin-left: 3px;
}

/* Meta tiles row (énergie / eau / tokens) */
.meta-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; margin-top: 14px; }
.meta {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid #1f2620;
  border-radius: 8px;
  padding: 9px 12px;
}
.meta .k {
  font: 500 9px 'Sobria Geist', sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: #a5a39a;
}
.meta .v {
  font: 400 16px 'Sobria Instrument', serif;
  font-style: italic;
  color: #f0ece3;
  margin-top: 3px;
  letter-spacing: -0.01em;
}
.meta .v .u {
  font: 400 9px 'Sobria Geist', sans-serif;
  font-style: normal;
  color: #72706a;
  margin-left: 2px;
}

/* Détail breakdown */
.brk { display: flex; flex-direction: column; gap: 10px; }
.brk-row {
  display: grid;
  grid-template-columns: 120px 1fr 64px;
  gap: 12px;
  align-items: center;
  font: 400 12px 'Sobria Geist', sans-serif;
  color: #cfcfcf;
}
.brk-row .k { color: #a5a39a; }
.brk-row .bar { height: 6px; background: rgba(255, 255, 255, 0.04); border-radius: 3px; overflow: hidden; }
.brk-row .bar .fill { height: 100%; background: linear-gradient(90deg, #7a9a32, #c5f04a); border-radius: 3px; }
.brk-row .v { font-family: 'Sobria Mono', monospace; color: #f0ece3; text-align: right; font-size: 11px; }
.brk-tot {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #1f2620;
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}
.brk-tot .l {
  font: 500 10px 'Sobria Geist', sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: #a5a39a;
}
.brk-tot .v {
  font: 400 20px 'Sobria Instrument', serif;
  font-style: italic;
  color: #c5f04a;
}
.brk-tot .v .u {
  font: 400 11px 'Sobria Geist', sans-serif;
  font-style: normal;
  color: #a5a39a;
  margin-left: 3px;
}

/* Cumul session */
.spark-wrap {
  padding: 14px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid #1f2620;
  border-radius: 10px;
  margin-bottom: 12px;
}
.spark-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 8px;
}
.spark-head .l {
  font: 500 10px 'Sobria Geist', sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: #a5a39a;
}
.spark-head .v {
  font: 400 22px 'Sobria Instrument', serif;
  font-style: italic;
  color: #c5f04a;
}
.spark-head .v .u {
  font: 400 10px 'Sobria Geist', sans-serif;
  font-style: normal;
  color: #a5a39a;
  margin-left: 3px;
}
.cum-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px dashed #1f2620;
  font: 400 12px 'Sobria Geist', sans-serif;
  color: #cfcfcf;
}
.cum-row:last-child { border: none; }
.cum-row .k { color: #a5a39a; }
.cum-row .v { font-family: 'Sobria Mono', monospace; font-size: 11px; color: #f0ece3; }

/* Méthode */
.method { display: flex; flex-direction: column; gap: 12px; font: 400 12px/1.5 'Sobria Geist', sans-serif; color: #cfcfcf; }
.method .src {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 10px 12px;
  background: rgba(126, 182, 255, 0.04);
  border: 1px dashed rgba(126, 182, 255, 0.2);
  border-radius: 8px;
}
.method .src .ic {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: rgba(126, 182, 255, 0.08);
  display: grid;
  place-items: center;
  color: #7eb6ff;
  flex-shrink: 0;
}
.method .src .ic svg { width: 14px; height: 14px; }
.method .src .b { flex: 1; }
.method .src .nm { font: 500 11px 'Sobria Geist', sans-serif; color: #f0ece3; margin-bottom: 2px; }
.method .src .dt { font: 400 10px 'Sobria Mono', monospace; color: #7eb6ff; }
.method .src .ds { font: 400 11px 'Sobria Geist', sans-serif; color: #a5a39a; margin-top: 3px; font-style: italic; }
.method .formula {
  font-family: 'Sobria Mono', monospace;
  font-size: 11px;
  background: rgba(255, 255, 255, 0.025);
  border: 1px solid #1f2620;
  border-radius: 8px;
  padding: 12px 14px;
  color: #c5f04a;
  line-height: 1.7;
}
.method .formula .com { color: #72706a; }

/* Footer */
.sb-foot {
  padding: 12px 18px;
  background: #070908;
  border-top: 1px solid #1f2620;
  display: flex;
  align-items: center;
  gap: 10px;
  font: 400 10px 'Sobria Geist', sans-serif;
  color: #a5a39a;
  font-style: italic;
}
.sb-foot .pill {
  background: rgba(197, 240, 74, 0.06);
  border: 1px solid rgba(197, 240, 74, 0.2);
  color: #c5f04a;
  padding: 3px 8px;
  border-radius: 99px;
  font: 500 9px 'Sobria Mono', monospace;
  font-style: normal;
  letter-spacing: 0.02em;
}
.sb-foot .grow { flex: 1; }
</style>

<button class="sb-btn" data-sobria-action="toggle" type="button" aria-expanded="false">
  <span class="leaf-ic">${MARK_SVG}</span>
  <span class="val">${gco2} g CO₂eq</span>
  <span class="by">Sobr.ia</span>
  <span class="chev" aria-hidden="true">▾</span>
</button>

<div class="sb-pop" role="region" aria-label="Détail empreinte Sobr.ia">
  <div class="head">
    <span class="rich-mark">${MARK_RICH_SVG}</span>
    <span class="eye">Empreinte de cette réponse</span>
    <span class="live">par Sobr.ia · LIVE</span>
    <button class="close" data-sobria-action="close" type="button" aria-label="Fermer le détail">×</button>
  </div>

  <div class="sb-hero">
    <div>
      <div class="lab">CO₂ équivalent · cette réponse</div>
      <div class="big">${gco2}<span class="u">g CO₂eq</span></div>
      <div class="ci">P5 ${p5} ─ P95 ${p95} · IC 90 %</div>
    </div>
    <div class="gauge">
      <svg width="90" height="90" viewBox="0 0 90 90">
        <circle cx="45" cy="45" r="36" fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="6"/>
        <circle cx="45" cy="45" r="36" fill="none" stroke="#c5f04a" stroke-width="6"
          stroke-dasharray="${gaugeCircumference}" stroke-dashoffset="${gaugeOffset}"
          stroke-linecap="round" filter="drop-shadow(0 0 6px rgba(197,240,74,0.5))"/>
      </svg>
      <div class="gr">${letter}</div>
      <div class="gl">Score</div>
    </div>
  </div>

  <div class="sb-tabs" role="tablist">
    <button class="sb-tab on" data-sobria-tab="equiv" type="button">Équivalents</button>
    <button class="sb-tab" data-sobria-tab="detail" type="button">Détail</button>
    <button class="sb-tab" data-sobria-tab="cumul" type="button">Cumul session</button>
    <button class="sb-tab" data-sobria-tab="method" type="button">Méthode</button>
  </div>

  <div class="sb-body on" data-sobria-body="equiv">
    <div class="eq-grid">${equivTiles}</div>
    <div class="meta-grid">
      <div class="meta"><div class="k">Énergie</div><div class="v">${energy}<span class="u">Wh</span></div></div>
      <div class="meta"><div class="k">Eau</div><div class="v">${water}<span class="u">mL</span></div></div>
      <div class="meta"><div class="k">Tokens</div><div class="v">${tokensIn}<span class="u">/ ~${tokensOut}</span></div></div>
    </div>
  </div>

  <div class="sb-body" data-sobria-body="detail">
    <div class="brk">
      <div class="brk-row"><span class="k">Usage (énergie × IF)</span><div class="bar"><div class="fill" style="width:${Math.min(95, (estimate.gco2eqUsage / estimate.gco2eq) * 100)}%"></div></div><span class="v">${usage} g</span></div>
      <div class="brk-row"><span class="k">Embodied amorti</span><div class="bar"><div class="fill" style="width:${Math.min(95, (estimate.gco2eqEmbodied / Math.max(estimate.gco2eq, 1e-9)) * 100)}%"></div></div><span class="v">${embodied} g</span></div>
    </div>
    <div class="brk-tot">
      <span class="l">Total livré</span>
      <span class="v">${gco2} <span class="u">g CO₂eq</span></span>
    </div>
    <div class="meta-grid" style="margin-top:14px">
      <div class="meta"><div class="k">Modèle</div><div class="v">${estimate.modelId}</div></div>
      <div class="meta"><div class="k">Tokens in</div><div class="v">${tokensIn}</div></div>
      <div class="meta"><div class="k">Tokens out</div><div class="v">~${tokensOut}</div></div>
    </div>
  </div>

  <div class="sb-body" data-sobria-body="cumul">
    <div class="spark-wrap">
      <div class="spark-head">
        <span class="l">Aujourd'hui · ${sessionTotal.count} prompt${sessionTotal.count > 1 ? 's' : ''}</span>
        <span class="v">${sessionGco2}<span class="u">g CO₂eq</span></span>
      </div>
    </div>
    <div class="cum-row"><span class="k">Énergie totale</span><span class="v">${sessionEnergy} Wh</span></div>
    <div class="cum-row"><span class="k">Eau utilisée</span><span class="v">${sessionWater} mL</span></div>
    <div class="cum-row"><span class="k">Tokens cumulés</span><span class="v">~${sessionTokensIn} in / ~${sessionTokensOut} out</span></div>
    <div class="cum-row"><span class="k">Équivalent voiture</span><span class="v">~ ${sessionKmCar} km</span></div>
  </div>

  <div class="sb-body" data-sobria-body="method">
    <div class="method">
      <div class="formula"><span class="com">// Estimation simplifiée (${methodLabel})</span><br>
        CO₂ = (Wh<sub>compute</sub> × PUE) × mix<sub>g/kWh</sub>
      </div>
      ${estimate.notes
        .map(
          (note) => `<div class="src">
        <div class="ic"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg></div>
        <div class="b"><div class="nm">${note}</div></div>
      </div>`
        )
        .join('')}
    </div>
  </div>

  <div class="sb-foot">
    <span class="pill">🔒 100 % local</span>
    <span>Aucune donnée n'est envoyée. Calcul fait dans ton navigateur.</span>
  </div>
</div>
`.trim();
}

/**
 * Injecte le badge dans la rangée d'actions du message bot.
 *
 * `target` peut être :
 *  - La rangée d'actions du bot → on append le badge à l'intérieur.
 *  - Le message bot lui-même → fallback, append à la fin du message.
 */
export function injectBadge(
  target: Element,
  estimate: Estimate,
  options: { session?: DailyTotal | null; onClick?: () => void } = {}
): HTMLElement {
  // Supprime un badge frère/intérieur existant pour éviter les doublons.
  const existing = target.querySelector(`[${HOST_ATTR}]`);
  existing?.remove();
  const adjacent =
    target.nextElementSibling?.matches?.(`[${HOST_ATTR}]`) === true
      ? target.nextElementSibling
      : null;
  adjacent?.remove();

  const host = document.createElement('span');
  host.setAttribute(HOST_ATTR, '1');
  host.style.display = 'inline-flex';
  host.style.marginLeft = '6px';
  host.style.verticalAlign = 'middle';

  const shadow = host.attachShadow({ mode: 'open' });
  shadow.innerHTML = renderBadgeMarkup(estimate, options.session ?? null);

  // ─── Repositionnement en flux à l'ouverture ────────────────────────────
  //
  // À l'ouverture du popout, on déplace le host (qui contient bouton +
  // popout dans son shadow) **juste après la rangée d'actions** dans le
  // corps du message bot. Le host devient `display: block`, le popout
  // s'affiche en-dessous du bouton, le chat s'adapte naturellement
  // (push down du contenu suivant). Pas de `position: fixed` :
  // **la conversation peut scroller**, le badge suit le scroll.
  //
  // À la fermeture, le host repart dans la rangée d'actions, en pill inline.

  let savedParent: HTMLElement | null = null;
  let savedNextSibling: Node | null = null;
  let savedHostStyle: {
    display: string;
    width: string;
    margin: string;
    marginLeft: string;
    marginTop: string;
    verticalAlign: string;
  } | null = null;

  /**
   * Cherche un parent où afficher le host en bloc (sibling après la row
   * d'actions). Stratégie : remonter au premier conteneur qui n'est pas
   * un flex inline row (i.e., qui peut accueillir un block sans casser
   * le layout horizontal des autres boutons d'action).
   */
  function findBlockContainer(): { parent: HTMLElement; insertBefore: Node | null } | null {
    let cursor: HTMLElement | null = host.parentElement;
    while (cursor && cursor !== document.body) {
      const style = getComputedStyle(cursor);
      // Préférer un parent en flex-direction: column (corps du message bot).
      if (style.flexDirection === 'column' || style.display === 'block') {
        // On insère après le frère qui contient le host (typiquement
        // l'actionsRow lui-même).
        const ancestorOfHost =
          cursor === host.parentElement ? host : findChildContaining(cursor, host);
        const insertBefore = ancestorOfHost?.nextSibling ?? null;
        return { parent: cursor, insertBefore };
      }
      cursor = cursor.parentElement;
    }
    return null;
  }

  function findChildContaining(parent: HTMLElement, node: Node): HTMLElement | null {
    for (const child of Array.from(parent.children)) {
      if (child.contains(node)) return child as HTMLElement;
    }
    return null;
  }

  function expandHostInline(): void {
    if (savedParent) return; // déjà déplacé
    const container = findBlockContainer();
    if (!container) return; // pas de parent block trouvé — fallback : on garde le host en place
    savedParent = host.parentElement;
    savedNextSibling = host.nextSibling;
    savedHostStyle = {
      display: host.style.display,
      width: host.style.width,
      margin: host.style.margin,
      marginLeft: host.style.marginLeft,
      marginTop: host.style.marginTop,
      verticalAlign: host.style.verticalAlign
    };
    container.parent.insertBefore(host, container.insertBefore);
    // Block flow, pleine largeur du conteneur message → la conversation
    // se push naturellement vers le bas.
    host.style.display = 'block';
    host.style.width = '100%';
    host.style.margin = '8px 0 0';
    host.style.marginLeft = '0';
    host.style.marginTop = '8px';
    host.style.verticalAlign = 'baseline';
  }

  function collapseHostInline(): void {
    if (!savedParent || !savedHostStyle) return;
    Object.assign(host.style, savedHostStyle);
    if (savedParent.isConnected) {
      if (savedNextSibling && (savedNextSibling as Node).parentNode === savedParent) {
        savedParent.insertBefore(host, savedNextSibling);
      } else {
        savedParent.appendChild(host);
      }
    }
    savedParent = null;
    savedNextSibling = null;
    savedHostStyle = null;
  }

  /**
   * Reset des styles inline du popout (pas de position fixed).
   * Le popout s'affiche en flux normal sous le bouton, plein largeur du host.
   * La conversation scroll naturellement pour le voir entièrement.
   */
  function positionPop(): void {
    const pop = shadow.querySelector<HTMLElement>('.sb-pop');
    if (!pop) return;
    pop.style.top = '';
    pop.style.right = '';
    pop.style.left = '';
    pop.style.maxHeight = '';
  }

  /** Outside-click : ferme le popout si l'utilisateur clique en dehors du host. */
  let outsideClickHandler: ((ev: Event) => void) | null = null;
  function attachOutsideClick(): void {
    outsideClickHandler = (ev: Event) => {
      const path = ev.composedPath();
      if (!path.includes(host)) closePop();
    };
    document.addEventListener('click', outsideClickHandler, { capture: true });
  }
  function detachOutsideClick(): void {
    if (outsideClickHandler) {
      document.removeEventListener('click', outsideClickHandler, { capture: true });
      outsideClickHandler = null;
    }
  }

  function openPop(): void {
    // Repositionne le host en flux **dans la conversation** (sibling après la
    // rangée d'actions) — la chat s'adapte naturellement, plus de position:fixed.
    expandHostInline();
    host.setAttribute('data-expanded', '1');
    const btn = shadow.querySelector('.sb-btn');
    btn?.classList.add('open');
    btn?.setAttribute('aria-expanded', 'true');
    positionPop();
    attachOutsideClick();
    // Scroll auto pour que le popout soit visible après expansion.
    requestAnimationFrame(() => {
      host.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    });
  }
  function closePop(): void {
    host.setAttribute('data-expanded', '0');
    const btn = shadow.querySelector('.sb-btn');
    btn?.classList.remove('open');
    btn?.setAttribute('aria-expanded', 'false');
    detachOutsideClick();
    collapseHostInline();
  }

  shadow.addEventListener('click', (event) => {
    const el = event.target as HTMLElement | null;
    const action = el?.closest('[data-sobria-action]')?.getAttribute('data-sobria-action');
    const tab = el?.closest('[data-sobria-tab]')?.getAttribute('data-sobria-tab');

    if (action === 'toggle') {
      // Le clic sur le pill compte comme une interaction utilisateur :
      // l'onClick est appelé à chaque toggle (open ET close), conformément
      // au contrat consommé par les content scripts (compteur d'analytics
      // local par exemple).
      const expanded = host.getAttribute('data-expanded') === '1';
      if (expanded) closePop();
      else openPop();
      options.onClick?.();
    } else if (action === 'close') {
      closePop();
    } else if (tab) {
      shadow.querySelectorAll('.sb-tab').forEach((t) => t.classList.remove('on'));
      shadow.querySelectorAll('.sb-body').forEach((b) => b.classList.remove('on'));
      shadow.querySelector(`[data-sobria-tab="${tab}"]`)?.classList.add('on');
      shadow.querySelector(`[data-sobria-body="${tab}"]`)?.classList.add('on');
    }
  });

  // Si target est une rangée d'actions → append à la fin (frère des autres boutons).
  // Sinon → append comme dernier enfant (le badge se cale en bas du contenu).
  target.appendChild(host);
  return host;
}

/**
 * Injecte un badge dégradé « Modèle non pris en charge » quand `extractModelId`
 * n'a pas reconnu le modèle. Pas de chiffre, pas de popout : juste un pill
 * informatif pour ne pas induire en erreur l'utilisateur.
 *
 * Le pill apparaît dans la même rangée d'actions que le badge normal — design
 * cohérent (mark Sobr.ia + texte + séparateur Sobr.ia), mais ton neutre
 * (ivory au lieu de lime) et icône « ? » au lieu de la valeur.
 */
export function injectUnsupportedBadge(
  target: Element,
  options: { modelLabel?: string } = {}
): HTMLElement {
  // Supprime un badge existant pour éviter les doublons.
  const existing = target.querySelector(`[${HOST_ATTR}]`);
  existing?.remove();
  const adjacent =
    target.nextElementSibling?.matches?.(`[${HOST_ATTR}]`) === true
      ? target.nextElementSibling
      : null;
  adjacent?.remove();

  const host = document.createElement('span');
  host.setAttribute(HOST_ATTR, '1');
  host.setAttribute('data-sobria-unsupported', '1');
  host.style.display = 'inline-flex';
  host.style.marginLeft = '6px';
  host.style.verticalAlign = 'middle';

  const shadow = host.attachShadow({ mode: 'open' });
  const fontFace = renderFontFace();
  const tooltipText = options.modelLabel
    ? `Modèle « ${options.modelLabel} » pas encore dans le registry Sobr.ia v0.7.0`
    : 'Ce modèle n’est pas encore dans le registry Sobr.ia v0.7.0';

  shadow.innerHTML = `
<style>
${fontFace}
:host {
  all: initial;
  display: inline-flex;
  font-family: 'Sobria Geist', system-ui, -apple-system, 'Segoe UI', sans-serif;
  font-variant-numeric: tabular-nums;
}
.sb-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 11px 5px 8px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px dashed rgba(255, 255, 255, 0.2);
  border-radius: 999px;
  color: rgba(255, 255, 255, 0.55);
  font: 500 11px/1 'Sobria Geist', system-ui, sans-serif;
  cursor: help;
  position: relative;
}
.leaf {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
  display: inline-flex;
  color: rgba(255, 255, 255, 0.4);
}
.leaf svg { width: 100%; height: 100%; }
.q {
  font-weight: 600;
  font-size: 12px;
  margin-left: 2px;
}
.lab {
  letter-spacing: 0.02em;
}
.by {
  font: 400 9px 'Sobria Geist', sans-serif;
  color: rgba(255, 255, 255, 0.35);
  letter-spacing: 0.04em;
  margin-left: 4px;
  border-left: 1px solid rgba(255, 255, 255, 0.18);
  padding-left: 6px;
  text-transform: uppercase;
}
.tooltip {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  background: #0a0d0b;
  color: #f0ece3;
  font: 400 10px/1.4 'Sobria Geist', sans-serif;
  padding: 6px 10px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 6px 14px rgba(0, 0, 0, 0.4);
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 180ms ease-out;
  z-index: 10;
}
.sb-pill:hover .tooltip,
.sb-pill:focus .tooltip { opacity: 1; }
</style>
<span class="sb-pill" role="img" tabindex="0" aria-label="${tooltipText}">
  <span class="leaf">${MARK_SVG}</span>
  <span class="q">?</span>
  <span class="lab">Modèle non pris en charge</span>
  <span class="by">Sobr.ia</span>
  <span class="tooltip">${tooltipText}</span>
</span>
`.trim();

  target.appendChild(host);
  return host;
}

/** Retire tous les badges Sobr.ia présents sous `root`. */
export function removeAllBadges(root: Element | Document = document): void {
  root.querySelectorAll(`[${HOST_ATTR}]`).forEach((el) => el.remove());
}

/** Exposé pour les tests / debug : ratio gauge selon gco2eq. */
export { pickGrade };

/** @deprecated alias historique pour compat tests existants (jusqu'à C27.4). */
export function pickTone(gco2eq: number): 'lime' | 'amber' | 'coral' {
  if (gco2eq < 1.0) return 'lime';
  if (gco2eq < 5.0) return 'amber';
  return 'coral';
}
