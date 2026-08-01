//! Tooling-only process-wide allocation traffic measurement.
//!
//! The wrapper always delegates to [`std::alloc::System`]. Measurement is an
//! explicit process-wide window, and the safe API fails closed instead of
//! publishing partial counters after any protocol fault.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::mem::{align_of, needs_drop, size_of};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

#[cfg(test)]
use std::sync::atomic::AtomicBool;

mod active;
mod close;

const PHASE_SHIFT: u32 = 61;
const PHASE_MASK: u64 = 0b111 << PHASE_SHIFT;
const PHASE_IDLE: u64 = 0;
const PHASE_STARTING: u64 = 0b001 << PHASE_SHIFT;
const PHASE_ACTIVE: u64 = 0b010 << PHASE_SHIFT;
const PHASE_CLOSING: u64 = 0b011 << PHASE_SHIFT;
const PHASE_POISONED: u64 = 0b100 << PHASE_SHIFT;

const HOOK_SEEN_BIT: u64 = 1 << 60;
const WINDOW_SHIFT: u32 = 32;
const WINDOW_VALUE_MASK: u64 = (1 << (60 - WINDOW_SHIFT)) - 1;
const WINDOW_MASK: u64 = WINDOW_VALUE_MASK << WINDOW_SHIFT;
const FAULT_MASK: u64 = u32::MAX as u64;

const FAULT_NESTED_WINDOW: u64 = 1 << 0;
const FAULT_ABANDONED_WINDOW: u64 = 1 << 1;
const FAULT_PROCESS_MISMATCH: u64 = 1 << 2;
const FAULT_STALE_WINDOW: u64 = 1 << 3;
const FAULT_COUNTER_OVERFLOW: u64 = 1 << 4;
const FAULT_WINDOW_ID_OVERFLOW: u64 = 1 << 5;
const FAULT_SLOT_EXHAUSTED: u64 = 1 << 6;
const FAULT_CLOSE_TIMEOUT: u64 = 1 << 7;
const FAULT_INVALID_CALL: u64 = 1 << 8;
const FAULT_HOOK_UNAVAILABLE: u64 = 1 << 9;
const FAULT_PROTOCOL: u64 = 1 << 10;
const FAULT_TLS_UNAVAILABLE: u64 = 1 << 11;
const FAULT_SEQUENCE_OVERFLOW: u64 = 1 << 12;
const FAULT_RECURSION: u64 = 1 << 13;

const SLOT_COUNT: usize = 4_096;
const UNCLAIMED_SLOT: u32 = u32::MAX;
#[cfg(not(test))]
const CLOSE_HANDSHAKE_LIMIT: usize = 10_000_000;
#[cfg(test)]
const CLOSE_HANDSHAKE_LIMIT: usize = 50_000;

struct CounterState {
    control: AtomicU64,
    next_window_id: AtomicU64,
    owner_pid: AtomicU32,
    faults: AtomicU64,
}

impl CounterState {
    const fn new() -> Self {
        Self {
            control: AtomicU64::new(PHASE_IDLE),
            next_window_id: AtomicU64::new(0),
            owner_pid: AtomicU32::new(0),
            faults: AtomicU64::new(0),
        }
    }
}

#[repr(align(128))]
struct AllocationSlot {
    sequence: AtomicU64,
    alloc_count: AtomicU64,
    alloc_bytes: AtomicU64,
    alloc_zeroed_count: AtomicU64,
    alloc_zeroed_bytes: AtomicU64,
    realloc_count: AtomicU64,
    realloc_bytes: AtomicU64,
}

impl AllocationSlot {
    const fn new() -> Self {
        Self {
            sequence: AtomicU64::new(0),
            alloc_count: AtomicU64::new(0),
            alloc_bytes: AtomicU64::new(0),
            alloc_zeroed_count: AtomicU64::new(0),
            alloc_zeroed_bytes: AtomicU64::new(0),
            realloc_count: AtomicU64::new(0),
            realloc_bytes: AtomicU64::new(0),
        }
    }

