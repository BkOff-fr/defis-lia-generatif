// Sobr.ia popup — bilan du jour, hiérarchie en 3 niveaux (C43).
//
//   1. LE chiffre du jour : g CO₂eq cumulés aujourd'hui (+ équivalent voiture).
//   2. État d'association en une ligne : app desktop / serveur équipe /
//      navigateur seul → bouton « Associer » (ouvre les réglages).
//   3. Une rangée d'actions : « Voir dans Sobr.ia » + « Réglages ».
//
// Le reste (dernier prompt, eau/énergie, méthode de calcul) est replié dans
// « Détails du jour » pour rester lisible en 3 secondes.
//
// Source des données : `chrome.storage.local` (lecture directe, pas via SW,
// pour rester réactif au reload popup). Toggle méthodo persisté via `setMethod`.

import './popup.css';
import { applyVersionLabels } from '../lib/registry-meta.js';
import {
  getMethod,
  setMethod,
  getLastEstimate,
  getTodayTotal,
  getPairingState
} from '../content/shared/storage.js';
import { getTeamState } from '../content/shared/team-storage.js';
import {
  addProject,
  getProjectForThread,
  getProjectsList,
  setProjectForThread,
  threadKeyFromUrl
} from '../content/shared/projects.js';
import { pickGrade } from '../lib/empreinte/grade.js';
import type { EmpreinteMethod } from '../lib/types.js';
import type { DailyTotal, Host, LastEstimate } from '../lib/messages.js';

/** Formatage FR à `digits` chiffres significatifs (« 12 », « 1,8 »). */
export function fmtFr(n: number, digits = 2): string {
  return new Intl.NumberFormat('fr-FR', { maximumSignificantDigits: digits }).format(n);
}

/** Horodatage relatif en français (« à l’instant », « il y a 7 min »). */
export function fmtRelative(iso: string, now: number = Date.now()): string {
  const ts = new Date(iso).getTime();
  const diff = Math.max(0, now - ts);
  const min = Math.floor(diff / 60_000);
  if (min < 1) return 'à l’instant';
  if (min < 60) return `il y a ${min} min`;
  const h = Math.floor(min / 60);
  if (h < 24) return `il y a ${h} h`;
  return new Date(iso).toLocaleDateString('fr-FR');
}

/** Ton couleur selon la note A-F (cohérent badge in-page). */
export function toneOf(gco2eq: number): 'lime' | 'amber' | 'coral' {
  const g = pickGrade(gco2eq);
  if (g === 'A' || g === 'B') return 'lime';
  if (g === 'C' || g === 'D') return 'amber';
  return 'coral';
}

/** Libellés lisibles des sites suivis. */
const HOST_LABELS: Record<Host, string> = {
  chatgpt: 'ChatGPT',
  claude: 'Claude',
  'le-chat': 'Le Chat'
};

// Voiture thermique moyenne ≈ 192 g CO₂eq/km (ADEME, Base Empreinte — même
// coefficient que l'équivalent voiture du badge in-page, design 38).
const CAR_G_PER_KM = 192;

/** Équivalent distance voiture pour rendre le chiffre parlant (« ≈ 65 m »). */
export function carEquivalentLabel(gco2eq: number): string {
  const meters = (gco2eq / CAR_G_PER_KM) * 1000;
  if (meters < 1) return 'moins d’un mètre en voiture';
  if (meters < 1000) return `≈ ${fmtFr(meters)} m en voiture`;
  return `≈ ${fmtFr(meters / 1000)} km en voiture`;
}

/** Sous-ligne du chiffre du jour : « 7 prompts mesurés · ≈ 65 m en voiture ». */
export function todaySubline(total: DailyTotal): string {
  const prompts = total.count > 1 ? `${total.count} prompts mesurés` : '1 prompt mesuré';
  return `${prompts} · ${carEquivalentLabel(total.gco2eq)}`;
}

