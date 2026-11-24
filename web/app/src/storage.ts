export const store = {
  get(k: string): string | null {
    try {
      return localStorage.getItem(k);
    } catch {
      return null;
    }
  },
  set(k: string, v: string) {
    try {
      localStorage.setItem(k, v);
    } catch {
      /* storage unavailable: setting is per-session only */
    }
  },
  remove(k: string) {
    try {
      localStorage.removeItem(k);
    } catch {
      /* nothing to remove */
    }
  },
};

export function getJson<T>(k: string, fallback: T): T {
  try {
    return JSON.parse(store.get(k) ?? '') as T;
  } catch {
    return fallback;
  }
}
