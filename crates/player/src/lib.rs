#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::CanonicalError;
use next_contracts::ids::{InputSourceId, PersistentId, SchemaId};
use next_contracts::input::{
    ActionBindingTransformV1, ActionMapManifestV1, INPUT_SAMPLE_SCHEMA_VERSION,
    InputContextStackV1, InputContractError, InputSampleV1, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionV1, PlayerActionValueV1, RuntimeAdmissionLimitsV1,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformContractError, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};

pub const MAX_PLATFORM_EVENTS_PER_INPUT_FRAME: usize = 4_096;
pub const MAX_HELD_CONTROLS: usize = 256;
pub const MAX_PLATFORM_INPUT_SOURCES: usize = 256;
pub const MAX_PENDING_CONTROL_IDENTITIES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlayerInputDiagnosticCodeV1 {
    InputDeviceLost,
}

impl PlayerInputDiagnosticCodeV1 {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::InputDeviceLost => "INPUT_DEVICE_LOST",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PlayerInputError {
    Contract(InputContractError),
    Platform(PlatformContractError),
    Canonical(CanonicalError),
    EventLimitExceeded { actual: usize, limit: usize },
    HeldControlLimitExceeded { actual: usize, limit: usize },
    SessionStateLimitExceeded { actual: usize, limit: usize },
    PlatformEventIdentityCollision,
    SourceSequenceGap,
    SourceSequenceNonMonotonic,
    LogicalFrameSequenceNonMonotonic,
    ContextStale,
    BindingConflict,
    CounterOverflow,
    RecoveryInvalid,
}

impl PlayerInputError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Platform(_) => "INPUT_EVENT_INVALID",
            Self::PlatformEventIdentityCollision => "INPUT_EVENT_IDENTITY_COLLISION",
            Self::SourceSequenceGap => "INPUT_SEQUENCE_GAP",
            Self::SourceSequenceNonMonotonic | Self::LogicalFrameSequenceNonMonotonic => {
                "INPUT_SEQUENCE_NON_MONOTONIC"
            }
            Self::ContextStale => "INPUT_CONTEXT_STALE",
            Self::BindingConflict => "INPUT_BINDING_CONFLICT",
            Self::Contract(InputContractError::ResourceLimit)
            | Self::EventLimitExceeded { .. }
            | Self::HeldControlLimitExceeded { .. }
            | Self::SessionStateLimitExceeded { .. } => "INPUT_RESOURCE_LIMIT",
            Self::Contract(_) | Self::Canonical(_) | Self::CounterOverflow => "INPUT_EVENT_INVALID",
            Self::RecoveryInvalid => "INPUT_EVENT_INVALID",
        }
    }
}

impl Display for PlayerInputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "input contract invalid: {error}"),
            Self::Platform(error) => write!(formatter, "platform event invalid: {error}"),
            Self::Canonical(error) => write!(formatter, "input canonicalization failed: {error}"),
            Self::EventLimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "input frame has {actual} platform events; limit is {limit}"
                )
            }
            Self::HeldControlLimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "input session has {actual} held controls; limit is {limit}"
                )
            }
            Self::SessionStateLimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "input session has {actual} pending identities; limit is {limit}"
                )
            }
            Self::PlatformEventIdentityCollision => {
                formatter.write_str("platform event identity maps to different event bodies")
            }
            Self::SourceSequenceGap => {
                formatter.write_str("platform source sequence contains a gap")
            }
            Self::SourceSequenceNonMonotonic => {
                formatter.write_str("platform source sequence is not monotonic")
            }
            Self::LogicalFrameSequenceNonMonotonic => {
                formatter.write_str("logical input frame sequence is not monotonic")
            }
            Self::ContextStale => {
                formatter.write_str("input context does not match the action map")
            }
            Self::BindingConflict => {
                formatter.write_str("input bindings do not resolve deterministically")
            }
            Self::CounterOverflow => formatter.write_str("input counter overflow"),
            Self::RecoveryInvalid => formatter.write_str("input session recovery state is invalid"),
        }
    }
}

