use super::*;

#[derive(Clone, Copy)]
struct Admission {
    slot_index: usize,
    next_even_sequence: u64,
}

#[derive(Clone, Copy)]
enum Operation {
    Alloc,
    AllocZeroed,
    Realloc,
}

#[inline(always)]
fn admit_nonrecursive(thread: &AllocationThreadState, observed: u64) -> Option<Admission> {
    let mut slot_index = thread.slot_index.get();
    if slot_index == UNCLAIMED_SLOT {
        let Some(claimed) = claim_slot_slow() else {
            let _ = record_preadmission_fault(observed, FAULT_SLOT_EXHAUSTED);
            return None;
        };
        thread.slot_index.set(claimed);
        slot_index = claimed;
    }
    let slot_index = slot_index as usize;
    let Some(slot) = SLOTS.get(slot_index) else {
        let _ = record_preadmission_fault(observed, FAULT_PROTOCOL);
        return None;
    };

    #[cfg(test)]
    pause_before_admission();

    let sequence = slot.sequence.load(Ordering::Relaxed);
    let Some(next_even_sequence) = sequence.checked_add(2) else {
        let _ = record_preadmission_fault(observed, FAULT_SEQUENCE_OVERFLOW);
        return None;
    };
    if sequence & 1 != 0 {
        let _ = record_preadmission_fault(observed, FAULT_PROTOCOL);
        return None;
    }
    let previous = slot.sequence.fetch_add(1, Ordering::SeqCst);
    if previous != sequence {
        record_admitted_fault(FAULT_PROTOCOL);
        slot.sequence.store(next_even_sequence, Ordering::Release);
        return None;
    }
    #[cfg(test)]
    pause_after_odd();
    let confirmed = STATE.control.load(Ordering::SeqCst);
    if confirmed != observed {
        slot.sequence.store(next_even_sequence, Ordering::Release);
        return None;
    }
    Some(Admission {
        slot_index,
        next_even_sequence,
    })
}

#[inline(always)]
fn complete_admission(admission: Admission) {
    if let Some(slot) = SLOTS.get(admission.slot_index) {
        slot.sequence
            .store(admission.next_even_sequence, Ordering::Release);
    } else {
        record_admitted_fault(FAULT_PROTOCOL);
    }
}

#[inline(always)]
fn observe_foreign_success(
    slot: &AllocationSlot,
    operation: Operation,
    bytes: usize,
    result: *mut u8,
) {
    if result.is_null() {
        return;
    }
    let bytes = bytes as u64;
    let (count, byte_count) = match operation {
        Operation::Alloc => (&slot.alloc_count, &slot.alloc_bytes),
        Operation::AllocZeroed => (&slot.alloc_zeroed_count, &slot.alloc_zeroed_bytes),
        Operation::Realloc => (&slot.realloc_count, &slot.realloc_bytes),
    };
    if !checked_increment_atomic(count, 1) || !checked_increment_atomic(byte_count, bytes) {
        record_admitted_fault(FAULT_COUNTER_OVERFLOW);
    }
}

#[inline(always)]
fn observe_owner_success(
    thread: &AllocationThreadState,
    operation: Operation,
    bytes: usize,
    result: *mut u8,
    observed: u64,
) {
    if result.is_null() {
        return;
    }
    let bytes = bytes as u64;
    let (count, byte_count) = match operation {
        Operation::Alloc => (&thread.owner_alloc_count, &thread.owner_alloc_bytes),
        Operation::AllocZeroed => (
            &thread.owner_alloc_zeroed_count,
            &thread.owner_alloc_zeroed_bytes,
        ),
        Operation::Realloc => (&thread.owner_realloc_count, &thread.owner_realloc_bytes),
    };
    if !checked_increment_cell(count, 1) || !checked_increment_cell(byte_count, bytes) {
        let _ = record_preadmission_fault(observed, FAULT_COUNTER_OVERFLOW);
    }
}

#[inline(always)]
fn checked_increment_atomic(counter: &AtomicU64, increment: u64) -> bool {
    let current = counter.load(Ordering::Relaxed);
    let Some(next) = current.checked_add(increment) else {
        return false;
    };
    counter.store(next, Ordering::Relaxed);
    true
}

#[inline(always)]
fn checked_increment_cell(counter: &Cell<u64>, increment: u64) -> bool {
    let current = counter.get();
    let Some(next) = current.checked_add(increment) else {
        return false;
    };
    counter.set(next);
    true
}

#[inline(always)]
pub(super) fn validate_non_zero(size: usize) {
    if size == 0 {
        record_admitted_fault(FAULT_INVALID_CALL);
    }
}

#[inline(always)]
fn validate_owner_non_zero(size: usize, observed: u64) {
    if size == 0 {
        let _ = record_preadmission_fault(observed, FAULT_INVALID_CALL);
    }
}

#[allow(
    unsafe_code,
    reason = "ADR-041 permits the owner and foreign active GlobalAlloc helpers"
)]
impl ProcessAllocationCounter {
    #[inline(never)]
    pub(super) unsafe fn alloc_owner_active(&self, layout: Layout, observed: u64) -> *mut u8 {
        #[cfg(test)]
        if force_tls_failure(observed) {
            // SAFETY: the same caller-owned contract is forwarded once.
            return unsafe { self.system.alloc(layout) };
        }
        let result = THREAD_STATE.try_with(|thread| {
            if thread.owner_window_id.get() != control_window(observed) {
                // SAFETY: the foreign helper preserves this allocation contract.
                return unsafe { self.alloc_foreign_active(thread, layout, observed) };
            }
            validate_owner_non_zero(layout.size(), observed);
            // SAFETY: the caller supplied the matching valid allocation contract.
            let result = unsafe { self.system.alloc(layout) };
            observe_owner_success(thread, Operation::Alloc, layout.size(), result, observed);
            result
        });
        match result {
            Ok(result) => result,
            Err(_) => {
                let _ = record_preadmission_fault(observed, FAULT_TLS_UNAVAILABLE);
                // SAFETY: the same caller-owned contract is forwarded once.
                unsafe { self.system.alloc(layout) }
            }
        }
    }