    fn reset_counters(&self) {
        self.alloc_count.store(0, Ordering::Relaxed);
        self.alloc_bytes.store(0, Ordering::Relaxed);
        self.alloc_zeroed_count.store(0, Ordering::Relaxed);
        self.alloc_zeroed_bytes.store(0, Ordering::Relaxed);
        self.realloc_count.store(0, Ordering::Relaxed);
        self.realloc_bytes.store(0, Ordering::Relaxed);
    }
}

struct AllocationThreadState {
    slot_index: Cell<u32>,
    in_callback: Cell<bool>,
    owner_window_id: Cell<u64>,
    owner_alloc_count: Cell<u64>,
    owner_alloc_bytes: Cell<u64>,
    owner_alloc_zeroed_count: Cell<u64>,
    owner_alloc_zeroed_bytes: Cell<u64>,
    owner_realloc_count: Cell<u64>,
    owner_realloc_bytes: Cell<u64>,
}

impl AllocationThreadState {
    const fn new() -> Self {
        Self {
            slot_index: Cell::new(UNCLAIMED_SLOT),
            in_callback: Cell::new(false),
            owner_window_id: Cell::new(0),
            owner_alloc_count: Cell::new(0),
            owner_alloc_bytes: Cell::new(0),
            owner_alloc_zeroed_count: Cell::new(0),
            owner_alloc_zeroed_bytes: Cell::new(0),
            owner_realloc_count: Cell::new(0),
            owner_realloc_bytes: Cell::new(0),
        }
    }

    fn reset_owner_counters(&self) {
        self.owner_alloc_count.set(0);
        self.owner_alloc_bytes.set(0);
        self.owner_alloc_zeroed_count.set(0);
        self.owner_alloc_zeroed_bytes.set(0);
        self.owner_realloc_count.set(0);
        self.owner_realloc_bytes.set(0);
    }

    fn owner_snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            alloc_count: self.owner_alloc_count.get(),
            alloc_bytes: self.owner_alloc_bytes.get(),
            alloc_zeroed_count: self.owner_alloc_zeroed_count.get(),
            alloc_zeroed_bytes: self.owner_alloc_zeroed_bytes.get(),
            realloc_count: self.owner_realloc_count.get(),
            realloc_bytes: self.owner_realloc_bytes.get(),
        }
    }
}

#[derive(Clone, Copy)]
struct CounterSnapshot {
    alloc_count: u64,
    alloc_bytes: u64,
    alloc_zeroed_count: u64,
    alloc_zeroed_bytes: u64,
    realloc_count: u64,
    realloc_bytes: u64,
}

impl CounterSnapshot {
    const fn zero() -> Self {
        Self {
            alloc_count: 0,
            alloc_bytes: 0,
            alloc_zeroed_count: 0,
            alloc_zeroed_bytes: 0,
            realloc_count: 0,
            realloc_bytes: 0,
        }
    }

    fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            alloc_count: self.alloc_count.checked_add(other.alloc_count)?,
            alloc_bytes: self.alloc_bytes.checked_add(other.alloc_bytes)?,
            alloc_zeroed_count: self
                .alloc_zeroed_count
                .checked_add(other.alloc_zeroed_count)?,
            alloc_zeroed_bytes: self
                .alloc_zeroed_bytes
                .checked_add(other.alloc_zeroed_bytes)?,
            realloc_count: self.realloc_count.checked_add(other.realloc_count)?,
            realloc_bytes: self.realloc_bytes.checked_add(other.realloc_bytes)?,
        })
    }
}

thread_local! {
    static THREAD_STATE: AllocationThreadState = const { AllocationThreadState::new() };
}

static STATE: CounterState = CounterState::new();
static NEXT_SLOT: AtomicU64 = AtomicU64::new(0);
static SLOTS: [AllocationSlot; SLOT_COUNT] = [const { AllocationSlot::new() }; SLOT_COUNT];

