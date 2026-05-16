// Sobr.ia extension — tests badge-injector (design 38, C27.3 v3).

import { describe, it, expect, beforeEach, vi } from 'vitest';

import {
  injectBadge,
  pickGrade,
  removeAllBadges
} from '../../src/content/shared/badge-injector.js';
import type { Estimate } from '../../src/lib/types.js';

function buildEstimate(overrides: Partial<Estimate> = {}): Estimate {
  return {
    method: 'afnor_sobria',
    modelId: 'llama-3-1-70b',
    tokensIn: 100,
    tokensOut: 500,
    gco2eq: 2.14,
    gco2eqUsage: 1.8,
    gco2eqEmbodied: 0.34,
    waterMl: 18.3,
    energyWh: 5.46,
    equivalents: [
      { label: 'en voiture thermique', value: 13, unit: 'm', icon: 'car', source: 'ADEME' },
      { label: 'de douche chaude', value: 2.4, unit: 'sec', icon: 'shower', source: 'ADEME' },
      { label: 'ampoule LED 9W', value: 38, unit: 'min', icon: 'led', source: 'LED 9W' },
      {
        label: 'recharge smartphone',
        value: 0,
        unit: 'charge',
        display: '~ 1/6',
        icon: 'phone',
        source: 'smartphone 15 Wh'
      }
    ],
    notes: ['AFNOR test'],
    ...overrides
  };
}

beforeEach(() => {
  document.body.innerHTML = '';
});

describe('pickGrade — score A-F', () => {
  it('< 1 g → A', () => {
    expect(pickGrade(0.5).letter).toBe('A');
  });
  it('1-3 g → B', () => {
    expect(pickGrade(2.14).letter).toBe('B');
  });
  it('3-5 g → C', () => {
    expect(pickGrade(4).letter).toBe('C');
  });
  it('5-10 g → D', () => {
    expect(pickGrade(7).letter).toBe('D');
  });
  it('10-20 g → E', () => {
    expect(pickGrade(15).letter).toBe('E');
  });
  it('>= 20 g → F', () => {
    expect(pickGrade(30).letter).toBe('F');
  });
  it('ratio remplissage croissant avec gco2eq', () => {
    expect(pickGrade(0.5).ratio).toBeLessThan(pickGrade(30).ratio);
  });
});

describe('injectBadge — insertion dans row actions du bot', () => {
  it("append le badge comme dernier enfant de la row d'actions", () => {
    const row = document.createElement('div');
    row.className = 'actions';
    document.body.appendChild(row);

    const badge = injectBadge(row, buildEstimate());

    expect(badge.getAttribute('data-sobria-badge')).toBe('1');
    expect(row.lastElementChild).toBe(badge);
    expect(badge.shadowRoot).not.toBeNull();
  });

  it('remplace un badge existant dans la même row (pas de duplication)', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);

    injectBadge(row, buildEstimate({ gco2eq: 0.5 }));
    injectBadge(row, buildEstimate({ gco2eq: 12 }));

    const all = row.querySelectorAll('[data-sobria-badge]');
    expect(all.length).toBe(1);
    // Vérifie qu'on a bien le nouveau (gauge ratio plus haut pour 12 g).
    const text = (all[0] as HTMLElement).shadowRoot!.textContent ?? '';
    expect(text).toContain('12');
  });
});

describe('injectBadge — button compact (design 38)', () => {
  it('affiche le bouton "[mark] X g CO₂eq | Sobr.ia ▾"', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());

    const btn = badge.shadowRoot!.querySelector('.sb-btn');
    expect(btn).not.toBeNull();
    expect(btn?.textContent ?? '').toContain('2,14');
    expect(btn?.textContent ?? '').toContain('CO₂eq');
    expect(btn?.textContent ?? '').toContain('Sobr.ia');
  });

  it("le button contient un SVG mark inline (pas d'emoji)", () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());

    const svg = badge.shadowRoot!.querySelector('.sb-btn svg');
    expect(svg).not.toBeNull();
    expect(badge.shadowRoot!.innerHTML).not.toContain('🌱');
  });
});

