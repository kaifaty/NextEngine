#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2};
use next_contracts::presentation::{
    CameraInterpolationPolicyV1, CameraPresentationRecordV2, CameraProjectionProfileV1,
    CameraResultSampleV1, CameraRoleV1, CameraViewportV1, PresentationContractError,
    PresentationObjectKeyV1, PresentationRoleV1, PresentationSnapshotV2,
    QuantizedPresentationTransformV1, ScenePresentationFlagsV1, ScenePresentationRecordV2,
    ThirdPersonCameraIntentSampleV1,
};
use next_contracts::project::{AssetRevisionRefV1, domain_hash};
use next_contracts::render_content::AabbI64V1;

mod recovery_codec;

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
    accepted_snapshot: Option<PresentationSnapshotV2>,
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
        if max_scene_records_per_batch == 0 || max_camera_records_per_batch == 0 {
            return Err(PresentationExtractionError::InvalidProfile);
        }
        Ok(Self {
            snapshot_epoch,
            next_snapshot_sequence: 0,
            presentation_profile_hash,
            max_scene_records_per_batch,
            max_camera_records_per_batch,
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
        let (snapshot, max_scene_records_per_batch, max_camera_records_per_batch) =
            recovery_codec::decode_snapshot(
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
            accepted_snapshot: Some(snapshot),
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
    ) -> Result<(Self, PresentationSnapshotV2), PresentationExtractionError> {
        let (persisted, max_scene_records_per_batch, max_camera_records_per_batch) =
            recovery_codec::decode_snapshot(
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
    ) -> Result<&PresentationSnapshotV2, PresentationExtractionError> {
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
    ) -> Result<&PresentationSnapshotV2, PresentationExtractionError> {
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
        let candidate = PresentationSnapshotV2::new_with_camera_records(
            self.snapshot_epoch,
            self.next_snapshot_sequence,
            simulation_tick,
            project_composition_lock_hash,
            content_manifest_hash,
            self.presentation_profile_hash,
            records,
            camera_records,
            self.max_scene_records_per_batch,
            self.max_camera_records_per_batch,
            domain_hash("nextengine.presentation.environment.empty.v1", &[]),
        )?;
        candidate.validate()?;
        self.next_snapshot_sequence = self
            .next_snapshot_sequence
            .checked_add(1)
            .ok_or(PresentationExtractionError::SequenceOverflow)?;
        self.accepted_snapshot = Some(candidate);
        self.accepted_snapshot
            .as_ref()
            .ok_or(PresentationExtractionError::PublicationFailed)
    }

    #[must_use]
    pub fn accepted_snapshot(&self) -> Option<&PresentationSnapshotV2> {
        self.accepted_snapshot.as_ref()
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
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use next_contracts::ids::{AssetId, PhysicsWorldId};
    use next_contracts::input::TickRateProfileV1;
    use next_contracts::physics::{
        AuthoritativeNumericProfileV1, PhysicsCoordinateProfileV1, PhysicsLimitsProfileV1,
        PhysicsQuantizationProfileV1, PhysicsSolverSemanticsProfileV1,
        PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    };

    #[test]
    fn binding_order_does_not_change_snapshot_and_failed_candidate_is_not_published() {
        let lock = domain_hash("test.lock", b"lock");
        let content = domain_hash("test.content", b"content");
        let mut first =
            PresentationExtractorV1::new(lock, domain_hash("test.profile", b"profile"), 2)
                .expect("extractor");
        let mut second = first.clone();
        let physics = empty_physics();
        let mut bindings = vec![binding(2), binding(1)];
        let forward = first
            .extract(0, lock, content, &physics, &bindings)
            .expect("snapshot")
            .clone();
        bindings.reverse();
        let reverse = second
            .extract(0, lock, content, &physics, &bindings)
            .expect("snapshot")
            .clone();
        assert_eq!(forward, reverse);

        let prior = first.accepted_snapshot().expect("published").clone();
        assert!(
            first
                .extract(1, lock, content, &physics, &[binding(1), binding(1)],)
                .is_err()
        );
        assert_eq!(first.accepted_snapshot(), Some(&prior));
    }

    #[test]
    fn camera_extraction_carries_previous_sample_and_rejects_key_collisions_atomically() {
        let lock = domain_hash("test.lock", b"lock");
        let content = domain_hash("test.content", b"content");
        let mut extractor =
            PresentationExtractorV1::new(lock, domain_hash("test.profile", b"profile"), 2)
                .expect("extractor");
        let physics = empty_physics();
        let first_camera = camera_binding(0);
        extractor
            .extract_with_cameras(0, lock, content, &physics, &[], &[first_camera])
            .expect("first camera snapshot");
        let mut next_camera = first_camera;
        next_camera
            .current_result_sample
            .pose
            .translation_micrometres[0] = 1_000_000;
        let second = extractor
            .extract_with_cameras(1, lock, content, &physics, &[], &[next_camera])
            .expect("second camera snapshot")
            .clone();
        let record = second.camera_records().next().expect("camera");
        assert_eq!(
            record.previous_result_sample,
            first_camera.current_result_sample
        );
        assert_eq!(
            record.current_result_sample,
            next_camera.current_result_sample
        );

        let prior = extractor.accepted_snapshot().expect("published").clone();
        assert!(matches!(
            extractor.extract_with_cameras(
                2,
                lock,
                content,
                &physics,
                &[],
                &[next_camera, next_camera],
            ),
            Err(PresentationExtractionError::DuplicateCameraBinding)
        ));
        assert_eq!(extractor.accepted_snapshot(), Some(&prior));
    }

    #[test]
    fn recovery_round_trip_preserves_snapshot_sequence_and_interpolation_history() {
        let lock = domain_hash("test.recovery.lock", b"lock");
        let content = domain_hash("test.recovery.content", b"content");
        let mut extractor = PresentationExtractorV1::new_with_snapshot_epoch_and_batch_limits(
            domain_hash("test.recovery.epoch", b"epoch"),
            domain_hash("test.recovery.profile", b"profile"),
            2,
            1,
        )
        .expect("extractor");
        let physics = empty_physics();
        let initial_binding = binding(7);
        let initial_camera = camera_binding(0);
        let initial = extractor
            .extract_with_cameras(
                7,
                lock,
                content,
                &physics,
                &[initial_binding],
                &[initial_camera],
            )
            .expect("initial snapshot")
            .clone();

        let bytes = extractor.recovery_bytes().expect("recovery bytes");
        let mut resumed =
            PresentationExtractorV1::resume_from_recovery_bytes(&bytes).expect("resume extractor");
        assert_eq!(resumed.accepted_snapshot(), Some(&initial));
        assert_eq!(
            resumed.recovery_bytes().expect("canonical round trip"),
            bytes
        );

        let mut next_binding = initial_binding;
        next_binding.fallback_transform.translation_micrometres[0] = 2_000_000;
        let mut next_camera = initial_camera;
        next_camera
            .current_result_sample
            .pose
            .translation_micrometres[0] = 3_000_000;
        let next = resumed
            .extract_with_cameras(8, lock, content, &physics, &[next_binding], &[next_camera])
            .expect("next snapshot");
        assert_eq!(next.snapshot_sequence, initial.snapshot_sequence + 1);
        assert_eq!(
            next.scene_records()
                .next()
                .expect("scene record")
                .previous_transform,
            initial_binding.fallback_transform
        );
        assert_eq!(
            next.camera_records()
                .next()
                .expect("camera record")
                .previous_result_sample,
            initial_camera.current_result_sample
        );

        let mut malformed = bytes.clone();
        malformed[0] ^= 0xff;
        assert!(matches!(
            PresentationExtractorV1::resume_from_recovery_bytes(&malformed),
            Err(PresentationExtractionError::RecoverySnapshotInvalid)
        ));
        let mut trailing = bytes;
        trailing.push(0);
        assert!(matches!(
            PresentationExtractorV1::resume_from_recovery_bytes(&trailing),
            Err(PresentationExtractionError::RecoverySnapshotInvalid)
        ));
    }

    #[test]
    fn authoritative_recovery_requires_the_locked_profile_and_publishes_a_new_cut_epoch() {
        let lock = domain_hash("test.authoritative-recovery.lock", b"lock");
        let content = domain_hash("test.authoritative-recovery.content", b"content");
        let profile = domain_hash("test.authoritative-recovery.profile", b"profile");
        let mut extractor = PresentationExtractorV1::new_with_snapshot_epoch_and_batch_limits(
            domain_hash("test.authoritative-recovery.epoch", b"epoch"),
            profile,
            2,
            1,
        )
        .expect("extractor");
        let physics = empty_physics();
        let scene = binding(9);
        let camera = camera_binding(0);
        let persisted = extractor
            .extract_with_cameras(19, lock, content, &physics, &[scene], &[camera])
            .expect("persisted snapshot")
            .clone();
        let recovery_bytes = extractor.recovery_bytes().expect("recovery bytes");

        assert!(matches!(
            PresentationExtractorV1::begin_authoritative_recovery_from_bytes(
                &recovery_bytes,
                domain_hash("test.authoritative-recovery.foreign-profile", b"foreign"),
            ),
            Err(PresentationExtractionError::RecoveryProfileMismatch)
        ));

        let (mut recovered, evidence) =
            PresentationExtractorV1::begin_authoritative_recovery_from_bytes(
                &recovery_bytes,
                profile,
            )
            .expect("authoritative recovery");
        assert_eq!(evidence, persisted);
        assert!(recovered.accepted_snapshot().is_none());

        let mut cut_camera = camera;
        cut_camera.cut = true;
        cut_camera.interpolation_policy = CameraInterpolationPolicyV1::Hold;
        let cut = recovered
            .extract_with_cameras(19, lock, content, &physics, &[scene], &[cut_camera])
            .expect("recovery cut");
        assert_ne!(cut.snapshot_epoch, persisted.snapshot_epoch);
        assert_eq!(cut.snapshot_sequence, 0);
        assert_eq!(cut.presentation_profile_hash, profile);
        assert!(
            cut.scene_records()
                .all(|record| record.object_key.snapshot_epoch == cut.snapshot_epoch)
        );
        assert!(cut.camera_records().all(|record| {
            record.snapshot_epoch == cut.snapshot_epoch
                && record.cut
                && record.interpolation_policy == CameraInterpolationPolicyV1::Hold
                && record.previous_result_sample == record.current_result_sample
        }));
    }

    fn binding(id: u8) -> PresentationBindingV1 {
        PresentationBindingV1 {
            persistent_id: PersistentId::from_bytes([id; 16]),
            presentation_role: PresentationRoleV1::Item,
            incarnation: 0,
            presentation_layer: 3,
            mesh_revision: AssetRevisionRefV1 {
                asset_id: AssetId::from_bytes([id; 16]),
                record_sha256: domain_hash("test.mesh", &[id]),
            },
            material_revision: AssetRevisionRefV1 {
                asset_id: AssetId::from_bytes([id.saturating_add(32); 16]),
                record_sha256: domain_hash("test.material", &[id]),
            },
            instance_ordinal: 0,
            local_bounds: AabbI64V1::new([-1_000_000; 3], [1_000_001; 3]).expect("bounds"),
            feature_flags: ScenePresentationFlagsV1::NONE,
            physics_body_id: None,
            fallback_transform: QuantizedPresentationTransformV1::default(),
            visible: true,
        }
    }

    fn camera_binding(viewport_id: u16) -> CameraPresentationBindingV1 {
        CameraPresentationBindingV1::primary_third_person(
            PersistentId::from_bytes([0xc0; 16]),
            viewport_id,
            CameraProjectionProfileV1::new(60_000, 100_000, 100_000_000).expect("projection"),
            ThirdPersonCameraIntentSampleV1 {
                focus_subject_id: Some(PersistentId::from_bytes([1; 16])),
                focus_point_micrometres: [0, 1_000_000, 0],
                orbit_yaw_millidegrees: 0,
                orbit_pitch_millidegrees: -15_000,
                distance_micrometres: 3_000_000,
                shoulder_offset_micrometres: [350_000, 0, 0],
            },
            CameraResultSampleV1 {
                pose: QuantizedPresentationTransformV1 {
                    translation_micrometres: [0, 2_000_000, 3_000_000],
                    ..QuantizedPresentationTransformV1::default()
                },
                focus_point_micrometres: [0, 1_000_000, 0],
            },
            AssetRevisionRefV1 {
                asset_id: AssetId::from_bytes([0xe0; 16]),
                record_sha256: domain_hash("test.camera.exposure", b"exposure"),
            },
            false,
        )
    }

    fn empty_physics() -> PhysicsCanonicalSnapshotV2 {
        let tick_rate = TickRateProfileV1::at_30_hz();
        let quantization =
            PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
        let numeric =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
        let catalog = PhysicsWorldCatalogV1::new(
            PhysicsWorldId::from_bytes([9; 16]),
            PhysicsWorldCatalogProfilesV1 {
                coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinate"),
                limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
                solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
                tick_rate_hash: tick_rate.profile_hash().expect("tick-rate hash"),
                authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
                quantization_hash: quantization.profile_hash().expect("quantization hash"),
            },
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .expect("catalog");
        PhysicsCanonicalSnapshotV2::genesis(&catalog, &tick_rate, &numeric, &quantization)
            .expect("snapshot")
    }
}
