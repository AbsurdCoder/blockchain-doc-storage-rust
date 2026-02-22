import { DocumentVerifier } from '../components/DocumentVerifier';

export function Verify() {
  return (
    <div>
      <h1 className="text-2xl font-semibold text-slate-900">Verify Document</h1>
      <p className="mt-2 text-slate-600">
        Verify document integrity by pasting a hash or uploading a file.
      </p>
      <div className="mt-8">
        <DocumentVerifier />
      </div>
    </div>
  );
}