const _: () = assert!(size_of::<AllocationSlot>() == 128);
const _: () = assert!(align_of::<AllocationSlot>() == 128);
const _: () = assert!(!needs_drop::<AllocationThreadState>());

#[cfg(test)]
static TEST_PAUSE_BEFORE_ADMISSION: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_BEFORE_ADMISSION: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_RELEASE_BEFORE_ADMISSION: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_PAUSE_AFTER_ODD: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_ODD_PUBLISHED: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_RELEASE_AFTER_ODD: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_PAUSE_AFTER_ADMISSION: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_ADMITTED: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_RELEASE_ADMITTED: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_FORCE_TLS_FAILURE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_PAUSE_PREADMISSION_FAULT: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_PREADMISSION_FAULT_READY: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static TEST_RELEASE_PREADMISSION_FAULT: AtomicBool = AtomicBool::new(false);

/// A tooling-only global allocator wrapper that delegates exactly to `System`.
pub struct ProcessAllocationCounter {
    system: System,
}

impl ProcessAllocationCounter {
    /// Creates a wrapper suitable for an `#[global_allocator]` static.
    #[must_use]
    pub const fn system() -> Self {
        Self { system: System }
    }
}

/// Exact gross successful allocation traffic observed in one window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AllocationSnapshot {
    pub producer_pid: u32,
    pub window_id: u64,
    pub alloc_count: u64,
    pub alloc_bytes: u64,
    pub alloc_zeroed_count: u64,
    pub alloc_zeroed_bytes: u64,
    pub realloc_count: u64,
    pub realloc_bytes: u64,
    pub allocator_allocation_count: u64,
    pub allocator_allocated_bytes: u64,
}

/// A unique process-wide measurement token.
///
/// The token is deliberately bound to the thread that opened the window.
///
/// ```compile_fail
/// fn require_send<T: Send>() {}
/// require_send::<next_process_allocation_counter::Measurement>();
/// ```
///
/// ```compile_fail
/// fn require_sync<T: Sync>() {}
/// require_sync::<next_process_allocation_counter::Measurement>();
/// ```
#[derive(Debug)]
pub struct Measurement {
    window_id: u64,
    producer_pid: u32,
    finished: bool,
    same_thread: PhantomData<Rc<()>>,
}

impl Measurement {
    /// Closes the window and returns a complete snapshot.
    pub fn finish(mut self) -> Result<AllocationSnapshot, MeasurementError> {
        self.finished = true;
        close::finish_window(self.window_id, self.producer_pid)
    }
}

impl Drop for Measurement {
    fn drop(&mut self) {
        if !self.finished {
            abandon_window(self.window_id, self.producer_pid);
        }
    }
}

/// Stable failure classification for allocator measurement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeasurementError {
    CounterUnavailable,
    NestedWindow,
    AbandonedWindow,
    ProcessMismatch,
    StaleWindow,
    CounterOverflow,
    WindowIdOverflow,
    InFlightOverflow,
    SlotExhausted,
    TlsUnavailable,
    SequenceOverflow,
    Recursion,
    CloseTimeout,
    InvalidCall,
    Poisoned,
}

impl MeasurementError {
    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.as_str()
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CounterUnavailable => "PERF_ALLOCATOR_COUNTER_UNAVAILABLE",
            Self::NestedWindow => "PERF_ALLOCATOR_NESTED_WINDOW",
            Self::AbandonedWindow => "PERF_ALLOCATOR_ABANDONED_WINDOW",
            Self::ProcessMismatch => "PERF_ALLOCATOR_PROCESS_MISMATCH",
            Self::StaleWindow => "PERF_ALLOCATOR_STALE_WINDOW",
            Self::CounterOverflow => "PERF_ALLOCATOR_COUNTER_OVERFLOW",
            Self::WindowIdOverflow => "PERF_ALLOCATOR_WINDOW_ID_OVERFLOW",
            Self::InFlightOverflow => "PERF_ALLOCATOR_IN_FLIGHT_OVERFLOW",
            Self::SlotExhausted => "PERF_ALLOCATOR_SLOT_EXHAUSTED",
            Self::TlsUnavailable => "PERF_ALLOCATOR_TLS_UNAVAILABLE",
            Self::SequenceOverflow => "PERF_ALLOCATOR_SEQUENCE_OVERFLOW",
            Self::Recursion => "PERF_ALLOCATOR_RECURSION",
            Self::CloseTimeout => "PERF_ALLOCATOR_CLOSE_TIMEOUT",
            Self::InvalidCall => "PERF_ALLOCATOR_INVALID_CALL",
            Self::Poisoned => "PERF_ALLOCATOR_POISONED",
        }
    }
}

