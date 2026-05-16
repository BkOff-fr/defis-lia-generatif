// Formattage de nombres + dates pour l'affichage dashboard.
// Pas de dépendance Intl wrapper — l'API navigateur est suffisante.

/** gCO₂eq → string compact (mg/g/kg). */
export function formatCO2(grams: number): { value: string; unit: string } {
  if (!Number.isFinite(grams)) return { value: '—', unit: '' };
  if (grams >= 1000) return { value: (grams / 1000).toFixed(2), unit: 'kgCO₂eq' };
  if (grams >= 1) return { value: grams.toFixed(2), unit: 'gCO₂eq' };
  if (grams >= 0.001) return { value: (grams * 1000).toFixed(1), unit: 'mgCO₂eq' };
  return { value: grams.toExponential(1), unit: 'gCO₂eq' };
}

/** Watt-heures → Wh / kWh / MWh. */
export function formatEnergy(wh: number): { value: string; unit: string } {
  if (!Number.isFinite(wh)) return { value: '—', unit: '' };
  if (wh >= 1_000_000) return { value: (wh / 1_000_000).toFixed(2), unit: 'MWh' };
  if (wh >= 1000) return { value: (wh / 1000).toFixed(2), unit: 'kWh' };
  if (wh >= 1) return { value: wh.toFixed(2), unit: 'Wh' };
  return { value: (wh * 1000).toFixed(1), unit: 'mWh' };
}

/** Millilitres → mL / L. */
export function formatWater(ml: number): { value: string; unit: string } {
  if (!Number.isFinite(ml)) return { value: '—', unit: '' };
  if (ml >= 1000) return { value: (ml / 1000).toFixed(2), unit: 'L' };
  return { value: ml.toFixed(0), unit: 'mL' };
}

/** Compteur entier avec séparateur fr ("1 234 567"). */
export function formatCount(n: number): string {
  if (!Number.isFinite(n)) return '—';
  return Math.round(n).toLocaleString('fr-FR');
}

/** Date RFC3339 → "16 mai 2026 à 12:34" en fr-FR (Europe/Paris). */
export function formatDateTime(iso: string | null | undefined): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString('fr-FR', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

/** Date RFC3339 → "16 mai 2026" sans heure. */
export function formatDate(iso: string | null | undefined): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleDateString('fr-FR', {
    day: 'numeric',
    month: 'long',
    year: 'numeric'
  });
}

/** Génère un fingerprint stable pour le navigateur courant.
 *  Heuristique simple : ua + screen + timezone hash. Pas anti-spoof,
 *  c'est un identifiant ergonomique pour /enroll, pas une preuve. */
export function browserFingerprint(): string {
  const ua = navigator.userAgent || 'unknown';
  const platform = (navigator as { platform?: string }).platform || 'unknown';
  const screen = `${window.screen?.width || 0}x${window.screen?.height || 0}`;
  const tz = Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
  const raw = `${ua}|${platform}|${screen}|${tz}`;
  // Hash FNV-like, 8 chars hex.
  let h = 0x811c9dc5;
  for (let i = 0; i < raw.length; i++) {
    h ^= raw.charCodeAt(i);
    h = (h * 0x01000193) >>> 0;
  }
  const platformShort = platform.toLowerCase().replace(/[^a-z]/g, '').slice(0, 8) || 'web';
  return `web-${platformShort}-${h.toString(16).padStart(8, '0')}`;
}
