'use client';
import { SerializableShortLink, deleteLink } from '../app/api';

interface Props {
  links: Record<string, SerializableShortLink>;
  onEdit: (link: SerializableShortLink) => void;
  onDelete: () => void;
}

export default function LinkList({ links, onEdit, onDelete }: Props) {
  const handleDelete = async (code: string) => {
    if (confirm(`Delete link ${code}?`)) {
      await deleteLink(code);
      onDelete();
    }
  };

  return (
    <table className="min-w-full divide-y divide-gray-200 bg-white shadow rounded-md">
      <thead>
        <tr>
          <th className="px-2 py-1 text-left">Code</th>
          <th className="px-2 py-1 text-left">Target</th>
          <th className="px-2 py-1 text-left">Expires</th>
          <th className="px-2 py-1"></th>
        </tr>
      </thead>
      <tbody className="divide-y divide-gray-100">
        {Object.values(links).map((link) => (
          <tr key={link.short_code} className="hover:bg-gray-50">
            <td className="px-2 py-1 font-mono">{link.short_code}</td>
            <td className="px-2 py-1 break-all">
              <a href={link.target_url} className="text-blue-600 hover:underline" target="_blank" rel="noreferrer">
                {link.target_url}
              </a>
            </td>
            <td className="px-2 py-1 text-sm">{link.expires_at || '-'}</td>
            <td className="px-2 py-1 space-x-2 whitespace-nowrap">
              <button className="text-blue-600 hover:underline" onClick={() => onEdit(link)}>
                Edit
              </button>
              <button className="text-red-600 hover:underline" onClick={() => handleDelete(link.short_code)}>
                Delete
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
