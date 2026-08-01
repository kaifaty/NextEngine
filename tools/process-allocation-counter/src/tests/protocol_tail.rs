use super::*;

pub(super) fn fresh_dealloc_only_thread_claims_only_for_a_later_counted_call() {
    reset_state(true);
    let allocation_ready = Arc::new(Barrier::new(2));
    let start_deallocation = Arc::new(Barrier::new(2));
    let deallocation_complete = Arc::new(Barrier::new(2));
    let start_counted_call = Arc::new(Barrier::new(2));
    let worker = {
        let allocation_ready = Arc::clone(&allocation_ready);
        let start_deallocation = Arc::clone(&start_deallocation);
        let deallocation_complete = Arc::clone(&deallocation_complete);
        let start_counted_call = Arc::clone(&start_counted_call);
        thread::spawn(move || {
            let retired = Layout::from_size_align(64, 64).expect("over-aligned retired layout");
            let counted = Layout::from_size_align(24, 8).expect("counted layout");
            // SAFETY: both layouts are non-zero. Each successful pointer is
            // accessed only within its allocation and released exactly once
            // through the same allocator with its matching current layout.
            unsafe {
                let pointer = TEST_ALLOCATOR.alloc(retired);
                assert!(!pointer.is_null());
                assert_eq!((pointer as usize) % retired.align(), 0);
                for offset in 0..retired.size() {
                    pointer.add(offset).write(0xa5);
                }
                allocation_ready.wait();
                start_deallocation.wait();
                assert!((0..retired.size()).all(|offset| pointer.add(offset).read() == 0xa5));
                TEST_ALLOCATOR.dealloc(pointer, retired);
            }

            let slot_after_deallocation = THREAD_STATE.with(|thread| thread.slot_index.get());
            deallocation_complete.wait();
            start_counted_call.wait();

            // SAFETY: the non-zero layout is valid and the successful pointer
            // is released once through the same allocator.
            unsafe {
                let pointer = TEST_ALLOCATOR.alloc(counted);
                assert!(!pointer.is_null());
                for offset in 0..counted.size() {
                    pointer.add(offset).write(0x3c);
                }
                assert!((0..counted.size()).all(|offset| pointer.add(offset).read() == 0x3c));
                TEST_ALLOCATOR.dealloc(pointer, counted);
            }
            let counted_slot = THREAD_STATE.with(|thread| thread.slot_index.get());
            (slot_after_deallocation, counted_slot)
        })
    };

    allocation_ready.wait();
    assert_eq!(NEXT_SLOT.load(Ordering::Acquire), 0);
    let measurement = begin().expect("fresh-thread deallocation window opens");
    let owner_slot = current_thread_slot();
    assert_eq!(NEXT_SLOT.load(Ordering::Acquire), 1);

    start_deallocation.wait();
    deallocation_complete.wait();
    assert_eq!(NEXT_SLOT.load(Ordering::Acquire), 1);
    assert_eq!(STATE.faults.load(Ordering::Acquire), 0);
    assert_eq!(SLOTS[owner_slot].sequence.load(Ordering::Acquire), 0);
    assert_eq!(SLOTS[owner_slot].alloc_count.load(Ordering::Relaxed), 0);

    start_counted_call.wait();
    let (slot_after_deallocation, counted_slot) =
        worker.join().expect("fresh deallocation worker exits");
    assert_eq!(slot_after_deallocation, UNCLAIMED_SLOT);
    assert_ne!(counted_slot, UNCLAIMED_SLOT);
    let counted_slot = usize::try_from(counted_slot).expect("counted slot fits usize");
    assert_ne!(counted_slot, owner_slot);
    assert_eq!(NEXT_SLOT.load(Ordering::Acquire), 2);
    let counted_sequence = SLOTS[counted_slot].sequence.load(Ordering::Acquire);
    assert_eq!(counted_sequence & 1, 0);
    assert!(counted_sequence >= 2);

    let snapshot = measurement
        .finish()
        .expect("fresh-thread counted call is published");
    assert_eq!(snapshot.alloc_count, 1);
    assert_eq!(snapshot.alloc_bytes, 24);
    assert_eq!(snapshot.alloc_zeroed_count, 0);
    assert_eq!(snapshot.alloc_zeroed_bytes, 0);
    assert_eq!(snapshot.realloc_count, 0);
    assert_eq!(snapshot.realloc_bytes, 0);
    assert_eq!(snapshot.allocator_allocation_count, 1);
    assert_eq!(snapshot.allocator_allocated_bytes, 24);
}

