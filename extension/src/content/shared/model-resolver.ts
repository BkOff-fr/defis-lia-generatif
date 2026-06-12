// Sobr.ia — résolution libellé UI → id preset registry (partagé content scripts).

import { findPreset, MODEL_PRESETS } from '../../lib/presets.js';

/** Normalise un libellé DOM (casse, espaces, tirets Unicode). */
export function normalizeModelLabel(raw: string): string {
  return raw
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[\u2010-\u2015\u2212]/g, '-')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * Tente un identifiant preset direct (`gpt-5-5`, `claude-sonnet-4-6`, …).
 */
export function tryDirectPresetId(label: string): string | null {
  const norm = normalizeModelLabel(label);
  if (!norm) return null;
  const slug = norm.replace(/\s+/g, '-');
  if (findPreset(slug)) return slug;
  if (findPreset(norm)) return norm;
  return null;
}

/**
 * Résout un libellé via une table d'alias (clés = sous-chaînes, ordre = priorité d'insertion).
 */
export function resolveModelAlias(label: string, aliases: Record<string, string>): string | null {
  const direct = tryDirectPresetId(label);
  if (direct) return direct;

  const hay = normalizeModelLabel(label);
  if (!hay) return null;

  for (const [key, presetId] of Object.entries(aliases)) {
    if (hay.includes(normalizeModelLabel(key))) return presetId;
  }

  // Correspondance sur displayName / id des presets embarqués.
  for (const preset of MODEL_PRESETS) {
    const display = normalizeModelLabel(preset.displayName);
    const idSpaced = preset.id.replace(/-/g, ' ');
    if (hay.includes(display) || hay.includes(idSpaced)) {
      return preset.id;
    }
  }

  return null;
}

/** Collecte des libellés candidats depuis des sélecteurs (priorité = ordre du tableau). */
export function collectModelLabels(selectors: readonly string[]): string[] {
  const out: string[] = [];
  const seen = new Set<string>();
  for (const selector of selectors) {
    for (const el of document.querySelectorAll(selector)) {
      const parts = [
        el.getAttribute('data-model'),
        el.getAttribute('data-testid') === 'model-switcher-dropdown-button'
          ? (el.textContent ?? '')
          : null,
        el.textContent ?? '',
        el.getAttribute('aria-label') ?? ''
      ];
      for (const part of parts) {
        const trimmed = part?.trim();
        if (!trimmed || seen.has(trimmed)) continue;
        seen.add(trimmed);
        out.push(trimmed);
      }
    }
  }
  return out;
}

/**
 * Essaie chaque libellé jusqu'à obtenir un preset connu.
 * Retourne `{ presetId, label }` ou `null`.
 */
export function resolveModelFromLabels(
  labels: readonly string[],
  aliases: Record<string, string>
): { presetId: string; label: string } | null {
  for (const label of labels) {
    const presetId = resolveModelAlias(label, aliases);
    if (presetId && findPreset(presetId)) {
      return { presetId, label };
    }
  }
  return null;
}
