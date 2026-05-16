// Sobr.ia extension — détection des soumissions de prompt (C27.3).
//
// Helper générique abstrait :
//   - Observe le DOM pour repérer textarea + bouton d'envoi (sites SPA → ils
//     n'existent pas au chargement initial).
//   - Écoute le clic sur le bouton d'envoi (event delegation document-level).
//   - Écoute Enter (sans Shift) sur le textarea (raccourci clavier).
//   - Throttle de 200 ms pour éviter les double-déclenchements (clic + Enter
//     simultanés sur certains sites).
//
// Le helper est volontairement passif : il **n'envoie rien**, il **n'affiche
// rien**. Il invoque seulement `onSubmit({ prompt, modelId })`. C'est au site
// adapter (chatgpt.ts, claude.ts, le-chat.ts) d'orchestrer la suite (estimer,
// injecter le badge, persister).

/** Configuration d'un détecteur de prompt pour un site donné. */
export type PromptDetectorConfig = {
  /** Sélecteur CSS du textarea / contenteditable où l'utilisateur tape. */
  readonly selectorTextarea: string;
  /** Sélecteur CSS du bouton d'envoi (cible des clics utilisateur). */
  readonly selectorSendButton: string;
  /** Extracteur d'identifiant de modèle courant (peut retourner null). */
  readonly extractModelId: () => string | null;
  /**
   * Extracteur du texte du prompt à partir de l'élément textarea.
   * Par défaut : `value` (HTMLTextAreaElement) ou `textContent` sinon.
   */
  readonly extractPrompt?: (textarea: Element) => string;
  /** Appelé à chaque soumission détectée (throttlé 200 ms). */
  readonly onSubmit: (data: { prompt: string; modelId: string | null }) => void;
};

/** Durée du throttle anti-double-déclenchement (ms). */
const THROTTLE_MS = 200;

/** Extracteur par défaut du texte d'un input/textarea/contenteditable. */
function defaultExtractPrompt(el: Element): string {
  if (el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement) {
    return el.value;
  }
  // contenteditable=true
  return el.textContent ?? '';
}

/**
 * Démarre la détection sur la page courante.
 *
 * Retourne une fonction de cleanup qui dégage tous les écouteurs et
 * l'observer DOM. Idempotente : peut être appelée plusieurs fois sans
 * effet de bord.
 */
export function observePromptSubmission(config: PromptDetectorConfig): () => void {
  let lastFireMs = 0;
  let disposed = false;
  const extract = config.extractPrompt ?? defaultExtractPrompt;

  function fireIfPromptReady(): void {
    if (disposed) return;
    const now = Date.now();
    if (now - lastFireMs < THROTTLE_MS) return;

    const textarea = document.querySelector(config.selectorTextarea);
    if (!textarea) return;
    const prompt = extract(textarea).trim();
    if (prompt.length === 0) return;

    lastFireMs = now;
    const modelId = safeExtractModelId(config.extractModelId);
    config.onSubmit({ prompt, modelId });
  }

  function onClickCapture(event: Event): void {
    const target = event.target as Element | null;
    if (!target) return;
    const sendButton = document.querySelector(config.selectorSendButton);
    if (!sendButton) return;
    if (sendButton === target || sendButton.contains(target)) {
      fireIfPromptReady();
    }
  }

  function onKeydownCapture(event: KeyboardEvent): void {
    if (event.key !== 'Enter' || event.shiftKey) return;
    const target = event.target as Element | null;
    if (!target) return;
    const textarea = document.querySelector(config.selectorTextarea);
    if (!textarea) return;
    if (textarea === target || textarea.contains(target)) {
      fireIfPromptReady();
    }
  }

  // Capture-phase pour ne pas être bloqué par stopPropagation des sites.
  document.addEventListener('click', onClickCapture, { capture: true, passive: true });
  document.addEventListener('keydown', onKeydownCapture, { capture: true });

  // MutationObserver : utile pour les sites SPA où textarea / bouton n'existent
  // pas immédiatement au document_idle. Pas d'action directe — on relit les
  // sélecteurs à chaque évènement utilisateur. L'observer reste connecté pour
  // que l'on puisse débrancher proprement avec dispose().
  const observer = new MutationObserver(() => {
    /* no-op : on relit les sélecteurs à chaque interaction utilisateur */
  });
  observer.observe(document.documentElement, { childList: true, subtree: true });

  return function dispose(): void {
    disposed = true;
    document.removeEventListener('click', onClickCapture, { capture: true });
    document.removeEventListener('keydown', onKeydownCapture, { capture: true });
    observer.disconnect();
  };
}

/** Wrappe `extractModelId` pour ne jamais propager une exception au callback. */
function safeExtractModelId(extract: () => string | null): string | null {
  try {
    return extract();
  } catch {
    return null;
  }
}
