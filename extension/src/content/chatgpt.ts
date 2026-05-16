// Sobr.ia extension — content script ChatGPT (chat.openai.com + chatgpt.com).
//
// Composants livrés :
//   - Composer indicator : cercle compact à droite du composer pendant la frappe
//   - Bouton Sobr.ia injecté dans la **rangée d'actions du message bot**
//     (copier / like / dislike / lire / réécrire / partager / **Sobr.ia**)
//   - Popout 540 px sous le bouton avec 4 onglets

import { observePromptSubmission } from './shared/prompt-detector.js';
import { injectBadge, injectUnsupportedBadge } from './shared/badge-injector.js';
import { injectComposerIndicator } from './shared/composer-indicator.js';
import { estimateTokens, estimateOutputTokens } from './shared/tokens.js';
import {
  getMethod,
  appendToDailyTotal,
  appendToSessionLog,
  setLastEstimate,
  getSitesEnabled,
  getBadgeVisible
} from './shared/storage.js';
import { waitForNewMatching } from './shared/wait-for-bubble.js';
import { estimate } from '../lib/empreinte/index.js';
import { findPreset } from '../lib/presets.js';
import type { EstimationSubmittedMessage } from '../lib/messages.js';

const HOST = 'chatgpt' as const;
const ALLOWED_HOSTNAMES = new Set(['chat.openai.com', 'chatgpt.com']);

// ─── Sélecteurs ──────────────────────────────────────────────────────────────

const SELECTOR_TEXTAREA = '#prompt-textarea, div[contenteditable="true"]#prompt-textarea';
const SELECTOR_SEND_BUTTON =
  "[data-testid='send-button'], [data-testid='composer-send-button'], button[aria-label*='envoyer' i], button[aria-label*='send' i]";

const MODEL_NAME_TO_PRESET_ID: Record<string, string> = {
  'gpt-4o': 'gpt-4o',
  'gpt-4 turbo': 'gpt-4o',
  'gpt-4': 'gpt-4o',
  'gpt-4o mini': 'gpt-4o-mini',
  'gpt-4o-mini': 'gpt-4o-mini',
  'o1-preview': 'gpt-4o',
  'o1-mini': 'gpt-4o-mini',
  'gpt-5': 'gpt-4o' // mapping provisoire si GPT-5 visible
};

function extractModelId(): string | null {
  const candidates = document.querySelectorAll(
    "[data-testid='model-switcher-dropdown-button'], header button[aria-haspopup], button[aria-label*='Model']"
  );
  for (const el of Array.from(candidates)) {
    const raw = (el.textContent ?? '').trim().toLowerCase();
    if (!raw) continue;
    for (const [key, presetId] of Object.entries(MODEL_NAME_TO_PRESET_ID)) {
      if (raw.includes(key)) return presetId;
    }
  }
  const urlModel = new URL(window.location.href).searchParams.get('model');
  if (urlModel) {
    const mapped = MODEL_NAME_TO_PRESET_ID[urlModel.toLowerCase()];
    if (mapped) return mapped;
  }
  // Modèle inconnu → null (l'UI affichera « non pris en charge » au lieu de
  // mentir avec une fausse valeur). Mieux que de fallback silencieux sur GPT-4o.
  return null;
}

/**
 * Trouve la **rangée d'actions du dernier message bot** où injecter le badge.
 *
 * Stratégie en cascade :
 *  1. Cherche un élément qui contient les boutons natifs (copy / thumbs up / down).
 *  2. Fallback : prend le bouton « copier » et utilise son `parentElement`.
 *  3. Dernier fallback : le message bot lui-même (le badge sera attaché à la fin).
 */
function findBotActionsRow(): Element | null {
  // Heuristique : on cherche un bouton « copier » dans le dernier turn bot.
  const copyButtons = document.querySelectorAll<HTMLElement>(
    "[data-testid='copy-turn-action-button'], button[aria-label*='Copier' i], button[aria-label*='Copy' i]"
  );
  const lastCopy = copyButtons[copyButtons.length - 1];
  if (lastCopy) {
    return lastCopy.parentElement;
  }
  // Fallback : dernier message rendu (peut être user ou bot, mais reste cohérent).
  const turns = document.querySelectorAll(
    "[data-message-author-role='assistant'], article[data-testid^='conversation-turn']"
  );
  return turns[turns.length - 1] ?? null;
}

