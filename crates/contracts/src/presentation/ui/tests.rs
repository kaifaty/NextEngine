use super::*;

fn id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("identifier")
}

fn epoch() -> ContentHash {
    domain_hash("test.ui.epoch", b"epoch")
}

fn element(id_suffix: &str) -> UiSemanticElementV1 {
    UiSemanticElementV1::new(
        id(&format!("nextengine.ui.element.{id_suffix}")),
        UiElementRoleV1::Button,
        UiStyleRoleV1::Default,
        UiAccessibilityRoleV1::Standard,
        true,
        true,
        false,
        Some(
            UiTextRefV1::new(
                id("nextengine.text.ok"),
                vec![
                    UiTextArgumentV1::SignedInteger(-7),
                    UiTextArgumentV1::TextId(id("nextengine.text.nested")),
                ],
            )
            .expect("text ref"),
        ),
        UiElementValueV1::Scalar {
            current: 3,
            maximum: 10,
        },
        vec![
            UiActionAffordanceV1 {
                action_id: id("nextengine.action.ui-confirm"),
                enabled: true,
            },
            UiActionAffordanceV1 {
                action_id: id("nextengine.action.ui-back"),
                enabled: false,
            },
        ],
    )
    .expect("element")
}

fn panel(path: &str, elements: Vec<UiSemanticElementV1>) -> UiSemanticPanelV1 {
    UiSemanticPanelV1::new(id(path), elements).expect("panel")
}

fn snapshot() -> UiSemanticSnapshotV1 {
    UiSemanticSnapshotV1::new(
        id("nextengine.ui.surface.hud"),
        vec![
            panel(
                "nextengine.ui.panel.status",
                vec![element("a"), element("b")],
            ),
            panel("nextengine.ui.panel.menu", vec![element("c")]),
        ],
        vec![
            UiElementFocusKeyV1 {
                semantic_path_id: id("nextengine.ui.panel.status"),
                element_id: id("nextengine.ui.element.a"),
            },
            UiElementFocusKeyV1 {
                semantic_path_id: id("nextengine.ui.panel.menu"),
                element_id: id("nextengine.ui.element.c"),
            },
        ],
        Some(UiElementFocusKeyV1 {
            semantic_path_id: id("nextengine.ui.panel.menu"),
            element_id: id("nextengine.ui.element.c"),
        }),
    )
    .expect("snapshot")
}

#[test]
fn snapshot_hash_is_stable_under_construction_order() {
    let forward = snapshot();
    let reverse = UiSemanticSnapshotV1::new(
        id("nextengine.ui.surface.hud"),
        vec![
            panel("nextengine.ui.panel.menu", vec![element("c")]),
            panel(
                "nextengine.ui.panel.status",
                vec![element("b"), element("a")],
            ),
        ],
        vec![
            UiElementFocusKeyV1 {
                semantic_path_id: id("nextengine.ui.panel.status"),
                element_id: id("nextengine.ui.element.a"),
            },
            UiElementFocusKeyV1 {
                semantic_path_id: id("nextengine.ui.panel.menu"),
                element_id: id("nextengine.ui.element.c"),
            },
        ],
        Some(UiElementFocusKeyV1 {
            semantic_path_id: id("nextengine.ui.panel.menu"),
            element_id: id("nextengine.ui.element.c"),
        }),
    )
    .expect("snapshot");
    assert_eq!(forward, reverse);
    forward.validate().expect("valid snapshot");
}

#[test]
fn duplicate_element_and_panel_ids_are_rejected() {
    assert_eq!(
        UiSemanticPanelV1::new(
            id("nextengine.ui.panel.x"),
            vec![element("a"), element("a")]
        ),
        Err(PresentationContractError::DuplicateUiElementKey)
    );
    let duplicate_panel = panel("nextengine.ui.panel.status", vec![element("z")]);
    assert_eq!(
        UiSemanticSnapshotV1::new(
            id("nextengine.ui.surface.hud"),
            vec![
                panel("nextengine.ui.panel.status", vec![element("a")]),
                duplicate_panel
            ],
            Vec::new(),
            None,
        ),
        Err(PresentationContractError::DuplicateUiPanelId)
    );
}

