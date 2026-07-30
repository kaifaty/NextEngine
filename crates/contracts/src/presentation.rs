use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::ids::{ContentHash, PersistentId};
use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::project::{AssetRevisionRefV1, domain_hash};
use crate::render_content::AabbI64V1;

mod camera;

pub use camera::{
    CAMERA_PRESENTATION_RECORD_SCHEMA_VERSION, CameraInterpolationPolicyV1,
    CameraPresentationBatchV1, CameraPresentationRecordV2, CameraProjectionProfileV1,
    CameraResultSampleV1, CameraRoleV1, CameraViewportV1, PRESENTATION_MAX_CAMERA_RECORDS,
    ThirdPersonCameraIntentSampleV1,
};

use camera::{build_camera_batches, camera_batches_value, validate_camera_batches};

pub const PRESENTATION_SNAPSHOT_SCHEMA_VERSION: u32 = 2;
pub const PRESENTATION_SCENE_RECORD_SCHEMA_VERSION: u32 = 2;
pub const PRESENTATION_MAX_SCENE_RECORDS: usize = 16_384;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PresentationRoleV1 {
    Environment = 0,
    PlayerAvatar = 1,
    InteractiveObject = 2,
    Item = 3,
    Character = 4,
}

impl PresentationRoleV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Environment => "Environment",
            Self::PlayerAvatar => "PlayerAvatar",
            Self::InteractiveObject => "InteractiveObject",
            Self::Item => "Item",
            Self::Character => "Character",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct ScenePresentationFlagsV1(u32);

impl ScenePresentationFlagsV1 {
    pub const NONE: Self = Self(0);
    pub const DOUBLE_SIDED: Self = Self(1 << 0);
    pub const ALPHA_TESTED: Self = Self(1 << 1);
    pub const SKINNED: Self = Self(1 << 2);
    const KNOWN_BITS: u32 = Self::DOUBLE_SIDED.0 | Self::ALPHA_TESTED.0 | Self::SKINNED.0;

