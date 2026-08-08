use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, SessionRequestId, SessionTransitionId};
use next_contracts::session::{
    ApplicationLifecycleEventV1, ApplicationLifecycleRequestV1, ApplicationSessionManifestV1,
    ApplicationSessionStateV1, ApplicationSessionStatusV1, SessionContractError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchivedLifecycleRequestV1 {
    pub request_id: SessionRequestId,
    pub canonical_request_hash: ContentHash,
    pub canonical_request_bytes: Vec<u8>,
    pub event: ApplicationLifecycleEventV1,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SessionTransitionReferencesV1 {
    pub activation_receipt_hash: Option<ContentHash>,
    pub save_receipt_hash: Option<ContentHash>,
    pub diagnostic_hash: Option<ContentHash>,
    pub terminal_receipt_hash: Option<ContentHash>,
    pub active_runtime_revision: Option<u64>,
    pub active_save_generation_hash: Option<ContentHash>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionTransitionPlanV1 {
    Publish {
        prior_state_hash: ContentHash,
        request: ApplicationLifecycleRequestV1,
        event: ApplicationLifecycleEventV1,
        next_state: Box<ApplicationSessionStateV1>,
    },
    ExactRetry {
        event: ApplicationLifecycleEventV1,
        current_state: ApplicationSessionStateV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStatePublicationPlanV1 {
    pub prior_state_hash: ContentHash,
    pub next_state: ApplicationSessionStateV1,
}

#[derive(Clone, Debug)]
pub struct ApplicationSessionMachine {
    manifest: ApplicationSessionManifestV1,
    state: ApplicationSessionStateV1,
    requests: BTreeMap<SessionRequestId, ArchivedLifecycleRequestV1>,
}

impl ApplicationSessionMachine {
    pub fn new(manifest: ApplicationSessionManifestV1) -> Result<Self, SessionMachineError> {
        manifest.validate()?;
        let state = ApplicationSessionStateV1::created(&manifest);
        Ok(Self {
            manifest,
            state,
            requests: BTreeMap::new(),
        })
    }

    pub fn restore(
        manifest: ApplicationSessionManifestV1,
        state: ApplicationSessionStateV1,
        archived_requests: Vec<ArchivedLifecycleRequestV1>,
    ) -> Result<Self, SessionMachineError> {
        manifest.validate()?;
        state.validate()?;
        if manifest.body.session_id != state.session_id
            || manifest.canonical_hash != state.application_session_manifest_hash
            || manifest.body.project_composition_lock_hash != state.project_composition_lock_hash
        {
            return Err(SessionMachineError::ManifestStateMismatch);
        }
        let mut requests = BTreeMap::new();
        let mut history = Vec::new();
        for archived in archived_requests {
            let request = ApplicationLifecycleRequestV1::from_jcs_bytes(
                &archived.canonical_request_bytes,
                CanonicalDecodeLimits::default(),
            )?;
            let event = ApplicationLifecycleEventV1::from_jcs_bytes(
                &archived.event.canonical_bytes(),
                &request,
                CanonicalDecodeLimits::default(),
            )?;
            if archived.event.request_id != archived.request_id
                || archived.event.session_id != state.session_id
                || request.request_id != archived.request_id
                || request.canonical_hash != archived.canonical_request_hash
                || request.session_id != state.session_id
                || event != archived.event
                || event.transition_id != derive_transition_id(&request)
                || event.after_revision > state.revision
                || requests.insert(archived.request_id, archived).is_some()
            {
                return Err(SessionMachineError::ArchiveInvalid);
            }
            history.push(event);
        }
        history.sort_by_key(|event| event.before_revision);
        let expected_history_len =
            usize::try_from(state.revision).map_err(|_| SessionMachineError::ArchiveInvalid)?;
        if history.len() != expected_history_len {
            return Err(SessionMachineError::ArchiveInvalid);
        }
        if history.is_empty() {
            if state.state != ApplicationSessionStatusV1::Created
                || state.revision != 0
                || state.last_transition_id.is_some()
            {
                return Err(SessionMachineError::ArchiveInvalid);
            }
        } else {
            for (revision, event) in history.iter().enumerate() {
                let before_revision =
                    u64::try_from(revision).map_err(|_| SessionMachineError::ArchiveInvalid)?;
                let after_revision = before_revision
                    .checked_add(1)
                    .ok_or(SessionMachineError::ArchiveInvalid)?;
                if event.before_revision != before_revision
                    || event.after_revision != after_revision
                    || (before_revision == 0
                        && event.from_state != ApplicationSessionStatusV1::Created)
                {
                    return Err(SessionMachineError::ArchiveInvalid);
                }
            }
            if history.windows(2).any(|pair| {
                pair[0].after_revision != pair[1].before_revision
                    || pair[0].to_state != pair[1].from_state
            }) {
                return Err(SessionMachineError::ArchiveInvalid);
            }
            let last = history.last().expect("non-empty history was checked above");
            if last.after_revision != state.revision
                || last.to_state != state.state
                || state.last_transition_id != Some(last.transition_id)
            {
                return Err(SessionMachineError::ArchiveInvalid);
            }
        }
        Ok(Self {
            manifest,
            state,
            requests,
        })
    }

    #[must_use]
    pub const fn manifest(&self) -> &ApplicationSessionManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub const fn state(&self) -> &ApplicationSessionStateV1 {
        &self.state
    }

    #[must_use]
    pub fn archived_requests(&self) -> Vec<ArchivedLifecycleRequestV1> {
        self.requests.values().cloned().collect()
    }

    pub fn plan_transition(
        &self,
        request: ApplicationLifecycleRequestV1,
        references: SessionTransitionReferencesV1,
    ) -> Result<SessionTransitionPlanV1, SessionMachineError> {
        let canonical_bytes = request.canonical_bytes();
        if let Some(archived) = self.requests.get(&request.request_id) {
            if archived.canonical_request_hash != request.canonical_hash
                || archived.canonical_request_bytes != canonical_bytes
            {
                return Err(SessionMachineError::RequestIdentityCollision);
            }
            return Ok(SessionTransitionPlanV1::ExactRetry {
                event: archived.event.clone(),
                current_state: self.state.clone(),
            });
        }

        request.validate()?;
        if request.session_id != self.state.session_id {
            return Err(SessionMachineError::SessionMismatch);
        }
        if request.expected_revision != self.state.revision {
            return Err(SessionMachineError::ExpectedRevisionStale {
                expected: request.expected_revision,
                actual: self.state.revision,
            });
        }
        if request.expected_state != self.state.state {
            return Err(SessionMachineError::ExpectedStateStale {
                expected: request.expected_state,
                actual: self.state.state,
            });
        }
        let transition_id = derive_transition_id(&request);
        let event = ApplicationLifecycleEventV1::committed(
            transition_id,
            &request,
            references.activation_receipt_hash,
            references.save_receipt_hash,
            references.diagnostic_hash,
        )?;
        let mut next_state = event.next_state(&self.state, references.terminal_receipt_hash)?;
        if let Some(revision) = references.active_runtime_revision {
            next_state = next_state.with_active_runtime_revision(revision);
        }
        if let Some(generation_hash) = references.active_save_generation_hash {
            next_state = next_state.with_active_save_generation(generation_hash);
        }
        Ok(SessionTransitionPlanV1::Publish {
            prior_state_hash: self.state.canonical_hash,
            request,
            event,
            next_state: Box::new(next_state),
        })
    }

    pub fn plan_state_publication(
        &self,
        active_runtime_revision: Option<u64>,
        active_save_generation_hash: Option<ContentHash>,
    ) -> Result<SessionStatePublicationPlanV1, SessionMachineError> {
        if active_runtime_revision
            .zip(self.state.active_runtime_revision)
            .is_some_and(|(next, current)| next < current)
        {
            return Err(SessionMachineError::ObservationRegression);
        }
        let mut next_state = self.state.clone();
        if let Some(revision) = active_runtime_revision {
            next_state = next_state.with_active_runtime_revision(revision);
        }
        if let Some(generation_hash) = active_save_generation_hash {
            next_state = next_state.with_active_save_generation(generation_hash);
        }
        next_state.validate()?;
        Ok(SessionStatePublicationPlanV1 {
            prior_state_hash: self.state.canonical_hash,
            next_state,
        })
    }

    /// Plans one atomic observation replacement for an explicit user save
    /// load. Unlike ordinary runtime observations, a loaded save may carry an
    /// older authoritative revision; the verified save-generation identity is
    /// therefore required in the same publication, and loading is admitted
    /// only while the application session is suspended.
    pub fn plan_save_load_publication(
        &self,
        active_runtime_revision: u64,
        active_save_generation_hash: ContentHash,
    ) -> Result<SessionStatePublicationPlanV1, SessionMachineError> {
        if self.state.state != ApplicationSessionStatusV1::Suspended {
            return Err(SessionMachineError::ObservationRegression);
        }
        let next_state = self
            .state
            .with_active_runtime_revision(active_runtime_revision)
            .with_active_save_generation(active_save_generation_hash);
        next_state.validate()?;
        Ok(SessionStatePublicationPlanV1 {
            prior_state_hash: self.state.canonical_hash,
            next_state,
        })
    }

    pub fn commit_state_publication(&mut self, plan: SessionStatePublicationPlanV1) {
        assert_eq!(
            plan.prior_state_hash, self.state.canonical_hash,
            "durably published session-state plan must match in-memory prior state"
        );
        assert_eq!(
            plan.next_state.revision, self.state.revision,
            "observation publication does not create a lifecycle revision"
        );
        self.state = plan.next_state;
    }

    pub fn commit(&mut self, plan: SessionTransitionPlanV1) -> ApplicationLifecycleEventV1 {
        let SessionTransitionPlanV1::Publish {
            prior_state_hash,
            request,
            event,
            next_state,
        } = plan
        else {
            panic!("an exact retry has no in-memory commit");
        };
        assert_eq!(
            prior_state_hash, self.state.canonical_hash,
            "durably published transition plan must match in-memory prior state"
        );
        assert_eq!(
            next_state.revision,
            self.state.revision + 1,
            "session commit advances exactly one revision"
        );
        let archived = ArchivedLifecycleRequestV1 {
            request_id: request.request_id,
            canonical_request_hash: request.canonical_hash,
            canonical_request_bytes: request.canonical_bytes(),
            event: event.clone(),
        };
        assert!(
            self.requests.insert(request.request_id, archived).is_none(),
            "a new transition plan cannot replace an archived request"
        );
        self.state = *next_state;
        event
    }
}

fn derive_transition_id(request: &ApplicationLifecycleRequestV1) -> SessionTransitionId {
    let mut preimage = b"nextengine.session-transition-id.v1\0".to_vec();
    preimage.extend_from_slice(request.session_id.as_bytes());
    preimage.extend_from_slice(request.request_id.as_bytes());
    preimage.extend_from_slice(request.canonical_hash.as_bytes());
    let hash = sha256(&preimage);
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    SessionTransitionId::from_bytes(bytes)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionMachineError {
    Contract(SessionContractError),
    ManifestStateMismatch,
    ArchiveInvalid,
    SessionMismatch,
    ExpectedRevisionStale {
        expected: u64,
        actual: u64,
    },
    ExpectedStateStale {
        expected: ApplicationSessionStatusV1,
        actual: ApplicationSessionStatusV1,
    },
    RequestIdentityCollision,
    ObservationRegression,
}

impl SessionMachineError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::ExpectedRevisionStale { .. } | Self::ExpectedStateStale { .. } => {
                "SESSION_EXPECTED_REVISION_STALE"
            }
            Self::RequestIdentityCollision => "SESSION_REQUEST_IDENTITY_COLLISION",
            Self::ObservationRegression => "SESSION_TRANSITION_INVALID",
            Self::Contract(_)
            | Self::ManifestStateMismatch
            | Self::ArchiveInvalid
            | Self::SessionMismatch => "SESSION_TRANSITION_INVALID",
        }
    }
}

impl Display for SessionMachineError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::ManifestStateMismatch => {
                formatter.write_str("session manifest and state do not match")
            }
            Self::ArchiveInvalid => formatter.write_str("session request archive is invalid"),
            Self::SessionMismatch => formatter.write_str("session request targets another session"),
            Self::ExpectedRevisionStale { expected, actual } => {
                write!(
                    formatter,
                    "expected session revision {expected}, actual {actual}"
                )
            }
            Self::ExpectedStateStale { expected, actual } => {
                write!(
                    formatter,
                    "expected session state {expected:?}, actual {actual:?}"
                )
            }
            Self::RequestIdentityCollision => {
                formatter.write_str("session request ID has different canonical bytes")
            }
            Self::ObservationRegression => {
                formatter.write_str("session runtime observation regresses")
            }
        }
    }
}

impl Error for SessionMachineError {}

impl From<SessionContractError> for SessionMachineError {
    fn from(value: SessionContractError) -> Self {
        Self::Contract(value)
    }
}

#[cfg(test)]
mod tests;
