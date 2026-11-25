import { Editor } from '@tiptap/core';
import { afterEach, describe, expect, it } from 'vitest';
import { fromAdf, toAdf, unsupportedTypes, type AdfNode } from './adf';
import { COLORS, changeIndent, setAlign } from './extensions';
import { buildCommands } from './commands';
import { date, emoji, layoutSection, mediaBlock, mention, status } from './insert';
import { kkExtensions } from './schema';

const text = (t: string, marks?: AdfNode['marks']): AdfNode => ({ type: 'text', text: t, ...(marks ? { marks } : {}) });
const doc = (...content: AdfNode[]) => ({ type: 'doc', content });
const para = (t: string): AdfNode => ({ type: 'paragraph', content: [text(t)] });

const editors: Editor[] = [];
const open = (d: AdfNode = doc(para('hello'))) => {
  const e = new Editor({ extensions: kkExtensions(), content: fromAdf(d) });
  editors.push(e);
  return e;
};
afterEach(() => editors.splice(0).forEach((e) => e.destroy()));
// Ids are random per node; the shape is what the tests pin down.
const adf = (e: Editor) => JSON.parse(JSON.stringify(toAdf(e.getJSON() as Parameters<typeof toAdf>[0]).content).replace(/"localId":"[^"]+"/g, '"localId":"ID"')) as AdfNode[];
const roundTrip = (d: AdfNode) => toAdf(fromAdf(d)).content;

describe('marks', () => {
  it('text color', () => {
    const e = open();
    e.chain().selectAll().setMark('textColor', { color: COLORS[1] }).run();
    expect(adf(e)).toEqual([{ type: 'paragraph', content: [text('hello', [{ type: 'textColor', attrs: { color: '#bf2600' } }])] }]);
  });

  it('highlight is a backgroundColor mark', () => {
    const e = open();
    e.chain().selectAll().setMark('backgroundColor', { color: '#fff0b3' }).run();
    expect(adf(e)).toEqual([{ type: 'paragraph', content: [text('hello', [{ type: 'backgroundColor', attrs: { color: '#fff0b3' } }])] }]);
    e.chain().selectAll().unsetMark('backgroundColor').run();
    expect(adf(e)).toEqual([para('hello')]);
  });

  it('inline comment annotation', () => {
    const e = open();
    e.chain().setTextSelection({ from: 2, to: 4 }).setMark('annotation', { id: 'c-1' }).run();
    expect(adf(e)).toEqual([{ type: 'paragraph', content: [text('h'), text('el', [{ type: 'annotation', attrs: { id: 'c-1', annotationType: 'inlineComment' } }]), text('lo')] }]);
  });
});

describe('block formatting', () => {
  it('aligns paragraphs and headings; left removes the mark', () => {
    const e = open(doc({ type: 'heading', attrs: { level: 2 }, content: [text('t')] }, para('p')));
    setAlign(e, 'center');
    expect(adf(e)[0]).toEqual({ type: 'heading', attrs: { level: 2 }, marks: [{ type: 'alignment', attrs: { align: 'center' } }], content: [text('t')] });
    e.chain().selectAll().run();
    setAlign(e, 'end');
    expect(adf(e)[1]).toEqual({ type: 'paragraph', marks: [{ type: 'alignment', attrs: { align: 'end' } }], content: [text('p')] });
    setAlign(e, null);
    expect(adf(e)).toEqual([{ type: 'heading', attrs: { level: 2 }, content: [text('t')] }, para('p')]);
  });

  it('indents up to level 6 and back to none', () => {
    const e = open();
    for (let i = 0; i < 8; i++) changeIndent(e, 1);
    expect(adf(e)).toEqual([{ type: 'paragraph', marks: [{ type: 'indentation', attrs: { level: 6 } }], content: [text('hello')] }]);
    for (let i = 0; i < 8; i++) changeIndent(e, -1);
    expect(adf(e)).toEqual([para('hello')]);
  });

  it('keeps alignment and indentation together', () => {
    const d = { type: 'paragraph', marks: [{ type: 'alignment', attrs: { align: 'center' } }, { type: 'indentation', attrs: { level: 2 } }], content: [text('x')] };
    expect(roundTrip(doc(d))).toEqual([d]);
  });

  it('does not format paragraphs inside list items', () => {
    const e = open(doc({ type: 'bulletList', content: [{ type: 'listItem', content: [para('a')] }] }));
    setAlign(e, 'center');
    expect(adf(e)).toEqual([{ type: 'bulletList', content: [{ type: 'listItem', content: [para('a')] }] }]);
  });
});

describe('action items and decisions', () => {
  const tasks = { type: 'taskList', attrs: { localId: 'l1' }, content: [{ type: 'taskItem', attrs: { localId: 'i1', state: 'DONE' }, content: [text('done')] }, { type: 'taskItem', attrs: { localId: 'i2', state: 'TODO' }, content: [text('todo')] }] };
  it('round-trips with inline content', () => {
    expect(roundTrip(doc(tasks))).toEqual([tasks]);
  });
  it('converts a paragraph and assigns ids', () => {
    const e = open();
    e.chain().toggleList('taskList', 'taskItem').run();
    expect(adf(e)).toEqual([{ type: 'taskList', attrs: { localId: 'ID' }, content: [{ type: 'taskItem', attrs: { localId: 'ID', state: 'TODO' }, content: [text('hello')] }] }]);
  });
  it('Enter on an empty last item leaves the list', () => {
    const e = open(doc({ type: 'taskList', content: [{ type: 'taskItem', attrs: { localId: 'a', state: 'TODO' }, content: [text('one')] }] }));
    e.commands.focus('end');
    e.view.someProp('handleKeyDown', (f) => f(e.view, new KeyboardEvent('keydown', { key: 'Enter' })));
    e.view.someProp('handleKeyDown', (f) => f(e.view, new KeyboardEvent('keydown', { key: 'Enter' })));
    expect(adf(e).map((n) => n.type)).toEqual(['taskList', 'paragraph']);
  });

  it('splits on Enter and keeps the state of the first', () => {
    const e = open(doc(tasks));
    e.chain().setTextSelection(4).splitListItem('taskItem').run();
    expect(adf(e)[0]!.content!.map((i) => i.attrs?.state)).toEqual(['DONE', 'DONE', 'TODO']);
  });
  it('decision list', () => {
    const e = open();
    e.chain().toggleList('decisionList', 'decisionItem').run();
    expect(adf(e)).toEqual([{ type: 'decisionList', attrs: { localId: 'ID' }, content: [{ type: 'decisionItem', attrs: { localId: 'ID', state: 'DECIDED' }, content: [text('hello')] }] }]);
  });
  it('flattens a nested task list', () => {
    const nested = { type: 'taskList', attrs: { localId: 'a' }, content: [{ type: 'taskList', attrs: { localId: 'b' }, content: [tasks.content[1]!] }] };
    expect(roundTrip(doc(nested))).toEqual([{ type: 'taskList', attrs: { localId: 'a' }, content: [tasks.content[1]] }]);
  });
});

describe('inserts', () => {
  const cmds = buildCommands({ canUpload: true, status: () => undefined, date: () => undefined, file: () => undefined });
  const run = (id: string, e: Editor) => cmds.find((c) => c.id === id)!.run(e);

  it.each(['info', 'note', 'success', 'warning', 'error'])('%s panel; a second panel command switches the type', (t) => {
    const e = open(doc({ type: 'paragraph' }));
    run('panel-info', e);
    run(`panel-${t}`, e);
    expect(adf(e)).toEqual([{ type: 'panel', attrs: { panelType: t }, content: [{ type: 'paragraph' }] }]);
  });

  it('expand', () => {
    const e = open(doc({ type: 'paragraph' }));
    run('expand', e);
    expect(adf(e)).toEqual([{ type: 'expand', attrs: { title: '' }, content: [{ type: 'paragraph' }] }, { type: 'paragraph' }]);
  });

  it('two and three column layouts', () => {
    const col = (width: number) => ({ type: 'layoutColumn', attrs: { width }, content: [{ type: 'paragraph' }] });
    const e = open(doc({ type: 'paragraph' }));
    run('layout2', e);
    expect(adf(e)).toEqual([{ type: 'layoutSection', content: [col(50), col(50)] }, { type: 'paragraph' }]);
    const f = open(doc({ type: 'paragraph' }));
    f.chain().insertContent(layoutSection(3)).run();
    expect(adf(f)).toEqual([{ type: 'layoutSection', content: [col(33.33), col(33.33), col(33.33)] }, { type: 'paragraph' }]);
  });

  it('table of contents is an extension node', () => {
    const e = open(doc({ type: 'paragraph' }));
    run('toc', e);
    expect(adf(e)).toEqual([{ type: 'extension', attrs: { extensionType: 'kk.macro', extensionKey: 'toc', parameters: { macroParams: {} }, layout: 'default' } }, { type: 'paragraph' }]);
    expect(unsupportedTypes(doc(...adf(e)))).toEqual([]);
    expect(unsupportedTypes(doc({ type: 'extension', attrs: { extensionKey: 'other' } }))).toEqual(['extension']);
  });

  it('status, date, mention and emoji are inline nodes', () => {
    const e = open(doc({ type: 'paragraph' }));
    e.chain().insertContent([status('In review', 'purple'), date('2026-11-25'), mention('u1', 'Ada Admin'), emoji('smile', '😄')]).run();
    expect(adf(e)).toEqual([
      {
        type: 'paragraph',
        content: [
          { type: 'status', attrs: { text: 'In review', color: 'purple', localId: 'ID' } },
          { type: 'date', attrs: { timestamp: '1795564800000' } },
          { type: 'mention', attrs: { id: 'u1', text: '@Ada Admin' } },
          { type: 'emoji', attrs: { shortName: ':smile:', id: '1f604', text: '😄' } },
        ],
      },
    ]);
  });

  it('image becomes mediaSingle and other files become mediaGroup', () => {
    const img = { id: 'att1', type: 'file' as const, collection: 'contentId-12', __fileName: 'a.png' };
    const pdf = { id: 'att2', type: 'file' as const, collection: 'contentId-12', __fileName: 'b.pdf' };
    const e = open(doc({ type: 'paragraph' }));
    e.chain().insertContent([mediaBlock(img), mediaBlock(pdf)]).run();
    expect(adf(e).slice(0, 2)).toEqual([
      { type: 'mediaSingle', attrs: { layout: 'center' }, content: [{ type: 'media', attrs: { id: 'att1', type: 'file', collection: 'contentId-12', __fileName: 'a.png' } }] },
      { type: 'mediaGroup', content: [{ type: 'media', attrs: { id: 'att2', type: 'file', collection: 'contentId-12', __fileName: 'b.pdf' } }] },
    ]);
  });

  it('upload commands are unavailable without an uploader', () => {
    const off = buildCommands({ canUpload: false, status: () => undefined, date: () => undefined, file: () => undefined });
    expect(off.filter((c) => c.available === false).map((c) => c.id)).toEqual(['image', 'file']);
  });
});

describe('storage round trips', () => {
  it('keeps overlapping inline comment annotations and their attributes through load and save', () => {
    const a = (id: string) => ({ type: 'annotation', attrs: { id, annotationType: 'inlineComment' } });
    const d = doc({ type: 'paragraph', content: [text('plain '), text('one', [a('c-1')]), text('both', [a('c-1'), a('c-2')]), text('two', [{ type: 'strong' }, a('c-2')])] });
    expect(roundTrip(d)).toEqual(d.content);
    const e = open(d);
    expect(adf(e)).toEqual(d.content);
  });
  it('keeps panels, expands, layouts and inline nodes unchanged', () => {
    const d = doc(
      { type: 'panel', attrs: { panelType: 'warning' }, content: [para('careful')] },
      { type: 'expand', attrs: { title: 'More' }, content: [para('hidden')] },
      { type: 'layoutSection', content: [{ type: 'layoutColumn', attrs: { width: 50 }, content: [para('l')] }, { type: 'layoutColumn', attrs: { width: 50 }, content: [para('r')] }] },
      { type: 'paragraph', content: [{ type: 'status', attrs: { text: 'Done', color: 'green', localId: 's' } }, text(' '), { type: 'date', attrs: { timestamp: '1700000000000' } }] },
    );
    expect(roundTrip(d)).toEqual(d.content);
  });
  it('drops colors that are not hex when rendering through the schema', () => {
    const e = open(doc({ type: 'paragraph', content: [text('x', [{ type: 'textColor', attrs: { color: 'red;background:url(x)' } }])] }));
    expect(e.getHTML()).not.toContain('url(');
  });
});
