export interface User {
  id: number;
  email?: string;
  name?: string;
  role: string;
  created_at: string;
}

export interface AuthResponse {
  user: User;
  token: string;
}

export interface DocumentResponse {
  id: number;
  file_name: string;
  document_hash: string;
  file_size: number;
  mime_type: string;
  status: string;
  created_at: string;
  blockchain_status?: string;
}

export interface VerificationResponse {
  exists: boolean;
  document_hash: string;
  owner?: string;
  timestamp?: string;
  metadata?: unknown;
  blockchain_confirmed: boolean;
}

export interface MetricsSnapshot {
  documents_uploaded: number;
  documents_upload_errors: number;
  documents_listed: number;
  documents_fetched: number;
  documents_verified: number;
  transfers_initiated: number;
  auth_logins: number;
  auth_registrations: number;
}
