use std::time::Duration;

use next_contracts::ids::ContentHash;
use next_contracts::platform::{PlatformEventKindV1, PlatformEventV1};
use next_contracts::presentation::PresentationSnapshotV2;
use next_contracts::session::ApplicationSessionStatusV1;

use crate::{ApplicationCoordinator, ApplicationError, ApplicationRunOutcomeV1};

const REFERENCE_GAMEPLAY_HZ: u128 = 30;
const SCALED_SECOND: u128 = 1_000_000_000;
const MAXIMUM_STEPS_PER_PUMP: u32 = 8;
// One scheduler backlog becomes one production input frame at the next fixed
// boundary. Keep it aligned with both the desktop adapter's per-pump batch
// limit and the player input resolver's per-frame admission limit.
const MAXIMUM_PENDING_PLATFORM_EVENTS: usize = 4_096;

/// Host-side fixed-step scheduler for the reference live game.
///
/// Wall time determines only how many already-declared fixed gameplay
/// boundaries are due. Active catch-up is bounded to one pump budget, so a
/// host that cannot sustain real time slows the live simulation instead of
/// accumulating an unbounded wall-time debt. Platform events remain buffered
/// until the next boundary, and rendering may read the latest published
/// presentation snapshot at any cadence.
#[derive(Debug, Default)]
pub struct FixedStepLiveSchedulerV1 {
    accumulated_scaled_nanoseconds: u128,
    pending_events: Vec<PlatformEventV1>,
    deferred_events: Vec<PlatformEventV1>,
    consumed_lifecycle_events: Vec<PlatformEventV1>,
    preprocessed_lifecycle_event_ids: Vec<ContentHash>,
}

impl FixedStepLiveSchedulerV1 {
    #[must_use]
    pub const fn reference_game_v1() -> Self {
        Self {
            accumulated_scaled_nanoseconds: 0,
            pending_events: Vec::new(),
            deferred_events: Vec::new(),
            consumed_lifecycle_events: Vec::new(),
            preprocessed_lifecycle_event_ids: Vec::new(),
        }
    }

    /// Admits one host-pump interval and advances at most the bounded catch-up
    /// count. Fixed boundaries made due by `elapsed` are closed before events
    /// observed by the current host callback are admitted, so a slow render
    /// callback cannot retroactively place new input into an earlier gameplay
    /// tick. An oversized combined event backlog is rejected before changing
    /// either the accumulator or the event queues. A failed gameplay step keeps
    /// its unconsumed time and prior events so the caller can diagnose or retry
    /// without silently dropping input.
    pub fn advance_reference_game(
        &mut self,
        application: &mut ApplicationCoordinator,
        elapsed: Duration,
        events: &[PlatformEventV1],
    ) -> Result<Option<ApplicationRunOutcomeV1>, ApplicationError> {
        let crossed_suspend_boundary = self.retry_consumed_lifecycle_events(application)?;
        if crossed_suspend_boundary {
            self.accumulated_scaled_nanoseconds = 0;
        }
        let mut lifecycle_plan = self.pending_events.clone();
        lifecycle_plan.extend_from_slice(&self.deferred_events);
        lifecycle_plan.extend_from_slice(events);
        canonicalize_platform_events(&mut lifecycle_plan);
        application.with_platform_event_admission(events, &lifecycle_plan, |application| {
            self.advance_reference_game_with_admitted_events(
                application,
                elapsed,
                events,
                ApplicationCoordinator::advance_reference_game_live_admitted,
            )
        })
    }

    /// Interactive hot path: advances the same fixed authoritative boundaries
    /// while returning only the immutable presentation projection between
    /// durable checkpoint boundaries.
    pub fn advance_reference_game_presentation(
        &mut self,
        application: &mut ApplicationCoordinator,
        elapsed: Duration,
        events: &[PlatformEventV1],
    ) -> Result<Option<PresentationSnapshotV2>, ApplicationError> {
        let crossed_suspend_boundary = self.retry_consumed_lifecycle_events(application)?;
        if crossed_suspend_boundary {
            self.accumulated_scaled_nanoseconds = 0;
        }
        let mut lifecycle_plan = self.pending_events.clone();
        lifecycle_plan.extend_from_slice(&self.deferred_events);
        lifecycle_plan.extend_from_slice(events);
        canonicalize_platform_events(&mut lifecycle_plan);
        application.with_platform_event_admission(events, &lifecycle_plan, |application| {
            self.advance_reference_game_with_admitted_events(
                application,
                elapsed,
                events,
                ApplicationCoordinator::advance_reference_game_live_presentation_admitted,
            )
        })
    }

