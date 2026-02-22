import { useState } from 'react';
import { verifyDocument } from '../api/documents';
import type { VerificationResponse } from '../types';

export function DocumentVerifier() {
  const [hash, setHash] = useState('');
  const [file, setFile] = useState<File | null>(null);
  const [result, setResult] = useState<VerificationResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleVerifyHash() {
    if (!hash.trim()) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const res = await verifyDocument(hash.trim());
      setResult(res);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed');
    } finally {
      setLoading(false);
    }
  }

  async function handleVerifyFile() {
    if (!file) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const reader = new FileReader();
      const base64 = await new Promise<string>((resolve, reject) => {
        reader.onload = () => {
          const r = reader.result as string;
          resolve(r.split(',')[1] || '');
        };
        reader.onerror = reject;
        reader.readAsDataURL(file);
      });
      const res = await verifyDocument(undefined, base64);
      setResult(res);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed');
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-6">
      <div className="rounded-lg border border-slate-200 bg-white p-6">
        <h3 className="mb-4 font-medium text-slate-900">Verify by hash</h3>
        <div className="flex gap-3">
          <input
            type="text"
            value={hash}
            onChange={(e) => setHash(e.target.value)}
            placeholder="Document hash (hex)"
            className="flex-1 rounded-md border border-slate-300 px-3 py-2 font-mono text-sm"
          />
          <button
            onClick={handleVerifyHash}
            disabled={loading || !hash.trim()}
            className="rounded-md bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-900 disabled:opacity-50"
          >
            {loading ? 'Verifying...' : 'Verify'}
          </button>
        </div>
      </div>

      <div className="rounded-lg border border-slate-200 bg-white p-6">
        <h3 className="mb-4 font-medium text-slate-900">Verify by file</h3>
        <div className="flex gap-3">
          <input
            type="file"
            onChange={(e) => setFile(e.target.files?.[0] ?? null)}
            className="flex-1 rounded-md border border-slate-300 px-3 py-2 text-sm"
          />
          <button
            onClick={handleVerifyFile}
            disabled={loading || !file}
            className="rounded-md bg-slate-800 px-4 py-2 text-sm font-medium text-white hover:bg-slate-900 disabled:opacity-50"
          >
            {loading ? 'Verifying...' : 'Verify'}
          </button>
        </div>
      </div>

      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-red-700">
          {error}
        </div>
      )}

      {result && (
        <div className="rounded-lg border border-emerald-200 bg-emerald-50 p-6">
          <h3 className="mb-3 font-medium text-emerald-900">Result</h3>
          <dl className="space-y-2 text-sm">
            <div>
              <dt className="text-emerald-700">Exists</dt>
              <dd className="font-medium">{result.exists ? 'Yes' : 'No'}</dd>
            </div>
            <div>
              <dt className="text-emerald-700">Document hash</dt>
              <dd className="font-mono text-xs">{result.document_hash}</dd>
            </div>
            <div>
              <dt className="text-emerald-700">Blockchain confirmed</dt>
              <dd className="font-medium">
                {result.blockchain_confirmed ? 'Yes' : 'No'}
              </dd>
            </div>
            {result.owner && (
              <div>
                <dt className="text-emerald-700">Owner</dt>
                <dd>{result.owner}</dd>
              </div>
            )}
          </dl>
        </div>
      )}
    </div>
  );
}
