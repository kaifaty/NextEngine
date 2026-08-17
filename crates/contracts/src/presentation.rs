use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::ids::{ContentHash, PersistentId};
use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::project::{AssetRevisionRefV1, domain_hash};
use crate::render_content::AabbI64V1;

mod camera;
mod skinning;
mod ui;

pub mod audio_scene;

pub use camera::{
    CAMERA_PRESENTATION_RECORD_SCHEMA_VERSION, CameraInterpolationPolicyV1,
    CameraPresentationBatchV1, CameraPresentationRecordV2, CameraProjectionProfileV1,
    CameraResultSampleV1, CameraRoleV1, CameraViewportV1, PRESENTATION_MAX_CAMERA_RECORDS,
    ThirdPersonCameraIntentSampleV1,
};
pub use skinning::{
    BaseSkinningProjectionModeV1, CHARACTER_SKINNING_PRESENTATION_RECORD_SCHEMA_VERSION,
    CharacterSkinningPresentationBatchV1, CharacterSkinningPresentationRecordV1,
    PRESENTATION_MAX_CHARACTER_SKINNING_RECORDS, PRESENTATION_MAX_RENDER_JOINT_POSES,
    RenderJointPoseV1,
};
pub use ui::{
    PRESENTATION_DEFAULT_SEMANTIC_UI_RECORDS_PER_BATCH, PRESENTATION_MAX_SEMANTIC_UI_RECORDS,
    SEMANTIC_UI_PRESENTATION_RECORD_SCHEMA_VERSION, SemanticUiPresentationBatchV1,
    SemanticUiPresentationRecordV1, UI_MAX_AFFORDANCES_PER_ELEMENT, UI_MAX_ELEMENTS_PER_PANEL,
    UI_MAX_FOCUS_ORDER, UI_MAX_PANELS_PER_SURFACE, UI_MAX_TEXT_ARGUMENTS,
    UI_SEMANTIC_SNAPSHOT_SCHEMA_VERSION, UiAccessibilityRoleV1, UiActionAffordanceV1,
    UiElementFocusKeyV1, UiElementRoleV1, UiElementValueV1, UiSemanticElementV1, UiSemanticPanelV1,
    UiSemanticSnapshotV1, UiStyleRoleV1, UiTextArgumentV1, UiTextRefV1, build_semantic_ui_batches,
    validate_semantic_ui_batches,
};

use camera::{build_camera_batches, camera_batches_value, validate_camera_batches};
use skinning::{
    build_character_skinning_batches, character_skinning_batches_value,
    validate_character_skinning_batches,
};
use ui::semantic_ui_batches_value;

pub const PRESENTATION_SNAPSHOT_SCHEMA_VERSION: u32 = 3;
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

    #[must_use]
    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
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
pub struct PresentationSnapshotV3 {
    pub schema_version: u32,
    pub snapshot_epoch: ContentHash,
    pub snapshot_sequence: u64,
    pub simulation_tick: u64,
    pub project_composition_lock_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub presentation_profile_hash: ContentHash,
    pub scene_batches: Vec<ScenePresentationBatchV1>,
    pub camera_batches: Vec<CameraPresentationBatchV1>,
    pub semantic_ui_batches: Vec<SemanticUiPresentationBatchV1>,
    pub character_skinning_batches: Vec<CharacterSkinningPresentationBatchV1>,
    pub cue_batches: Vec<ContentHash>,
    pub environment_batch: ContentHash,
    pub canonical_hash: ContentHash,
}

