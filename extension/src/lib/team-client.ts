// Sobr.ia extension — client REST Mode Équipe (C28.6).
//
// Parle au serveur `sobria-team-aggregator` self-hosted (cf. ADR-0013
// Phase 2 + brief C28). Auth : JWT Bearer + rotation refresh auto sur
// 401. Pas de gestion programmatique du cert auto-signé — il faut que
// l'utilisateur ait visité l'URL HTTPS au moins une fois dans le
// navigateur pour accepter manuellement le warning (le browser stocke
// son consentement par origin).
//
// Le payload `/api/v1/estimations` est compatible v0.6.0 (camelCase
// `{ estimate, host, modelDisplayName, ts }`).

import {
  clearTeamSession,
  getTeamAccessToken,
  getTeamRefreshToken,
  recordEnrollment,
  requireTeamUrl,
  setTeamTokens
} from '../content/shared/team-storage.js';
import type { Estimate } from './types.js';

export class TeamApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string
  ) {
    super(message);
  }
}

export type EnrollResponse = {
  user_id: string;
  access_token: string;
  refresh_token: string;
  access_expires_at: string;
  refresh_expires_at: string;
};

export type EstimatePayload = {
  estimate: Estimate;
  host: string;
  modelDisplayName: string;
  ts: string;
  /** Étiquette projet de la conversation (C44) — optionnelle, ≤ 64 chars. */
  project?: string;
};

async function joinUrl(path: string): Promise<string> {
  const base = await requireTeamUrl();
  return `${base}${path}`;
}

async function rawFetch(path: string, init: RequestInit = {}, retry = true): Promise<Response> {
  const headers = new Headers(init.headers || {});
  if (init.body && !headers.has('content-type')) {
    headers.set('content-type', 'application/json');
  }
  const token = await getTeamAccessToken();
  if (token) headers.set('authorization', `Bearer ${token}`);

  const url = await joinUrl(path);
  const resp = await fetch(url, { ...init, headers });

  if (resp.status === 401 && retry && token) {
    const refreshed = await tryRefresh();
    if (refreshed) return rawFetch(path, init, false);
  }
  return resp;
}

async function tryRefresh(): Promise<boolean> {
  const refresh = await getTeamRefreshToken();
  if (!refresh) return false;
  try {
    const url = await joinUrl('/api/v1/refresh');
    const r = await fetch(url, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ refresh_token: refresh })
    });
    if (!r.ok) {
      await clearTeamSession();
      return false;
    }
    const body = (await r.json()) as {
      access_token: string;
      refresh_token: string;
    };
    await setTeamTokens({ access: body.access_token, refresh: body.refresh_token });
    return true;
  } catch {
    await clearTeamSession();
    return false;
  }
}

async function parseOrThrow<T>(resp: Response): Promise<T> {
  if (resp.ok) return (await resp.json()) as T;
  let code = 'http_error';
  let message = `HTTP ${resp.status}`;
  try {
    const body = await resp.json();
    if (typeof body?.error === 'string') message = body.error;
    if (typeof body?.code === 'string') code = body.code;
  } catch {
    /* fallback */
  }
  throw new TeamApiError(resp.status, code, message);
}

/** Vérifie la connectivité serveur (GET /health). */
export async function ping(): Promise<{ ok: boolean; version: string }> {
  const url = await joinUrl('/health');
  const r = await fetch(url, { method: 'GET' });
  return parseOrThrow(r);
}

/** Enrôle l'utilisateur avec un code 12 chiffres. */
export async function enroll(args: {
  code: string;
  password: string;
  fingerprint: string;
  displayName?: string;
}): Promise<EnrollResponse> {
  const url = await joinUrl('/api/v1/enroll');
  const r = await fetch(url, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      code: args.code,
      password: args.password,
      fingerprint: args.fingerprint,
      display_name: args.displayName
    })
  });
  const body = await parseOrThrow<EnrollResponse>(r);
  await setTeamTokens({ access: body.access_token, refresh: body.refresh_token });
  await recordEnrollment({
    userId: body.user_id,
    fingerprint: args.fingerprint,
    enrolledAt: new Date().toISOString()
  });
  return body;
}

/** Pousse une estimation au serveur. Lance `TeamApiError` en cas d'échec
 *  (réseau, cert, 401 même après refresh, validation 400). */
export async function pushEstimation(
  payload: EstimatePayload
): Promise<{ id: string; ack: boolean }> {
  const resp = await rawFetch('/api/v1/estimations', {
    method: 'POST',
    body: JSON.stringify(payload)
  });
  return parseOrThrow(resp);
}

/** Déconnecte le user côté client (purge tokens). */
export async function logout(): Promise<void> {
  await clearTeamSession();
}
