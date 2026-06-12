import { describe, it, expect } from 'vitest';
import pkg from '../../package.json' with { type: 'json' };
import {
  EXTENSION_VERSION,
  REGISTRY_VERSION,
  REGISTRY_MODEL_COUNT,
  registryLabel,
  unsupportedModelTooltip
} from '../../src/lib/registry-meta.js';
import { MODEL_PRESETS } from '../../src/lib/presets.js';

describe('registry-meta', () => {
  it('aligne la version UI sur package.json', () => {
    expect(EXTENSION_VERSION).toBe(pkg.version);
    expect(REGISTRY_VERSION).toBe(pkg.version);
  });

  it('expose le nombre de presets du registry embarqué', () => {
    expect(REGISTRY_MODEL_COUNT).toBe(MODEL_PRESETS.length);
    expect(REGISTRY_MODEL_COUNT).toBeGreaterThanOrEqual(33);
  });

  it('formate le libellé registry pour les tooltips', () => {
    expect(registryLabel()).toBe(`registry Sobr.ia v${pkg.version}`);
    expect(unsupportedModelTooltip()).toContain(pkg.version);
    expect(unsupportedModelTooltip('GPT-5')).toContain('GPT-5');
    expect(unsupportedModelTooltip()).toContain(String(REGISTRY_MODEL_COUNT));
  });
});
