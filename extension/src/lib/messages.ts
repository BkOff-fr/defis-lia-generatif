// Sobr.ia extension — contrats de messages typés (C27.3).
//
// Tous les messages échangés entre service worker, content scripts et
// popup/options passent par ce contrat unique. Strict discriminated union
// pour empêcher les mismatch types entre émetteur et récepteur.
//
// Mirror minimaliste de `crates/sobria-app/src/dto.rs` côté Tauri IPC.

import type { EmpreinteMethod, Estimate } from './types.js';

/** Identifiant du site source d'une estimation extension. */
export type Host = 'chatgpt' | 'claude' | 'le-chat';

/** Total quotidien agrégé (1 entrée par date locale). */
export type DailyTotal = {
  readonly date: string; // YYYY-MM-DD (locale du device)
  readonly count: number;
  readonly gco2eq: number;
  readonly waterMl: number;
  readonly energyWh: number;
};

/** Snapshot du dernier prompt mesuré (pour la popup C27.4). */
export type LastEstimate = {
  readonly estimate: Estimate;
  readonly host: Host;
  readonly modelDisplayName: string;
  readonly ts: string; // ISO 8601
};

// ─── Messages content → service worker ───────────────────────────────────────

export type EstimationSubmittedMessage = {
  readonly type: 'estimation_submitted';
  readonly estimate: Estimate;
  readonly host: Host;
  readonly modelDisplayName: string;
};

export type GetMethodMessage = {
  readonly type: 'get_method';
};

// ─── Messages popup/options → service worker ─────────────────────────────────

export type SetMethodMessage = {
  readonly type: 'set_method';
  readonly method: EmpreinteMethod;
};

export type GetDailyTotalMessage = {
  readonly type: 'get_daily_total';
};

export type GetLastEstimateMessage = {
  readonly type: 'get_last_estimate';
};

export type PurgeDataMessage = {
  readonly type: 'purge_data';
};

// ─── Pairing (C27.5) ─────────────────────────────────────────────────────────

/** État de pairing exposé aux UI extension (popup / options). */
export type PairingStatus = {
  readonly paired: boolean;
  readonly pairingId?: string;
  readonly fingerprint?: string;
  readonly pairedAt?: string;
  /** Le bridge natif répond — l'app desktop est joignable. */
  readonly bridgeAvailable: boolean;
};

export type GetPairingStatusMessage = { readonly type: 'get_pairing_status' };

export type PairWithCodeMessage = {
  readonly type: 'pair_with_code';
  readonly code: string;
};

export type RevokePairingMessage = { readonly type: 'revoke_pairing' };

// ─── Union discriminée ───────────────────────────────────────────────────────

export type SobriaMessage =
  | EstimationSubmittedMessage
  | GetMethodMessage
  | SetMethodMessage
  | GetDailyTotalMessage
  | GetLastEstimateMessage
  | PurgeDataMessage
  | GetPairingStatusMessage
  | PairWithCodeMessage
  | RevokePairingMessage;

// ─── Réponses ────────────────────────────────────────────────────────────────

export type GetMethodResponse = { readonly method: EmpreinteMethod };
export type SetMethodResponse = { readonly ok: true };
export type EstimationSubmittedResponse = { readonly ok: true };
export type GetDailyTotalResponse = { readonly total: DailyTotal };
export type GetLastEstimateResponse = { readonly last: LastEstimate | null };
export type PurgeDataResponse = { readonly ok: true };

export type GetPairingStatusResponse = { readonly status: PairingStatus };
export type PairWithCodeResponse =
  | { readonly ok: true; readonly status: PairingStatus }
  | { readonly ok: false; readonly error: string };
export type RevokePairingResponse = { readonly ok: true };
