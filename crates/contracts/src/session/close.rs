use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{ApplicationSessionId, CloseRequestId, ContentHash};

use super::codec::{
    close_request_id, decoded_object, encoded, hash, nested_object, number, object, optional_hash,
    reject_unknown, session_hash, session_id, string, take, text, u32_value, u64_value,
};
use super::{
    APPLICATION_SESSION_SCHEMA_VERSION, ApplicationSessionStatusV1, CausalInputReferenceV1,
    CausalInputSourceKindV1, LifecycleReasonKindV1, LifecycleReasonV1, SessionContractError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseSessionRequestV2 {
    pub schema_version: u32,
    pub close_request_id: CloseRequestId,
    pub session_id: ApplicationSessionId,
    pub expected_session_revision: u64,
    pub expected_session_state: ApplicationSessionStatusV1,
    pub reason: LifecycleReasonV1,
    pub causal_input_reference: CausalInputReferenceV1,
    pub canonical_hash: ContentHash,
}

impl CloseSessionRequestV2 {
    pub fn new(
        close_request_id: CloseRequestId,
        session_id: ApplicationSessionId,
        expected_session_revision: u64,
        expected_session_state: ApplicationSessionStatusV1,
        reason: LifecycleReasonV1,
        causal_input_reference: CausalInputReferenceV1,
    ) -> Result<Self, SessionContractError> {
        if !matches!(
            expected_session_state,
            ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
        ) {
            return Err(SessionContractError::InvalidCloseState);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            close_request_id,
            session_id,
            expected_session_revision,
            expected_session_state,
            reason,
            causal_input_reference,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash("nextengine.close-session-request.v2", &value.body());
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if Self::new(
            self.close_request_id,
            self.session_id,
            self.expected_session_revision,
            self.expected_session_state,
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

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encoded(&self.body())
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SessionContractError> {
        let mut value = decoded_object(bytes, limits, "close_session_request")?;
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let mut causal = nested_object(
            take(&mut value, "causal_input_reference")?,
            "causal_input_reference",
        )?;
        let request = Self::new(
            close_request_id(take(&mut value, "close_request_id")?, "close_request_id")?,
            session_id(take(&mut value, "session_id")?, "session_id")?,
            u64_value(
                take(&mut value, "expected_session_revision")?,
                "expected_session_revision",
            )?,
            parse_status(&text(
                take(&mut value, "expected_session_state")?,
                "expected_session_state",
            )?)?,
            LifecycleReasonV1 {
                kind: parse_reason(&text(take(&mut value, "reason_kind")?, "reason_kind")?)?,
                reason_code: crate::ids::SchemaId::new(text(
                    take(&mut value, "reason_code")?,
                    "reason_code",
                )?)?,
            },
            CausalInputReferenceV1 {
                source_kind: parse_source(&text(
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

    fn body(&self) -> crate::manifest_jcs::JcsValue {
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
            ("close_request_id", string(self.close_request_id.to_hex())),
            (
                "expected_session_revision",
                number(self.expected_session_revision),
            ),
            (
                "expected_session_state",
                string(self.expected_session_state.token()),
            ),
            ("reason_code", string(self.reason.reason_code.as_str())),
            ("reason_kind", string(self.reason.kind.token())),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
        ])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseSessionJournalStageV2 {
    Prepared,
    SavePublished,
}

impl CloseSessionJournalStageV2 {
    const fn token(self) -> &'static str {
        match self {
            Self::Prepared => "Prepared",
            Self::SavePublished => "SavePublished",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseSessionJournalV2 {
    pub schema_version: u32,
    pub close_request_id: CloseRequestId,
    pub close_request_hash: ContentHash,
    pub stage: CloseSessionJournalStageV2,
    pub save_image_hash: ContentHash,
    pub save_generation_hash: Option<ContentHash>,
    pub canonical_hash: ContentHash,
}

impl CloseSessionJournalV2 {
    pub fn prepared(request: &CloseSessionRequestV2, save_image_hash: ContentHash) -> Self {
        Self::new(
            request.close_request_id,
            request.canonical_hash,
            CloseSessionJournalStageV2::Prepared,
            save_image_hash,
            None,
        )
        .expect("prepared close journal is structurally valid")
    }

    pub fn save_published(
        &self,
        save_generation_hash: ContentHash,
    ) -> Result<Self, SessionContractError> {
        Self::new(
            self.close_request_id,
            self.close_request_hash,
            CloseSessionJournalStageV2::SavePublished,
            self.save_image_hash,
            Some(save_generation_hash),
        )
    }

    pub fn new(
        close_request_id: CloseRequestId,
        close_request_hash: ContentHash,
        stage: CloseSessionJournalStageV2,
        save_image_hash: ContentHash,
        save_generation_hash: Option<ContentHash>,
    ) -> Result<Self, SessionContractError> {
        if (stage == CloseSessionJournalStageV2::SavePublished) != save_generation_hash.is_some() {
            return Err(SessionContractError::InvalidCloseState);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            close_request_id,
            close_request_hash,
            stage,
            save_image_hash,
            save_generation_hash,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash("nextengine.close-session-journal.v2", &value.body());
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION
            || Self::new(
                self.close_request_id,
                self.close_request_hash,
                self.stage,
                self.save_image_hash,
                self.save_generation_hash,
            )?
            .canonical_hash
                != self.canonical_hash
        {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encoded(&self.body())
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SessionContractError> {
        let mut value = decoded_object(bytes, limits, "close_session_journal")?;
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let stage = match text(take(&mut value, "stage")?, "stage")?.as_str() {
            "Prepared" => CloseSessionJournalStageV2::Prepared,
            "SavePublished" => CloseSessionJournalStageV2::SavePublished,
            _ => return Err(SessionContractError::InvalidCloseState),
        };
        let journal = Self::new(
            close_request_id(take(&mut value, "close_request_id")?, "close_request_id")?,
            hash(
                take(&mut value, "close_request_hash")?,
                "close_request_hash",
            )?,
            stage,
            hash(take(&mut value, "save_image_hash")?, "save_image_hash")?,
            super::codec::optional_hash_value(
                take(&mut value, "save_generation_hash_or_none")?,
                "save_generation_hash_or_none",
            )?,
        )?;
        reject_unknown(value)?;
        if journal.canonical_bytes() != bytes {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(journal)
    }

    fn body(&self) -> crate::manifest_jcs::JcsValue {
        object([
            (
                "close_request_hash",
                string(self.close_request_hash.to_hex()),
            ),
            ("close_request_id", string(self.close_request_id.to_hex())),
            (
                "save_generation_hash_or_none",
                optional_hash(self.save_generation_hash),
            ),
            ("save_image_hash", string(self.save_image_hash.to_hex())),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("stage", string(self.stage.token())),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseSessionReceiptV2 {
    pub schema_version: u32,
    pub close_request_id: CloseRequestId,
    pub session_id: ApplicationSessionId,
    pub save_generation_hash: ContentHash,
    pub canonical_hash: ContentHash,
}

impl CloseSessionReceiptV2 {
    #[must_use]
    pub fn new(
        close_request_id: CloseRequestId,
        session_id: ApplicationSessionId,
        save_generation_hash: ContentHash,
    ) -> Self {
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            close_request_id,
            session_id,
            save_generation_hash,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash("nextengine.close-session-receipt.v2", &value.body());
        value
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION
            || Self::new(
                self.close_request_id,
                self.session_id,
                self.save_generation_hash,
            )
            .canonical_hash
                != self.canonical_hash
        {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encoded(&self.body())
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SessionContractError> {
        let mut value = decoded_object(bytes, limits, "close_session_receipt")?;
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let receipt = Self::new(
            close_request_id(take(&mut value, "close_request_id")?, "close_request_id")?,
            session_id(take(&mut value, "session_id")?, "session_id")?,
            hash(
                take(&mut value, "save_generation_hash")?,
                "save_generation_hash",
            )?,
        );
        reject_unknown(value)?;
        if receipt.canonical_bytes() != bytes {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(receipt)
    }

    fn body(&self) -> crate::manifest_jcs::JcsValue {
        object([
            ("close_request_id", string(self.close_request_id.to_hex())),
            (
                "save_generation_hash",
                string(self.save_generation_hash.to_hex()),
            ),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
        ])
    }
}

fn parse_status(value: &str) -> Result<ApplicationSessionStatusV1, SessionContractError> {
    match value {
        "Active" => Ok(ApplicationSessionStatusV1::Active),
        "Suspended" => Ok(ApplicationSessionStatusV1::Suspended),
        _ => Err(SessionContractError::InvalidCloseState),
    }
}

fn parse_source(value: &str) -> Result<CausalInputSourceKindV1, SessionContractError> {
    match value {
        "PlatformEvent" => Ok(CausalInputSourceKindV1::PlatformEvent),
        "PlayerAction" => Ok(CausalInputSourceKindV1::PlayerAction),
        "ToolRequest" => Ok(CausalInputSourceKindV1::ToolRequest),
        "System" => Ok(CausalInputSourceKindV1::System),
        _ => Err(SessionContractError::UnknownClosedValue),
    }
}

fn parse_reason(value: &str) -> Result<LifecycleReasonKindV1, SessionContractError> {
    match value {
        "UserCloseRequested" => Ok(LifecycleReasonKindV1::UserCloseRequested),
        "HostCloseRequested" => Ok(LifecycleReasonKindV1::HostCloseRequested),
        "FatalHostFault" => Ok(LifecycleReasonKindV1::FatalHostFault),
        _ => Err(SessionContractError::UnknownClosedValue),
    }
}
