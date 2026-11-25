import { isImage, TOC_ATTRS, type MediaAttrs } from './extensions';

type J = { type: string; attrs?: Record<string, unknown>; content?: J[]; text?: string };
const p = (): J => ({ type: 'paragraph' });

export const layoutSection = (columns: 2 | 3): J => ({
  type: 'layoutSection',
  content: Array.from({ length: columns }, () => ({ type: 'layoutColumn', attrs: { width: Math.round((100 / columns) * 100) / 100 }, content: [p()] })),
});
export const panel = (panelType: string): J => ({ type: 'panel', attrs: { panelType }, content: [p()] });
export const expand = (): J => ({ type: 'expand', attrs: { title: '' }, content: [p()] });
export const toc = (): J => ({ type: 'extension', attrs: TOC_ATTRS });
export const status = (text: string, color: string): J => ({ type: 'status', attrs: { text, color } });
export const date = (iso: string): J => ({ type: 'date', attrs: { timestamp: String(Date.parse(`${iso}T00:00:00Z`)) } });
export const mention = (id: string, name: string): J => ({ type: 'mention', attrs: { id, text: `@${name}` } });
export const emoji = (shortName: string, text: string): J => ({ type: 'emoji', attrs: { shortName: `:${shortName}:`, id: [...text].map((c) => c.codePointAt(0)!.toString(16)).join('-'), text } });
export const decisionList = (): J => ({ type: 'decisionList', content: [{ type: 'decisionItem', content: [p()] }] });
export const media = (a: MediaAttrs): J => ({ type: 'media', attrs: { ...a } });
export const mediaBlock = (a: MediaAttrs): J => (isImage(a) ? { type: 'mediaSingle', attrs: { layout: 'center' }, content: [media(a)] } : { type: 'mediaGroup', content: [media(a)] });

export const EMOJIS: [string, string][] = [
  ['smile', '😄'], ['grin', '😁'], ['joy', '😂'], ['wink', '😉'], ['heart_eyes', '😍'], ['thinking', '🤔'], ['sunglasses', '😎'], ['cry', '😢'],
  ['thumbsup', '👍'], ['thumbsdown', '👎'], ['clap', '👏'], ['pray', '🙏'], ['wave', '👋'], ['raised_hands', '🙌'], ['muscle', '💪'], ['eyes', '👀'],
  ['heart', '❤️'], ['fire', '🔥'], ['star', '⭐'], ['sparkles', '✨'], ['tada', '🎉'], ['rocket', '🚀'], ['bulb', '💡'], ['warning', '⚠️'],
  ['white_check_mark', '✅'], ['x', '❌'], ['question', '❓'], ['bug', '🐛'], ['memo', '📝'], ['book', '📖'], ['calendar', '📅'], ['lock', '🔒'],
];
