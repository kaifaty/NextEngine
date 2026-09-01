use std::path::{Path, PathBuf};

#[cfg(feature = "desktop-sdl-ash")]
use std::fs::File;
#[cfg(feature = "desktop-sdl-ash")]
use std::io::Read;
#[cfg(feature = "desktop-sdl-ash")]
use std::sync::Arc;

#[cfg(feature = "desktop-sdl-ash")]
use next_contracts::ids::{AssetId, PersistentId};
#[cfg(feature = "desktop-sdl-ash")]
use next_contracts::presentation::{
    CameraInterpolationPolicyV1, CameraPresentationRecordV2, CameraProjectionProfileV1,
    CameraResultSampleV1, CameraRoleV1, CameraViewportV1, PresentationObjectKeyV1,
    PresentationRoleV1, PresentationSnapshotV3, QuantizedPresentationTransformV1,
    ScenePresentationFlagsV1, ScenePresentationRecordV2, ThirdPersonCameraIntentSampleV1,
};
#[cfg(feature = "desktop-sdl-ash")]
use next_contracts::project::domain_hash;
#[cfg(any(feature = "desktop-sdl-ash", test))]
use next_contracts::render_content::AabbI64V1;
#[cfg(feature = "desktop-sdl-ash")]
use next_contracts::render_content::{
    MaterialAlphaModeV1, MaterialColorSpaceV1, MaterialTextureSlotV1, MeshPrimitiveTopologyV1,
    NeutralMaterialTextureBindingV1, NeutralMaterialV1, NeutralMeshPrimitiveV1, NeutralMeshV1,
    NeutralRenderRecordV1, NeutralTexelEncodingV1, NeutralTextureAlphaSemanticsV1,
    NeutralTextureColorSpaceV1, NeutralTextureDimensionV1, NeutralTextureMipLevelV1,
    NeutralTextureV1, UvTransformV1,
};
#[cfg(feature = "desktop-sdl-ash")]
use next_desktop_sdl_ash::{
    DesktopApplicationFinalization, DesktopFramePublicationV1, DynamicSurfaceProfileV1,
    DynamicSurfaceUpdateV1,
};
#[cfg(feature = "desktop-sdl-ash")]
use next_render::{RenderTargetV1, build_b0_frame_plan};
#[cfg(any(feature = "desktop-sdl-ash", test))]
use sha2::{Digest, Sha256};

#[cfg(feature = "desktop-sdl-ash")]
const MAX_OBJ_BYTES: u64 = 32 * 1024 * 1024;
#[cfg(any(feature = "desktop-sdl-ash", test))]
const MAX_VERTICES: usize = 100_000;
#[cfg(any(feature = "desktop-sdl-ash", test))]
const MAX_TRIANGLES: usize = 200_000;
const MAX_KEYFRAMES: usize = 256;
#[cfg(feature = "desktop-sdl-ash")]
const WATER_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xf1; 16]);
#[cfg(feature = "desktop-sdl-ash")]
const WATER_TEXTURE_ASSET_ID: AssetId = AssetId::from_bytes([0xf2; 16]);
#[cfg(feature = "desktop-sdl-ash")]
const WATER_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xf3; 16]);

pub(super) struct WaterPreviewRequest {
    /// One OBJ renders the static catalog path. Two or more OBJ keyframes
    /// declare one presentation-only dynamic surface and cycle through it.
    meshes: Vec<PathBuf>,
    frames: u64,
    extent: [u32; 2],
    /// Event-loop pumps each keyframe stays current before the next one is
    /// published.
    hold: u64,
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
    root: &Path,
) -> Result<WaterPreviewRequest, String> {
    let mut meshes = Vec::new();
    let mut frames = 600_u64;
    let mut extent = [1280, 720];
    let mut hold = 8_u64;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--mesh" => {
                let value = PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| "water-preview --mesh requires a path".to_owned())?,
                );
                if meshes.len() >= MAX_KEYFRAMES {
                    return Err(format!(
                        "water-preview accepts at most {MAX_KEYFRAMES} --mesh keyframes"
                    ));
                }
                meshes.push(if value.is_relative() {
                    root.join(value)
                } else {
                    value
                });
            }
            "--frames" => {
                frames = arguments
                    .next()
                    .ok_or_else(|| "water-preview --frames requires a count".to_owned())?
                    .parse()
                    .map_err(|_| "water-preview frame count is invalid".to_owned())?;
                if frames == 0 || frames > 65_536 {
                    return Err("water-preview frames must be in 1..=65536".to_owned());
                }
            }
            "--hold" => {
                hold = arguments
                    .next()
                    .ok_or_else(|| "water-preview --hold requires a pump count".to_owned())?
                    .parse()
                    .map_err(|_| "water-preview hold count is invalid".to_owned())?;
                if hold == 0 || hold > 65_536 {
                    return Err("water-preview hold must be in 1..=65536".to_owned());
                }
            }
            "--extent" => {
                extent =
                    parse_extent(&arguments.next().ok_or_else(|| {
                        "water-preview --extent requires WIDTHxHEIGHT".to_owned()
                    })?)?;
            }
            _ => return Err(format!("unknown water-preview argument: {argument}")),
        }
    }
    if meshes.is_empty() {
        return Err("water-preview requires at least one --mesh <surface.obj>".to_owned());
    }
    Ok(WaterPreviewRequest {
        meshes,
        frames,
        extent,
        hold,
    })
}