impl fmt::Display for MeasurementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::error::Error for MeasurementError {}

/// Opens the sole process-wide allocation measurement window.
pub fn begin() -> Result<Measurement, MeasurementError> {
    let current = STATE.control.load(Ordering::SeqCst);
    if phase(current) == PHASE_POISONED {
        return Err(fault_error(current));
    }
    if phase(current) != PHASE_IDLE {
        let _ = poison_exact(current, FAULT_NESTED_WINDOW);
        return Err(MeasurementError::NestedWindow);
    }
    if control_window(current) != 0 || control_faults(current) != 0 {
        let _ = poison_exact(current, FAULT_PROTOCOL);
        return Err(MeasurementError::Poisoned);
    }
    let starting = (current & HOOK_SEEN_BIT) | PHASE_STARTING;
    if let Err(observed) =
        STATE
            .control
            .compare_exchange(current, starting, Ordering::SeqCst, Ordering::SeqCst)
    {
        let _ = poison_exact(observed, FAULT_NESTED_WINDOW);
        return Err(MeasurementError::NestedWindow);
    }
    if !hook_seen(starting) {
        let _ = poison_exact(starting, FAULT_HOOK_UNAVAILABLE);
        return Err(MeasurementError::CounterUnavailable);
    }

    let window_id = match next_window_id() {
        Some(window_id) => window_id,
        None => {
            let _ = poison_exact(starting, FAULT_WINDOW_ID_OVERFLOW);
            return Err(MeasurementError::WindowIdOverflow);
        }
    };
    STATE.faults.store(0, Ordering::Relaxed);
    for slot in &SLOTS {
        slot.reset_counters();
    }
    if let Err(error) = prepare_owner_thread(window_id) {
        let _ = poison_exact(starting, fault_for_error(error));
        return Err(error);
    }

    let producer_pid = std::process::id();
    STATE.owner_pid.store(producer_pid, Ordering::Relaxed);
    let active = encode_control(PHASE_ACTIVE, window_id);
    if let Err(observed) =
        STATE
            .control
            .compare_exchange(starting, active, Ordering::SeqCst, Ordering::SeqCst)
    {
        let _ = clear_owner_window(window_id);
        return Err(fault_error(observed));
    }
    Ok(Measurement {
        window_id,
        producer_pid,
        finished: false,
        same_thread: PhantomData,
    })
}

/// Returns the fixed process-local storage reserved by this instrumentation.
#[must_use]
pub const fn reserved_bytes() -> usize {
    size_of::<CounterState>()
        + size_of::<AtomicU64>()
        + size_of::<[AllocationSlot; SLOT_COUNT]>()
        + size_of::<AllocationThreadState>() * SLOT_COUNT
        + size_of::<ProcessAllocationCounter>()
}

fn prepare_owner_thread(window_id: u64) -> Result<(), MeasurementError> {
    THREAD_STATE
        .try_with(|thread| {
            if thread.in_callback.get() {
                return Err(MeasurementError::Recursion);
            }
            if thread.owner_window_id.get() != 0 {
                return Err(MeasurementError::StaleWindow);
            }
            if thread.slot_index.get() == UNCLAIMED_SLOT {
                let slot = claim_slot_slow().ok_or(MeasurementError::SlotExhausted)?;
                thread.slot_index.set(slot);
            }
            let slot_index =
                usize::try_from(thread.slot_index.get()).map_err(|_| MeasurementError::Poisoned)?;
            let slot = SLOTS.get(slot_index).ok_or(MeasurementError::Poisoned)?;
            if slot.sequence.load(Ordering::Relaxed) & 1 != 0 {
                return Err(MeasurementError::Recursion);
            }
            thread.owner_window_id.set(window_id);
            thread.reset_owner_counters();
            Ok(())
        })
        .map_err(|_| MeasurementError::TlsUnavailable)?
}

