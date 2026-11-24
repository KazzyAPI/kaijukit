import type { SVGProps } from 'react';

const paths = {
  chevronRight: 'M9 6l6 6-6 6',
  plus: 'M12 5v14M5 12h14',
  more: 'M5 12h.01M12 12h.01M19 12h.01',
  close: 'M6 6l12 12M18 6L6 18',
  check: 'M5 12.5l4.5 4.5L19 7.5',
  page: 'M7 3h7l5 5v13H7zM14 3v5h5',
  space: 'M4 5h16v14H4zM9 5v14',
  edit: 'M4 20h4L19 9l-4-4L4 16zM13 7l4 4',
  sun: 'M12 8a4 4 0 100 8 4 4 0 000-8zM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5L19 19M5 19l1.5-1.5M17.5 6.5L19 5',
  moon: 'M20 14.5A8 8 0 019.5 4 8 8 0 1020 14.5z',
  bold: 'M7 5h6a3.5 3.5 0 010 7H7zM7 12h7a3.5 3.5 0 010 7H7z',
  italic: 'M10 5h8M6 19h8M14 5l-4 14',
  underline: 'M7 4v7a5 5 0 0010 0V4M5 20h14',
  strike: 'M5 12h14M16 7.5C15 6 13.5 5.5 12 5.5c-2.5 0-4 1.2-4 3 0 1.3.8 2.2 2.5 2.7M8 16.5c1 1.5 2.5 2 4.2 2 2.5 0 4-1.2 4-3',
  code: 'M9 7l-5 5 5 5M15 7l5 5-5 5',
  link: 'M10 14a4 4 0 005.7 0l3-3a4 4 0 00-5.7-5.7l-1 1M14 10a4 4 0 00-5.7 0l-3 3A4 4 0 0011 18.7l1-1',
  bullets: 'M9 6h11M9 12h11M9 18h11M4.5 6h.01M4.5 12h.01M4.5 18h.01',
  numbers: 'M10 6h10M10 12h10M10 18h10M4 5l1-1v4M4 11h2l-2 3h2M4 17h2v1.5H4.5M4 20h2v-1.5',
  quote: 'M5 6v12M9 8h10M9 12h10M9 16h6',
  rule: 'M4 12h16',
  table: 'M4 5h16v14H4zM4 10h16M4 15h16M10 5v14M15 5v14',
  undo: 'M9 14L4 9l5-5M4 9h10a6 6 0 010 12h-3',
  redo: 'M15 14l5-5-5-5M20 9H10a6 6 0 000 12h3',
  trash: 'M5 7h14M10 7V4h4v3M7 7l1 13h8l1-13',
  copy: 'M9 9h10v11H9zM5 15V4h10',
  search: 'M11 4a7 7 0 100 14 7 7 0 000-14zM16 16l5 5',
  panel: 'M4 5h16v14H4zM9 5v14',
} as const;

export type IconName = keyof typeof paths;

export function Icon({ name, size = 16, ...rest }: { name: IconName; size?: number } & Omit<SVGProps<SVGSVGElement>, 'name'>) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false" {...rest}>
      <path d={paths[name]} />
    </svg>
  );
}
