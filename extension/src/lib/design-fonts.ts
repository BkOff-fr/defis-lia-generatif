// Sobr.ia extension — typo alignée sur web/src/app.css (app Tauri).
//
// Les content scripts injectent ces @font-face dans le Shadow DOM (mêmes
// familles et fichiers WOFF2 que popup/options via tokens.css).

/** Stack UI corps (Geist). */
export const FONT_UI = "'Geist', system-ui, -apple-system, 'Segoe UI', sans-serif";

/** Stack titres / métriques éditoriales (Instrument Serif italic). */
export const FONT_DISPLAY = "'Instrument Serif', 'Cormorant Garamond', Georgia, serif";

/** Stack chiffres / labels techniques (JetBrains Mono). */
export const FONT_MONO = "'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace";

/** OpenType Geist — identique app desktop. */
export const FONT_FEATURES_UI = "'ss01', 'cv11'";

/** Bloc :host partagé (shadow DOM badges + indicateur composer). */
export const SHADOW_HOST_TYPO = `
  font-family: ${FONT_UI};
  font-variant-numeric: tabular-nums;
  font-feature-settings: ${FONT_FEATURES_UI};
  -webkit-font-smoothing: antialiased;
  text-rendering: optimizeLegibility;
`;

function extensionUrl(path: string): string {
  try {
    const api = (globalThis as { chrome?: { runtime?: { getURL?: (p: string) => string } } }).chrome
      ?.runtime?.getURL;
    return api ? api(path) : '';
  } catch {
    return '';
  }
}

/**
 * @font-face complets pour le Shadow DOM (miroir extension/src/styles/tokens.css).
 */
export function renderShadowFontFaces(): string {
  const geistLatin = extensionUrl('fonts/geist-latin.woff2');
  const geistLatinExt = extensionUrl('fonts/geist-latin-ext.woff2');
  const instrumentLatin = extensionUrl('fonts/instrument-serif-latin.woff2');
  const instrumentLatinExt = extensionUrl('fonts/instrument-serif-latin-ext.woff2');
  const instrumentItalicLatin = extensionUrl('fonts/instrument-serif-italic-latin.woff2');
  const instrumentItalicLatinExt = extensionUrl('fonts/instrument-serif-italic-latin-ext.woff2');
  const monoLatin = extensionUrl('fonts/jetbrains-mono-latin.woff2');
  const monoLatinExt = extensionUrl('fonts/jetbrains-mono-latin-ext.woff2');

  if (!geistLatin) return '';

  const latinRange =
    'U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD';
  const latinExtRange =
    'U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF, U+0304, U+0308, U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020, U+20A0-20AB, U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF';

  const face = (family: string, style: string, weight: string, src: string, range: string) =>
    `@font-face{font-family:${family};font-style:${style};font-weight:${weight};font-display:swap;src:url('${src}') format('woff2');unicode-range:${range}}`;

  return [
    face('Geist', 'normal', '300 700', geistLatin, latinRange),
    geistLatinExt ? face('Geist', 'normal', '300 700', geistLatinExt, latinExtRange) : '',
    instrumentLatin ? face('Instrument Serif', 'normal', '400', instrumentLatin, latinRange) : '',
    instrumentLatinExt
      ? face('Instrument Serif', 'normal', '400', instrumentLatinExt, latinExtRange)
      : '',
    instrumentItalicLatin
      ? face('Instrument Serif', 'italic', '400', instrumentItalicLatin, latinRange)
      : '',
    instrumentItalicLatinExt
      ? face('Instrument Serif', 'italic', '400', instrumentItalicLatinExt, latinExtRange)
      : '',
    monoLatin ? face('JetBrains Mono', 'normal', '400 600', monoLatin, latinRange) : '',
    monoLatinExt ? face('JetBrains Mono', 'normal', '400 600', monoLatinExt, latinExtRange) : ''
  ]
    .filter(Boolean)
    .join('\n');
}
