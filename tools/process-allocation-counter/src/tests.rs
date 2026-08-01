use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Barrier};
use std::thread;

use super::*;

static TEST_ALLOCATOR: ProcessAllocationCounter = ProcessAllocationCounter::system();

#[derive(Debug, Eq, PartialEq)]
struct ThreadStateProbe {
    slot_index: u32,
    in_callback: bool,
    owner_window_id: u64,
    owner_counters: [u64; 6],
}

#[derive(Debug, Eq, PartialEq)]
struct MeasurementStateProbe {
    control: u64,
    next_window_id: u64,
    owner_pid: u32,
    faults: u64,
    next_slot: u64,
    first_slot: [u64; 7],
    last_slot: [u64; 7],
    thread: ThreadStateProbe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModelPhase {
    Idle,
    Starting,
    Active,
    Closing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ModelIdentity {
    phase: ModelPhase,
    current_window_id: Option<u64>,
    next_window_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ModelDeallocationProbe {
    identity: ModelIdentity,
    next_slot: u64,
    participants: u64,
    counted_participant_joined: bool,
    in_flight_counted_callbacks: u64,
    counters: [u64; 6],
    faults: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ModelClosedWindow {
    window_id: u64,
    counters: [u64; 6],
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeallocationLifecycleModel {
    identity: ModelIdentity,
    next_slot: u64,
    participants: u64,
    counted_participant_joined: bool,
    in_flight_counted_callbacks: u64,
    counters: [u64; 6],
    faults: u64,
    closed_windows: Vec<ModelClosedWindow>,
    close_attempts: u64,
    blocked_closes: u64,
}

impl DeallocationLifecycleModel {
    fn new(next_window_id: u64) -> Self {
        Self {
            identity: ModelIdentity {
                phase: ModelPhase::Idle,
                current_window_id: None,
                next_window_id,
            },
            // The lifecycle owner is already a process-lifetime participant.
            next_slot: 1,
            participants: 1,
            counted_participant_joined: false,
            in_flight_counted_callbacks: 0,
            counters: [0; 6],
            faults: 0,
            closed_windows: Vec::new(),
            close_attempts: 0,
            blocked_closes: 0,
        }
    }

    fn deallocation_probe(&self) -> ModelDeallocationProbe {
        ModelDeallocationProbe {
            identity: self.identity,
            next_slot: self.next_slot,
            participants: self.participants,
            counted_participant_joined: self.counted_participant_joined,
            in_flight_counted_callbacks: self.in_flight_counted_callbacks,
            counters: self.counters,
            faults: self.faults,
        }
    }

    fn begin(&mut self, window_id: u64) {
        assert_eq!(self.identity.phase, ModelPhase::Idle);
        assert_eq!(window_id, self.identity.next_window_id + 1);
        self.identity.phase = ModelPhase::Starting;
        self.identity.current_window_id = Some(window_id);
        self.identity.next_window_id = window_id;
        self.counters = [0; 6];
    }

    fn activate(&mut self) {
        assert_eq!(self.identity.phase, ModelPhase::Starting);
        self.identity.phase = ModelPhase::Active;
    }

    fn start_close(&mut self) {
        assert_eq!(self.identity.phase, ModelPhase::Active);
        self.identity.phase = ModelPhase::Closing;
    }

    fn finish_close(&mut self) -> bool {
        assert_eq!(self.identity.phase, ModelPhase::Closing);
        self.close_attempts += 1;
        if self.in_flight_counted_callbacks != 0 {
            self.blocked_closes += 1;
            return false;
        }
        self.closed_windows.push(ModelClosedWindow {
            window_id: self
                .identity
                .current_window_id
                .expect("closing model window has identity"),
            counters: self.counters,
        });
        self.identity.phase = ModelPhase::Idle;
        self.identity.current_window_id = None;
        true
    }

    fn counted_alloc(&mut self, bytes: u64) -> u64 {
        assert_eq!(self.identity.phase, ModelPhase::Active);
        let window_id = self
            .identity
            .current_window_id
            .expect("active model window has identity");
        if !self.counted_participant_joined {
            self.next_slot += 1;
            self.participants += 1;
            self.counted_participant_joined = true;
        }
        self.in_flight_counted_callbacks += 1;
        self.counters[0] += 1;
        self.counters[1] += bytes;
        self.in_flight_counted_callbacks -= 1;
        window_id
    }
}

#[derive(Clone, Copy)]
enum ModelLifecycleStep {
    Begin(u64),
    Activate,
    StartClose,
    FinishClose,
}

#[test]
fn allocator_counter_protocol_contract_suite() {
    exact_known_counts_and_system_semantics();
    model_only_deallocation_cuts_preserve_windows_and_counted_admission();
    direct_deallocation_is_unobserved_in_every_control_phase();
    dealloc_only_unclaimed_participant_preserves_capacity_and_wrap_boundaries();
    fresh_dealloc_only_thread_claims_only_for_a_later_counted_call();
    owner_and_foreign_operation_kinds_are_aggregated_once();
    thread_slots_are_lazy_unique_and_process_lifetime();
    rotating_owners_use_distinct_shared_slots();
    close_waits_for_an_admitted_call();
    close_between_odd_and_postcheck_is_untracked();
    callback_observed_before_odd_is_not_missed_or_counted();
    preadmission_fault_and_close_linearize_exactly();
    recursion_before_odd_cannot_publish_a_false_snapshot();
    stale_callback_cannot_enter_a_sequential_window();
    owner_slot_stays_dormant_across_many_windows();
    nested_abandoned_pid_and_stale_tokens_fail_closed();
    counter_sequence_slot_tls_and_timeout_faults_fail_closed();
    pre_admission_fault_cannot_poison_an_already_closed_window();
    stable_codes_and_reserved_storage_are_bounded();
    reset_state(true);
}

fn exact_known_counts_and_system_semantics() {
    reset_state(false);
    observe_hook_once();
    let measurement = begin().expect("known-count window opens");
    assert_eq!(NEXT_SLOT.load(Ordering::Relaxed), 1);
    let owner_slot = current_thread_slot();
    assert_eq!(SLOTS[owner_slot].sequence.load(Ordering::Relaxed), 0);

    let small = Layout::from_size_align(16, 8).expect("small layout");
    let zeroed = Layout::from_size_align(32, 8).expect("zeroed layout");
    // SAFETY: both layouts are non-zero and every live pointer is passed back
    // to the same allocator with its matching current layout.
    let (resized, zeroed_pointer) = unsafe {
        let pointer = TEST_ALLOCATOR.alloc(small);
        assert!(!pointer.is_null());
        for offset in 0..small.size() {
            pointer.add(offset).write(0x5a);
        }
        let zeroed_pointer = TEST_ALLOCATOR.alloc_zeroed(zeroed);
        assert!(!zeroed_pointer.is_null());
        assert!((0..zeroed.size()).all(|offset| zeroed_pointer.add(offset).read() == 0));
        let resized = TEST_ALLOCATOR.realloc(pointer, small, 48);
        assert!(!resized.is_null());
        assert!((0..small.size()).all(|offset| resized.add(offset).read() == 0x5a));
        (resized, zeroed_pointer)
    };
    let resized_layout = Layout::from_size_align(48, 8).expect("resized layout");
    // SAFETY: the pointers and layouts match the successful operations above.
    unsafe {
        TEST_ALLOCATOR.dealloc(resized, resized_layout);
        TEST_ALLOCATOR.dealloc(zeroed_pointer, zeroed);
    }
    assert_eq!(SLOTS[owner_slot].sequence.load(Ordering::Relaxed), 0);
    assert_eq!(SLOTS[owner_slot].alloc_count.load(Ordering::Relaxed), 0);
    assert_eq!(SLOTS[owner_slot].realloc_count.load(Ordering::Relaxed), 0);

    let snapshot = measurement.finish().expect("known-count window closes");
    assert_eq!(snapshot.alloc_count, 1);
    assert_eq!(snapshot.alloc_bytes, 16);
    assert_eq!(snapshot.alloc_zeroed_count, 1);
    assert_eq!(snapshot.alloc_zeroed_bytes, 32);
    assert_eq!(snapshot.realloc_count, 1);
    assert_eq!(snapshot.realloc_bytes, 48);
    assert_eq!(snapshot.allocator_allocation_count, 3);
    assert_eq!(snapshot.allocator_allocated_bytes, 96);
    assert_eq!(snapshot.producer_pid, std::process::id());
    assert_ne!(snapshot.window_id, 0);
    THREAD_STATE.with(|thread| assert_eq!(thread.owner_window_id.get(), 0));
}

fn model_only_deallocation_cuts_preserve_windows_and_counted_admission() {
    const WINDOW_N: u64 = 73;
    const COUNTED_BYTES: u64 = 24;
    let lifecycle = [
        ModelLifecycleStep::Begin(WINDOW_N),
        ModelLifecycleStep::Activate,
        ModelLifecycleStep::StartClose,
        ModelLifecycleStep::FinishClose,
        ModelLifecycleStep::Begin(WINDOW_N + 1),
        ModelLifecycleStep::Activate,
    ];

    // Insert the unobserved transition before, between, and after every
    // lifecycle transition through the next Active window.
    for cut in 0..=lifecycle.len() {
        let mut model = DeallocationLifecycleModel::new(WINDOW_N - 1);
        for step in &lifecycle[..cut] {
            apply_model_lifecycle_step(&mut model, *step);
        }

        let before = model.deallocation_probe();
        model = model_direct_deallocation(model);
        assert_eq!(
            model.deallocation_probe(),
            before,
            "model deallocation changed measurement state at lifecycle cut {cut}"
        );

        let mut counted_window_id = None;
        if model.identity.phase == ModelPhase::Active {
            counted_window_id = Some(model.counted_alloc(COUNTED_BYTES));
        }
        for step in &lifecycle[cut..] {
            apply_model_lifecycle_step(&mut model, *step);
            if counted_window_id.is_none() && model.identity.phase == ModelPhase::Active {
                counted_window_id = Some(model.counted_alloc(COUNTED_BYTES));
            }
        }
        let counted_window_id =
            counted_window_id.expect("a current Active window follows each cut");

        if model.identity.phase == ModelPhase::Active {
            model.start_close();
            assert!(model.finish_close(), "next model window closes immediately");
        }

        assert_eq!(model.identity.phase, ModelPhase::Idle);
        assert_eq!(model.identity.current_window_id, None);
        assert_eq!(model.identity.next_window_id, WINDOW_N + 1);
        assert_eq!(model.next_slot, 2);
        assert_eq!(model.participants, 2);
        assert_eq!(model.in_flight_counted_callbacks, 0);
        assert_eq!(model.faults, 0);
        assert_eq!(model.close_attempts, 2);
        assert_eq!(model.blocked_closes, 0);
        assert_eq!(model.closed_windows.len(), 2);
        for closed in &model.closed_windows {
            let expected = if closed.window_id == counted_window_id {
                [1, COUNTED_BYTES, 0, 0, 0, 0]
            } else {
                [0; 6]
            };
            assert_eq!(
                closed.counters, expected,
                "counted callback was attributed to the wrong window at cut {cut}"
            );
        }
    }
}

fn apply_model_lifecycle_step(model: &mut DeallocationLifecycleModel, step: ModelLifecycleStep) {
    match step {
        ModelLifecycleStep::Begin(window_id) => model.begin(window_id),
        ModelLifecycleStep::Activate => model.activate(),
        ModelLifecycleStep::StartClose => model.start_close(),
        ModelLifecycleStep::FinishClose => {
            assert!(model.finish_close(), "model close is not blocked")
        }
    }
}

fn model_direct_deallocation(model: DeallocationLifecycleModel) -> DeallocationLifecycleModel {
    // ADR-042 gives direct deallocation no measurement-memory transition.
    model
}

fn direct_deallocation_is_unobserved_in_every_control_phase() {
    let controls = [
        PHASE_IDLE,
        encode_control(PHASE_STARTING, 1),
        encode_control(PHASE_ACTIVE, 2),
        encode_control(PHASE_CLOSING, 3),
        encode_control(PHASE_POISONED, 4) | FAULT_RECURSION,
    ];
    let layout = Layout::from_size_align(64, 64).expect("over-aligned deallocation layout");

    let logical_slot_cursors = [SLOT_COUNT as u64 - 1, SLOT_COUNT as u64, u64::MAX];

    for control in controls {
        for next_slot in logical_slot_cursors {
            reset_state(true);
            // SAFETY: the non-zero layout is valid and the resulting live pointer
            // is released once through the same allocator below.
            let pointer = unsafe { TEST_ALLOCATOR.alloc(layout) };
            assert!(!pointer.is_null());
            assert_eq!((pointer as usize) % layout.align(), 0);
            // SAFETY: the successful allocation is valid for `layout.size()` bytes.
            unsafe {
                for offset in 0..layout.size() {
                    pointer.add(offset).write((offset as u8).wrapping_mul(3));
                }
                assert!(
                    (0..layout.size())
                        .all(|offset| pointer.add(offset).read() == (offset as u8).wrapping_mul(3))
                );
            }

            seed_deallocation_probe_state(control, next_slot);
            TEST_FORCE_TLS_FAILURE.store(true, Ordering::Release);
            let before = measurement_state_probe();
            // SAFETY: `pointer` is still live and `layout` is exactly the layout
            // used by the successful allocation above.
            unsafe { TEST_ALLOCATOR.dealloc(pointer, layout) };
            let after = measurement_state_probe();
            assert_eq!(
                after, before,
                "deallocation changed measurement state for control {control:#018x} and logical slot cursor {next_slot}"
            );
        }
    }

    reset_state(true);
}

fn dealloc_only_unclaimed_participant_preserves_capacity_and_wrap_boundaries() {
    #[derive(Clone, Copy)]
    struct BoundaryCase {
        initial_cursor: u64,
        expected_claim: Option<u32>,
        expected_cursor: u64,
    }

    let cases = [
        BoundaryCase {
            initial_cursor: SLOT_COUNT as u64 - 1,
            expected_claim: Some((SLOT_COUNT - 1) as u32),
            expected_cursor: SLOT_COUNT as u64,
        },
        BoundaryCase {
            initial_cursor: SLOT_COUNT as u64,
            expected_claim: None,
            expected_cursor: SLOT_COUNT as u64,
        },
        BoundaryCase {
            initial_cursor: u64::MAX,
            expected_claim: None,
            expected_cursor: u64::MAX,
        },
    ];
    let layout = Layout::from_size_align(128, 128).expect("capacity-boundary layout");

    for case in cases {
        reset_state(true);
        // SAFETY: the non-zero over-aligned layout is valid and the successful
        // pointer is released exactly once below with this same layout.
        let pointer = unsafe { TEST_ALLOCATOR.alloc(layout) };
        assert!(!pointer.is_null());
        assert_eq!((pointer as usize) % layout.align(), 0);
        // SAFETY: the successful allocation is live for all `layout.size()` bytes.
        unsafe {
            for offset in 0..layout.size() {
                pointer.add(offset).write((offset as u8).wrapping_add(11));
            }
        }

        STATE
            .control
            .store(encode_control(PHASE_ACTIVE, 91), Ordering::SeqCst);
        NEXT_SLOT.store(case.initial_cursor, Ordering::SeqCst);
        THREAD_STATE.with(|thread| {
            thread.slot_index.set(UNCLAIMED_SLOT);
            thread.in_callback.set(false);
            thread.owner_window_id.set(0);
            thread.reset_owner_counters();
        });
        let before = measurement_state_probe();
        // SAFETY: `pointer` remains live, its initialized bytes are not read
        // after this call, and `layout` exactly matches its allocation.
        unsafe { TEST_ALLOCATOR.dealloc(pointer, layout) };
        assert_eq!(measurement_state_probe(), before);
        THREAD_STATE.with(|thread| assert_eq!(thread.slot_index.get(), UNCLAIMED_SLOT));

        let claimed = claim_slot_slow();
        assert_eq!(claimed, case.expected_claim);
        if let Some(slot) = claimed {
            assert!(usize::try_from(slot).expect("claimed slot fits usize") < SLOT_COUNT);
        }
        assert_eq!(NEXT_SLOT.load(Ordering::SeqCst), case.expected_cursor);
    }

    reset_state(true);
}

mod protocol_tail;

use protocol_tail::*;
