// Sobr.ia extension — helper de recherche du thread scrollable du chat.
//
// La banner doit être insérée **dans** le conteneur scrollable de la
// conversation pour rester sticky lors du scroll (et ne pas être clippée
// par les overflow:hidden parents de ChatGPT/Claude/Le Chat).
//
// Stratégie : trouver le premier message rendu, puis remonter dans le DOM
// jusqu'au premier ancêtre `overflow-y: auto | scroll`. Fallback : le parent
// direct du premier message.

/**
 * Trouve le conteneur scrollable du thread de chat.
 *
 * Cherche le premier message rendu via une liste de sélecteurs robustes
 * (multi-sites), puis remonte la chaîne des parents jusqu'à un ancêtre
 * dont `overflow-y` est `auto` ou `scroll`. Retourne `null` si rien ne
 * matche (la banner sera alors injectée dans `<main>` en fallback).
 */
export function findChatThread(messageSelectors: readonly string[]): HTMLElement | null {
  for (const sel of messageSelectors) {
    const el = document.querySelector(sel);
    if (!el) continue;
    let cursor: HTMLElement | null = el.parentElement;
    while (cursor && cursor !== document.body) {
      const style = getComputedStyle(cursor);
      if (style.overflowY === 'auto' || style.overflowY === 'scroll') {
        return cursor;
      }
      cursor = cursor.parentElement;
    }
    // Fallback : parent direct du premier message.
    if (el.parentElement instanceof HTMLElement) return el.parentElement;
  }
  return null;
}