pub(super) fn thread_slots_are_lazy_unique_and_process_lifetime() {
    reset_state(true);
    assert_eq!(NEXT_SLOT.load(Ordering::Relaxed), 0);

    let first_owner = begin().expect("first owner window opens");
    let first_owner_slot = current_thread_slot();
    first_owner.finish().expect("first owner window closes");
    assert_eq!(NEXT_SLOT.load(Ordering::Relaxed), 1);

    let worker = thread::spawn(|| {
        assert_eq!(
            THREAD_STATE.with(|thread| thread.slot_index.get()),
            UNCLAIMED_SLOT
        );
        let first = begin().expect("worker owner window opens");
        let first_slot = current_thread_slot();
        first.finish().expect("worker owner window closes");
        let second = begin().expect("worker second owner window opens");
        let second_slot = current_thread_slot();
        second.finish().expect("worker second owner window closes");
        (first_slot, second_slot)
    });
    let (worker_first_slot, worker_second_slot) = worker.join().expect("worker exits");
    assert_eq!(worker_first_slot, worker_second_slot);
    assert_ne!(worker_first_slot, first_owner_slot);
    assert_eq!(NEXT_SLOT.load(Ordering::Relaxed), 2);

    let second_owner = begin().expect("main owner slot is reusable");
    assert_eq!(current_thread_slot(), first_owner_slot);
    second_owner.finish().expect("main owner window closes");
    assert_eq!(NEXT_SLOT.load(Ordering::Relaxed), 2);
}

pub(super) fn owner_and_foreign_operation_kinds_are_aggregated_once() {
    reset_state(true);
    let ready = Arc::new(Barrier::new(2));
    let start = Arc::new(Barrier::new(2));
    let complete = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker = {
        let ready = Arc::clone(&ready);
        let start = Arc::clone(&start);
        let complete = Arc::clone(&complete);
        let release = Arc::clone(&release);
        thread::spawn(move || {
            let allocated = Layout::from_size_align(24, 8).expect("foreign allocation layout");
            let zeroed = Layout::from_size_align(8, 8).expect("foreign zeroed layout");
            ready.wait();
            start.wait();
            // SAFETY: layouts are non-zero and each live pointer is released
            // exactly once through the same allocator with its current layout.
            unsafe {
                let pointer = TEST_ALLOCATOR.alloc(allocated);
                assert!(!pointer.is_null());
                let resized = TEST_ALLOCATOR.realloc(pointer, allocated, 40);
                assert!(!resized.is_null());
                let zeroed_pointer = TEST_ALLOCATOR.alloc_zeroed(zeroed);
                assert!(!zeroed_pointer.is_null());
                assert!((0..zeroed.size()).all(|offset| zeroed_pointer.add(offset).read() == 0));
                TEST_ALLOCATOR.dealloc(resized, Layout::from_size_align_unchecked(40, 8));
                TEST_ALLOCATOR.dealloc(zeroed_pointer, zeroed);
            }
            complete.wait();
            release.wait();
        })
    };
    ready.wait();

    let measurement = begin().expect("mixed owner/foreign window opens");
    let allocated = Layout::from_size_align(16, 8).expect("owner allocation layout");
    let zeroed = Layout::from_size_align(32, 8).expect("owner zeroed layout");
    // SAFETY: layouts are non-zero and each live pointer is released exactly
    // once through the same allocator with its current layout.
    unsafe {
        let pointer = TEST_ALLOCATOR.alloc(allocated);
        assert!(!pointer.is_null());
        let resized = TEST_ALLOCATOR.realloc(pointer, allocated, 48);
        assert!(!resized.is_null());
        let zeroed_pointer = TEST_ALLOCATOR.alloc_zeroed(zeroed);
        assert!(!zeroed_pointer.is_null());
        assert!((0..zeroed.size()).all(|offset| zeroed_pointer.add(offset).read() == 0));
        TEST_ALLOCATOR.dealloc(resized, Layout::from_size_align_unchecked(48, 8));
        TEST_ALLOCATOR.dealloc(zeroed_pointer, zeroed);
    }
    start.wait();
    complete.wait();

    let snapshot = measurement.finish().expect("mixed window closes");
    release.wait();
    worker.join().expect("mixed-operation worker exits");
    assert_eq!(snapshot.alloc_count, 2);
    assert_eq!(snapshot.alloc_bytes, 40);
    assert_eq!(snapshot.alloc_zeroed_count, 2);
    assert_eq!(snapshot.alloc_zeroed_bytes, 40);
    assert_eq!(snapshot.realloc_count, 2);
    assert_eq!(snapshot.realloc_bytes, 88);
    assert_eq!(snapshot.allocator_allocation_count, 6);
    assert_eq!(snapshot.allocator_allocated_bytes, 168);
}

