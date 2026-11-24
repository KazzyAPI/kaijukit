import { describe, expect, it } from 'vitest';
import { pageUrl, relativeTime, slugify, suggestKey } from './format';

describe('format', () => {
  it('slugifies titles and builds page urls', () => {
    expect(slugify('Hello, World! 2')).toBe('hello-world-2');
    expect(pageUrl('DOC', '12', 'Hello World')).toBe('/wiki/spaces/DOC/pages/12/hello-world');
    expect(pageUrl('DOC', '12', '')).toBe('/wiki/spaces/DOC/pages/12');
  });
  it('suggests a key and formats relative time', () => {
    expect(suggestKey('My team space!')).toBe('MYTEAMSP');
    const now = Date.parse('2026-01-02T00:00:00Z');
    expect(relativeTime('2026-01-01T22:00:00Z', now)).toBe('2 hours ago');
    expect(relativeTime('2026-01-02T00:00:00Z', now)).toBe('just now');
  });
});
