import { describe, expect, it } from 'vitest';
import { fromAdf, toAdf, unsupportedTypes, wordCount, type AdfDoc } from './adf';

const sample: AdfDoc = {
  version: 1,
  type: 'doc',
  content: [
    { type: 'heading', attrs: { level: 2 }, content: [{ type: 'text', text: 'Title' }] },
    {
      type: 'paragraph',
      content: [
        { type: 'text', text: 'plain ' },
        { type: 'text', text: 'bold', marks: [{ type: 'strong' }] },
        { type: 'text', text: ' link', marks: [{ type: 'link', attrs: { href: 'https://example.com' } }] },
        { type: 'hardBreak' },
        { type: 'text', text: 'code', marks: [{ type: 'code' }] },
      ],
    },
    { type: 'bulletList', content: [{ type: 'listItem', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'a' }] }] }] },
    { type: 'orderedList', attrs: { order: 3 }, content: [{ type: 'listItem', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'b' }] }] }] },
    { type: 'codeBlock', attrs: { language: 'ts' }, content: [{ type: 'text', text: 'let x = 1;' }] },
    { type: 'blockquote', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'q', marks: [{ type: 'em' }, { type: 'underline' }, { type: 'strike' }] }] }] },
    { type: 'rule' },
    {
      type: 'table',
      content: [
        { type: 'tableRow', content: [{ type: 'tableHeader', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'h' }] }] }] },
        { type: 'tableRow', content: [{ type: 'tableCell', attrs: { colspan: 2 }, content: [{ type: 'paragraph' }] }] },
      ],
    },
    { type: 'paragraph' },
  ],
};

describe('ADF <-> editor JSON', () => {
  it('converts ADF to the literal editor document', () => {
    expect(fromAdf({ type: 'doc', content: sample.content.slice(0, 2) })).toEqual({
      type: 'doc',
      content: [
        { type: 'heading', attrs: { level: 2 }, content: [{ type: 'text', text: 'Title' }] },
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'plain ' },
            { type: 'text', text: 'bold', marks: [{ type: 'bold' }] },
            { type: 'text', text: ' link', marks: [{ type: 'link', attrs: { href: 'https://example.com' } }] },
            { type: 'hardBreak' },
            { type: 'text', text: 'code', marks: [{ type: 'code' }] },
          ],
        },
      ],
    });
  });

  it('round-trips every M1 node and mark', () => {
    expect(toAdf(fromAdf(sample))).toEqual(sample);
  });

  it('writes an empty document as one empty paragraph', () => {
    expect(toAdf({ type: 'doc' })).toEqual({ version: 1, type: 'doc', content: [{ type: 'paragraph' }] });
  });

  it('flattens unsupported nodes and reports them', () => {
    const doc = { type: 'doc', content: [{ type: 'nestedExpand', attrs: { title: 'x' }, content: [{ type: 'paragraph', content: [{ type: 'text', text: 'in panel' }] }] }, { type: 'blockCard' }] };
    expect(unsupportedTypes(doc)).toEqual(['nestedExpand', 'blockCard']);
    expect(toAdf(fromAdf(doc)).content).toEqual([
      { type: 'paragraph', content: [{ type: 'text', text: 'in panel' }] },
      { type: 'paragraph', content: [{ type: 'text', text: '[blockCard]' }] },
    ]);
  });

  it('counts words across blocks', () => {
    const doc = { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'one two' }] }, { type: 'paragraph', content: [{ type: 'text', text: 'three' }] }] };
    expect(wordCount(doc)).toBe(3);
    expect(wordCount(null)).toBe(0);
  });
});
