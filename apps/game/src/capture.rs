//! Diagnostic frame capture for the plan `continuum-water/09` look gate:
//! one rendered frame copied back by the desktop adapter and written as a
//! PNG. Evidence only; nothing here feeds the session or its roots.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::cli::AppFailure;

/// The capture request of one interactive run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaptureRequest {
    pub(crate) rendered_frame_index: u64,
    pub(crate) png: PathBuf,
    /// Plan `continuum-water/18`: which image to read.
    pub(crate) source: next_desktop_sdl_ash::DesktopCaptureSourceV1,
    /// Plan 18: consecutive frames from `rendered_frame_index` (the adapter
    /// clamps the burst); frames after the first are written with a
    /// `-<index>` suffix before the extension.
    pub(crate) frame_count: u32,
}

/// Parses `--capture-buffer` values.
pub(crate) fn parse_capture_source(
    value: &str,
) -> Option<next_desktop_sdl_ash::DesktopCaptureSourceV1> {
    use next_desktop_sdl_ash::DesktopCaptureSourceV1 as Source;
    Some(match value {
        "color" => Source::Color,
        "scene" => Source::Scene,
        "albedo" => Source::AlbedoMask,
        "normal" => Source::NormalRoughness,
        "motion" => Source::Motion,
        "depth" => Source::LinearDepth,
        _ => return None,
    })
}

impl CaptureRequest {
    pub(crate) fn adapter_request(&self) -> next_desktop_sdl_ash::DesktopFrameCaptureRequestV1 {
        next_desktop_sdl_ash::DesktopFrameCaptureRequestV1 {
            rendered_frame_index: self.rendered_frame_index,
            frame_count: self.frame_count,
            source: self.source,
        }
    }

    /// Writes the captured frame; colour captures are written opaque (the
    /// particle pass leaves alpha `0` on fluid pixels as its coverage
    /// channel), G-buffer captures keep their raw bytes (the albedo alpha is
    /// the group mask, `motion` and `depth` are raw floats).
    pub(crate) fn write(
        &self,
        frames: &[next_desktop_sdl_ash::DesktopCapturedFrameV1],
    ) -> Result<(), AppFailure> {
        let count = u64::from(self.frame_count.max(1));
        for offset in 0..count {
            let index = self.rendered_frame_index + offset;
            let path = if offset == 0 {
                self.png.clone()
            } else {
                let stem = self
                    .png
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.png.with_file_name(format!("{stem}-{index}.png"))
            };
            self.write_frame(frames, index, &path)?;
        }
        Ok(())
    }

    fn write_frame(
        &self,
        frames: &[next_desktop_sdl_ash::DesktopCapturedFrameV1],
        rendered_frame_index: u64,
        path: &Path,
    ) -> Result<(), AppFailure> {
        let frame = frames
            .iter()
            .find(|frame| frame.rendered_frame_index == rendered_frame_index)
            .ok_or_else(|| {
                AppFailure::cli(
                    "GAME_CAPTURE_FRAME_MISSING",
                    format!("the adapter did not capture rendered frame {rendered_frame_index}"),
                )
            })?;
        let mut opaque = frame.rgba8.clone();
        if matches!(
            self.source,
            next_desktop_sdl_ash::DesktopCaptureSourceV1::Color
                | next_desktop_sdl_ash::DesktopCaptureSourceV1::Scene
        ) {
            for pixel in opaque.chunks_exact_mut(4) {
                pixel[3] = u8::MAX;
            }
        }
        let png = encode_png_rgba8(frame.extent, &opaque)
            .map_err(|message| AppFailure::cli("GAME_CAPTURE_ENCODE_FAILED", message))?;
        write_file(path, &png)
    }
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), AppFailure> {
    std::fs::write(path, bytes).map_err(|error| {
        AppFailure::cli(
            "GAME_CAPTURE_WRITE_FAILED",
            format!("{}: {error}", path.display()),
        )
    })
}

/// Minimal PNG encoder (8-bit RGBA, filter type zero, one zlib stream); it
/// adds no image dependency to the workspace.
fn encode_png_rgba8(extent: [u32; 2], rgba8: &[u8]) -> Result<Vec<u8>, String> {
    let [width, height] = extent;
    let row_bytes = usize::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or_else(|| "capture width overflow".to_owned())?;
    let expected = usize::try_from(height)
        .ok()
        .and_then(|height| height.checked_mul(row_bytes))
        .ok_or_else(|| "capture height overflow".to_owned())?;
    if rgba8.len() != expected || width == 0 || height == 0 {
        return Err("capture payload does not match its extent".to_owned());
    }
    let mut filtered = Vec::with_capacity(expected + height as usize);
    for row in rgba8.chunks_exact(row_bytes) {
        filtered.push(0);
        filtered.extend_from_slice(row);
    }
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    let compressed = encoder
        .write_all(&filtered)
        .and_then(|()| encoder.finish())
        .map_err(|error| format!("PNG compression failed: {error}"))?;
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
        let length = u32::try_from(payload.len()).map_err(|_| "PNG chunk overflow".to_owned())?;
        png.extend_from_slice(&length.to_be_bytes());
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(kind);
        hasher.update(payload);
        png.extend_from_slice(kind);
        png.extend_from_slice(payload);
        png.extend_from_slice(&hasher.finalize().to_be_bytes());
    }
    Ok(png)
}
