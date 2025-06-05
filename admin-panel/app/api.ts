export interface SerializableShortLink {
  short_code: string;
  target_url: string;
  created_at: string;
  expires_at?: string | null;
}

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL ||
  process.env.VITE_API_BASE_URL ||
  '';
const ADMIN_ROUTE_PREFIX =
  process.env.NEXT_PUBLIC_ADMIN_ROUTE_PREFIX ||
  process.env.VITE_ADMIN_ROUTE_PREFIX ||
  '/admin';

const HEALTH_ROUTE_PREFIX =
  process.env.NEXT_PUBLIC_HEALTH_ROUTE_PREFIX ||
  process.env.VITE_HEALTH_ROUTE_PREFIX ||
  '/health';

const HEALTH_TOKEN =
  process.env.NEXT_PUBLIC_HEALTH_TOKEN ||
  process.env.VITE_HEALTH_TOKEN ||
  '';

function base(path: string) {
  return `${API_BASE_URL}${ADMIN_ROUTE_PREFIX}${path}`;
}

export async function fetchLinks(): Promise<Record<string, SerializableShortLink>> {
  const res = await fetch(base('/link'), {
    credentials: 'include',
    cache: 'no-store',
  });
  if (!res.ok) {
    throw new Error('Failed to fetch links');
  }
  const json = await res.json();
  return json.data || {};
}

export interface LinkPayload {
  code?: string;
  target: string;
  expires_at?: string | null;
}

export async function createLink(payload: LinkPayload) {
  const res = await fetch(base('/link'), {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    credentials: 'include',
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    throw new Error('Failed to create link');
  }
}

export async function updateLink(code: string, payload: LinkPayload) {
  const res = await fetch(base(`/link/${code}`), {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    credentials: 'include',
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    throw new Error('Failed to update link');
  }
}

export async function deleteLink(code: string) {
  const res = await fetch(base(`/link/${code}`), {
    method: 'DELETE',
    credentials: 'include',
  });
  if (!res.ok) {
    throw new Error('Failed to delete link');
  }
}

export async function login(token: string) {
  const res = await fetch(base('/login'), {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
    },
    credentials: 'include',
  });
  if (!res.ok) {
    throw new Error('Login failed');
  }
}

export async function logout() {
  await fetch(base('/logout'), { method: 'POST', credentials: 'include' });
}

export interface HealthResponse {
  status: string;
  [key: string]: any;
}

export async function fetchHealth(): Promise<HealthResponse> {
  const res = await fetch(`${API_BASE_URL}${HEALTH_ROUTE_PREFIX}`, {
    headers: HEALTH_TOKEN ? { Authorization: `Bearer ${HEALTH_TOKEN}` } : {},
    cache: 'no-store',
  });
  if (!res.ok) {
    throw new Error('Failed to fetch health');
  }
  return res.json();
}
