// Sobr.ia extension — tests helpers popup (bilan du jour, C43).

import { describe, it, expect } from 'vitest';

import {
  carEquivalentLabel,
  fmtRelative,
  linkLabel,
  resolveLinkState,
  todaySubline,
  toneOf
} from '../../src/popup/main.js';
import type { DailyTotal } from '../../src/lib/messages.js';

function total(overrides: Partial<DailyTotal> = {}): DailyTotal {
  return {
    date: '2026-06-12',
    count: 7,
    gco2eq: 12.4,
    waterMl: 96.2,
    energyWh: 31.5,
    ...overrides
  };
}

describe('resolveLinkState — où partent les mesures', () => {
  it('rien → browser', () => {
    expect(resolveLinkState(false, false)).toBe('browser');
  });
  it('pairing app seul → app', () => {
    expect(resolveLinkState(true, false)).toBe('app');
  });
  it('serveur équipe seul → team', () => {
    expect(resolveLinkState(false, true)).toBe('team');
  });
  it('les deux → both', () => {
    expect(resolveLinkState(true, true)).toBe('both');
  });
});

describe('linkLabel — une ligne claire, vouvoiement', () => {
  it('browser : message local + pas de tutoiement', () => {
    expect(linkLabel('browser')).toBe('Vos mesures restent dans ce navigateur.');
  });
  it('app : mentionne l’app desktop', () => {
    expect(linkLabel('app')).toContain('app Sobr.ia');
  });
  it('team : mentionne le serveur équipe', () => {
    expect(linkLabel('team')).toContain('équipe');
  });
  it('both : mentionne les deux destinations', () => {
    const label = linkLabel('both');
    expect(label).toContain('app');
    expect(label).toContain('équipe');
  });
});

describe('todaySubline — sous-ligne du chiffre du jour', () => {
  it('pluriel + équivalent voiture', () => {
    expect(todaySubline(total())).toBe('7 prompts mesurés · ≈ 65 m en voiture');
  });
  it('singulier pour 1 prompt', () => {
    expect(todaySubline(total({ count: 1, gco2eq: 1.92 }))).toContain('1 prompt mesuré');
  });
});

describe('carEquivalentLabel — équivalent distance (ADEME 192 g/km)', () => {
  it('petites valeurs en mètres', () => {
    expect(carEquivalentLabel(1.92)).toBe('≈ 10 m en voiture');
  });
  it('valeur sous le mètre', () => {
    expect(carEquivalentLabel(0.1)).toBe('moins d’un mètre en voiture');
  });
  it('grosses valeurs en kilomètres', () => {
    expect(carEquivalentLabel(384)).toBe('≈ 2 km en voiture');
  });
});

describe('fmtRelative — horodatage relatif FR', () => {
  const now = new Date('2026-06-12T12:00:00Z').getTime();
  it('« à l’instant » sous la minute', () => {
    expect(fmtRelative('2026-06-12T11:59:40Z', now)).toBe('à l’instant');
  });
  it('minutes', () => {
    expect(fmtRelative('2026-06-12T11:53:00Z', now)).toBe('il y a 7 min');
  });
  it('heures', () => {
    expect(fmtRelative('2026-06-12T09:00:00Z', now)).toBe('il y a 3 h');
  });
});

describe('toneOf — cohérence notes A-F', () => {
  it('A/B → lime', () => {
    expect(toneOf(0.4)).toBe('lime');
  });
  it('C/D → amber', () => {
    expect(toneOf(2)).toBe('amber');
  });
  it('E/F → coral', () => {
    expect(toneOf(20)).toBe('coral');
  });
});
