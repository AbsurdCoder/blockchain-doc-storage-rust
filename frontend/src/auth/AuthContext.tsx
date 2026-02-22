import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from 'react';
import { getToken } from '../api/client';
import * as authApi from '../api/auth';
import type { User } from '../types';

interface AuthState {
  user: User | null;
  token: string | null;
  loading: boolean;
}

interface AuthContextValue extends AuthState {
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, name?: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>({
    user: null,
    token: getToken(),
    loading: true,
  });

  const logout = useCallback(() => {
    authApi.logout();
    setState({ user: null, token: null, loading: false });
  }, []);

  useEffect(() => {
    const token = getToken();
    if (!token) {
      setState((s) => ({ ...s, loading: false }));
      return;
    }
    // Optionally validate token with GET /api/auth/me
    setState((s) => ({ ...s, loading: false }));
  }, []);

  const login = useCallback(async (email: string, password: string) => {
    const res = await authApi.login(email, password);
    setState({
      user: res.user,
      token: res.token,
      loading: false,
    });
  }, []);

  const register = useCallback(
    async (email: string, password: string, name?: string) => {
      const res = await authApi.register(email, password, name);
      setState({
        user: res.user,
        token: res.token,
        loading: false,
      });
    },
    []
  );

  const value: AuthContextValue = {
    ...state,
    login,
    register,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
