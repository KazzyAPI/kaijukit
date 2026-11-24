import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from 'react';
import { setCredentials } from './api';
import { getJson, store } from './storage';

interface User {
  email: string;
  token: string;
}
interface AuthValue {
  user: User | null;
  login: (u: User) => void;
  logout: () => void;
}

const KEY = 'kk.auth';
const Ctx = createContext<AuthValue>({ user: null, login: () => undefined, logout: () => undefined });
export const useAuth = () => useContext(Ctx);

function initial(): User | null {
  const u = getJson<User | null>(KEY, null);
  setCredentials(u);
  return u?.email && u.token ? u : null;
}

export function AuthProvider({ children, onLogout }: { children: ReactNode; onLogout: (ref: () => void) => void }) {
  const [user, setUser] = useState<User | null>(initial);
  const login = useCallback((u: User) => {
    setCredentials(u);
    store.set(KEY, JSON.stringify(u));
    setUser(u);
  }, []);
  const logout = useCallback(() => {
    setCredentials(null);
    store.remove(KEY);
    setUser(null);
  }, []);
  onLogout(logout);
  const value = useMemo(() => ({ user, login, logout }), [user, login, logout]);
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}