fn validate_owner_thread(window_id: u64) -> Result<(), MeasurementError> {
    THREAD_STATE
        .try_with(|thread| {
            if thread.owner_window_id.get() != window_id
                || thread.slot_index.get() == UNCLAIMED_SLOT
            {
                return Err(MeasurementError::StaleWindow);
            }
            if thread.in_callback.get() {
                return Err(MeasurementError::Recursion);
            }
            Ok(())
        })
        .map_err(|_| MeasurementError::TlsUnavailable)?
}

fn take_owner_snapshot(window_id: u64) -> Result<CounterSnapshot, MeasurementError> {
    THREAD_STATE
        .try_with(|thread| {
            if thread.owner_window_id.get() != window_id
                || thread.slot_index.get() == UNCLAIMED_SLOT
            {
                return Err(MeasurementError::StaleWindow);
            }
            if thread.in_callback.get() {
                return Err(MeasurementError::Recursion);
            }
            let snapshot = thread.owner_snapshot();
            thread.owner_window_id.set(0);
            thread.reset_owner_counters();
            Ok(snapshot)
        })
        .map_err(|_| MeasurementError::TlsUnavailable)?
}

fn clear_owner_window(window_id: u64) -> Result<bool, MeasurementError> {
    THREAD_STATE
        .try_with(|thread| {
            if thread.owner_window_id.get() != window_id {
                return false;
            }
            thread.owner_window_id.set(0);
            thread.reset_owner_counters();
            true
        })
        .map_err(|_| MeasurementError::TlsUnavailable)
}

fn abandon_window(window_id: u64, producer_pid: u32) {
    let active = encode_control(PHASE_ACTIVE, window_id);
    let current_pid = std::process::id();
    let fault =
        if current_pid != producer_pid || STATE.owner_pid.load(Ordering::Acquire) != current_pid {
            FAULT_PROCESS_MISMATCH
        } else {
            match validate_owner_thread(window_id) {
                Ok(()) => FAULT_ABANDONED_WINDOW,
                Err(error) => fault_for_error(error),
            }
        };
    let _ = poison_exact(active, fault);
    let _ = clear_owner_window(window_id);
}

#[cold]
#[inline(never)]
fn claim_slot_slow() -> Option<u32> {
    let mut current = NEXT_SLOT.load(Ordering::Relaxed);
    loop {
        if current >= SLOT_COUNT as u64 {
            return None;
        }
        match NEXT_SLOT.compare_exchange_weak(
            current,
            current + 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Some(current as u32),
            Err(observed) => current = observed,
        }
    }
}

fn next_window_id() -> Option<u64> {
    let mut current = STATE.next_window_id.load(Ordering::Relaxed);
    loop {
        let next = current.checked_add(1)?;
        if next > WINDOW_VALUE_MASK {
            return None;
        }
        match STATE.next_window_id.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Some(next),
            Err(observed) => current = observed,
        }
    }
}

#[inline(always)]
fn phase(control: u64) -> u64 {
    control & PHASE_MASK
}

#[inline(always)]
fn control_window(control: u64) -> u64 {
    (control & WINDOW_MASK) >> WINDOW_SHIFT
}

#[inline(always)]
fn control_faults(control: u64) -> u64 {
    control & FAULT_MASK
}

#[inline(always)]
fn encode_control(phase: u64, window_id: u64) -> u64 {
    phase | HOOK_SEEN_BIT | (window_id << WINDOW_SHIFT)
}

