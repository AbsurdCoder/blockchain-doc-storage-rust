import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { getDocument } from '../api/documents';
import { TransferOwnershipModal } from '../components/TransferOwnershipModal';
import type { DocumentResponse } from '../types';

export function DocumentDetail() {
  const { id } = useParams<{ id: string }>();
  const [doc, setDoc] = useState<DocumentResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showTransfer, setShowTransfer] = useState(false);

  useEffect(() => {
    const docId = parseInt(id ?? '', 10);
    if (isNaN(docId)) {
      setError('Invalid document ID');
      setLoading(false);
      return;
    }
    getDocument(docId)
      .then(setDoc)
      .catch((err) => setError(err instanceof Error ? err.message : 'Failed to load'))
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) return <div className="text-slate-500">Loading...</div>;
  if (error) {
    return (
      <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-red-700">
        {error}
      </div>
    );
  }
  if (!doc) return null;

  return (
    <div>
      <Link
        to="/documents"
        className="mb-4 inline-block text-sm text-slate-600 hover:text-slate-900"
      >
        ← Back to documents
      </Link>
      <div className="rounded-lg border border-slate-200 bg-white p-6">
        <div className="flex items-start justify-between">
          <div>
            <h1 className="text-xl font-semibold text-slate-900">{doc.file_name}</h1>
            <dl className="mt-4 space-y-2 text-sm">
              <div>
                <dt className="text-slate-500">Document hash</dt>
                <dd className="font-mono text-slate-900">{doc.document_hash}</dd>
              </div>
              <div>
                <dt className="text-slate-500">Status</dt>
                <dd>{doc.status}</dd>
              </div>
              <div>
                <dt className="text-slate-500">MIME type</dt>
                <dd>{doc.mime_type}</dd>
              </div>
              <div>
                <dt className="text-slate-500">Created</dt>
                <dd>{new Date(doc.created_at).toLocaleString()}</dd>
              </div>
              {doc.blockchain_status && (
                <div>
                  <dt className="text-slate-500">Blockchain</dt>
                  <dd>{doc.blockchain_status}</dd>
                </div>
              )}
            </dl>
          </div>
          <button
            onClick={() => setShowTransfer(true)}
            className="rounded-md bg-amber-100 px-4 py-2 text-sm font-medium text-amber-800 hover:bg-amber-200"
          >
            Transfer ownership
          </button>
        </div>
      </div>
      <TransferOwnershipModal
        doc={showTransfer ? doc : null}
        onClose={() => setShowTransfer(false)}
        onSuccess={() => setShowTransfer(false)}
      />
    </div>
  );
}
