use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use next_assets::ContentStore;
use next_contracts::canonical::sha256;
use next_contracts::ids::{AssetId, ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    KEYBOARD_D_CONTROL_PATH_ID, KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_E_CONTROL_PATH_ID,
    KEYBOARD_ESCAPE_CONTROL_PATH_ID, KEYBOARD_Q_CONTROL_PATH_ID, KEYBOARD_R_CONTROL_PATH_ID,
    KEYBOARD_W_CONTROL_PATH_ID,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_contracts::presentation::{PresentationRoleV1, PresentationSnapshotV2};
use next_presentation::{TextCatalogResolverV1, rasterize_semantic_ui};
use next_render::{RenderTargetV1, build_b0_frame_plan};
use serde::Serialize;
use sha2::{Digest, Sha256};

const CAPTURE_WIDTH: u32 = 960;
const CAPTURE_HEIGHT: u32 = 540;

pub(super) struct VisualSmokeRequest {
    output: PathBuf,
}

#[derive(Serialize)]
struct VisualSmokeManifestV1 {
    schema_version: u32,
    project_lock: String,
    capture_extent: [u32; 2],
    shader_hashes: BTreeMap<String, String>,
    frames: Vec<VisualSmokeFrameV1>,
}

#[derive(Serialize)]
struct VisualSmokeFrameV1 {
    name: String,
    bmp: String,
    snapshot_root: String,
    frame_plan_root: String,
    camera_hash: Option<String>,
    simulation_tick: u64,
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
    root: &Path,
) -> Result<VisualSmokeRequest, String> {
    let mut output = root.join("target").join("visual-smoke");
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--output" => {
                output = PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| "visual-smoke --output requires a path".to_owned())?,
                );
                if output.is_relative() {
                    output = root.join(output);
                }
            }
            _ => return Err(format!("unknown visual-smoke argument: {argument}")),
        }
    }
    Ok(VisualSmokeRequest { output })
}

