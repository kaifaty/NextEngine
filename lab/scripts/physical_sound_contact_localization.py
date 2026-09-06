"""Report-only motion-region discriminator, not a calibrated contact detector.

Use only existing, selected TRAIN-source records and preserve experiment roles.
No audio enters localization. Published videos carry original recorded audio.
"""

import argparse
import io
import json
import subprocess
import tarfile
from pathlib import Path

import numpy as np
import physical_sound_syncfusion_pilot as pilot
import soundfile as sf
from PIL import Image
from scipy.ndimage import uniform_filter


def gray(frame):
    value = np.asarray(frame, dtype=np.float64)
    if value.shape != (240, 320, 3) or not np.isfinite(value).all():
        raise ValueError("Expected finite author 320x240 RGB frame")
    return value.mean(-1)


def translation(reference, other):
    """Integer phase correlation: shift OTHER onto REFERENCE, no rescaling."""
    product = np.fft.fft2(reference) * np.fft.fft2(other).conj()
    product /= np.maximum(np.abs(product), 1e-12)
    peak = np.unravel_index(np.argmax(np.fft.ifft2(product).real), reference.shape)
    return tuple(
        int(p if p <= n // 2 else p - n) for p, n in zip(peak, reference.shape)
    )


def motion_region(frames):
    if len(frames) != 7:
        raise ValueError("Expected seven frames centered on nominal onset")
    values = [gray(f) for f in frames]
    anchor = values[3]
    differences, shifts = [], []
    valid = np.ones(anchor.shape, dtype=bool)
    for value in values:
        dy, dx = translation(anchor, value)
        if abs(dy) > 24 or abs(dx) > 32:
            return {"status": "abstain", "reason": "large global translation"}, None
        aligned = np.roll(value, (dy, dx), axis=(0, 1))
        mask = np.ones(anchor.shape, dtype=bool)
        if dy:
            if dy > 0:
                mask[:dy] = False
            else:
                mask[dy:] = False
        if dx:
            if dx > 0:
                mask[:, :dx] = False
            else:
                mask[:, dx:] = False
        valid &= mask
        differences.append(np.abs(aligned - anchor))
        shifts.append([dy, dx])
    # Fixed quarter-frame window; no crop-size/threshold sweep on real examples.
    energy = np.mean(differences, axis=0) * valid
    if energy.max() < 1e-6:
        return {
            "status": "abstain",
            "reason": "no residual motion",
            "shifts": shifts,
        }, energy
    scores = uniform_filter(energy, size=(60, 80), mode="constant")
    # Only complete windows; deterministic first argmax.
    y, x = np.unravel_index(np.argmax(scores[30:211, 40:281]), (181, 241))
    box = [int(x), int(y), 80, 60]
    fraction = float(energy[y : y + 60, x : x + 80].sum() / energy.sum())
    return {
        "status": "candidate_not_contact_label",
        "box_xywh": box,
        "shifts_dy_dx": shifts,
        "motion_mass_fraction": fraction,
        "claim": "highest residual-motion window; can be hand, stick, shadow or camera artifact",
    }, energy


def run(data, frames, fitted, archive_path, output):
    original = json.loads((data / "data.json").read_text())
    fit = json.loads((fitted / "fit.json").read_text())
    if (
        original["status"] != "complete"
        or pilot.sha(data / "data.json") != fit["data_sha256"]
        or pilot.sha(archive_path) != original["archive_sha256"]
        or pilot.sha(frames / "frames.json") != fit["frames_sha256"]
    ):
        raise ValueError("Changed source identity")
    output.mkdir(parents=True, exist_ok=False)
    result = {
        "status": "running",
        "data_sha256": pilot.sha(data / "data.json"),
        "archive_sha256": original["archive_sha256"],
        "localizer_audio_input": False,
        "soundtrack": "ORIGINAL RECORDING, not generated; shared playback gain .5",
        "new_training_updates": 0,
        "cases": [],
    }
    pilot.save(output / "result.json", result)
    with tarfile.open(archive_path, "r:") as archive:
        members = {m.name: m for m in archive if m.isfile()}

        def read(name, limit):
            member = members[name]
            if member.size > limit:
                raise ValueError("Oversized source member")
            return archive.extractfile(member).read()

        for case in fit["cases"]:
            index = case["row"]
            row = original["rows"][index]
            if row["role"] == "train" or not row["recording"].startswith("train/"):
                raise ValueError(
                    "Expected already opened development role in author TRAIN"
                )
            key = row["recording"].removesuffix(".times.csv")
            metadata = json.loads(read(key + ".metadata.json", 10000))
            if metadata["processed"]["video_frame_rate"] != 15:
                raise ValueError("Unexpected source fps")
            center = int(row["start"] * 15)  # zero-based nominal onset frame
            clip_start = center - 4
            if clip_start < 0:
                raise ValueError("Insufficient pre-onset context")
            name = f"{row['material']}-{row['motion']}"
            folder = output / name
            folder.mkdir()
            images, sources = [], []
            for offset in range(12):
                member = key + f".frame_{clip_start + offset + 1:06d}.jpg"
                path = folder / f"frame-{offset:02d}.jpg"
                path.write_bytes(read(member, 1024**2))
                with Image.open(path) as image:
                    images.append(np.array(image.convert("RGB")))
                sources.append({"member": member, "sha256": pilot.sha(path)})
            localized, _ = motion_region(images[1:8])
            # Audio is loaded only after the image-only decision has been made.
            raw = read(key + ".resampled.wav", 100 * 1024**2)
            with sf.SoundFile(io.BytesIO(raw)) as source:
                if source.samplerate != pilot.RATE or source.channels != 1:
                    raise ValueError("Unexpected author audio layout")
                source.seek(clip_start * 3200)
                sound = source.read(12 * 3200, dtype="float32")
            if (
                len(sound) != 38400
                or not np.isfinite(sound).all()
                or np.abs(sound).max() * 0.5 >= 0.98
            ):
                raise ValueError("Reference soundtrack length/finite/headroom")
            sf.write(folder / "recorded.wav", sound * 0.5, pilot.RATE, subtype="PCM_16")
            filters = []
            if "box_xywh" in localized:
                x, y, w, h = localized["box_xywh"]
                filters.append(f"drawbox=x={x}:y={y}:w={w}:h={h}:color=yellow:t=2")
            filters += [
                "scale=640:480:flags=neighbor",
                "drawtext=text='RECORDED AUDIO - motion candidate, not contact':fontcolor=white:fontsize=15:box=1:boxcolor=black@0.7:x=8:y=8",
                "tpad=stop_mode=clone:stop_duration=0.4",
            ]
            clip = folder / "diagnostic.mp4"
            subprocess.run(
                [
                    "ffmpeg",
                    "-v",
                    "error",
                    "-n",
                    "-framerate",
                    "15",
                    "-i",
                    str(folder / "frame-%02d.jpg"),
                    "-i",
                    str(folder / "recorded.wav"),
                    "-vf",
                    ",".join(filters),
                    "-af",
                    "apad=pad_dur=0.4",
                    "-t",
                    "1.2",
                    "-c:v",
                    "libx264",
                    "-pix_fmt",
                    "yuv420p",
                    "-c:a",
                    "aac",
                    str(clip),
                ],
                check=True,
            )
            repeated = output / f"{name}-recorded-diagnostic.mp4"
            subprocess.run(
                [
                    "ffmpeg",
                    "-v",
                    "error",
                    "-n",
                    "-stream_loop",
                    "2",
                    "-i",
                    str(clip),
                    "-c",
                    "copy",
                    str(repeated),
                ],
                check=True,
            )
            entry = {
                "row": index,
                "recording": row["recording"],
                "role": row["role"],
                "material": row["material"],
                "motion": row["motion"],
                "nominal_onset": row["start"],
                "clip_start": clip_start / 15,
                "localization": localized,
                "frames": sources,
                "video": str(repeated),
                "sha256": pilot.sha(repeated),
            }
            result["cases"].append(entry)
            pilot.save(output / "result.json", result)
            print(json.dumps(entry), flush=True)
    result["status"] = "complete_diagnostic_not_admission"
    pilot.save(output / "result.json", result)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for argument in ("data", "frames", "fitted", "archive", "output"):
        parser.add_argument("--" + argument, type=Path, required=True)
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        parser.error("Artifacts must remain external")
    run(args.data, args.frames, args.fitted, args.archive, args.output)
