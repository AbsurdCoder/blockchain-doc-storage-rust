import { useEffect, useState } from 'react';
import { getMetrics } from '../api/metrics';
import type { MetricsSnapshot } from '../types';

export function Metrics() {
  const [data, setData] = useState<MetricsSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getMetrics()
      .then(setData)
      .catch((err) => setError(err instanceof Error ? err.message : 'Failed to load'))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="text-slate-500">Loading metrics...</div>;
  if (error) {
    return (
      <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-red-700">
        {error}
      </div>
    );
  }
  if (!data) return null;

  const rows: { label: string; value: number }[] = [
    { label: 'Documents uploaded', value: data.documents_uploaded },
    { label: 'Upload errors', value: data.documents_upload_errors },
    { label: 'Documents listed', value: data.documents_listed },
    { label: 'Documents fetched', value: data.documents_fetched },
    { label: 'Documents verified', value: data.documents_verified },
    { label: 'Transfers initiated', value: data.transfers_initiated },
    { label: 'Auth logins', value: data.auth_logins },
    { label: 'Auth registrations', value: data.auth_registrations },
  ];

  return (
    <div>
      <h1 className="text-2xl font-semibold text-slate-900">Processing Metrics</h1>
      <p className="mt-2 text-slate-600">
        Application processing statistics.
      </p>
      <div className="mt-8 overflow-hidden rounded-lg border border-slate-200 bg-white shadow">
        <table className="min-w-full divide-y divide-slate-200">
          <thead>
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium uppercase text-slate-500">
                Metric
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium uppercase text-slate-500">
                Count
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-200">
            {rows.map((row) => (
              <tr key={row.label}>
                <td className="px-6 py-4 text-sm text-slate-900">{row.label}</td>
                <td className="px-6 py-4 text-right font-mono text-sm text-slate-700">
                  {row.value}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