pub(super) fn rotating_owners_use_distinct_shared_slots() {
    reset_state(true);
    const OWNER_COUNT: usize = 8;
    let mut slots = Vec::with_capacity(OWNER_COUNT);
    for _ in 0..OWNER_COUNT {
        let slot = thread::spawn(|| {
            let measurement = begin().expect("rotating owner window opens");
            let slot = current_thread_slot();
            measurement.finish().expect("rotating owner window closes");
            slot
        })
        .join()
        .expect("rotating owner exits");
        slots.push(slot);
    }
    let mut unique = slots.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), OWNER_COUNT);
    assert_eq!(NEXT_SLOT.load(Ordering::Relaxed), OWNER_COUNT as u64);
}

pub(super) fn close_waits_for_an_admitted_call() {
    reset_state(true);
    TEST_PAUSE_AFTER_ADMISSION.store(true, Ordering::Release);
    let measurement = begin().expect("admitted close-race window opens");
    let worker = thread::spawn(|| {
        let layout = Layout::new::<u64>();
        // SAFETY: the valid pointer is released through the matching allocator.
        unsafe {
            let pointer = TEST_ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            TEST_ALLOCATOR.dealloc(pointer, layout);
        }
    });
    while !TEST_ADMITTED.load(Ordering::Acquire) {
        thread::yield_now();
    }
    let releaser = thread::spawn(|| {
        while phase(STATE.control.load(Ordering::SeqCst)) != PHASE_CLOSING {
            thread::yield_now();
        }
        TEST_RELEASE_ADMITTED.store(true, Ordering::Release);
    });
    let snapshot = measurement
        .finish()
        .expect("close waits for the admitted callback");
    releaser.join().expect("admission releaser exits");
    worker.join().expect("admitted worker exits");
    assert_eq!(snapshot.alloc_count, 1);
    assert_eq!(snapshot.alloc_bytes, 8);
    assert_eq!(snapshot.allocator_allocation_count, 1);
    assert_eq!(snapshot.allocator_allocated_bytes, 8);
}

