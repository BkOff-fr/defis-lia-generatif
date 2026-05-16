// Sobr.ia — service worker (background).
// C27.3 : ingestion locale des messages content scripts.
// C27.5 : forward des estimations au bridge natif si pairé.

import {
  appendToDailyTotal,
  getMethod,
  setMethod,
  setLastEstimate,
  getLastEstimate,
  getTodayTotal,
  purgeAll,
  getPairingState,
  setPairingState,
  clearPairingState
} from '../content/shared/storage.js';
import { getTeamState } from '../content/shared/team-storage.js';
import { pushEstimation } from '../lib/team-client.js';
import { BridgeClient } from './native-messaging.js';
import type {
  SobriaMessage,
  EstimationSubmittedResponse,
  GetMethodResponse,
  SetMethodResponse,
  GetDailyTotalResponse,
  GetLastEstimateResponse,
  PurgeDataResponse,
  GetPairingStatusResponse,
  PairWithCodeResponse,
  RevokePairingResponse,
  PairingStatus
} from '../lib/messages.js';

self.addEventListener('install', () => {
  console.info('[sobria] service worker installé (v0.6.0)');
});

self.addEventListener('activate', () => {
  console.info('[sobria] service worker activé');
});

// ─── Routeur de messages ─────────────────────────────────────────────────────

type AnyResponse =
  | EstimationSubmittedResponse
  | GetMethodResponse
  | SetMethodResponse
  | GetDailyTotalResponse
  | GetLastEstimateResponse
  | PurgeDataResponse
  | GetPairingStatusResponse
  | PairWithCodeResponse
  | RevokePairingResponse;

/** Construit un nouveau BridgeClient connecté, ou `null` si bridge indisponible. */
async function tryConnectBridge(): Promise<BridgeClient | null> {
  const client = new BridgeClient();
  const ok = await client.connect();
  if (!ok) {
    client.disconnect();
    return null;
  }
  return client;
}

/** Calcule un statut pairing combinant storage + check bridge (ping). */
async function computePairingStatus(): Promise<PairingStatus> {
  const state = await getPairingState();
  const bridge = await tryConnectBridge();
  const bridgeAvailable = bridge !== null;
  bridge?.disconnect();
  if (!state) return { paired: false, bridgeAvailable };
  return {
    paired: true,
    pairingId: state.pairingId,
    fingerprint: state.fingerprint,
    pairedAt: state.pairedAt,
    bridgeAvailable
  };
}

async function handleMessage(message: SobriaMessage): Promise<AnyResponse> {
  switch (message.type) {
    case 'estimation_submitted': {
      await appendToDailyTotal({
        gco2eq: message.estimate.gco2eq,
        waterMl: message.estimate.waterMl,
        energyWh: message.estimate.energyWh
      });
      const ts = new Date().toISOString();
      await setLastEstimate({
        estimate: message.estimate,
        host: message.host,
        modelDisplayName: message.modelDisplayName,
        ts
      });

      // Dispatch des estimations selon le Mode Équipe (C28.6) :
      // - 'local' (défaut) → bridge natif uniquement (pairing perso C27).
      // - 'team'           → serveur équipe HTTPS uniquement.
      // - 'both'           → les deux (transition / coexistence).
      // Best-effort sur chaque destination : un échec ne bloque pas l'autre.
      const teamState = await getTeamState();
      const dispatchLocal = teamState.mode === 'local' || teamState.mode === 'both';
      const dispatchTeam =
        (teamState.mode === 'team' || teamState.mode === 'both') && teamState.enrolled;

      if (dispatchLocal) {
        const pairing = await getPairingState();
        if (pairing) {
          tryConnectBridge()
            .then(async (bridge) => {
              if (!bridge) return;
              try {
                await bridge.sendEstimate(pairing.secret, {
                  estimate: message.estimate,
                  host: message.host,
                  modelDisplayName: message.modelDisplayName,
                  ts
                });
              } finally {
                bridge.disconnect();
              }
            })
            .catch((err) => console.warn('[sobria] forward bridge échec:', err));
        }
      }

      if (dispatchTeam) {
        pushEstimation({
          estimate: message.estimate,
          host: message.host,
          modelDisplayName: message.modelDisplayName,
          ts
        }).catch((err) => console.warn('[sobria] forward team échec:', err));
      }
      return { ok: true };
    }
    case 'get_method': {
      const method = await getMethod();
      return { method };
    }
    case 'set_method': {
      await setMethod(message.method);
      return { ok: true };
    }
    case 'get_daily_total': {
      const total = await getTodayTotal();
      return { total };
    }
    case 'get_last_estimate': {
      const last = await getLastEstimate();
      return { last };
    }
    case 'purge_data': {
      await purgeAll();
      return { ok: true };
    }
    case 'get_pairing_status': {
      const status = await computePairingStatus();
      return { status };
    }
    case 'pair_with_code': {
      const bridge = await tryConnectBridge();
      if (!bridge) {
        return { ok: false, error: 'Bridge natif indisponible — installer l’app Sobr.ia desktop.' };
      }
      try {
        const res = await bridge.pair(message.code);
        if (!res.ok || !res.secret || !res.pairingId || !res.fingerprint) {
          return { ok: false, error: res.error ?? 'Code invalide ou expiré.' };
        }
        const pairedAt = new Date().toISOString();
        await setPairingState({
          secret: res.secret,
          pairingId: res.pairingId,
          fingerprint: res.fingerprint,
          pairedAt
        });
        return {
          ok: true,
          status: {
            paired: true,
            pairingId: res.pairingId,
            fingerprint: res.fingerprint,
            pairedAt,
            bridgeAvailable: true
          }
        };
      } finally {
        bridge.disconnect();
      }
    }
    case 'revoke_pairing': {
      const state = await getPairingState();
      if (state) {
        const bridge = await tryConnectBridge();
        if (bridge) {
          try {
            await bridge.revoke(state.secret).catch(() => undefined);
          } finally {
            bridge.disconnect();
          }
        }
      }
      await clearPairingState();
      return { ok: true };
    }
    default: {
      const exhaustive: never = message;
      throw new Error(`message type inconnu: ${String(exhaustive)}`);
    }
  }
}

chrome.runtime.onMessage.addListener((rawMessage, _sender, sendResponse) => {
  const message = rawMessage as SobriaMessage;
  handleMessage(message)
    .then((response) => sendResponse(response))
    .catch((err: unknown) => {
      console.error('[sobria] handleMessage error :', err);
      sendResponse({ ok: false, error: String(err) });
    });
  return true;
});
