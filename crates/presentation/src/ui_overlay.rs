//! Deterministic CPU rasterization of semantic UI records into an RGBA
//! overlay image for the minimal engine-owned widget adapter (ADR-044).
//!
//! The adapter owns layout: contracts carry only widget-free semantic
//! elements, so this projection maps canonical record order onto a simple
//! vertical stack of surfaces and panels anchored at the top-left margin.
//! Rasterization is a pure function of the record set, the resolved text and
//! the target extent — the same inputs always produce the same pixels, which
//! keeps the overlay outside gameplay authority and replay-safe.

use next_contracts::ids::ContentHash;
use next_contracts::presentation::{
    SemanticUiPresentationRecordV1, UiElementRoleV1, UiElementValueV1, UiStyleRoleV1,
};
use next_contracts::project::domain_hash;

use crate::text::TextCatalogResolverV1;
use crate::ui_font::{UI_OVERLAY_GLYPH_HEIGHT, UI_OVERLAY_GLYPH_WIDTH, ui_overlay_glyph_rows};

pub const UI_OVERLAY_TEXT_SCALE: u32 = 2;

const CELL: u32 = UI_OVERLAY_GLYPH_WIDTH * UI_OVERLAY_TEXT_SCALE;
const MARGIN: u32 = 8;
const PANEL_PADDING: u32 = 4;
const PANEL_SPACING: u32 = CELL;
const SURFACE_SPACING: u32 = CELL;
const METER_BAR_CELLS: u32 = 16;
const METER_BAR_HEIGHT: u32 = 8;
const LIST_ITEM_INDENT_CELLS: u32 = 1;

const PANEL_BACKGROUND: [u8; 4] = [12, 14, 20, 160];
const SELECTED_BACKGROUND: [u8; 4] = [54, 84, 130, 190];
const METER_TRACK: [u8; 4] = [48, 52, 60, 220];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiOverlayImageV1 {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl UiOverlayImageV1 {
    /// Stable content identity over dimensions and pixels. Adapters use it as
    /// an upload-cache key; tests use it as deterministic golden evidence.
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        let mut preimage = Vec::with_capacity(8_usize.saturating_add(self.rgba.len()));
        preimage.extend_from_slice(&self.width.to_le_bytes());
        preimage.extend_from_slice(&self.height.to_le_bytes());
        preimage.extend_from_slice(&self.rgba);
        domain_hash("nextengine.ui-overlay-image.v1", &preimage)
    }
}

/// Rasterizes one publication's semantic UI records into a transparent RGBA
/// overlay sized to the render target.
///
/// Records are re-sorted into canonical (surface, panel, element) order so
/// the projection is self-sufficient. Invisible elements are skipped,
/// disabled elements render dimmed, and a selected row carries a highlight.
/// Returns `None` when there is nothing to composite.
#[must_use]
pub fn rasterize_semantic_ui(
    records: &[SemanticUiPresentationRecordV1],
    resolver: &TextCatalogResolverV1,
    width: u32,
    height: u32,
) -> Option<UiOverlayImageV1> {
    if records.is_empty() || width == 0 || height == 0 {
        return None;
    }
    let mut ordered: Vec<&SemanticUiPresentationRecordV1> = records.iter().collect();
    ordered.sort_by(|left, right| {
        (
            &left.surface_id,
            &left.semantic_path_id,
            &left.element.element_id,
        )
            .cmp(&(
                &right.surface_id,
                &right.semantic_path_id,
                &right.element.element_id,
            ))
    });

    let pixel_count = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(4)?;
    let mut image = UiOverlayImageV1 {
        width,
        height,
        rgba: vec![0_u8; pixel_count],
    };
    let mut cursor_y = MARGIN;
    let mut previous_surface: Option<&next_contracts::ids::SchemaId> = None;
    let mut panel_start = 0_usize;
    let mut any_drawn = false;
    while panel_start < ordered.len() {
        let surface_id = &ordered[panel_start].surface_id;
        let semantic_path_id = &ordered[panel_start].semantic_path_id;
        let mut panel_end = panel_start + 1;
        while panel_end < ordered.len()
            && ordered[panel_end].surface_id == *surface_id
            && ordered[panel_end].semantic_path_id == *semantic_path_id
        {
            panel_end += 1;
        }
        if previous_surface.is_some_and(|previous| previous != surface_id) {
            cursor_y = cursor_y.saturating_add(SURFACE_SPACING);
        }
        previous_surface = Some(surface_id);
        any_drawn |= rasterize_panel(
            &mut image,
            &ordered[panel_start..panel_end],
            resolver,
            MARGIN,
            &mut cursor_y,
        );
        panel_start = panel_end;
    }
    any_drawn.then_some(image)
}

