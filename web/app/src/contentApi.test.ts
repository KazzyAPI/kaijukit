import { describe, expect, it } from 'vitest';
import { cql } from './contentApi';

describe('cql', () => {
  it('quotes and escapes quotes and backslashes', () => {
    expect(cql('plain')).toBe('"plain"');
    expect(cql('a "b" \\ c')).toBe('"a \\"b\\" \\\\ c"');
  });
});