    fn advance_reference_game_with_admitted_events<T>(
        &mut self,
        application: &mut ApplicationCoordinator,
        elapsed: Duration,
        events: &[PlatformEventV1],
        mut advance_step: impl FnMut(
            &mut ApplicationCoordinator,
            &[PlatformEventV1],
        ) -> Result<T, ApplicationError>,
    ) -> Result<Option<T>, ApplicationError> {
        if application.state().state == ApplicationSessionStatusV1::Suspended {
            return self.advance_suspended(application, events);
        }
        if application.state().state != ApplicationSessionStatusV1::Active {
            return Err(ApplicationError::CloseStateInvalid);
        }

        let scaled_delta = elapsed
            .as_nanos()
            .checked_mul(REFERENCE_GAMEPLAY_HZ)
            .ok_or(ApplicationError::LiveTickBacklogExceeded)?;
        let candidate_accumulator = self
            .accumulated_scaled_nanoseconds
            .checked_add(scaled_delta)
            .ok_or(ApplicationError::LiveTickBacklogExceeded)?;
        let maximum_accumulator = u128::from(MAXIMUM_STEPS_PER_PUMP).saturating_mul(SCALED_SECOND);
        let bounded_accumulator = candidate_accumulator.min(maximum_accumulator);
        let due_steps = bounded_accumulator / SCALED_SECOND;
        let pending_after_due = if due_steps == 0 {
            self.pending_events.len()
        } else {
            0
        };
        let candidate_pending_event_count = pending_after_due
            .checked_add(self.deferred_events.len())
            .and_then(|count| count.checked_add(events.len()))
            .ok_or(ApplicationError::LivePlatformEventBacklogExceeded)?;
        if candidate_pending_event_count > MAXIMUM_PENDING_PLATFORM_EVENTS {
            return Err(ApplicationError::LivePlatformEventBacklogExceeded);
        }
        self.accumulated_scaled_nanoseconds = bounded_accumulator;

        let mut latest = None;
        let mut steps = 0_u32;
        while self.accumulated_scaled_nanoseconds >= SCALED_SECOND && steps < MAXIMUM_STEPS_PER_PUMP
        {
            let mut consumed_events = self.pending_events.clone();
            canonicalize_platform_events(&mut consumed_events);
            let state_before_step = application.state().state;
            let run = advance_step(application, &consumed_events)?;
            let crossed_suspend_boundary = state_before_step == ApplicationSessionStatusV1::Active
                && application.state().state == ApplicationSessionStatusV1::Suspended;
            self.pending_events.clear();
            self.accumulated_scaled_nanoseconds -= SCALED_SECOND;
            steps = steps
                .checked_add(1)
                .ok_or(ApplicationError::LiveTickBacklogExceeded)?;
            latest = Some(run);
            if crossed_suspend_boundary {
                let applied_suspend = consumed_events
                    .iter()
                    .find(|event| event.kind == PlatformEventKindV1::SuspendRequested)
                    .ok_or(ApplicationError::DurableSnapshotInvalid)?;
                self.preprocessed_lifecycle_event_ids
                    .push(applied_suspend.platform_event_id);
            }
            self.stage_consumed_lifecycle_events(consumed_events);
            let retried_suspend_boundary = self.retry_consumed_lifecycle_events(application)?;
            if crossed_suspend_boundary || retried_suspend_boundary {
                self.accumulated_scaled_nanoseconds = 0;
                break;
            }
        }
        if application.state().state == ApplicationSessionStatusV1::Suspended
            || self.accumulated_scaled_nanoseconds >= SCALED_SECOND
        {
            self.deferred_events.extend_from_slice(events);
        } else {
            self.pending_events.append(&mut self.deferred_events);
            self.pending_events.extend_from_slice(events);
        }
        Ok(latest)
    }