fn rasterize_panel(
    image: &mut UiOverlayImageV1,
    panel: &[&SemanticUiPresentationRecordV1],
    resolver: &TextCatalogResolverV1,
    origin_x: u32,
    cursor_y: &mut u32,
) -> bool {
    let rows: Vec<OverlayRow> = panel
        .iter()
        .flat_map(|record| overlay_rows(record, resolver))
        .collect();
    if rows.is_empty() {
        return false;
    }
    let content_width = rows.iter().map(OverlayRow::width).max().unwrap_or_default();
    let content_height: u32 = rows.iter().map(OverlayRow::height).sum();
    fill_rect(
        image,
        origin_x.saturating_sub(PANEL_PADDING),
        cursor_y.saturating_sub(PANEL_PADDING),
        content_width.saturating_add(PANEL_PADDING.saturating_mul(2)),
        content_height.saturating_add(PANEL_PADDING.saturating_mul(2)),
        PANEL_BACKGROUND,
    );
    let mut row_y = *cursor_y;
    for row in &rows {
        row.rasterize(image, origin_x, row_y, content_width);
        row_y = row_y.saturating_add(row.height());
    }
    *cursor_y = row_y.saturating_add(PANEL_SPACING);
    true
}

enum OverlayRow {
    Text {
        text: String,
        color: [u8; 4],
        selected: bool,
        indent_cells: u32,
    },
    MeterBar {
        current: i64,
        maximum: i64,
        color: [u8; 4],
    },
}

impl OverlayRow {
    fn width(&self) -> u32 {
        match self {
            Self::Text {
                text, indent_cells, ..
            } => indent_cells
                .saturating_add(u32::try_from(text.chars().count()).unwrap_or(u32::MAX))
                .saturating_mul(CELL),
            Self::MeterBar { .. } => METER_BAR_CELLS.saturating_mul(CELL),
        }
    }

    const fn height(&self) -> u32 {
        match self {
            Self::Text { .. } => CELL,
            Self::MeterBar { .. } => METER_BAR_HEIGHT,
        }
    }

    fn rasterize(&self, image: &mut UiOverlayImageV1, origin_x: u32, row_y: u32, width: u32) {
        match self {
            Self::Text {
                text,
                color,
                selected,
                indent_cells,
            } => {
                if *selected {
                    fill_rect(image, origin_x, row_y, width, CELL, SELECTED_BACKGROUND);
                }
                draw_text(
                    image,
                    origin_x.saturating_add(indent_cells.saturating_mul(CELL)),
                    row_y,
                    text,
                    *color,
                );
            }
            Self::MeterBar {
                current,
                maximum,
                color,
            } => {
                let bar_width = METER_BAR_CELLS.saturating_mul(CELL);
                fill_rect(
                    image,
                    origin_x,
                    row_y,
                    bar_width,
                    METER_BAR_HEIGHT,
                    METER_TRACK,
                );
                let fill_width = if *maximum > 0 && *current > 0 {
                    u32::try_from(
                        u64::from(bar_width)
                            .saturating_mul(*current as u64)
                            .checked_div(*maximum as u64)
                            .unwrap_or_default(),
                    )
                    .unwrap_or(bar_width)
                    .min(bar_width)
                } else {
                    0
                };
                fill_rect(image, origin_x, row_y, fill_width, METER_BAR_HEIGHT, *color);
            }
        }
    }
}

/// Projects one visible element into zero, one or two overlay rows: a meter
/// contributes its text row (when present) plus its bar row, every other role
/// contributes at most its text row.
fn overlay_rows(
    record: &SemanticUiPresentationRecordV1,
    resolver: &TextCatalogResolverV1,
) -> Vec<OverlayRow> {
    let element = &record.element;
    if !element.visible {
        return Vec::new();
    }
    let mut color = style_color(element.style_role);
    if !element.enabled {
        color = dim_color(color);
    }
    let text_row = element
        .text_or_none
        .as_ref()
        .map(|text_ref| OverlayRow::Text {
            text: resolver.resolve(text_ref).text,
            color,
            selected: element.selected,
            indent_cells: if element.role == UiElementRoleV1::ListItem {
                LIST_ITEM_INDENT_CELLS
            } else {
                0
            },
        });
    if element.role == UiElementRoleV1::Meter {
        let (current, maximum) = match element.value {
            UiElementValueV1::Scalar { current, maximum } => (current, maximum),
            UiElementValueV1::None => (0, 0),
        };
        let mut rows = Vec::with_capacity(2);
        if let Some(row) = text_row {
            rows.push(row);
        }
        rows.push(OverlayRow::MeterBar {
            current,
            maximum,
            color,
        });
        rows
    } else {
        text_row.into_iter().collect()
    }
}

