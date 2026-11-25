import { describe, expect, it } from 'vitest';
import { commentText, isPlainComment, toCommentAdf } from './commentText';

describe('comment text', () => {
  it('round-trips lines as paragraphs and drops blank lines', () => {
    const doc = JSON.parse(toCommentAdf('one\n\n two '));
    expect(doc.content).toEqual([
      { type: 'paragraph', content: [{ type: 'text', text: 'one' }] },
      { type: 'paragraph', content: [{ type: 'text', text: ' two ' }] },
    ]);
    expect(commentText(doc)).toBe('one\n two ');
    expect(isPlainComment(doc)).toBe(true);
  });

  it('refuses to flatten formatted comments', () => {
    const bold = { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'x', marks: [{ type: 'strong' }] }] }] };
    expect(isPlainComment(bold)).toBe(false);
    expect(isPlainComment({ type: 'doc', content: [{ type: 'bulletList' }] })).toBe(false);
  });
});
