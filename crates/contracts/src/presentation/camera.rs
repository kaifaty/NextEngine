use std::collections::BTreeSet;

use crate::ids::{ContentHash, PersistentId};
use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::project::{AssetRevisionRefV1, domain_hash};

use super::{
    PresentationContractError, QuantizedPresentationTransformV1, asset_revision_value, hex_bytes,
    number, object, string, transform_value, validate_orientation,
};

pub const CAMERA_PRESENTATION_RECORD_SCHEMA_VERSION: u32 = 2;
pub const PRESENTATION_MAX_CAMERA_RECORDS: usize = 64;

const CAMERA_POSITION_LIMIT_MICROMETRES: i64 = 100_000_000_000;
const CAMERA_DISTANCE_LIMIT_MICROMETRES: u64 = 100_000_000;
const CAMERA_SHOULDER_LIMIT_MICROMETRES: i64 = 10_000_000;
const CAMERA_FAR_PLANE_LIMIT_MICROMETRES: u64 = 1_000_000_000_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CameraRoleV1 {
    PrimaryThirdPerson = 0,
    DeveloperCapture = 1,
}

impl CameraRoleV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::PrimaryThirdPerson => "PrimaryThirdPerson",
            Self::DeveloperCapture => "DeveloperCapture",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CameraViewportV1 {
    pub viewport_id: u16,
    pub origin_unorm16: [u16; 2],
    pub extent_unorm16: [u16; 2],
}

impl CameraViewportV1 {
    pub fn new(
        viewport_id: u16,
        origin_unorm16: [u16; 2],
        extent_unorm16: [u16; 2],
    ) -> Result<Self, PresentationContractError> {
        let value = Self {
            viewport_id,
            origin_unorm16,
            extent_unorm16,
        };
        value.validate()?;
        Ok(value)
    }

    pub const fn full(viewport_id: u16) -> Self {
        Self {
            viewport_id,
            origin_unorm16: [0, 0],
            extent_unorm16: [u16::MAX, u16::MAX],
        }
    }