pub(super) fn close_between_odd_and_postcheck_is_untracked() {
    reset_state(true);
    TEST_PAUSE_AFTER_ODD.store(true, Ordering::Release);
    let measurement = begin().expect("odd-postcheck race window opens");
    let worker = thread::spawn(|| {
        let layout = Layout::new::<u64>();
        // SAFETY: the valid pointer is released through the matching allocator.
        unsafe {
            let pointer = TEST_ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            TEST_ALLOCATOR.dealloc(pointer, layout);
        }
    });
    while !TEST_ODD_PUBLISHED.load(Ordering::Acquire) {
        thread::yield_now();
    }
    let releaser = thread::spawn(|| {
        while phase(STATE.control.load(Ordering::SeqCst)) != PHASE_CLOSING {
            thread::yield_now();
        }
        TEST_RELEASE_AFTER_ODD.store(true, Ordering::Release);
    });
    let snapshot = measurement
        .finish()
        .expect("close wins between odd publication and identity postcheck");
    releaser.join().expect("odd-postcheck releaser exits");
    worker.join().expect("odd-postcheck worker exits");
    assert_eq!(snapshot.allocator_allocation_count, 0);
    assert_eq!(snapshot.allocator_allocated_bytes, 0);
}

pub(super) fn callback_observed_before_odd_is_not_missed_or_counted() {
    reset_state(true);
    TEST_PAUSE_BEFORE_ADMISSION.store(true, Ordering::Release);
    let measurement = begin().expect("before-odd race window opens");
    let worker = thread::spawn(|| {
        let layout = Layout::new::<u64>();
        // SAFETY: the valid pointer is released through the matching allocator.
        unsafe {
            let pointer = TEST_ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            TEST_ALLOCATOR.dealloc(pointer, layout);
        }
    });
    while !TEST_BEFORE_ADMISSION.load(Ordering::Acquire) {
        thread::yield_now();
    }
    let snapshot = measurement
        .finish()
        .expect("close may pass a callback that has not published odd");
    assert_eq!(snapshot.allocator_allocation_count, 0);
    assert_eq!(snapshot.allocator_allocated_bytes, 0);
    TEST_RELEASE_BEFORE_ADMISSION.store(true, Ordering::Release);
    worker.join().expect("late callback exits");
    assert_eq!(phase(STATE.control.load(Ordering::SeqCst)), PHASE_IDLE);
}

pub(super) fn preadmission_fault_and_close_linearize_exactly() {
    reset_state(true);
    TEST_FORCE_TLS_FAILURE.store(true, Ordering::Release);
    TEST_PAUSE_PREADMISSION_FAULT.store(true, Ordering::Release);
    let measurement = begin().expect("fault-close race window opens");
    let worker = thread::spawn(|| {
        let layout = Layout::new::<u64>();
        // SAFETY: the valid pointer is released through the matching allocator.
        unsafe {
            let pointer = TEST_ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            TEST_ALLOCATOR.dealloc(pointer, layout);
        }
    });
    while !TEST_PREADMISSION_FAULT_READY.load(Ordering::Acquire) {
        thread::yield_now();
    }
    let snapshot = measurement
        .finish()
        .expect("close wins against a callback that has not admitted");
    assert_eq!(snapshot.allocator_allocation_count, 0);
    assert_eq!(snapshot.allocator_allocated_bytes, 0);
    TEST_RELEASE_PREADMISSION_FAULT.store(true, Ordering::Release);
    worker.join().expect("late preadmission fault exits");
    TEST_FORCE_TLS_FAILURE.store(false, Ordering::Release);
    assert_eq!(phase(STATE.control.load(Ordering::SeqCst)), PHASE_IDLE);
    assert_eq!(STATE.faults.load(Ordering::Acquire), 0);
}

pub(super) fn stale_callback_cannot_enter_a_sequential_window() {
    reset_state(true);
    TEST_PAUSE_BEFORE_ADMISSION.store(true, Ordering::Release);
    let first = begin().expect("first sequential window opens");
    let first_window_id = first.window_id;
    let worker = thread::spawn(|| {
        let layout = Layout::new::<u64>();
        // SAFETY: the valid pointer is released through the matching allocator.
        unsafe {
            let pointer = TEST_ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            TEST_ALLOCATOR.dealloc(pointer, layout);
        }
    });
    while !TEST_BEFORE_ADMISSION.load(Ordering::Acquire) {
        thread::yield_now();
    }
    let first_snapshot = first.finish().expect("first window closes");
    let second = begin().expect("second sequential window opens");
    let second_window_id = second.window_id;
    assert!(second_window_id > first_window_id);
    TEST_RELEASE_BEFORE_ADMISSION.store(true, Ordering::Release);
    worker.join().expect("stale callback exits");
    let second_snapshot = second.finish().expect("second window closes");
    assert_eq!(first_snapshot.allocator_allocation_count, 0);
    assert_eq!(second_snapshot.allocator_allocation_count, 0);
}

