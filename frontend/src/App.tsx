import { BrowserRouter, Link, Navigate, Route, Routes } from 'react-router-dom';
import { AuthProvider } from './auth/AuthContext';
import { ProtectedRoute } from './auth/ProtectedRoute';
import { Layout } from './components/Layout/Layout';
import { Dashboard } from './pages/Dashboard';
import { DocumentDetail } from './pages/DocumentDetail';
import { Documents } from './pages/Documents';
import { Login } from './pages/Login';
import { Metrics } from './pages/Metrics';
import { Register } from './pages/Register';
import { Verify } from './pages/Verify';

function PublicLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-slate-50">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex h-12 max-w-6xl items-center justify-between px-4">
          <Link to="/" className="text-sm font-semibold text-slate-800">
            Blockchain Doc Storage
          </Link>
          <nav className="flex gap-4">
            <Link to="/verify" className="text-sm text-slate-600 hover:text-slate-900">
              Verify
            </Link>
            <Link to="/metrics" className="text-sm text-slate-600 hover:text-slate-900">
              Metrics
            </Link>
            <Link to="/login" className="text-sm text-slate-600 hover:text-slate-900">
              Sign in
            </Link>
          </nav>
        </div>
      </header>
      <main className="mx-auto max-w-6xl px-4 py-8">{children}</main>
    </div>
  );
}

function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/login" element={<Login />} />
          <Route path="/register" element={<Register />} />
          <Route
            path="/verify"
            element={
              <PublicLayout>
                <Verify />
              </PublicLayout>
            }
          />
          <Route
            path="/metrics"
            element={
              <PublicLayout>
                <Metrics />
              </PublicLayout>
            }
          />
          <Route
            path="/dashboard"
            element={
              <ProtectedRoute>
                <Layout>
                  <Dashboard />
                </Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/documents"
            element={
              <ProtectedRoute>
                <Layout>
                  <Documents />
                </Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/documents/:id"
            element={
              <ProtectedRoute>
                <Layout>
                  <DocumentDetail />
                </Layout>
              </ProtectedRoute>
            }
          />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  );
}

export default App;