pub(super) fn run(root: &Path, request: &VisualSmokeRequest) -> Result<(), String> {
    fs::create_dir_all(&request.output).map_err(|error| error.to_string())?;
    let scratch = root
        .join("target")
        .join(format!("visual-smoke-state-{}", std::process::id()));
    if scratch.exists() {
        fs::remove_dir_all(&scratch).map_err(|error| error.to_string())?;
    }
    let store = ContentStore::new(&scratch);
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    store
        .publish(&cooked.publication().map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    let activated =
        next_project::activate_project_package(&store).map_err(|error| error.to_string())?;
    let snapshots = fixed_snapshots(&activated)?;
    let resolver = TextCatalogResolverV1::new(activated.project.text_catalogs.clone(), "en")
        .map_err(|error| error.to_string())?;
    let mut frames = Vec::with_capacity(snapshots.len());
    for (index, (name, snapshot)) in snapshots.into_iter().enumerate() {
        let target = RenderTargetV1 {
            extent: [CAPTURE_WIDTH, CAPTURE_HEIGHT],
            target_revision: u64::try_from(index + 1).map_err(|_| "capture index overflow")?,
        };
        let plan =
            build_b0_frame_plan(&snapshot, &activated.project.render_content_catalog, target)
                .map_err(|error| error.to_string())?;
        let mut canvas = Canvas::new(CAPTURE_WIDTH, CAPTURE_HEIGHT);
        canvas.draw_background();
        canvas.draw_scene(&snapshot, &activated.project.render_content_catalog);
        let ui_records = snapshot.semantic_ui_records().cloned().collect::<Vec<_>>();
        if let Some(overlay) = rasterize_semantic_ui(
            &ui_records,
            &resolver,
            CAPTURE_WIDTH,
            CAPTURE_HEIGHT,
            next_presentation::UI_OVERLAY_TEXT_SCALE,
        ) {
            canvas.composite(&overlay.rgba);
        }
        let bmp_name = format!("{index:02}-{name}.bmp");
        write_bmp(&request.output.join(&bmp_name), &canvas)?;
        frames.push(VisualSmokeFrameV1 {
            name: name.to_owned(),
            bmp: bmp_name,
            snapshot_root: snapshot.canonical_hash.to_hex(),
            frame_plan_root: plan.frame_plan_hash.to_hex(),
            camera_hash: snapshot
                .camera_records()
                .next()
                .map(|camera| camera.canonical_hash.to_hex()),
            simulation_tick: snapshot.simulation_tick,
        });
    }
    let manifest = VisualSmokeManifestV1 {
        schema_version: 1,
        project_lock: activated.project.project_lock.project_lock_sha256.to_hex(),
        capture_extent: [CAPTURE_WIDTH, CAPTURE_HEIGHT],
        shader_hashes: shader_hashes(root)?,
        frames,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    fs::write(request.output.join("manifest.json"), manifest_bytes)
        .map_err(|error| error.to_string())?;
    fs::remove_dir_all(&scratch).map_err(|error| error.to_string())?;
    println!(
        "{}",
        serde_json::json!({
            "schema_version": 1,
            "command": "visual-smoke",
            "status": "PASS",
            "output": request.output,
            "frame_count": manifest.frames.len(),
            "project_lock": manifest.project_lock,
        })
    );
    Ok(())
}

fn fixed_snapshots(
    activated: &next_project::ActivatedProjectPackage,
) -> Result<Vec<(&'static str, PresentationSnapshotV2)>, String> {
    let spawn_driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .map_err(|error| error.to_string())?;
    let spawn = spawn_driver
        .state()
        .map_err(|error| error.to_string())?
        .presentation_snapshot
        .clone();

    let mut offer_driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .map_err(|error| error.to_string())?;
    let quest_offer = offer_driver
        .advance(&[control_event(
            KEYBOARD_E_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            1,
        )])
        .map_err(|error| error.to_string())?
        .clone();

    let mut pause_driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .map_err(|error| error.to_string())?;
    let pause = pause_driver
        .advance(&[control_event(
            KEYBOARD_ESCAPE_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            1,
        )])
        .map_err(|error| error.to_string())?
        .clone();

    let mut driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .map_err(|error| error.to_string())?;
    let script = visual_script();
    let mut relay_inactive = None;
    let mut relay_active = None;
    let mut combat = None;
    for (index, events) in script.into_iter().enumerate() {
        let snapshot = driver.advance(&events).map_err(|error| error.to_string())?;
        match index {
            7 => relay_inactive = Some(snapshot.clone()),
            9 => relay_active = Some(snapshot.clone()),
            12 => combat = Some(snapshot.clone()),
            _ => {}
        }
    }
    Ok(vec![
        ("spawn", spawn),
        ("quest-offer", quest_offer),
        ("combat", combat.ok_or("combat snapshot missing")?),
        (
            "relay-inactive",
            relay_inactive.ok_or("relay inactive snapshot missing")?,
        ),
        (
            "relay-active",
            relay_active.ok_or("relay active snapshot missing")?,
        ),
        ("pause-menu", pause),
    ])
}

fn visual_script() -> Vec<Vec<PlatformEventV1>> {
    use NormalizedControlPhaseV1::{Completed, Started};
    let mut sequence = 10_u64;
    let mut frame = |events: Vec<(&str, NormalizedControlPhaseV1)>| {
        events
            .into_iter()
            .map(|(path, phase)| {
                sequence += 1;
                control_event(path, phase, sequence)
            })
            .collect::<Vec<_>>()
    };
    vec![
        frame(vec![(KEYBOARD_W_CONTROL_PATH_ID, Started)]),
        frame(vec![]),
        frame(vec![]),
        frame(vec![]),
        frame(vec![(KEYBOARD_Q_CONTROL_PATH_ID, Started)]),
        frame(vec![(KEYBOARD_Q_CONTROL_PATH_ID, Completed)]),
        frame(vec![(KEYBOARD_R_CONTROL_PATH_ID, Started)]),
        frame(vec![(KEYBOARD_R_CONTROL_PATH_ID, Completed)]),
        frame(vec![(KEYBOARD_E_CONTROL_PATH_ID, Started)]),
        frame(vec![(KEYBOARD_E_CONTROL_PATH_ID, Completed)]),
        frame(vec![
            (KEYBOARD_W_CONTROL_PATH_ID, Completed),
            (KEYBOARD_D_CONTROL_PATH_ID, Started),
        ]),
        frame(vec![]),
        frame(vec![]),
    ]
}

fn control_event(
    path: &str,
    phase: NormalizedControlPhaseV1,
    source_sequence: u64,
) -> PlatformEventV1 {
    let value = if phase == NormalizedControlPhaseV1::Started {
        i16::MAX
    } else {
        0
    };
    let control = NormalizedControlEventV1::new(
        SchemaId::new(KEYBOARD_DEVICE_CLASS_ID).expect("keyboard class"),
        PersistentId::from_bytes([0x7a; 16]),
        SchemaId::new(path).expect("control path"),
        phase,
        vec![value],
        Vec::new(),
        source_sequence,
        source_sequence,
    )
    .expect("visual smoke control");
    PlatformEventV1::new(
        PersistentId::from_bytes([0x7b; 16]),
        SchemaId::new("nextengine.platform.source.visual-smoke").expect("source"),
        source_sequence,
        source_sequence,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(sha256(b"nextengine.visual-smoke.capabilities.v1")),
    )
    .expect("visual smoke platform event")
}

fn shader_hashes(root: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut values = BTreeMap::new();
    for name in [
        "b0_textured.vert.spv",
        "b0_textured.frag.spv",
        "b0_textured_no_shadow.frag.spv",
        "shadow_depth.vert.spv",
        "sky_gradient.vert.spv",
        "sky_gradient.frag.spv",
        "ui_overlay.vert.spv",
        "ui_overlay.frag.spv",
    ] {
        let bytes = fs::read(root.join("crates/desktop-sdl-ash/shaders").join(name))
            .map_err(|error| error.to_string())?;
        values.insert(name.to_owned(), format!("{:x}", Sha256::digest(bytes)));
    }
    Ok(values)
}

struct Canvas {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            rgba: vec![0; width as usize * height as usize * 4],
        }
    }

    fn draw_background(&mut self) {
        let horizon = self.height * 3 / 5;
        for y in 0..self.height {
            let t = if y < horizon {
                y * 255 / horizon.max(1)
            } else {
                (y - horizon) * 255 / (self.height - horizon).max(1)
            } as u8;
            let color = if y < horizon {
                [
                    28_u8.saturating_add(t / 3),
                    58_u8.saturating_add(t / 2),
                    96_u8.saturating_add(t / 2),
                    255,
                ]
            } else {
                [
                    74_u8.saturating_sub(t / 8),
                    92_u8.saturating_sub(t / 10),
                    70_u8.saturating_sub(t / 12),
                    255,
                ]
            };
            self.rect(0, y as i32, self.width as i32, 1, color);
        }
    }

    fn draw_scene(
        &mut self,
        snapshot: &PresentationSnapshotV2,
        catalog: &next_contracts::render_content::RenderContentCatalogV1,
    ) {
        for record in snapshot.scene_records().filter(|record| record.visible) {
            let color = catalog
                .materials()
                .iter()
                .find(|material| material.asset_revision().ok() == Some(record.material_revision))
                .map(|material| {
                    material
                        .base_color_rgba_unorm16()
                        .map(|value| (value >> 8) as u8)
                })
                .unwrap_or([220, 70, 220, 255]);
            let [x, y, z] = record.current_transform.translation_micrometres;
            let screen_x = self.width as i32 / 2 + (x / 20_000) as i32;
            let screen_y =
                (self.height * 7 / 10) as i32 - (z / 35_000) as i32 - (y / 30_000) as i32;
            if record.mesh_revision.asset_id == AssetId::from_bytes([0xc7; 16]) {
                self.ring(screen_x, screen_y + 28, 28, color);
                continue;
            }
            if record.mesh_revision.asset_id == AssetId::from_bytes([0xc8; 16]) {
                self.triangle(screen_x, screen_y - 48, 14, 22, color);
                continue;
            }
            let bounds = record.local_bounds;
            let min = bounds.min();
            let max = bounds.max();
            let size_x = ((max[0] - min[0]).unsigned_abs() / 45_000).clamp(8, 150) as i32;
            let size_y = ((max[1] - min[1]).unsigned_abs() / 35_000).clamp(8, 110) as i32;
            match record.object_key.presentation_role {
                PresentationRoleV1::PlayerAvatar | PresentationRoleV1::Character => {
                    self.rect(
                        screen_x - size_x / 4,
                        screen_y - size_y,
                        size_x / 2,
                        size_y,
                        color,
                    );
                    self.ring(screen_x, screen_y - size_y - 7, 7, color);
                }
                PresentationRoleV1::Item => self.triangle(screen_x, screen_y - 12, 10, 18, color),
                _ => self.rect(
                    screen_x - size_x / 2,
                    screen_y - size_y,
                    size_x,
                    size_y,
                    color,
                ),
            }
        }
    }

    fn composite(&mut self, overlay: &[u8]) {
        for (destination, source) in self.rgba.chunks_exact_mut(4).zip(overlay.chunks_exact(4)) {
            let alpha = u32::from(source[3]);
            for channel in 0..3 {
                destination[channel] = ((u32::from(source[channel]) * alpha
                    + u32::from(destination[channel]) * (255 - alpha)
                    + 127)
                    / 255) as u8;
            }
        }
    }

    fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: [u8; 4]) {
        for py in y.max(0)..(y + height).min(self.height as i32) {
            for px in x.max(0)..(x + width).min(self.width as i32) {
                let index = (py as usize * self.width as usize + px as usize) * 4;
                self.rgba[index..index + 4].copy_from_slice(&color);
            }
        }
    }

    fn ring(&mut self, x: i32, y: i32, radius: i32, color: [u8; 4]) {
        let inner = (radius - 3).max(0);
        for py in -radius..=radius {
            for px in -radius..=radius {
                let squared = px * px + py * py;
                if squared <= radius * radius && squared >= inner * inner {
                    self.rect(x + px, y + py, 1, 1, color);
                }
            }
        }
    }

    fn triangle(&mut self, x: i32, y: i32, half_width: i32, height: i32, color: [u8; 4]) {
        for row in 0..height {
            let width = half_width * (row + 1) / height.max(1);
            self.rect(x - width, y + row, width * 2 + 1, 1, color);
        }
    }
}

fn write_bmp(path: &Path, canvas: &Canvas) -> Result<(), String> {
    let pixel_bytes = canvas
        .width
        .checked_mul(canvas.height)
        .and_then(|value| value.checked_mul(4))
        .ok_or("BMP size overflow")?;
    let file_size = 54_u32.checked_add(pixel_bytes).ok_or("BMP size overflow")?;
    let mut bytes = Vec::with_capacity(file_size as usize);
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&file_size.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54_u32.to_le_bytes());
    bytes.extend_from_slice(&40_u32.to_le_bytes());
    bytes.extend_from_slice(&(canvas.width as i32).to_le_bytes());
    bytes.extend_from_slice(&(canvas.height as i32).to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&32_u16.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&pixel_bytes.to_le_bytes());
    bytes.extend_from_slice(&[0; 16]);
    for row in (0..canvas.height).rev() {
        for column in 0..canvas.width {
            let index = (row as usize * canvas.width as usize + column as usize) * 4;
            let pixel = &canvas.rgba[index..index + 4];
            bytes.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }
    fs::write(path, bytes).map_err(|error| error.to_string())
}
