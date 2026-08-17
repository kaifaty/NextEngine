use super::*;
use crate::animation_content::NeutralTransformV1;
use crate::ids::{AssetId, SchemaId};

#[test]
fn extraction_order_and_batch_profile_produce_canonical_records() {
    let epoch = domain_hash("test.presentation.epoch", b"epoch");
    let mut records = vec![
        record(epoch, 2, PresentationRoleV1::Item),
        record(epoch, 1, PresentationRoleV1::PlayerAvatar),
    ];
    let forward = PresentationSnapshotV3::new(
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
    let reverse = PresentationSnapshotV3::new(
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
        PresentationSnapshotV3::new(
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
        PresentationSnapshotV3::new(
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
        PresentationSnapshotV3::new(
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

#[test]
fn skinned_scene_requires_one_exact_pose_record_and_mesh_revision() {
    let epoch = domain_hash("test.presentation.skinning.epoch", b"epoch");
    let mut scene = record(epoch, 1, PresentationRoleV1::PlayerAvatar);
    scene = ScenePresentationRecordV2::new(
        scene.presentation_layer,
        scene.object_key,
        scene.mesh_revision,
        scene.material_revision,
        scene.instance_ordinal,
        scene.local_bounds,
        ScenePresentationFlagsV1::SKINNED,
        scene.previous_transform,
        scene.current_transform,
        scene.visible,
    );
    let build = |skinning_records| {
        PresentationSnapshotV3::new_with_character_skinning_records(
            epoch,
            0,
            0,
            domain_hash("test.lock", b"lock"),
            domain_hash("test.content", b"content"),
            domain_hash("test.profile", b"profile"),
            vec![scene.clone()],
            Vec::new(),
            Vec::new(),
            skinning_records,
            8,
            8,
            8,
            domain_hash("test.environment", b"environment"),
        )
    };
    assert_eq!(
        build(Vec::new()),
        Err(PresentationContractError::SkinningClosureInvalid)
    );
    let skinning_record = |mesh_revision| {
        CharacterSkinningPresentationRecordV1::new(
            scene.object_key,
            mesh_revision,
            asset_revision(41, "test.skinning-profile"),
            asset_revision(42, "test.skeleton"),
            asset_revision(43, "test.body-schema"),
            domain_hash("test.animation-profile", b"profile"),
            BaseSkinningProjectionModeV1::Sampled,
            vec![RenderJointPoseV1 {
                render_joint_id: SchemaId::new("test.render-joint.root").expect("joint id"),
                local_transform: NeutralTransformV1::translated([0, 0, 0]),
            }],
        )
        .expect("skinning record")
    };
    assert_eq!(
        build(vec![skinning_record(asset_revision(44, "test.other-mesh"))]),
        Err(PresentationContractError::SkinningClosureInvalid)
    );
    let valid = build(vec![skinning_record(scene.mesh_revision)]).expect("exact closure");
    valid.validate().expect("snapshot validates");
    assert_eq!(valid.character_skinning_records().count(), 1);
}

#[test]
fn semantic_ui_records_publish_typed_canonical_batches() {
    let epoch = domain_hash("test.presentation.epoch", b"epoch");
    let lock = domain_hash("test.lock", b"lock");
    let content = domain_hash("test.content", b"content");
    let profile = domain_hash("test.profile", b"profile");
    let environment = domain_hash("test.environment", b"environment");
    let mut ui_records = vec![
        ui_record(
            epoch,
            "nextengine.test.ui.surface.b",
            "nextengine.test.ui.panel.status",
            "nextengine.test.ui.element.mana",
            40,
        ),
        ui_record(
            epoch,
            "nextengine.test.ui.surface.a",
            "nextengine.test.ui.panel.status",
            "nextengine.test.ui.element.health",
            90,
        ),
    ];
    let forward = PresentationSnapshotV3::new_with_camera_and_semantic_ui_records(
        epoch,
        0,
        3,
        lock,
        content,
        profile,
        vec![record(epoch, 1, PresentationRoleV1::PlayerAvatar)],
        Vec::new(),
        ui_records.clone(),
        8,
        8,
        1,
        environment,
    )
    .expect("snapshot");
    ui_records.reverse();
    let reverse = PresentationSnapshotV3::new_with_camera_and_semantic_ui_records(
        epoch,
        0,
        3,
        lock,
        content,
        profile,
        vec![record(epoch, 1, PresentationRoleV1::PlayerAvatar)],
        Vec::new(),
        ui_records,
        8,
        8,
        1,
        environment,
    )
    .expect("snapshot");
    assert_eq!(forward, reverse);
    forward.validate().expect("valid");
    assert_eq!(forward.semantic_ui_batches.len(), 2);
    assert_eq!(forward.semantic_ui_records().count(), 2);
    assert!(
        forward
            .semantic_ui_records()
            .all(|ui| ui.snapshot_epoch == epoch)
    );
}

#[test]
fn duplicate_semantic_ui_element_key_rejects_snapshot() {
    let epoch = domain_hash("test.presentation.epoch", b"epoch");
    let duplicate = ui_record(
        epoch,
        "nextengine.test.ui.surface.a",
        "nextengine.test.ui.panel.status",
        "nextengine.test.ui.element.health",
        90,
    );
    assert_eq!(
        PresentationSnapshotV3::new_with_camera_and_semantic_ui_records(
            epoch,
            0,
            0,
            domain_hash("test.lock", b"lock"),
            domain_hash("test.content", b"content"),
            domain_hash("test.profile", b"profile"),
            Vec::new(),
            Vec::new(),
            vec![duplicate.clone(), duplicate],
            8,
            8,
            8,
            domain_hash("test.environment", b"environment"),
        ),
        Err(PresentationContractError::DuplicateUiElementKey)
    );
}

#[test]
fn semantic_ui_record_from_another_epoch_rejects_snapshot() {
    let snapshot_epoch = domain_hash("test.presentation.epoch", b"snapshot");
    let record_epoch = domain_hash("test.presentation.epoch", b"record");
    assert_eq!(
        PresentationSnapshotV3::new_with_camera_and_semantic_ui_records(
            snapshot_epoch,
            0,
            0,
            domain_hash("test.lock", b"lock"),
            domain_hash("test.content", b"content"),
            domain_hash("test.profile", b"profile"),
            Vec::new(),
            Vec::new(),
            vec![ui_record(
                record_epoch,
                "nextengine.test.ui.surface.a",
                "nextengine.test.ui.panel.status",
                "nextengine.test.ui.element.health",
                90,
            )],
            8,
            8,
            8,
            domain_hash("test.environment", b"environment"),
        ),
        Err(PresentationContractError::SnapshotEpochMismatch)
    );
}

#[test]
fn tampered_semantic_ui_batch_fails_validation() {
    let epoch = domain_hash("test.presentation.epoch", b"epoch");
    let mut snapshot = PresentationSnapshotV3::new_with_camera_and_semantic_ui_records(
        epoch,
        0,
        0,
        domain_hash("test.lock", b"lock"),
        domain_hash("test.content", b"content"),
        domain_hash("test.profile", b"profile"),
        Vec::new(),
        Vec::new(),
        vec![ui_record(
            epoch,
            "nextengine.test.ui.surface.a",
            "nextengine.test.ui.panel.status",
            "nextengine.test.ui.element.health",
            90,
        )],
        8,
        8,
        8,
        domain_hash("test.environment", b"environment"),
    )
    .expect("snapshot");
    snapshot.semantic_ui_batches[0].records_root = ContentHash::default();
    assert_eq!(
        snapshot.validate(),
        Err(PresentationContractError::HashMismatch)
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
                    SchemaId::new("nextengine.test.ui.text.value").expect("text id"),
                    vec![UiTextArgumentV1::SignedInteger(current)],
                )
                .expect("text ref"),
            ),
            UiElementValueV1::Scalar {
                current,
                maximum: 100,
            },
            Vec::new(),
        )
        .expect("element"),
    )
    .expect("ui record")
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
