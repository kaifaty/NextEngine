use std::collections::BTreeMap;

use super::*;
use next_contracts::animation_content::NeutralTransformV1;
use next_contracts::ids::{AssetId, PhysicsWorldId, SchemaId};
use next_contracts::input::TickRateProfileV1;
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, PhysicsCoordinateProfileV1, PhysicsLimitsProfileV1,
    PhysicsQuantizationProfileV1, PhysicsSolverSemanticsProfileV1, PhysicsWorldCatalogProfilesV1,
    PhysicsWorldCatalogV1,
};
use next_contracts::presentation::{
    BaseSkinningProjectionModeV1, RenderJointPoseV1, UiAccessibilityRoleV1, UiActionAffordanceV1,
    UiElementRoleV1, UiElementValueV1, UiSemanticElementV1, UiStyleRoleV1, UiTextArgumentV1,
    UiTextRefV1,
};

#[test]
fn staged_extractor_clone_shares_only_the_immutable_prior_snapshot() {
    let lock = domain_hash("test.staged-clone.lock", b"lock");
    let content = domain_hash("test.staged-clone.content", b"content");
    let mut live = PresentationExtractorV1::new(lock, domain_hash("test.profile", b"profile"), 2)
        .expect("extractor");
    let physics = empty_physics();
    live.extract(0, lock, content, &physics, &[binding(1)])
        .expect("initial snapshot");

    let live_snapshot = live.accepted_snapshot_shared().expect("live snapshot");
    let mut staged = live.clone();
    let staged_snapshot = staged.accepted_snapshot_shared().expect("staged snapshot");
    assert!(Arc::ptr_eq(&live_snapshot, &staged_snapshot));
    staged
        .extract(1, lock, content, &physics, &[binding(1)])
        .expect("staged snapshot");

    let staged_snapshot = staged.accepted_snapshot_shared().expect("staged snapshot");
    assert!(!Arc::ptr_eq(&live_snapshot, &staged_snapshot));
    assert_eq!(
        live.accepted_snapshot()
            .expect("live snapshot")
            .simulation_tick,
        0
    );
    assert_eq!(
        staged
            .accepted_snapshot()
            .expect("staged snapshot")
            .simulation_tick,
        1,
    );
}

#[test]
fn binding_order_does_not_change_snapshot_and_failed_candidate_is_not_published() {
    let lock = domain_hash("test.lock", b"lock");
    let content = domain_hash("test.content", b"content");
    let mut first = PresentationExtractorV1::new(lock, domain_hash("test.profile", b"profile"), 2)
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
fn recovery_round_trip_preserves_exact_character_skinning_records() {
    let lock = domain_hash("test.skinning-recovery.lock", b"lock");
    let content = domain_hash("test.skinning-recovery.content", b"content");
    let epoch = domain_hash("test.skinning-recovery.epoch", b"epoch");
    let mut extractor = PresentationExtractorV1::new_with_snapshot_epoch_and_batch_limits(
        epoch,
        domain_hash("test.skinning-recovery.profile", b"profile"),
        2,
        1,
    )
    .expect("extractor");
    let mut scene = binding(11);
    scene.presentation_role = PresentationRoleV1::PlayerAvatar;
    scene.feature_flags = ScenePresentationFlagsV1::SKINNED;
    let skinning = CharacterSkinningPresentationRecordV1::new(
        PresentationObjectKeyV1 {
            snapshot_epoch: epoch,
            persistent_id: scene.persistent_id,
            presentation_role: scene.presentation_role,
            incarnation: scene.incarnation,
        },
        scene.mesh_revision,
        AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([0x51; 16]),
            record_sha256: domain_hash("test.skinning-profile", b"profile"),
        },
        AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([0x52; 16]),
            record_sha256: domain_hash("test.skinning-skeleton", b"skeleton"),
        },
        AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([0x53; 16]),
            record_sha256: domain_hash("test.skinning-body", b"body"),
        },
        domain_hash("test.skinning-animation-profile", b"animation"),
        BaseSkinningProjectionModeV1::Sampled,
        vec![RenderJointPoseV1 {
            render_joint_id: SchemaId::new("test.render-joint.root").expect("joint id"),
            local_transform: NeutralTransformV1::translated([0, 5_000, 0]),
        }],
    )
    .expect("skinning record");
    let snapshot = extractor
        .extract_with_character_skinning(
            7,
            lock,
            content,
            &empty_physics(),
            &[scene],
            &[],
            Vec::new(),
            vec![skinning],
        )
        .expect("skinned snapshot")
        .clone();
    let bytes = extractor.recovery_bytes().expect("recovery bytes");
    let resumed =
        PresentationExtractorV1::resume_from_recovery_bytes(&bytes).expect("resume extractor");
    assert_eq!(resumed.accepted_snapshot(), Some(&snapshot));
    assert_eq!(snapshot.character_skinning_records().count(), 1);
    assert_eq!(
        resumed.recovery_bytes().expect("canonical recovery bytes"),
        bytes
    );
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
        PresentationExtractorV1::begin_authoritative_recovery_from_bytes(&recovery_bytes, profile)
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

