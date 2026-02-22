use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe processing metrics, updated by handlers.
#[derive(Debug, Default)]
pub struct Metrics {
    pub documents_uploaded: AtomicU64,
    pub documents_upload_errors: AtomicU64,
    pub documents_listed: AtomicU64,
    pub documents_fetched: AtomicU64,
    pub documents_verified: AtomicU64,
    pub transfers_initiated: AtomicU64,
    pub auth_logins: AtomicU64,
    pub auth_registrations: AtomicU64,
}

impl Metrics {
    pub fn record_document_uploaded(&self) {
        self.documents_uploaded.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_document_upload_error(&self) {
        self.documents_upload_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_documents_listed(&self) {
        self.documents_listed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_document_fetched(&self) {
        self.documents_fetched.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_document_verified(&self) {
        self.documents_verified.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_transfer_initiated(&self) {
        self.transfers_initiated.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_auth_login(&self) {
        self.auth_logins.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_auth_registration(&self) {
        self.auth_registrations.fetch_add(1, Ordering::Relaxed);
    }

    /// Snapshot of current metric values for JSON serialization.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            documents_uploaded: self.documents_uploaded.load(Ordering::Relaxed),
            documents_upload_errors: self.documents_upload_errors.load(Ordering::Relaxed),
            documents_listed: self.documents_listed.load(Ordering::Relaxed),
            documents_fetched: self.documents_fetched.load(Ordering::Relaxed),
            documents_verified: self.documents_verified.load(Ordering::Relaxed),
            transfers_initiated: self.transfers_initiated.load(Ordering::Relaxed),
            auth_logins: self.auth_logins.load(Ordering::Relaxed),
            auth_registrations: self.auth_registrations.load(Ordering::Relaxed),
        }
    }
}

/// Serializable snapshot of metrics for API response.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub documents_uploaded: u64,
    pub documents_upload_errors: u64,
    pub documents_listed: u64,
    pub documents_fetched: u64,
    pub documents_verified: u64,
    pub transfers_initiated: u64,
    pub auth_logins: u64,
    pub auth_registrations: u64,
}
