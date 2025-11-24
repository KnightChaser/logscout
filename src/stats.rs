// src/stats.rs
use std::sync::atomic::{AtomicU64, Ordering};

/// Statistics for processed log lines
/// total: total lines processed
/// included: lines that passed the regex filters
/// excluded: lines that were regex filtered out
#[derive(Debug)]
pub struct Stats {
    total: AtomicU64,
    included: AtomicU64,
    excluded: AtomicU64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            included: AtomicU64::new(0),
            excluded: AtomicU64::new(0),
        }
    }

    pub fn inc_total(&self) {
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_included(&self) {
        self.included.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_excluded(&self) {
        self.excluded.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.total.load(Ordering::Relaxed),
            self.included.load(Ordering::Relaxed),
            self.excluded.load(Ordering::Relaxed),
        )
    }
}
