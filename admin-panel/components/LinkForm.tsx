'use client';
import { useState, useEffect } from 'react';
import { createLink, updateLink, LinkPayload, SerializableShortLink } from '../app/api';

interface Props {
  editing?: SerializableShortLink | null;
  onDone: () => void;
}

export default function LinkForm({ editing, onDone }: Props) {
  const [code, setCode] = useState('');
  const [target, setTarget] = useState('');
  const [expiresAt, setExpiresAt] = useState<string>('');

  useEffect(() => {
    if (editing) {
      setCode(editing.short_code);
      setTarget(editing.target_url);
      setExpiresAt(editing.expires_at || '');
    } else {
      setCode('');
      setTarget('');
      setExpiresAt('');
    }
  }, [editing]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const payload: LinkPayload = { code: code || undefined, target, expires_at: expiresAt || undefined };
    if (editing) {
      await updateLink(editing.short_code, { target, expires_at: expiresAt || undefined });
    } else {
      await createLink(payload);
    }
    onDone();
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 p-4 border rounded-md bg-white shadow">
      <div className="flex flex-col gap-1">
        <label className="font-medium">Code</label>
        <input
          type="text"
          value={code}
          onChange={(e) => setCode(e.target.value)}
          className="border rounded px-2 py-1 focus:outline-none focus:ring focus:border-blue-500"
          disabled={!!editing}
        />
      </div>
      <div className="flex flex-col gap-1">
        <label className="font-medium">Target URL</label>
        <input
          type="text"
          value={target}
          onChange={(e) => setTarget(e.target.value)}
          required
          className="border rounded px-2 py-1 focus:outline-none focus:ring focus:border-blue-500"
        />
      </div>
      <div className="flex flex-col gap-1">
        <label className="font-medium">Expires At</label>
        <input
          type="text"
          value={expiresAt}
          onChange={(e) => setExpiresAt(e.target.value)}
          placeholder="optional"
          className="border rounded px-2 py-1 focus:outline-none focus:ring focus:border-blue-500"
        />
      </div>
      <button
        type="submit"
        className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-500"
      >
        {editing ? 'Update' : 'Create'}
      </button>
    </form>
  );
}
