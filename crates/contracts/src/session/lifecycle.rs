use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{ApplicationSessionId, ContentHash, SessionRequestId, SessionTransitionId};
use crate::manifest_jcs::JcsValue;

use super::codec::{
    decoded_object, encoded, hash, number, object, optional_hash, optional_hash_value,
    reject_unknown, session_hash, session_id, string, take, text, u32_value,
};
use super::{
    APPLICATION_SESSION_MANIFEST_FORMAT_V1, APPLICATION_SESSION_SCHEMA_VERSION,
    ApplicationSessionStatusV1, CausalInputReferenceV1, CompositionRootV1, LifecycleReasonV1,
    PresentationTargetKindV1, SessionContractError, validate_root_target,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSessionManifestBodyV1 {
    pub session_id: ApplicationSessionId,
    pub composition_root: CompositionRootV1,
    pub project_composition_lock_hash: ContentHash,
    pub launch_profile_hash: ContentHash,
    pub platform_capability_set_hash: Option<ContentHash>,
    pub runtime_determinism_profile_hash: ContentHash,
    pub schema_registry_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub recovery_policy_hash: ContentHash,
    pub shutdown_policy_hash: ContentHash,
    pub recovery_session_link_hash: Option<ContentHash>,
    pub presentation_target_kind: PresentationTargetKindV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSessionManifestV1 {
    pub schema_version: u32,
    pub body: ApplicationSessionManifestBodyV1,
    pub canonical_hash: ContentHash,
}

impl ApplicationSessionManifestV1 {
    pub fn new(body: ApplicationSessionManifestBodyV1) -> Result<Self, SessionContractError> {
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
            session_hash(APPLICATION_SESSION_MANIFEST_FORMAT_V1, &value.body_value());
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
        if format != APPLICATION_SESSION_MANIFEST_FORMAT_V1 {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let body = ApplicationSessionManifestBodyV1 {
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
            recovery_policy_hash: hash(
                take(&mut value, "recovery_policy_hash")?,
                "recovery_policy_hash",
            )?,
            shutdown_policy_hash: hash(
                take(&mut value, "shutdown_policy_hash")?,
                "shutdown_policy_hash",
            )?,
            recovery_session_link_hash: optional_hash_value(
                take(&mut value, "recovery_session_link_hash_or_none")?,
                "recovery_session_link_hash_or_none",
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
                string(APPLICATION_SESSION_MANIFEST_FORMAT_V1),
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
                "recovery_policy_hash",
                string(self.body.recovery_policy_hash.to_hex()),
            ),
            (
                "recovery_session_link_hash_or_none",
                optional_hash(self.body.recovery_session_link_hash),
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
            (
                "shutdown_policy_hash",
                string(self.body.shutdown_policy_hash.to_hex()),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSessionStateV1 {
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

impl ApplicationSessionStateV1 {
    #[must_use]
    pub fn created(manifest: &ApplicationSessionManifestV1) -> Self {
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
            "nextengine.application-session-state.v1",
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
pub struct ApplicationLifecycleRequestV1 {
    pub schema_version: u32,
    pub request_id: SessionRequestId,
    pub session_id: ApplicationSessionId,
    pub expected_revision: u64,
    pub expected_state: ApplicationSessionStatusV1,
    pub requested_state: ApplicationSessionStatusV1,
    pub reason: LifecycleReasonV1,
    pub policy_hash: ContentHash,
    pub causal_input_reference: CausalInputReferenceV1,
    pub canonical_hash: ContentHash,
}

impl ApplicationLifecycleRequestV1 {
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
        policy_hash: ContentHash,
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
            policy_hash,
            causal_input_reference,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash(
            "nextengine.application-lifecycle-request.v1",
            &value.body_value(),
        );
        Ok(value)
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encoded(&self.body_value())
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
            self.policy_hash,
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
            ("policy_hash", string(self.policy_hash.to_hex())),
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
pub struct ApplicationLifecycleEventV1 {
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

impl ApplicationLifecycleEventV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the immutable event records all exact transition references"
    )]
    pub fn committed(
        transition_id: SessionTransitionId,
        request: &ApplicationLifecycleRequestV1,
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
            "nextengine.application-lifecycle-event.v1",
            &value.body_value(),
        );
        Ok(value)
    }

    pub fn next_state(
        &self,
        current: &ApplicationSessionStateV1,
        terminal_receipt_hash: Option<ContentHash>,
    ) -> Result<ApplicationSessionStateV1, SessionContractError> {
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
        Ok(ApplicationSessionStateV1::new_unchecked(
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