fn style_color(style_role: UiStyleRoleV1) -> [u8; 4] {
    match style_role {
        UiStyleRoleV1::Default => [230, 230, 230, 255],
        UiStyleRoleV1::Muted => [150, 152, 158, 255],
        UiStyleRoleV1::Accent => [96, 200, 220, 255],
        UiStyleRoleV1::Warning => [230, 200, 96, 255],
        UiStyleRoleV1::Danger => [220, 90, 90, 255],
    }
}

fn dim_color(color: [u8; 4]) -> [u8; 4] {
    [color[0] / 2, color[1] / 2, color[2] / 2, color[3]]
}

fn draw_text(
    image: &mut UiOverlayImageV1,
    origin_x: u32,
    origin_y: u32,
    text: &str,
    color: [u8; 4],
) {
    for (index, glyph) in text.chars().enumerate() {
        let glyph_x = origin_x.saturating_add(
            u32::try_from(index)
                .unwrap_or(u32::MAX)
                .saturating_mul(CELL),
        );
        draw_glyph(
            image,
            glyph_x,
            origin_y,
            ui_overlay_glyph_rows(glyph),
            color,
        );
    }
}

fn draw_glyph(
    image: &mut UiOverlayImageV1,
    origin_x: u32,
    origin_y: u32,
    rows: [u8; 8],
    color: [u8; 4],
) {
    for (row_index, row) in rows.iter().enumerate() {
        for column in 0..UI_OVERLAY_GLYPH_WIDTH {
            if row & (1_u8 << column) != 0 {
                fill_rect(
                    image,
                    origin_x.saturating_add(column.saturating_mul(UI_OVERLAY_TEXT_SCALE)),
                    origin_y.saturating_add(
                        u32::try_from(row_index)
                            .unwrap_or(UI_OVERLAY_GLYPH_HEIGHT)
                            .saturating_mul(UI_OVERLAY_TEXT_SCALE),
                    ),
                    UI_OVERLAY_TEXT_SCALE,
                    UI_OVERLAY_TEXT_SCALE,
                    color,
                );
            }
        }
    }
}

fn fill_rect(
    image: &mut UiOverlayImageV1,
    origin_x: u32,
    origin_y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
) {
    if width == 0 || height == 0 || origin_x >= image.width || origin_y >= image.height {
        return;
    }
    let end_x = origin_x.saturating_add(width).min(image.width);
    let end_y = origin_y.saturating_add(height).min(image.height);
    let row_stride = usize::try_from(image.width).unwrap_or(usize::MAX / 4);
    for y in origin_y..end_y {
        for x in origin_x..end_x {
            let pixel = (usize::try_from(y).unwrap_or(usize::MAX / 4))
                .saturating_mul(row_stride)
                .saturating_add(usize::try_from(x).unwrap_or(usize::MAX / 4))
                .saturating_mul(4);
            if let Some(destination) = image.rgba.get_mut(pixel..pixel.saturating_add(4)) {
                blend_source_over(destination, color);
            }
        }
    }
}

fn blend_source_over(destination: &mut [u8], source: [u8; 4]) {
    let source_alpha = u32::from(source[3]);
    let inverse_alpha = 255_u32.saturating_sub(source_alpha);
    for channel in 0..3 {
        destination[channel] = ((u32::from(source[channel]) * source_alpha
            + u32::from(destination[channel]) * inverse_alpha
            + 127)
            / 255) as u8;
    }
    destination[3] = (source_alpha + (u32::from(destination[3]) * inverse_alpha + 127) / 255) as u8;
}

#[cfg(test)]
mod tests {
    use next_contracts::ids::{AssetId, SchemaId};
    use next_contracts::localization::{TextCatalogEntryV1, TextCatalogV1, TextLocaleTagV1};
    use next_contracts::presentation::{
        UiAccessibilityRoleV1, UiElementRoleV1, UiElementValueV1, UiSemanticElementV1,
        UiTextArgumentV1, UiTextRefV1,
    };
    use next_contracts::project::domain_hash;

    use super::*;

    const EXTENT: (u32, u32) = (320, 180);

    fn schema_id(value: &str) -> SchemaId {
        SchemaId::new(value).expect("schema ID")
    }

