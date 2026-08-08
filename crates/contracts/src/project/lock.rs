use std::collections::BTreeMap;

use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{ContentHash, ProjectId};
use crate::manifest_jcs::{JcsValue, decode_canonical_jcs, encode_canonical_jcs};
use crate::platform::PresentationTargetKindV1;

use super::PROJECT_LOCK_FORMAT_V3;
use super::codec::{
    ProjectContractError, array, domain_hash, ensure_unique, expect_format, hash, object,
    reject_unknown, string, take, text, u64_text,
};

/// Exact, content-addressed project closure consumed by every runtime root.
///
/// The lock deliberately contains no package-selection metadata, session
/// policy, storage policy, or implementation-owned values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLockV3 {
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub authoring_sha256: ContentHash,
    pub schema_registry_manifest_sha256: ContentHash,
    pub content_manifest_sha256: ContentHash,
    pub world_partition_manifest_sha256: ContentHash,
    pub mechanics_lock_sha256: ContentHash,
    pub runtime_determinism_profile_sha256: ContentHash,
    pub launch_profiles_sha256: ContentHash,
    pub platform_capability_profile_sha256: ContentHash,
    pub platform_timebase_profile_sha256: ContentHash,
    pub allowed_presentation_targets: Vec<PresentationTargetKindV1>,
    pub project_lock_sha256: ContentHash,
}

impl ProjectLockV3 {
    pub fn new(mut value: Self) -> Result<Self, ProjectContractError> {
        if value.project_revision == 0 {
            return Err(ProjectContractError::ZeroRevision);
        }
        value.allowed_presentation_targets.sort();
        ensure_unique(value.allowed_presentation_targets.iter().copied())?;
        if value.allowed_presentation_targets.is_empty() {
            return Err(ProjectContractError::MissingReference);
        }
        if [
            value.authoring_sha256,
            value.schema_registry_manifest_sha256,
            value.content_manifest_sha256,
            value.world_partition_manifest_sha256,
            value.mechanics_lock_sha256,
            value.runtime_determinism_profile_sha256,
            value.launch_profiles_sha256,
            value.platform_capability_profile_sha256,
            value.platform_timebase_profile_sha256,
        ]
        .contains(&ContentHash::default())
        {
            return Err(ProjectContractError::MissingReference);
        }
        value.project_lock_sha256 = ContentHash::default();
        value.project_lock_sha256 = domain_hash(PROJECT_LOCK_FORMAT_V3, &value.body_bytes());
        Ok(value)
    }

    #[must_use]
    pub fn to_jcs_bytes(&self) -> Vec<u8> {
        self.body_bytes()
    }

    pub fn validate(&self) -> Result<(), ProjectContractError> {
        let canonical = Self::new(self.clone())?;
        if canonical.project_lock_sha256 != self.project_lock_sha256 {
            return Err(ProjectContractError::HashMismatch);
        }
        Ok(())
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ProjectContractError> {
        let value = decode_canonical_jcs(bytes, limits)?;
        let mut object = object(value, "project_lock")?;
        expect_format(&mut object, PROJECT_LOCK_FORMAT_V3)?;
        let value = Self {
            project_id: ProjectId::new(text(take(&mut object, "project_id")?, "project_id")?)?,
            project_revision: u64_text(take(&mut object, "project_revision")?, "project_revision")?,
            authoring_sha256: hash(take(&mut object, "authoring_sha256")?, "authoring_sha256")?,
            schema_registry_manifest_sha256: hash(
                take(&mut object, "schema_registry_manifest_sha256")?,
                "schema_registry_manifest_sha256",
            )?,
            content_manifest_sha256: hash(
                take(&mut object, "content_manifest_sha256")?,
                "content_manifest_sha256",
            )?,
            world_partition_manifest_sha256: hash(
                take(&mut object, "world_partition_manifest_sha256")?,
                "world_partition_manifest_sha256",
            )?,
            mechanics_lock_sha256: hash(
                take(&mut object, "mechanics_lock_sha256")?,
                "mechanics_lock_sha256",
            )?,
            runtime_determinism_profile_sha256: hash(
                take(&mut object, "runtime_determinism_profile_sha256")?,
                "runtime_determinism_profile_sha256",
            )?,
            launch_profiles_sha256: hash(
                take(&mut object, "launch_profiles_sha256")?,
                "launch_profiles_sha256",
            )?,
            platform_capability_profile_sha256: hash(
                take(&mut object, "platform_capability_profile_sha256")?,
                "platform_capability_profile_sha256",
            )?,
            platform_timebase_profile_sha256: hash(
                take(&mut object, "platform_timebase_profile_sha256")?,
                "platform_timebase_profile_sha256",
            )?,
            allowed_presentation_targets: decode_presentation_targets(take(
                &mut object,
                "allowed_presentation_targets",
            )?)?,
            project_lock_sha256: ContentHash::default(),
        };
        reject_unknown(object)?;
        let value = Self::new(value)?;
        if value.body_bytes() != bytes {
            return Err(ProjectContractError::NonCanonical);
        }
        Ok(value)
    }

    fn body_bytes(&self) -> Vec<u8> {
        let mut body = BTreeMap::new();
        body.insert(
            "allowed_presentation_targets".to_owned(),
            JcsValue::Array(
                self.allowed_presentation_targets
                    .iter()
                    .map(|target| string(target.token()))
                    .collect(),
            ),
        );
        insert_hash(&mut body, "authoring_sha256", self.authoring_sha256);
        insert_hash(
            &mut body,
            "content_manifest_sha256",
            self.content_manifest_sha256,
        );
        insert_hash(
            &mut body,
            "launch_profiles_sha256",
            self.launch_profiles_sha256,
        );
        body.insert("lock_format".to_owned(), string(PROJECT_LOCK_FORMAT_V3));
        insert_hash(
            &mut body,
            "mechanics_lock_sha256",
            self.mechanics_lock_sha256,
        );
        insert_hash(
            &mut body,
            "platform_capability_profile_sha256",
            self.platform_capability_profile_sha256,
        );
        insert_hash(
            &mut body,
            "platform_timebase_profile_sha256",
            self.platform_timebase_profile_sha256,
        );
        body.insert("project_id".to_owned(), string(self.project_id.as_str()));
        body.insert(
            "project_revision".to_owned(),
            string(self.project_revision.to_string()),
        );
        insert_hash(
            &mut body,
            "runtime_determinism_profile_sha256",
            self.runtime_determinism_profile_sha256,
        );
        insert_hash(
            &mut body,
            "schema_registry_manifest_sha256",
            self.schema_registry_manifest_sha256,
        );
        insert_hash(
            &mut body,
            "world_partition_manifest_sha256",
            self.world_partition_manifest_sha256,
        );
        encode_canonical_jcs(&JcsValue::Object(body))
    }
}

fn insert_hash(body: &mut BTreeMap<String, JcsValue>, name: &str, value: ContentHash) {
    body.insert(name.to_owned(), string(value.to_hex()));
}

fn decode_presentation_targets(
    value: JcsValue,
) -> Result<Vec<PresentationTargetKindV1>, ProjectContractError> {
    array(value, "allowed_presentation_targets")?
        .into_iter()
        .map(|value| {
            PresentationTargetKindV1::parse(&text(value, "allowed_presentation_target")?)
                .map_err(|_| ProjectContractError::UnknownClosedValue)
        })
        .collect()
}
