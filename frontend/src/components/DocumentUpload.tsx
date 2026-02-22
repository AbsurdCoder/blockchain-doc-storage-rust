import { useState, useRef } from 'react';
import { uploadDocument } from '../api/documents';
import type { DocumentResponse } from '../types';

interface Props {
  onSuccess: (doc: DocumentResponse) => void;
  onError: (err: string) => void;
}

export function DocumentUpload({ onSuccess, onError }: Props) {
  const [uploading, setUploading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  async function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploading(true);
    try {
      const doc = await uploadDocument(file);
      onSuccess(doc);
      if (inputRef.current) inputRef.current.value = '';
    } catch (err) {
      onError(err instanceof Error ? err.message : 'Upload failed');
    } finally {
      setUploading(false);
    }
  }

  return (
    <div className="flex items-center gap-3">
      <input
        ref={inputRef}
        type="file"
        className="hidden"
        id="doc-upload"
        onChange={handleChange}
        disabled={uploading}
      />
      <label
        htmlFor="doc-upload"
        className={`cursor-pointer rounded-lg px-4 py-2 text-sm font-medium ${
          uploading
            ? 'bg-slate-200 text-slate-500'
            : 'bg-emerald-600 text-white hover:bg-emerald-700'
        }`}
      >
        {uploading ? 'Uploading...' : 'Upload Document'}
      </label>
    </div>
  );
}
