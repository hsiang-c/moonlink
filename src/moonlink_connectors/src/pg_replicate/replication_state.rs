use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::watch;
use tokio_postgres::types::PgLsn;

use super::table::SrcTableId;

/// Tracks replication progress and notifies listeners when the replicated
/// LSN advances.
pub struct ReplicationState {
    current: AtomicU64,
    tx: watch::Sender<u64>,
}

impl ReplicationState {
    /// Create a new state initialised to LSN 0.
    pub fn new() -> Arc<Self> {
        let (tx, _rx) = watch::channel(0);
        Arc::new(Self {
            current: AtomicU64::new(0),
            tx,
        })
    }

    /// Mark the replication position as `lsn` if it is newer than the current
    /// value.
    pub fn mark(&self, lsn: PgLsn) {
        let lsn_u64: u64 = lsn.into();
        if lsn_u64 > self.current.load(Ordering::Relaxed) {
            self.current.store(lsn_u64, Ordering::Release);
            let _ = self.tx.send(lsn_u64);
        }
    }

    /// Subscribe for async notifications when the replicated LSN advances.
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.tx.subscribe()
    }

    /// Get the current replicated LSN value.
    pub fn now(&self) -> u64 {
        self.current.load(Ordering::Acquire)
    }
}