pub(super) fn owner_slot_stays_dormant_across_many_windows() {
    reset_state(true);
    let mut previous_window = 0;
    let mut owner_slot = None;
    for _ in 0..100 {
        let measurement = begin().expect("sequential window opens");
        assert!(measurement.window_id > previous_window);
        previous_window = measurement.window_id;
        let layout = Layout::new::<u8>();
        // SAFETY: the valid pointer is released through the matching allocator.
        unsafe {
            let pointer = TEST_ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            TEST_ALLOCATOR.dealloc(pointer, layout);
        }
        let slot_index = current_thread_slot();
        assert_eq!(*owner_slot.get_or_insert(slot_index), slot_index);
        assert_eq!(SLOTS[slot_index].sequence.load(Ordering::Acquire), 0);
        assert_eq!(SLOTS[slot_index].alloc_count.load(Ordering::Relaxed), 0);
        assert_eq!(SLOTS[slot_index].alloc_bytes.load(Ordering::Relaxed), 0);
        let snapshot = measurement.finish().expect("sequential window closes");
        assert_eq!(snapshot.allocator_allocation_count, 1);
    }
    assert_eq!(NEXT_SLOT.load(Ordering::Relaxed), 1);
}

pub(super) fn nested_abandoned_pid_and_stale_tokens_fail_closed() {
    reset_state(true);
    let outer = begin().expect("outer window opens");
    assert_eq!(
        begin().expect_err("nested window is rejected"),
        MeasurementError::NestedWindow
    );
    assert_eq!(
        outer.finish().expect_err("outer window is poisoned"),
        MeasurementError::NestedWindow
    );

    reset_state(true);
    let abandoned = begin().expect("abandoned window opens");
    drop(abandoned);
    assert_eq!(
        begin().expect_err("abandoned window remains sticky"),
        MeasurementError::AbandonedWindow
    );

    reset_state(true);
    let process = begin().expect("PID window opens");
    STATE
        .owner_pid
        .store(std::process::id().wrapping_add(1), Ordering::Release);
    assert_eq!(
        process.finish().expect_err("PID mismatch is rejected"),
        MeasurementError::ProcessMismatch
    );

    reset_state(true);
    let wrong_marker = begin().expect("wrong-marker window opens");
    let wrong_marker_id = wrong_marker.window_id.saturating_add(1);
    THREAD_STATE.with(|thread| thread.owner_window_id.set(wrong_marker_id));
    assert_eq!(
        wrong_marker
            .finish()
            .expect_err("wrong owner marker is rejected"),
        MeasurementError::StaleWindow
    );
    THREAD_STATE.with(|thread| assert_eq!(thread.owner_window_id.get(), wrong_marker_id));

    reset_state(true);
    let mut stale = begin().expect("stale-token window opens");
    stale.window_id = stale.window_id.saturating_add(1);
    assert_eq!(
        stale.finish().expect_err("stale token is rejected"),
        MeasurementError::StaleWindow
    );
    assert_eq!(phase(STATE.control.load(Ordering::SeqCst)), PHASE_POISONED);

    reset_state(false);
    assert_eq!(
        begin().expect_err("missing hook is rejected"),
        MeasurementError::CounterUnavailable
    );

    reset_state(true);
    STATE
        .next_window_id
        .store(WINDOW_VALUE_MASK, Ordering::Relaxed);
    assert_eq!(
        begin().expect_err("window identity overflow is rejected"),
        MeasurementError::WindowIdOverflow
    );
}

