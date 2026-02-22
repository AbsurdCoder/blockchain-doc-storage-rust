# Frontend Design – Blockchain Document Verification Platform

Design for a web frontend that interacts with the Rust API at `http://127.0.0.1:8080`.

---

## 1. Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Framework | React 18 + TypeScript | Component-based, strong typing, large ecosystem |
| Build | Vite | Fast dev server, simple config |
| Routing | React Router v6 | Standard SPA routing |
| HTTP | Fetch / axios | Simple REST client; add interceptors for JWT |
| State | React Context + useState/useReducer | Sufficient for auth + document list; add Zustand if needed |
| Styling | Tailwind CSS | Utility-first, quick iteration |
| File handling | FileReader API + base64 | API expects base64 `file_content` |

### Alternatives
- **Vue 3 + Vite** – simpler mental model
- **SvelteKit** – smaller bundle, less boilerplate
- **Vanilla JS + Vite** – minimal dependencies

---

## 2. Project Structure

```
frontend/
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
├── .env.example          # VITE_API_URL=http://127.0.0.1:8080
├── public/
│   └── favicon.ico
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── api/
    │   ├── client.ts     # Base fetch with base URL, JWT injection
    │   ├── auth.ts       # register, login, logout, me
    │   ├── documents.ts  # upload, list, get, verify, transfer
    │   └── metrics.ts    # getMetrics
    ├── auth/
    │   ├── AuthContext.tsx
    │   └── ProtectedRoute.tsx
    ├── components/
    │   ├── Layout/
    │   │   ├── Header.tsx
    │   │   └── Sidebar.tsx
    │   ├── DocumentUpload.tsx
    │   ├── DocumentList.tsx
    │   ├── DocumentCard.tsx
    │   ├── DocumentVerifier.tsx
    │   └── TransferOwnershipModal.tsx
    ├── pages/
    │   ├── Login.tsx
    │   ├── Register.tsx
    │   ├── Dashboard.tsx
    │   ├── Documents.tsx
    │   ├── Verify.tsx
    │   └── Metrics.tsx
    ├── hooks/
    │   └── useApi.ts
    └── types/
        └── index.ts
```

---

## 3. Routes & Pages

| Route | Page | Auth | Purpose |
|-------|------|------|---------|
| `/` | Redirect to `/login` or `/dashboard` | - | Entry |
| `/login` | Login | No | Email + password |
| `/register` | Register | No | Email, password, name |
| `/dashboard` | Dashboard | Yes | Overview, quick actions |
| `/documents` | Documents | Yes | List, upload, manage |
| `/documents/:id` | Document Detail | Yes | View, transfer |
| `/verify` | Verify Document | No | Hash or file verification |
| `/metrics` | Metrics | No | Processing stats (internal/admin) |

---

## 4. Screen Designs

### 4.1 Login
- **Layout**: Centered card, logo, title "Blockchain Document Storage"
- **Fields**: Email (text), Password (password)
- **Actions**: Login button, "Register" link
- **Validation**: Required, basic email format
- **Flow**: On success → store JWT in memory/localStorage → redirect to `/dashboard`

### 4.2 Register
- **Fields**: Email, Password, Name (optional)
- **Actions**: Register button, "Already have an account? Login" link
- **Flow**: On success → same as Login

### 4.3 Dashboard
- **Header**: Logo, user email, Logout
- **Content**: 
  - Welcome message
  - Quick actions: "Upload Document", "Verify Document", "View Documents"
  - Optional: summary cards (e.g. document count, last upload)

### 4.4 Documents
- **Header**: Same as Dashboard
- **Actions**: Upload button (opens modal or inline form)
- **List**: Table or cards with: filename, hash (truncated), status, created date, actions (View, Transfer)
- **Upload form**: File picker → read as base64 → `POST /api/documents`
- **Empty state**: "No documents yet. Upload your first document."

### 4.5 Document Detail
- **Content**: Full metadata (filename, hash, mime type, status, created)
- **Actions**: Transfer ownership (opens modal with email input)

### 4.6 Verify
- **Option A**: Paste document hash
- **Option B**: Upload file → compute hash client-side or send base64
- **Result**: Shows `exists`, `blockchain_confirmed`, `owner` (if any)
- **Auth**: Not required (public verification)

