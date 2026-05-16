// Sobr.ia extension — wrapper typé chrome.storage.local (C27.3).
//
// chrome.storage.local survit aux fermetures de navigateur (~10 Mo quota).
// Toutes les écritures passent par ce module pour garder une schémathèque
// unique et faciliter les migrations futures.

import type { EmpreinteMethod } from '../../lib/types.js';
import type { DailyTotal, LastEstimate } from '../../lib/messages.js';

/** Clés de stockage. Toute évolution doit être documentée pour migration. */
const KEYS = {
  method: 'method',
  lastEstimate: 'last_estimate',
  dailyPrefix: 'daily_',
  sessionLog: 'session_log',
  sitesEnabled: 'sites_enabled',
  badgeVisible: 'badge_visible',
  pairing: 'pairing_state'
} as const;

/** État de pairing local avec l'app Sobr.ia desktop (C27.5). */
export type PairingState = {
  readonly secret: string;
  readonly pairingId: string;
  readonly fingerprint: string;
  readonly pairedAt: string;
};

/** Récupère l'état de pairing courant (null si non pairée). */
export async function getPairingState(): Promise<PairingState | null> {
  const result = await chrome.storage.local.get(KEYS.pairing);
  const stored = result[KEYS.pairing];
  if (!stored || typeof stored !== 'object') return null;
  return stored as PairingState;
}

/** Persiste l'état de pairing après validation côté app. */
export async function setPairingState(state: PairingState): Promise<void> {
  await chrome.storage.local.set({ [KEYS.pairing]: state });
}

/** Efface l'état de pairing (dépairer côté extension). */
export async function clearPairingState(): Promise<void> {
  await chrome.storage.local.remove(KEYS.pairing);
}

/** Sites monitorés par l'extension (toggles options). */
export type SitesEnabled = {
  readonly chatgpt: boolean;
  readonly claude: boolean;
  readonly leChat: boolean;
};

const DEFAULT_SITES: SitesEnabled = { chatgpt: true, claude: true, leChat: true };

export async function getSitesEnabled(): Promise<SitesEnabled> {
  const result = await chrome.storage.local.get(KEYS.sitesEnabled);
  const stored = result[KEYS.sitesEnabled];
  if (!stored || typeof stored !== 'object') return DEFAULT_SITES;
  return { ...DEFAULT_SITES, ...(stored as Partial<SitesEnabled>) };
}

export async function setSitesEnabled(sites: SitesEnabled): Promise<void> {
  await chrome.storage.local.set({ [KEYS.sitesEnabled]: sites });
}

/** Affichage du badge en page (peut être désactivé via options). Défaut : true. */
export async function getBadgeVisible(): Promise<boolean> {
  const result = await chrome.storage.local.get(KEYS.badgeVisible);
  const stored = result[KEYS.badgeVisible];
  if (typeof stored !== 'boolean') return true;
  return stored;
}

export async function setBadgeVisible(visible: boolean): Promise<void> {
  await chrome.storage.local.set({ [KEYS.badgeVisible]: visible });
}

/** Nombre max d'entrées dans le sparkline cumul session. */
const SESSION_LOG_MAX = 50;

/** Entrée du log session (1 par estimation, pour sparkline cumul). */
export type SessionLogEntry = {
  readonly ts: string;
  readonly gco2eq: number;
  readonly waterMl: number;
  readonly energyWh: number;
};

/** Méthodologie active (défaut AFNOR Sobr.ia, cohérent app desktop ADR-0012). */
export async function getMethod(): Promise<EmpreinteMethod> {
  const result = await chrome.storage.local.get(KEYS.method);
  const stored = result[KEYS.method];
  if (stored === 'afnor_sobria' || stored === 'ecologits') return stored;
  return 'afnor_sobria';
}

/** Persiste la méthodologie active. */
export async function setMethod(method: EmpreinteMethod): Promise<void> {
  await chrome.storage.local.set({ [KEYS.method]: method });
}

/** Snapshot du dernier prompt mesuré (lu par popup C27.4). */
export async function getLastEstimate(): Promise<LastEstimate | null> {
  const result = await chrome.storage.local.get(KEYS.lastEstimate);
  const stored = result[KEYS.lastEstimate];
  if (!stored || typeof stored !== 'object') return null;
  return stored as LastEstimate;
}

/** Persiste le dernier prompt mesuré. */
export async function setLastEstimate(estimate: LastEstimate): Promise<void> {
  await chrome.storage.local.set({ [KEYS.lastEstimate]: estimate });
}

/** Calcule l'identifiant de la journée locale au format YYYY-MM-DD. */
export function todayKey(now: Date = new Date()): string {
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, '0');
  const d = String(now.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

/** Récupère le total quotidien pour aujourd'hui (0 si aucun prompt mesuré). */
export async function getTodayTotal(): Promise<DailyTotal> {
  const date = todayKey();
  const key = `${KEYS.dailyPrefix}${date}`;
  const result = await chrome.storage.local.get(key);
  const stored = result[key];
  if (!stored || typeof stored !== 'object') {
    return { date, count: 0, gco2eq: 0, waterMl: 0, energyWh: 0 };
  }
  return stored as DailyTotal;
}

/**
 * Incrémente le total quotidien avec une nouvelle mesure.
 *
 * Opération idempotente sur l'estampille de la mesure : rien n'empêche un
 * double-fire si le content script appelle deux fois. C'est au détecteur de
 * dédupliquer (throttle 200 ms côté prompt-detector).
 */
export async function appendToDailyTotal(args: {
  gco2eq: number;
  waterMl: number;
  energyWh: number;
}): Promise<DailyTotal> {
  const date = todayKey();
  const key = `${KEYS.dailyPrefix}${date}`;
  const current = await getTodayTotal();
  const next: DailyTotal = {
    date,
    count: current.count + 1,
    gco2eq: current.gco2eq + args.gco2eq,
    waterMl: current.waterMl + args.waterMl,
    energyWh: current.energyWh + args.energyWh
  };
  await chrome.storage.local.set({ [key]: next });
  return next;
}

/** Log de session — N derniers prompts (anneau circulaire). */
export async function getSessionLog(): Promise<readonly SessionLogEntry[]> {
  const result = await chrome.storage.local.get(KEYS.sessionLog);
  const stored = result[KEYS.sessionLog];
  if (!Array.isArray(stored)) return [];
  return stored as SessionLogEntry[];
}

/** Pousse une entrée dans le log session, tronqué à `SESSION_LOG_MAX`. */
export async function appendToSessionLog(
  entry: SessionLogEntry
): Promise<readonly SessionLogEntry[]> {
  const current = await getSessionLog();
  const next = [...current, entry].slice(-SESSION_LOG_MAX);
  await chrome.storage.local.set({ [KEYS.sessionLog]: next });
  return next;
}

/** Purge complète (utilisé par options → confidentialité). */
export async function purgeAll(): Promise<void> {
  await chrome.storage.local.clear();
}
