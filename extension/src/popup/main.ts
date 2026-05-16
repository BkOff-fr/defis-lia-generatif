// Sobr.ia popup — bilan compact (C27.4).
//
// 3 sections :
//   - Carte « Dernier prompt » (modèle, métriques)
//   - Carte « Aujourd'hui » (compteur + totaux)
//   - Toggle méthodologie AFNOR ⇄ EcoLogits
//
// Source des données : `chrome.storage.local` (read direct, pas via SW pour
// rester réactif au reload popup). Toggle méthodo persisté via `setMethod`.

import './popup.css';
import { getMethod, setMethod, getLastEstimate, getTodayTotal } from '../content/shared/storage.js';
import { pickGrade } from '../lib/empreinte/grade.js';
import type { EmpreinteMethod } from '../lib/types.js';
import type { LastEstimate, DailyTotal } from '../lib/messages.js';

function fmtFr(n: number, digits = 2): string {
  return new Intl.NumberFormat('fr-FR', { maximumSignificantDigits: digits }).format(n);
}

function fmtRelative(iso: string): string {
  const now = Date.now();
  const ts = new Date(iso).getTime();
  const diff = Math.max(0, now - ts);
  const min = Math.floor(diff / 60_000);
  if (min < 1) return 'à l’instant';
  if (min < 60) return `il y a ${min} min`;
  const h = Math.floor(min / 60);
  if (h < 24) return `il y a ${h} h`;
  return new Date(iso).toLocaleDateString('fr-FR');
}

function toneOf(gco2eq: number): 'lime' | 'amber' | 'coral' {
  const g = pickGrade(gco2eq);
  if (g === 'A' || g === 'B') return 'lime';
  if (g === 'C' || g === 'D') return 'amber';
  return 'coral';
}

function renderLastPrompt(last: LastEstimate | null): void {
  const body = document.getElementById('last-prompt-body');
  const tsEl = document.getElementById('last-prompt-ts');
  if (!body) return;
  if (!last) {
    if (tsEl) tsEl.textContent = '—';
    return; // garde l'empty-state HTML d'origine
  }
  const e = last.estimate;
  const grade = pickGrade(e.gco2eq);
  if (tsEl) tsEl.textContent = fmtRelative(last.ts);
  body.innerHTML = `
    <div class="last-grade" data-tone="${toneOf(e.gco2eq)}">
      <span class="last-grade__badge">${grade}</span>
      <span class="last-metric">${fmtFr(e.gco2eq)}<span class="u">g CO₂eq</span></span>
    </div>
    <div class="last-model">${last.modelDisplayName} · ${last.host}</div>
    <div class="last-meta-row">
      <div class="last-meta"><span class="last-meta__k">Tokens</span><span class="last-meta__v">${e.tokensIn} → ~${e.tokensOut}</span></div>
      <div class="last-meta"><span class="last-meta__k">Eau</span><span class="last-meta__v">${fmtFr(e.waterMl)} mL</span></div>
      <div class="last-meta"><span class="last-meta__k">Énergie</span><span class="last-meta__v">${fmtFr(e.energyWh, 3)} Wh</span></div>
    </div>
  `;
}

function renderToday(total: DailyTotal): void {
  const countEl = document.getElementById('today-count');
  const gco2El = document.getElementById('today-gco2');
  const waterEl = document.getElementById('today-water');
  const energyEl = document.getElementById('today-energy');
  if (countEl) {
    countEl.textContent = `${total.count} prompt${total.count > 1 ? 's' : ''}`;
  }
  if (gco2El) gco2El.innerHTML = `${fmtFr(total.gco2eq)}<span class="u">g</span>`;
  if (waterEl) waterEl.innerHTML = `${fmtFr(total.waterMl)}<span class="u">mL</span>`;
  if (energyEl) energyEl.innerHTML = `${fmtFr(total.energyWh, 3)}<span class="u">Wh</span>`;
}

function renderMethodToggle(method: EmpreinteMethod): void {
  document.querySelectorAll<HTMLButtonElement>('.method-toggle__opt').forEach((btn) => {
    const checked = btn.dataset['method'] === method;
    btn.setAttribute('aria-checked', String(checked));
  });
}

async function init(): Promise<void> {
  const [method, last, today] = await Promise.all([
    getMethod(),
    getLastEstimate(),
    getTodayTotal()
  ]);

  renderLastPrompt(last);
  renderToday(today);
  renderMethodToggle(method);

  // Toggle méthodologie → persiste via storage.
  document.querySelectorAll<HTMLButtonElement>('.method-toggle__opt').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const next = btn.dataset['method'] as EmpreinteMethod;
      if (next !== 'afnor_sobria' && next !== 'ecologits') return;
      await setMethod(next);
      renderMethodToggle(next);
    });
  });

  // Bouton « Voir dans Sobr.ia » : disabled tant que pas de bridge (C27.5).
  // Aucune action pour l'instant.

  // Bouton « Réglages » → ouvre la page options.
  document.getElementById('open-options-btn')?.addEventListener('click', () => {
    chrome.runtime.openOptionsPage();
  });
}

document.addEventListener('DOMContentLoaded', () => {
  init().catch((err) => {
    console.error('[sobria popup] init failed:', err);
  });
});