#[test]
fn semantic_ui_extraction_publishes_typed_batches_and_recovers_byte_exact() {
    let lock = domain_hash("test.ui-recovery.lock", b"lock");
    let content = domain_hash("test.ui-recovery.content", b"content");
    let mut extractor = PresentationExtractorV1::new_with_snapshot_epoch_and_ui_batch_limits(
        domain_hash("test.ui-recovery.epoch", b"epoch"),
        domain_hash("test.ui-recovery.profile", b"profile"),
        2,
        1,
        1,
    )
    .expect("extractor");
    let physics = empty_physics();
    let initial = extractor
        .extract_with_cameras_and_semantic_ui(
            7,
            lock,
            content,
            &physics,
            &[binding(7)],
            &[camera_binding(0)],
            vec![ui_record(
                extractor.snapshot_epoch(),
                "nextengine.test.ui.surface.hud",
                "nextengine.test.ui.panel.status",
                "nextengine.test.ui.element.health",
                90,
            )],
        )
        .expect("initial snapshot")
        .clone();
    assert_eq!(initial.semantic_ui_records().count(), 1);
    assert_eq!(
        initial
            .semantic_ui_records()
            .next()
            .expect("ui record")
            .element
            .value,
        UiElementValueV1::Scalar {
            current: 90,
            maximum: 100,
        }
    );

    let bytes = extractor.recovery_bytes().expect("recovery bytes");
    let mut resumed =
        PresentationExtractorV1::resume_from_recovery_bytes(&bytes).expect("resume extractor");
    assert_eq!(resumed.accepted_snapshot(), Some(&initial));
    assert_eq!(
        resumed.recovery_bytes().expect("canonical round trip"),
        bytes
    );

    let next = resumed
        .extract_with_cameras_and_semantic_ui(
            8,
            lock,
            content,
            &physics,
            &[binding(7)],
            &[camera_binding(0)],
            vec![ui_record(
                resumed.snapshot_epoch(),
                "nextengine.test.ui.surface.hud",
                "nextengine.test.ui.panel.status",
                "nextengine.test.ui.element.health",
                70,
            )],
        )
        .expect("next snapshot");
    assert_eq!(next.snapshot_sequence, initial.snapshot_sequence + 1);
    assert_eq!(
        next.semantic_ui_records()
            .next()
            .expect("ui record")
            .element
            .value,
        UiElementValueV1::Scalar {
            current: 70,
            maximum: 100,
        }
    );

    let prior = resumed.accepted_snapshot().expect("published").clone();
    assert!(matches!(
        resumed.extract_with_cameras_and_semantic_ui(
            9,
            lock,
            content,
            &physics,
            &[],
            &[],
            vec![ui_record(
                domain_hash("test.ui-recovery.foreign-epoch", b"foreign"),
                "nextengine.test.ui.surface.hud",
                "nextengine.test.ui.panel.status",
                "nextengine.test.ui.element.health",
                50,
            )],
        ),
        Err(PresentationExtractionError::Contract(
            PresentationContractError::SnapshotEpochMismatch
        ))
    ));
    assert_eq!(resumed.accepted_snapshot(), Some(&prior));

    let mut tampered = bytes;
    let middle = tampered.len() / 2;
    tampered[middle] ^= 0xff;
    assert!(matches!(
        PresentationExtractorV1::resume_from_recovery_bytes(&tampered),
        Err(PresentationExtractionError::RecoverySnapshotInvalid)
    ));
}

