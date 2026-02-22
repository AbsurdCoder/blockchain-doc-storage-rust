import { api } from './client';
import type { MetricsSnapshot } from '../types';

export async function getMetrics(): Promise<MetricsSnapshot> {
  return api<MetricsSnapshot>('/api/metrics');
}
