import { Link } from 'react-router-dom';
import { useAuth } from '../../auth/AuthContext';

export function Header() {
  const { user, logout } = useAuth();

  return (
    <header className="border-b border-slate-200 bg-white">
      <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-4">
        <Link to="/dashboard" className="text-lg font-semibold text-slate-800">
          Blockchain Doc Storage
        </Link>
        <nav className="flex items-center gap-6">
          <Link
            to="/documents"
            className="text-sm text-slate-600 hover:text-slate-900"
          >
            Documents
          </Link>
          <Link
            to="/verify"
            className="text-sm text-slate-600 hover:text-slate-900"
          >
            Verify
          </Link>
          <Link
            to="/metrics"
            className="text-sm text-slate-600 hover:text-slate-900"
          >
            Metrics
          </Link>
          <span className="text-sm text-slate-500">
            {user?.email ?? user?.name ?? 'User'}
          </span>
          <button
            onClick={logout}
            className="rounded-md bg-slate-100 px-3 py-1.5 text-sm text-slate-700 hover:bg-slate-200"
          >
            Logout
          </button>
        </nav>
      </div>
    </header>
  );
}
