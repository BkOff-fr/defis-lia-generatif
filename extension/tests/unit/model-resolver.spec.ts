import { describe, it, expect } from 'vitest';
import {
  normalizeModelLabel,
  resolveModelAlias,
  tryDirectPresetId
} from '../../src/content/shared/model-resolver.js';

const CHATGPT_ALIASES: Record<string, string> = {
  'gpt-5.5 thinking': 'gpt-5-5-thinking',
  'gpt-5.5': 'gpt-5-5',
  'chatgpt 4o': 'gpt-4o',
  'chatgpt 4o mini': 'gpt-4o-mini',
  'gpt-4o': 'gpt-4o'
};

describe('model-resolver', () => {
  it('normalise tirets et espaces', () => {
    expect(normalizeModelLabel('  ChatGPT\u00a04o  ')).toBe('chatgpt 4o');
  });

  it('résout ChatGPT 4o (libellé UI courant)', () => {
    expect(resolveModelAlias('ChatGPT 4o', CHATGPT_ALIASES)).toBe('gpt-4o');
  });

  it('résout un id preset direct', () => {
    expect(tryDirectPresetId('claude-sonnet-4-6')).toBe('claude-sonnet-4-6');
  });

  it('résout via displayName registry', () => {
    expect(resolveModelAlias('Claude Sonnet 4.6', {})).toBe('claude-sonnet-4-6');
  });
});
