import { api } from './client';
import type {
  DocumentResponse,
  VerificationResponse,
} from '../types';

export async function listDocuments(search?: string, status?: string): Promise<DocumentResponse[]> {
  const params = new URLSearchParams();
  if (search) params.set('search', search);
  if (status) params.set('status', status);
  const q = params.toString();
  return api<DocumentResponse[]>(`/api/documents${q ? `?${q}` : ''}`);
}

export async function getDocument(id: number): Promise<DocumentResponse> {
  return api<DocumentResponse>(`/api/documents/${id}`);
}

export async function uploadDocument(
  file: File
): Promise<DocumentResponse> {
  const base64 = await fileToBase64(file);
  return api<DocumentResponse>('/api/documents', {
    method: 'POST',
    body: JSON.stringify({
      file_name: file.name,
      file_content: base64,
      mime_type: file.type || 'application/octet-stream',
    }),
  });
}

export async function verifyDocument(
  documentHash?: string,
  fileContent?: string
): Promise<VerificationResponse> {
  return api<VerificationResponse>('/api/documents/verify', {
    method: 'POST',
    body: JSON.stringify({
      document_hash: documentHash || null,
      file_content: fileContent || null,
    }),
  });
}

export async function transferOwnership(
  documentId: number,
  newOwnerEmail: string
): Promise<void> {
  await api(`/api/documents/${documentId}/transfer`, {
    method: 'POST',
    body: JSON.stringify({ new_owner_email: newOwnerEmail }),
  });
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      const base64 = result.split(',')[1];
      resolve(base64 || '');
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}
