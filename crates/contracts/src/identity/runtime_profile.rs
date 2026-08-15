use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CanonicalDecodeLimits,
    CanonicalError, CanonicalField, decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::ids::{ContentHash, content_hash_from_bytes};

use super::codec::{read_hash, read_u16, read_u32, require_envelope, require_fields};
use super::derivation::runtime_profile_hash;
use super::error::IdentityContractError;

pub const RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const RUNTIME_MAXIMUM_FUTURE_COMMAND_TICKS: u32 = 3600;

const RUNTIME_PROFILE_OWNER_ID: &str = "nextengine.runtime";
pub(super) const RUNTIME_PROFILE_SCHEMA_ID: &str = "nextengine.runtime-determinism-profile";
const RUNTIME_PROFILE_SEGMENT_ID: &str = "v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeDeterminismProfileV1 {
    pub schema_version: u16,
    pub tick_rate_profile_hash: ContentHash,
    pub ingress_assignment_profile_hash: ContentHash,
    pub command_identity_profile_hash: ContentHash,
    pub command_ledger_profile_hash: ContentHash,
    pub rng_profile_hash: ContentHash,
    pub schedule_manifest_hash: ContentHash,
    pub task_merge_profile_hash: ContentHash,
    pub numeric_profile_hash: ContentHash,
    pub physics_quantization_profile_hash: ContentHash,
    pub command_kind_registry_hash: ContentHash,
    pub admission_limits_profile_hash: ContentHash,
    pub maximum_future_command_ticks: u32,
}

impl RuntimeDeterminismProfileV1 {
    pub(super) fn from_materialized_r4a(
        command_kind_registry_hash: ContentHash,
        schedule_manifest_hash: ContentHash,
    ) -> Self {
        let fixed = |name: &[u8]| content_hash_from_bytes(sha256(name));
        let admission_limits = crate::input::RuntimeAdmissionLimitsV1::default();
        let tick_rate_profile = crate::input::TickRateProfileV1::at_30_hz();
        let ingress_assignment_profile =
            crate::input::IngressAssignmentProfileV1::core_v1(&admission_limits)
                .expect("the built-in ingress profile is canonically representable");
        let physics_quantization_profile =
            crate::physics::PhysicsQuantizationProfileV1::capsule_reference_v1()
                .expect("the built-in physics profile identifiers are valid");
        let authoritative_numeric_profile =
            crate::physics::AuthoritativeNumericProfileV1::capsule_reference_v1(
                &physics_quantization_profile,
            )
            .expect("the built-in numeric profile is canonically representable");
        Self {
            schema_version: RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION,
            tick_rate_profile_hash: tick_rate_profile
                .profile_hash()
                .expect("the built-in tick profile is canonically representable"),
            ingress_assignment_profile_hash: ingress_assignment_profile
                .profile_hash()
                .expect("the built-in ingress profile is canonically representable"),
            command_identity_profile_hash: fixed(b"nextengine.bootstrap.command-identity.v2"),
            command_ledger_profile_hash: fixed(b"nextengine.bootstrap.command-ledger.v2"),
            rng_profile_hash: fixed(b"nextengine.bootstrap.rng.v1"),
            schedule_manifest_hash,
            task_merge_profile_hash: fixed(b"nextengine.bootstrap.task-merge.v1"),
            numeric_profile_hash: authoritative_numeric_profile
                .profile_hash()
                .expect("the built-in numeric profile is canonically representable"),
            physics_quantization_profile_hash: physics_quantization_profile
                .profile_hash()
                .expect("the built-in physics profile is canonically representable"),
            command_kind_registry_hash,
            admission_limits_profile_hash: admission_limits
                .profile_hash()
                .expect("the built-in admission profile is canonically representable"),
            maximum_future_command_ticks: 120,
        }
    }

    pub fn validate(&self) -> Result<(), IdentityContractError> {
        if self.schema_version != RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION {
            return Err(IdentityContractError::UnsupportedVersion {
                contract: "runtime determinism profile",
                version: self.schema_version,
            });
        }
        if self.maximum_future_command_ticks > RUNTIME_MAXIMUM_FUTURE_COMMAND_TICKS {
            return Err(IdentityContractError::FutureHorizonExceeded(
                self.maximum_future_command_ticks,
            ));
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let hashes = [
            self.tick_rate_profile_hash,
            self.ingress_assignment_profile_hash,
            self.command_identity_profile_hash,
            self.command_ledger_profile_hash,
            self.rng_profile_hash,
            self.schedule_manifest_hash,
            self.task_merge_profile_hash,
            self.numeric_profile_hash,
            self.physics_quantization_profile_hash,
            self.command_kind_registry_hash,
            self.admission_limits_profile_hash,
        ];
        let mut fields = vec![CanonicalField::new(
            1,
            CANONICAL_TYPE_U16,
            self.schema_version.to_le_bytes().to_vec(),
        )];
        for (offset, hash) in hashes.into_iter().enumerate() {
            fields.push(CanonicalField::new(
                u32::try_from(offset).map_err(|_| CanonicalError::LengthOverflow)? + 2,
                CANONICAL_TYPE_HASH256,
                hash.as_bytes().to_vec(),
            ));
        }
        fields.push(CanonicalField::new(
            13,
            CANONICAL_TYPE_U32,
            self.maximum_future_command_ticks.to_le_bytes().to_vec(),
        ));
        encode_canonical_segment(
            RUNTIME_PROFILE_OWNER_ID,
            RUNTIME_PROFILE_SCHEMA_ID,
            RUNTIME_PROFILE_SEGMENT_ID,
            fields,
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, IdentityContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        require_envelope(
            &segment,
            RUNTIME_PROFILE_OWNER_ID,
            RUNTIME_PROFILE_SCHEMA_ID,
            RUNTIME_PROFILE_SEGMENT_ID,
        )?;
        let mut expected = vec![(1, CANONICAL_TYPE_U16)];
        expected.extend((2..=12).map(|id| (id, CANONICAL_TYPE_HASH256)));
        expected.push((13, CANONICAL_TYPE_U32));
        require_fields(&segment, &expected)?;
        let profile = Self {
            schema_version: read_u16(&segment, 1)?,
            tick_rate_profile_hash: read_hash(&segment, 2)?,
            ingress_assignment_profile_hash: read_hash(&segment, 3)?,
            command_identity_profile_hash: read_hash(&segment, 4)?,
            command_ledger_profile_hash: read_hash(&segment, 5)?,
            rng_profile_hash: read_hash(&segment, 6)?,
            schedule_manifest_hash: read_hash(&segment, 7)?,
            task_merge_profile_hash: read_hash(&segment, 8)?,
            numeric_profile_hash: read_hash(&segment, 9)?,
            physics_quantization_profile_hash: read_hash(&segment, 10)?,
            command_kind_registry_hash: read_hash(&segment, 11)?,
            admission_limits_profile_hash: read_hash(&segment, 12)?,
            maximum_future_command_ticks: read_u32(&segment, 13)?,
        };
        profile.validate()?;
        if profile.canonical_bytes()? != bytes {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }
        Ok(profile)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        runtime_profile_hash(self)
    }
}