/** État d'association de l'extension (où partent les mesures). */
export type LinkState = 'browser' | 'app' | 'team' | 'both';

export function resolveLinkState(pairedToApp: boolean, enrolledToTeam: boolean): LinkState {
  if (pairedToApp && enrolledToTeam) return 'both';
  if (enrolledToTeam) return 'team';
  if (pairedToApp) return 'app';
  return 'browser';
}

/** Une ligne claire par état — vouvoiement, pas de jargon. */
export function linkLabel(state: LinkState): string {
  switch (state) {
    case 'app':
      return 'Associée à l’app Sobr.ia desktop.';
    case 'team':
      return 'Reliée au serveur de votre équipe.';
    case 'both':
      return 'Associée à l’app Sobr.ia et au serveur équipe.';
    case 'browser':
      return 'Vos mesures restent dans ce navigateur.';
  }
}

// ─── Rendu ────────────────────────────────────────────────────────────────────

function renderToday(total: DailyTotal): void {
  const filled = document.getElementById('hero-filled');
  const empty = document.getElementById('hero-empty');
  const value = document.getElementById('hero-value');
  const sub = document.getElementById('hero-sub');
  const hasUsage = total.count > 0;
  filled?.toggleAttribute('hidden', !hasUsage);
  empty?.toggleAttribute('hidden', hasUsage);
  if (!hasUsage) return;
  if (value) value.textContent = fmtFr(total.gco2eq);
  if (sub) sub.textContent = todaySubline(total);
}

function renderLink(state: LinkState): void {
  const row = document.getElementById('link-row');
  const label = document.getElementById('link-label');
  const action = document.getElementById('link-action');
  if (!row || !label || !action) return;
  row.setAttribute('data-state', state);
  label.textContent = linkLabel(state);
  action.toggleAttribute('hidden', state !== 'browser');
}

function renderLastPrompt(last: LastEstimate | null): void {
  const block = document.getElementById('last-block');
  const body = document.getElementById('last-body');
  if (!block || !body) return;
  if (!last) {
    block.setAttribute('hidden', '');
    return;
  }
  block.removeAttribute('hidden');
  const e = last.estimate;
  const grade = pickGrade(e.gco2eq);
  body.innerHTML = `
    <span class="last__badge" data-tone="${toneOf(e.gco2eq)}" aria-label="Note ${grade}">${grade}</span>
    <span class="last__value">${fmtFr(e.gco2eq)} g CO₂eq</span>
    <span class="last__meta">${last.modelDisplayName} · ${HOST_LABELS[last.host]} · ${fmtRelative(last.ts)}</span>
  `;
}

function renderStats(total: DailyTotal): void {
  const water = document.getElementById('stat-water');
  const energy = document.getElementById('stat-energy');
  const count = document.getElementById('stat-count');
  if (water) water.innerHTML = `${fmtFr(total.waterMl)}<span class="u">mL</span>`;
  if (energy) energy.innerHTML = `${fmtFr(total.energyWh, 3)}<span class="u">Wh</span>`;
  if (count) count.textContent = String(total.count);
}

function renderMethodToggle(method: EmpreinteMethod): void {
  document.querySelectorAll<HTMLButtonElement>('.method-toggle__opt').forEach((btn) => {
    const checked = btn.dataset['method'] === method;
    btn.setAttribute('aria-checked', String(checked));
  });
}

// ─── Projet de la conversation (C44) ─────────────────────────────────────────
//
// Visible uniquement si : (a) l'onglet actif est une conversation sur un
// site suivi, (b) le Mode Équipe est enrôlé (l'étiquette ne sert qu'au
// serveur d'équipe). Le choix vaut pour TOUTE la conversation — les
// mesures suivantes partent étiquetées (résolution côté service worker).

/** Hôtes suivis — alignés sur les content scripts du manifest. */
const TRACKED_HOSTS = ['chatgpt.com', 'chat.openai.com', 'claude.ai', 'chat.mistral.ai'];