pub(super) fn counter_sequence_slot_tls_and_timeout_faults_fail_closed() {
    reset_state(true);
    let counter = begin().expect("counter-overflow window opens");
    THREAD_STATE.with(|thread| thread.owner_alloc_count.set(u64::MAX));
    let layout = Layout::new::<u8>();
    // SAFETY: the valid pointer is released through the matching allocator.
    unsafe {
        let pointer = TEST_ALLOCATOR.alloc(layout);
        assert!(!pointer.is_null());
        TEST_ALLOCATOR.dealloc(pointer, layout);
    }
    assert_eq!(
        counter.finish().expect_err("counter overflow is rejected"),
        MeasurementError::CounterOverflow
    );

    reset_state(true);
    let aggregate = begin().expect("aggregate-overflow window opens");
    SLOTS[0].alloc_count.store(u64::MAX, Ordering::Relaxed);
    SLOTS[1].alloc_count.store(1, Ordering::Relaxed);
    assert_eq!(
        aggregate
            .finish()
            .expect_err("aggregate overflow is rejected"),
        MeasurementError::CounterOverflow
    );

    reset_state(true);
    let sequence = begin().expect("sequence-overflow window opens");
    let foreign_slot =
        usize::try_from(NEXT_SLOT.load(Ordering::Relaxed)).expect("next foreign slot fits usize");
    SLOTS[foreign_slot]
        .sequence
        .store(u64::MAX - 1, Ordering::Relaxed);
    thread::spawn(move || {
        // SAFETY: the valid pointer is released through the matching allocator.
        unsafe {
            let pointer = TEST_ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            TEST_ALLOCATOR.dealloc(pointer, layout);
        }
    })
    .join()
    .expect("sequence-overflow worker exits");
    assert_eq!(
        sequence
            .finish()
            .expect_err("sequence overflow is rejected"),
        MeasurementError::SequenceOverflow
    );

    reset_state(true);
    let exhausted = begin().expect("slot-exhaustion window opens");
    NEXT_SLOT.store(SLOT_COUNT as u64, Ordering::Relaxed);
    let worker = thread::spawn(|| {
        let layout = Layout::new::<u8>();
        // SAFETY: the valid pointer is released through the matching allocator.
        unsafe {
            let pointer = TEST_ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            TEST_ALLOCATOR.dealloc(pointer, layout);
        }
    });
    worker.join().expect("exhausted worker exits");
    assert_eq!(
        exhausted.finish().expect_err("slot 4097 is rejected"),
        MeasurementError::SlotExhausted
    );

    reset_state(true);
    NEXT_SLOT.store((SLOT_COUNT - 1) as u64, Ordering::Relaxed);
    let last_owner = thread::spawn(|| {
        begin()
            .expect("last shared owner slot is available")
            .finish()
            .expect("last-slot owner closes")
    })
    .join()
    .expect("last-slot owner exits");
    assert_eq!(last_owner.allocator_allocation_count, 0);
    assert_eq!(NEXT_SLOT.load(Ordering::Relaxed), SLOT_COUNT as u64);
    assert_eq!(
        begin().expect_err("distinct owner 4097 is rejected"),
        MeasurementError::SlotExhausted
    );

    reset_state(true);
    let tls = begin().expect("TLS-failure window opens");
    TEST_FORCE_TLS_FAILURE.store(true, Ordering::Release);
    // SAFETY: the valid pointer is released through the same allocator.
    unsafe {
        let pointer = TEST_ALLOCATOR.alloc(layout);
        assert!(!pointer.is_null());
        TEST_ALLOCATOR.dealloc(pointer, layout);
    }
    TEST_FORCE_TLS_FAILURE.store(false, Ordering::Release);
    assert_eq!(
        tls.finish().expect_err("TLS failure is rejected"),
        MeasurementError::TlsUnavailable
    );

    reset_state(true);
    let invalid = begin().expect("invalid-call window opens");
    active::validate_non_zero(0);
    assert_eq!(
        invalid.finish().expect_err("zero size is rejected"),
        MeasurementError::InvalidCall
    );

    reset_state(true);
    let timeout = begin().expect("close-timeout window opens");
    let slot = current_thread_slot();
    SLOTS[slot].sequence.store(1, Ordering::Relaxed);
    assert_eq!(
        timeout.finish().expect_err("odd slot times out"),
        MeasurementError::CloseTimeout
    );
}