    fn advance_suspended<T>(
        &mut self,
        application: &mut ApplicationCoordinator,
        events: &[PlatformEventV1],
    ) -> Result<Option<T>, ApplicationError> {
        let candidate_pending_event_count = self
            .pending_events
            .len()
            .checked_add(self.deferred_events.len())
            .and_then(|count| count.checked_add(events.len()))
            .ok_or(ApplicationError::LivePlatformEventBacklogExceeded)?;
        if candidate_pending_event_count > MAXIMUM_PENDING_PLATFORM_EVENTS {
            return Err(ApplicationError::LivePlatformEventBacklogExceeded);
        }

        let mut candidate_pending = self.pending_events.clone();
        candidate_pending.extend_from_slice(&self.deferred_events);
        candidate_pending.extend_from_slice(events);
        canonicalize_platform_events(&mut candidate_pending);
        application.validate_platform_lifecycle_plan(&candidate_pending)?;
        let resume = candidate_pending
            .iter()
            .find(|event| {
                event.kind == PlatformEventKindV1::ResumeRequested
                    && !self
                        .preprocessed_lifecycle_event_ids
                        .contains(&event.platform_event_id)
            })
            .cloned();
        if let Some(resume) = resume {
            application.resume_from_admitted_platform_event(&resume)?;
            self.preprocessed_lifecycle_event_ids
                .push(resume.platform_event_id);
        }
        self.accumulated_scaled_nanoseconds = 0;
        self.pending_events = candidate_pending;
        self.deferred_events.clear();
        Ok(None)
    }

    fn stage_consumed_lifecycle_events(&mut self, consumed_events: Vec<PlatformEventV1>) {
        for event in consumed_events {
            if let Some(index) = self
                .preprocessed_lifecycle_event_ids
                .iter()
                .position(|event_id| *event_id == event.platform_event_id)
            {
                self.preprocessed_lifecycle_event_ids.remove(index);
                continue;
            }
            if matches!(
                event.kind,
                PlatformEventKindV1::SuspendRequested | PlatformEventKindV1::ResumeRequested
            ) {
                self.consumed_lifecycle_events.push(event);
            }
        }
    }

    fn retry_consumed_lifecycle_events(
        &mut self,
        application: &mut ApplicationCoordinator,
    ) -> Result<bool, ApplicationError> {
        application.validate_platform_lifecycle_plan(&self.consumed_lifecycle_events)?;
        let mut crossed_suspend_boundary = false;
        while let Some(event) = self.consumed_lifecycle_events.first().cloned() {
            let before = application.state().state;
            match event.kind {
                PlatformEventKindV1::SuspendRequested => {
                    application.suspend_from_admitted_platform_event(&event)?;
                }
                PlatformEventKindV1::ResumeRequested => {
                    application.resume_from_admitted_platform_event(&event)?;
                }
                _ => return Err(ApplicationError::DurableSnapshotInvalid),
            }
            crossed_suspend_boundary |= before == ApplicationSessionStatusV1::Active
                && application.state().state == ApplicationSessionStatusV1::Suspended;
            self.consumed_lifecycle_events.remove(0);
        }
        Ok(crossed_suspend_boundary)
    }

    #[cfg(test)]
    pub(crate) fn pending_event_count(&self) -> usize {
        self.pending_events.len() + self.deferred_events.len()
    }

    #[cfg(test)]
    pub(crate) fn pending_events(&self) -> Vec<PlatformEventV1> {
        self.pending_events
            .iter()
            .chain(&self.deferred_events)
            .cloned()
            .collect()
    }

    #[cfg(test)]
    pub(crate) const fn accumulated_scaled_nanoseconds(&self) -> u128 {
        self.accumulated_scaled_nanoseconds
    }
}

fn canonicalize_platform_events(events: &mut [PlatformEventV1]) {
    events.sort_by(|left, right| {
        (
            left.host_instance_id,
            &left.source_class,
            left.source_sequence,
            left.platform_event_id,
        )
            .cmp(&(
                right.host_instance_id,
                &right.source_class,
                right.source_sequence,
                right.platform_event_id,
            ))
    });
}
