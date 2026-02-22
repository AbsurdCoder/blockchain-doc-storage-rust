# Blockchain Document Storage – Frontend

React + Vite + TypeScript frontend for the Rust API.

## Setup

```bash
cd frontend
npm install
cp .env.example .env   # optional; proxy is configured for dev
```

## Development

```bash
npm run dev
```

Runs at `http://localhost:5173`. The Vite dev server proxies `/api` and `/health` to the Rust backend at `http://127.0.0.1:8080`.

**Start the Rust backend first** in another terminal:

```bash
cargo run --release
```

## Build

```bash
npm run build
```

Output is in `dist/`. For production, set `VITE_API_URL` to your API base URL (e.g. `https://api.example.com`).

## Routes

| Route | Auth | Description |
|-------|------|-------------|
| `/` | - | Redirects to `/dashboard` |
| `/login` | No | Sign in |
| `/register` | No | Create account |
| `/dashboard` | Yes | Overview |
| `/documents` | Yes | List & upload documents |
| `/documents/:id` | Yes | Document detail & transfer |
| `/verify` | No | Verify by hash or file |
| `/metrics` | No | Processing metrics |
