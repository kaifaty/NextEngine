use super::*;

#[test]
fn pause_menu_scenario_is_golden() {
    let resolver = resolver();
    let mut records = hud_records(37);
    records.push(record(
        "nextengine.ui.surface.pause-menu",
        "nextengine.ui.panel.pause-menu.root",
        "nextengine.ui.element.pause-menu.title",
        UiElementRoleV1::Label,
        UiStyleRoleV1::Default,
        true,
        true,
        false,
        Some(text_ref("title", Vec::new())),
        UiElementValueV1::None,
    ));
    records.push(record(
        "nextengine.ui.surface.pause-menu",
        "nextengine.ui.panel.pause-menu.root",
        "nextengine.ui.element.pause-menu.resume",
        UiElementRoleV1::Button,
        UiStyleRoleV1::Accent,
        true,
        true,
        true,
        Some(text_ref("resume", Vec::new())),
        UiElementValueV1::None,
    ));
    let image = rasterize_semantic_ui(
        &records,
        &resolver,
        EXTENT.0,
        EXTENT.1,
        UI_OVERLAY_TEXT_SCALE,
    )
    .expect("image");
    assert_eq!(
        image.content_hash().to_hex(),
        "e3c0b312312a599204089c7f6126ca0e0471f470a354fcc3d74169302ab1a13c"
    );
}

#[test]
fn acceptance_surfaces_are_golden_at_720p_and_1080p() {
    let resolver = resolver();
    let pseudo = TextCatalogResolverV1::new(catalogs(), "qps-ploc").expect("pseudo resolver");
    let inventory = vec![record(
        INVENTORY_SURFACE_ID,
        "nextengine.ui.panel.inventory.root",
        "nextengine.ui.element.inventory.item",
        UiElementRoleV1::ListItem,
        UiStyleRoleV1::Default,
        true,
        true,
        false,
        Some(text_ref("inventory", Vec::new())),
        UiElementValueV1::None,
    )];
    let dialogue = vec![
        record(
            DIALOGUE_SURFACE_ID,
            "nextengine.ui.panel.dialogue.root",
            DIALOGUE_NODE_ELEMENT_ID,
            UiElementRoleV1::Label,
            UiStyleRoleV1::Default,
            true,
            true,
            false,
            Some(text_ref("dialogue", Vec::new())),
            UiElementValueV1::None,
        ),
        record(
            DIALOGUE_SURFACE_ID,
            "nextengine.ui.panel.dialogue.root",
            DIALOGUE_ACCEPT_ELEMENT_ID,
            UiElementRoleV1::Button,
            UiStyleRoleV1::Accent,
            true,
            true,
            true,
            Some(text_ref("accept", Vec::new())),
            UiElementValueV1::None,
        ),
    ];
    let pause = [
        (PAUSE_RESUME_ELEMENT_ID, "resume", true),
        (PAUSE_SAVE_ELEMENT_ID, "save", false),
        (PAUSE_LOAD_ELEMENT_ID, "load", false),
    ]
    .into_iter()
    .map(|(element, text, selected)| {
        record(
            PAUSE_MENU_SURFACE_ID,
            "nextengine.ui.panel.pause-menu.root",
            element,
            UiElementRoleV1::Button,
            UiStyleRoleV1::Accent,
            true,
            true,
            selected,
            Some(text_ref(text, Vec::new())),
            UiElementValueV1::None,
        )
    })
    .collect::<Vec<_>>();
    let scenarios = [
        ("hud", hud_records(37), &resolver),
        ("pseudo", hud_records(37), &pseudo),
        ("inventory", inventory, &resolver),
        ("dialogue", dialogue, &resolver),
        ("pause-save-load", pause, &resolver),
    ];
    let mut hashes = Vec::new();
    for (name, records, scenario_resolver) in scenarios {
        for (width, height) in [(1280, 720), (1920, 1080)] {
            let image = rasterize_semantic_ui(
                &records,
                scenario_resolver,
                width,
                height,
                UI_OVERLAY_TEXT_SCALE,
            )
            .expect("golden image");
            let hash = image.content_hash().to_hex();
            let _ = name;
            hashes.push(hash);
        }
    }
    assert_eq!(
        hashes,
        [
            "02cad0d043aa3564dd157af2027805dde0c9db1afaadfdac4522943f7711699a",
            "bc0fcb3f6bfa9248458300ec5d1fe793bb9cb385566e3b482ae4b37633eb4a10",
            "793054123382a708d803adbef44245d4f86da0892576e77d24aeaa13c44d6159",
            "1d768fcfbf1f060b3b29bc6e94278694df3689b5c49718f16b33855a829c960b",
            "0462bdc63d6f9087bc304773cf3d5448aa89489b97103cfbfa1dc6404de25e83",
            "ee3ea09cee76830c77dc7fe27f07504792e57a1f8a6c6423ed8d71b758e02083",
            "681c0adcad690a7b28167880d8aaec4321d0b1dd759693f128e127d3c408650b",
            "7cf36877d76dfd8221d0277edb0cc98178dc43b2b93bc0345299c2ccde6fc1fe",
            "7b088518a512bf4a6edc58bd2df8027c57c7c17cb2a03d0d69f18500a9f4c83e",
            "dd01483035f40ec568e66ec3f3440e45f64df27fe35c063276ecad540b055b8d",
        ]
    );
}