pub(super) fn pre_admission_fault_cannot_poison_an_already_closed_window() {
    reset_state(true);
    let measurement = begin().expect("linearization window opens");
    let active = encode_control(PHASE_ACTIVE, measurement.window_id);
    let snapshot = measurement.finish().expect("empty window closes");
    assert_eq!(snapshot.allocator_allocation_count, 0);
    assert!(!poison_exact(active, FAULT_TLS_UNAVAILABLE));
    assert_eq!(phase(STATE.control.load(Ordering::SeqCst)), PHASE_IDLE);
}

pub(super) fn stable_codes_and_reserved_storage_are_bounded() {
    assert_eq!(
        MeasurementError::CounterUnavailable.code(),
        "PERF_ALLOCATOR_COUNTER_UNAVAILABLE"
    );
    assert_eq!(
        MeasurementError::SlotExhausted.as_str(),
        "PERF_ALLOCATOR_SLOT_EXHAUSTED"
    );
    assert_eq!(
        MeasurementError::TlsUnavailable.as_str(),
        "PERF_ALLOCATOR_TLS_UNAVAILABLE"
    );
    assert_eq!(
        MeasurementError::Poisoned.to_string(),
        "PERF_ALLOCATOR_POISONED"
    );
    assert_eq!(SLOT_COUNT, 4_096);
    assert_eq!(size_of::<AllocationSlot>(), 128);
    assert_eq!(align_of::<AllocationSlot>(), 128);
    assert!(!needs_drop::<AllocationThreadState>());
    assert!(reserved_bytes() >= size_of::<[AllocationSlot; SLOT_COUNT]>());
    assert!(reserved_bytes() < 1024 * 1024);
    assert!(reserved_bytes() <= 64 * 1024 * 1024);
}

pub(super) fn seed_deallocation_probe_state(control: u64, next_slot: u64) {
    STATE.control.store(control, Ordering::SeqCst);
    STATE.next_window_id.store(41, Ordering::SeqCst);
    STATE.owner_pid.store(std::process::id(), Ordering::SeqCst);
    STATE.faults.store(FAULT_COUNTER_OVERFLOW, Ordering::SeqCst);
    NEXT_SLOT.store(next_slot, Ordering::SeqCst);

    let slot = &SLOTS[0];
    slot.sequence.store(42, Ordering::SeqCst);
    slot.alloc_count.store(43, Ordering::SeqCst);
    slot.alloc_bytes.store(44, Ordering::SeqCst);
    slot.alloc_zeroed_count.store(45, Ordering::SeqCst);
    slot.alloc_zeroed_bytes.store(46, Ordering::SeqCst);
    slot.realloc_count.store(47, Ordering::SeqCst);
    slot.realloc_bytes.store(48, Ordering::SeqCst);

    THREAD_STATE.with(|thread| {
        thread.slot_index.set(UNCLAIMED_SLOT);
        thread.owner_window_id.set(49);
        thread.owner_alloc_count.set(50);
        thread.owner_alloc_bytes.set(51);
        thread.owner_alloc_zeroed_count.set(52);
        thread.owner_alloc_zeroed_bytes.set(53);
        thread.owner_realloc_count.set(54);
        thread.owner_realloc_bytes.set(55);
    });
}

