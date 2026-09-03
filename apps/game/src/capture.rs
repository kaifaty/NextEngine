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
}

impl CaptureRequest {
    pub(crate) fn adapter_request(&self) -> next_desktop_sdl_ash::DesktopFrameCaptureRequestV1 {
        next_desktop_sdl_ash::DesktopFrameCaptureRequestV1 {
            rendered_frame_index: self.rendered_frame_index,
            frame_count: 1,
        }
    }

    /// Writes the captured frame opaque (the particle pass leaves alpha `0`
    /// on fluid pixels as its coverage channel).
    pub(crate) fn write(
        &self,
        frames: &[next_desktop_sdl_ash::DesktopCapturedFrameV1],
    ) -> Result<(), AppFailure> {
        let frame = frames
            .iter()
            .find(|frame| frame.rendered_frame_index == self.rendered_frame_index)
            .ok_or_else(|| {
                AppFailure::cli(
                    "GAME_CAPTURE_FRAME_MISSING",
                    format!(
                        "the adapter did not capture rendered frame {}",
                        self.rendered_frame_index
                    ),
                )
            })?;
        let mut opaque = frame.rgba8.clone();
        for pixel in opaque.chunks_exact_mut(4) {
            pixel[3] = u8::MAX;
        }
        let png = encode_png_rgba8(frame.extent, &opaque)
            .map_err(|message| AppFailure::cli("GAME_CAPTURE_ENCODE_FAILED", message))?;
        write_file(&self.png, &png)
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
