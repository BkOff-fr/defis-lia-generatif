// Sobr.ia extension — persistance Mode Équipe (C28.6).
//
// chrome.storage.local survit aux fermetures de navigateur et n'est pas
// exposé aux page scripts (sandbox). On y stocke :
//
//   - team_url        : URL HTTPS du serveur self-hosted (cf. ADR-0013 Phase 2)
//   - team_mode       : 'local' | 'team' | 'both'  (toggle dispatch estimations)
//   - team_user_id    : ULID du user retourné par /enroll
//   - team_access     : JWT 24h (recopié au refresh)
//   - team_refresh    : refresh token `<ulid>.<uuid>` (rotation à chaque /refresh)
//   - team_fingerprint: fingerprint envoyé à /enroll (déterministe par device)
//   - team_enrolled_at: ISO timestamp
//
// Compromis sécurité : pour qu'un user reste enrôlé entre deux ouvertures
// de navigateur, on persiste l'access token aussi (le SW se redémarre).
// chrome.storage.local n'est pas readable par les pages → risque XSS réduit
// au workflow extension. Pas idéal mais conforme à la pratique des
// WebExtensions et acceptable pour ce mode opt-in.

const KEYS = {
  url: 'team_url',
  mode: 'team_mode',
  userId: 'team_user_id',
  access: 'team_access',
  refresh: 'team_refresh',
  fingerprint: 'team_fingerprint',
  enrolledAt: 'team_enrolled_at'
} as const;

/** Mode de dispatch des estimations. */
export type TeamMode = 'local' | 'team' | 'both';

/** Vue d'ensemble de l'état Mode Équipe (lu par Options + service-worker). */
export type TeamState = {
  enrolled: boolean;
  url: string | null;
  userId: string | null;
  mode: TeamMode;
  fingerprint: string | null;
  enrolledAt: string | null;
};

/** Lit l'état complet (défauts si vide). */
export async function getTeamState(): Promise<TeamState> {
  const r = await chrome.storage.local.get([
    KEYS.url,
    KEYS.mode,
    KEYS.userId,
    KEYS.fingerprint,
    KEYS.enrolledAt,
    KEYS.access
  ]);
  const mode = normalizeMode(r[KEYS.mode]);
  return {
    enrolled: typeof r[KEYS.access] === 'string' && (r[KEYS.access] as string).length > 0,
    url: typeof r[KEYS.url] === 'string' ? (r[KEYS.url] as string) : null,
    userId: typeof r[KEYS.userId] === 'string' ? (r[KEYS.userId] as string) : null,
    mode,
    fingerprint:
      typeof r[KEYS.fingerprint] === 'string' ? (r[KEYS.fingerprint] as string) : null,
    enrolledAt:
      typeof r[KEYS.enrolledAt] === 'string' ? (r[KEYS.enrolledAt] as string) : null
  };
}

function normalizeMode(raw: unknown): TeamMode {
  if (raw === 'team' || raw === 'both' || raw === 'local') return raw;
  return 'local';
}

/** Pose l'URL du serveur (sans tokens). */
export async function setTeamUrl(url: string): Promise<void> {
  const trimmed = url.trim().replace(/\/+$/, '');
  await chrome.storage.local.set({ [KEYS.url]: trimmed });
}

/** Définit le mode de dispatch. */
export async function setTeamMode(mode: TeamMode): Promise<void> {
  await chrome.storage.local.set({ [KEYS.mode]: mode });
}

/** Lit l'access token courant (le SW peut être redémarré entre deux calls). */
export async function getTeamAccessToken(): Promise<string | null> {
  const r = await chrome.storage.local.get(KEYS.access);
  return typeof r[KEYS.access] === 'string' ? (r[KEYS.access] as string) : null;
}

/** Lit le refresh token courant. */
export async function getTeamRefreshToken(): Promise<string | null> {
  const r = await chrome.storage.local.get(KEYS.refresh);
  return typeof r[KEYS.refresh] === 'string' ? (r[KEYS.refresh] as string) : null;
}

/** Persiste un nouveau couple access+refresh (post /enroll ou /refresh). */
export async function setTeamTokens(args: {
  access: string;
  refresh: string;
}): Promise<void> {
  await chrome.storage.local.set({
    [KEYS.access]: args.access,
    [KEYS.refresh]: args.refresh
  });
}

/** Enregistre les méta-données après un /enroll réussi. */
export async function recordEnrollment(args: {
  userId: string;
  fingerprint: string;
  enrolledAt: string;
}): Promise<void> {
  await chrome.storage.local.set({
    [KEYS.userId]: args.userId,
    [KEYS.fingerprint]: args.fingerprint,
    [KEYS.enrolledAt]: args.enrolledAt
  });
}

/** Logout : efface tokens + user_id (garde url + mode pour faciliter le ré-enrôlement). */
export async function clearTeamSession(): Promise<void> {
  await chrome.storage.local.remove([
    KEYS.access,
    KEYS.refresh,
    KEYS.userId,
    KEYS.fingerprint,
    KEYS.enrolledAt
  ]);
}

/** Lit l'URL ; throw si non configurée. */
export async function requireTeamUrl(): Promise<string> {
  const r = await chrome.storage.local.get(KEYS.url);
  const url = r[KEYS.url];
  if (typeof url !== 'string' || url.length === 0) {
    throw new Error('URL serveur équipe non configurée');
  }
  return url;
}

/** Fingerprint stable basé sur user-agent + plateforme + timezone. */
export function browserFingerprint(): string {
  const ua = navigator.userAgent || 'unknown';
  const platform = (navigator as { platform?: string }).platform || 'unknown';
  const tz = Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
  const raw = `${ua}|${platform}|${tz}`;
  let h = 0x811c9dc5;
  for (let i = 0; i < raw.length; i++) {
    h ^= raw.charCodeAt(i);
    h = (h * 0x01000193) >>> 0;
  }
  const platformShort = platform.toLowerCase().replace(/[^a-z]/g, '').slice(0, 8) || 'ext';
  return `ext-${platformShort}-${h.toString(16).padStart(8, '0')}`;
}