#[inline(always)]
fn hook_seen(control: u64) -> bool {
    control & HOOK_SEEN_BIT != 0
}

#[cold]
#[inline(never)]
fn poison_exact(expected: u64, fault: u64) -> bool {
    let poisoned =
        (expected & (HOOK_SEEN_BIT | WINDOW_MASK)) | PHASE_POISONED | (fault & FAULT_MASK);
    STATE
        .control
        .compare_exchange(expected, poisoned, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

#[cold]
#[inline(never)]
fn record_admitted_fault(fault: u64) {
    STATE.faults.fetch_or(fault & FAULT_MASK, Ordering::Release);
}

#[cold]
#[inline(never)]
fn record_preadmission_fault(expected: u64, fault: u64) -> bool {
    #[cfg(test)]
    pause_before_preadmission_fault();
    poison_exact(expected, fault)
}

fn fault_error(control: u64) -> MeasurementError {
    let faults = control_faults(control) | STATE.faults.load(Ordering::Acquire);
    if faults & FAULT_HOOK_UNAVAILABLE != 0 {
        MeasurementError::CounterUnavailable
    } else if faults & FAULT_NESTED_WINDOW != 0 {
        MeasurementError::NestedWindow
    } else if faults & FAULT_ABANDONED_WINDOW != 0 {
        MeasurementError::AbandonedWindow
    } else if faults & FAULT_PROCESS_MISMATCH != 0 {
        MeasurementError::ProcessMismatch
    } else if faults & FAULT_STALE_WINDOW != 0 {
        MeasurementError::StaleWindow
    } else if faults & FAULT_COUNTER_OVERFLOW != 0 {
        MeasurementError::CounterOverflow
    } else if faults & FAULT_WINDOW_ID_OVERFLOW != 0 {
        MeasurementError::WindowIdOverflow
    } else if faults & FAULT_SLOT_EXHAUSTED != 0 {
        MeasurementError::SlotExhausted
    } else if faults & FAULT_CLOSE_TIMEOUT != 0 {
        MeasurementError::CloseTimeout
    } else if faults & FAULT_INVALID_CALL != 0 {
        MeasurementError::InvalidCall
    } else if faults & FAULT_TLS_UNAVAILABLE != 0 {
        MeasurementError::TlsUnavailable
    } else if faults & FAULT_SEQUENCE_OVERFLOW != 0 {
        MeasurementError::SequenceOverflow
    } else if faults & FAULT_RECURSION != 0 {
        MeasurementError::Recursion
    } else {
        MeasurementError::Poisoned
    }
}

const fn fault_for_error(error: MeasurementError) -> u64 {
    match error {
        MeasurementError::CounterUnavailable => FAULT_HOOK_UNAVAILABLE,
        MeasurementError::NestedWindow => FAULT_NESTED_WINDOW,
        MeasurementError::AbandonedWindow => FAULT_ABANDONED_WINDOW,
        MeasurementError::ProcessMismatch => FAULT_PROCESS_MISMATCH,
        MeasurementError::StaleWindow => FAULT_STALE_WINDOW,
        MeasurementError::CounterOverflow => FAULT_COUNTER_OVERFLOW,
        MeasurementError::WindowIdOverflow => FAULT_WINDOW_ID_OVERFLOW,
        MeasurementError::SlotExhausted => FAULT_SLOT_EXHAUSTED,
        MeasurementError::TlsUnavailable => FAULT_TLS_UNAVAILABLE,
        MeasurementError::SequenceOverflow => FAULT_SEQUENCE_OVERFLOW,
        MeasurementError::Recursion => FAULT_RECURSION,
        MeasurementError::CloseTimeout => FAULT_CLOSE_TIMEOUT,
        MeasurementError::InvalidCall => FAULT_INVALID_CALL,
        MeasurementError::InFlightOverflow | MeasurementError::Poisoned => FAULT_PROTOCOL,
    }
}

#[cold]
#[inline(never)]
fn observe_hook_slow(mut current: u64) -> u64 {
    loop {
        if hook_seen(current) {
            return current;
        }
        let observed = current | HOOK_SEEN_BIT;
        match STATE.control.compare_exchange_weak(
            current,
            observed,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => return observed,
            Err(changed) => current = changed,
        }
    }
}

#[inline(always)]
fn observe_control() -> u64 {
    let current = STATE.control.load(Ordering::SeqCst);
    if hook_seen(current) {
        current
    } else {
        observe_hook_slow(current)
    }
}

#[cfg(test)]
fn pause_before_admission() {
    if TEST_PAUSE_BEFORE_ADMISSION.load(Ordering::Acquire) {
        TEST_BEFORE_ADMISSION.store(true, Ordering::Release);
        while !TEST_RELEASE_BEFORE_ADMISSION.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
    }
}

#[cfg(test)]
fn pause_after_odd() {
    if TEST_PAUSE_AFTER_ODD.load(Ordering::Acquire) {
        TEST_ODD_PUBLISHED.store(true, Ordering::Release);
        while !TEST_RELEASE_AFTER_ODD.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
    }
}

#[cfg(test)]
fn pause_after_admission() {
    if TEST_PAUSE_AFTER_ADMISSION.load(Ordering::Acquire) {
        TEST_ADMITTED.store(true, Ordering::Release);
        while !TEST_RELEASE_ADMITTED.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
    }
}

#[cfg(test)]
fn pause_before_preadmission_fault() {
    if TEST_PAUSE_PREADMISSION_FAULT.load(Ordering::Acquire) {
        TEST_PREADMISSION_FAULT_READY.store(true, Ordering::Release);
        while !TEST_RELEASE_PREADMISSION_FAULT.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
    }
}

#[cfg(test)]
fn force_tls_failure(observed: u64) -> bool {
    if TEST_FORCE_TLS_FAILURE.load(Ordering::Acquire) {
        let _ = record_preadmission_fault(observed, FAULT_TLS_UNAVAILABLE);
        true
    } else {
        false
    }
}

#[allow(
    unsafe_code,
    reason = "ADR-039 permits this single GlobalAlloc boundary"
)]
// SAFETY: every operation forwards the caller-provided GlobalAlloc contract
// unchanged to exactly one matching `System` operation. The wrapper neither
// changes pointer/layout ownership nor fabricates references.
unsafe impl GlobalAlloc for ProcessAllocationCounter {
    #[inline(always)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let observed = observe_control();
        if phase(observed) != PHASE_ACTIVE {
            // SAFETY: the valid caller-owned layout is forwarded unchanged.
            return unsafe { self.system.alloc(layout) };
        }
        // SAFETY: the helper preserves and delegates the same contract once.
        unsafe { self.alloc_owner_active(layout, observed) }
    }

    #[inline(always)]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let observed = observe_control();
        if phase(observed) != PHASE_ACTIVE {
            // SAFETY: the valid caller-owned layout is forwarded unchanged.
            return unsafe { self.system.alloc_zeroed(layout) };
        }
        // SAFETY: the helper preserves and delegates the same contract once.
        unsafe { self.alloc_zeroed_owner_active(layout, observed) }
    }

    #[inline(always)]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let observed = observe_control();
        if phase(observed) != PHASE_ACTIVE {
            // SAFETY: the caller-owned pointer/layout/size are forwarded once.
            return unsafe { self.system.realloc(ptr, layout, new_size) };
        }
        // SAFETY: the helper preserves and delegates the same contract once.
        unsafe { self.realloc_owner_active(ptr, layout, new_size, observed) }
    }

    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: the caller-owned live pointer/layout are forwarded unchanged
        // to exactly one matching `System` operation. ADR-042 intentionally
        // keeps deallocation outside all measurement state and TLS paths.
        unsafe { self.system.dealloc(ptr, layout) };
    }
}

#[cfg(test)]
#[allow(
    unsafe_code,
    reason = "contract tests exercise valid GlobalAlloc calls directly"
)]
mod tests;
