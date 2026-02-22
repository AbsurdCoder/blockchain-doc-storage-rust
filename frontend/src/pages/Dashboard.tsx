import { Link } from 'react-router-dom';
import { useAuth } from '../auth/AuthContext';

export function Dashboard() {
  const { user } = useAuth();

  return (
    <div>
      <h1 className="text-2xl font-semibold text-slate-900">
        Welcome{user?.name ? `, ${user.name}` : ''}
      </h1>
      <p className="mt-2 text-slate-600">
        Blockchain Document Storage – upload, verify, and manage documents.
      </p>
      <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <Link
          to="/documents"
          className="rounded-lg border border-slate-200 bg-white p-6 shadow-sm transition hover:border-emerald-300 hover:shadow-md"
        >
          <h2 className="font-medium text-slate-900">Upload Document</h2>
          <p className="mt-2 text-sm text-slate-600">
            Upload and store documents with blockchain verification.
          </p>
        </Link>
        <Link
          to="/verify"
          className="rounded-lg border border-slate-200 bg-white p-6 shadow-sm transition hover:border-slate-300 hover:shadow-md"
        >
          <h2 className="font-medium text-slate-900">Verify Document</h2>
          <p className="mt-2 text-sm text-slate-600">
            Verify document integrity by hash or file upload.
          </p>
        </Link>
        <Link
          to="/documents"
          className="rounded-lg border border-slate-200 bg-white p-6 shadow-sm transition hover:border-slate-300 hover:shadow-md"
        >
          <h2 className="font-medium text-slate-900">View Documents</h2>
          <p className="mt-2 text-sm text-slate-600">
            List and manage your stored documents.
          </p>
        </Link>
      </div>
    </div>
  );
}
