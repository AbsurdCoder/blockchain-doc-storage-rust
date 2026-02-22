import { Link } from 'react-router-dom';
import type { DocumentResponse } from '../types';

interface Props {
  doc: DocumentResponse;
  onTransfer?: (doc: DocumentResponse) => void;
}

export function DocumentCard({ doc, onTransfer }: Props) {
  const shortHash = doc.document_hash.slice(0, 16) + '…';

  return (
    <div className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
      <div className="flex items-start justify-between">
        <div>
          <h3 className="font-medium text-slate-900">{doc.file_name}</h3>
          <p className="mt-1 font-mono text-xs text-slate-500">{shortHash}</p>
          <p className="mt-1 text-sm text-slate-600">
            {doc.mime_type} · {doc.status}
          </p>
          <p className="mt-0.5 text-xs text-slate-400">
            {new Date(doc.created_at).toLocaleString()}
          </p>
        </div>
        <div className="flex gap-2">
          <Link
            to={`/documents/${doc.id}`}
            className="rounded-md bg-slate-100 px-3 py-1.5 text-sm text-slate-700 hover:bg-slate-200"
          >
            View
          </Link>
          {onTransfer && (
            <button
              onClick={() => onTransfer(doc)}
              className="rounded-md bg-amber-100 px-3 py-1.5 text-sm text-amber-800 hover:bg-amber-200"
            >
              Transfer
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