#[test]
fn focus_graph_rejects_unknown_duplicate_and_unlisted_focus() {
    let unknown = UiSemanticSnapshotV1::new(
        id("nextengine.ui.surface.hud"),
        vec![panel("nextengine.ui.panel.status", vec![element("a")])],
        vec![UiElementFocusKeyV1 {
            semantic_path_id: id("nextengine.ui.panel.status"),
            element_id: id("nextengine.ui.element.missing"),
        }],
        None,
    );
    assert_eq!(unknown, Err(PresentationContractError::InvalidUiFocusGraph));
    let key = UiElementFocusKeyV1 {
        semantic_path_id: id("nextengine.ui.panel.status"),
        element_id: id("nextengine.ui.element.a"),
    };
    let duplicate = UiSemanticSnapshotV1::new(
        id("nextengine.ui.surface.hud"),
        vec![panel("nextengine.ui.panel.status", vec![element("a")])],
        vec![key.clone(), key.clone()],
        None,
    );
    assert_eq!(
        duplicate,
        Err(PresentationContractError::InvalidUiFocusGraph)
    );
    let unlisted_focus = UiSemanticSnapshotV1::new(
        id("nextengine.ui.surface.hud"),
        vec![panel("nextengine.ui.panel.status", vec![element("a")])],
        Vec::new(),
        Some(key),
    );
    assert_eq!(
        unlisted_focus,
        Err(PresentationContractError::InvalidUiFocusGraph)
    );
}

#[test]
fn invalid_text_value_and_affordances_are_rejected() {
    let mut too_many_arguments = Vec::new();
    for index in 0..=UI_MAX_TEXT_ARGUMENTS {
        too_many_arguments.push(UiTextArgumentV1::SignedInteger(index as i64));
    }
    assert_eq!(
        UiTextRefV1::new(id("nextengine.text.ok"), too_many_arguments),
        Err(PresentationContractError::InvalidUiText)
    );
    let mut invalid_value = element("v");
    invalid_value.value = UiElementValueV1::Scalar {
        current: 11,
        maximum: 10,
    };
    assert_eq!(
        invalid_value.validate(),
        Err(PresentationContractError::InvalidUiValue)
    );
    let mut duplicate_affordance = element("w");
    duplicate_affordance.affordances.push(UiActionAffordanceV1 {
        action_id: id("nextengine.action.ui-back"),
        enabled: true,
    });
    assert_eq!(
        duplicate_affordance.validate(),
        Err(PresentationContractError::InvalidUiAffordance)
    );
}

#[test]
fn tampered_hash_is_detected_and_version_mismatch_is_incompatible() {
    let mut tampered = snapshot();
    tampered.surface_id = id("nextengine.ui.surface.other");
    assert_eq!(
        tampered.validate(),
        Err(PresentationContractError::HashMismatch)
    );
    let mut wrong_version = snapshot();
    wrong_version.schema_version = 0;
    assert_eq!(
        wrong_version.validate(),
        Err(PresentationContractError::UiSchemaIncompatible)
    );
}

fn record(surface: &str, path: &str, element_id: &str) -> SemanticUiPresentationRecordV1 {
    SemanticUiPresentationRecordV1::new(
        epoch(),
        id(surface),
        id(path),
        snapshot().canonical_hash,
        element(element_id),
    )
    .expect("record")
}

#[test]
fn semantic_ui_batches_are_canonical_and_bounded() {
    let forward = build_semantic_ui_batches(
        epoch(),
        vec![
            record("nextengine.ui.surface.a", "nextengine.ui.panel.b", "x"),
            record("nextengine.ui.surface.a", "nextengine.ui.panel.a", "y"),
        ],
        1,
    )
    .expect("batches");
    let reverse = build_semantic_ui_batches(
        epoch(),
        vec![
            record("nextengine.ui.surface.a", "nextengine.ui.panel.a", "y"),
            record("nextengine.ui.surface.a", "nextengine.ui.panel.b", "x"),
        ],
        1,
    )
    .expect("batches");
    assert_eq!(forward, reverse);
    validate_semantic_ui_batches(epoch(), &forward).expect("valid batches");
    assert_eq!(
        build_semantic_ui_batches(
            epoch(),
            vec![
                record("nextengine.ui.surface.a", "nextengine.ui.panel.a", "y"),
                record("nextengine.ui.surface.a", "nextengine.ui.panel.a", "y"),
            ],
            8,
        ),
        Err(PresentationContractError::DuplicateUiElementKey)
    );
    assert_eq!(
        build_semantic_ui_batches(epoch(), Vec::new(), 0),
        Err(PresentationContractError::InvalidBatchProfile)
    );
}

#[test]
fn batch_validation_rejects_tamper_and_epoch_mismatch() {
    let batches = build_semantic_ui_batches(
        epoch(),
        vec![record(
            "nextengine.ui.surface.a",
            "nextengine.ui.panel.a",
            "y",
        )],
        8,
    )
    .expect("batches");
    let foreign_epoch = domain_hash("test.ui.epoch", b"foreign");
    assert_eq!(
        validate_semantic_ui_batches(foreign_epoch, &batches),
        Err(PresentationContractError::SnapshotEpochMismatch)
    );
    let mut tampered_record = record("nextengine.ui.surface.a", "nextengine.ui.panel.a", "y");
    tampered_record.surface_id = id("nextengine.ui.surface.b");
    assert_eq!(
        build_semantic_ui_batches(epoch(), vec![tampered_record], 8),
        Err(PresentationContractError::HashMismatch)
    );
}