fn parse_extent(value: &str) -> Result<[u32; 2], String> {
    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| "water-preview extent must be WIDTHxHEIGHT".to_owned())?;
    let extent = [
        width
            .parse()
            .map_err(|_| "water-preview width is invalid".to_owned())?,
        height
            .parse()
            .map_err(|_| "water-preview height is invalid".to_owned())?,
    ];
    if extent[0] < 320 || extent[1] < 240 || extent[0] > 7680 || extent[1] > 4320 {
        return Err("water-preview extent is outside 320x240..=7680x4320".to_owned());
    }
    Ok(extent)
}

/// Deterministic keyframe selection for one event-loop pump. The schedule
/// depends only on the pump index and the hold count, never on wall time.
#[cfg(any(feature = "desktop-sdl-ash", test))]
fn keyframe_for_pump(pump_index: u64, hold: u64, keyframe_count: usize) -> usize {
    let keyframe_count = u64::try_from(keyframe_count.max(1)).unwrap_or(1);
    let slot = pump_index / hold.max(1) % keyframe_count;
    usize::try_from(slot).unwrap_or(0)
}

#[cfg(feature = "desktop-sdl-ash")]
pub(super) fn run(request: &WaterPreviewRequest) -> Result<(), String> {
    let preview = build_preview(request)?;
    let dynamic = preview.dynamic.as_ref();
    let options = next_desktop_sdl_ash::DesktopRunOptions {
        title: "Next Engine — Nonlocal Water Preview".to_owned(),
        initial_extent: request.extent,
        maximum_frames: Some(request.frames),
        maximum_event_loop_iterations: Some(request.frames.saturating_mul(8)),
        frame_profiling_sample_capacity: u32::try_from(request.frames)
            .map_err(|_| "water-preview profiling capacity overflow".to_owned())?,
        audio_output_enabled: false,
        dynamic_surfaces: dynamic
            .map(|dynamic| vec![dynamic.profile])
            .unwrap_or_default(),
        ..next_desktop_sdl_ash::DesktopRunOptions::default()
    };
    let mut pump_index = 0_u64;
    let mut published_keyframe: Option<usize> = None;
    let mut publications = 0_u64;
    let report = if let Some(dynamic) = dynamic {
        next_desktop_sdl_ash::run_interactive_with_shared_frame_publication_and_finalize(
            Arc::new(preview.snapshot.clone()),
            &preview.catalog,
            &options,
            |_events, _elapsed, _audio| {
                let keyframe = keyframe_for_pump(pump_index, request.hold, dynamic.keyframes.len());
                pump_index = pump_index.saturating_add(1);
                if published_keyframe == Some(keyframe) {
                    return Ok(DesktopFramePublicationV1::default());
                }
                publications = publications.saturating_add(1);
                published_keyframe = Some(keyframe);
                let update = dynamic.keyframes[keyframe]
                    .update
                    .with_sequence(publications)?;
                Ok(DesktopFramePublicationV1 {
                    snapshot: None,
                    dynamic_surface_updates: vec![Arc::new(update)],
                })
            },
            || DesktopApplicationFinalization::Complete,
        )
    } else {
        next_desktop_sdl_ash::run_interactive(&preview.snapshot, &preview.catalog, &options)
    }
    .map_err(|error| error.to_string())?;
    if report.rendered_objects != u64::from(preview.visible_object_count)
        || report.indexed_draws != u64::from(preview.indexed_draw_count)
    {
        return Err("water-preview Vulkan report does not contain the planned draw".to_owned());
    }
    if let Some(dynamic) = dynamic {
        if report.dynamic_surface_publications != publications {
            return Err("water-preview adapter accepted a different publication count".to_owned());
        }
        if report.dynamic_surface_draws != 1 {
            return Err("water-preview last frame did not draw the dynamic surface".to_owned());
        }
        let slot_count = u64::try_from(next_desktop_sdl_ash::DESKTOP_FRAME_SLOT_COUNT)
            .map_err(|_| "water-preview frame slot count overflow".to_owned())?;
        if report.dynamic_surface_uploads < publications
            || report.dynamic_surface_uploads > publications.saturating_mul(slot_count)
        {
            return Err(
                "water-preview ring refresh count is outside the publication bound".to_owned(),
            );
        }
        let expected_hash = published_keyframe
            .map(|keyframe| dynamic.keyframes[keyframe].update.canonical_hash())
            .ok_or_else(|| "water-preview published no keyframe".to_owned())?;
        if report.dynamic_surface_hashes != vec![(dynamic.profile.mesh_revision, expected_hash)] {
            return Err("water-preview adapter holds a different current surface".to_owned());
        }
    }
    let critical_samples = report
        .frame_timings
        .iter()
        .map(|sample| {
            sample
                .cpu_extract_and_submit_microseconds
                .max(sample.gpu_duration_microseconds)
        })
        .collect::<Vec<_>>();
    let cpu_samples = report
        .frame_timings
        .iter()
        .map(|sample| sample.cpu_extract_and_submit_microseconds)
        .collect::<Vec<_>>();
    let gpu_samples = report
        .frame_timings
        .iter()
        .map(|sample| sample.gpu_duration_microseconds)
        .collect::<Vec<_>>();
    let upload_samples = report
        .frame_timings
        .iter()
        .map(|sample| sample.dynamic_surface_upload_microseconds)
        .collect::<Vec<_>>();
    let refresh_samples = report
        .frame_timings
        .iter()
        .filter(|sample| sample.dynamic_surface_uploads != 0)
        .map(|sample| sample.dynamic_surface_upload_microseconds)
        .collect::<Vec<_>>();
    let idle_samples = report
        .frame_timings
        .iter()
        .filter(|sample| sample.dynamic_surface_uploads == 0)
        .map(|sample| sample.dynamic_surface_upload_microseconds)
        .collect::<Vec<_>>();
    let frame_source_samples = report
        .frame_timings
        .iter()
        .map(|sample| sample.event_and_frame_source_update_microseconds)
        .collect::<Vec<_>>();
    let keyframes = preview
        .keyframes
        .iter()
        .map(|keyframe| {
            serde_json::json!({
                "path": keyframe.path,
                "source_sha256": keyframe.source_sha256,
                "vertices": keyframe.vertex_count,
                "triangles": keyframe.triangle_count,
                "update_hash": keyframe.update_hash.map(|hash| hash.to_hex()),
            })
        })
        .collect::<Vec<_>>();
    let timing = serde_json::json!({
        "cpu_extract_submit_p95_us": nearest_rank(&cpu_samples, 95),
        "gpu_duration_p95_us": nearest_rank(&gpu_samples, 95),
        "frame_critical_p95_us": nearest_rank(&critical_samples, 95),
        "frame_critical_p99_us": nearest_rank(&critical_samples, 99),
        "frame_critical_max_us": critical_samples.iter().copied().max(),
        "frame_source_update_p95_us": nearest_rank(&frame_source_samples, 95),
        "frame_source_update_max_us": frame_source_samples.iter().copied().max(),
        "vulkan_timestamp_queries": report.vulkan_timestamp_queries,
        "dropped_timing_samples": report.dropped_frame_timing_samples,
    });
    let dynamic_surface = serde_json::json!({
        "capacity": dynamic.map(|dynamic| serde_json::json!({
            "vertices": dynamic.profile.vertex_capacity,
            "indices": dynamic.profile.index_capacity,
        })),
        "publications": report.dynamic_surface_publications,
        "uploads": report.dynamic_surface_uploads,
        "upload_bytes": report.dynamic_surface_upload_bytes,
        "last_frame_draws": report.dynamic_surface_draws,
        "current_hashes": report
            .dynamic_surface_hashes
            .iter()
            .map(|(revision, hash)| serde_json::json!({
                "asset_id": revision.asset_id.to_hex(),
                "record_sha256": revision.record_sha256.to_hex(),
                "update_hash": hash.to_hex(),
            }))
            .collect::<Vec<_>>(),
        "upload_phase_p95_us": nearest_rank(&upload_samples, 95),
        "upload_phase_max_us": upload_samples.iter().copied().max(),
        "refresh_frames": refresh_samples.len(),
        "refresh_p95_us": nearest_rank(&refresh_samples, 95),
        "refresh_max_us": refresh_samples.iter().copied().max(),
        "idle_frames": idle_samples.len(),
        "idle_p95_us": nearest_rank(&idle_samples, 95),
    });
    println!(
        "{}",
        serde_json::json!({
            "schema_version": 2,
            "command": "water-preview",
            "status": "PASS",
            "authority": "PRESENTATION_ONLY_TOOL",
            "mode": if dynamic.is_some() { "dynamic-keyframes" } else { "static" },
            "keyframes": keyframes,
            "hold_pumps": request.hold,
            "mesh_revision": {
                "asset_id": preview.mesh_revision.asset_id.to_hex(),
                "record_sha256": preview.mesh_revision.record_sha256.to_hex(),
            },
            "declared_bounds_micrometres": {
                "min": preview.bounds.min(),
                "max": preview.bounds.max(),
            },
            "catalog_root": preview.catalog.catalog_sha256().to_hex(),
            "snapshot_root": preview.snapshot.canonical_hash.to_hex(),
            "frame_plan_root": preview.frame_plan_root.to_hex(),
            "rendered_frames": report.rendered_frames,
            "visible_objects_per_frame": preview.visible_object_count,
            "indexed_draws_per_frame": preview.indexed_draw_count,
            "last_frame_indexed_draws": report.indexed_draws,
            "frame_plan_cache_hits": report.frame_plan_cache_hits,
            "frame_plan_cache_misses": report.frame_plan_cache_misses,
            "catalog_rebuilds": 0,
            "device_allocation_bytes": report.device_allocation_bytes,
            "device_allocation_count": report.device_allocation_count,
            "physics_feedback": false,
            "timing": timing,
            "dynamic_surface": dynamic_surface,
        })
    );
    Ok(())
}