    pub fn from_bits(bits: u32) -> Result<Self, PresentationContractError> {
        if bits & !Self::KNOWN_BITS != 0 {
            return Err(PresentationContractError::UnknownSceneFeature);
        }
        Ok(Self(bits))
    }

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PresentationObjectKeyV1 {
    pub snapshot_epoch: ContentHash,
    pub persistent_id: PersistentId,
    pub presentation_role: PresentationRoleV1,
    pub incarnation: u32,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QuantizedPresentationTransformV1 {
    pub translation_micrometres: [i64; 3],
    pub orientation_q30: [i32; 4],
}

impl Default for QuantizedPresentationTransformV1 {
    fn default() -> Self {
        Self {
            translation_micrometres: [0; 3],
            orientation_q30: [0, 0, 0, 1 << 30],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenePresentationRecordV2 {
    pub schema_version: u32,
    pub presentation_layer: u16,
    pub object_key: PresentationObjectKeyV1,
    pub mesh_revision: AssetRevisionRefV1,
    pub material_revision: AssetRevisionRefV1,
    pub instance_ordinal: u32,
    pub local_bounds: AabbI64V1,
    pub feature_flags: ScenePresentationFlagsV1,
    pub previous_transform: QuantizedPresentationTransformV1,
    pub current_transform: QuantizedPresentationTransformV1,
    pub visible: bool,
    pub canonical_hash: ContentHash,
}

impl ScenePresentationRecordV2 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the closed presentation record constructor keeps all canonical fields explicit"
    )]
    pub fn new(
        presentation_layer: u16,
        object_key: PresentationObjectKeyV1,
        mesh_revision: AssetRevisionRefV1,
        material_revision: AssetRevisionRefV1,
        instance_ordinal: u32,
        local_bounds: AabbI64V1,
        feature_flags: ScenePresentationFlagsV1,
        previous_transform: QuantizedPresentationTransformV1,
        current_transform: QuantizedPresentationTransformV1,
        visible: bool,
    ) -> Self {
        let mut value = Self {
            schema_version: PRESENTATION_SCENE_RECORD_SCHEMA_VERSION,
            presentation_layer,
            object_key,
            mesh_revision,
            material_revision,
            instance_ordinal,
            local_bounds,
            feature_flags,
            previous_transform,
            current_transform,
            visible,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = value.computed_hash();
        value
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        if self.schema_version != PRESENTATION_SCENE_RECORD_SCHEMA_VERSION {
            return Err(PresentationContractError::UnsupportedVersion);
        }
        if self.mesh_revision.record_sha256 == ContentHash::default()
            || self.material_revision.record_sha256 == ContentHash::default()
        {
            return Err(PresentationContractError::InvalidAssetRevision);
        }
        validate_orientation(self.previous_transform.orientation_q30)?;
        validate_orientation(self.current_transform.orientation_q30)?;
        if self.computed_hash() != self.canonical_hash {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.scene-presentation-record.v2",
            &encode_canonical_jcs(&self.body_value()),
        )
    }

    fn body_value(&self) -> JcsValue {
        object([
            ("current_transform", transform_value(self.current_transform)),
            ("feature_flags", number(self.feature_flags.bits())),
            ("instance_ordinal", number(self.instance_ordinal)),
            ("local_bounds", bounds_value(self.local_bounds)),
            (
                "material_revision",
                asset_revision_value(self.material_revision),
            ),
            ("mesh_revision", asset_revision_value(self.mesh_revision)),
            ("object_key", object_key_value(self.object_key)),
            ("presentation_layer", number(self.presentation_layer)),
            (
                "previous_transform",
                transform_value(self.previous_transform),
            ),
            ("schema_version", number(self.schema_version)),
            (
                "visible",
                string(if self.visible { "true" } else { "false" }),
            ),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenePresentationBatchV1 {
    pub batch_index: u32,
    pub first_global_ordinal: u32,
    pub records: Vec<ScenePresentationRecordV2>,
    pub records_root: ContentHash,
}

impl ScenePresentationBatchV1 {
    fn new(
        batch_index: u32,
        first_global_ordinal: u32,
        records: Vec<ScenePresentationRecordV2>,
    ) -> Result<Self, PresentationContractError> {
        if records.is_empty() {
            return Err(PresentationContractError::EmptyBatch);
        }
        let mut value = Self {
            batch_index,
            first_global_ordinal,
            records,
            records_root: ContentHash::default(),
        };
        value.records_root = value.computed_root();
        Ok(value)
    }

    fn validate(&self) -> Result<(), PresentationContractError> {
        if self.records.is_empty() {
            return Err(PresentationContractError::EmptyBatch);
        }
        for record in &self.records {
            record.validate()?;
        }
        if self.computed_root() != self.records_root {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_root(&self) -> ContentHash {
        domain_hash(
            "nextengine.presentation-batch.v1",
            &encode_canonical_jcs(&object([
                ("batch_index", number(self.batch_index)),
                ("batch_kind", string("Scene")),
                ("first_global_ordinal", number(self.first_global_ordinal)),
                (
                    "record_count",
                    number(u32::try_from(self.records.len()).unwrap_or(u32::MAX)),
                ),
                (
                    "ordered_record_hashes",
                    JcsValue::Array(
                        self.records
                            .iter()
                            .map(|record| string(record.canonical_hash.to_hex()))
                            .collect(),
                    ),
                ),
            ])),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationSnapshotV2 {
    pub schema_version: u32,
    pub snapshot_epoch: ContentHash,
    pub snapshot_sequence: u64,
    pub simulation_tick: u64,
    pub project_composition_lock_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub presentation_profile_hash: ContentHash,
    pub scene_batches: Vec<ScenePresentationBatchV1>,
    pub camera_batches: Vec<CameraPresentationBatchV1>,
    pub semantic_ui_batches: Vec<ContentHash>,
    pub cue_batches: Vec<ContentHash>,
    pub environment_batch: ContentHash,
    pub canonical_hash: ContentHash,
}

impl PresentationSnapshotV2 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the snapshot publication boundary keeps all content and profile bindings explicit"
    )]
    pub fn new(
        snapshot_epoch: ContentHash,
        snapshot_sequence: u64,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        presentation_profile_hash: ContentHash,
        scene_records: Vec<ScenePresentationRecordV2>,
        max_records_per_batch: usize,
        environment_batch: ContentHash,
    ) -> Result<Self, PresentationContractError> {
        Self::new_with_camera_records(
            snapshot_epoch,
            snapshot_sequence,
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            presentation_profile_hash,
            scene_records,
            Vec::new(),
            max_records_per_batch,
            max_records_per_batch,
            environment_batch,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "scene and camera family limits are explicit at the atomic publication boundary"
    )]
    pub fn new_with_camera_records(
        snapshot_epoch: ContentHash,
        snapshot_sequence: u64,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        presentation_profile_hash: ContentHash,
        mut scene_records: Vec<ScenePresentationRecordV2>,
        camera_records: Vec<CameraPresentationRecordV2>,
        max_scene_records_per_batch: usize,
        max_camera_records_per_batch: usize,
        environment_batch: ContentHash,
    ) -> Result<Self, PresentationContractError> {
        if max_scene_records_per_batch == 0 || max_camera_records_per_batch == 0 {
            return Err(PresentationContractError::InvalidBatchProfile);
        }
        if scene_records.len() > PRESENTATION_MAX_SCENE_RECORDS {
            return Err(PresentationContractError::LimitExceeded);
        }
        for record in &scene_records {
            record.validate()?;
            if record.object_key.snapshot_epoch != snapshot_epoch {
                return Err(PresentationContractError::SnapshotEpochMismatch);
            }
        }
        scene_records.sort_by_key(scene_sort_key);
        ensure_record_keys_unique(&scene_records)?;
        let scene_batches = scene_records
            .chunks(max_scene_records_per_batch)
            .enumerate()
            .map(|(batch_index, records)| {
                let first = batch_index
                    .checked_mul(max_scene_records_per_batch)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or(PresentationContractError::LimitExceeded)?;
                ScenePresentationBatchV1::new(
                    u32::try_from(batch_index)
                        .map_err(|_| PresentationContractError::LimitExceeded)?,
                    first,
                    records.to_vec(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let camera_batches =
            build_camera_batches(snapshot_epoch, camera_records, max_camera_records_per_batch)?;
        let mut value = Self {
            schema_version: PRESENTATION_SNAPSHOT_SCHEMA_VERSION,
            snapshot_epoch,
            snapshot_sequence,
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            presentation_profile_hash,
            scene_batches,
            camera_batches,
            semantic_ui_batches: Vec::new(),
            cue_batches: Vec::new(),
            environment_batch,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        if self.schema_version != PRESENTATION_SNAPSHOT_SCHEMA_VERSION {
            return Err(PresentationContractError::UnsupportedVersion);
        }
        let mut expected_batch_index = 0_u32;
        let mut expected_ordinal = 0_u32;
        let mut records = Vec::new();
        for batch in &self.scene_batches {
            if batch.batch_index != expected_batch_index
                || batch.first_global_ordinal != expected_ordinal
            {
                return Err(PresentationContractError::InvalidBatchBoundary);
            }
            batch.validate()?;
            if batch
                .records
                .iter()
                .any(|record| record.object_key.snapshot_epoch != self.snapshot_epoch)
            {
                return Err(PresentationContractError::SnapshotEpochMismatch);
            }
            expected_batch_index = expected_batch_index
                .checked_add(1)
                .ok_or(PresentationContractError::LimitExceeded)?;
            expected_ordinal = expected_ordinal
                .checked_add(
                    u32::try_from(batch.records.len())
                        .map_err(|_| PresentationContractError::LimitExceeded)?,
                )
                .ok_or(PresentationContractError::LimitExceeded)?;
            records.extend(batch.records.iter());
        }
        if records.len() > PRESENTATION_MAX_SCENE_RECORDS
            || records
                .windows(2)
                .any(|pair| scene_sort_key(pair[0]) >= scene_sort_key(pair[1]))
        {
            return Err(PresentationContractError::NonCanonicalOrder);
        }
        ensure_record_refs_unique(&records)?;
        validate_camera_batches(self.snapshot_epoch, &self.camera_batches)?;
        if self.computed_hash() != self.canonical_hash {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    pub fn scene_records(&self) -> impl Iterator<Item = &ScenePresentationRecordV2> {
        self.scene_batches
            .iter()
            .flat_map(|batch| batch.records.iter())
    }

    pub fn camera_records(&self) -> impl Iterator<Item = &CameraPresentationRecordV2> {
        self.camera_batches
            .iter()
            .flat_map(|batch| batch.records.iter())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.presentation-snapshot.v2",
            &encode_canonical_jcs(&object([
                ("camera_batches", camera_batches_value(&self.camera_batches)),
                (
                    "content_manifest_hash",
                    string(self.content_manifest_hash.to_hex()),
                ),
                ("cue_batches", hash_array_value(&self.cue_batches)),
                ("environment_batch", string(self.environment_batch.to_hex())),
                (
                    "presentation_profile_hash",
                    string(self.presentation_profile_hash.to_hex()),
                ),
                (
                    "project_composition_lock_hash",
                    string(self.project_composition_lock_hash.to_hex()),
                ),
                (
                    "scene_batches",
                    JcsValue::Array(
                        self.scene_batches
                            .iter()
                            .map(|batch| {
                                object([
                                    ("batch_index", number(batch.batch_index)),
                                    ("first_global_ordinal", number(batch.first_global_ordinal)),
                                    ("records_root", string(batch.records_root.to_hex())),
                                ])
                            })
                            .collect(),
                    ),
                ),
                ("schema_version", number(self.schema_version)),
                (
                    "semantic_ui_batches",
                    hash_array_value(&self.semantic_ui_batches),
                ),
                ("simulation_tick", JcsValue::Number(self.simulation_tick)),
                ("snapshot_epoch", string(self.snapshot_epoch.to_hex())),
                (
                    "snapshot_sequence",
                    JcsValue::Number(self.snapshot_sequence),
                ),
            ])),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PresentationContractError {
    UnsupportedVersion,
    InvalidOrientation,
    UnknownSceneFeature,
    InvalidAssetRevision,
    SnapshotEpochMismatch,
    HashMismatch,
    DuplicateObjectKey,
    DuplicateCameraKey,
    NonCanonicalOrder,
    InvalidBatchProfile,
    InvalidBatchBoundary,
    EmptyBatch,
    LimitExceeded,
    InvalidCameraViewport,
    InvalidCameraProjection,
    InvalidCameraIntent,
    InvalidCameraResult,
    InvalidCameraPolicy,
}

impl Display for PresentationContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedVersion => "presentation schema version unsupported",
            Self::InvalidOrientation => "presentation orientation is invalid",
            Self::UnknownSceneFeature => "presentation scene feature flag is unknown",
            Self::InvalidAssetRevision => "presentation asset revision hash is zero",
            Self::SnapshotEpochMismatch => "presentation record snapshot epoch does not match",
            Self::HashMismatch => "presentation canonical hash mismatch",
            Self::DuplicateObjectKey => "presentation object key is duplicated",
            Self::DuplicateCameraKey => "presentation camera key is duplicated",
            Self::NonCanonicalOrder => "presentation records are not canonically ordered",
            Self::InvalidBatchProfile => "presentation batch profile is invalid",
            Self::InvalidBatchBoundary => "presentation batch boundary is invalid",
            Self::EmptyBatch => "presentation batch may not be empty",
            Self::LimitExceeded => "presentation contract limit exceeded",
            Self::InvalidCameraViewport => "presentation camera viewport is invalid",
            Self::InvalidCameraProjection => "presentation camera projection is invalid",
            Self::InvalidCameraIntent => "presentation camera intent is invalid",
            Self::InvalidCameraResult => "presentation camera result is invalid",
            Self::InvalidCameraPolicy => "presentation camera cut/interpolation policy is invalid",
        })
    }
}

impl Error for PresentationContractError {}

fn validate_orientation(orientation: [i32; 4]) -> Result<(), PresentationContractError> {
    let norm = orientation.iter().try_fold(0_i128, |sum, value| {
        let value = i128::from(*value);
        sum.checked_add(value * value)
    });
    if norm != Some(1_i128 << 60) {
        Err(PresentationContractError::InvalidOrientation)
    } else {
        Ok(())
    }
}

fn scene_sort_key(
    record: &ScenePresentationRecordV2,
) -> (
    u16,
    PresentationObjectKeyV1,
    AssetRevisionRefV1,
    AssetRevisionRefV1,
    u32,
) {
    (
        record.presentation_layer,
        record.object_key,
        record.mesh_revision,
        record.material_revision,
        record.instance_ordinal,
    )
}

fn ensure_record_keys_unique(
    records: &[ScenePresentationRecordV2],
) -> Result<(), PresentationContractError> {
    ensure_record_refs_unique(&records.iter().collect::<Vec<_>>())
}

fn ensure_record_refs_unique(
    records: &[&ScenePresentationRecordV2],
) -> Result<(), PresentationContractError> {
    let mut object_keys = BTreeSet::new();
    for record in records {
        if !object_keys.insert(record.object_key) {
            return Err(PresentationContractError::DuplicateObjectKey);
        }
    }
    Ok(())
}

fn object_key_value(key: PresentationObjectKeyV1) -> JcsValue {
    object([
        ("incarnation", number(key.incarnation)),
        (
            "persistent_id",
            string(hex_bytes(key.persistent_id.as_bytes())),
        ),
        ("presentation_role", string(key.presentation_role.token())),
        ("snapshot_epoch", string(key.snapshot_epoch.to_hex())),
    ])
}

fn transform_value(transform: QuantizedPresentationTransformV1) -> JcsValue {
    object([
        (
            "orientation_q30",
            JcsValue::Array(
                transform
                    .orientation_q30
                    .iter()
                    .map(|value| string(format!("{:08x}", *value as u32)))
                    .collect(),
            ),
        ),
        (
            "translation_micrometres",
            JcsValue::Array(
                transform
                    .translation_micrometres
                    .iter()
                    .map(|value| string(format!("{:016x}", *value as u64)))
                    .collect(),
            ),
        ),
    ])
}

fn asset_revision_value(revision: AssetRevisionRefV1) -> JcsValue {
    object([
        ("asset_id", string(revision.asset_id.to_hex())),
        ("record_sha256", string(revision.record_sha256.to_hex())),
    ])
}

fn bounds_value(bounds: AabbI64V1) -> JcsValue {
    object([
        (
            "max",
            JcsValue::Array(
                bounds
                    .max()
                    .iter()
                    .map(|value| string(format!("{:016x}", *value as u64)))
                    .collect(),
            ),
        ),
        (
            "min",
            JcsValue::Array(
                bounds
                    .min()
                    .iter()
                    .map(|value| string(format!("{:016x}", *value as u64)))
                    .collect(),
            ),
        ),
    ])
}

fn hash_array_value(values: &[ContentHash]) -> JcsValue {
    JcsValue::Array(values.iter().map(|value| string(value.to_hex())).collect())
}

fn object<const N: usize>(entries: [(&str, JcsValue); N]) -> JcsValue {
    JcsValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn number(value: impl Into<u64>) -> JcsValue {
    JcsValue::Number(value.into())
}

fn string(value: impl Into<String>) -> JcsValue {
    JcsValue::String(value.into())
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[usize::from(*byte >> 4)]));
        value.push(char::from(HEX[usize::from(*byte & 0x0f)]));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::AssetId;

    #[test]
    fn extraction_order_and_batch_profile_produce_canonical_records() {
        let epoch = domain_hash("test.presentation.epoch", b"epoch");
        let mut records = vec![
            record(epoch, 2, PresentationRoleV1::Item),
            record(epoch, 1, PresentationRoleV1::PlayerAvatar),
        ];
        let forward = PresentationSnapshotV2::new(
            epoch,
            0,
            3,
            domain_hash("test.lock", b"lock"),
            domain_hash("test.content", b"content"),
            domain_hash("test.profile", b"profile"),
            records.clone(),
            1,
            domain_hash("test.environment", b"environment"),
        )
        .expect("snapshot");
        records.reverse();
        let reverse = PresentationSnapshotV2::new(
            epoch,
            0,
            3,
            domain_hash("test.lock", b"lock"),
            domain_hash("test.content", b"content"),
            domain_hash("test.profile", b"profile"),
            records,
            1,
            domain_hash("test.environment", b"environment"),
        )
        .expect("snapshot");
        assert_eq!(forward, reverse);
        forward.validate().expect("valid");
    }

    #[test]
    fn duplicate_object_key_rejects_whole_snapshot() {
        let epoch = domain_hash("test.presentation.epoch", b"epoch");
        let duplicate = record(epoch, 1, PresentationRoleV1::Item);
        assert_eq!(
            PresentationSnapshotV2::new(
                epoch,
                0,
                0,
                domain_hash("test.lock", b"lock"),
                domain_hash("test.content", b"content"),
                domain_hash("test.profile", b"profile"),
                vec![duplicate.clone(), duplicate],
                8,
                domain_hash("test.environment", b"environment"),
            ),
            Err(PresentationContractError::DuplicateObjectKey)
        );
    }

    #[test]
    fn duplicate_object_key_with_different_asset_is_rejected() {
        let epoch = domain_hash("test.presentation.epoch", b"epoch");
        let first = record(epoch, 1, PresentationRoleV1::Item);
        let second = ScenePresentationRecordV2::new(
            first.presentation_layer,
            first.object_key,
            asset_revision(9, "test.mesh.second"),
            first.material_revision,
            1,
            first.local_bounds,
            first.feature_flags,
            first.previous_transform,
            first.current_transform,
            first.visible,
        );
        assert_eq!(
            PresentationSnapshotV2::new(
                epoch,
                0,
                0,
                domain_hash("test.lock", b"lock"),
                domain_hash("test.content", b"content"),
                domain_hash("test.profile", b"profile"),
                vec![first, second],
                8,
                domain_hash("test.environment", b"environment"),
            ),
            Err(PresentationContractError::DuplicateObjectKey)
        );
    }

    #[test]
    fn scene_record_rejects_zero_exact_revision_hashes() {
        let epoch = domain_hash("test.presentation.epoch", b"epoch");
        let zero_mesh = ScenePresentationRecordV2::new(
            0,
            PresentationObjectKeyV1 {
                snapshot_epoch: epoch,
                persistent_id: PersistentId::from_bytes([1; 16]),
                presentation_role: PresentationRoleV1::Item,
                incarnation: 0,
            },
            AssetRevisionRefV1 {
                asset_id: AssetId::from_bytes([2; 16]),
                record_sha256: ContentHash::default(),
            },
            asset_revision(3, "test.material"),
            0,
            AabbI64V1::new([-1; 3], [1; 3]).expect("bounds"),
            ScenePresentationFlagsV1::NONE,
            QuantizedPresentationTransformV1::default(),
            QuantizedPresentationTransformV1::default(),
            true,
        );
        assert_eq!(
            zero_mesh.validate(),
            Err(PresentationContractError::InvalidAssetRevision)
        );
    }

    #[test]
    fn snapshot_rejects_record_from_another_epoch() {
        let snapshot_epoch = domain_hash("test.presentation.epoch", b"snapshot");
        let record_epoch = domain_hash("test.presentation.epoch", b"record");
        assert_eq!(
            PresentationSnapshotV2::new(
                snapshot_epoch,
                0,
                0,
                domain_hash("test.lock", b"lock"),
                domain_hash("test.content", b"content"),
                domain_hash("test.profile", b"profile"),
                vec![record(record_epoch, 1, PresentationRoleV1::PlayerAvatar)],
                8,
                domain_hash("test.environment", b"environment"),
            ),
            Err(PresentationContractError::SnapshotEpochMismatch)
        );
    }

    fn record(
        epoch: ContentHash,
        persistent: u8,
        role: PresentationRoleV1,
    ) -> ScenePresentationRecordV2 {
        ScenePresentationRecordV2::new(
            role as u16,
            PresentationObjectKeyV1 {
                snapshot_epoch: epoch,
                persistent_id: PersistentId::from_bytes([persistent; 16]),
                presentation_role: role,
                incarnation: 0,
            },
            asset_revision(persistent, "test.mesh"),
            asset_revision(persistent.saturating_add(32), "test.material"),
            0,
            AabbI64V1::new([-1_000_000; 3], [1_000_001; 3]).expect("bounds"),
            ScenePresentationFlagsV1::NONE,
            QuantizedPresentationTransformV1::default(),
            QuantizedPresentationTransformV1::default(),
            true,
        )
    }

    fn asset_revision(id: u8, domain: &str) -> AssetRevisionRefV1 {
        AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([id; 16]),
            record_sha256: domain_hash(domain, &[id]),
        }
    }
}
