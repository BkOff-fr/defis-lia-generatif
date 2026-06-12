// Sobr.ia extension — content script Le Chat (chat.mistral.ai) — design 38 v3.

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
import { registryLabel } from '../lib/registry-meta.js';
import {
  collectModelLabels,
  resolveModelFromLabels
} from './shared/model-resolver.js';
import type { EstimationSubmittedMessage } from '../lib/messages.js';

const HOST = 'le-chat' as const;

const SELECTOR_TEXTAREA = "textarea[name='message'], textarea[placeholder*='Ask' i]";
const SELECTOR_SEND_BUTTON = "button[type='submit'], button[aria-label*='Send' i]";

// Ordre = specific-first (« large 3 » avant « large » sinon `includes()`
// matche le générique en premier sur les pages affichant la version).
const MODEL_NAME_TO_PRESET_ID: Record<string, string> = {
  // 2025 (C34.2)
  'mistral medium 3.5': 'mistral-medium-3-5',
  'mistral medium 3': 'mistral-medium-3',
  'mistral large 3': 'mistral-large-3',
  'mistral small 4': 'mistral-small-4',
  // génériques (sans version → dernier en date)
  'mistral medium': 'mistral-medium-3-5',
  'mistral large': 'mistral-large-3',
  'mistral small': 'mistral-small-4',
  // 2024 (deprecated)
  'mistral large 2': 'mistral-large-2'
};

const LE_CHAT_MODEL_LABEL_SELECTORS = [
  '[data-model]',
  "button[aria-label*='Modèle' i]",
  "button[aria-label*='model' i]",
  'button[aria-haspopup]'
] as const;

function readModelLabelFromUi(): string {
  const labels = collectModelLabels(LE_CHAT_MODEL_LABEL_SELECTORS).filter((l) =>
    /mistral/i.test(l)
  );
  return resolveModelFromLabels(labels, MODEL_NAME_TO_PRESET_ID)?.label ?? '';
}

function extractModelId(): string | null {
  const labels = collectModelLabels(LE_CHAT_MODEL_LABEL_SELECTORS);
  const mistralLabels = labels.filter((l) => /mistral/i.test(l) || l.includes('-'));
  return resolveModelFromLabels(mistralLabels, MODEL_NAME_TO_PRESET_ID)?.presetId ?? null;
}

function findBotActionsRow(): Element | null {
  const copyBtns = document.querySelectorAll<HTMLElement>(
    "button[aria-label*='Copier' i], button[aria-label*='Copy' i]"
  );
  const last = copyBtns[copyBtns.length - 1];
  if (last) {
    return last.parentElement;
  }
  const turns = document.querySelectorAll("[data-role='assistant'], article[data-role='bot']");
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
    composerSelector: 'form, footer',
    sendButtonSelector: SELECTOR_SEND_BUTTON,
    extractModelId
  });
}

(async function bootstrap(): Promise<void> {
  if (window.location.hostname !== 'chat.mistral.ai') return;
  const sites = await getSitesEnabled();
  if (!sites.leChat) {
    console.info('[sobria] Le Chat désactivé via options — skip.');
    return;
  }
  console.info(`[sobria] content script Le Chat chargé (${registryLabel()}, design 38)`);

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
            "button[aria-label*='Copier' i], button[aria-label*='Copy' i], [data-role='assistant']"
          );
          const actionsRow = findBotActionsRow();
          const label = readModelLabelFromUi();
          if (actionsRow) {
            injectUnsupportedBadge(actionsRow, label ? { modelLabel: label } : {});
          }
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
            "button[aria-label*='Copier' i], button[aria-label*='Copy' i], [data-role='assistant']"
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
        console.error('[sobria] estimation Le Chat échouée :', err);
      }
    }
  });
})();
