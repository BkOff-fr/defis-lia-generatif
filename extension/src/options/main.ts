// Sobr.ia options — sections fonctionnelles (C27.4).
//
// Sections câblées :
//   - Pairing : placeholder (C27.5 livrera le bridge + code 6 chiffres)
//   - Sites surveillés : 3 toggles persistés dans chrome.storage
//   - Confidentialité : toggle badge + boutons purge + export JSON
//   - Méthodologie : info statique
//   - À propos : version statique

import './options.css';
import {
  getSitesEnabled,
  setSitesEnabled,
  getBadgeVisible,
  setBadgeVisible,
  purgeAll
} from '../content/shared/storage.js';
import type {
  GetPairingStatusMessage,
  GetPairingStatusResponse,
  PairWithCodeMessage,
  PairWithCodeResponse,
  RevokePairingMessage,
  RevokePairingResponse,
  PairingStatus
} from '../lib/messages.js';

type SiteKey = 'chatgpt' | 'claude' | 'leChat';

// ─── Pairing helpers (C27.5) ────────────────────────────────────────────────

async function fetchPairingStatus(): Promise<PairingStatus> {
  const msg: GetPairingStatusMessage = { type: 'get_pairing_status' };
  const res = (await chrome.runtime.sendMessage(msg)) as GetPairingStatusResponse;
  return res.status;
}

async function pairWithCode(code: string): Promise<PairWithCodeResponse> {
  const msg: PairWithCodeMessage = { type: 'pair_with_code', code };
  return (await chrome.runtime.sendMessage(msg)) as PairWithCodeResponse;
}

async function revokePairing(): Promise<RevokePairingResponse> {
  const msg: RevokePairingMessage = { type: 'revoke_pairing' };
  return (await chrome.runtime.sendMessage(msg)) as RevokePairingResponse;
}

function renderPairing(status: PairingStatus): void {
  const statusEl = document.getElementById('pairing-status');
  const labelEl = statusEl?.querySelector<HTMLElement>('.pairing-status__label');
  const formPane = document.getElementById('pairing-form');
  const infoPane = document.getElementById('pairing-info');
  const noBridgePane = document.getElementById('pairing-no-bridge');

  formPane?.setAttribute('hidden', '');
  infoPane?.setAttribute('hidden', '');
  noBridgePane?.setAttribute('hidden', '');

  if (!status.bridgeAvailable) {
    statusEl?.setAttribute('data-state', 'no-bridge');
    if (labelEl) labelEl.textContent = 'App desktop non détectée';
    noBridgePane?.removeAttribute('hidden');
    return;
  }
  if (!status.paired) {
    statusEl?.setAttribute('data-state', 'unpaired');
    if (labelEl) labelEl.textContent = 'Bridge détecté · non pairée';
    formPane?.removeAttribute('hidden');
    return;
  }
  statusEl?.setAttribute('data-state', 'paired');
  if (labelEl) labelEl.textContent = 'Pairée';
  infoPane?.removeAttribute('hidden');
  const fp = document.getElementById('pairing-fingerprint');
  const since = document.getElementById('pairing-since');
  const id = document.getElementById('pairing-id');
  if (fp) fp.textContent = status.fingerprint ?? '—';
  if (since && status.pairedAt)
    since.textContent = new Date(status.pairedAt).toLocaleString('fr-FR');
  if (id) id.textContent = status.pairingId ?? '—';
}

