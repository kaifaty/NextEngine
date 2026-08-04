use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::platform::{
    PlatformCapabilitySetV1, PlatformContractError, PlatformEventKindV1, PlatformEventV1,
    PresentationTargetKindV1,
};
use next_contracts::session::{
    ApplicationLifecycleRequestV1, ApplicationSessionStatusV1, CausalInputSourceKindV1,
    CloseSessionRequestV1,
};

use crate::ApplicationError;

use super::ApplicationCoordinator;

static PLATFORM_HOST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) const MAXIMUM_PLATFORM_EVENT_SOURCES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlatformSourceCursorV1 {
    source_sequence: u64,
    event_id: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RegisteredPlatformHostV1 {
    pub host_instance_id: PersistentId,
    pub capability_set_hash: ContentHash,
    source_cursors: BTreeMap<next_contracts::ids::SchemaId, PlatformSourceCursorV1>,
}

impl ApplicationCoordinator {
    /// Registers one fresh platform-adapter lifetime against this application
    /// session. The capability descriptor is already immutable launch input;
    /// registration binds its exact hash to an ephemeral host identity.
    pub fn register_platform_host(
        &mut self,
        capabilities: &PlatformCapabilitySetV1,
    ) -> Result<PersistentId, ApplicationError> {
        capabilities.validate()?;
        if self.launch.presentation_target == PresentationTargetKindV1::None {
            return Err(ApplicationError::ProjectTargetForbidden);
        }
        if self.durable.manifest.body.platform_capability_set_hash
            != Some(capabilities.canonical_hash)
        {
            return Err(ApplicationError::PlatformCapabilityRequired);
        }
        let host_instance_id = fresh_platform_host_instance_id(
            self.machine.state().session_id.as_bytes(),
            self.current_generation.as_bytes(),
        );
        self.platform_host = Some(RegisteredPlatformHostV1 {
            host_instance_id,
            capability_set_hash: capabilities.canonical_hash,
            source_cursors: BTreeMap::new(),
        });
        Ok(host_instance_id)
    }

    /// Same-session in-process live-run reload (S5 pause-menu load): the
    /// adapter lifetime outlives the coordinator, so its event stream keeps
    /// the already-issued host identity instead of a fresh registration.
    /// Same capability binding; admission cursors start empty and the
    /// continuing stream re-establishes them. Stale adapters from earlier
    /// process lifetimes are still rejected because their identity never
    /// matches the current registration.
    pub fn register_platform_host_continuing(
        &mut self,
        host_instance_id: PersistentId,
        capabilities: &PlatformCapabilitySetV1,
    ) -> Result<PersistentId, ApplicationError> {
        capabilities.validate()?;
        if self.launch.presentation_target == PresentationTargetKindV1::None {
            return Err(ApplicationError::ProjectTargetForbidden);
        }
        if self.durable.manifest.body.platform_capability_set_hash
            != Some(capabilities.canonical_hash)
        {
            return Err(ApplicationError::PlatformCapabilityRequired);
        }
        self.platform_host = Some(RegisteredPlatformHostV1 {
            host_instance_id,
            capability_set_hash: capabilities.canonical_hash,
            source_cursors: BTreeMap::new(),
        });
        Ok(host_instance_id)
    }

    pub(crate) fn with_platform_event_admission<T>(
        &mut self,
        events: &[PlatformEventV1],
        lifecycle_plan: &[PlatformEventV1],
        operation: impl FnOnce(&mut Self) -> Result<T, ApplicationError>,
    ) -> Result<T, ApplicationError> {
        let mut ordered = events.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| {
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
        let mut candidate = self.platform_host.clone();
        for event in ordered {
            self.validate_platform_event_against(event, &mut candidate)?;
        }
        self.validate_platform_lifecycle_plan(lifecycle_plan)?;
        let result = operation(self)?;
        self.platform_host = candidate;
        Ok(result)
    }

    pub(crate) fn validate_platform_lifecycle_plan(
        &self,
        events: &[PlatformEventV1],
    ) -> Result<(), ApplicationError> {
        let mut ordered = events
            .iter()
            .filter(|event| {
                matches!(
                    event.kind,
                    PlatformEventKindV1::SuspendRequested | PlatformEventKindV1::ResumeRequested
                )
            })
            .collect::<Vec<_>>();
        ordered.sort_by(|left, right| {
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
        let mut state = self.machine.state().state;
        let mut prior_event_id = None;
        for event in ordered {
            if prior_event_id == Some(event.platform_event_id) {
                continue;
            }
            prior_event_id = Some(event.platform_event_id);
            if self.is_exact_archived_platform_event(event)? {
                continue;
            }
            state = match (state, event.kind) {
                (ApplicationSessionStatusV1::Active, PlatformEventKindV1::SuspendRequested) => {
                    ApplicationSessionStatusV1::Suspended
                }
                (ApplicationSessionStatusV1::Suspended, PlatformEventKindV1::ResumeRequested) => {
                    ApplicationSessionStatusV1::Active
                }
                _ => return Err(ApplicationError::CloseStateInvalid),
            };
        }
        Ok(())
    }

    fn validate_platform_event_against(
        &self,
        event: &PlatformEventV1,
        candidate: &mut Option<RegisteredPlatformHostV1>,
    ) -> Result<(), ApplicationError> {
        event.validate()?;
        if self.launch.presentation_target == PresentationTargetKindV1::None {
            return Err(ApplicationError::ProjectTargetForbidden);
        }
        if self.is_exact_archived_platform_event(event)? {
            return Ok(());
        }
        let binding = candidate
            .as_mut()
            .ok_or(ApplicationError::PlatformCapabilityRequired)?;
        if event.host_instance_id != binding.host_instance_id
            || event.capability_set_hash != binding.capability_set_hash
        {
            return Err(ApplicationError::PlatformEventIdentityCollision);
        }
        if let Some(previous) = binding.source_cursors.get(&event.source_class) {
            match event.source_sequence.cmp(&previous.source_sequence) {
                std::cmp::Ordering::Equal if event.platform_event_id == previous.event_id => {
                    return Ok(());
                }
                std::cmp::Ordering::Equal => {
                    return Err(ApplicationError::PlatformEventIdentityCollision);
                }
                std::cmp::Ordering::Less => {
                    return Err(ApplicationError::PlatformEventSequenceInvalid);
                }
                std::cmp::Ordering::Greater
                    if previous.source_sequence.checked_add(1) != Some(event.source_sequence) =>
                {
                    return Err(ApplicationError::PlatformEventSequenceInvalid);
                }
                std::cmp::Ordering::Greater => {}
            }
        }
        if !binding.source_cursors.contains_key(&event.source_class)
            && binding.source_cursors.len() == MAXIMUM_PLATFORM_EVENT_SOURCES
        {
            return Err(PlatformContractError::LimitExceeded {
                actual: binding.source_cursors.len() + 1,
                limit: MAXIMUM_PLATFORM_EVENT_SOURCES,
            }
            .into());
        }
        binding.source_cursors.insert(
            event.source_class.clone(),
            PlatformSourceCursorV1 {
                source_sequence: event.source_sequence,
                event_id: event.platform_event_id,
            },
        );
        Ok(())
    }

    fn is_exact_archived_platform_event(
        &self,
        event: &PlatformEventV1,
    ) -> Result<bool, ApplicationError> {
        for archived in self.machine.archived_requests() {
            let request = ApplicationLifecycleRequestV1::from_jcs_bytes(
                &archived.canonical_request_bytes,
                CanonicalDecodeLimits::default(),
            )?;
            if request.causal_input_reference.source_kind == CausalInputSourceKindV1::PlatformEvent
                && request.causal_input_reference.canonical_hash == event.platform_event_id
                && lifecycle_kind_matches(&request, event.kind)
            {
                return Ok(true);
            }
        }
        let Some(close) = self.durable.close.as_ref() else {
            return Ok(false);
        };
        let request = CloseSessionRequestV1::from_canonical_bytes(
            &close.canonical_close_request_bytes,
            CanonicalDecodeLimits::default(),
        )?;
        Ok(
            request.causal_input_reference.source_kind == CausalInputSourceKindV1::PlatformEvent
                && request.causal_input_reference.canonical_hash == event.platform_event_id
                && event.kind == PlatformEventKindV1::CloseRequested,
        )
    }
}

fn lifecycle_kind_matches(
    request: &ApplicationLifecycleRequestV1,
    event_kind: PlatformEventKindV1,
) -> bool {
    matches!(
        (request.expected_state, request.requested_state, event_kind),
        (
            ApplicationSessionStatusV1::Active,
            ApplicationSessionStatusV1::Suspended,
            PlatformEventKindV1::SuspendRequested
        ) | (
            ApplicationSessionStatusV1::Suspended,
            ApplicationSessionStatusV1::Active,
            PlatformEventKindV1::ResumeRequested
        ) | (
            ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended,
            ApplicationSessionStatusV1::Quiescing,
            PlatformEventKindV1::CloseRequested
        )
    )
}

fn fresh_platform_host_instance_id(
    session_id: &[u8; 16],
    current_generation: &[u8; 32],
) -> PersistentId {
    let mut preimage = Vec::with_capacity(16 + 32 + 4 + 16 + 8);
    preimage.extend_from_slice(session_id);
    preimage.extend_from_slice(current_generation);
    preimage.extend_from_slice(&std::process::id().to_le_bytes());
    preimage.extend_from_slice(
        &SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_le_bytes(),
    );
    preimage.extend_from_slice(
        &PLATFORM_HOST_SEQUENCE
            .fetch_add(1, Ordering::Relaxed)
            .to_le_bytes(),
    );
    let digest = sha256(&preimage);
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    PersistentId::from_bytes(bytes)
}
