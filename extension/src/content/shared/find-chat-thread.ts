// Sobr.ia extension — trouve le conteneur scrollable du chat (design 38).
//
// La banner doit être insérée **dans** ce conteneur, en `position: sticky;
// top: 0` pour rester visible quand l'utilisateur scrolle dans la conversation
// (cf. design 38 §"Sobr.ia subtle top banner").
//
// Stratégie en cascade :
//   1. Cherche le premier message rendu (user ou bot) via les sélecteurs
//      passés en paramètre (variables selon le site).
//   2. Remonte les ancêtres jusqu'à trouver le premier qui scrolle vraiment
//      (`overflow-y: auto|scroll` ET hauteur visible < scrollHeight).
//   3. Fallback : retourne le parent direct du premier message, ou `null`.

/**
 * Cherche le conteneur scrollable du thread de chat, qui contient les bulles
 * de messages. Si trouvé, la banner doit y être insérée comme premier enfant.
 *
 * @param messageSelectors Liste de sélecteurs CSS qui matchent au moins UN
 *   message (user ou bot) une fois la conversation chargée. Le premier
 *   sélecteur qui matche est utilisé.
 * @returns L'élément scrollable, ou `null` si introuvable (page de garde
 *   sans conversation active par exemple).
 */
export function findChatScrollContainer(messageSelectors: string[]): HTMLElement | null {
  let firstMessage: Element | null = null;
  for (const sel of messageSelectors) {
    firstMessage = document.querySelector(sel);
    if (firstMessage) break;
  }
  if (!firstMessage) return null;

  // Remonte les ancêtres jusqu'à trouver un élément scrollable.
  let el: HTMLElement | null = firstMessage.parentElement;
  while (el && el !== document.body) {
    const style = getComputedStyle(el);
    const overflowY = style.overflowY;
    if (
      (overflowY === 'auto' || overflowY === 'scroll') &&
      el.scrollHeight > el.clientHeight + 8 // marge anti-bruit
    ) {
      return el;
    }
    el = el.parentElement;
  }

  // Aucun ancêtre scrollable détecté — fallback sur le parent direct (le
  // thread est sans doute en `overflow: visible` et `<main>` ou `<body>` scrolle).
  return firstMessage.parentElement as HTMLElement | null;
}

/** Alias court utilisé dans les content scripts. */
export const findChatThread = findChatScrollContainer;
