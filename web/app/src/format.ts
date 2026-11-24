const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });
const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [['year', 31536000], ['month', 2592000], ['day', 86400], ['hour', 3600], ['minute', 60]];

export function relativeTime(iso: string, now = Date.now()): string {
  const s = Math.round((new Date(iso).getTime() - now) / 1000);
  for (const [u, n] of UNITS) if (Math.abs(s) >= n) return rtf.format(Math.round(s / n), u);
  return 'just now';
}
export const exactTime = (iso: string) => new Date(iso).toLocaleString();
export const slugify = (t: string) => t.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
export const suggestKey = (name: string) => name.replace(/[^A-Za-z0-9]/g, '').toUpperCase().slice(0, 8);
export const pageUrl = (key: string, id: string, title: string) => `/wiki/spaces/${key}/pages/${id}${title ? `/${slugify(title)}` : ''}`;
export const readingMinutes = (words: number) => Math.max(1, Math.ceil(words / 200));
export const shortDate = (iso: string) => new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: new Date(iso).getFullYear() === new Date().getFullYear() ? undefined : 'numeric' });
