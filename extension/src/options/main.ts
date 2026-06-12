// Sobr.ia options — sections fonctionnelles (C27.4).
//
// Sections câblées :
//   - Pairing : placeholder (C27.5 livrera le bridge + code 6 chiffres)
//   - Sites surveillés : 3 toggles persistés dans chrome.storage
//   - Confidentialité : toggle badge + boutons purge + export JSON
//   - Méthodologie : info statique
//   - À propos : version statique

import './options.css';
import { applyVersionLabels } from '../lib/registry-meta.js';
import {
  getSitesEnabled,
  setSitesEnabled,
  getBadgeVisible,
  setBadgeVisible,
  purgeAll
} from '../content/shared/storage.js';
import {
  browserFingerprint,
  getTeamState,
  setTeamMode,
  setTeamUrl,
  type TeamMode,
  type TeamState
} from '../content/shared/team-storage.js';
import { enroll, logout as teamLogout, ping, TeamApiError } from '../lib/team-client.js';
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

// ─── Mode Équipe (C28.6) ─────────────────────────────────────────────────────

function renderTeam(state: TeamState): void {
  const statusEl = document.getElementById('team-status');
  const labelEl = statusEl?.querySelector<HTMLElement>('.team-status__label');
  const enrollPane = document.getElementById('team-enroll-form');
  const infoPane = document.getElementById('team-info');
  const urlInput = document.querySelector<HTMLInputElement>('#team-url-input');

  enrollPane?.setAttribute('hidden', '');
  infoPane?.setAttribute('hidden', '');

  if (urlInput && state.url) urlInput.value = state.url;

  if (state.enrolled) {
    statusEl?.setAttribute('data-state', 'paired');
    if (labelEl) labelEl.textContent = 'Enrôlé';
    infoPane?.removeAttribute('hidden');
    const urlOut = document.getElementById('team-info-url');
    const userIdOut = document.getElementById('team-info-user-id');
    const fpOut = document.getElementById('team-info-fingerprint');
    const sinceOut = document.getElementById('team-info-since');
    if (urlOut) urlOut.textContent = state.url ?? '—';
    if (userIdOut) userIdOut.textContent = state.userId ?? '—';
    if (fpOut) fpOut.textContent = state.fingerprint ?? '—';
    if (sinceOut && state.enrolledAt)
      sinceOut.textContent = new Date(state.enrolledAt).toLocaleString('fr-FR');
    document.querySelectorAll<HTMLInputElement>('input[name="team-mode"]').forEach((r) => {
      r.checked = r.value === state.mode;
    });
  } else if (state.url) {
    statusEl?.setAttribute('data-state', 'unpaired');
    if (labelEl) labelEl.textContent = 'Serveur configuré · non enrôlé';
    enrollPane?.removeAttribute('hidden');
  } else {
    statusEl?.setAttribute('data-state', 'unpaired');
    if (labelEl) labelEl.textContent = 'Non configuré';
  }
}

function setTeamFieldError(id: string, msg: string, isOk = false): void {
  const el = document.getElementById(id);
  if (!el) return;
  el.textContent = msg;
  el.style.color = !msg ? '' : isOk ? 'var(--lime)' : 'var(--coral)';
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

  // ─── Mode Équipe ─────────────────────────────────────────────────────────
  try {
    renderTeam(await getTeamState());
  } catch (err) {
    console.error('[sobria options] getTeamState failed', err);
  }

  const teamUrlInput = document.querySelector<HTMLInputElement>('#team-url-input');
  const teamPingBtn = document.querySelector<HTMLButtonElement>('#team-ping-btn');
  const teamCodeInput = document.querySelector<HTMLInputElement>('#team-code-input');
  const teamDisplayInput = document.querySelector<HTMLInputElement>('#team-displayname-input');
  const teamPasswordInput = document.querySelector<HTMLInputElement>('#team-password-input');
  const teamEnrollBtn = document.querySelector<HTMLButtonElement>('#team-enroll-btn');
  const teamLogoutBtn = document.querySelector<HTMLButtonElement>('#team-logout-btn');

  teamCodeInput?.addEventListener('input', () => {
    teamCodeInput.value = teamCodeInput.value.replace(/[^0-9]/g, '').slice(0, 12);
  });

  teamPingBtn?.addEventListener('click', async () => {
    if (!teamUrlInput) return;
    setTeamFieldError('team-ping-error', '');
    const raw = teamUrlInput.value.trim();
    if (!/^https:\/\//.test(raw)) {
      setTeamFieldError('team-ping-error', 'URL doit commencer par https://');
      return;
    }
    teamPingBtn.disabled = true;
    try {
      await setTeamUrl(raw);
      const health = await ping();
      setTeamFieldError('team-ping-error', `Serveur joignable · version ${health.version}`, true);
      renderTeam(await getTeamState());
    } catch (err) {
      const msg =
        err instanceof TeamApiError
          ? err.message
          : err instanceof TypeError
            ? 'Connexion refusée — vérifier que vous avez accepté le certificat dans un autre onglet.'
            : String(err);
      setTeamFieldError('team-ping-error', msg);
    } finally {
      teamPingBtn.disabled = false;
    }
  });

  teamEnrollBtn?.addEventListener('click', async () => {
    if (!teamCodeInput || !teamPasswordInput) return;
    setTeamFieldError('team-enroll-error', '');
    const code = teamCodeInput.value.trim();
    const password = teamPasswordInput.value;
    const displayName = teamDisplayInput?.value.trim() || undefined;
    if (!/^\d{12}$/.test(code)) {
      setTeamFieldError('team-enroll-error', 'Code doit être 12 chiffres.');
      return;
    }
    if (password.length < 8) {
      setTeamFieldError('team-enroll-error', 'Mot de passe ≥ 8 caractères requis.');
      return;
    }
    teamEnrollBtn.disabled = true;
    try {
      const fingerprint = browserFingerprint();
      await enroll(
        displayName === undefined
          ? { code, password, fingerprint }
          : { code, password, fingerprint, displayName }
      );
      // Active 'both' au premier enrollment — les estimations remontent
      // automatiquement à l'équipe sans toggler manuellement.
      await setTeamMode('both');
      renderTeam(await getTeamState());
      teamCodeInput.value = '';
      teamPasswordInput.value = '';
      if (teamDisplayInput) teamDisplayInput.value = '';
    } catch (err) {
      const msg =
        err instanceof TeamApiError
          ? err.message
          : err instanceof TypeError
            ? "Connexion refusée — vérifier l'URL et le certificat."
            : String(err);
      setTeamFieldError('team-enroll-error', msg);
    } finally {
      teamEnrollBtn.disabled = false;
    }
  });

  document.querySelectorAll<HTMLInputElement>('input[name="team-mode"]').forEach((r) => {
    r.addEventListener('change', async () => {
      if (!r.checked) return;
      await setTeamMode(r.value as TeamMode);
      setStatus(`Mode équipe : ${r.value}.`);
    });
  });

  teamLogoutBtn?.addEventListener('click', async () => {
    if (!confirm('Te déconnecter du serveur équipe ? Les tokens seront effacés.')) return;
    teamLogoutBtn.disabled = true;
    try {
      await teamLogout();
      await setTeamMode('local');
      renderTeam(await getTeamState());
    } finally {
      teamLogoutBtn.disabled = false;
    }
  });

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
  applyVersionLabels();
  init().catch((err) => console.error('[sobria options] init failed:', err));
});
