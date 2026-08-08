use std::sync::{Mutex, MutexGuard};

static NUMERIC_PERFORMANCE_MEASUREMENT: Mutex<()> = Mutex::new(());

pub(crate) fn lock_numeric_performance_measurement() -> MutexGuard<'static, ()> {
    NUMERIC_PERFORMANCE_MEASUREMENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
