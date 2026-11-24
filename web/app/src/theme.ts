import { useEffect, useState } from 'react';
import { store } from './storage';

export type Theme = 'light' | 'dark' | 'auto';
const KEY = 'kk.theme';

export function applyTheme(t: Theme) {
  if (t === 'auto') document.documentElement.removeAttribute('data-theme');
  else document.documentElement.setAttribute('data-theme', t);
}

export function useTheme() {
  const [theme, set] = useState<Theme>(() => (store.get(KEY) as Theme | null) ?? 'auto');
  useEffect(() => {
    applyTheme(theme);
    store.set(KEY, theme);
  }, [theme]);
  return [theme, set] as const;
}
