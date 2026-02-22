import { useState, useCallback } from 'react';
import { DocumentUpload } from '../components/DocumentUpload';
import { DocumentList } from '../components/DocumentList';
import { TransferOwnershipModal } from '../components/TransferOwnershipModal';
import type { DocumentResponse } from '../types';

export function Documents() {
  const [refresh, setRefresh] = useState(0);
  const [transferDoc, setTransferDoc] = useState<DocumentResponse | null>(null);

  const handleUploadSuccess = useCallback(() => {
    setRefresh((r) => r + 1);
  }, []);

  const handleTransferSuccess = useCallback(() => {
    setTransferDoc(null);
    setRefresh((r) => r + 1);
  }, []);

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-slate-900">Documents</h1>
        <DocumentUpload
          onSuccess={handleUploadSuccess}
          onError={(err) => alert(err)}
        />
      </div>
      <DocumentList
        refreshTrigger={refresh}
        onTransfer={setTransferDoc}
      />
      <TransferOwnershipModal
        doc={transferDoc}
        onClose={() => setTransferDoc(null)}
        onSuccess={handleTransferSuccess}
      />
    </div>
  );
}