### 4.7 Metrics
- **Content**: Table or cards for each metric (documents_uploaded, documents_listed, etc.)
- **Refresh**: Manual or auto-refresh every 30s

---

## 5. API Client Design

### Base client
```typescript
// api/client.ts
const API_URL = import.meta.env.VITE_API_URL || 'http://127.0.0.1:8080';

function getToken(): string | null {
  return localStorage.getItem('token');
}

export async function api<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${API_URL}${path}`, { ...options, headers });
  if (!res.ok) throw new Error(await res.text() || res.statusText);
  return res.json().catch(() => ({}));
}
```

### Document upload (base64)
```typescript
// Convert File to base64
const reader = new FileReader();
reader.readAsDataURL(file);
reader.onload = () => {
  const base64 = (reader.result as string).split(',')[1];
  api('/api/documents', {
    method: 'POST',
    body: JSON.stringify({
      file_name: file.name,
      file_content: base64,
      mime_type: file.type || 'application/octet-stream',
    }),
  });
};
```

---

## 6. Auth Flow

```
┌──────────┐     POST /api/auth/login      ┌──────────┐
│  Login   │ ────────────────────────────► │   API    │
│  Page    │ ◄──────────────────────────── │          │
└──────────┘     { user, token }           └──────────┘
       │
       │ store token (localStorage / memory)
       ▼
┌──────────┐     GET /api/auth/me          ┌──────────┐
│ Protected│     Authorization: Bearer     │   API    │
│  Route   │ ────────────────────────────► │          │
└──────────┘     { user }                  └──────────┘
       │
       │ 401 → clear token, redirect to /login
       ▼
   Render page
```

- **Token storage**: `localStorage` for persistence across refreshes
- **AuthContext**: Exposes `{ user, token, login, logout, loading }`
- **ProtectedRoute**: If no token → redirect to `/login`

---

## 7. Key User Flows

### Upload flow
1. User clicks "Upload" → file picker
2. Select file → show filename, size, progress
3. Read file as base64 (FileReader)
4. `POST /api/documents` with `{ file_name, file_content, mime_type }`
5. On success → add to list, show success toast
6. On error (400/401) → show error message

### Verify flow
1. User goes to `/verify`
2. Paste hash **or** drop/select file
3. If file: compute SHA-256 client-side (e.g. Web Crypto API) or send base64 to API
4. `POST /api/documents/verify` with `{ document_hash }` or `{ file_content }`
5. Display result (exists, blockchain_confirmed, owner)

### Transfer flow
1. From document list or detail, click "Transfer"
2. Modal with email input
3. `POST /api/documents/{id}/transfer` with `{ new_owner_email }`
4. On success → close modal, show toast

---

## 8. Component Hierarchy

```
App
├── AuthProvider
│   └── Router
│       ├── Login
│       ├── Register
│       ├── ProtectedRoute
│       │   ├── Layout (Header + outlet)
│       │   │   ├── Dashboard
│       │   │   ├── Documents
│       │   │   │   ├── DocumentUpload
│       │   │   │   └── DocumentList
│       │   │   │       └── DocumentCard[]
│       │   │   └── DocumentDetail
│       │   │       └── TransferOwnershipModal
│       │   └── ...
│       ├── Verify
│       └── Metrics
```

---

## 9. Environment

```env
# .env
VITE_API_URL=http://127.0.0.1:8080
```

Vite exposes env vars prefixed with `VITE_` to the client.

---

## 10. CORS

The Rust app uses `Cors::permissive()`. For production, restrict:

```rust
Cors::default()
  .allowed_origin("https://your-frontend.com")
  .allowed_methods(vec!["GET", "POST"])
  .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
```

---

## 11. Implementation Order

1. **Scaffold** – Vite + React + TypeScript + Tailwind
2. **API client** – Base `api()` + auth helpers
3. **Auth** – Login, Register, AuthContext, ProtectedRoute
4. **Dashboard** – Simple layout, logout
5. **Documents** – List + Upload
6. **Verify** – Hash/file verification
7. **Document detail** – View + Transfer
8. **Metrics** – Admin page
9. **Polish** – Loading states, error toasts, responsive layout

---

## 12. TypeScript Types (align with API)

```typescript
// types/index.ts
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
```