async function exportData(): Promise<void> {
  const all = await chrome.storage.local.get(null);
  const blob = new Blob([JSON.stringify(all, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  const date = new Date().toISOString().slice(0, 10);
  a.download = `sobria-export-${date}.json`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function setStatus(msg: string, isError = false): void {
  const el = document.getElementById('confid-status');
  if (!el) return;
  el.textContent = msg;
  el.style.color = isError ? 'var(--coral)' : 'var(--lime)';
  // Auto-clear après 4 secondes.
  setTimeout(() => {
    if (el.textContent === msg) el.textContent = '';
  }, 4000);
}

async function init(): Promise<void> {
  // ─── Pairing ──────────────────────────────────────────────────────────────
  try {
    const status = await fetchPairingStatus();
    renderPairing(status);
  } catch (err) {
    console.error('[sobria options] fetchPairingStatus failed', err);
    renderPairing({ paired: false, bridgeAvailable: false });
  }

  const codeInput = document.querySelector<HTMLInputElement>('#pairing-code-input');
  const connectBtn = document.querySelector<HTMLButtonElement>('#pairing-connect-btn');
  const pairingError = document.getElementById('pairing-error');
  const revokeBtn = document.querySelector<HTMLButtonElement>('#pairing-revoke-btn');

  // Sanitize : ne garder que les chiffres dans le champ code.
  codeInput?.addEventListener('input', () => {
    codeInput.value = codeInput.value.replace(/[^0-9]/g, '').slice(0, 6);
  });

  connectBtn?.addEventListener('click', async () => {
    if (!codeInput || pairingError === null) return;
    const code = codeInput.value.trim();
    if (!/^\d{6}$/.test(code)) {
      pairingError.textContent = 'Le code doit faire 6 chiffres.';
      return;
    }
    connectBtn.disabled = true;
    pairingError.textContent = '';
    try {
      const res = await pairWithCode(code);
      if (!res.ok) {
        pairingError.textContent = res.error;
        connectBtn.disabled = false;
        return;
      }
      renderPairing(res.status);
      codeInput.value = '';
    } catch (err) {
      pairingError.textContent = String(err);
      connectBtn.disabled = false;
    }
  });

  revokeBtn?.addEventListener('click', async () => {
    if (!confirm('Dépaire cette extension de l’app Sobr.ia desktop ?')) return;
    revokeBtn.disabled = true;
    try {
      await revokePairing();
      const status = await fetchPairingStatus();
      renderPairing(status);
    } catch (err) {
      console.error(err);
    } finally {
      revokeBtn.disabled = false;
    }
  });

  // Sites enabled toggles.
  const sites = await getSitesEnabled();
  for (const key of ['chatgpt', 'claude', 'leChat'] as SiteKey[]) {
    const cb = document.querySelector<HTMLInputElement>(`input[data-site='${key}']`);
    if (!cb) continue;
    cb.checked = sites[key];
    cb.addEventListener('change', async () => {
      const current = await getSitesEnabled();
      const next = { ...current, [key]: cb.checked };
      await setSitesEnabled(next);
      setStatus(`${key} ${cb.checked ? 'activé' : 'désactivé'}.`);
    });
  }

  // Badge visible toggle.
  const badgeCb = document.querySelector<HTMLInputElement>('#badge-visible-toggle');
  if (badgeCb) {
    badgeCb.checked = await getBadgeVisible();
    badgeCb.addEventListener('change', async () => {
      await setBadgeVisible(badgeCb.checked);
      setStatus(`Badge ${badgeCb.checked ? 'activé' : 'masqué'} sur les pages.`);
    });
  }

  // Export JSON.
  document.getElementById('export-btn')?.addEventListener('click', () => {
    exportData()
      .then(() => setStatus('Export téléchargé.'))
      .catch((err) => {
        console.error(err);
        setStatus('Échec de l’export.', true);
      });
  });

  // Purge.
  document.getElementById('purge-btn')?.addEventListener('click', async () => {
    if (!confirm('Supprimer toutes les données locales Sobr.ia ? Action irréversible.')) {
      return;
    }
    try {
      await purgeAll();
      // Réinitialise l'état affiché à défaut.
      for (const key of ['chatgpt', 'claude', 'leChat'] as SiteKey[]) {
        const cb = document.querySelector<HTMLInputElement>(`input[data-site='${key}']`);
        if (cb) cb.checked = true;
      }
      if (badgeCb) badgeCb.checked = true;
      setStatus('Toutes les données locales ont été purgées.');
    } catch (err) {
      console.error(err);
      setStatus('Échec de la purge.', true);
    }
  });
}

document.addEventListener('DOMContentLoaded', () => {
  init().catch((err) => console.error('[sobria options] init failed:', err));
});