#[cfg(not(feature = "desktop-sdl-ash"))]
pub(super) fn run(request: &WaterPreviewRequest) -> Result<(), String> {
    Err(format!(
        "water-preview for {} keyframe(s) starting at {} ({} frames at {}x{}, hold {}) requires --features desktop-sdl-ash",
        request.meshes.len(),
        request.meshes[0].display(),
        request.frames,
        request.extent[0],
        request.extent[1],
        request.hold
    ))
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn nearest_rank(values: &[u64], percentile: usize) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered.get(rank.saturating_sub(1)).copied()
}

#[cfg(feature = "desktop-sdl-ash")]
struct PreviewKeyframe {
    path: PathBuf,
    source_sha256: String,
    vertex_count: usize,
    triangle_count: usize,
    update_hash: Option<next_contracts::ids::ContentHash>,
}

#[cfg(feature = "desktop-sdl-ash")]
struct DynamicKeyframe {
    update: DynamicSurfaceUpdateV1,
}

#[cfg(feature = "desktop-sdl-ash")]
struct DynamicPreview {
    profile: DynamicSurfaceProfileV1,
    keyframes: Vec<DynamicKeyframe>,
}

#[cfg(feature = "desktop-sdl-ash")]
struct WaterPreview {
    catalog: next_contracts::render_content::RenderContentCatalogV1,
    snapshot: PresentationSnapshotV3,
    mesh_revision: next_contracts::project::AssetRevisionRefV1,
    bounds: AabbI64V1,
    frame_plan_root: next_contracts::ids::ContentHash,
    keyframes: Vec<PreviewKeyframe>,
    dynamic: Option<DynamicPreview>,
    visible_object_count: u32,
    indexed_draw_count: u32,
}