#[test]
fn subtitle_semantic_ui_role_recovers_byte_exact() {
    let lock = domain_hash("test.subtitle-recovery.lock", b"lock");
    let content = domain_hash("test.subtitle-recovery.content", b"content");
    let epoch = domain_hash("test.subtitle-recovery.epoch", b"epoch");
    let mut extractor = PresentationExtractorV1::new_with_snapshot_epoch_and_ui_batch_limits(
        epoch,
        domain_hash("test.subtitle-recovery.profile", b"profile"),
        2,
        1,
        2,
    )
    .expect("extractor");
    let subtitle = SemanticUiPresentationRecordV1::new(
        epoch,
        SchemaId::new("nextengine.test.ui.surface.subtitle").expect("surface id"),
        SchemaId::new("nextengine.test.ui.panel.subtitle").expect("panel id"),
        domain_hash("test.subtitle-recovery.source", b"subtitle"),
        UiSemanticElementV1::new(
            SchemaId::new("nextengine.test.ui.element.subtitle").expect("element id"),
            UiElementRoleV1::Subtitle,
            UiStyleRoleV1::Default,
            UiAccessibilityRoleV1::Standard,
            true,
            true,
            false,
            Some(
                UiTextRefV1::new(
                    SchemaId::new("nextengine.test.ui.text.subtitle").expect("text id"),
                    Vec::new(),
                )
                .expect("text ref"),
            ),
            UiElementValueV1::None,
            Vec::new(),
        )
        .expect("subtitle element"),
    )
    .expect("subtitle record");
    let snapshot = extractor
        .extract_with_cameras_and_semantic_ui(
            23,
            lock,
            content,
            &empty_physics(),
            &[],
            &[],
            vec![subtitle],
        )
        .expect("subtitle snapshot")
        .clone();

    let bytes = extractor.recovery_bytes().expect("subtitle recovery bytes");
    let resumed =
        PresentationExtractorV1::resume_from_recovery_bytes(&bytes).expect("resume subtitle");
    let recovered = resumed.accepted_snapshot().expect("recovered snapshot");
    assert_eq!(recovered, &snapshot);
    assert_eq!(
        recovered
            .semantic_ui_records()
            .next()
            .expect("recovered subtitle")
            .element
            .role,
        UiElementRoleV1::Subtitle
    );
    assert_eq!(
        resumed.recovery_bytes().expect("canonical round trip"),
        bytes
    );
}

fn ui_record(
    epoch: ContentHash,
    surface: &str,
    panel: &str,
    element: &str,
    current: i64,
) -> SemanticUiPresentationRecordV1 {
    SemanticUiPresentationRecordV1::new(
        epoch,
        SchemaId::new(surface).expect("surface id"),
        SchemaId::new(panel).expect("panel id"),
        domain_hash("test.ui-source", &[current as u8]),
        UiSemanticElementV1::new(
            SchemaId::new(element).expect("element id"),
            UiElementRoleV1::Meter,
            UiStyleRoleV1::Default,
            UiAccessibilityRoleV1::Status,
            true,
            true,
            false,
            Some(
                UiTextRefV1::new(
                    SchemaId::new("nextengine.test.ui.text.meter").expect("text id"),
                    vec![
                        UiTextArgumentV1::SignedInteger(current),
                        UiTextArgumentV1::SignedInteger(100),
                    ],
                )
                .expect("text ref"),
            ),
            UiElementValueV1::Scalar {
                current,
                maximum: 100,
            },
            vec![UiActionAffordanceV1 {
                action_id: SchemaId::new("nextengine.test.action.ui-confirm").expect("action id"),
                enabled: true,
            }],
        )
        .expect("element"),
    )
    .expect("ui record")
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
    let quantization = PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
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
