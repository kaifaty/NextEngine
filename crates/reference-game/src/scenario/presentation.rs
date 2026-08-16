use super::*;

pub(crate) fn fixture_presentation_bindings(
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
) -> Result<Vec<PresentationBindingV1>, ReferenceGameError> {
    let revision = |asset_id| {
        fixture
            .activated_project
            .content_manifest
            .body
            .asset_entries
            .iter()
            .find(|entry| entry.asset_revision.asset_id == asset_id)
            .map(|entry| entry.asset_revision)
            .ok_or(ReferenceGameError::PresentationAssetMissing)
    };
    let mesh = |asset_id| {
        let asset_revision = revision(asset_id)?;
        let bounds = fixture
            .activated_project
            .render_content_catalog
            .meshes()
            .iter()
            .find(|mesh| mesh.asset_id() == asset_revision.asset_id)
            .map(next_contracts::render_content::NeutralMeshV1::bounds)
            .ok_or(ReferenceGameError::PresentationAssetMissing)?;
        Ok::<_, ReferenceGameError>((asset_revision, bounds))
    };
    let (floor_mesh, floor_bounds) = mesh(crate::source::REFERENCE_FLOOR_MESH_ASSET_ID)?;
    let (humanoid_mesh, humanoid_bounds) = mesh(crate::source::REFERENCE_HUMANOID_MESH_ASSET_ID)?;
    let (enemy_mesh, enemy_bounds) = mesh(crate::source::REFERENCE_ENEMY_MESH_ASSET_ID)?;
    let (quest_giver_mesh, quest_giver_bounds) =
        mesh(crate::source::REFERENCE_QUEST_GIVER_MESH_ASSET_ID)?;
    let (blade_mesh, blade_bounds) = mesh(crate::source::REFERENCE_BLADE_MESH_ASSET_ID)?;
    let (relay_mesh, relay_bounds) = mesh(crate::source::REFERENCE_RELAY_MESH_ASSET_ID)?;
    let (relay_approach_mesh, relay_approach_bounds) =
        mesh(crate::source::REFERENCE_RELAY_APPROACH_MESH_ASSET_ID)?;
    let (focus_ring_mesh, focus_ring_bounds) =
        mesh(crate::source::REFERENCE_FOCUS_RING_MESH_ASSET_ID)?;
    let (quest_marker_mesh, quest_marker_bounds) =
        mesh(crate::source::REFERENCE_QUEST_MARKER_MESH_ASSET_ID)?;
    let floor_material = revision(crate::source::REFERENCE_BASE_MATERIAL_ASSET_ID)?;
    let player_material = revision(crate::source::REFERENCE_PLAYER_MATERIAL_ASSET_ID)?;
    let enemy_material = revision(crate::source::REFERENCE_ENEMY_MATERIAL_ASSET_ID)?;
    let defeated_enemy_material =
        revision(crate::source::REFERENCE_DEFEATED_ENEMY_MATERIAL_ASSET_ID)?;
    let quest_giver_material = revision(crate::source::REFERENCE_QUEST_GIVER_MATERIAL_ASSET_ID)?;
    let blade_material = revision(crate::source::REFERENCE_BLADE_MATERIAL_ASSET_ID)?;
    let relay_inactive_material =
        revision(crate::source::REFERENCE_RELAY_INACTIVE_MATERIAL_ASSET_ID)?;
    let relay_active_material = revision(crate::source::REFERENCE_RELAY_ACTIVE_MATERIAL_ASSET_ID)?;
    let relay_approach_material =
        revision(crate::source::REFERENCE_RELAY_APPROACH_MATERIAL_ASSET_ID)?;
    let indicator_material = revision(crate::source::REFERENCE_INDICATOR_MATERIAL_ASSET_ID)?;

    let pickup_collected = matches!(
        aggregate_payload(rpg, RpgAggregateKindV1::Inventory, fixture.player_inventory_id),
        Some(RpgAggregatePayloadV1::Inventory(inventory))
            if inventory.item_ids.contains(&fixture.pickup_item_id)
    );
    let npc_alive =
        match aggregate_payload(rpg, RpgAggregateKindV1::Character, fixture.npc_character_id) {
            Some(RpgAggregatePayloadV1::Character(character)) => character
                .resources
                .iter()
                .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
                .is_none_or(|resource| resource.current_value > 0),
            _ => true,
        };
    let relay_active = matches!(
        aggregate_payload(
            rpg,
            RpgAggregateKindV1::InteractiveObject,
            fixture.interactive_object_id,
        ),
        Some(RpgAggregatePayloadV1::InteractiveObject(relay))
            if relay.state_id.as_str()
                == next_contracts::rpg::CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
    );
    let relay_material = if relay_active {
        relay_active_material
    } else {
        relay_inactive_material
    };
    let enemy_material = if npc_alive {
        enemy_material
    } else {
        defeated_enemy_material
    };
    let pickup_equipped = matches!(
        aggregate_payload(rpg, RpgAggregateKindV1::Equipment, fixture.player_equipment_id),
        Some(RpgAggregatePayloadV1::Equipment(equipment))
            if equipment
                .assignments
                .iter()
                .any(|assignment| assignment.item_id == fixture.pickup_item_id)
    );
    let quest_definition = fixture
        .activated_project
        .rpg_definitions
        .quests
        .first()
        .ok_or(ReferenceGameError::PresentationAssetMissing)?;
    let quest_state = match aggregate_payload(rpg, RpgAggregateKindV1::Quest, fixture.quest_id) {
        Some(RpgAggregatePayloadV1::Quest(quest)) => Some(&quest.state_id),
        _ => None,
    };
    let quest_complete = quest_state.is_some_and(|state| {
        !quest_definition
            .transitions
            .iter()
            .any(|transition| transition.source_state_id == *state)
    });
    let quest_offer = quest_state == Some(&quest_definition.entry_state_id);
    let focus_body_id = if quest_complete {
        None
    } else if quest_offer {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.quest_giver_character_id,
            body_slot: 0,
        })
    } else if !pickup_collected {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.pickup_proxy_id,
            body_slot: 0,
        })
    } else if !pickup_equipped {
        Some(fixture.physics_body_id)
    } else if npc_alive {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.npc_character_id,
            body_slot: 0,
        })
    } else if !relay_active {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.interactive_object_id,
            body_slot: 0,
        })
    } else {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.quest_giver_character_id,
            body_slot: 0,
        })
    };
    let mut bindings = vec![
        PresentationBindingV1 {
            persistent_id: PersistentId::from_bytes([0x57; 16]),
            presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
            incarnation: 0,
            presentation_layer: 0,
            mesh_revision: floor_mesh,
            material_revision: floor_material,
            instance_ordinal: 0,
            local_bounds: floor_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([0x57; 16]),
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.body_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::PlayerAvatar,
            incarnation: 0,
            presentation_layer: 1,
            mesh_revision: humanoid_mesh,
            material_revision: player_material,
            instance_ordinal: 0,
            local_bounds: humanoid_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(fixture.physics_body_id),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.interactive_object_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::InteractiveObject,
            incarnation: 0,
            presentation_layer: 2,
            mesh_revision: relay_mesh,
            material_revision: relay_material,
            instance_ordinal: 0,
            local_bounds: relay_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.interactive_object_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.pickup_item_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::Item,
            incarnation: 0,
            presentation_layer: 3,
            mesh_revision: blade_mesh,
            material_revision: blade_material,
            instance_ordinal: 0,
            local_bounds: blade_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.pickup_proxy_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: !pickup_collected,
        },
        PresentationBindingV1 {
            persistent_id: fixture.npc_character_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::Character,
            incarnation: 0,
            presentation_layer: 4,
            mesh_revision: enemy_mesh,
            material_revision: enemy_material,
            instance_ordinal: 0,
            local_bounds: enemy_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.npc_character_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            // The current grounded-capsule checkpoint keeps this solid body
            // for exact contact/replay continuation. Keep a darkened defeated
            // body visible so its collider never turns into an invisible
            // obstacle; physical removal belongs to the later topology path.
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.quest_giver_character_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::Character,
            incarnation: 0,
            presentation_layer: 5,
            mesh_revision: quest_giver_mesh,
            material_revision: quest_giver_material,
            instance_ordinal: 0,
            local_bounds: quest_giver_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.quest_giver_character_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
    ];

    // V1-alpha environment composition is presentation-only. One batched
    // neutral kit piece combines the relay approach and rock field so the
    // visual landmarks do not multiply B0 draw submissions. It comes through
    // the same cooked content and extraction boundary as gameplay-owned actors
    // and never becomes a second source of world or collision truth.
    let static_environment = |persistent_byte,
                              presentation_layer,
                              mesh_revision,
                              material_revision,
                              local_bounds,
                              translation_micrometres,
                              orientation_q30| PresentationBindingV1 {
        persistent_id: PersistentId::from_bytes([persistent_byte; 16]),
        presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
        incarnation: 0,
        presentation_layer,
        mesh_revision,
        material_revision,
        instance_ordinal: 0,
        local_bounds,
        feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
        physics_body_id: None,
        fallback_transform: next_contracts::presentation::QuantizedPresentationTransformV1 {
            translation_micrometres,
            orientation_q30,
        },
        visible: true,
    };
    const IDENTITY_Q30: [i32; 4] = [0, 0, 0, 1 << 30];
    bindings.push(static_environment(
        0x70,
        10,
        relay_approach_mesh,
        relay_approach_material,
        relay_approach_bounds,
        [0, 0, 0],
        IDENTITY_Q30,
    ));
    if let Some(focus_body_id) = focus_body_id {
        bindings.push(PresentationBindingV1 {
            persistent_id: PersistentId::from_bytes([0x75; 16]),
            presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
            incarnation: 0,
            presentation_layer: 240,
            mesh_revision: focus_ring_mesh,
            material_revision: indicator_material,
            instance_ordinal: 240,
            local_bounds: focus_ring_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(focus_body_id),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        });
        if !quest_offer {
            bindings.push(PresentationBindingV1 {
                persistent_id: PersistentId::from_bytes([0x76; 16]),
                presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
                incarnation: 0,
                presentation_layer: 241,
                mesh_revision: quest_marker_mesh,
                material_revision: indicator_material,
                instance_ordinal: 241,
                local_bounds: quest_marker_bounds,
                feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
                physics_body_id: Some(focus_body_id),
                fallback_transform:
                    next_contracts::presentation::QuantizedPresentationTransformV1::default(),
                visible: true,
            });
        }
    }
    Ok(bindings)
}