    #[inline(never)]
    unsafe fn alloc_foreign_active(
        &self,
        thread: &AllocationThreadState,
        layout: Layout,
        observed: u64,
    ) -> *mut u8 {
        let Some(admission) = admit_nonrecursive(thread, observed) else {
            // SAFETY: the same caller-owned contract is forwarded once.
            return unsafe { self.system.alloc(layout) };
        };
        validate_non_zero(layout.size());
        #[cfg(test)]
        pause_after_admission();
        // SAFETY: the caller supplied the matching valid allocation contract.
        let result = unsafe { self.system.alloc(layout) };
        if let Some(slot) = SLOTS.get(admission.slot_index) {
            observe_foreign_success(slot, Operation::Alloc, layout.size(), result);
        } else {
            record_admitted_fault(FAULT_PROTOCOL);
        }
        complete_admission(admission);
        result
    }

    #[inline(never)]
    pub(super) unsafe fn alloc_zeroed_owner_active(
        &self,
        layout: Layout,
        observed: u64,
    ) -> *mut u8 {
        #[cfg(test)]
        if force_tls_failure(observed) {
            // SAFETY: the same caller-owned contract is forwarded once.
            return unsafe { self.system.alloc_zeroed(layout) };
        }
        let result = THREAD_STATE.try_with(|thread| {
            if thread.owner_window_id.get() != control_window(observed) {
                // SAFETY: the foreign helper preserves this allocation contract.
                return unsafe { self.alloc_zeroed_foreign_active(thread, layout, observed) };
            }
            validate_owner_non_zero(layout.size(), observed);
            // SAFETY: the caller supplied the matching zeroed-allocation contract.
            let result = unsafe { self.system.alloc_zeroed(layout) };
            observe_owner_success(
                thread,
                Operation::AllocZeroed,
                layout.size(),
                result,
                observed,
            );
            result
        });
        match result {
            Ok(result) => result,
            Err(_) => {
                let _ = record_preadmission_fault(observed, FAULT_TLS_UNAVAILABLE);
                // SAFETY: the same caller-owned contract is forwarded once.
                unsafe { self.system.alloc_zeroed(layout) }
            }
        }
    }

    #[inline(never)]
    unsafe fn alloc_zeroed_foreign_active(
        &self,
        thread: &AllocationThreadState,
        layout: Layout,
        observed: u64,
    ) -> *mut u8 {
        let Some(admission) = admit_nonrecursive(thread, observed) else {
            // SAFETY: the same caller-owned contract is forwarded once.
            return unsafe { self.system.alloc_zeroed(layout) };
        };
        validate_non_zero(layout.size());
        // SAFETY: the caller supplied the matching zeroed-allocation contract.
        let result = unsafe { self.system.alloc_zeroed(layout) };
        if let Some(slot) = SLOTS.get(admission.slot_index) {
            observe_foreign_success(slot, Operation::AllocZeroed, layout.size(), result);
        } else {
            record_admitted_fault(FAULT_PROTOCOL);
        }
        complete_admission(admission);
        result
    }

    #[inline(never)]
    pub(super) unsafe fn realloc_owner_active(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
        observed: u64,
    ) -> *mut u8 {
        #[cfg(test)]
        if force_tls_failure(observed) {
            // SAFETY: the same caller-owned contract is forwarded once.
            return unsafe { self.system.realloc(ptr, layout, new_size) };
        }
        let result = THREAD_STATE.try_with(|thread| {
            if thread.owner_window_id.get() != control_window(observed) {
                // SAFETY: the foreign helper preserves this reallocation contract.
                return unsafe {
                    self.realloc_foreign_active(thread, ptr, layout, new_size, observed)
                };
            }
            validate_owner_non_zero(layout.size(), observed);
            validate_owner_non_zero(new_size, observed);
            // SAFETY: the caller-owned pointer/layout/new-size contract is forwarded.
            let result = unsafe { self.system.realloc(ptr, layout, new_size) };
            observe_owner_success(thread, Operation::Realloc, new_size, result, observed);
            result
        });
        match result {
            Ok(result) => result,
            Err(_) => {
                let _ = record_preadmission_fault(observed, FAULT_TLS_UNAVAILABLE);
                // SAFETY: the same caller-owned contract is forwarded once.
                unsafe { self.system.realloc(ptr, layout, new_size) }
            }
        }
    }

    #[inline(never)]
    unsafe fn realloc_foreign_active(
        &self,
        thread: &AllocationThreadState,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
        observed: u64,
    ) -> *mut u8 {
        let Some(admission) = admit_nonrecursive(thread, observed) else {
            // SAFETY: the same caller-owned contract is forwarded once.
            return unsafe { self.system.realloc(ptr, layout, new_size) };
        };
        validate_non_zero(layout.size());
        validate_non_zero(new_size);
        // SAFETY: the caller-owned pointer/layout/new-size contract is forwarded.
        let result = unsafe { self.system.realloc(ptr, layout, new_size) };
        if let Some(slot) = SLOTS.get(admission.slot_index) {
            observe_foreign_success(slot, Operation::Realloc, new_size, result);
        } else {
            record_admitted_fault(FAULT_PROTOCOL);
        }
        complete_admission(admission);
        result
    }
}
