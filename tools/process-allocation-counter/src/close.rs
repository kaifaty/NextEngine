use super::*;

pub(super) fn finish_window(
    window_id: u64,
    producer_pid: u32,
) -> Result<AllocationSnapshot, MeasurementError> {
    let active = encode_control(PHASE_ACTIVE, window_id);
    let current_pid = std::process::id();
    if current_pid != producer_pid || STATE.owner_pid.load(Ordering::Acquire) != current_pid {
        let _ = poison_exact(active, FAULT_PROCESS_MISMATCH);
        let _ = clear_owner_window(window_id);
        return Err(MeasurementError::ProcessMismatch);
    }
    if let Err(error) = validate_owner_thread(window_id) {
        if !poison_exact(active, fault_for_error(error)) {
            let observed = STATE.control.load(Ordering::SeqCst);
            if phase(observed) == PHASE_ACTIVE {
                let _ = poison_exact(observed, FAULT_STALE_WINDOW);
            }
        }
        let _ = clear_owner_window(window_id);
        return Err(error);
    }
    let closing = encode_control(PHASE_CLOSING, window_id);
    match STATE
        .control
        .compare_exchange(active, closing, Ordering::SeqCst, Ordering::SeqCst)
    {
        Ok(_) => {}
        Err(observed) if phase(observed) == PHASE_POISONED => {
            let _ = clear_owner_window(window_id);
            return Err(fault_error(observed));
        }
        Err(observed) => {
            if phase(observed) == PHASE_ACTIVE {
                let _ = poison_exact(observed, FAULT_STALE_WINDOW);
            }
            let _ = clear_owner_window(window_id);
            return Err(MeasurementError::StaleWindow);
        }
    }

    let owner_totals = match take_owner_snapshot(window_id) {
        Ok(totals) => totals,
        Err(error) => {
            let _ = poison_exact(closing, fault_for_error(error));
            let _ = clear_owner_window(window_id);
            return Err(error);
        }
    };
    let foreign_totals = match snapshot_all_slots() {
        Ok(totals) => totals,
        Err(error) => {
            let _ = poison_exact(closing, fault_for_error(error));
            return Err(error);
        }
    };
    let Some(totals) = owner_totals.checked_add(foreign_totals) else {
        let _ = poison_exact(closing, FAULT_COUNTER_OVERFLOW);
        return Err(MeasurementError::CounterOverflow);
    };
    let admitted_faults = STATE.faults.load(Ordering::Acquire) & FAULT_MASK;
    if admitted_faults != 0 {
        let _ = poison_exact(closing, admitted_faults);
        return Err(fault_error(STATE.control.load(Ordering::SeqCst)));
    }
    if STATE.owner_pid.load(Ordering::Acquire) != current_pid {
        let _ = poison_exact(closing, FAULT_PROCESS_MISMATCH);
        return Err(MeasurementError::ProcessMismatch);
    }

    let Some(allocator_allocation_count) = totals
        .alloc_count
        .checked_add(totals.alloc_zeroed_count)
        .and_then(|sum| sum.checked_add(totals.realloc_count))
    else {
        let _ = poison_exact(closing, FAULT_COUNTER_OVERFLOW);
        return Err(MeasurementError::CounterOverflow);
    };
    let Some(allocator_allocated_bytes) = totals
        .alloc_bytes
        .checked_add(totals.alloc_zeroed_bytes)
        .and_then(|sum| sum.checked_add(totals.realloc_bytes))
    else {
        let _ = poison_exact(closing, FAULT_COUNTER_OVERFLOW);
        return Err(MeasurementError::CounterOverflow);
    };

    if STATE
        .control
        .compare_exchange(
            closing,
            PHASE_IDLE | HOOK_SEEN_BIT,
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        .is_err()
    {
        return Err(MeasurementError::Poisoned);
    }
    Ok(AllocationSnapshot {
        producer_pid,
        window_id,
        alloc_count: totals.alloc_count,
        alloc_bytes: totals.alloc_bytes,
        alloc_zeroed_count: totals.alloc_zeroed_count,
        alloc_zeroed_bytes: totals.alloc_zeroed_bytes,
        realloc_count: totals.realloc_count,
        realloc_bytes: totals.realloc_bytes,
        allocator_allocation_count,
        allocator_allocated_bytes,
    })
}

fn snapshot_all_slots() -> Result<CounterSnapshot, MeasurementError> {
    let mut remaining = CLOSE_HANDSHAKE_LIMIT;
    let mut totals = CounterSnapshot::zero();
    for slot in &SLOTS {
        let snapshot = loop {
            if remaining < 2 {
                return Err(MeasurementError::CloseTimeout);
            }
            remaining -= 1;
            let before = slot.sequence.fetch_add(0, Ordering::SeqCst);
            if before & 1 != 0 {
                wait_for_slot(remaining);
                continue;
            }
            let candidate = CounterSnapshot {
                alloc_count: slot.alloc_count.load(Ordering::Relaxed),
                alloc_bytes: slot.alloc_bytes.load(Ordering::Relaxed),
                alloc_zeroed_count: slot.alloc_zeroed_count.load(Ordering::Relaxed),
                alloc_zeroed_bytes: slot.alloc_zeroed_bytes.load(Ordering::Relaxed),
                realloc_count: slot.realloc_count.load(Ordering::Relaxed),
                realloc_bytes: slot.realloc_bytes.load(Ordering::Relaxed),
            };
            remaining -= 1;
            let after = slot.sequence.fetch_add(0, Ordering::SeqCst);
            if before == after && after & 1 == 0 {
                break candidate;
            }
            wait_for_slot(remaining);
        };
        totals = totals
            .checked_add(snapshot)
            .ok_or(MeasurementError::CounterOverflow)?;
    }
    Ok(totals)
}

fn wait_for_slot(remaining: usize) {
    if remaining & 0x3f == 0 {
        std::thread::yield_now();
    } else {
        std::hint::spin_loop();
    }
}
