const API_URL = import.meta.env.VITE_API_URL || '';

const STORAGE_KEY = 'blockchain_doc_token';

export function getToken(): string | null {
  return localStorage.getItem(STORAGE_KEY);
}

export function setToken(token: string | null): void {
  if (token) {
    localStorage.setItem(STORAGE_KEY, token);
  } else {
    localStorage.removeItem(STORAGE_KEY);
  }
}

export async function api<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const base = API_URL || '';
  const url = path.startsWith('http') ? path : `${base}${path}`;
  const token = getToken();

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const res = await fetch(url, { ...options, headers });
  const text = await res.text();
  if (!res.ok) {
    let msg = res.statusText;
    try {
      const j = JSON.parse(text);
      msg = j.message ?? j.detail ?? text || msg;
    } catch {
      msg = text || msg;
    }
    throw new Error(msg);
  }
  if (!text.trim()) return {} as T;
  return JSON.parse(text) as T;
}
