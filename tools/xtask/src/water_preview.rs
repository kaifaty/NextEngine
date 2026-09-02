use std::path::{Path, PathBuf};

#[cfg(feature = "desktop-sdl-ash")]
use std::fs::File;
#[cfg(feature = "desktop-sdl-ash")]
use std::io::Read;
#[cfg(feature = "desktop-sdl-ash")]
use std::sync::Arc;
#[cfg(feature = "desktop-sdl-ash")]
use std::time::Duration;

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
    DynamicSurfaceResidencyV1, DynamicSurfaceUpdateV1,
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
#[cfg(feature = "desktop-sdl-ash")]
const BASIN_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xf6; 16]);
#[cfg(feature = "desktop-sdl-ash")]
const BASIN_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xf7; 16]);
#[cfg(feature = "desktop-sdl-ash")]
const BASIN_TEXTURE_ASSET_ID: AssetId = AssetId::from_bytes([0xf9; 16]);

pub(super) struct WaterPreviewRequest {
    /// One OBJ renders the static catalog path. Two or more OBJ keyframes
    /// declare one presentation-only dynamic surface and cycle through it.
    meshes: Vec<PathBuf>,
    frames: u64,
    extent: [u32; 2],
    /// Event-loop pumps each keyframe stays current before the next one is
    /// published.
    hold: u64,
    /// Live surface stream from the external research solver process.
    stream: Option<StreamRequest>,
    /// Dynamic surface ring residency: `device` (default) or `host` control.
    device_local_ring: bool,
    /// Render until the window is closed instead of a bounded frame count.
    until_close: bool,
    /// Copy one rendered frame to a PNG file (developer evidence only).
    capture: Option<CaptureRequest>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct CaptureRequest {
    rendered_frame_index: u64,
    png: PathBuf,
}

/// `nonlocal-feasibility --game-surface-stream` child-process parameters.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct StreamRequest {
    binary: PathBuf,
    lane: String,
    steps: u32,
    every: u32,
    /// Zero streams until the renderer finishes.
    cycles: u32,
    /// Ordered extraction worker threads inside the solver process.
    workers: u32,
    /// Surface extractor inside the solver process: `cpu`, `gpu` or `verify`.
    extractor: String,
    /// Presentation height model inside the solver process: `sphere` (frozen
    /// NGQ5) or `closing` (NGQ6 revision 2).
    surface_model: String,
    /// Stream seconds published per wall second; zero disables pacing and
    /// publishes every frame as soon as it arrives.
    rate: f64,
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
    root: &Path,
) -> Result<WaterPreviewRequest, String> {
    let mut meshes = Vec::new();
    let mut frames = 600_u64;
    let mut extent = [1280, 720];
    let mut hold = 8_u64;
    let mut stream_binary: Option<PathBuf> = None;
    let mut stream_lane = "4k".to_owned();
    let mut stream_steps = 960_u32;
    let mut stream_every = 4_u32;
    let mut stream_cycles = 0_u32;
    let mut stream_workers = 3_u32;
    let mut stream_extractor = "gpu".to_owned();
    let mut stream_surface_model = "closing".to_owned();
    let mut stream_rate = 1.0_f64;
    let mut device_local_ring = true;
    let mut until_close = false;
    let mut capture_frame: Option<u64> = None;
    let mut capture_png: Option<PathBuf> = None;
    let bounded_u32 = |arguments: &mut dyn Iterator<Item = String>,
                       name: &str,
                       low: u32,
                       high: u32|
     -> Result<u32, String> {
        let value: u32 = arguments
            .next()
            .ok_or_else(|| format!("water-preview {name} requires a value"))?
            .parse()
            .map_err(|_| format!("water-preview {name} is invalid"))?;
        if value < low || value > high {
            return Err(format!("water-preview {name} must be in {low}..={high}"));
        }
        Ok(value)
    };
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--until-close" => {
                until_close = true;
            }
            "--capture-frame" => {
                capture_frame = Some(
                    arguments
                        .next()
                        .ok_or_else(|| {
                            "water-preview --capture-frame requires an index".to_owned()
                        })?
                        .parse()
                        .map_err(|_| "water-preview capture frame index is invalid".to_owned())?,
                );
            }
            "--capture-png" => {
                let value = PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| "water-preview --capture-png requires a path".to_owned())?,
                );
                capture_png = Some(if value.is_relative() {
                    root.join(value)
                } else {
                    value
                });
            }
            "--ring" => {
                device_local_ring = match arguments
                    .next()
                    .ok_or_else(|| "water-preview --ring requires device or host".to_owned())?
                    .as_str()
                {
                    "device" => true,
                    "host" => false,
                    _ => return Err("water-preview --ring must be device or host".to_owned()),
                };
            }
            "--stream-binary" => {
                let value =
                    PathBuf::from(arguments.next().ok_or_else(|| {
                        "water-preview --stream-binary requires a path".to_owned()
                    })?);
                stream_binary = Some(if value.is_relative() {
                    root.join(value)
                } else {
                    value
                });
            }
            "--stream-lane" => {
                stream_lane = arguments.next().ok_or_else(|| {
                    "water-preview --stream-lane requires 4k, 16k, 48k or 48k-dam".to_owned()
                })?;
                if !matches!(stream_lane.as_str(), "4k" | "16k" | "48k" | "48k-dam") {
                    return Err(
                        "water-preview --stream-lane must be 4k, 16k, 48k or 48k-dam".to_owned(),
                    );
                }
            }
            "--stream-steps" => {
                stream_steps = bounded_u32(&mut arguments, "--stream-steps", 1, 100_000)?;
            }
            "--stream-every" => {
                stream_every = bounded_u32(&mut arguments, "--stream-every", 1, 240)?;
            }
            "--stream-cycles" => {
                stream_cycles = bounded_u32(&mut arguments, "--stream-cycles", 0, 1_000_000)?;
            }
            "--stream-workers" => {
                stream_workers = bounded_u32(&mut arguments, "--stream-workers", 1, 16)?;
            }
            "--stream-surface-model" => {
                stream_surface_model = arguments.next().ok_or_else(|| {
                    "water-preview --stream-surface-model requires sphere or closing".to_owned()
                })?;
                if !matches!(stream_surface_model.as_str(), "sphere" | "closing") {
                    return Err(
                        "water-preview --stream-surface-model must be sphere or closing".to_owned(),
                    );
                }
            }
            "--stream-extractor" => {
                stream_extractor = arguments.next().ok_or_else(|| {
                    "water-preview --stream-extractor requires cpu, gpu or verify".to_owned()
                })?;
                if !matches!(stream_extractor.as_str(), "cpu" | "gpu" | "verify") {
                    return Err(
                        "water-preview --stream-extractor must be cpu, gpu or verify".to_owned(),
                    );
                }
            }
            "--stream-rate" => {
                stream_rate = arguments
                    .next()
                    .ok_or_else(|| "water-preview --stream-rate requires a value".to_owned())?
                    .parse()
                    .map_err(|_| "water-preview stream rate is invalid".to_owned())?;
                if !stream_rate.is_finite() || !(0.0..=100.0).contains(&stream_rate) {
                    return Err("water-preview stream rate must be in 0..=100".to_owned());
                }
            }
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
    let stream = stream_binary.map(|binary| StreamRequest {
        binary,
        lane: stream_lane,
        steps: stream_steps,
        every: stream_every,
        cycles: stream_cycles,
        workers: stream_workers,
        extractor: stream_extractor,
        surface_model: stream_surface_model,
        rate: stream_rate,
    });
    if let Some(stream) = &stream {
        if !meshes.is_empty() {
            return Err("water-preview --stream-binary cannot be combined with --mesh".to_owned());
        }
        if stream.every > stream.steps {
            return Err("water-preview --stream-every cannot exceed --stream-steps".to_owned());
        }
    } else if meshes.is_empty() {
        return Err(
            "water-preview requires --mesh <surface.obj> or --stream-binary <path>".to_owned(),
        );
    }
    let capture = match (capture_frame, capture_png) {
        (Some(rendered_frame_index), Some(png)) => Some(CaptureRequest {
            rendered_frame_index,
            png,
        }),
        (None, None) => None,
        _ => {
            return Err(
                "water-preview --capture-frame and --capture-png must be given together".to_owned(),
            );
        }
    };
    if let Some(capture) = &capture
        && !until_close
        && capture.rendered_frame_index >= frames
    {
        return Err("water-preview --capture-frame must be below --frames".to_owned());
    }
    Ok(WaterPreviewRequest {
        meshes,
        frames,
        extent,
        hold,
        stream,
        device_local_ring,
        until_close,
        capture,
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
    let mut stream_session = request
        .stream
        .as_ref()
        .map(StreamSession::spawn)
        .transpose()?;
    let source = match stream_session.as_mut() {
        Some(session) => PreviewSource::Stream(session.take_first_frame()?),
        None => PreviewSource::Keyframes(
            request
                .meshes
                .iter()
                .map(|path| parse_obj(path))
                .collect::<Result<Vec<_>, _>>()?,
        ),
    };
    let preview = build_preview(request, source)?;
    let dynamic = preview.dynamic.as_ref();
    let options = next_desktop_sdl_ash::DesktopRunOptions {
        title: "Next Engine — Nonlocal Water Preview".to_owned(),
        initial_extent: request.extent,
        maximum_frames: (!request.until_close).then_some(request.frames),
        maximum_event_loop_iterations: (!request.until_close)
            .then(|| request.frames.saturating_mul(8)),
        frame_profiling_sample_capacity: if request.until_close {
            next_desktop_sdl_ash::MAX_FRAME_PROFILING_SAMPLES
        } else {
            u32::try_from(request.frames)
                .map_err(|_| "water-preview profiling capacity overflow".to_owned())?
        },
        frame_capture: request.capture.as_ref().map(|capture| {
            next_desktop_sdl_ash::DesktopFrameCaptureRequestV1 {
                rendered_frame_index: capture.rendered_frame_index,
            }
        }),
        audio_output_enabled: false,
        dynamic_surfaces: dynamic
            .map(|dynamic| vec![dynamic.profile])
            .unwrap_or_default(),
        ..next_desktop_sdl_ash::DesktopRunOptions::default()
    };
    let mut feed = match (dynamic, stream_session.as_mut()) {
        (Some(dynamic), Some(session)) => {
            session.start_conversion(dynamic.profile.mesh_revision);
            DynamicFeed::Stream(Box::new(StreamFeed::new(session, request)?))
        }
        (Some(_), None) => DynamicFeed::Keyframes(KeyframeFeed::default()),
        (None, _) => DynamicFeed::Static,
    };
    let report = if let Some(dynamic) = dynamic {
        next_desktop_sdl_ash::run_interactive_with_shared_frame_publication_and_finalize(
            Arc::new(preview.snapshot.clone()),
            &preview.catalog,
            &options,
            |_events, elapsed, _audio| {
                let update = feed.next_update(request, dynamic, elapsed)?;
                Ok(DesktopFramePublicationV1 {
                    snapshot: None,
                    dynamic_surface_updates: update.into_iter().map(Arc::new).collect(),
                })
            },
            || DesktopApplicationFinalization::Complete,
        )
    } else {
        next_desktop_sdl_ash::run_interactive(&preview.snapshot, &preview.catalog, &options)
    }
    .map_err(|error| error.to_string())?;
    let mode = match &feed {
        DynamicFeed::Static => "static",
        DynamicFeed::Keyframes(_) => "dynamic-keyframes",
        DynamicFeed::Stream(_) => "dynamic-stream",
    };
    let feed_summary = feed.summary();
    // Dropping the feed closes the converted-frame channel so the child
    // observes a closed pipe and exits before the summary is collected.
    drop(feed);
    let stream_summary = stream_session.map(StreamSession::finish).transpose()?;
    if report.rendered_objects != u64::from(preview.visible_object_count)
        || report.indexed_draws != u64::from(preview.indexed_draw_count)
    {
        return Err("water-preview Vulkan report does not contain the planned draw".to_owned());
    }
    let capture_json = match (&request.capture, &report.captured_frame) {
        (Some(capture), Some(frame)) => {
            let png = encode_png_rgba8(frame.extent, &frame.rgba8)?;
            std::fs::write(&capture.png, &png)
                .map_err(|error| format!("{}: {error}", capture.png.display()))?;
            Some(serde_json::json!({
                "rendered_frame_index": frame.rendered_frame_index,
                "extent": frame.extent,
                "png": capture.png,
                "png_sha256": format!("{:x}", Sha256::digest(&png)),
                "rgba8_sha256": format!("{:x}", Sha256::digest(&frame.rgba8)),
            }))
        }
        (Some(capture), None) => {
            return Err(format!(
                "water-preview did not reach capture frame {}",
                capture.rendered_frame_index
            ));
        }
        (None, _) => None,
    };
    if let Some(dynamic) = dynamic {
        if report.dynamic_surface_publications != feed_summary.publications {
            return Err("water-preview adapter accepted a different publication count".to_owned());
        }
        if report.dynamic_surface_draws != 1 {
            return Err("water-preview last frame did not draw the dynamic surface".to_owned());
        }
        let slot_count = u64::try_from(next_desktop_sdl_ash::DESKTOP_FRAME_SLOT_COUNT)
            .map_err(|_| "water-preview frame slot count overflow".to_owned())?;
        if report.dynamic_surface_uploads < feed_summary.publications
            || report.dynamic_surface_uploads > feed_summary.publications.saturating_mul(slot_count)
        {
            return Err(
                "water-preview ring refresh count is outside the publication bound".to_owned(),
            );
        }
        let expected_hash = feed_summary
            .last_hash
            .ok_or_else(|| "water-preview published no surface update".to_owned())?;
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
    let stream = request.stream.as_ref().map(|stream| {
        serde_json::json!({
            "binary": stream.binary,
            "lane": stream.lane,
            "steps": stream.steps,
            "every": stream.every,
            "cycles": stream.cycles,
            "workers": stream.workers,
            "extractor": stream.extractor,
            "surface_model": stream.surface_model,
            "rate": stream.rate,
            "frames_received": feed_summary.frames_received,
            "frames_published": feed_summary.publications,
            "frames_skipped": feed_summary.frames_skipped,
            "first_frame_wait_ms": feed_summary.first_frame_wait_ms,
            "published_stream_seconds": feed_summary.published_stream_seconds,
            "wall_seconds": feed_summary.wall_seconds,
            "realtime_ratio": (feed_summary.wall_seconds > 0.0)
                .then(|| feed_summary.published_stream_seconds / feed_summary.wall_seconds),
            "last_published_step": feed_summary.last_step,
            "last_published_cycle": feed_summary.last_cycle,
            "extraction_ms_p95": nearest_rank_f64(&feed_summary.extraction_ms, 95),
            "physics_ms_per_frame_p95": nearest_rank_f64(&feed_summary.physics_ms, 95),
            "child": stream_summary,
        })
    });
    println!(
        "{}",
        serde_json::json!({
            "schema_version": 3,
            "command": "water-preview",
            "status": "PASS",
            "authority": "PRESENTATION_ONLY_TOOL",
            "mode": mode,
            "ring_residency": if request.device_local_ring { "device-local" } else { "host-visible" },
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
            "stream": stream,
            "capture": capture_json,
            "until_close": request.until_close,
        })
    );
    Ok(())
}

#[cfg(not(feature = "desktop-sdl-ash"))]
pub(super) fn run(request: &WaterPreviewRequest) -> Result<(), String> {
    Err(format!(
        "water-preview for {} keyframe(s), stream {:?} ({} frames at {}x{}, hold {}, {} ring, until-close {}, capture {:?}) requires --features desktop-sdl-ash",
        request.meshes.len(),
        request.stream.as_ref().map(|stream| stream.lane.as_str()),
        request.frames,
        request.extent[0],
        request.extent[1],
        request.hold,
        if request.device_local_ring {
            "device-local"
        } else {
            "host-visible"
        },
        request.until_close,
        request
            .capture
            .as_ref()
            .map(|capture| capture.rendered_frame_index)
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
fn nearest_rank_f64(values: &[f64], percentile: usize) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered.get(rank.saturating_sub(1)).copied()
}

/// Minimal PNG encoder (8-bit RGBA, filter type zero, one zlib stream) for
/// developer evidence; it adds no image dependency to the workspace.
#[cfg(feature = "desktop-sdl-ash")]
fn encode_png_rgba8(extent: [u32; 2], rgba8: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Write as _;
    let [width, height] = extent;
    let row_bytes = usize::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or_else(|| "water-preview capture width overflow".to_owned())?;
    let expected = usize::try_from(height)
        .ok()
        .and_then(|height| height.checked_mul(row_bytes))
        .ok_or_else(|| "water-preview capture height overflow".to_owned())?;
    if rgba8.len() != expected || width == 0 || height == 0 {
        return Err("water-preview capture payload does not match its extent".to_owned());
    }
    let mut filtered = Vec::with_capacity(expected + height as usize);
    for row in rgba8.chunks_exact(row_bytes) {
        filtered.push(0);
        filtered.extend_from_slice(row);
    }
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(&filtered)
        .and_then(|()| encoder.finish())
        .map_err(|error| format!("water-preview PNG compression failed: {error}"))
        .and_then(|compressed| {
            let mut png = Vec::new();
            png.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
            let mut ihdr = Vec::with_capacity(13);
            ihdr.extend_from_slice(&width.to_be_bytes());
            ihdr.extend_from_slice(&height.to_be_bytes());
            ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
            for (kind, payload) in [
                (b"IHDR", ihdr.as_slice()),
                (b"IDAT", compressed.as_slice()),
                (b"IEND", &[][..]),
            ] {
                let length = u32::try_from(payload.len())
                    .map_err(|_| "water-preview PNG chunk overflow".to_owned())?;
                png.extend_from_slice(&length.to_be_bytes());
                let mut hasher = crc32fast::Hasher::new();
                hasher.update(kind);
                hasher.update(payload);
                png.extend_from_slice(kind);
                png.extend_from_slice(payload);
                png.extend_from_slice(&hasher.finalize().to_be_bytes());
            }
            Ok(png)
        })
}

/// One converted stream frame ready for publication.
#[cfg(feature = "desktop-sdl-ash")]
struct ConvertedStreamFrame {
    step: i32,
    cycle: i32,
    stream_seconds: f64,
    extraction_ms: f64,
    physics_ms: f64,
    update: DynamicSurfaceUpdateV1,
}

#[cfg(feature = "desktop-sdl-ash")]
struct StreamSession {
    child: std::process::Child,
    frames: Option<std::sync::mpsc::Receiver<super::water_stream::StreamFrame>>,
    converted: Option<std::sync::mpsc::Receiver<ConvertedStreamFrame>>,
    reader: Option<std::thread::JoinHandle<Result<(), String>>>,
    converter: Option<std::thread::JoinHandle<Result<(), String>>>,
    stderr: Option<std::thread::JoinHandle<String>>,
    first_frame: Option<super::water_stream::StreamFrame>,
    first_frame_wait_ms: f64,
    steps: u32,
}

#[cfg(feature = "desktop-sdl-ash")]
impl StreamSession {
    fn spawn(request: &StreamRequest) -> Result<Self, String> {
        use std::process::{Command, Stdio};
        let mut child = Command::new(&request.binary)
            .args([
                "--game-surface-stream",
                "--lane",
                request.lane.as_str(),
                "--steps",
                &request.steps.to_string(),
                "--every",
                &request.every.to_string(),
                "--cycles",
                &request.cycles.to_string(),
                "--workers",
                &request.workers.to_string(),
                "--extractor",
                request.extractor.as_str(),
                "--surface-model",
                request.surface_model.as_str(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("{}: {error}", request.binary.display()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "water-preview stream child has no stdout".to_owned())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "water-preview stream child has no stderr".to_owned())?;
        let (frame_sender, frame_receiver) = std::sync::mpsc::sync_channel(8);
        let reader = std::thread::spawn(move || {
            let mut reader = std::io::BufReader::with_capacity(1 << 20, stdout);
            loop {
                match super::water_stream::read_frame(&mut reader, MAX_VERTICES, MAX_TRIANGLES)? {
                    Some(frame) => {
                        if frame_sender.send(frame).is_err() {
                            return Ok(());
                        }
                    }
                    None => return Ok(()),
                }
            }
        });
        let stderr = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = std::io::Read::read_to_string(&mut std::io::BufReader::new(stderr), &mut text);
            text
        });
        Ok(Self {
            child,
            frames: Some(frame_receiver),
            converted: None,
            reader: Some(reader),
            converter: None,
            stderr: Some(stderr),
            first_frame: None,
            first_frame_wait_ms: 0.0,
            steps: request.steps,
        })
    }

    /// Blocks for the solver's step-0 surface, which seeds the catalog mesh.
    fn take_first_frame(&mut self) -> Result<super::water_stream::StreamFrame, String> {
        let started = std::time::Instant::now();
        let frame = self
            .frames
            .as_ref()
            .ok_or_else(|| "water-preview stream already converted".to_owned())?
            .recv_timeout(std::time::Duration::from_secs(120))
            .map_err(|_| self.take_failure("water-preview stream produced no first frame"))?;
        self.first_frame_wait_ms = started.elapsed().as_secs_f64() * 1_000.0;
        self.first_frame = Some(frame.clone());
        Ok(frame)
    }

    fn take_failure(&mut self, context: &str) -> String {
        let reader = self.reader.take().and_then(|handle| handle.join().ok());
        match reader {
            Some(Err(error)) => format!("{context}: {error}"),
            _ => context.to_owned(),
        }
    }

    /// Moves normal reconstruction and update hashing off the render thread.
    fn start_conversion(&mut self, mesh_revision: next_contracts::project::AssetRevisionRefV1) {
        let Some(frames) = self.frames.take() else {
            return;
        };
        let (sender, receiver) = std::sync::mpsc::sync_channel(8);
        let steps = f64::from(self.steps);
        let first = self.first_frame.take();
        self.converter = Some(std::thread::spawn(move || {
            // Sequences follow arrival order and may leave gaps when the
            // feed skips a frame; the adapter requires only strict growth,
            // so the render thread publishes without cloning the payload.
            let mut sequence = 0_u64;
            let mut convert =
                |frame: super::water_stream::StreamFrame| -> Result<ConvertedStreamFrame, String> {
                    sequence += 1;
                    let normals = smooth_normals(&frame.positions_micrometres, &frame.indices)?;
                    let update = DynamicSurfaceUpdateV1::new(
                        mesh_revision,
                        sequence,
                        frame.positions_micrometres,
                        normals,
                        frame.indices,
                    )
                    .map_err(|error| error.to_string())?;
                    Ok(ConvertedStreamFrame {
                        step: frame.step,
                        cycle: frame.cycle,
                        stream_seconds: f64::from(frame.cycle)
                            * steps
                            * super::water_stream::SIMULATION_STEP_SECONDS
                            + frame.simulation_seconds,
                        extraction_ms: frame.extraction_ms,
                        physics_ms: frame.physics_ms,
                        update,
                    })
                };
            if let Some(first) = first
                && sender.send(convert(first)?).is_err()
            {
                return Ok(());
            }
            for frame in frames {
                if sender.send(convert(frame)?).is_err() {
                    return Ok(());
                }
            }
            Ok(())
        }));
        self.converted = Some(receiver);
    }

    fn finish(mut self) -> Result<serde_json::Value, String> {
        drop(self.converted.take());
        drop(self.frames.take());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Ok(None) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    break;
                }
                Err(error) => {
                    return Err(format!("water-preview stream child wait failed: {error}"));
                }
            }
        }
        let reader = self
            .reader
            .take()
            .and_then(|handle| handle.join().ok())
            .unwrap_or(Ok(()));
        let converter = self
            .converter
            .take()
            .and_then(|handle| handle.join().ok())
            .unwrap_or(Ok(()));
        let stderr = self
            .stderr
            .take()
            .and_then(|handle| handle.join().ok())
            .unwrap_or_default();
        reader?;
        converter?;
        let summary = stderr
            .lines()
            .rev()
            .find(|line| line.starts_with('{'))
            .map(serde_json::from_str::<serde_json::Value>)
            .transpose()
            .map_err(|error| format!("water-preview stream summary is not JSON: {error}"))?;
        Ok(summary.unwrap_or(serde_json::Value::Null))
    }
}

#[cfg(feature = "desktop-sdl-ash")]
#[derive(Default)]
struct FeedSummary {
    publications: u64,
    last_hash: Option<next_contracts::ids::ContentHash>,
    frames_received: u64,
    frames_skipped: u64,
    first_frame_wait_ms: f64,
    published_stream_seconds: f64,
    wall_seconds: f64,
    last_step: Option<i32>,
    last_cycle: Option<i32>,
    extraction_ms: Vec<f64>,
    physics_ms: Vec<f64>,
}

#[cfg(feature = "desktop-sdl-ash")]
#[derive(Default)]
struct KeyframeFeed {
    pump_index: u64,
    published_keyframe: Option<usize>,
    publications: u64,
    last_hash: Option<next_contracts::ids::ContentHash>,
}

#[cfg(feature = "desktop-sdl-ash")]
struct StreamFeed {
    converted: std::sync::mpsc::Receiver<ConvertedStreamFrame>,
    rate: f64,
    pending: Option<ConvertedStreamFrame>,
    summary: FeedSummary,
}

#[cfg(feature = "desktop-sdl-ash")]
impl StreamFeed {
    fn new(session: &mut StreamSession, request: &WaterPreviewRequest) -> Result<Self, String> {
        Ok(Self {
            converted: session
                .converted
                .take()
                .ok_or_else(|| "water-preview stream conversion was not started".to_owned())?,
            rate: request.stream.as_ref().map_or(1.0, |stream| stream.rate),
            pending: None,
            summary: FeedSummary {
                first_frame_wait_ms: session.first_frame_wait_ms,
                ..FeedSummary::default()
            },
        })
    }

    /// Publishes the newest frame whose stream time has been reached by the
    /// paced wall clock; earlier eligible frames are skipped, never queued.
    fn next(&mut self, elapsed: Duration) -> Option<DynamicSurfaceUpdateV1> {
        self.summary.wall_seconds += elapsed.as_secs_f64();
        let budget = if self.rate == 0.0 {
            f64::INFINITY
        } else {
            self.summary.wall_seconds * self.rate
        };
        let mut chosen: Option<ConvertedStreamFrame> = None;
        loop {
            if self.pending.is_none() {
                match self.converted.try_recv() {
                    Ok(frame) => {
                        self.summary.frames_received += 1;
                        self.pending = Some(frame);
                    }
                    Err(_) => break,
                }
            }
            let eligible = self
                .pending
                .as_ref()
                .is_some_and(|frame| frame.stream_seconds <= budget);
            if !eligible {
                break;
            }
            if chosen.is_some() {
                self.summary.frames_skipped += 1;
            }
            chosen = self.pending.take();
        }
        let frame = chosen?;
        self.summary.publications += 1;
        self.summary.last_hash = Some(frame.update.canonical_hash());
        self.summary.published_stream_seconds = frame.stream_seconds;
        self.summary.last_step = Some(frame.step);
        self.summary.last_cycle = Some(frame.cycle);
        self.summary.extraction_ms.push(frame.extraction_ms);
        self.summary.physics_ms.push(frame.physics_ms);
        Some(frame.update)
    }
}

#[cfg(feature = "desktop-sdl-ash")]
enum DynamicFeed {
    Static,
    Keyframes(KeyframeFeed),
    Stream(Box<StreamFeed>),
}

#[cfg(feature = "desktop-sdl-ash")]
impl DynamicFeed {
    fn next_update(
        &mut self,
        request: &WaterPreviewRequest,
        dynamic: &DynamicPreview,
        elapsed: Duration,
    ) -> Result<Option<DynamicSurfaceUpdateV1>, next_desktop_sdl_ash::DesktopAdapterError> {
        match self {
            Self::Static => Ok(None),
            Self::Keyframes(feed) => {
                let keyframe =
                    keyframe_for_pump(feed.pump_index, request.hold, dynamic.keyframes.len());
                feed.pump_index = feed.pump_index.saturating_add(1);
                if feed.published_keyframe == Some(keyframe) {
                    return Ok(None);
                }
                feed.publications = feed.publications.saturating_add(1);
                feed.published_keyframe = Some(keyframe);
                let update = dynamic.keyframes[keyframe]
                    .update
                    .with_sequence(feed.publications)?;
                feed.last_hash = Some(update.canonical_hash());
                Ok(Some(update))
            }
            Self::Stream(feed) => Ok(feed.next(elapsed)),
        }
    }

    fn summary(&self) -> FeedSummary {
        match self {
            Self::Static => FeedSummary::default(),
            Self::Keyframes(feed) => FeedSummary {
                publications: feed.publications,
                last_hash: feed.last_hash,
                ..FeedSummary::default()
            },
            Self::Stream(feed) => FeedSummary {
                publications: feed.summary.publications,
                last_hash: feed.summary.last_hash,
                frames_received: feed.summary.frames_received,
                frames_skipped: feed.summary.frames_skipped,
                first_frame_wait_ms: feed.summary.first_frame_wait_ms,
                published_stream_seconds: feed.summary.published_stream_seconds,
                wall_seconds: feed.summary.wall_seconds,
                last_step: feed.summary.last_step,
                last_cycle: feed.summary.last_cycle,
                extraction_ms: feed.summary.extraction_ms.clone(),
                physics_ms: feed.summary.physics_ms.clone(),
            },
        }
    }
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

/// Where the catalog placeholder mesh and the dynamic declaration come from.
#[cfg(feature = "desktop-sdl-ash")]
enum PreviewSource {
    Keyframes(Vec<ParsedObj>),
    Stream(super::water_stream::StreamFrame),
}

/// Declared envelope for a streamed lane: the solver box plus one pixel
/// pitch on every side, so any extracted height stays inside the bounds the
/// frame plan validates.
#[cfg(feature = "desktop-sdl-ash")]
fn stream_bounds(frame: &super::water_stream::StreamFrame) -> Result<AabbI64V1, String> {
    let margin = super::water_stream::SURFACE_PIXEL_PITCH_METRES;
    let quantize = |value: f64| -> Result<i64, String> {
        let micrometres = value * 1_000_000.0;
        if !micrometres.is_finite() || micrometres.abs() > 1.0e15 {
            return Err("water-preview stream box is outside the bounded profile".to_owned());
        }
        Ok(micrometres.round() as i64)
    };
    let mut min = [0_i64; 3];
    let mut max = [0_i64; 3];
    for axis in 0..3 {
        min[axis] = quantize(frame.box_min_metres[axis] - margin)?;
        max[axis] = quantize(frame.box_max_metres[axis] + margin)?;
    }
    AabbI64V1::new(min, max).map_err(|error| error.to_string())
}

#[cfg(feature = "desktop-sdl-ash")]
fn build_preview(
    request: &WaterPreviewRequest,
    source: PreviewSource,
) -> Result<WaterPreview, String> {
    let (parsed, bounds, capacity) = match source {
        PreviewSource::Keyframes(parsed) => {
            let bounds = union_bounds(parsed.iter().map(|obj| obj.bounds))?;
            let capacity = if parsed.len() >= 2 {
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
                Some((vertex_capacity, index_capacity))
            } else {
                None
            };
            (parsed, bounds, capacity)
        }
        PreviewSource::Stream(frame) => {
            let bounds = stream_bounds(&frame)?;
            let (vertex_capacity, index_capacity) =
                super::water_stream::surface_capacity(frame.box_min_metres, frame.box_max_metres)?;
            let normals = smooth_normals(&frame.positions_micrometres, &frame.indices)?;
            let uv = planar_uv(&frame.positions_micrometres, bounds);
            let mut preimage = Vec::new();
            for position in &frame.positions_micrometres {
                for component in position {
                    preimage.extend_from_slice(&component.to_le_bytes());
                }
            }
            for index in &frame.indices {
                preimage.extend_from_slice(&index.to_le_bytes());
            }
            let placeholder = ParsedObj {
                positions: frame.positions_micrometres,
                normals,
                uv,
                indices: frame.indices,
                bounds,
                source_sha256: format!("{:x}", Sha256::digest(&preimage)),
            };
            (
                vec![placeholder],
                bounds,
                Some((vertex_capacity, index_capacity)),
            )
        }
    };
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
        texture_schema.clone(),
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
        material_schema.clone(),
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
    // A static basin (floor plus four inward-facing walls at the declared
    // envelope) gives the water a visual reference. It is ordinary static
    // catalog content and never changes during the run.
    let basin_texture = NeutralTextureV1::new(
        texture_schema.clone(),
        BASIN_TEXTURE_ASSET_ID,
        1,
        NeutralTextureDimensionV1::D2,
        [1, 1, 1],
        1,
        NeutralTextureColorSpaceV1::Srgb,
        NeutralTextureAlphaSemanticsV1::Opaque,
        NeutralTexelEncodingV1::Rgba8Unorm,
        vec![NeutralTextureMipLevelV1::new(
            [1, 1, 1],
            vec![168, 166, 160, 255],
        )],
    )
    .map_err(|error| error.to_string())?;
    let basin_texture_revision = basin_texture
        .asset_revision()
        .map_err(|error| error.to_string())?;
    let basin_material = NeutralMaterialV1::new(
        material_schema.clone(),
        BASIN_MATERIAL_ASSET_ID,
        1,
        [u16::MAX, u16::MAX, u16::MAX, u16::MAX],
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
                basin_texture_revision,
                0,
                UvTransformV1::identity(),
            )
            .map_err(|error| error.to_string())?,
        ],
        Vec::new(),
    )
    .map_err(|error| error.to_string())?;
    let basin_material_revision = basin_material
        .asset_revision()
        .map_err(|error| error.to_string())?;
    let basin_mesh = basin_mesh(mesh_schema.clone(), bounds)?;
    let basin_mesh_revision = basin_mesh
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
    authoring_preimage.extend_from_slice(basin_mesh_revision.record_sha256.as_bytes());
    authoring_preimage.extend_from_slice(basin_material_revision.record_sha256.as_bytes());
    authoring_preimage.extend_from_slice(basin_texture_revision.record_sha256.as_bytes());
    source.authoring_sha256 =
        domain_hash("nextengine.water-preview.authoring.v2", &authoring_preimage);
    source.render_records.extend([
        NeutralRenderRecordV1::from(mesh.clone()),
        NeutralRenderRecordV1::from(texture),
        NeutralRenderRecordV1::from(material),
        NeutralRenderRecordV1::from(basin_mesh.clone()),
        NeutralRenderRecordV1::from(basin_texture),
        NeutralRenderRecordV1::from(basin_material),
    ]);
    source.root_asset_ids.extend([
        WATER_MESH_ASSET_ID,
        WATER_TEXTURE_ASSET_ID,
        WATER_MATERIAL_ASSET_ID,
        BASIN_MESH_ASSET_ID,
        BASIN_TEXTURE_ASSET_ID,
        BASIN_MATERIAL_ASSET_ID,
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
    let basin_scene = ScenePresentationRecordV2::new(
        16,
        PresentationObjectKeyV1 {
            snapshot_epoch: epoch,
            persistent_id: PersistentId::from_bytes([0xf8; 16]),
            presentation_role: PresentationRoleV1::Environment,
            incarnation: 0,
        },
        basin_mesh_revision,
        basin_material_revision,
        0,
        basin_mesh.bounds(),
        ScenePresentationFlagsV1::NONE,
        QuantizedPresentationTransformV1::default(),
        QuantizedPresentationTransformV1::default(),
        true,
    );
    // Frame the declared envelope: look at its centre from above and behind
    // so both lanes fit the same viewport.
    let centre = [
        (bounds.min()[0] + bounds.max()[0]) / 2,
        bounds.min()[1] + (bounds.max()[1] - bounds.min()[1]) / 3,
        (bounds.min()[2] + bounds.max()[2]) / 2,
    ];
    let span = (bounds.max()[0] - bounds.min()[0]).max(bounds.max()[2] - bounds.min()[2]);
    let distance = span.saturating_mul(9).saturating_div(10).max(2_000_000);
    let focus = centre;
    let camera_result = CameraResultSampleV1 {
        pose: QuantizedPresentationTransformV1 {
            translation_micrometres: [
                centre[0],
                centre[1].saturating_add(distance.saturating_mul(17).saturating_div(40)),
                centre[2].saturating_add(distance.saturating_mul(37).saturating_div(40)),
            ],
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
            distance_micrometres: u64::try_from(distance)
                .map_err(|_| "water-preview camera distance overflow".to_owned())?,
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
        vec![scene, basin_scene],
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

    let dynamic = capacity
        .map(|(vertex_capacity, index_capacity)| {
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
            Ok::<_, String>(DynamicPreview {
                profile: DynamicSurfaceProfileV1 {
                    mesh_revision,
                    vertex_capacity,
                    index_capacity,
                    residency: if request.device_local_ring {
                        DynamicSurfaceResidencyV1::DeviceLocal
                    } else {
                        DynamicSurfaceResidencyV1::HostVisible
                    },
                },
                keyframes,
            })
        })
        .transpose()?;
    let keyframes = parsed
        .iter()
        .enumerate()
        .map(|(index, obj)| PreviewKeyframe {
            path: request
                .meshes
                .get(index)
                .cloned()
                .unwrap_or_else(|| PathBuf::from("<stream step 0>")),
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

/// Floor plus four walls of the declared envelope, faces wound so that the
/// inward normals are front faces under the B0 counter-clockwise rule; the
/// wall between the camera and the water is therefore back-face culled.
#[cfg(feature = "desktop-sdl-ash")]
fn basin_mesh(
    schema: next_contracts::project::SchemaRefV1,
    bounds: AabbI64V1,
) -> Result<NeutralMeshV1, String> {
    let [x0, y0, z0] = bounds.min();
    let [x1, y1, z1] = bounds.max();
    let unit = i16::MAX;
    // (corners in winding order, normal)
    let faces: [([[i64; 3]; 4], [i16; 3]); 5] = [
        (
            [[x0, y0, z0], [x0, y0, z1], [x1, y0, z1], [x1, y0, z0]],
            [0, unit, 0],
        ),
        (
            [[x0, y0, z0], [x0, y1, z0], [x0, y1, z1], [x0, y0, z1]],
            [unit, 0, 0],
        ),
        (
            [[x1, y0, z1], [x1, y1, z1], [x1, y1, z0], [x1, y0, z0]],
            [-unit, 0, 0],
        ),
        (
            [[x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0]],
            [0, 0, unit],
        ),
        (
            [[x1, y0, z1], [x0, y0, z1], [x0, y1, z1], [x1, y1, z1]],
            [0, 0, -unit],
        ),
    ];
    let mut positions = Vec::with_capacity(20);
    let mut normals = Vec::with_capacity(20);
    let mut uv = Vec::with_capacity(20);
    let mut indices = Vec::with_capacity(30);
    for (corners, normal) in faces {
        let base = u32::try_from(positions.len())
            .map_err(|_| "water-preview basin vertex overflow".to_owned())?;
        for (corner_index, corner) in corners.iter().enumerate() {
            positions.push(*corner);
            normals.push(normal);
            uv.push(match corner_index {
                0 => [0, 0],
                1 => [0, 65_536],
                2 => [65_536, 65_536],
                _ => [65_536, 0],
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let mesh_bounds = AabbI64V1::new(
        [x0, y0, z0],
        [
            x1.checked_add(1)
                .ok_or_else(|| "water-preview basin bounds overflow".to_owned())?,
            y1.checked_add(1)
                .ok_or_else(|| "water-preview basin bounds overflow".to_owned())?,
            z1.checked_add(1)
                .ok_or_else(|| "water-preview basin bounds overflow".to_owned())?,
        ],
    )
    .map_err(|error| error.to_string())?;
    NeutralMeshV1::new(
        schema,
        BASIN_MESH_ASSET_ID,
        1,
        mesh_bounds,
        positions,
        Some(normals),
        None,
        vec![uv],
        indices,
        vec![
            NeutralMeshPrimitiveV1::new(MeshPrimitiveTopologyV1::Triangles, 0, 30, 0)
                .map_err(|error| error.to_string())?,
        ],
    )
    .map_err(|error| error.to_string())
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
        let request = WaterPreviewRequest {
            meshes: vec![path.clone()],
            frames: 1,
            extent: [640, 480],
            hold: 1,
            stream: None,
            device_local_ring: true,
            until_close: false,
            capture: None,
        };
        let source = PreviewSource::Keyframes(
            request
                .meshes
                .iter()
                .map(|path| parse_obj(path))
                .collect::<Result<Vec<_>, _>>()
                .expect("parses keyframe"),
        );
        let preview = build_preview(&request, source).expect("builds B0 preview");
        std::fs::remove_file(path).expect("removes bounded OBJ fixture");
        assert_eq!(preview.visible_object_count, 2);
        assert_eq!(preview.indexed_draw_count, 2);
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
        let request = WaterPreviewRequest {
            meshes: vec![first.clone(), second.clone()],
            frames: 1,
            extent: [640, 480],
            hold: 2,
            stream: None,
            device_local_ring: true,
            until_close: false,
            capture: None,
        };
        let source = PreviewSource::Keyframes(
            request
                .meshes
                .iter()
                .map(|path| parse_obj(path))
                .collect::<Result<Vec<_>, _>>()
                .expect("parses keyframes"),
        );
        let preview = build_preview(&request, source).expect("builds dynamic preview");
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
        assert_eq!(preview.visible_object_count, 2);
        assert_eq!(preview.indexed_draw_count, 2);
    }
}