    fn text_ref(name: &str, arguments: Vec<UiTextArgumentV1>) -> UiTextRefV1 {
        UiTextRefV1::new(
            schema_id(&format!("nextengine.test.text.{name}")),
            arguments,
        )
        .expect("text ref")
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the helper keeps every semantic element field explicit"
    )]
    fn record(
        surface: &str,
        panel: &str,
        element: &str,
        role: UiElementRoleV1,
        style_role: UiStyleRoleV1,
        enabled: bool,
        visible: bool,
        selected: bool,
        text: Option<UiTextRefV1>,
        value: UiElementValueV1,
    ) -> SemanticUiPresentationRecordV1 {
        SemanticUiPresentationRecordV1::new(
            domain_hash("nextengine.test.epoch.v1", &[]),
            schema_id(surface),
            schema_id(panel),
            domain_hash("nextengine.test.source.v1", &[0x01]),
            UiSemanticElementV1::new(
                schema_id(element),
                role,
                style_role,
                UiAccessibilityRoleV1::Standard,
                enabled,
                visible,
                selected,
                text,
                value,
                Vec::new(),
            )
            .expect("element"),
        )
        .expect("record")
    }

    fn catalogs() -> Vec<TextCatalogV1> {
        let catalog = |byte: u8, locale: &str, fallback: Option<&str>, entries: &[(&str, &str)]| {
            TextCatalogV1::new(
                AssetId::from_bytes([byte; 16]),
                1,
                TextLocaleTagV1::new(locale).expect("locale"),
                fallback.map(|value| TextLocaleTagV1::new(value).expect("fallback")),
                entries
                    .iter()
                    .map(|(name, template)| {
                        TextCatalogEntryV1::new(
                            schema_id(&format!("nextengine.test.text.{name}")),
                            (*template).to_owned(),
                        )
                        .expect("entry")
                    })
                    .collect(),
            )
            .expect("catalog")
        };
        vec![
            catalog(
                0x91,
                "en",
                None,
                &[
                    ("health", "Health {0}/{1}"),
                    ("quest", "Quest: {0}"),
                    ("state-active", "Active"),
                    ("title", "Paused"),
                    ("resume", "Resume"),
                ],
            ),
            catalog(
                0x92,
                "qps-ploc",
                Some("en"),
                &[("health", "⟦Ħēåłŧħ⟧ {0}/{1}")],
            ),
        ]
    }

    fn resolver() -> TextCatalogResolverV1 {
        TextCatalogResolverV1::new(catalogs(), "en").expect("resolver")
    }

    fn hud_records(health: i64) -> Vec<SemanticUiPresentationRecordV1> {
        vec![
            record(
                "nextengine.ui.surface.hud",
                "nextengine.ui.panel.hud.status",
                "nextengine.ui.element.hud.health",
                UiElementRoleV1::Meter,
                UiStyleRoleV1::Default,
                true,
                true,
                false,
                Some(text_ref(
                    "health",
                    vec![
                        UiTextArgumentV1::SignedInteger(health),
                        UiTextArgumentV1::SignedInteger(100),
                    ],
                )),
                UiElementValueV1::Scalar {
                    current: health,
                    maximum: 100,
                },
            ),
            record(
                "nextengine.ui.surface.hud",
                "nextengine.ui.panel.hud.status",
                "nextengine.ui.element.hud.quest",
                UiElementRoleV1::Label,
                UiStyleRoleV1::Muted,
                true,
                true,
                false,
                Some(text_ref(
                    "quest",
                    vec![UiTextArgumentV1::TextId(schema_id(
                        "nextengine.test.text.state-active",
                    ))],
                )),
                UiElementValueV1::None,
            ),
        ]
    }

    fn luma(image: &UiOverlayImageV1) -> u64 {
        image
            .rgba
            .chunks_exact(4)
            .map(|pixel| u64::from(pixel[0]) + u64::from(pixel[1]) + u64::from(pixel[2]))
            .sum()
    }

    #[test]
    fn empty_records_or_empty_extent_yield_no_overlay() {
        let resolver = resolver();
        assert_eq!(
            rasterize_semantic_ui(&[], &resolver, EXTENT.0, EXTENT.1),
            None
        );
        assert_eq!(
            rasterize_semantic_ui(&hud_records(37), &resolver, 0, EXTENT.1),
            None
        );
        assert_eq!(
            rasterize_semantic_ui(&hud_records(37), &resolver, EXTENT.0, 0),
            None
        );
    }

    #[test]
    fn hud_overlay_is_pixel_deterministic() {
        let resolver = resolver();
        let records = hud_records(37);
        let first = rasterize_semantic_ui(&records, &resolver, EXTENT.0, EXTENT.1).expect("image");
        let second = rasterize_semantic_ui(&records, &resolver, EXTENT.0, EXTENT.1).expect("image");
        assert_eq!(first, second);
        assert_eq!(
            first.content_hash().to_hex(),
            "d3e6d0008fa730a7ce6406397f3c74367af4f71a7e6cfc3abc68d5eb6422f03a"
        );
    }

    #[test]
    fn invisible_elements_are_skipped() {
        let resolver = resolver();
        let mut records = hud_records(37);
        for value in &mut records {
            value.element.visible = false;
        }
        assert_eq!(
            rasterize_semantic_ui(&records, &resolver, EXTENT.0, EXTENT.1),
            None
        );
    }

    #[test]
    fn meter_fill_width_tracks_the_scalar_value() {
        let resolver = resolver();
        let image =
            rasterize_semantic_ui(&hud_records(37), &resolver, EXTENT.0, EXTENT.1).expect("image");
        // The bar band sits directly below the first text row; only the
        // opaque fill reaches alpha 255 over the translucent panel.
        let band_y = MARGIN + CELL;
        let mut opaque = 0_u32;
        for y in band_y..band_y + METER_BAR_HEIGHT {
            for x in 0..image.width {
                let pixel = ((y * image.width + x) * 4) as usize;
                if image.rgba[pixel + 3] == 255 {
                    opaque += 1;
                }
            }
        }
        let expected_fill = METER_BAR_CELLS * CELL * 37 / 100;
        assert_eq!(opaque, expected_fill * METER_BAR_HEIGHT);

        let fuller =
            rasterize_semantic_ui(&hud_records(74), &resolver, EXTENT.0, EXTENT.1).expect("image");
        assert_ne!(image.rgba, fuller.rgba);
    }

    #[test]
    fn disabled_and_selected_states_change_pixels() {
        let resolver = resolver();
        let enabled = rasterize_semantic_ui(&hud_records(37)[..1], &resolver, EXTENT.0, EXTENT.1)
            .expect("image");
        let mut disabled_record = hud_records(37).remove(0);
        disabled_record.element.enabled = false;
        let disabled = rasterize_semantic_ui(&[disabled_record], &resolver, EXTENT.0, EXTENT.1)
            .expect("image");
        assert_ne!(disabled.rgba, enabled.rgba);
        assert!(luma(&disabled) < luma(&enabled));

        let mut selected_record = hud_records(37).remove(0);
        selected_record.element.selected = true;
        let selected = rasterize_semantic_ui(&[selected_record], &resolver, EXTENT.0, EXTENT.1)
            .expect("image");
        assert_ne!(selected.rgba, enabled.rgba);
        assert!(luma(&selected) > luma(&enabled));
    }

    #[test]
    fn missing_text_still_renders_the_readable_placeholder() {
        let resolver = resolver();
        let records = [record(
            "nextengine.ui.surface.hud",
            "nextengine.ui.panel.hud.status",
            "nextengine.ui.element.hud.quest",
            UiElementRoleV1::Label,
            UiStyleRoleV1::Default,
            true,
            true,
            false,
            Some(text_ref("unknown", Vec::new())),
            UiElementValueV1::None,
        )];
        let image = rasterize_semantic_ui(&records, &resolver, EXTENT.0, EXTENT.1).expect("image");
        assert!(image.rgba.iter().any(|value| *value != 0));
    }

    #[test]
    fn pseudo_locale_text_rasterizes_authored_glyphs() {
        let pseudo = TextCatalogResolverV1::new(catalogs(), "qps-ploc").expect("resolver");
        let records = hud_records(37);
        let image = rasterize_semantic_ui(&records, &pseudo, EXTENT.0, EXTENT.1).expect("image");
        let english =
            rasterize_semantic_ui(&records, &resolver(), EXTENT.0, EXTENT.1).expect("image");
        assert_ne!(image.rgba, english.rgba);
    }

    #[test]
    fn tiny_extent_clips_without_panicking() {
        let resolver = resolver();
        let image = rasterize_semantic_ui(&hud_records(37), &resolver, 8, 8).expect("image");
        assert_eq!(image.rgba.len(), 8 * 8 * 4);
    }

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
        let image = rasterize_semantic_ui(&records, &resolver, EXTENT.0, EXTENT.1).expect("image");
        assert_eq!(
            image.content_hash().to_hex(),
            "169bf329638d3619ad29d16d86b6d27a261e794a83220da6e4e00ce294da3d12"
        );
    }
}