/**
 * (Re)injecte l'indicateur live au-dessus du composer ChatGPT, si le textarea
 * est présent. Idempotent — la fonction supprime un indicateur existant avant
 * de réinjecter.
 */
let composerCleanup: (() => void) | null = null;
function tryInjectComposerIndicator(): void {
  const textarea = document.querySelector(SELECTOR_TEXTAREA);
  if (!textarea) return;
  // Si l'indicateur existe déjà et pointe sur le même textarea, on garde.
  if (document.querySelector('[data-sobria-typing]') && composerCleanup) return;
  composerCleanup?.();
  composerCleanup = injectComposerIndicator({
    textarea,
    composerSelector:
      "form, [data-testid='composer'], [data-testid='composer-root'], div[role='presentation']",
    sendButtonSelector: SELECTOR_SEND_BUTTON,
    extractModelId
  });
}

/**
 * Refresh banner avec totaux session courants.
 *
 * Cible le **thread scrollable** (banner sticky `top:0` au-dessus du premier
 * message, suit le scroll de la conversation). Fallback : `<main>` ou `<body>`.
 */

// ─── Bootstrap ───────────────────────────────────────────────────────────────

(async function bootstrap(): Promise<void> {
  if (!ALLOWED_HOSTNAMES.has(window.location.hostname)) return;
  const sites = await getSitesEnabled();
  if (!sites.chatgpt) {
    console.info('[sobria] ChatGPT désactivé via options — skip.');
    return;
  }
  console.info('[sobria] content script ChatGPT chargé (v0.6.0, design 38)');

  // Indicateur composer initial + re-injection sur changement de DOM.
  tryInjectComposerIndicator();

  // Observer : re-(re)injecte l'indicateur si le textarea est remplacé par
  // ChatGPT (changement de conversation).
  const domObserver = new MutationObserver(() => {
    if (!document.querySelector('[data-sobria-typing]')) {
      tryInjectComposerIndicator();
    }
  });
  domObserver.observe(document.documentElement, { childList: true, subtree: true });

  observePromptSubmission({
    selectorTextarea: SELECTOR_TEXTAREA,
    selectorSendButton: SELECTOR_SEND_BUTTON,
    extractModelId,
    onSubmit: async ({ prompt, modelId }) => {
      try {
        // Modèle non pris en charge → badge dégradé, on n'enregistre rien
        // (pas de fausse estimation dans les totaux session/jour).
        if (modelId === null) {
          await waitForNewMatching(
            "[data-testid='copy-turn-action-button'], article[data-testid^='conversation-turn']:last-of-type"
          );
          const actionsRow = findBotActionsRow();
          if (actionsRow) injectUnsupportedBadge(actionsRow);
          return;
        }

        const method = await getMethod();
        const tokensIn = estimateTokens(prompt);
        const tokensOut = estimateOutputTokens(tokensIn);
        const result = estimate({
          method,
          modelId,
          tokensIn,
          tokensOut
        });

        // Persistance immédiate (ne bloque pas l'injection visuelle).
        const session = await appendToDailyTotal({
          gco2eq: result.gco2eq,
          waterMl: result.waterMl,
          energyWh: result.energyWh
        });
        await appendToSessionLog({
          ts: new Date().toISOString(),
          gco2eq: result.gco2eq,
          waterMl: result.waterMl,
          energyWh: result.energyWh
        });
        const preset = findPreset(modelId);
        await setLastEstimate({
          estimate: result,
          host: HOST,
          modelDisplayName: preset?.displayName ?? modelId,
          ts: new Date().toISOString()
        });

        // Attend l'apparition de la row d'actions du nouveau message bot.
        const badgeVisible = await getBadgeVisible();
        if (badgeVisible) {
          await waitForNewMatching(
            "[data-testid='copy-turn-action-button'], article[data-testid^='conversation-turn']:last-of-type"
          );
          const actionsRow = findBotActionsRow();
          if (actionsRow) {
            injectBadge(actionsRow, result, { session });
          }
        }

        const message: EstimationSubmittedMessage = {
          type: 'estimation_submitted',
          estimate: result,
          host: HOST,
          modelDisplayName: preset?.displayName ?? modelId
        };
        chrome.runtime.sendMessage(message).catch(() => {
          /* best-effort */
        });
      } catch (err) {
        console.error('[sobria] estimation ChatGPT échouée :', err);
      }
    }
  });
})();
