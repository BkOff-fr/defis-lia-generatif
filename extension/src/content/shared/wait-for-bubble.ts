// Sobr.ia extension — attente d'apparition d'un élément DOM (C27.3 itération UX).
//
// Les sites comme ChatGPT/Claude/Le Chat sont des SPA : à la soumission d'un
// prompt, la bulle du message utilisateur est rendue dans le DOM avec un
// léger délai (50-300 ms). Pour attacher le badge `juste après cette bulle`,
// on observe l'arrivée d'un nouveau nœud qui matche `selector` et qui
// n'existait pas avant la soumission.

/**
 * Attend l'apparition d'un nouvel élément matchant `selector` après l'appel.
 *
 * Retourne :
 * - Le **dernier** nouvel élément apparu (le plus récent) si on en repère un.
 * - Le dernier élément matchant déjà présent au timeout (fallback ad-hoc).
 * - `null` si rien ne matche au timeout.
 *
 * Pourquoi un fallback "dernier élément existant" : certains sites
 * réutilisent les nœuds (le message utilisateur est intégré au DOM avant le
 * clic d'envoi, dans une zone tampon). Dans ce cas, il vaut mieux pointer
 * sur le dernier nœud existant que de retourner `null`.
 */
export function waitForNewMatching(selector: string, timeoutMs = 3000): Promise<Element | null> {
  return new Promise((resolve) => {
    const initialSet = new Set<Element>(document.querySelectorAll(selector));

    let settled = false;
    const settle = (el: Element | null): void => {
      if (settled) return;
      settled = true;
      observer.disconnect();
      clearTimeout(timer);
      resolve(el);
    };

    const observer = new MutationObserver(() => {
      const matches = document.querySelectorAll(selector);
      let newest: Element | null = null;
      for (const el of Array.from(matches)) {
        if (!initialSet.has(el)) newest = el;
      }
      if (newest) settle(newest);
    });

    observer.observe(document.body, { childList: true, subtree: true });

    const timer = setTimeout(() => {
      // Fallback : dernier élément existant si rien de nouveau n'a poussé.
      const all = document.querySelectorAll(selector);
      const last = all.length > 0 ? (all[all.length - 1] ?? null) : null;
      settle(last);
    }, timeoutMs);
  });
}
