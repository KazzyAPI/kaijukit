import { InputRule, type AnyExtension } from '@tiptap/core';
import Link from '@tiptap/extension-link';
import Placeholder from '@tiptap/extension-placeholder';
import { Table } from '@tiptap/extension-table';
import TableCell from '@tiptap/extension-table-cell';
import TableHeader from '@tiptap/extension-table-header';
import TableRow from '@tiptap/extension-table-row';
import Underline from '@tiptap/extension-underline';
import StarterKit from '@tiptap/starter-kit';
import { Annotation, BackgroundColor, BlockFormat, DateNode, DecisionItem, DecisionList, Emoji, Expand, LayoutColumn, LayoutSection, LocalIds, Media, MediaGroup, MediaSingle, Mention, Panel, Status, TaskItem, TaskList, TextColor, TocExtension, type MediaOptions } from './extensions';

const MarkdownLink = Link.extend({
  addInputRules() {
    return [
      new InputRule({
        find: /\[([^\]]+)\]\((https?:\/\/[^\s)]+|mailto:[^\s)]+|tel:[^\s)]+)\)$/,
        handler: ({ range, match, chain }) => {
          const [, text = '', href = ''] = match;
          chain().insertContentAt(range, { type: 'text', text, marks: [{ type: 'link', attrs: { href } }] }).run();
        },
      }),
    ];
  },
});

export function kkExtensions(media: MediaOptions['resolve'] = () => Promise.resolve(''), placeholder = "Type '/' for commands"): AnyExtension[] {
  return [
    StarterKit,
    Underline,
    MarkdownLink.configure({ openOnClick: false, autolink: true, protocols: ['mailto', 'tel'], HTMLAttributes: { rel: 'noopener noreferrer nofollow' } }),
    Table,
    TableRow,
    TableHeader,
    TableCell,
    TextColor,
    BackgroundColor,
    Annotation,
    BlockFormat,
    LocalIds,
    TaskList,
    TaskItem,
    DecisionList,
    DecisionItem,
    Panel,
    Expand,
    LayoutSection,
    LayoutColumn,
    Status,
    DateNode,
    Mention,
    Emoji,
    TocExtension,
    MediaSingle,
    MediaGroup,
    Media.configure({ resolve: media }),
    Placeholder.configure({ placeholder }),
  ];
}
