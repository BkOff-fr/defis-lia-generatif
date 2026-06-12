// Sobr.ia — étiquettes projet par conversation (C44, ADR-0016).
//
// Le projet est choisi PAR CONVERSATION (pas par prompt) : la clé est
// dérivée de l'URL de l'onglet (host + pathname, sans query/hash). Le
// mapping vit dans `chrome.storage.local` — il ne quitte le navigateur
// que sous forme d'étiquette (`project`) attachée aux estimations
// poussées vers le serveur ÉQUIPE (jamais le contenu de la page).
//
// Utilisé par :
// - le popup (sélecteur « Projet de cette conversation ») ;
// - le service worker (résolution au moment du dispatch équipe, via
//   `sender.tab.url` — zéro modification des content scripts).

const PROJECTS_LIST_KEY = 'sobria_projects_list';
const THREAD_PROJECTS_KEY = 'sobria_thread_projects';

/** Longueur max d'une étiquette (alignée sur la normalisation serveur). */
export const PROJECT_MAX_LEN = 64;

/**
 * Clé de conversation depuis une URL d'onglet : `host/pathname` normalisé
 * (sans query/hash, sans trailing slash). Retourne `null` si l'URL n'est
 * pas exploitable (page interne, about:blank…).
 */
export function threadKeyFromUrl(url: string | undefined | null): string | null {
  if (!url) return null;
  try {
    const u = new URL(url);
    if (u.protocol !== 'https:') return null;
    const path = u.pathname.replace(/\/+$/, '');
    return `${u.host}${path || '/'}`;
  } catch {
    return null;
  }
}

/** Normalise une étiquette projet (trim + longueur max). `null` si vide. */
export function normalizeProjectName(raw: string): string | null {
  const t = raw.trim().slice(0, PROJECT_MAX_LEN);
  return t.length > 0 ? t : null;
}

/** Liste des projets connus de ce navigateur (ordre de création). */
export async function getProjectsList(): Promise<string[]> {
  const data = await chrome.storage.local.get(PROJECTS_LIST_KEY);
  const list = data[PROJECTS_LIST_KEY];
  return Array.isArray(list) ? (list as string[]) : [];
}

/** Ajoute un projet à la liste (idempotent). Retourne la liste à jour. */
export async function addProject(raw: string): Promise<string[]> {
  const name = normalizeProjectName(raw);
  const list = await getProjectsList();
  if (name === null || list.includes(name)) return list;
  const next = [...list, name];
  await chrome.storage.local.set({ [PROJECTS_LIST_KEY]: next });
  return next;
}

/** Projet affecté à une conversation (`null` = hors projet). */
export async function getProjectForThread(threadKey: string): Promise<string | null> {
  const data = await chrome.storage.local.get(THREAD_PROJECTS_KEY);
  const map = (data[THREAD_PROJECTS_KEY] ?? {}) as Record<string, string>;
  return map[threadKey] ?? null;
}

/** Affecte (ou retire, avec `null`) le projet d'une conversation. */
export async function setProjectForThread(
  threadKey: string,
  project: string | null
): Promise<void> {
  const data = await chrome.storage.local.get(THREAD_PROJECTS_KEY);
  const map = (data[THREAD_PROJECTS_KEY] ?? {}) as Record<string, string>;
  const next = Object.fromEntries(Object.entries(map).filter(([k]) => k !== threadKey));
  if (project !== null) {
    next[threadKey] = project;
  }
  await chrome.storage.local.set({ [THREAD_PROJECTS_KEY]: next });
}

/**
 * Résolution complète depuis une URL d'onglet — utilisée par le service
 * worker au dispatch : `null` si pas de clé ou pas d'affectation.
 */
export async function resolveProjectForUrl(url: string | undefined): Promise<string | null> {
  const key = threadKeyFromUrl(url);
  if (key === null) return null;
  return getProjectForThread(key);
}
