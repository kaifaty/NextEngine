use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{AssetId, PersistentId};
use next_render::{RenderTargetV1, build_b0_frame_plan};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "nextengine-reference-visual-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

#[test]
fn reference_visual_bindings_replace_markers_and_follow_rpg_state() {
    let root = test_root();
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v4(
        next_reference_game::project_source_v4().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let quest_giver_character_id = activated
        .project
        .world_routine_catalog_or_none
        .as_ref()
        .expect("reference routine catalog")
        .routine
        .subject_id;

    let driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("live driver");
    let initial = driver.state().expect("initial state");
    let initial_scene = initial
        .presentation_snapshot
        .scene_records()
        .collect::<Vec<_>>();
    assert_eq!(initial_scene.len(), 8);
    assert!(
        initial_scene
            .iter()
            .all(|record| record.mesh_revision.asset_id != AssetId::from_bytes([0x82; 16]))
    );
    let action = initial
        .presentation_snapshot
        .semantic_ui_records()
        .find(|record| {
            record.element.element_id.as_str() == next_reference_game::HUD_ACTION_ELEMENT_ID
        })
        .expect("initial next-action presentation");
    assert_eq!(
        action
            .element
            .text_or_none
            .as_ref()
            .expect("action text")
            .text_id
            .as_str(),
        next_reference_game::HUD_ACTION_ACCEPT_TEXT_ID
    );

    let player = initial_scene
        .iter()
        .find(|record| {
            record.object_key.presentation_role
                == next_contracts::presentation::PresentationRoleV1::PlayerAvatar
        })
        .expect("player presentation");
    assert_eq!(
        player.mesh_revision.asset_id,
        AssetId::from_bytes([0xc1; 16])
    );
    assert_eq!(
        player.material_revision.asset_id,
        AssetId::from_bytes([0xd1; 16])
    );
    let enemy = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x59; 16]))
        .expect("enemy presentation");
    assert_eq!(
        enemy.mesh_revision.asset_id,
        AssetId::from_bytes([0xc9; 16])
    );
    let quest_giver = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == quest_giver_character_id)
        .expect("quest-giver presentation");
    assert_eq!(
        quest_giver.mesh_revision.asset_id,
        AssetId::from_bytes([0xca; 16])
    );
    let focus_ring = initial_scene
        .iter()
        .find(|record| record.instance_ordinal == 240)
        .expect("focus ring presentation");
    assert_eq!(
        focus_ring.mesh_revision.asset_id,
        AssetId::from_bytes([0xc7; 16])
    );
    assert_eq!(
        focus_ring.material_revision.asset_id,
        AssetId::from_bytes([0xd7; 16])
    );
    let initial_plan = build_b0_frame_plan(
        &initial.presentation_snapshot,
        &activated.project.render_content_catalog,
        RenderTargetV1 {
            extent: [1_920, 1_080],
            target_revision: 1,
        },
    )
    .expect("initial visual plan");
    assert!(initial_plan.draws.iter().any(|draw| {
        draw.material_revision.asset_id == AssetId::from_bytes([0xd7; 16]) && !draw.casts_shadow
    }));
    assert!(initial_plan.draws.iter().all(|draw| {
        draw.material_revision.asset_id == AssetId::from_bytes([0xd7; 16]) || draw.casts_shadow
    }));

    let initial_pickup = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x5f; 16]))
        .expect("pickup presentation");
    assert_eq!(
        initial_pickup.mesh_revision.asset_id,
        AssetId::from_bytes([0xc2; 16])
    );
    assert!(initial_pickup.visible);

    let initial_relay = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x58; 16]))
        .expect("relay presentation");
    assert_eq!(
        initial_relay.mesh_revision.asset_id,
        AssetId::from_bytes([0xc3; 16])
    );
    assert_eq!(
        initial_relay.material_revision.asset_id,
        AssetId::from_bytes([0xd5; 16])
    );

    let environment_instances = initial_scene
        .iter()
        .filter(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x70; 16]))
        .collect::<Vec<_>>();
    assert_eq!(environment_instances.len(), 1);
    assert!(environment_instances.iter().all(|record| {
        record.object_key.presentation_role
            == next_contracts::presentation::PresentationRoleV1::Environment
            && record.visible
    }));
    let relay_approach = environment_instances
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x70; 16]))
        .expect("relay approach presentation");
    assert_eq!(
        relay_approach.mesh_revision.asset_id,
        AssetId::from_bytes([0xc6; 16])
    );
    assert_eq!(
        relay_approach.material_revision.asset_id,
        AssetId::from_bytes([0xd9; 16])
    );
    let outcome = next_reference_game::run_reference_game(activated, true).expect("reference run");
    let final_pickup = outcome
        .presentation_bindings
        .iter()
        .find(|binding| binding.persistent_id == outcome.pickup_item_id)
        .expect("final pickup binding");
    assert!(!final_pickup.visible);
    let final_enemy = outcome
        .presentation_bindings
        .iter()
        .find(|binding| binding.persistent_id == outcome.npc_character_id)
        .expect("final enemy binding");
    assert!(final_enemy.visible);
    assert_eq!(
        final_enemy.material_revision.asset_id,
        AssetId::from_bytes([0xd8; 16])
    );
    let final_relay = outcome
        .presentation_bindings
        .iter()
        .find(|binding| binding.persistent_id == outcome.interactive_object_id)
        .expect("final relay binding");
    assert_eq!(
        final_relay.material_revision.asset_id,
        AssetId::from_bytes([0xd6; 16])
    );
    assert_eq!(outcome.presentation_bindings.len(), 7);
    assert!(outcome.presentation_bindings.iter().all(|binding| {
        binding.persistent_id != PersistentId::from_bytes([0x70; 16])
            || (binding.presentation_role
                == next_contracts::presentation::PresentationRoleV1::Environment
                && binding.physics_body_id.is_none()
                && binding.visible)
    }));
    std::fs::remove_dir_all(root).expect("cleanup");
}
