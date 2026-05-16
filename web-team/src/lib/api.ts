// API client : fetch wrapper avec JWT Bearer + refresh auto sur 401.
//
// L'access token vit uniquement en mémoire (cf. brief C28.4 — pas de
// localStorage à cause du risque XSS). Le refresh token est conservé en
// sessionStorage : il survit aux reloads du même onglet, mais pas à la
// fermeture de l'onglet. Compromis UX/sécurité raisonnable.
//
// La rotation du refresh est gérée côté serveur — chaque /refresh émet
// un nouveau couple et révoque l'ancien.

const REFRESH_KEY = 'sobria_team_refresh';

export type Role = 'user' | 'admin';

export interface Tokens {
  access_token: string;
  refresh_token: string;
  role?: Role;
  subject_id?: string;
  access_expires_at?: string;
  refresh_expires_at?: string;
}

// In-memory access token (mutable variable, exposé via accesseurs).
let _accessToken: string | null = null;
let _role: Role | null = null;
let _subjectId: string | null = null;

export function getAccessToken(): string | null {
  return _accessToken;
}

export function getRole(): Role | null {
  return _role;
}

export function getSubjectId(): string | null {
  return _subjectId;
}

export function setTokens(t: Tokens): void {
  _accessToken = t.access_token;
  if (t.role) _role = t.role;
  if (t.subject_id) _subjectId = t.subject_id;
  if (typeof window !== 'undefined') {
    sessionStorage.setItem(REFRESH_KEY, t.refresh_token);
  }
}

export function clearTokens(): void {
  _accessToken = null;
  _role = null;
  _subjectId = null;
  if (typeof window !== 'undefined') {
    sessionStorage.removeItem(REFRESH_KEY);
  }
}

export function getRefreshToken(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem(REFRESH_KEY);
}

async function tryRefresh(): Promise<boolean> {
  const refresh = getRefreshToken();
  if (!refresh) return false;
  try {
    const r = await fetch('/api/v1/refresh', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ refresh_token: refresh })
    });
    if (!r.ok) {
      clearTokens();
      return false;
    }
    const body = (await r.json()) as Tokens;
    setTokens(body);
    return true;
  } catch {
    clearTokens();
    return false;
  }
}

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string
  ) {
    super(message);
  }
}

async function rawFetch(
  path: string,
  init: RequestInit = {},
  retry = true
): Promise<Response> {
  const headers = new Headers(init.headers || {});
  if (init.body && !headers.has('content-type')) {
    headers.set('content-type', 'application/json');
  }
  if (_accessToken) {
    headers.set('authorization', `Bearer ${_accessToken}`);
  }
  const resp = await fetch(path, { ...init, headers });
  if (resp.status === 401 && retry && _accessToken) {
    const ok = await tryRefresh();
    if (ok) {
      return rawFetch(path, init, false);
    }
  }
  return resp;
}

export async function apiGet<T>(path: string): Promise<T> {
  const resp = await rawFetch(path, { method: 'GET' });
  return parseResponse<T>(resp);
}

export async function apiPost<T>(path: string, body?: unknown): Promise<T> {
  const resp = await rawFetch(path, {
    method: 'POST',
    body: body === undefined ? undefined : JSON.stringify(body)
  });
  return parseResponse<T>(resp);
}

export async function apiDelete<T>(path: string): Promise<T> {
  const resp = await rawFetch(path, { method: 'DELETE' });
  return parseResponse<T>(resp);
}

async function parseResponse<T>(resp: Response): Promise<T> {
  if (resp.ok) {
    return (await resp.json()) as T;
  }
  let code = 'http_error';
  let message = `HTTP ${resp.status}`;
  try {
    const body = await resp.json();
    if (typeof body?.error === 'string') message = body.error;
    if (typeof body?.code === 'string') code = body.code;
  } catch {
    // garde le message par défaut
  }
  throw new ApiError(resp.status, code, message);
}

/** Tente de restaurer une session via le refresh stocké (au mount root). */
export async function restoreSession(): Promise<boolean> {
  return tryRefresh();
}
