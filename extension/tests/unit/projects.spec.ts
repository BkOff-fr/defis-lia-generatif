// C44 — étiquettes projet par conversation : fonctions pures (la partie
// storage est un passe-plat chrome.storage, couvert par l'usage).
import { describe, expect, it } from 'vitest';
import {
  normalizeProjectName,
  PROJECT_MAX_LEN,
  threadKeyFromUrl
} from '../../src/content/shared/projects.js';
import { isTrackedUrl } from '../../src/popup/main.js';

describe('threadKeyFromUrl', () => {
  it('normalise host + pathname, sans query ni hash ni trailing slash', () => {
    expect(threadKeyFromUrl('https://claude.ai/chat/abc-123?x=1#y')).toBe('claude.ai/chat/abc-123');
    expect(threadKeyFromUrl('https://chatgpt.com/c/uuid-456/')).toBe('chatgpt.com/c/uuid-456');
    expect(threadKeyFromUrl('https://chat.mistral.ai/chat/789')).toBe('chat.mistral.ai/chat/789');
  });

  it('racine du site → host/', () => {
    expect(threadKeyFromUrl('https://claude.ai/')).toBe('claude.ai/');
  });

  it('rejette les URLs inexploitables', () => {
    expect(threadKeyFromUrl(undefined)).toBeNull();
    expect(threadKeyFromUrl(null)).toBeNull();
    expect(threadKeyFromUrl('')).toBeNull();
    expect(threadKeyFromUrl('about:blank')).toBeNull();
    expect(threadKeyFromUrl('http://claude.ai/chat/x')).toBeNull(); // pas https
    expect(threadKeyFromUrl('pas une url')).toBeNull();
  });
});

describe('normalizeProjectName', () => {
  it('trim + longueur max + vide → null', () => {
    expect(normalizeProjectName('  Refonte site  ')).toBe('Refonte site');
    expect(normalizeProjectName('   ')).toBeNull();
    expect(normalizeProjectName('')).toBeNull();
    const long = 'x'.repeat(PROJECT_MAX_LEN + 20);
    expect(normalizeProjectName(long)).toHaveLength(PROJECT_MAX_LEN);
  });
});

describe('isTrackedUrl', () => {
  it('accepte les 3 sites suivis (et chat.openai.com legacy)', () => {
    expect(isTrackedUrl('https://claude.ai/chat/abc')).toBe(true);
    expect(isTrackedUrl('https://chatgpt.com/c/def')).toBe(true);
    expect(isTrackedUrl('https://chat.openai.com/c/def')).toBe(true);
    expect(isTrackedUrl('https://chat.mistral.ai/chat/ghi')).toBe(true);
  });

  it('rejette le reste', () => {
    expect(isTrackedUrl('https://example.com/chat/abc')).toBe(false);
    expect(isTrackedUrl('https://claude.ai.evil.com/chat/abc')).toBe(false);
    expect(isTrackedUrl(undefined)).toBe(false);
  });
});
