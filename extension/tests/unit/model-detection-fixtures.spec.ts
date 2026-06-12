import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  collectModelLabels,
  resolveModelFromLabels
} from '../../src/content/shared/model-resolver.js';

const __dirname = dirname(fileURLToPath(import.meta.url));

function loadFixture(name: string): string {
  return readFileSync(resolve(__dirname, '..', 'fixtures', name), 'utf8');
}

function setFixture(html: string): void {
  document.documentElement.innerHTML = html.replace(/^<!doctype html>/i, '').trim();
}

const CHATGPT_ALIASES: Record<string, string> = {
  'chatgpt 4o': 'gpt-4o',
  'gpt-4o': 'gpt-4o'
};

const CLAUDE_ALIASES: Record<string, string> = {
  'claude 3.5 sonnet': 'claude-3-5-sonnet'
};

const LE_CHAT_ALIASES: Record<string, string> = {
  'mistral large 2': 'mistral-large-2'
};

describe('model-detection — fixtures HTML', () => {
  it('ChatGPT 4o (libellé bouton modèle)', () => {
    setFixture(loadFixture('chatgpt-2026-05.html'));
    const labels = collectModelLabels(["[data-testid='model-switcher-dropdown-button']"]);
    expect(resolveModelFromLabels(labels, CHATGPT_ALIASES)?.presetId).toBe('gpt-4o');
  });

  it('Claude 3.5 Sonnet', () => {
    setFixture(loadFixture('claude-2026-05.html'));
    const labels = collectModelLabels(['nav button']).filter((l) => /claude/i.test(l));
    expect(resolveModelFromLabels(labels, CLAUDE_ALIASES)?.presetId).toBe('claude-3-5-sonnet');
  });

  it('Le Chat data-model', () => {
    setFixture(loadFixture('le-chat-2026-05.html'));
    const labels = collectModelLabels(['[data-model]']);
    expect(resolveModelFromLabels(labels, LE_CHAT_ALIASES)?.presetId).toBe('mistral-large-2');
  });
});