impl Error for PlayerInputError {}

impl From<InputContractError> for PlayerInputError {
    fn from(error: InputContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<PlatformContractError> for PlayerInputError {
    fn from(error: PlatformContractError) -> Self {
        Self::Platform(error)
    }
}

impl From<CanonicalError> for PlayerInputError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

mod recovery;
mod resolution;

use resolution::{
    action_shapes_match, binding_matches_identity, canonical_value_sort_key, control_is_active,
    matching_bindings, neutral_value, resolve_definition, validate_context_compatibility,
    validate_control_shape,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPlayerInputFrameV1 {
    pub frame: PlayerActionFrameV1,
    pub sample: InputSampleV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerInputFrameOutcomeV1 {
    pub resolved: Option<ResolvedPlayerInputFrameV1>,
    pub diagnostics: Vec<PlayerInputDiagnosticCodeV1>,
    pub configuration_changed: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PlatformSourceKey {
    host_instance_id: PersistentId,
    source_class: SchemaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlatformSourceCursor {
    source_sequence: u64,
    last_event_id: next_contracts::ids::ContentHash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceContinuityV1 {
    AlreadyAdmitted,
    Admit,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ControlIdentity {
    device_class: SchemaId,
    device_instance_nonce: PersistentId,
    control_path_id: SchemaId,
    modifier_set: Vec<SchemaId>,
}

impl ControlIdentity {
    fn from_control(control: &NormalizedControlEventV1) -> Self {
        Self {
            device_class: control.device_class.clone(),
            device_instance_nonce: control.device_instance_nonce,
            control_path_id: control.control_path_id.clone(),
            modifier_set: control.modifier_set.clone(),
        }
    }

    fn same_physical_control(&self, control: &NormalizedControlEventV1) -> bool {
        self.device_class == control.device_class
            && self.device_instance_nonce == control.device_instance_nonce
            && self.control_path_id == control.control_path_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PrioritizedPlayerAction {
    context_priority: u32,
    action: PlayerActionV1,
}

#[derive(Clone, Debug)]
pub struct PlayerInputSessionV1 {
    controller_id: PersistentId,
    source_id: InputSourceId,
    action_map: ActionMapManifestV1,
    context_stack: InputContextStackV1,
    pending_action_map: Option<ActionMapManifestV1>,
    pending_context_stack: Option<InputContextStackV1>,
    source_cursors: BTreeMap<PlatformSourceKey, PlatformSourceCursor>,
    pending_platform_events: Vec<PlatformEventV1>,
    pending_host_consumed_events: Vec<PlatformEventV1>,
    last_logical_frame_sequence: Option<u64>,
    held_controls: BTreeMap<ControlIdentity, Vec<i16>>,
    started_controls: BTreeSet<ControlIdentity>,
    pending_deltas: BTreeMap<ControlIdentity, [i64; 2]>,
    active_actions: BTreeMap<SchemaId, PlayerActionValueV1>,
    cancelled_action_ids: BTreeSet<SchemaId>,
    diagnostics: BTreeSet<PlayerInputDiagnosticCodeV1>,
}

impl PlayerInputSessionV1 {
    pub fn new(
        controller_id: PersistentId,
        source_id: InputSourceId,
        action_map: ActionMapManifestV1,
        context_stack: InputContextStackV1,
    ) -> Result<Self, PlayerInputError> {
        action_map.validate()?;
        context_stack.validate()?;
        validate_context_compatibility(&action_map, &context_stack)?;
        Ok(Self {
            controller_id,
            source_id,
            action_map,
            context_stack,
            pending_action_map: None,
            pending_context_stack: None,
            source_cursors: BTreeMap::new(),
            pending_platform_events: Vec::new(),
            pending_host_consumed_events: Vec::new(),
            last_logical_frame_sequence: None,
            held_controls: BTreeMap::new(),
            started_controls: BTreeSet::new(),
            pending_deltas: BTreeMap::new(),
            active_actions: BTreeMap::new(),
            cancelled_action_ids: BTreeSet::new(),
            diagnostics: BTreeSet::new(),
        })
    }

    #[must_use]
    pub const fn controller_id(&self) -> PersistentId {
        self.controller_id
    }

    #[must_use]
    pub const fn source_id(&self) -> InputSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn action_map(&self) -> &ActionMapManifestV1 {
        &self.action_map
    }

    #[must_use]
    pub const fn context_stack(&self) -> &InputContextStackV1 {
        &self.context_stack
    }

    #[must_use]
    pub const fn last_logical_frame_sequence(&self) -> Option<u64> {
        self.last_logical_frame_sequence
    }

    pub fn queue_action_map(
        &mut self,
        action_map: ActionMapManifestV1,
    ) -> Result<(), PlayerInputError> {
        action_map.validate()?;
        let current = self.pending_action_map.as_ref().unwrap_or(&self.action_map);
        if action_map.action_map_id != current.action_map_id
            || action_map.revision <= current.revision
            || !action_shapes_match(current, &action_map)
        {
            return Err(PlayerInputError::ContextStale);
        }
        let context = self
            .pending_context_stack
            .as_ref()
            .unwrap_or(&self.context_stack);
        validate_context_compatibility(&action_map, context)?;
        self.pending_action_map = Some(action_map);
        Ok(())
    }

    pub fn queue_context_stack(
        &mut self,
        context_stack: InputContextStackV1,
    ) -> Result<(), PlayerInputError> {
        context_stack.validate()?;
        let current = self
            .pending_context_stack
            .as_ref()
            .unwrap_or(&self.context_stack);
        if context_stack.stack_id != current.stack_id || context_stack.revision <= current.revision
        {
            return Err(PlayerInputError::ContextStale);
        }
        let action_map = self.pending_action_map.as_ref().unwrap_or(&self.action_map);
        validate_context_compatibility(action_map, &context_stack)?;
        self.pending_context_stack = Some(context_stack);
        Ok(())
    }

    /// Queues platform events the engine-owned host consumed outside the
    /// game input stream (interactive pause-menu keys while the session was
    /// suspended). They join the next closed frame in canonical order as
    /// cursor-only admissions: validated and continuity-checked like real
    /// events, but without control, diagnostic or action effects. This
    /// keeps per-source sequence continuity exact across input the game can
    /// never observe. The queue is transient and never recoverable, like
    /// the regular pending-event queue.
    pub fn queue_host_consumed_platform_events(
        &mut self,
        events: &[PlatformEventV1],
    ) -> Result<(), PlayerInputError> {
        let candidate_event_count = self
            .pending_host_consumed_events
            .len()
            .checked_add(events.len())
            .ok_or(PlayerInputError::CounterOverflow)?;
        if candidate_event_count > MAX_PLATFORM_EVENTS_PER_INPUT_FRAME {
            return Err(PlayerInputError::EventLimitExceeded {
                actual: candidate_event_count,
                limit: MAX_PLATFORM_EVENTS_PER_INPUT_FRAME,
            });
        }
        for event in events {
            event.validate()?;
        }
        self.pending_host_consumed_events.extend_from_slice(events);
        Ok(())
    }

    pub fn submit_platform_events(
        &mut self,
        events: &[PlatformEventV1],
    ) -> Result<(), PlayerInputError> {
        let candidate_event_count = self
            .pending_platform_events
            .len()
            .checked_add(events.len())
            .ok_or(PlayerInputError::CounterOverflow)?;
        if candidate_event_count > MAX_PLATFORM_EVENTS_PER_INPUT_FRAME {
            return Err(PlayerInputError::EventLimitExceeded {
                actual: candidate_event_count,
                limit: MAX_PLATFORM_EVENTS_PER_INPUT_FRAME,
            });
        }
        for event in events {
            event.validate()?;
        }
        self.pending_platform_events.extend_from_slice(events);
        Ok(())
    }

    pub fn close_frame(
        &mut self,
        logical_frame_sequence: u64,
    ) -> Result<PlayerInputFrameOutcomeV1, PlayerInputError> {
        let mut candidate = self.clone();
        let outcome = candidate.close_frame_in_place(logical_frame_sequence)?;
        *self = candidate;
        Ok(outcome)
    }

    fn close_frame_in_place(
        &mut self,
        logical_frame_sequence: u64,
    ) -> Result<PlayerInputFrameOutcomeV1, PlayerInputError> {
        if self
            .last_logical_frame_sequence
            .is_some_and(|previous| logical_frame_sequence <= previous)
        {
            return Err(PlayerInputError::LogicalFrameSequenceNonMonotonic);
        }

        self.apply_pending_platform_events()?;
        let mut actions = self.resolve_actions()?;
        actions.sort_by(|left, right| {
            right
                .context_priority
                .cmp(&left.context_priority)
                .then_with(|| {
                    (
                        &left.action.action_id,
                        left.action.phase,
                        canonical_value_sort_key(left.action.value),
                    )
                        .cmp(&(
                            &right.action.action_id,
                            right.action.phase,
                            canonical_value_sort_key(right.action.value),
                        ))
                })
        });
        for (ordinal, action) in actions.iter_mut().enumerate() {
            action.action.semantic_occurrence_ordinal =
                u32::try_from(ordinal).map_err(|_| PlayerInputError::CounterOverflow)?;
        }
        let actions = actions
            .into_iter()
            .map(|resolved| resolved.action)
            .collect::<Vec<_>>();

        let resolved = if actions.is_empty() {
            None
        } else {
            let frame = PlayerActionFrameV1 {
                schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
                controller_id: self.controller_id,
                logical_frame_sequence,
                action_map_hash: self.action_map.content_hash,
                action_map_revision: self.action_map.revision,
                context_stack_hash: self.context_stack.content_hash,
                context_stack_revision: self.context_stack.revision,
                actions,
            };
            // new()/queue_*() revalidate the action-map/context-stack pair on
            // every mutation, so the per-frame pair check is guaranteed here.
            frame.validate_against_validated_pair(&self.action_map, &self.context_stack)?;
            let payload = frame.canonical_bytes()?;
            let sample = InputSampleV1 {
                schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
                source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS)
                    .map_err(InputContractError::from)?,
                source_id: self.source_id,
                source_sequence: logical_frame_sequence,
                payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)
                    .map_err(InputContractError::from)?,
                payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
                payload,
                sampled_wall_time: None,
            };
            sample.validate(&RuntimeAdmissionLimitsV1::default())?;
            Some(ResolvedPlayerInputFrameV1 { frame, sample })
        };

        let diagnostics = self.diagnostics.iter().copied().collect();
        self.diagnostics.clear();
        self.started_controls.clear();
        self.pending_deltas.clear();
        self.cancelled_action_ids.clear();
        self.pending_platform_events.clear();
        self.last_logical_frame_sequence = Some(logical_frame_sequence);

        let configuration_changed =
            self.pending_action_map.is_some() || self.pending_context_stack.is_some();
        if let Some(action_map) = self.pending_action_map.take() {
            self.action_map = action_map;
        }
        if let Some(context_stack) = self.pending_context_stack.take() {
            self.context_stack = context_stack;
        }

        Ok(PlayerInputFrameOutcomeV1 {
            resolved,
            diagnostics,
            configuration_changed,
        })
    }

    fn apply_pending_platform_events(&mut self) -> Result<(), PlayerInputError> {
        let real_events = std::mem::take(&mut self.pending_platform_events);
        let host_consumed_events = std::mem::take(&mut self.pending_host_consumed_events);
        let merged_event_count = real_events
            .len()
            .checked_add(host_consumed_events.len())
            .ok_or(PlayerInputError::CounterOverflow)?;
        if merged_event_count > MAX_PLATFORM_EVENTS_PER_INPUT_FRAME {
            return Err(PlayerInputError::EventLimitExceeded {
                actual: merged_event_count,
                limit: MAX_PLATFORM_EVENTS_PER_INPUT_FRAME,
            });
        }
        let mut ordered: Vec<(bool, PlatformEventV1)> = real_events
            .into_iter()
            .map(|event| (false, event))
            .chain(host_consumed_events.into_iter().map(|event| (true, event)))
            .collect();
        ordered.sort_by(|left, right| {
            (
                left.1.host_instance_id,
                &left.1.source_class,
                left.1.source_sequence,
                left.1.platform_event_id,
            )
                .cmp(&(
                    right.1.host_instance_id,
                    &right.1.source_class,
                    right.1.source_sequence,
                    right.1.platform_event_id,
                ))
        });
        let mut disconnected_devices = BTreeSet::new();
        for (host_consumed, event) in ordered {
            if host_consumed {
                self.process_host_consumed_platform_event(&event)?;
            } else {
                self.process_platform_event(&event, &mut disconnected_devices)?;
            }
        }
        if self.held_controls.len() > MAX_HELD_CONTROLS {
            return Err(PlayerInputError::HeldControlLimitExceeded {
                actual: self.held_controls.len(),
                limit: MAX_HELD_CONTROLS,
            });
        }
        if self.source_cursors.len() > MAX_PLATFORM_INPUT_SOURCES {
            return Err(PlayerInputError::SessionStateLimitExceeded {
                actual: self.source_cursors.len(),
                limit: MAX_PLATFORM_INPUT_SOURCES,
            });
        }
        let pending_identity_count = self
            .started_controls
            .len()
            .checked_add(self.pending_deltas.len())
            .ok_or(PlayerInputError::CounterOverflow)?;
        if pending_identity_count > MAX_PENDING_CONTROL_IDENTITIES {
            return Err(PlayerInputError::SessionStateLimitExceeded {
                actual: pending_identity_count,
                limit: MAX_PENDING_CONTROL_IDENTITIES,
            });
        }
        Ok(())
    }

    /// Cursor-only admission for a host-consumed event: full identity and
    /// continuity validation without control, diagnostic or action effects.
    fn process_host_consumed_platform_event(
        &mut self,
        event: &PlatformEventV1,
    ) -> Result<(), PlayerInputError> {
        event.validate()?;
        if matches!(
            self.check_source_continuity(event)?,
            SourceContinuityV1::AlreadyAdmitted
        ) {
            return Ok(());
        }
        self.admit_source_cursor(event);
        Ok(())
    }

    fn process_platform_event(
        &mut self,
        event: &PlatformEventV1,
        disconnected_devices: &mut BTreeSet<(SchemaId, PersistentId)>,
    ) -> Result<(), PlayerInputError> {
        event.validate()?;
        if matches!(
            self.check_source_continuity(event)?,
            SourceContinuityV1::AlreadyAdmitted
        ) {
            return Ok(());
        }

        match (&event.kind, &event.payload) {
            (PlatformEventKindV1::Control, PlatformEventPayloadV1::Control(control)) => {
                if disconnected_devices
                    .contains(&(control.device_class.clone(), control.device_instance_nonce))
                {
                    Ok(())
                } else {
                    self.process_control(control)
                }
            }
            (
                PlatformEventKindV1::FocusChanged,
                PlatformEventPayloadV1::FocusChanged { focused: false },
            )
            | (PlatformEventKindV1::SuspendRequested, PlatformEventPayloadV1::Reason { .. }) => {
                self.cancel_all_active();
                Ok(())
            }
            (
                PlatformEventKindV1::DeviceDisconnected,
                PlatformEventPayloadV1::Device {
                    device_class,
                    device_instance_nonce,
                },
            ) => {
                disconnected_devices.insert((device_class.clone(), *device_instance_nonce));
                self.cancel_device(device_class, *device_instance_nonce);
                self.diagnostics
                    .insert(PlayerInputDiagnosticCodeV1::InputDeviceLost);
                Ok(())
            }
            (
                PlatformEventKindV1::DeviceConnected,
                PlatformEventPayloadV1::Device {
                    device_class,
                    device_instance_nonce,
                },
            ) => {
                disconnected_devices.remove(&(device_class.clone(), *device_instance_nonce));
                Ok(())
            }
            _ => Ok(()),
        }?;
        self.admit_source_cursor(event);
        Ok(())
    }

    fn check_source_continuity(
        &self,
        event: &PlatformEventV1,
    ) -> Result<SourceContinuityV1, PlayerInputError> {
        let source_key = PlatformSourceKey {
            host_instance_id: event.host_instance_id,
            source_class: event.source_class.clone(),
        };
        let Some(previous) = self.source_cursors.get(&source_key) else {
            return Ok(SourceContinuityV1::Admit);
        };
        match event.source_sequence.cmp(&previous.source_sequence) {
            std::cmp::Ordering::Equal if event.platform_event_id == previous.last_event_id => {
                Ok(SourceContinuityV1::AlreadyAdmitted)
            }
            std::cmp::Ordering::Equal => Err(PlayerInputError::PlatformEventIdentityCollision),
            std::cmp::Ordering::Less => Err(PlayerInputError::SourceSequenceNonMonotonic),
            std::cmp::Ordering::Greater
                if previous.source_sequence.checked_add(1) != Some(event.source_sequence) =>
            {
                Err(PlayerInputError::SourceSequenceGap)
            }
            std::cmp::Ordering::Greater => Ok(SourceContinuityV1::Admit),
        }
    }

    fn admit_source_cursor(&mut self, event: &PlatformEventV1) {
        self.source_cursors.insert(
            PlatformSourceKey {
                host_instance_id: event.host_instance_id,
                source_class: event.source_class.clone(),
            },
            PlatformSourceCursor {
                source_sequence: event.source_sequence,
                last_event_id: event.platform_event_id,
            },
        );
    }

    fn process_control(
        &mut self,
        control: &NormalizedControlEventV1,
    ) -> Result<(), PlayerInputError> {
        if matches!(
            control.phase,
            NormalizedControlPhaseV1::Completed | NormalizedControlPhaseV1::Cancelled
        ) {
            let prior_identities = self
                .held_controls
                .keys()
                .chain(self.started_controls.iter())
                .filter(|identity| identity.same_physical_control(control))
                .cloned()
                .collect::<BTreeSet<_>>();
            let affected_action_ids = {
                let bindings = matching_bindings(&self.action_map, control);
                for (_, binding) in &bindings {
                    validate_control_shape(control, binding.transform)?;
                }
                let mut affected_action_ids = bindings
                    .iter()
                    .map(|(action, _)| action.action_id.clone())
                    .collect::<BTreeSet<_>>();
                if control.phase == NormalizedControlPhaseV1::Cancelled {
                    for action in &self.action_map.actions {
                        if action.binding_slots.iter().any(|binding| {
                            prior_identities
                                .iter()
                                .any(|identity| binding_matches_identity(binding, identity))
                        }) {
                            affected_action_ids.insert(action.action_id.clone());
                        }
                    }
                }
                affected_action_ids
            };
            self.remove_held_physical_control(control);
            if control.phase == NormalizedControlPhaseV1::Cancelled {
                self.started_controls
                    .retain(|identity| !identity.same_physical_control(control));
                self.cancelled_action_ids.extend(affected_action_ids);
            }
            return Ok(());
        }
        let bindings = matching_bindings(&self.action_map, control);
        if bindings.is_empty() {
            return Ok(());
        }
        for (_, binding) in &bindings {
            validate_control_shape(control, binding.transform)?;
        }
        let has_delta_binding = bindings.iter().any(|(_, binding)| {
            matches!(
                binding.transform,
                ActionBindingTransformV1::Vector2PassthroughQ15 { .. }
            )
        });
        let is_active = control_is_active(control, &bindings);
        let identity = ControlIdentity::from_control(control);
        match control.phase {
            NormalizedControlPhaseV1::Started | NormalizedControlPhaseV1::Changed => {
                if has_delta_binding {
                    let delta = self.pending_deltas.entry(identity).or_insert([0, 0]);
                    delta[0] = delta[0]
                        .checked_add(i64::from(control.quantized_value[0]))
                        .ok_or(PlayerInputError::CounterOverflow)?;
                    delta[1] = delta[1]
                        .checked_add(i64::from(control.quantized_value[1]))
                        .ok_or(PlayerInputError::CounterOverflow)?;
                    return Ok(());
                }
                self.remove_physical_control(control);
                if is_active {
                    if control.phase == NormalizedControlPhaseV1::Started {
                        self.started_controls.insert(identity.clone());
                    }
                    self.held_controls
                        .insert(identity, control.quantized_value.clone());
                }
                Ok(())
            }
            NormalizedControlPhaseV1::Completed | NormalizedControlPhaseV1::Cancelled => {
                unreachable!("terminal controls return before active binding resolution")
            }
        }
    }

    fn remove_physical_control(&mut self, control: &NormalizedControlEventV1) {
        self.remove_held_physical_control(control);
        self.started_controls
            .retain(|identity| !identity.same_physical_control(control));
    }

    fn remove_held_physical_control(&mut self, control: &NormalizedControlEventV1) {
        self.held_controls
            .retain(|identity, _| !identity.same_physical_control(control));
    }

    fn cancel_all_active(&mut self) {
        self.cancelled_action_ids
            .extend(self.active_actions.keys().cloned());
        self.held_controls.clear();
        self.started_controls.clear();
        self.pending_deltas.clear();
    }

    fn cancel_device(&mut self, device_class: &SchemaId, device_instance_nonce: PersistentId) {
        let removed = self
            .held_controls
            .keys()
            .chain(self.started_controls.iter())
            .chain(self.pending_deltas.keys())
            .filter(|identity| {
                &identity.device_class == device_class
                    && identity.device_instance_nonce == device_instance_nonce
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        if !removed.is_empty() {
            for action in &self.action_map.actions {
                if action.binding_slots.iter().any(|binding| {
                    &binding.device_class == device_class
                        && removed
                            .iter()
                            .any(|identity| identity.control_path_id == binding.control_path_id)
                }) {
                    self.cancelled_action_ids.insert(action.action_id.clone());
                }
            }
            self.held_controls
                .retain(|identity, _| !removed.contains(identity));
            self.started_controls
                .retain(|identity| !removed.contains(identity));
            self.pending_deltas
                .retain(|identity, _| !removed.contains(identity));
            self.diagnostics
                .insert(PlayerInputDiagnosticCodeV1::InputDeviceLost);
        }
    }

    fn resolve_actions(&mut self) -> Result<Vec<PrioritizedPlayerAction>, PlayerInputError> {
        let mut current_stateful = BTreeMap::new();
        let mut emitted = Vec::new();

        for definition in &self.action_map.actions {
            let Some(context_priority) = self.context_stack.action_priority(&definition.action_id)
            else {
                continue;
            };
            if !definition.allowed_context_ids.iter().any(|context_id| {
                self.context_stack
                    .entries
                    .iter()
                    .any(|entry| &entry.context_id == context_id)
            }) {
                continue;
            }
            let resolution = resolve_definition(
                definition,
                &self.held_controls,
                &self.started_controls,
                &self.pending_deltas,
            )?;
            let Some((value, stateful, pulse)) = resolution else {
                continue;
            };
            if stateful {
                current_stateful.insert(definition.action_id.clone(), value);
            }
            let previous = self.active_actions.get(&definition.action_id);
            let phase = if pulse {
                PlayerActionPhaseV1::Started
            } else if !stateful {
                PlayerActionPhaseV1::Performed
            } else if previous.is_none() {
                PlayerActionPhaseV1::Started
            } else if previous != Some(&value)
                || definition
                    .allowed_phases
                    .binary_search(&PlayerActionPhaseV1::Performed)
                    .is_ok()
            {
                PlayerActionPhaseV1::Performed
            } else {
                continue;
            };
            if definition.allowed_phases.binary_search(&phase).is_err() {
                return Err(PlayerInputError::BindingConflict);
            }
            emitted.push(PrioritizedPlayerAction {
                context_priority,
                action: PlayerActionV1 {
                    action_id: definition.action_id.clone(),
                    phase,
                    value,
                    semantic_occurrence_ordinal: 0,
                },
            });
        }

        for (action_id, previous) in &self.active_actions {
            if current_stateful.contains_key(action_id) {
                continue;
            }
            let phase = if self.cancelled_action_ids.contains(action_id) {
                PlayerActionPhaseV1::Cancelled
            } else {
                PlayerActionPhaseV1::Completed
            };
            if let Some(definition) = self.action_map.action(action_id)
                && let Some(context_priority) = self.context_stack.action_priority(action_id)
                && definition.allowed_phases.binary_search(&phase).is_ok()
            {
                emitted.push(PrioritizedPlayerAction {
                    context_priority,
                    action: PlayerActionV1 {
                        action_id: action_id.clone(),
                        phase,
                        value: neutral_value(*previous),
                        semantic_occurrence_ordinal: 0,
                    },
                });
            }
        }

        if let Some(next_context_stack) = self.pending_context_stack.as_ref() {
            let transition_cancellations = current_stateful
                .keys()
                .filter(|action_id| !next_context_stack.allows_action(action_id))
                .cloned()
                .collect::<Vec<_>>();
            for action_id in transition_cancellations {
                let previous = current_stateful
                    .remove(&action_id)
                    .ok_or(PlayerInputError::BindingConflict)?;
                let definition = self
                    .action_map
                    .action(&action_id)
                    .ok_or(PlayerInputError::BindingConflict)?;
                if definition
                    .allowed_phases
                    .binary_search(&PlayerActionPhaseV1::Cancelled)
                    .is_err()
                {
                    return Err(PlayerInputError::BindingConflict);
                }
                let context_priority = self
                    .context_stack
                    .action_priority(&action_id)
                    .ok_or(PlayerInputError::BindingConflict)?;
                emitted.push(PrioritizedPlayerAction {
                    context_priority,
                    action: PlayerActionV1 {
                        action_id,
                        phase: PlayerActionPhaseV1::Cancelled,
                        value: neutral_value(previous),
                        semantic_occurrence_ordinal: 0,
                    },
                });
            }
        }

        // An action-map revision owns binding semantics as well as phase
        // policy. Do not carry stateful actions across that revision boundary:
        // close them under the exact old map captured by this frame, then let
        // still-held controls start again under the new map on the next frame.
        if self.pending_action_map.is_some() {
            let revision_cancellations = current_stateful
                .iter()
                .map(|(action_id, value)| (action_id.clone(), *value))
                .collect::<Vec<_>>();
            current_stateful.clear();
            for (action_id, previous) in revision_cancellations {
                let definition = self
                    .action_map
                    .action(&action_id)
                    .ok_or(PlayerInputError::BindingConflict)?;
                if definition
                    .allowed_phases
                    .binary_search(&PlayerActionPhaseV1::Cancelled)
                    .is_err()
                {
                    return Err(PlayerInputError::BindingConflict);
                }
                let context_priority = self
                    .context_stack
                    .action_priority(&action_id)
                    .ok_or(PlayerInputError::BindingConflict)?;
                emitted.push(PrioritizedPlayerAction {
                    context_priority,
                    action: PlayerActionV1 {
                        action_id,
                        phase: PlayerActionPhaseV1::Cancelled,
                        value: neutral_value(previous),
                        semantic_occurrence_ordinal: 0,
                    },
                });
            }
        }

        self.active_actions = current_stateful;
        Ok(emitted)
    }
}

#[cfg(test)]
mod tests;