#[cfg(feature = "desktop-sdl-ash")]
fn build_preview(request: &WaterPreviewRequest) -> Result<WaterPreview, String> {
    let parsed = request
        .meshes
        .iter()
        .map(|path| parse_obj(path))
        .collect::<Result<Vec<_>, _>>()?;
    let bounds = union_bounds(parsed.iter().map(|obj| obj.bounds))?;
    let first = &parsed[0];
    let mut source =
        next_reference_game::project_source_v7_with_id("org.nextengine.developer.water-preview")
            .map_err(|error| error.to_string())?;
    let mesh_schema = source
        .render_records
        .iter()
        .find_map(|record| match record {
            NeutralRenderRecordV1::Mesh(mesh) => Some(mesh.schema_ref().clone()),
            _ => None,
        })
        .ok_or_else(|| "reference project has no mesh schema".to_owned())?;
    let texture_schema = source
        .render_records
        .iter()
        .find_map(|record| match record {
            NeutralRenderRecordV1::Texture(texture) => Some(texture.schema_ref().clone()),
            _ => None,
        })
        .ok_or_else(|| "reference project has no texture schema".to_owned())?;
    let material_schema = source
        .render_records
        .iter()
        .find_map(|record| match record {
            NeutralRenderRecordV1::Material(material) => Some(material.schema_ref().clone()),
            _ => None,
        })
        .ok_or_else(|| "reference project has no material schema".to_owned())?;

    let texture = NeutralTextureV1::new(
        texture_schema,
        WATER_TEXTURE_ASSET_ID,
        1,
        NeutralTextureDimensionV1::D2,
        [1, 1, 1],
        1,
        NeutralTextureColorSpaceV1::Srgb,
        NeutralTextureAlphaSemanticsV1::Opaque,
        NeutralTexelEncodingV1::Rgba8Unorm,
        vec![NeutralTextureMipLevelV1::new(
            [1, 1, 1],
            vec![52, 142, 220, 255],
        )],
    )
    .map_err(|error| error.to_string())?;
    let texture_revision = texture
        .asset_revision()
        .map_err(|error| error.to_string())?;
    let material = NeutralMaterialV1::new(
        material_schema,
        WATER_MATERIAL_ASSET_ID,
        1,
        [52_000, 58_000, u16::MAX, u16::MAX],
        MaterialColorSpaceV1::Linear,
        0,
        u16::MAX,
        [0; 3],
        MaterialColorSpaceV1::Linear,
        0,
        65_536,
        u16::MAX,
        MaterialAlphaModeV1::Opaque,
        0,
        false,
        vec![
            NeutralMaterialTextureBindingV1::new(
                MaterialTextureSlotV1::BaseColor,
                texture_revision,
                0,
                UvTransformV1::identity(),
            )
            .map_err(|error| error.to_string())?,
        ],
        Vec::new(),
    )
    .map_err(|error| error.to_string())?;
    let material_revision = material
        .asset_revision()
        .map_err(|error| error.to_string())?;
    // The catalog mesh is the stable identity and declared presentation
    // envelope: the first keyframe's geometry inside the union bounds of every
    // keyframe. Dynamic updates replace only the renderer ring, never this
    // record, so the catalog root is constant for the whole run.
    let mesh = NeutralMeshV1::new(
        mesh_schema,
        WATER_MESH_ASSET_ID,
        1,
        bounds,
        first.positions.clone(),
        Some(first.normals.clone()),
        None,
        vec![first.uv.clone()],
        first.indices.clone(),
        vec![
            NeutralMeshPrimitiveV1::new(
                MeshPrimitiveTopologyV1::Triangles,
                0,
                u32::try_from(first.indices.len())
                    .map_err(|_| "water-preview index count overflow".to_owned())?,
                0,
            )
            .map_err(|error| error.to_string())?,
        ],
    )
    .map_err(|error| error.to_string())?;
    let mesh_revision = mesh.asset_revision().map_err(|error| error.to_string())?;

    let mut authoring_preimage = source.authoring_sha256.as_bytes().to_vec();
    authoring_preimage.extend_from_slice(mesh_revision.record_sha256.as_bytes());
    authoring_preimage.extend_from_slice(material_revision.record_sha256.as_bytes());
    authoring_preimage.extend_from_slice(texture_revision.record_sha256.as_bytes());
    source.authoring_sha256 =
        domain_hash("nextengine.water-preview.authoring.v1", &authoring_preimage);
    source.render_records.extend([
        NeutralRenderRecordV1::from(mesh.clone()),
        NeutralRenderRecordV1::from(texture),
        NeutralRenderRecordV1::from(material),
    ]);
    source.root_asset_ids.extend([
        WATER_MESH_ASSET_ID,
        WATER_TEXTURE_ASSET_ID,
        WATER_MATERIAL_ASSET_ID,
    ]);
    let cooked = next_project::cook_project_v7(source).map_err(|error| error.to_string())?;
    let epoch = domain_hash(
        "nextengine.water-preview.epoch.v1",
        mesh_revision.record_sha256.as_bytes(),
    );
    let scene = ScenePresentationRecordV2::new(
        32,
        PresentationObjectKeyV1 {
            snapshot_epoch: epoch,
            persistent_id: PersistentId::from_bytes([0xf4; 16]),
            presentation_role: PresentationRoleV1::Environment,
            incarnation: 0,
        },
        mesh_revision,
        material_revision,
        0,
        mesh.bounds(),
        ScenePresentationFlagsV1::NONE,
        QuantizedPresentationTransformV1::default(),
        QuantizedPresentationTransformV1::default(),
        true,
    );
    let focus = [800_000, 230_000, 500_000];
    let camera_result = CameraResultSampleV1 {
        pose: QuantizedPresentationTransformV1 {
            translation_micrometres: [800_000, 1_250_000, 2_350_000],
            ..QuantizedPresentationTransformV1::default()
        },
        focus_point_micrometres: focus,
    };
    let camera = CameraPresentationRecordV2::new(
        epoch,
        PersistentId::from_bytes([0xf5; 16]),
        CameraRoleV1::PrimaryThirdPerson,
        CameraViewportV1::full(0),
        CameraProjectionProfileV1::new(52_000, 20_000, 20_000_000)
            .map_err(|error| error.to_string())?,
        ThirdPersonCameraIntentSampleV1 {
            focus_subject_id: None,
            focus_point_micrometres: focus,
            orbit_yaw_millidegrees: 0,
            orbit_pitch_millidegrees: -24_000,
            distance_micrometres: 2_400_000,
            shoulder_offset_micrometres: [0; 3],
        },
        camera_result,
        camera_result,
        cooked.render_content_catalog.profile_revision(),
        true,
        CameraInterpolationPolicyV1::Hold,
    )
    .map_err(|error| error.to_string())?;
    let snapshot = PresentationSnapshotV3::new_with_camera_records(
        epoch,
        0,
        0,
        cooked.project_lock.project_lock_sha256,
        cooked.content_manifest.content_manifest_sha256,
        domain_hash(
            "nextengine.water-preview.presentation-profile.v1",
            b"static-obj",
        ),
        vec![scene],
        vec![camera],
        8,
        1,
        domain_hash("nextengine.water-preview.environment.v1", b"clear-sky"),
    )
    .map_err(|error| error.to_string())?;
    let target = RenderTargetV1 {
        extent: request.extent,
        target_revision: 1,
    };
    let frame_plan = build_b0_frame_plan(&snapshot, &cooked.render_content_catalog, target)
        .map_err(|error| error.to_string())?;

    let dynamic = if parsed.len() >= 2 {
        let vertex_capacity = parsed
            .iter()
            .map(|obj| obj.positions.len())
            .max()
            .and_then(|count| u32::try_from(count).ok())
            .ok_or_else(|| "water-preview vertex capacity overflow".to_owned())?;
        let index_capacity = parsed
            .iter()
            .map(|obj| obj.indices.len())
            .max()
            .and_then(|count| u32::try_from(count).ok())
            .ok_or_else(|| "water-preview index capacity overflow".to_owned())?;
        let keyframes = parsed
            .iter()
            .map(|obj| {
                DynamicSurfaceUpdateV1::new(
                    mesh_revision,
                    1,
                    obj.positions.clone(),
                    obj.normals.clone(),
                    obj.indices.clone(),
                )
                .map(|update| DynamicKeyframe { update })
                .map_err(|error| error.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Some(DynamicPreview {
            profile: DynamicSurfaceProfileV1 {
                mesh_revision,
                vertex_capacity,
                index_capacity,
            },
            keyframes,
        })
    } else {
        None
    };
    let keyframes = request
        .meshes
        .iter()
        .zip(&parsed)
        .enumerate()
        .map(|(index, (path, obj))| PreviewKeyframe {
            path: path.clone(),
            source_sha256: obj.source_sha256.clone(),
            vertex_count: obj.positions.len(),
            triangle_count: obj.indices.len() / 3,
            update_hash: dynamic
                .as_ref()
                .map(|dynamic| dynamic.keyframes[index].update.canonical_hash()),
        })
        .collect();
    Ok(WaterPreview {
        catalog: cooked.render_content_catalog,
        snapshot,
        mesh_revision,
        bounds,
        frame_plan_root: frame_plan.frame_plan_hash,
        keyframes,
        dynamic,
        visible_object_count: frame_plan.visible_object_count,
        indexed_draw_count: frame_plan.indexed_draw_count,
    })
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
struct ParsedObj {
    positions: Vec<[i64; 3]>,
    normals: Vec<[i16; 3]>,
    uv: Vec<[i32; 2]>,
    indices: Vec<u32>,
    bounds: AabbI64V1,
    source_sha256: String,
}

#[cfg(feature = "desktop-sdl-ash")]
fn parse_obj(path: &Path) -> Result<ParsedObj, String> {
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_OBJ_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_OBJ_BYTES {
        return Err("water-preview OBJ size is outside the bounded profile".to_owned());
    }
    let source =
        std::str::from_utf8(&bytes).map_err(|_| "water-preview OBJ is not UTF-8".to_owned())?;
    parse_obj_text(source)
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn parse_obj_text(source: &str) -> Result<ParsedObj, String> {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for (line_index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_ascii_whitespace();
        match fields.next() {
            Some("v") => {
                if positions.len() >= MAX_VERTICES {
                    return Err("water-preview OBJ vertex capacity exceeded".to_owned());
                }
                let mut position = [0_i64; 3];
                for component in &mut position {
                    let value: f64 = fields
                        .next()
                        .ok_or_else(|| format!("OBJ line {} has a short vertex", line_index + 1))?
                        .parse()
                        .map_err(|_| {
                            format!("OBJ line {} has an invalid vertex", line_index + 1)
                        })?;
                    let micrometres = value * 1_000_000.0;
                    if !micrometres.is_finite()
                        || micrometres < i64::MIN as f64
                        || micrometres > i64::MAX as f64
                    {
                        return Err(format!(
                            "OBJ line {} has a non-finite/out-of-range vertex",
                            line_index + 1
                        ));
                    }
                    *component = micrometres.round() as i64;
                }
                if fields.next().is_some() {
                    return Err(format!(
                        "OBJ line {} has extra vertex fields",
                        line_index + 1
                    ));
                }
                positions.push(position);
            }
            Some("f") => {
                if indices.len() / 3 >= MAX_TRIANGLES {
                    return Err("water-preview OBJ triangle capacity exceeded".to_owned());
                }
                for _ in 0..3 {
                    let token = fields
                        .next()
                        .ok_or_else(|| format!("OBJ line {} has a short face", line_index + 1))?;
                    let raw = token.split('/').next().ok_or_else(|| {
                        format!("OBJ line {} has an invalid face", line_index + 1)
                    })?;
                    let index: usize = raw
                        .parse()
                        .map_err(|_| format!("OBJ line {} has an invalid face", line_index + 1))?;
                    let zero_based = index.checked_sub(1).ok_or_else(|| {
                        format!("OBJ line {} uses a zero face index", line_index + 1)
                    })?;
                    indices.push(u32::try_from(zero_based).map_err(|_| {
                        format!("OBJ line {} face index overflows", line_index + 1)
                    })?);
                }
                if fields.next().is_some() {
                    return Err(format!("OBJ line {} is not a triangle", line_index + 1));
                }
            }
            Some(_) | None => {
                return Err(format!(
                    "OBJ line {} uses an unsupported record",
                    line_index + 1
                ));
            }
        }
    }
    if positions.len() < 3 || indices.is_empty() {
        return Err("water-preview OBJ has no renderable triangles".to_owned());
    }
    if indices
        .iter()
        .any(|index| usize::try_from(*index).map_or(true, |index| index >= positions.len()))
    {
        return Err("water-preview OBJ face index is outside the vertex array".to_owned());
    }
    let bounds = obj_bounds(&positions)?;
    let normals = smooth_normals(&positions, &indices)?;
    let uv = planar_uv(&positions, bounds);
    Ok(ParsedObj {
        positions,
        normals,
        uv,
        indices,
        bounds,
        source_sha256: format!("{:x}", Sha256::digest(source.as_bytes())),
    })
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn obj_bounds(positions: &[[i64; 3]]) -> Result<AabbI64V1, String> {
    let mut min = positions[0];
    let mut max = positions[0];
    for position in &positions[1..] {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    for axis in 0..3 {
        max[axis] = max[axis]
            .checked_add(1)
            .ok_or_else(|| "water-preview OBJ bounds overflow".to_owned())?;
        if min[axis] == max[axis] {
            return Err("water-preview OBJ has invalid bounds".to_owned());
        }
    }
    AabbI64V1::new(min, max).map_err(|error| error.to_string())
}

/// Smallest axis-aligned box containing every keyframe. It becomes the
/// declared presentation envelope of the catalog mesh, so every later
/// dynamic update stays inside the bounds the frame plan validated.
#[cfg(any(feature = "desktop-sdl-ash", test))]
fn union_bounds(bounds: impl IntoIterator<Item = AabbI64V1>) -> Result<AabbI64V1, String> {
    let mut bounds = bounds.into_iter();
    let first = bounds
        .next()
        .ok_or_else(|| "water-preview has no keyframe bounds".to_owned())?;
    let mut min = first.min();
    let mut max = first.max();
    for next in bounds {
        for axis in 0..3 {
            min[axis] = min[axis].min(next.min()[axis]);
            max[axis] = max[axis].max(next.max()[axis]);
        }
    }
    AabbI64V1::new(min, max).map_err(|error| error.to_string())
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn smooth_normals(positions: &[[i64; 3]], indices: &[u32]) -> Result<Vec<[i16; 3]>, String> {
    let mut sums = vec![[0.0_f64; 3]; positions.len()];
    for triangle in indices.chunks_exact(3) {
        let a = positions[triangle[0] as usize].map(|value| value as f64);
        let b = positions[triangle[1] as usize].map(|value| value as f64);
        let c = positions[triangle[2] as usize].map(|value| value as f64);
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let normal = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        if normal.iter().all(|value| *value == 0.0) {
            return Err("water-preview OBJ contains a degenerate triangle".to_owned());
        }
        for index in triangle {
            let sum = &mut sums[*index as usize];
            for axis in 0..3 {
                sum[axis] += normal[axis];
            }
        }
    }
    sums.into_iter()
        .map(|sum| {
            let length = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
            if !length.is_finite() {
                return Err("water-preview OBJ contains an invalid vertex normal".to_owned());
            }
            if length == 0.0 {
                // Sparse height-field export may retain a vertex whose adjacent
                // cells were culled. It is never indexed, but the neutral mesh
                // still requires a finite unit attribute for every vertex.
                return Ok([0, i16::MAX, 0]);
            }
            let mut normal = [0_i16; 3];
            for axis in 0..3 {
                let value = (sum[axis] / length * f64::from(i16::MAX)).round();
                normal[axis] = value.clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16;
            }
            Ok(normal)
        })
        .collect()
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn planar_uv(positions: &[[i64; 3]], bounds: AabbI64V1) -> Vec<[i32; 2]> {
    let min = bounds.min();
    let max = bounds.max();
    let span_x = (max[0] - min[0]).max(1) as f64;
    let span_z = (max[2] - min[2]).max(1) as f64;
    positions
        .iter()
        .map(|position| {
            [
                ((position[0] - min[0]) as f64 / span_x * 65_536.0).round() as i32,
                ((position[2] - min[2]) as f64 / span_z * 65_536.0).round() as i32,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obj_parser_builds_bounded_triangle_mesh() {
        let parsed =
            parse_obj_text("v 0 0 0\nv 1 0 0\nv 0 0 1\nf 1 2 3\n").expect("triangle parses");
        assert_eq!(parsed.positions.len(), 3);
        assert_eq!(parsed.indices, [0, 1, 2]);
        assert_eq!(parsed.normals, [[0, -32_767, 0]; 3]);
        assert_eq!(parsed.uv.len(), 3);
        assert_eq!(parsed.source_sha256.len(), 64);
        assert!(parsed.bounds.contains([0, 0, 0]));
    }

    #[test]
    fn obj_parser_rejects_non_triangles_and_nonfinite_vertices() {
        assert!(parse_obj_text("v 0 0 0\nv 1 0 0\nv 0 0 1\nf 1 2 3 1\n").is_err());
        assert!(parse_obj_text("v NaN 0 0\nv 1 0 0\nv 0 0 1\nf 1 2 3\n").is_err());
    }

    #[test]
    fn nearest_rank_retains_outliers() {
        assert_eq!(nearest_rank(&[1, 2, 3, 100], 95), Some(100));
        assert_eq!(nearest_rank(&[], 95), None);
    }

    #[test]
    fn arguments_accept_repeated_keyframes_and_hold() {
        let root = Path::new("/root");
        let request = parse_arguments(
            [
                "--mesh",
                "a.obj",
                "--mesh",
                "/abs/b.obj",
                "--hold",
                "3",
                "--frames",
                "12",
            ]
            .into_iter()
            .map(str::to_owned),
            root,
        )
        .expect("keyframe arguments parse");
        assert_eq!(
            request.meshes,
            [PathBuf::from("/root/a.obj"), PathBuf::from("/abs/b.obj")]
        );
        assert_eq!(request.hold, 3);
        assert_eq!(request.frames, 12);
        assert!(parse_arguments(std::iter::empty(), root).is_err());
        assert!(
            parse_arguments(
                ["--mesh", "a.obj", "--hold", "0"]
                    .into_iter()
                    .map(str::to_owned),
                root
            )
            .is_err()
        );
    }

    #[test]
    fn keyframe_schedule_is_a_pure_function_of_the_pump_index() {
        assert_eq!(keyframe_for_pump(0, 3, 5), 0);
        assert_eq!(keyframe_for_pump(2, 3, 5), 0);
        assert_eq!(keyframe_for_pump(3, 3, 5), 1);
        assert_eq!(keyframe_for_pump(14, 3, 5), 4);
        assert_eq!(keyframe_for_pump(15, 3, 5), 0);
        assert_eq!(keyframe_for_pump(7, 1, 1), 0);
    }

    #[test]
    fn union_bounds_contains_every_keyframe() {
        let first = obj_bounds(&[[0, 0, 0], [10, 5, 10]]).expect("first bounds");
        let second = obj_bounds(&[[-4, 2, 3], [6, 9, 20]]).expect("second bounds");
        let union = union_bounds([first, second]).expect("union");
        assert_eq!(union.min(), [-4, 0, 0]);
        assert_eq!(union.max(), [11, 10, 21]);
        assert!(union.contains([10, 5, 10]));
        assert!(union.contains([-4, 9, 20]));
        assert!(union_bounds(std::iter::empty()).is_err());
    }

    #[cfg(feature = "desktop-sdl-ash")]
    #[test]
    fn preview_builds_one_b0_indexed_draw() {
        let path = std::env::temp_dir().join(format!(
            "nextengine-water-preview-{}.obj",
            std::process::id()
        ));
        std::fs::write(&path, "v 0 0 0\nv 0 0 1\nv 1 0 0\nf 1 2 3\n")
            .expect("writes bounded OBJ fixture");
        let preview = build_preview(&WaterPreviewRequest {
            meshes: vec![path.clone()],
            frames: 1,
            extent: [640, 480],
            hold: 1,
        })
        .expect("builds B0 preview");
        std::fs::remove_file(path).expect("removes bounded OBJ fixture");
        assert_eq!(preview.visible_object_count, 1);
        assert_eq!(preview.indexed_draw_count, 1);
        assert!(preview.dynamic.is_none());
        assert_eq!(preview.keyframes.len(), 1);
    }

    #[cfg(feature = "desktop-sdl-ash")]
    #[test]
    fn keyframe_sequence_declares_one_bounded_dynamic_surface() {
        let first = std::env::temp_dir().join(format!(
            "nextengine-water-preview-{}-a.obj",
            std::process::id()
        ));
        let second = std::env::temp_dir().join(format!(
            "nextengine-water-preview-{}-b.obj",
            std::process::id()
        ));
        std::fs::write(&first, "v 0 0 0\nv 0 0 1\nv 1 0 0\nf 1 2 3\n").expect("first keyframe");
        std::fs::write(
            &second,
            "v 0 0.5 0\nv 0 0 2\nv 2 0 0\nv 2 0.25 2\nf 1 2 3\nf 3 2 4\n",
        )
        .expect("second keyframe");
        let preview = build_preview(&WaterPreviewRequest {
            meshes: vec![first.clone(), second.clone()],
            frames: 1,
            extent: [640, 480],
            hold: 2,
        })
        .expect("builds dynamic preview");
        std::fs::remove_file(first).expect("removes first keyframe");
        std::fs::remove_file(second).expect("removes second keyframe");
        let dynamic = preview.dynamic.as_ref().expect("dynamic surface declared");
        assert_eq!(dynamic.profile.mesh_revision, preview.mesh_revision);
        assert_eq!(dynamic.profile.vertex_capacity, 4);
        assert_eq!(dynamic.profile.index_capacity, 6);
        assert_eq!(dynamic.keyframes.len(), 2);
        assert_ne!(
            dynamic.keyframes[0].update.canonical_hash(),
            dynamic.keyframes[1].update.canonical_hash()
        );
        for keyframe in &dynamic.keyframes {
            assert!(
                keyframe
                    .update
                    .positions_micrometres()
                    .iter()
                    .all(|position| preview.bounds.contains(*position))
            );
        }
        assert_eq!(
            preview
                .catalog
                .mesh(preview.mesh_revision)
                .map(|mesh| mesh.bounds()),
            Some(preview.bounds)
        );
        assert_eq!(preview.visible_object_count, 1);
        assert_eq!(preview.indexed_draw_count, 1);
    }
}
