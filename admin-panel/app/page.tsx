'use client';
import { useState, useEffect } from 'react';
import LinkForm from '../components/LinkForm';
import LinkList from '../components/LinkList';
import {
  fetchLinks,
  SerializableShortLink,
  fetchHealth,
  HealthResponse,
  login as apiLogin,
  logout as apiLogout,
} from './api';

export default function AdminPage() {
  const [loggedIn, setLoggedIn] = useState(false);
  const [links, setLinks] = useState<Record<string, SerializableShortLink>>({});
  const [editing, setEditing] = useState<SerializableShortLink | null>(null);
  const [tokenInput, setTokenInput] = useState('');
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [checking, setChecking] = useState(false);

  const loadLinks = async () => {
    try {
      const data = await fetchLinks();
      setLinks(data);
      setLoggedIn(true);
    } catch (e) {
      console.error(e);
      setLoggedIn(false);
    }
  };

  const loadHealth = async () => {
    try {
      setChecking(true);
      const data = await fetchHealth();
      setHealth(data);
    } catch (e) {
      console.error(e);
      setHealth(null);
    } finally {
      setChecking(false);
    }
  };

  useEffect(() => {
    loadLinks();
    loadHealth();
    const id = setInterval(loadHealth, 30000);
    return () => clearInterval(id);
  }, []);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await apiLogin(tokenInput);
      setTokenInput('');
      await loadLinks();
    } catch (err) {
      alert('Login failed');
    }
  };

  const refresh = () => {
    loadLinks();
    setEditing(null);
  };

  if (!loggedIn) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4 bg-gray-50">
        <form
          onSubmit={handleLogin}
          className="space-y-4 w-full max-w-sm bg-white p-6 rounded-lg shadow"
        >
          <h1 className="text-xl font-semibold text-center">Admin Login</h1>
          <input
            type="password"
            value={tokenInput}
            onChange={(e) => setTokenInput(e.target.value)}
            placeholder="Admin Token"
            className="w-full border rounded px-3 py-2 focus:outline-none focus:ring focus:border-blue-500"
          />
          <button
            type="submit"
            className="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-500"
          >
            Enter
          </button>
        </form>
      </div>
    );
  }

  const statusColor =
    health?.status === 'healthy'
      ? 'bg-green-500'
      : health?.status === 'unhealthy'
      ? 'bg-red-500'
      : 'bg-gray-400';

  return (
    <div className="p-4 space-y-6 max-w-4xl mx-auto">
      <header className="flex justify-between items-center pb-4 border-b">
        <h1 className="text-2xl font-bold">Short Links</h1>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2 text-sm">
            <span className="relative flex h-3 w-3">
              <span
                className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${statusColor}`}
              ></span>
              <span className={`relative inline-flex rounded-full h-3 w-3 ${statusColor}`}></span>
            </span>
            <span>{checking ? 'checking' : health?.status ?? 'unknown'}</span>
          </div>
          <button
            className="text-sm text-red-600 hover:underline"
            onClick={async () => {
              await apiLogout();
              setLoggedIn(false);
            }}
          >
            Logout
          </button>
        </div>
      </header>

      <LinkForm editing={editing} onDone={refresh} />

      <div className="overflow-x-auto">
        <LinkList links={links} onEdit={setEditing} onDelete={refresh} />
      </div>
    </div>
  );
}

// 添加这个函数使页面在构建时静态生成
export async function generateStaticParams() {
  return [];
}