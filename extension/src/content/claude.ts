// Sobr.ia extension — content script Claude (claude.ai) — design 38 v3.

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

const HOST = 'claude' as const;

const SELECTOR_TEXTAREA =
  "div[contenteditable='true'][role='textbox'], div[contenteditable='true']";
const SELECTOR_SEND_BUTTON =
  "button[aria-label='Send Message'], button[aria-label*='envoyer' i], button[type='submit']";

const MODEL_NAME_TO_PRESET_ID: Record<string, string> = {
  'claude 3.5 sonnet': 'claude-3-5-sonnet',
  'claude 3.5 haiku': 'claude-3-5-sonnet',
  'claude 3 opus': 'claude-3-5-sonnet',
  'claude 3 sonnet': 'claude-3-5-sonnet',
  'claude 3 haiku': 'claude-3-5-sonnet'
};

function extractModelId(): string | null {
  const candidates = document.querySelectorAll("button, [role='button']");
  for (const el of Array.from(candidates)) {
    const text = (el.textContent ?? '').toLowerCase();
    if (!text.includes('claude')) continue;
    for (const [key, presetId] of Object.entries(MODEL_NAME_TO_PRESET_ID)) {
      if (text.includes(key)) return presetId;
    }
  }
  // Modèle inconnu (Opus 4.x, futurs Claude…) → null pour afficher
  // « non pris en charge » au lieu de mentir avec Sonnet 3.5.
  return null;
}

function findBotActionsRow(): Element | null {
  const copyBtns = document.querySelectorAll<HTMLElement>(
    "button[aria-label*='Copier' i], button[aria-label*='Copy' i]"
  );
  const last = copyBtns[copyBtns.length - 1];
  if (last) {
    return last.parentElement;
  }
  const turns = document.querySelectorAll(
    "[data-is-streaming='false'], [class*='font-claude-message']"
  );
  return turns[turns.length - 1] ?? null;
}

let composerCleanup: (() => void) | null = null;
function tryInjectComposerIndicator(): void {
  const textarea = document.querySelector(SELECTOR_TEXTAREA);
  if (!textarea) return;
  if (document.querySelector('[data-sobria-typing]') && composerCleanup) return;
  composerCleanup?.();
  composerCleanup = injectComposerIndicator({
    textarea,
    composerSelector: "form, fieldset, div[role='presentation']",
    sendButtonSelector: SELECTOR_SEND_BUTTON,
    extractModelId
  });
}

(async function bootstrap(): Promise<void> {
  if (window.location.hostname !== 'claude.ai') return;
  const sites = await getSitesEnabled();
  if (!sites.claude) {
    console.info('[sobria] Claude désactivé via options — skip.');
    return;
  }
  console.info('[sobria] content script Claude chargé (v0.6.0, design 38)');

  tryInjectComposerIndicator();

  const domObserver = new MutationObserver(() => {
    if (!document.querySelector('[data-sobria-typing]')) tryInjectComposerIndicator();
  });
  domObserver.observe(document.documentElement, { childList: true, subtree: true });

  observePromptSubmission({
    selectorTextarea: SELECTOR_TEXTAREA,
    selectorSendButton: SELECTOR_SEND_BUTTON,
    extractModelId,
    onSubmit: async ({ prompt, modelId }) => {
      try {
        if (modelId === null) {
          await waitForNewMatching(
            "button[aria-label*='Copier' i], button[aria-label*='Copy' i], [data-is-streaming='false']"
          );
          const actionsRow = findBotActionsRow();
          if (actionsRow) injectUnsupportedBadge(actionsRow);
          return;
        }

        const method = await getMethod();
        const tokensIn = estimateTokens(prompt);
        const tokensOut = estimateOutputTokens(tokensIn);
        const result = estimate({ method, modelId, tokensIn, tokensOut });

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

        const badgeVisible = await getBadgeVisible();
        if (badgeVisible) {
          await waitForNewMatching(
            "button[aria-label*='Copier' i], button[aria-label*='Copy' i], [data-is-streaming='false']"
          );
          const actionsRow = findBotActionsRow();
          if (actionsRow) injectBadge(actionsRow, result, { session });
        }

        const message: EstimationSubmittedMessage = {
          type: 'estimation_submitted',
          estimate: result,
          host: HOST,
          modelDisplayName: preset?.displayName ?? modelId
        };
        chrome.runtime.sendMessage(message).catch(() => undefined);
      } catch (err) {
        console.error('[sobria] estimation Claude échouée :', err);
      }
    }
  });
})();