export function isTrackedUrl(url: string | undefined): boolean {
  const key = threadKeyFromUrl(url);
  return key !== null && TRACKED_HOSTS.some((h) => key === h || key.startsWith(`${h}/`));
}

async function initProjectSection(): Promise<void> {
  const row = document.getElementById('project-row');
  const select = document.getElementById('project-select') as HTMLSelectElement | null;
  const addBtn = document.getElementById('project-add');
  const input = document.getElementById('project-input') as HTMLInputElement | null;
  if (!row || !select || !addBtn || !input) return;

  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  const threadKey = threadKeyFromUrl(tab?.url);
  if (threadKey === null || !isTrackedUrl(tab?.url)) return;

  const team = await getTeamState().catch(() => null);
  if (!team?.enrolled) return;

  const [list, current] = await Promise.all([
    getProjectsList(),
    getProjectForThread(threadKey)
  ]);
  renderProjectOptions(select, list, current);
  row.removeAttribute('hidden');

  select.addEventListener('change', () => {
    const value = select.value === '' ? null : select.value;
    void setProjectForThread(threadKey, value);
  });

  addBtn.addEventListener('click', () => {
    input.toggleAttribute('hidden');
    if (!input.hasAttribute('hidden')) input.focus();
  });

  input.addEventListener('keydown', (ev) => {
    if (ev.key !== 'Enter') return;
    void (async () => {
      const next = await addProject(input.value);
      const created = input.value.trim().slice(0, 64);
      input.value = '';
      input.setAttribute('hidden', '');
      if (created.length === 0) return;
      renderProjectOptions(select, next, created);
      await setProjectForThread(threadKey, created);
    })();
  });
}

function renderProjectOptions(
  select: HTMLSelectElement,
  list: string[],
  current: string | null
): void {
  select.innerHTML = '';
  const none = document.createElement('option');
  none.value = '';
  none.textContent = 'Hors projet';
  select.append(none);
  for (const name of list) {
    const opt = document.createElement('option');
    opt.value = name;
    opt.textContent = name;
    select.append(opt);
  }
  select.value = current ?? '';
}

// ─── Init ─────────────────────────────────────────────────────────────────────

async function init(): Promise<void> {
  const [method, last, today] = await Promise.all([
    getMethod(),
    getLastEstimate(),
    getTodayTotal()
  ]);

  renderToday(today);
  renderStats(today);
  renderLastPrompt(last);
  renderMethodToggle(method);

  // L'état d'association ne doit jamais bloquer le bilan : best-effort séparé.
  try {
    const [pairing, team] = await Promise.all([getPairingState(), getTeamState()]);
    renderLink(resolveLinkState(pairing !== null, team.enrolled));
  } catch {
    renderLink('browser');
  }

  // C44 — sélecteur de projet par conversation (best-effort, jamais bloquant).
  initProjectSection().catch((err) => {
    console.warn('[sobria popup] projet de conversation indisponible:', err);
  });

  // Toggle méthodologie → persiste via storage.
  document.querySelectorAll<HTMLButtonElement>('.method-toggle__opt').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const next = btn.dataset['method'] as EmpreinteMethod;
      if (next !== 'afnor_sobria' && next !== 'ecologits') return;
      await setMethod(next);
      renderMethodToggle(next);
    });
  });

  // « Associer » → la section pairing est en tête de la page réglages.
  document.getElementById('link-action')?.addEventListener('click', () => {
    chrome.runtime.openOptionsPage();
  });

  // « Voir dans Sobr.ia » : désactivé tant que le bridge n'est pas livré (C27.5).

  // « Réglages » → ouvre la page options.
  document.getElementById('open-options-btn')?.addEventListener('click', () => {
    chrome.runtime.openOptionsPage();
  });
}

document.addEventListener('DOMContentLoaded', () => {
  applyVersionLabels();
  init().catch((err) => {
    console.error('[sobria popup] init failed:', err);
  });
});
