import { useEffect, useState } from 'react';
import { listDocuments } from '../api/documents';
import { DocumentCard } from './DocumentCard';
import type { DocumentResponse } from '../types';

interface Props {
  onTransfer?: (doc: DocumentResponse) => void;
  refreshTrigger?: number;
}

export function DocumentList({ onTransfer, refreshTrigger = 0 }: Props) {
  const [docs, setDocs] = useState<DocumentResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    listDocuments()
      .then((list) => {
        if (!cancelled) setDocs(list);
      })
      .catch((err) => {
        if (!cancelled) setError(err instanceof Error ? err.message : 'Failed to load');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => { cancelled = true; };
  }, [refreshTrigger]);

  if (loading) {
    return <div className="text-slate-500">Loading documents...</div>;
  }

  if (error) {
    return (
      <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-red-700">
        {error}
      </div>
    );
  }

  if (docs.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-slate-300 bg-white p-12 text-center text-slate-500">
        No documents yet. Upload your first document.
      </div>
    );
  }

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {docs.map((doc) => (
        <DocumentCard key={doc.id} doc={doc} onTransfer={onTransfer} />
      ))}
    </div>
  );
}
