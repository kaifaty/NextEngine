#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2};
use next_contracts::presentation::{
    CameraInterpolationPolicyV1, CameraPresentationRecordV2, CameraProjectionProfileV1,
    CameraResultSampleV1, CameraRoleV1, CameraViewportV1, CharacterSkinningPresentationRecordV1,
    PRESENTATION_DEFAULT_SEMANTIC_UI_RECORDS_PER_BATCH, PresentationContractError,
    PresentationObjectKeyV1, PresentationRoleV1, PresentationSnapshotV3,
    QuantizedPresentationTransformV1, ScenePresentationFlagsV1, ScenePresentationRecordV2,
    SemanticUiPresentationRecordV1, ThirdPersonCameraIntentSampleV1,
};
use next_contracts::project::{AssetRevisionRefV1, domain_hash};
use next_contracts::render_content::AabbI64V1;

mod recovery_codec;
mod text;
mod ui_font;
mod ui_overlay;

pub mod audio_mix;
pub mod audio_scene;

pub use text::{
    LocalizationDiagnosticV1, TEXT_RESOLUTION_MAX_DEPTH, TextCatalogResolverV1, TextResolutionV1,
    TextResolverErrorV1,
};
pub use ui_overlay::{UI_OVERLAY_TEXT_SCALE, UiOverlayImageV1, rasterize_semantic_ui};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PresentationBindingV1 {
    pub persistent_id: PersistentId,
    pub presentation_role: PresentationRoleV1,
    pub incarnation: u32,
    pub presentation_layer: u16,
    pub mesh_revision: AssetRevisionRefV1,
    pub material_revision: AssetRevisionRefV1,
    pub instance_ordinal: u32,
    pub local_bounds: AabbI64V1,
    pub feature_flags: ScenePresentationFlagsV1,
    pub physics_body_id: Option<PhysicsBodyIdV1>,
    pub fallback_transform: QuantizedPresentationTransformV1,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CameraPresentationBindingV1 {
    pub camera_id: PersistentId,
    pub camera_role: CameraRoleV1,
    pub viewport: CameraViewportV1,
    pub projection_profile: CameraProjectionProfileV1,
    pub intent_sample: ThirdPersonCameraIntentSampleV1,
    pub current_result_sample: CameraResultSampleV1,
    pub exposure_profile_revision: AssetRevisionRefV1,
    pub cut: bool,
    pub interpolation_policy: CameraInterpolationPolicyV1,
}

impl CameraPresentationBindingV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the helper keeps every authored third-person camera input explicit"
    )]
    #[must_use]
    pub const fn primary_third_person(
        camera_id: PersistentId,
        viewport_id: u16,
        projection_profile: CameraProjectionProfileV1,
        intent_sample: ThirdPersonCameraIntentSampleV1,
        current_result_sample: CameraResultSampleV1,
        exposure_profile_revision: AssetRevisionRefV1,
        cut: bool,
    ) -> Self {
        Self {
            camera_id,
            camera_role: CameraRoleV1::PrimaryThirdPerson,
            viewport: CameraViewportV1::full(viewport_id),
            projection_profile,
            intent_sample,
            current_result_sample,
            exposure_profile_revision,
            cut,
            interpolation_policy: if cut {
                CameraInterpolationPolicyV1::Hold
            } else {
                CameraInterpolationPolicyV1::LinearPose
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct PresentationExtractorV1 {
    snapshot_epoch: ContentHash,
    next_snapshot_sequence: u64,
    presentation_profile_hash: ContentHash,
    max_scene_records_per_batch: usize,
    max_camera_records_per_batch: usize,
    max_semantic_ui_records_per_batch: usize,
    accepted_snapshot: Option<Arc<PresentationSnapshotV3>>,
}

impl PresentationExtractorV1 {
    pub fn new(
        project_composition_lock_hash: ContentHash,
        presentation_profile_hash: ContentHash,
        max_scene_records_per_batch: usize,
    ) -> Result<Self, PresentationExtractionError> {
        Self::new_with_batch_limits(
            project_composition_lock_hash,
            presentation_profile_hash,
            max_scene_records_per_batch,
            max_scene_records_per_batch,
        )
    }

    pub fn new_with_batch_limits(
        project_composition_lock_hash: ContentHash,
        presentation_profile_hash: ContentHash,
        max_scene_records_per_batch: usize,
        max_camera_records_per_batch: usize,
    ) -> Result<Self, PresentationExtractionError> {
        Self::new_with_snapshot_epoch_and_batch_limits(
            domain_hash(
                "nextengine.presentation-snapshot-epoch.v1",
                project_composition_lock_hash.as_bytes(),
            ),
            presentation_profile_hash,
            max_scene_records_per_batch,
            max_camera_records_per_batch,
        )
    }

    pub fn new_with_snapshot_epoch_and_batch_limits(
        snapshot_epoch: ContentHash,
        presentation_profile_hash: ContentHash,
        max_scene_records_per_batch: usize,
        max_camera_records_per_batch: usize,
    ) -> Result<Self, PresentationExtractionError> {
        Self::new_with_snapshot_epoch_and_ui_batch_limits(
            snapshot_epoch,
            presentation_profile_hash,
            max_scene_records_per_batch,
            max_camera_records_per_batch,
            PRESENTATION_DEFAULT_SEMANTIC_UI_RECORDS_PER_BATCH,
        )
    }

    pub fn new_with_snapshot_epoch_and_ui_batch_limits(
        snapshot_epoch: ContentHash,
        presentation_profile_hash: ContentHash,
        max_scene_records_per_batch: usize,
        max_camera_records_per_batch: usize,
        max_semantic_ui_records_per_batch: usize,
    ) -> Result<Self, PresentationExtractionError> {
        if max_scene_records_per_batch == 0
            || max_camera_records_per_batch == 0
            || max_semantic_ui_records_per_batch == 0
        {
            return Err(PresentationExtractionError::InvalidProfile);
        }
        Ok(Self {
            snapshot_epoch,
            next_snapshot_sequence: 0,
            presentation_profile_hash,
            max_scene_records_per_batch,
            max_camera_records_per_batch,
            max_semantic_ui_records_per_batch,
            accepted_snapshot: None,
        })
    }

    /// Restores the exact last accepted presentation publication for
    /// byte-evidence validation and non-authoritative continuation.
    ///
    /// Authoritative runtime recovery must use
    /// [`Self::begin_authoritative_recovery_from_bytes`] so the recovered
    /// publication cannot be mistaken for the prior presentation timeline.
    pub fn resume_from_recovery_bytes(bytes: &[u8]) -> Result<Self, PresentationExtractionError> {
        let (
            snapshot,
            max_scene_records_per_batch,
            max_camera_records_per_batch,
            max_semantic_ui_records_per_batch,
        ) = recovery_codec::decode_snapshot(
            bytes,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )
        .map_err(|()| PresentationExtractionError::RecoverySnapshotInvalid)?;
        let next_snapshot_sequence = snapshot
            .snapshot_sequence
            .checked_add(1)
            .ok_or(PresentationExtractionError::SequenceOverflow)?;
        Ok(Self {
            snapshot_epoch: snapshot.snapshot_epoch,
            next_snapshot_sequence,
            presentation_profile_hash: snapshot.presentation_profile_hash,
            max_scene_records_per_batch,
            max_camera_records_per_batch,
            max_semantic_ui_records_per_batch,
            accepted_snapshot: Some(Arc::new(snapshot)),
        })
    }

    /// Validates persisted presentation evidence against the caller's locked
    /// profile and creates an empty extractor for an authoritative recovery
    /// cut. The returned persisted snapshot is evidence only; the caller must
    /// rebuild the current projection through `extract*`, which publishes
    /// sequence zero under the derived recovery epoch.
    pub fn begin_authoritative_recovery_from_bytes(
        bytes: &[u8],
        expected_presentation_profile_hash: ContentHash,
    ) -> Result<(Self, PresentationSnapshotV3), PresentationExtractionError> {
        let (
            persisted,
            max_scene_records_per_batch,
            max_camera_records_per_batch,
            max_semantic_ui_records_per_batch,
        ) = recovery_codec::decode_snapshot(
            bytes,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )
        .map_err(|()| PresentationExtractionError::RecoverySnapshotInvalid)?;
        if persisted.presentation_profile_hash != expected_presentation_profile_hash {
            return Err(PresentationExtractionError::RecoveryProfileMismatch);
        }
        let recovery_epoch = domain_hash(
            "nextengine.presentation-authoritative-recovery-epoch.v1",
            persisted.canonical_hash.as_bytes(),
        );
        if recovery_epoch == persisted.snapshot_epoch {
            return Err(PresentationExtractionError::RecoverySnapshotInvalid);
        }
        Ok((
            Self {
                snapshot_epoch: recovery_epoch,
                next_snapshot_sequence: 0,
                presentation_profile_hash: expected_presentation_profile_hash,
                max_scene_records_per_batch,
                max_camera_records_per_batch,
                max_semantic_ui_records_per_batch,
                accepted_snapshot: None,
            },
            persisted,
        ))
    }

    pub fn recovery_bytes(&self) -> Result<Vec<u8>, PresentationExtractionError> {
        let snapshot = self
            .accepted_snapshot
            .as_ref()
            .ok_or(PresentationExtractionError::PublicationFailed)?;
        recovery_codec::encode_snapshot(
            snapshot,
            self.max_scene_records_per_batch,
            self.max_camera_records_per_batch,
            self.max_semantic_ui_records_per_batch,
        )
        .map_err(|()| PresentationExtractionError::RecoverySnapshotInvalid)
    }

    pub fn extract(
        &mut self,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        physics: &PhysicsCanonicalSnapshotV2,
        bindings: &[PresentationBindingV1],
    ) -> Result<&PresentationSnapshotV3, PresentationExtractionError> {
        self.extract_with_cameras(
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            physics,
            bindings,
            &[],
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "scene and camera projection inputs meet at one atomic snapshot boundary"
    )]
    pub fn extract_with_cameras(
        &mut self,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        physics: &PhysicsCanonicalSnapshotV2,
        bindings: &[PresentationBindingV1],
        camera_bindings: &[CameraPresentationBindingV1],
    ) -> Result<&PresentationSnapshotV3, PresentationExtractionError> {
        self.extract_with_cameras_and_semantic_ui(
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            physics,
            bindings,
            camera_bindings,
            Vec::new(),
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "scene, camera and semantic UI projection inputs meet at one atomic snapshot boundary"
    )]
    pub fn extract_with_cameras_and_semantic_ui(
        &mut self,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        physics: &PhysicsCanonicalSnapshotV2,
        bindings: &[PresentationBindingV1],
        camera_bindings: &[CameraPresentationBindingV1],
        semantic_ui_records: Vec<SemanticUiPresentationRecordV1>,
    ) -> Result<&PresentationSnapshotV3, PresentationExtractionError> {
        self.extract_with_character_skinning(
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            physics,
            bindings,
            camera_bindings,
            semantic_ui_records,
            Vec::new(),
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "all immutable presentation families meet at one atomic successor boundary"
    )]
    pub fn extract_with_character_skinning(
        &mut self,
        simulation_tick: u64,
        project_composition_lock_hash: ContentHash,
        content_manifest_hash: ContentHash,
        physics: &PhysicsCanonicalSnapshotV2,
        bindings: &[PresentationBindingV1],
        camera_bindings: &[CameraPresentationBindingV1],
        semantic_ui_records: Vec<SemanticUiPresentationRecordV1>,
        character_skinning_records: Vec<CharacterSkinningPresentationRecordV1>,
    ) -> Result<&PresentationSnapshotV3, PresentationExtractionError> {
        let mut canonical_bindings = bindings.to_vec();
        canonical_bindings.sort();
        if canonical_bindings.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(PresentationExtractionError::DuplicateBinding);
        }
        let records = canonical_bindings
            .iter()
            .map(|binding| {
                let current = binding
                    .physics_body_id
                    .and_then(|body_id| physics.sorted_body_states.get(&body_id))
                    .map_or(binding.fallback_transform, |body| {
                        QuantizedPresentationTransformV1 {
                            translation_micrometres: body.pose.translation_micrometres,
                            orientation_q30: body.pose.rotation_q1_30,
                        }
                    });
                let previous = self
                    .accepted_snapshot
                    .as_ref()
                    .and_then(|snapshot| {
                        snapshot.scene_records().find(|record| {
                            record.object_key.persistent_id == binding.persistent_id
                                && record.object_key.presentation_role == binding.presentation_role
                                && record.object_key.incarnation == binding.incarnation
                        })
                    })
                    .map_or(current, |record| record.current_transform);
                ScenePresentationRecordV2::new(
                    binding.presentation_layer,
                    PresentationObjectKeyV1 {
                        snapshot_epoch: self.snapshot_epoch,
                        persistent_id: binding.persistent_id,
                        presentation_role: binding.presentation_role,
                        incarnation: binding.incarnation,
                    },
                    binding.mesh_revision,
                    binding.material_revision,
                    binding.instance_ordinal,
                    binding.local_bounds,
                    binding.feature_flags,
                    previous,
                    current,
                    binding.visible,
                )
            })
            .collect();
        let mut canonical_camera_bindings = camera_bindings.to_vec();
        canonical_camera_bindings.sort_by_key(|binding| {
            (
                binding.viewport.viewport_id,
                binding.camera_role,
                binding.camera_id,
            )
        });
        if canonical_camera_bindings.windows(2).any(|pair| {
            (
                pair[0].viewport.viewport_id,
                pair[0].camera_role,
                pair[0].camera_id,
            ) == (
                pair[1].viewport.viewport_id,
                pair[1].camera_role,
                pair[1].camera_id,
            )
        }) {
            return Err(PresentationExtractionError::DuplicateCameraBinding);
        }
        let camera_records = canonical_camera_bindings
            .iter()
            .map(|binding| {
                let previous_result_sample = if binding.cut {
                    binding.current_result_sample
                } else {
                    self.accepted_snapshot
                        .as_ref()
                        .and_then(|snapshot| {
                            snapshot.camera_records().find(|record| {
                                record.viewport.viewport_id == binding.viewport.viewport_id
                                    && record.camera_role == binding.camera_role
                                    && record.camera_id == binding.camera_id
                            })
                        })
                        .map_or(binding.current_result_sample, |record| {
                            record.current_result_sample
                        })
                };
                CameraPresentationRecordV2::new(
                    self.snapshot_epoch,
                    binding.camera_id,
                    binding.camera_role,
                    binding.viewport,
                    binding.projection_profile,
                    binding.intent_sample,
                    previous_result_sample,
                    binding.current_result_sample,
                    binding.exposure_profile_revision,
                    binding.cut,
                    binding.interpolation_policy,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let candidate = PresentationSnapshotV3::new_with_character_skinning_records(
            self.snapshot_epoch,
            self.next_snapshot_sequence,
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            self.presentation_profile_hash,
            records,
            camera_records,
            semantic_ui_records,
            character_skinning_records,
            self.max_scene_records_per_batch,
            self.max_camera_records_per_batch,
            self.max_semantic_ui_records_per_batch,
            domain_hash("nextengine.presentation.environment.empty.v1", &[]),
        )?;
        // The constructor above already enforces the complete canonical
        // validation surface for a freshly built candidate: per-record
        // validation, snapshot-epoch match, canonical sort and key uniqueness
        // for both record families, canonical batch partitioning, family
        // limits and the published canonical hash. Re-running `validate()`
        // only re-verifies those same invariants and recomputes the identical
        // snapshot hash, which dominated extraction cost (~50%). Keep the
        // full check in debug/test builds as a constructor regression gate;
        // decode, restore and recovery paths still call `validate()` on
        // untrusted bytes.
        debug_assert!(
            candidate.validate().is_ok(),
            "freshly constructed presentation candidate must be valid"
        );
        self.next_snapshot_sequence = self
            .next_snapshot_sequence
            .checked_add(1)
            .ok_or(PresentationExtractionError::SequenceOverflow)?;
        self.accepted_snapshot = Some(Arc::new(candidate));
        self.accepted_snapshot
            .as_deref()
            .ok_or(PresentationExtractionError::PublicationFailed)
    }

    /// Returns the snapshot epoch this extractor stamps into every published
    /// record family. Projection callers need it to build semantic UI records
    /// for the same publication boundary.
    #[must_use]
    pub const fn snapshot_epoch(&self) -> ContentHash {
        self.snapshot_epoch
    }

    #[must_use]
    pub fn accepted_snapshot(&self) -> Option<&PresentationSnapshotV3> {
        self.accepted_snapshot.as_deref()
    }

    /// Returns shared ownership of the last accepted immutable publication.
    ///
    /// Callers that retain a snapshot across a staged commit can use this
    /// projection without cloning its scene and camera records.
    #[must_use]
    pub fn accepted_snapshot_shared(&self) -> Option<Arc<PresentationSnapshotV3>> {
        self.accepted_snapshot.clone()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PresentationExtractionError {
    Contract(PresentationContractError),
    InvalidProfile,
    DuplicateBinding,
    DuplicateCameraBinding,
    SequenceOverflow,
    PublicationFailed,
    RecoverySnapshotInvalid,
    RecoveryProfileMismatch,
}

impl Display for PresentationExtractionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::InvalidProfile => formatter.write_str("presentation extraction profile invalid"),
            Self::DuplicateBinding => {
                formatter.write_str("presentation extraction binding duplicated")
            }
            Self::DuplicateCameraBinding => {
                formatter.write_str("presentation camera extraction binding duplicated")
            }
            Self::SequenceOverflow => {
                formatter.write_str("presentation snapshot sequence overflow")
            }
            Self::PublicationFailed => {
                formatter.write_str("presentation snapshot publication failed")
            }
            Self::RecoverySnapshotInvalid => {
                formatter.write_str("presentation recovery snapshot is invalid")
            }
            Self::RecoveryProfileMismatch => {
                formatter.write_str("presentation recovery profile does not match")
            }
        }
    }
}

impl Error for PresentationExtractionError {}

impl From<PresentationContractError> for PresentationExtractionError {
    fn from(error: PresentationContractError) -> Self {
        Self::Contract(error)
    }
}

#[cfg(test)]
mod tests;