    fn validate(self) -> Result<(), PresentationContractError> {
        if self.extent_unorm16.contains(&0)
            || self
                .origin_unorm16
                .into_iter()
                .zip(self.extent_unorm16)
                .any(|(origin, extent)| u32::from(origin) + u32::from(extent) > u32::from(u16::MAX))
        {
            return Err(PresentationContractError::InvalidCameraViewport);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CameraProjectionProfileV1 {
    pub vertical_fov_millidegrees: u32,
    pub near_plane_micrometres: u64,
    pub far_plane_micrometres: u64,
}

impl CameraProjectionProfileV1 {
    pub fn new(
        vertical_fov_millidegrees: u32,
        near_plane_micrometres: u64,
        far_plane_micrometres: u64,
    ) -> Result<Self, PresentationContractError> {
        let value = Self {
            vertical_fov_millidegrees,
            near_plane_micrometres,
            far_plane_micrometres,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(self) -> Result<(), PresentationContractError> {
        if !(1..179_000).contains(&self.vertical_fov_millidegrees)
            || self.near_plane_micrometres == 0
            || self.far_plane_micrometres <= self.near_plane_micrometres
            || self.far_plane_micrometres > CAMERA_FAR_PLANE_LIMIT_MICROMETRES
        {
            return Err(PresentationContractError::InvalidCameraProjection);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ThirdPersonCameraIntentSampleV1 {
    pub focus_subject_id: Option<PersistentId>,
    pub focus_point_micrometres: [i64; 3],
    pub orbit_yaw_millidegrees: i32,
    pub orbit_pitch_millidegrees: i32,
    pub distance_micrometres: u64,
    pub shoulder_offset_micrometres: [i64; 3],
}

impl ThirdPersonCameraIntentSampleV1 {
    pub fn validate(self) -> Result<(), PresentationContractError> {
        if !(-180_000..180_000).contains(&self.orbit_yaw_millidegrees)
            || !(-89_900..=89_900).contains(&self.orbit_pitch_millidegrees)
            || self.distance_micrometres == 0
            || self.distance_micrometres > CAMERA_DISTANCE_LIMIT_MICROMETRES
            || !coordinates_are_bounded(self.focus_point_micrometres)
            || self
                .shoulder_offset_micrometres
                .iter()
                .any(|value| value.unsigned_abs() > CAMERA_SHOULDER_LIMIT_MICROMETRES as u64)
        {
            return Err(PresentationContractError::InvalidCameraIntent);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CameraResultSampleV1 {
    pub pose: QuantizedPresentationTransformV1,
    pub focus_point_micrometres: [i64; 3],
}

impl CameraResultSampleV1 {
    pub fn validate(self) -> Result<(), PresentationContractError> {
        validate_orientation(self.pose.orientation_q30)?;
        if !coordinates_are_bounded(self.pose.translation_micrometres)
            || !coordinates_are_bounded(self.focus_point_micrometres)
        {
            return Err(PresentationContractError::InvalidCameraResult);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CameraInterpolationPolicyV1 {
    Hold = 0,
    LinearPose = 1,
}

impl CameraInterpolationPolicyV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Hold => "Hold",
            Self::LinearPose => "LinearPose",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CameraPresentationRecordV2 {
    pub schema_version: u32,
    pub snapshot_epoch: ContentHash,
    pub camera_id: PersistentId,
    pub camera_role: CameraRoleV1,
    pub viewport: CameraViewportV1,
    pub projection_profile: CameraProjectionProfileV1,
    pub intent_sample: ThirdPersonCameraIntentSampleV1,
    pub previous_result_sample: CameraResultSampleV1,
    pub current_result_sample: CameraResultSampleV1,
    pub exposure_profile_revision: AssetRevisionRefV1,
    pub cut: bool,
    pub interpolation_policy: CameraInterpolationPolicyV1,
    pub canonical_hash: ContentHash,
}

impl CameraPresentationRecordV2 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the typed camera record keeps every canonical presentation input explicit"
    )]
    pub fn new(
        snapshot_epoch: ContentHash,
        camera_id: PersistentId,
        camera_role: CameraRoleV1,
        viewport: CameraViewportV1,
        projection_profile: CameraProjectionProfileV1,
        intent_sample: ThirdPersonCameraIntentSampleV1,
        previous_result_sample: CameraResultSampleV1,
        current_result_sample: CameraResultSampleV1,
        exposure_profile_revision: AssetRevisionRefV1,
        cut: bool,
        interpolation_policy: CameraInterpolationPolicyV1,
    ) -> Result<Self, PresentationContractError> {
        let mut value = Self {
            schema_version: CAMERA_PRESENTATION_RECORD_SCHEMA_VERSION,
            snapshot_epoch,
            camera_id,
            camera_role,
            viewport,
            projection_profile,
            intent_sample,
            previous_result_sample,
            current_result_sample,
            exposure_profile_revision,
            cut,
            interpolation_policy,
            canonical_hash: ContentHash::default(),
        };
        value.validate_body()?;
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PresentationContractError> {
        if self.schema_version != CAMERA_PRESENTATION_RECORD_SCHEMA_VERSION {
            return Err(PresentationContractError::UnsupportedVersion);
        }
        self.validate_body()?;
        if self.computed_hash() != self.canonical_hash {
            return Err(PresentationContractError::HashMismatch);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), PresentationContractError> {
        self.viewport.validate()?;
        self.projection_profile.validate()?;
        self.intent_sample.validate()?;
        self.previous_result_sample.validate()?;
        self.current_result_sample.validate()?;
        if self.exposure_profile_revision.record_sha256 == ContentHash::default() {
            return Err(PresentationContractError::InvalidAssetRevision);
        }
        if self.cut && self.interpolation_policy != CameraInterpolationPolicyV1::Hold {
            return Err(PresentationContractError::InvalidCameraPolicy);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.camera-presentation-record.v2",
            &encode_canonical_jcs(&self.body_value()),
        )
    }

    fn body_value(&self) -> JcsValue {
        object([
            ("camera_id", string(hex_bytes(self.camera_id.as_bytes()))),
            ("camera_role", string(self.camera_role.token())),
            ("cut", string(if self.cut { "true" } else { "false" })),
            (
                "current_result_sample",
                result_sample_value(self.current_result_sample),
            ),
            (
                "exposure_profile_revision",
                asset_revision_value(self.exposure_profile_revision),
            ),
            ("intent_sample", intent_sample_value(self.intent_sample)),
            (
                "interpolation_policy",
                string(self.interpolation_policy.token()),
            ),
            (
                "previous_result_sample",
                result_sample_value(self.previous_result_sample),
            ),
            (
                "projection_profile",
                projection_value(self.projection_profile),
            ),
            ("schema_version", number(self.schema_version)),
            ("snapshot_epoch", string(self.snapshot_epoch.to_hex())),
            ("viewport", viewport_value(self.viewport)),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CameraPresentationBatchV1 {
    pub batch_index: u32,
    pub first_global_ordinal: u32,
    pub records: Vec<CameraPresentationRecordV2>,
    pub records_root: ContentHash,
}

impl CameraPresentationBatchV1 {
    fn new(
        batch_index: u32,
        first_global_ordinal: u32,
        records: Vec<CameraPresentationRecordV2>,
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
                ("batch_kind", string("Camera")),
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

pub(super) fn build_camera_batches(
    snapshot_epoch: ContentHash,
    mut records: Vec<CameraPresentationRecordV2>,
    max_records_per_batch: usize,
) -> Result<Vec<CameraPresentationBatchV1>, PresentationContractError> {
    if max_records_per_batch == 0 {
        return Err(PresentationContractError::InvalidBatchProfile);
    }
    if records.len() > PRESENTATION_MAX_CAMERA_RECORDS {
        return Err(PresentationContractError::LimitExceeded);
    }
    for record in &records {
        record.validate()?;
        if record.snapshot_epoch != snapshot_epoch {
            return Err(PresentationContractError::SnapshotEpochMismatch);
        }
    }
    records.sort_by_key(camera_sort_key);
    ensure_camera_keys_unique(records.iter())?;
    records
        .chunks(max_records_per_batch)
        .enumerate()
        .map(|(batch_index, records)| {
            let first = batch_index
                .checked_mul(max_records_per_batch)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or(PresentationContractError::LimitExceeded)?;
            CameraPresentationBatchV1::new(
                u32::try_from(batch_index).map_err(|_| PresentationContractError::LimitExceeded)?,
                first,
                records.to_vec(),
            )
        })
        .collect()
}

pub(super) fn validate_camera_batches(
    snapshot_epoch: ContentHash,
    batches: &[CameraPresentationBatchV1],
) -> Result<(), PresentationContractError> {
    let mut expected_batch_index = 0_u32;
    let mut expected_ordinal = 0_u32;
    let mut records = Vec::new();
    for batch in batches {
        if batch.batch_index != expected_batch_index
            || batch.first_global_ordinal != expected_ordinal
        {
            return Err(PresentationContractError::InvalidBatchBoundary);
        }
        batch.validate()?;
        if batch
            .records
            .iter()
            .any(|record| record.snapshot_epoch != snapshot_epoch)
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
    if records.len() > PRESENTATION_MAX_CAMERA_RECORDS
        || records
            .windows(2)
            .any(|pair| camera_sort_key(pair[0]) >= camera_sort_key(pair[1]))
    {
        return Err(PresentationContractError::NonCanonicalOrder);
    }
    ensure_camera_keys_unique(records)
}

pub(super) fn camera_batches_value(batches: &[CameraPresentationBatchV1]) -> JcsValue {
    JcsValue::Array(
        batches
            .iter()
            .map(|batch| {
                object([
                    ("batch_index", number(batch.batch_index)),
                    ("first_global_ordinal", number(batch.first_global_ordinal)),
                    ("records_root", string(batch.records_root.to_hex())),
                ])
            })
            .collect(),
    )
}

fn camera_sort_key(record: &CameraPresentationRecordV2) -> (u16, CameraRoleV1, PersistentId) {
    (
        record.viewport.viewport_id,
        record.camera_role,
        record.camera_id,
    )
}

fn ensure_camera_keys_unique<'a>(
    records: impl IntoIterator<Item = &'a CameraPresentationRecordV2>,
) -> Result<(), PresentationContractError> {
    let mut keys = BTreeSet::new();
    for record in records {
        if !keys.insert(camera_sort_key(record)) {
            return Err(PresentationContractError::DuplicateCameraKey);
        }
    }
    Ok(())
}

fn viewport_value(viewport: CameraViewportV1) -> JcsValue {
    object([
        (
            "extent_unorm16",
            JcsValue::Array(viewport.extent_unorm16.into_iter().map(number).collect()),
        ),
        (
            "origin_unorm16",
            JcsValue::Array(viewport.origin_unorm16.into_iter().map(number).collect()),
        ),
        ("viewport_id", number(viewport.viewport_id)),
    ])
}

fn projection_value(projection: CameraProjectionProfileV1) -> JcsValue {
    object([
        (
            "far_plane_micrometres",
            JcsValue::Number(projection.far_plane_micrometres),
        ),
        (
            "near_plane_micrometres",
            JcsValue::Number(projection.near_plane_micrometres),
        ),
        (
            "vertical_fov_millidegrees",
            number(projection.vertical_fov_millidegrees),
        ),
    ])
}

fn intent_sample_value(intent: ThirdPersonCameraIntentSampleV1) -> JcsValue {
    object([
        (
            "distance_micrometres",
            JcsValue::Number(intent.distance_micrometres),
        ),
        (
            "focus_point_micrometres",
            signed_i64_array(intent.focus_point_micrometres),
        ),
        (
            "focus_subject_id_or_none",
            string(
                intent
                    .focus_subject_id
                    .map_or_else(|| "none".to_owned(), |id| hex_bytes(id.as_bytes())),
            ),
        ),
        (
            "orbit_pitch_millidegrees",
            string(format!("{:08x}", intent.orbit_pitch_millidegrees as u32)),
        ),
        (
            "orbit_yaw_millidegrees",
            string(format!("{:08x}", intent.orbit_yaw_millidegrees as u32)),
        ),
        (
            "shoulder_offset_micrometres",
            signed_i64_array(intent.shoulder_offset_micrometres),
        ),
    ])
}

fn result_sample_value(result: CameraResultSampleV1) -> JcsValue {
    object([
        (
            "focus_point_micrometres",
            signed_i64_array(result.focus_point_micrometres),
        ),
        ("pose", transform_value(result.pose)),
    ])
}

fn signed_i64_array(values: [i64; 3]) -> JcsValue {
    JcsValue::Array(
        values
            .into_iter()
            .map(|value| string(format!("{:016x}", value as u64)))
            .collect(),
    )
}

fn coordinates_are_bounded(values: [i64; 3]) -> bool {
    values
        .iter()
        .all(|value| value.unsigned_abs() <= CAMERA_POSITION_LIMIT_MICROMETRES as u64)
}

#[cfg(test)]
mod tests {
    use crate::ids::AssetId;

    use super::*;

    #[test]
    fn camera_order_and_batch_boundaries_are_canonical() {
        let epoch = domain_hash("test.camera.epoch", b"epoch");
        let first = record(epoch, 2, 1);
        let second = record(epoch, 1, 0);
        let forward = build_camera_batches(epoch, vec![first.clone(), second.clone()], 1)
            .expect("camera batches");
        let reverse = build_camera_batches(epoch, vec![second, first], 1).expect("camera batches");
        assert_eq!(forward, reverse);
        validate_camera_batches(epoch, &forward).expect("valid camera batches");
    }

    #[test]
    fn duplicate_camera_key_and_invalid_cut_policy_are_rejected() {
        let epoch = domain_hash("test.camera.epoch", b"epoch");
        let duplicate = record(epoch, 1, 0);
        assert_eq!(
            build_camera_batches(epoch, vec![duplicate.clone(), duplicate], 8),
            Err(PresentationContractError::DuplicateCameraKey)
        );
        assert_eq!(
            CameraPresentationRecordV2::new(
                epoch,
                PersistentId::from_bytes([3; 16]),
                CameraRoleV1::PrimaryThirdPerson,
                CameraViewportV1::full(0),
                projection(),
                intent(),
                result(),
                result(),
                exposure(),
                true,
                CameraInterpolationPolicyV1::LinearPose,
            ),
            Err(PresentationContractError::InvalidCameraPolicy)
        );
        let mut noncanonical_yaw = intent();
        noncanonical_yaw.orbit_yaw_millidegrees = 180_000;
        assert_eq!(
            CameraPresentationRecordV2::new(
                epoch,
                PersistentId::from_bytes([4; 16]),
                CameraRoleV1::PrimaryThirdPerson,
                CameraViewportV1::full(0),
                projection(),
                noncanonical_yaw,
                result(),
                result(),
                exposure(),
                false,
                CameraInterpolationPolicyV1::LinearPose,
            ),
            Err(PresentationContractError::InvalidCameraIntent)
        );
    }

    fn record(epoch: ContentHash, camera_id: u8, viewport_id: u16) -> CameraPresentationRecordV2 {
        CameraPresentationRecordV2::new(
            epoch,
            PersistentId::from_bytes([camera_id; 16]),
            CameraRoleV1::PrimaryThirdPerson,
            CameraViewportV1::full(viewport_id),
            projection(),
            intent(),
            result(),
            result(),
            exposure(),
            false,
            CameraInterpolationPolicyV1::LinearPose,
        )
        .expect("camera record")
    }

    fn projection() -> CameraProjectionProfileV1 {
        CameraProjectionProfileV1::new(60_000, 100_000, 100_000_000).expect("projection")
    }

    fn intent() -> ThirdPersonCameraIntentSampleV1 {
        ThirdPersonCameraIntentSampleV1 {
            focus_subject_id: Some(PersistentId::from_bytes([7; 16])),
            focus_point_micrometres: [0, 1_000_000, 0],
            orbit_yaw_millidegrees: 0,
            orbit_pitch_millidegrees: -15_000,
            distance_micrometres: 3_000_000,
            shoulder_offset_micrometres: [350_000, 0, 0],
        }
    }

    fn result() -> CameraResultSampleV1 {
        CameraResultSampleV1 {
            pose: QuantizedPresentationTransformV1 {
                translation_micrometres: [0, 2_000_000, 3_000_000],
                ..QuantizedPresentationTransformV1::default()
            },
            focus_point_micrometres: [0, 1_000_000, 0],
        }
    }

    fn exposure() -> AssetRevisionRefV1 {
        AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([8; 16]),
            record_sha256: domain_hash("test.camera.exposure", b"exposure"),
        }
    }
}