impl PresentationSnapshotV3 {
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
        scene_records: Vec<ScenePresentationRecordV2>,
        camera_records: Vec<CameraPresentationRecordV2>,
        max_scene_records_per_batch: usize,
        max_camera_records_per_batch: usize,
        environment_batch: ContentHash,
    ) -> Result<Self, PresentationContractError> {
        Self::new_with_camera_and_semantic_ui_records(
            snapshot_epoch,
            snapshot_sequence,
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            presentation_profile_hash,
            scene_records,
            camera_records,
            Vec::new(),
            max_scene_records_per_batch,
            max_camera_records_per_batch,
            PRESENTATION_DEFAULT_SEMANTIC_UI_RECORDS_PER_BATCH,
            environment_batch,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "scene, camera and semantic UI family limits are explicit at the atomic publication boundary"
    )]
    pub fn new_with_camera_and_semantic_ui_records(
        snapshot_epoch: ContentHash,
        snapshot_sequence: u64,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        presentation_profile_hash: ContentHash,
        scene_records: Vec<ScenePresentationRecordV2>,
        camera_records: Vec<CameraPresentationRecordV2>,
        semantic_ui_records: Vec<SemanticUiPresentationRecordV1>,
        max_scene_records_per_batch: usize,
        max_camera_records_per_batch: usize,
        max_semantic_ui_records_per_batch: usize,
        environment_batch: ContentHash,
    ) -> Result<Self, PresentationContractError> {
        Self::new_with_character_skinning_records(
            snapshot_epoch,
            snapshot_sequence,
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            presentation_profile_hash,
            scene_records,
            camera_records,
            semantic_ui_records,
            Vec::new(),
            max_scene_records_per_batch,
            max_camera_records_per_batch,
            max_semantic_ui_records_per_batch,
            environment_batch,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "all presentation families meet at one exact successor publication boundary"
    )]
    pub fn new_with_character_skinning_records(
        snapshot_epoch: ContentHash,
        snapshot_sequence: u64,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        presentation_profile_hash: ContentHash,
        mut scene_records: Vec<ScenePresentationRecordV2>,
        camera_records: Vec<CameraPresentationRecordV2>,
        semantic_ui_records: Vec<SemanticUiPresentationRecordV1>,
        character_skinning_records: Vec<CharacterSkinningPresentationRecordV1>,
        max_scene_records_per_batch: usize,
        max_camera_records_per_batch: usize,
        max_semantic_ui_records_per_batch: usize,
        environment_batch: ContentHash,
    ) -> Result<Self, PresentationContractError> {
        if max_scene_records_per_batch == 0
            || max_camera_records_per_batch == 0
            || max_semantic_ui_records_per_batch == 0
        {
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
        let semantic_ui_batches = build_semantic_ui_batches(
            snapshot_epoch,
            semantic_ui_records,
            max_semantic_ui_records_per_batch,
        )?;
        let character_skinning_batches = build_character_skinning_batches(
            snapshot_epoch,
            character_skinning_records,
            max_scene_records_per_batch,
        )?;
        validate_skinning_scene_closure(
            scene_batches.iter().flat_map(|batch| batch.records.iter()),
            character_skinning_batches
                .iter()
                .flat_map(|batch| batch.records.iter()),
        )?;
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
            semantic_ui_batches,
            character_skinning_batches,
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
        validate_semantic_ui_batches(self.snapshot_epoch, &self.semantic_ui_batches)?;
        validate_character_skinning_batches(self.snapshot_epoch, &self.character_skinning_batches)?;
        validate_skinning_scene_closure(self.scene_records(), self.character_skinning_records())?;
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

    pub fn semantic_ui_records(&self) -> impl Iterator<Item = &SemanticUiPresentationRecordV1> {
        self.semantic_ui_batches
            .iter()
            .flat_map(|batch| batch.records.iter())
    }

    pub fn character_skinning_records(
        &self,
    ) -> impl Iterator<Item = &CharacterSkinningPresentationRecordV1> {
        self.character_skinning_batches
            .iter()
            .flat_map(|batch| batch.records.iter())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.presentation-snapshot.v3",
            &encode_canonical_jcs(&object([
                ("camera_batches", camera_batches_value(&self.camera_batches)),
                (
                    "character_skinning_batches",
                    character_skinning_batches_value(&self.character_skinning_batches),
                ),
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
                    semantic_ui_batches_value(&self.semantic_ui_batches),
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
    DuplicateSkinningKey,
    InvalidSkinningPose,
    SkinningClosureInvalid,
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
    InvalidUiIdentifier,
    InvalidUiText,
    InvalidUiValue,
    InvalidUiAffordance,
    DuplicateUiElementKey,
    DuplicateUiPanelId,
    InvalidUiFocusGraph,
    UiSchemaIncompatible,
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
            Self::DuplicateSkinningKey => "presentation skinning key is duplicated",
            Self::InvalidSkinningPose => "presentation skinning pose is invalid",
            Self::SkinningClosureInvalid => "presentation skinning closure is invalid",
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
            Self::InvalidUiIdentifier => "semantic UI identifier or snapshot reference is invalid",
            Self::InvalidUiText => "semantic UI text reference is invalid",
            Self::InvalidUiValue => "semantic UI element value is invalid",
            Self::InvalidUiAffordance => "semantic UI action affordance is invalid",
            Self::DuplicateUiElementKey => "semantic UI element key is duplicated",
            Self::DuplicateUiPanelId => "semantic UI panel identifier is duplicated",
            Self::InvalidUiFocusGraph => "semantic UI focus graph is invalid",
            Self::UiSchemaIncompatible => "semantic UI schema is incompatible",
        })
    }
}

impl Error for PresentationContractError {}

fn validate_skinning_scene_closure<'a>(
    scene_records: impl Iterator<Item = &'a ScenePresentationRecordV2>,
    skinning_records: impl Iterator<Item = &'a CharacterSkinningPresentationRecordV1>,
) -> Result<(), PresentationContractError> {
    let scenes = scene_records
        .map(|record| (record.object_key, record))
        .collect::<BTreeMap<_, _>>();
    let skinning = skinning_records
        .map(|record| (record.object_key, record))
        .collect::<BTreeMap<_, _>>();
    for scene in scenes.values() {
        let uses_skinning = scene
            .feature_flags
            .contains(ScenePresentationFlagsV1::SKINNED);
        match skinning.get(&scene.object_key) {
            Some(record) if uses_skinning && record.mesh_revision == scene.mesh_revision => {}
            None if !uses_skinning => {}
            _ => return Err(PresentationContractError::SkinningClosureInvalid),
        }
    }
    if skinning.keys().any(|key| !scenes.contains_key(key)) {
        return Err(PresentationContractError::SkinningClosureInvalid);
    }
    Ok(())
}

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
mod tests;
