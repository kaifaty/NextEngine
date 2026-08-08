use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{ApplicationSessionId, ContentHash, SessionRequestId, SessionTransitionId};
use crate::manifest_jcs::JcsValue;

use super::codec::{
    decoded_object, encoded, hash, nested_object, number, object, optional_hash,
    optional_hash_value, optional_transition_id, reject_unknown, request_id, session_hash,
    session_id, string, take, text, transition_id, u32_value, u64_value,
};
use super::{
    APPLICATION_SESSION_MANIFEST_FORMAT_V2, APPLICATION_SESSION_SCHEMA_VERSION,
    ApplicationSessionStatusV1, CausalInputReferenceV1, CausalInputSourceKindV1, CompositionRootV1,
    LifecycleReasonKindV1, LifecycleReasonV1, PresentationTargetKindV1, SessionContractError,
    validate_root_target,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSessionManifestBodyV2 {
    pub session_id: ApplicationSessionId,
    pub composition_root: CompositionRootV1,
    pub project_composition_lock_hash: ContentHash,
    pub launch_profile_hash: ContentHash,
    pub platform_capability_set_hash: Option<ContentHash>,
    pub runtime_determinism_profile_hash: ContentHash,
    pub schema_registry_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub presentation_target_kind: PresentationTargetKindV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSessionManifestV2 {
    pub schema_version: u32,
    pub body: ApplicationSessionManifestBodyV2,
    pub canonical_hash: ContentHash,
}

impl ApplicationSessionManifestV2 {
    pub fn new(body: ApplicationSessionManifestBodyV2) -> Result<Self, SessionContractError> {
        validate_root_target(body.composition_root, body.presentation_target_kind)?;
        if body.composition_root == CompositionRootV1::Game
            && body.presentation_target_kind == PresentationTargetKindV1::Interactive
            && body.platform_capability_set_hash.is_none()
        {
            return Err(SessionContractError::InvalidRootTarget);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            body,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash =
            session_hash(APPLICATION_SESSION_MANIFEST_FORMAT_V2, &value.body_value());
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if Self::new(self.body.clone())?.canonical_hash != self.canonical_hash {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    #[must_use]
    pub fn to_jcs_bytes(&self) -> Vec<u8> {
        encoded(&self.body_value())
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SessionContractError> {
        let mut value = decoded_object(bytes, limits, "application_session_manifest")?;
        let format = text(take(&mut value, "manifest_format")?, "manifest_format")?;
        if format != APPLICATION_SESSION_MANIFEST_FORMAT_V2 {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let body = ApplicationSessionManifestBodyV2 {
            session_id: session_id(take(&mut value, "session_id")?, "session_id")?,
            composition_root: CompositionRootV1::parse(&text(
                take(&mut value, "composition_root")?,
                "composition_root",
            )?)?,
            project_composition_lock_hash: hash(
                take(&mut value, "project_composition_lock_hash")?,
                "project_composition_lock_hash",
            )?,
            launch_profile_hash: hash(
                take(&mut value, "launch_profile_hash")?,
                "launch_profile_hash",
            )?,
            platform_capability_set_hash: optional_hash_value(
                take(&mut value, "platform_capability_set_hash_or_none")?,
                "platform_capability_set_hash_or_none",
            )?,
            runtime_determinism_profile_hash: hash(
                take(&mut value, "runtime_determinism_profile_hash")?,
                "runtime_determinism_profile_hash",
            )?,
            schema_registry_hash: hash(
                take(&mut value, "schema_registry_hash")?,
                "schema_registry_hash",
            )?,
            content_manifest_hash: hash(
                take(&mut value, "content_manifest_hash")?,
                "content_manifest_hash",
            )?,
            presentation_target_kind: PresentationTargetKindV1::parse(&text(
                take(&mut value, "presentation_target_kind")?,
                "presentation_target_kind",
            )?)
            .map_err(|_| SessionContractError::UnknownClosedValue)?,
        };
        reject_unknown(value)?;
        let manifest = Self::new(body)?;
        if manifest.to_jcs_bytes() != bytes {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(manifest)
    }

    fn body_value(&self) -> JcsValue {
        object([
            (
                "composition_root",
                string(self.body.composition_root.token()),
            ),
            (
                "content_manifest_hash",
                string(self.body.content_manifest_hash.to_hex()),
            ),
            (
                "launch_profile_hash",
                string(self.body.launch_profile_hash.to_hex()),
            ),
            (
                "manifest_format",
                string(APPLICATION_SESSION_MANIFEST_FORMAT_V2),
            ),
            (
                "platform_capability_set_hash_or_none",
                optional_hash(self.body.platform_capability_set_hash),
            ),
            (
                "presentation_target_kind",
                string(self.body.presentation_target_kind.token()),
            ),
            (
                "project_composition_lock_hash",
                string(self.body.project_composition_lock_hash.to_hex()),
            ),
            (
                "runtime_determinism_profile_hash",
                string(self.body.runtime_determinism_profile_hash.to_hex()),
            ),
            (
                "schema_registry_hash",
                string(self.body.schema_registry_hash.to_hex()),
            ),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.body.session_id.to_hex())),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSessionStateV2 {
    pub schema_version: u32,
    pub session_id: ApplicationSessionId,
    pub state: ApplicationSessionStatusV1,
    pub revision: u64,
    pub application_session_manifest_hash: ContentHash,
    pub project_composition_lock_hash: ContentHash,
    pub active_runtime_revision: Option<u64>,
    pub active_save_generation_hash: Option<ContentHash>,
    pub last_transition_id: Option<SessionTransitionId>,
    pub terminal_receipt_hash: Option<ContentHash>,
    pub canonical_hash: ContentHash,
}

impl ApplicationSessionStateV2 {
    #[must_use]
    pub fn created(manifest: &ApplicationSessionManifestV2) -> Self {
        Self::new_unchecked(
            manifest.body.session_id,
            ApplicationSessionStatusV1::Created,
            0,
            manifest.canonical_hash,
            manifest.body.project_composition_lock_hash,
            None,
            None,
            None,
            None,
        )
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if (self.state == ApplicationSessionStatusV1::Created && self.revision != 0)
            || (self.state == ApplicationSessionStatusV1::Closed)
                != self.terminal_receipt_hash.is_some()
        {
            return Err(SessionContractError::InvalidStateFields);
        }
        let canonical = Self::new_unchecked(
            self.session_id,
            self.state,
            self.revision,
            self.application_session_manifest_hash,
            self.project_composition_lock_hash,
            self.active_runtime_revision,
            self.active_save_generation_hash,
            self.last_transition_id,
            self.terminal_receipt_hash,
        );
        if canonical.canonical_hash != self.canonical_hash {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    #[must_use]
    pub fn to_jcs_bytes(&self) -> Vec<u8> {
        encoded(&self.body_value())
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SessionContractError> {
        let mut value = decoded_object(bytes, limits, "application_session_state")?;
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let active_revision = text(
            take(&mut value, "active_runtime_revision_or_none")?,
            "active_runtime_revision_or_none",
        )?;
        let state = Self::new_unchecked(
            session_id(take(&mut value, "session_id")?, "session_id")?,
            parse_session_status(&text(take(&mut value, "state")?, "state")?)?,
            u64_value(take(&mut value, "revision")?, "revision")?,
            hash(
                take(&mut value, "application_session_manifest_hash")?,
                "application_session_manifest_hash",
            )?,
            hash(
                take(&mut value, "project_composition_lock_hash")?,
                "project_composition_lock_hash",
            )?,
            if active_revision == "none" {
                None
            } else {
                Some(
                    active_revision
                        .parse::<u64>()
                        .map_err(|_| SessionContractError::InvalidStateFields)?,
                )
            },
            optional_hash_value(
                take(&mut value, "active_save_generation_hash_or_none")?,
                "active_save_generation_hash_or_none",
            )?,
            optional_transition_id(
                take(&mut value, "last_transition_id_or_none")?,
                "last_transition_id_or_none",
            )?,
            optional_hash_value(
                take(&mut value, "terminal_receipt_hash_or_none")?,
                "terminal_receipt_hash_or_none",
            )?,
        );
        reject_unknown(value)?;
        state.validate()?;
        if state.to_jcs_bytes() != bytes {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(state)
    }

    #[must_use]
    pub fn with_active_runtime_revision(&self, revision: u64) -> Self {
        Self::new_unchecked(
            self.session_id,
            self.state,
            self.revision,
            self.application_session_manifest_hash,
            self.project_composition_lock_hash,
            Some(revision),
            self.active_save_generation_hash,
            self.last_transition_id,
            self.terminal_receipt_hash,
        )
    }

    #[must_use]
    pub fn with_active_save_generation(&self, generation_hash: ContentHash) -> Self {
        Self::new_unchecked(
            self.session_id,
            self.state,
            self.revision,
            self.application_session_manifest_hash,
            self.project_composition_lock_hash,
            self.active_runtime_revision,
            Some(generation_hash),
            self.last_transition_id,
            self.terminal_receipt_hash,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the immutable session state contract keeps each authority field explicit"
    )]
    pub(crate) fn new_unchecked(
        session_id: ApplicationSessionId,
        state: ApplicationSessionStatusV1,
        revision: u64,
        application_session_manifest_hash: ContentHash,
        project_composition_lock_hash: ContentHash,
        active_runtime_revision: Option<u64>,
        active_save_generation_hash: Option<ContentHash>,
        last_transition_id: Option<SessionTransitionId>,
        terminal_receipt_hash: Option<ContentHash>,
    ) -> Self {
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            session_id,
            state,
            revision,
            application_session_manifest_hash,
            project_composition_lock_hash,
            active_runtime_revision,
            active_save_generation_hash,
            last_transition_id,
            terminal_receipt_hash,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash(
            "nextengine.application-session-state.v2",
            &value.body_value(),
        );
        value
    }

    fn body_value(&self) -> JcsValue {
        object([
            (
                "active_runtime_revision_or_none",
                self.active_runtime_revision
                    .map_or_else(|| string("none"), |value| string(value.to_string())),
            ),
            (
                "active_save_generation_hash_or_none",
                optional_hash(self.active_save_generation_hash),
            ),
            (
                "application_session_manifest_hash",
                string(self.application_session_manifest_hash.to_hex()),
            ),
            (
                "last_transition_id_or_none",
                self.last_transition_id
                    .map_or_else(|| string("none"), |value| string(value.to_hex())),
            ),
            (
                "project_composition_lock_hash",
                string(self.project_composition_lock_hash.to_hex()),
            ),
            ("revision", number(self.revision)),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
            ("state", string(self.state.token())),
            (
                "terminal_receipt_hash_or_none",
                optional_hash(self.terminal_receipt_hash),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationLifecycleRequestV2 {
    pub schema_version: u32,
    pub request_id: SessionRequestId,
    pub session_id: ApplicationSessionId,
    pub expected_revision: u64,
    pub expected_state: ApplicationSessionStatusV1,
    pub requested_state: ApplicationSessionStatusV1,
    pub reason: LifecycleReasonV1,
    pub causal_input_reference: CausalInputReferenceV1,
    pub canonical_hash: ContentHash,
}

impl ApplicationLifecycleRequestV2 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the durable lifecycle request identity includes every transition precondition"
    )]
    pub fn new(
        request_id: SessionRequestId,
        session_id: ApplicationSessionId,
        expected_revision: u64,
        expected_state: ApplicationSessionStatusV1,
        requested_state: ApplicationSessionStatusV1,
        reason: LifecycleReasonV1,
        causal_input_reference: CausalInputReferenceV1,
    ) -> Result<Self, SessionContractError> {
        if !expected_state.can_transition_to(requested_state) {
            return Err(SessionContractError::InvalidTransition);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            request_id,
            session_id,
            expected_revision,
            expected_state,
            requested_state,
            reason,
            causal_input_reference,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash(
            "nextengine.application-lifecycle-request.v2",
            &value.body_value(),
        );
        Ok(value)
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encoded(&self.body_value())
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SessionContractError> {
        let mut value = decoded_object(bytes, limits, "application_lifecycle_request")?;
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let mut causal = nested_object(
            take(&mut value, "causal_input_reference")?,
            "causal_input_reference",
        )?;
        let request = Self::new(
            request_id(take(&mut value, "request_id")?, "request_id")?,
            session_id(take(&mut value, "session_id")?, "session_id")?,
            u64_value(take(&mut value, "expected_revision")?, "expected_revision")?,
            parse_session_status(&text(
                take(&mut value, "expected_state")?,
                "expected_state",
            )?)?,
            parse_session_status(&text(
                take(&mut value, "requested_state")?,
                "requested_state",
            )?)?,
            LifecycleReasonV1 {
                kind: parse_lifecycle_reason_kind(&text(
                    take(&mut value, "reason_kind")?,
                    "reason_kind",
                )?)?,
                reason_code: crate::ids::SchemaId::new(text(
                    take(&mut value, "reason_code")?,
                    "reason_code",
                )?)?,
            },
            CausalInputReferenceV1 {
                source_kind: parse_causal_source_kind(&text(
                    take(&mut causal, "source_kind")?,
                    "source_kind",
                )?)?,
                canonical_hash: hash(take(&mut causal, "canonical_hash")?, "canonical_hash")?,
            },
        )?;
        reject_unknown(causal)?;
        reject_unknown(value)?;
        if request.canonical_bytes() != bytes {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if Self::new(
            self.request_id,
            self.session_id,
            self.expected_revision,
            self.expected_state,
            self.requested_state,
            self.reason.clone(),
            self.causal_input_reference.clone(),
        )?
        .canonical_hash
            != self.canonical_hash
        {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    fn body_value(&self) -> JcsValue {
        object([
            (
                "causal_input_reference",
                object([
                    (
                        "canonical_hash",
                        string(self.causal_input_reference.canonical_hash.to_hex()),
                    ),
                    (
                        "source_kind",
                        string(self.causal_input_reference.source_kind.token()),
                    ),
                ]),
            ),
            ("expected_revision", number(self.expected_revision)),
            ("expected_state", string(self.expected_state.token())),
            ("reason_code", string(self.reason.reason_code.as_str())),
            ("reason_kind", string(self.reason.kind.token())),
            ("request_id", string(self.request_id.to_hex())),
            ("requested_state", string(self.requested_state.token())),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
        ])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleTransitionOutcomeV1 {
    Committed,
}

impl LifecycleTransitionOutcomeV1 {
    const fn token(self) -> &'static str {
        "Committed"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationLifecycleEventV2 {
    pub schema_version: u32,
    pub transition_id: SessionTransitionId,
    pub request_id: SessionRequestId,
    pub session_id: ApplicationSessionId,
    pub from_state: ApplicationSessionStatusV1,
    pub to_state: ApplicationSessionStatusV1,
    pub before_revision: u64,
    pub after_revision: u64,
    pub outcome: LifecycleTransitionOutcomeV1,
    pub activation_receipt_hash: Option<ContentHash>,
    pub save_receipt_hash: Option<ContentHash>,
    pub diagnostic_hash: Option<ContentHash>,
    pub canonical_hash: ContentHash,
}

impl ApplicationLifecycleEventV2 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the immutable event records all exact transition references"
    )]
    pub fn committed(
        transition_id: SessionTransitionId,
        request: &ApplicationLifecycleRequestV2,
        activation_receipt_hash: Option<ContentHash>,
        save_receipt_hash: Option<ContentHash>,
        diagnostic_hash: Option<ContentHash>,
    ) -> Result<Self, SessionContractError> {
        request.validate()?;
        let after_revision = request
            .expected_revision
            .checked_add(1)
            .ok_or(SessionContractError::InvalidStateFields)?;
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            transition_id,
            request_id: request.request_id,
            session_id: request.session_id,
            from_state: request.expected_state,
            to_state: request.requested_state,
            before_revision: request.expected_revision,
            after_revision,
            outcome: LifecycleTransitionOutcomeV1::Committed,
            activation_receipt_hash,
            save_receipt_hash,
            diagnostic_hash,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash(
            "nextengine.application-lifecycle-event.v2",
            &value.body_value(),
        );
        Ok(value)
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encoded(&self.body_value())
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        request: &ApplicationLifecycleRequestV2,
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SessionContractError> {
        let mut value = decoded_object(bytes, limits, "application_lifecycle_event")?;
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let decoded_transition_id =
            transition_id(take(&mut value, "transition_id")?, "transition_id")?;
        let decoded_request_id = request_id(take(&mut value, "request_id")?, "request_id")?;
        let decoded_session_id = session_id(take(&mut value, "session_id")?, "session_id")?;
        let decoded_from_state =
            parse_session_status(&text(take(&mut value, "from_state")?, "from_state")?)?;
        let decoded_to_state =
            parse_session_status(&text(take(&mut value, "to_state")?, "to_state")?)?;
        let decoded_before_revision =
            u64_value(take(&mut value, "before_revision")?, "before_revision")?;
        let decoded_after_revision =
            u64_value(take(&mut value, "after_revision")?, "after_revision")?;
        if text(take(&mut value, "outcome")?, "outcome")? != "Committed" {
            return Err(SessionContractError::UnknownClosedValue);
        }
        let activation_receipt_hash = optional_hash_value(
            take(&mut value, "activation_receipt_hash_or_none")?,
            "activation_receipt_hash_or_none",
        )?;
        let save_receipt_hash = optional_hash_value(
            take(&mut value, "save_receipt_hash_or_none")?,
            "save_receipt_hash_or_none",
        )?;
        let diagnostic_hash = optional_hash_value(
            take(&mut value, "diagnostic_hash_or_none")?,
            "diagnostic_hash_or_none",
        )?;
        reject_unknown(value)?;
        if decoded_request_id != request.request_id
            || decoded_session_id != request.session_id
            || decoded_from_state != request.expected_state
            || decoded_to_state != request.requested_state
            || decoded_before_revision != request.expected_revision
        {
            return Err(SessionContractError::InvalidTransition);
        }
        let event = Self::committed(
            decoded_transition_id,
            request,
            activation_receipt_hash,
            save_receipt_hash,
            diagnostic_hash,
        )?;
        if event.after_revision != decoded_after_revision || event.canonical_bytes() != bytes {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(event)
    }

    pub fn next_state(
        &self,
        current: &ApplicationSessionStateV2,
        terminal_receipt_hash: Option<ContentHash>,
    ) -> Result<ApplicationSessionStateV2, SessionContractError> {
        if current.session_id != self.session_id
            || current.state != self.from_state
            || current.revision != self.before_revision
            || self.after_revision != self.before_revision + 1
            || !self.from_state.can_transition_to(self.to_state)
        {
            return Err(SessionContractError::InvalidTransition);
        }
        let terminal_receipt_hash = if self.to_state == ApplicationSessionStatusV1::Closed {
            terminal_receipt_hash.ok_or(SessionContractError::InvalidStateFields)?
        } else {
            if terminal_receipt_hash.is_some() {
                return Err(SessionContractError::InvalidStateFields);
            }
            ContentHash::default()
        };
        Ok(ApplicationSessionStateV2::new_unchecked(
            current.session_id,
            self.to_state,
            self.after_revision,
            current.application_session_manifest_hash,
            current.project_composition_lock_hash,
            current.active_runtime_revision,
            current.active_save_generation_hash,
            Some(self.transition_id),
            (self.to_state == ApplicationSessionStatusV1::Closed).then_some(terminal_receipt_hash),
        ))
    }

    fn body_value(&self) -> JcsValue {
        object([
            (
                "activation_receipt_hash_or_none",
                optional_hash(self.activation_receipt_hash),
            ),
            ("after_revision", number(self.after_revision)),
            ("before_revision", number(self.before_revision)),
            (
                "diagnostic_hash_or_none",
                optional_hash(self.diagnostic_hash),
            ),
            ("from_state", string(self.from_state.token())),
            ("outcome", string(self.outcome.token())),
            ("request_id", string(self.request_id.to_hex())),
            (
                "save_receipt_hash_or_none",
                optional_hash(self.save_receipt_hash),
            ),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
            ("to_state", string(self.to_state.token())),
            ("transition_id", string(self.transition_id.to_hex())),
        ])
    }
}

fn parse_session_status(value: &str) -> Result<ApplicationSessionStatusV1, SessionContractError> {
    match value {
        "Created" => Ok(ApplicationSessionStatusV1::Created),
        "CompositionStaged" => Ok(ApplicationSessionStatusV1::CompositionStaged),
        "RuntimeStaged" => Ok(ApplicationSessionStatusV1::RuntimeStaged),
        "Active" => Ok(ApplicationSessionStatusV1::Active),
        "Suspended" => Ok(ApplicationSessionStatusV1::Suspended),
        "Quiescing" => Ok(ApplicationSessionStatusV1::Quiescing),
        "Finalizing" => Ok(ApplicationSessionStatusV1::Finalizing),
        "Closed" => Ok(ApplicationSessionStatusV1::Closed),
        _ => Err(SessionContractError::UnknownClosedValue),
    }
}

fn parse_lifecycle_reason_kind(value: &str) -> Result<LifecycleReasonKindV1, SessionContractError> {
    match value {
        "Launch" => Ok(LifecycleReasonKindV1::Launch),
        "CompositionReady" => Ok(LifecycleReasonKindV1::CompositionReady),
        "RuntimeReady" => Ok(LifecycleReasonKindV1::RuntimeReady),
        "SuspendRequested" => Ok(LifecycleReasonKindV1::SuspendRequested),
        "ResumeRequested" => Ok(LifecycleReasonKindV1::ResumeRequested),
        "UserCloseRequested" => Ok(LifecycleReasonKindV1::UserCloseRequested),
        "HostCloseRequested" => Ok(LifecycleReasonKindV1::HostCloseRequested),
        "FatalHostFault" => Ok(LifecycleReasonKindV1::FatalHostFault),
        "FinalSaveReady" => Ok(LifecycleReasonKindV1::FinalSaveReady),
        "Recovery" => Ok(LifecycleReasonKindV1::Recovery),
        _ => Err(SessionContractError::UnknownClosedValue),
    }
}

fn parse_causal_source_kind(value: &str) -> Result<CausalInputSourceKindV1, SessionContractError> {
    match value {
        "PlatformEvent" => Ok(CausalInputSourceKindV1::PlatformEvent),
        "PlayerAction" => Ok(CausalInputSourceKindV1::PlayerAction),
        "ToolRequest" => Ok(CausalInputSourceKindV1::ToolRequest),
        "System" => Ok(CausalInputSourceKindV1::System),
        _ => Err(SessionContractError::UnknownClosedValue),
    }
}