pub(super) fn measurement_state_probe() -> MeasurementStateProbe {
    let thread = THREAD_STATE.with(|thread| ThreadStateProbe {
        slot_index: thread.slot_index.get(),
        owner_window_id: thread.owner_window_id.get(),
        owner_counters: [
            thread.owner_alloc_count.get(),
            thread.owner_alloc_bytes.get(),
            thread.owner_alloc_zeroed_count.get(),
            thread.owner_alloc_zeroed_bytes.get(),
            thread.owner_realloc_count.get(),
            thread.owner_realloc_bytes.get(),
        ],
    });
    MeasurementStateProbe {
        control: STATE.control.load(Ordering::SeqCst),
        next_window_id: STATE.next_window_id.load(Ordering::SeqCst),
        owner_pid: STATE.owner_pid.load(Ordering::SeqCst),
        faults: STATE.faults.load(Ordering::SeqCst),
        next_slot: NEXT_SLOT.load(Ordering::SeqCst),
        first_slot: allocation_slot_probe(0),
        last_slot: allocation_slot_probe(SLOT_COUNT - 1),
        thread,
    }
}

fn allocation_slot_probe(index: usize) -> [u64; 7] {
    let slot = &SLOTS[index];
    [
        slot.sequence.load(Ordering::SeqCst),
        slot.alloc_count.load(Ordering::SeqCst),
        slot.alloc_bytes.load(Ordering::SeqCst),
        slot.alloc_zeroed_count.load(Ordering::SeqCst),
        slot.alloc_zeroed_bytes.load(Ordering::SeqCst),
        slot.realloc_count.load(Ordering::SeqCst),
        slot.realloc_bytes.load(Ordering::SeqCst),
    ]
}

pub(super) fn observe_hook_once() {
    let layout = Layout::new::<u8>();
    // SAFETY: the non-zero layout is valid and the pointer is released through
    // the same allocator before returning.
    unsafe {
        let pointer = TEST_ALLOCATOR.alloc(layout);
        assert!(!pointer.is_null());
        TEST_ALLOCATOR.dealloc(pointer, layout);
    }
    assert!(hook_seen(STATE.control.load(Ordering::SeqCst)));
}

pub(super) fn current_thread_slot() -> usize {
    THREAD_STATE
        .try_with(|thread| usize::try_from(thread.slot_index.get()).expect("slot fits usize"))
        .expect("test TLS remains available")
}

pub(super) fn reset_state(callback_seen: bool) {
    let hook = if callback_seen { HOOK_SEEN_BIT } else { 0 };
    STATE.control.store(PHASE_IDLE | hook, Ordering::SeqCst);
    STATE.next_window_id.store(0, Ordering::SeqCst);
    STATE.owner_pid.store(0, Ordering::SeqCst);
    STATE.faults.store(0, Ordering::SeqCst);
    NEXT_SLOT.store(0, Ordering::SeqCst);
    for slot in &SLOTS {
        slot.sequence.store(0, Ordering::SeqCst);
        slot.reset_counters();
    }
    THREAD_STATE
        .try_with(|thread| {
            thread.slot_index.set(UNCLAIMED_SLOT);
            thread.owner_window_id.set(0);
            thread.reset_owner_counters();
        })
        .expect("test TLS remains available");
    for flag in [
        &TEST_PAUSE_BEFORE_ADMISSION,
        &TEST_BEFORE_ADMISSION,
        &TEST_RELEASE_BEFORE_ADMISSION,
        &TEST_PAUSE_AFTER_ODD,
        &TEST_ODD_PUBLISHED,
        &TEST_RELEASE_AFTER_ODD,
        &TEST_PAUSE_AFTER_ADMISSION,
        &TEST_ADMITTED,
        &TEST_RELEASE_ADMITTED,
        &TEST_FORCE_TLS_FAILURE,
        &TEST_PAUSE_PREADMISSION_FAULT,
        &TEST_PREADMISSION_FAULT_READY,
        &TEST_RELEASE_PREADMISSION_FAULT,
    ] {
        flag.store(false, Ordering::SeqCst);
    }
}
