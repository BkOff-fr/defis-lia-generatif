// Sobr.ia extension — tests prompt-detector (C27.3).
//
// Charge les fixtures HTML statiques des 3 sites cibles dans happy-dom et
// vérifie que `observePromptSubmission` :
//   - Détecte un clic sur le bouton d'envoi
//   - Détecte la touche Entrée (sans Shift) sur le textarea
//   - Ignore Shift+Entrée (saut de ligne)
//   - Throttle les double-déclenchements (clic + Entrée < 200 ms)
//   - Récupère le bon `modelId` depuis le DOM

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

import { observePromptSubmission } from '../../src/content/shared/prompt-detector.js';

const __dirname = dirname(fileURLToPath(import.meta.url));

function loadFixture(name: string): string {
  return readFileSync(resolve(__dirname, '..', 'fixtures', name), 'utf8');
}

function setFixture(html: string): void {
  document.documentElement.innerHTML = html.replace(/^<!doctype html>/i, '').trim();
}

describe('prompt-detector — ChatGPT fixture', () => {
  let dispose: (() => void) | undefined;

  beforeEach(() => {
    setFixture(loadFixture('chatgpt-2026-05.html'));
  });

  afterEach(() => {
    dispose?.();
    dispose = undefined;
  });

  it('détecte un clic sur le bouton d’envoi', () => {
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: '#prompt-textarea',
      selectorSendButton: "[data-testid='send-button']",
      extractModelId: () => 'gpt-4o',
      onSubmit
    });

    const btn = document.querySelector<HTMLButtonElement>("[data-testid='send-button']")!;
    btn.click();

    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(onSubmit).toHaveBeenCalledWith({
      prompt: "Quelle est l'empreinte d'un prompt ?",
      modelId: 'gpt-4o'
    });
  });

  it('détecte la touche Entrée sur le textarea (sans Shift)', () => {
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: '#prompt-textarea',
      selectorSendButton: "[data-testid='send-button']",
      extractModelId: () => 'gpt-4o',
      onSubmit
    });

    const textarea = document.querySelector<HTMLTextAreaElement>('#prompt-textarea')!;
    textarea.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));

    expect(onSubmit).toHaveBeenCalledTimes(1);
  });

  it('ignore Shift+Entrée (saut de ligne)', () => {
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: '#prompt-textarea',
      selectorSendButton: "[data-testid='send-button']",
      extractModelId: () => 'gpt-4o',
      onSubmit
    });

    const textarea = document.querySelector<HTMLTextAreaElement>('#prompt-textarea')!;
    textarea.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Enter', shiftKey: true, bubbles: true })
    );

    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('throttle les double-déclenchements (clic + Entrée immédiats)', () => {
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: '#prompt-textarea',
      selectorSendButton: "[data-testid='send-button']",
      extractModelId: () => 'gpt-4o',
      onSubmit
    });

    const textarea = document.querySelector<HTMLTextAreaElement>('#prompt-textarea')!;
    const btn = document.querySelector<HTMLButtonElement>("[data-testid='send-button']")!;
    btn.click();
    textarea.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));

    // Throttle 200 ms : seul le premier doit passer.
    expect(onSubmit).toHaveBeenCalledTimes(1);
  });

  it('ignore les soumissions vides (textarea blanc)', () => {
    const textarea = document.querySelector<HTMLTextAreaElement>('#prompt-textarea')!;
    textarea.value = '   ';
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: '#prompt-textarea',
      selectorSendButton: "[data-testid='send-button']",
      extractModelId: () => 'gpt-4o',
      onSubmit
    });

    const btn = document.querySelector<HTMLButtonElement>("[data-testid='send-button']")!;
    btn.click();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('dispose() débranche les écouteurs', () => {
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: '#prompt-textarea',
      selectorSendButton: "[data-testid='send-button']",
      extractModelId: () => 'gpt-4o',
      onSubmit
    });
    dispose();

    const btn = document.querySelector<HTMLButtonElement>("[data-testid='send-button']")!;
    btn.click();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('extractModelId qui throw n’interrompt pas la détection', () => {
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: '#prompt-textarea',
      selectorSendButton: "[data-testid='send-button']",
      extractModelId: () => {
        throw new Error('boom');
      },
      onSubmit
    });

    const btn = document.querySelector<HTMLButtonElement>("[data-testid='send-button']")!;
    btn.click();

    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(onSubmit.mock.calls[0]?.[0]?.modelId).toBeNull();
  });
});

describe('prompt-detector — Claude fixture', () => {
  let dispose: (() => void) | undefined;

  beforeEach(() => {
    setFixture(loadFixture('claude-2026-05.html'));
  });

  afterEach(() => {
    dispose?.();
    dispose = undefined;
  });

  it("détecte clic sur button[aria-label='Send Message']", () => {
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: "div[contenteditable='true']",
      selectorSendButton: "button[aria-label='Send Message']",
      extractModelId: () => 'claude-3-5-sonnet',
      onSubmit
    });

    const btn = document.querySelector<HTMLButtonElement>("button[aria-label='Send Message']")!;
    btn.click();

    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(onSubmit.mock.calls[0]?.[0]?.modelId).toBe('claude-3-5-sonnet');
    expect(onSubmit.mock.calls[0]?.[0]?.prompt).toContain('empreinte CO2');
  });
});

describe('prompt-detector — Le Chat fixture', () => {
  let dispose: (() => void) | undefined;

  beforeEach(() => {
    setFixture(loadFixture('le-chat-2026-05.html'));
  });

  afterEach(() => {
    dispose?.();
    dispose = undefined;
  });

  it("détecte clic sur button[type='submit']", () => {
    const onSubmit = vi.fn();
    dispose = observePromptSubmission({
      selectorTextarea: "textarea[name='message']",
      selectorSendButton: "button[type='submit']",
      extractModelId: () => 'mistral-large-2',
      onSubmit
    });

    const btn = document.querySelector<HTMLButtonElement>("button[type='submit']")!;
    btn.click();

    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(onSubmit.mock.calls[0]?.[0]?.modelId).toBe('mistral-large-2');
  });
});