describe('injectBadge — popout 4 onglets (design 38)', () => {
  it('popout caché par défaut', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());
    expect(badge.getAttribute('data-expanded')).not.toBe('1');
  });

  it('ouvre le popout au clic sur le button', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());
    const btn = badge.shadowRoot!.querySelector<HTMLButtonElement>('.sb-btn')!;

    btn.dispatchEvent(new Event('click', { bubbles: true }));

    expect(badge.getAttribute('data-expanded')).toBe('1');
    expect(btn.classList.contains('open')).toBe(true);
  });

  it('expose les 4 onglets : Équivalents / Détail / Cumul / Méthode', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());
    const tabs = badge.shadowRoot!.querySelectorAll('.sb-tab');
    expect(tabs.length).toBe(4);
    const labels = Array.from(tabs).map((t) => t.textContent?.trim());
    expect(labels).toEqual(['Équivalents', 'Détail', 'Cumul session', 'Méthode']);
  });

  it("changer d'onglet active le bon body", () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());
    const detailTab = badge.shadowRoot!.querySelector<HTMLButtonElement>(
      '[data-sobria-tab="detail"]'
    )!;

    detailTab.dispatchEvent(new Event('click', { bubbles: true }));

    expect(detailTab.classList.contains('on')).toBe(true);
    const detailBody = badge.shadowRoot!.querySelector('[data-sobria-body="detail"]');
    expect(detailBody?.classList.contains('on')).toBe(true);
  });

  it('Hero metric utilise Instrument Serif italic + jauge SVG', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());
    const hero = badge.shadowRoot!.querySelector('.sb-hero');
    expect(hero).not.toBeNull();
    expect(hero?.textContent ?? '').toContain('2,14');
    expect(hero?.textContent ?? '').toContain('IC 90');
    // Score B attendu pour 2.14 g
    expect(badge.shadowRoot!.querySelector('.gauge .gr')?.textContent).toBe('B');
    // Gauge SVG circle present
    expect(badge.shadowRoot!.querySelectorAll('.gauge svg circle').length).toBe(2);
  });

  it('referme le popout au clic sur close', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());
    const btn = badge.shadowRoot!.querySelector<HTMLButtonElement>('.sb-btn')!;
    btn.dispatchEvent(new Event('click', { bubbles: true }));
    expect(badge.getAttribute('data-expanded')).toBe('1');

    const close = badge.shadowRoot!.querySelector<HTMLButtonElement>(
      '[data-sobria-action="close"]'
    )!;
    close.dispatchEvent(new Event('click', { bubbles: true }));

    expect(badge.getAttribute('data-expanded')).toBe('0');
  });
});

describe('injectBadge — équivalents (4 tiles)', () => {
  it('rend 4 tiles dans la grille avec icône + label sans unité + valeur+unité', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());

    const tiles = badge.shadowRoot!.querySelectorAll('.eq');
    expect(tiles.length).toBe(4);

    const text = badge.shadowRoot!.querySelector('[data-sobria-body="equiv"]')?.textContent ?? '';
    expect(text).toContain('en voiture thermique');
    expect(text).toContain('de douche chaude');
    expect(text).toContain('ampoule LED 9W');
    expect(text).toContain('recharge smartphone');
    // Valeurs avec unité (pas double-unité)
    expect(text).toContain('13 ');
    expect(text).toContain(' m');
    expect(text).toContain('38 ');
    expect(text).toContain(' min');
    expect(text).toContain('~ 1/6');
  });

  it("ne duplique pas l'unité dans le label (bug pré-design-38)", () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate());
    const text = badge.shadowRoot!.querySelector('[data-sobria-body="equiv"]')?.textContent ?? '';
    // Avant la refonte les labels contenaient "km en voiture thermique" → double unité.
    expect(text).not.toContain('km en voiture');
    expect(text).not.toContain('secondes de douche');
    expect(text).not.toContain("heures d'écran");
  });
});

describe('injectBadge — onglet Cumul session', () => {
  it('affiche le total session quand fourni', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const badge = injectBadge(row, buildEstimate(), {
      session: { date: '2026-05-16', count: 5, gco2eq: 14.2, waterMl: 32, energyWh: 34 }
    });
    const text = badge.shadowRoot!.querySelector('[data-sobria-body="cumul"]')?.textContent ?? '';
    expect(text).toContain('5 prompts');
    expect(text).toContain('14,2');
    expect(text).toContain('32');
    expect(text).toContain('34');
  });
});

describe('injectBadge — onClick callback', () => {
  it('appelle onClick à chaque toggle', () => {
    const row = document.createElement('div');
    document.body.appendChild(row);
    const onClick = vi.fn();
    const badge = injectBadge(row, buildEstimate(), { onClick });
    const btn = badge.shadowRoot!.querySelector<HTMLButtonElement>('.sb-btn')!;
    btn.dispatchEvent(new Event('click', { bubbles: true }));
    btn.dispatchEvent(new Event('click', { bubbles: true }));
    expect(onClick).toHaveBeenCalledTimes(2);
  });
});

describe('removeAllBadges', () => {
  it('purge tous les badges du document', () => {
    const r1 = document.createElement('div');
    const r2 = document.createElement('div');
    document.body.append(r1, r2);
    injectBadge(r1, buildEstimate());
    injectBadge(r2, buildEstimate({ gco2eq: 7 }));
    expect(document.querySelectorAll('[data-sobria-badge]').length).toBe(2);

    removeAllBadges();
    expect(document.querySelectorAll('[data-sobria-badge]').length).toBe(0);
  });
});
