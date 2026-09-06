"""One contact-attention residual shared by six objects; four-object local DEV.

All objects were published pretraining TRAIN. This is report-only fine-tuning
transfer, not pristine unseen-object evidence. Fit alone reads TRAIN recordings;
render loads geometry caches and weights only. No per-object parameters or gain.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_cohort as cohort
import physical_sound_sonicgauss_condition_probe as condition
import physical_sound_sonicgauss_pilot as pilot
import torch
from safetensors.torch import load_file, save_file
from scipy.io import wavfile
from scipy.signal import resample_poly
from torch import nn
from torch.nn import functional as F


class ContactResidual(nn.Module):
    def __init__(self, width=1024, rank=64):
        super().__init__()
        self.query = nn.Linear(width, rank, bias=False)
        self.key = nn.Linear(width, rank, bias=False)
        self.value = nn.Linear(width, rank, bias=False)
        self.output = nn.Linear(rank, width, bias=False)
        nn.init.zeros_(self.output.weight)

    def forward(self, gaussian, position):
        gs = F.layer_norm(gaussian, (gaussian.shape[-1],))
        pos = F.layer_norm(position, (position.shape[-1],))
        query = self.query(pos)[:, None]
        weights = (
            query @ self.key(gs).transpose(1, 2) / query.shape[-1] ** 0.5
        ).softmax(-1)
        return self.output(weights @ self.value(gs))


class AdaptedFusion(nn.Module):
    def __init__(self, original, adapter):
        super().__init__()
        self.original, self.adapter = original, adapter

    def forward(self, gaussian, position):
        base = self.original(gaussian, position)
        count = gaussian.shape[1]
        return torch.cat(
            [base[:, :count], base[:, count:] + self.adapter(gaussian, position)], dim=1
        )


def validate_inputs(manifest):
    rows = manifest["objects"]
    if [r["object_id"] for r in rows] != list(cohort.OBJECTS):
        raise ValueError("unexpected roster")
    for row in rows:
        expected = "train" if row["object_id"] in cohort.FIT_TRAIN else "development"
        if (
            row["local_fit_role"] != expected
            or row["source_role"] != "disclosed_generator_train"
            or row["pretrained_split"] != "author_train"
            or row["material_label"] != cohort.OBJECTS[row["object_id"]]
            or [c["index"] for c in row["contacts"]] != list(range(6))
        ):
            raise ValueError("role/material/contact mismatch")
    return rows


def caches(data, baseline):
    manifest = json.loads((data / "inputs.json").read_text())
    rows = validate_inputs(manifest)
    previous = json.loads((baseline / "result.json").read_text())
    if previous["input_manifest_sha256"] != pilot.sha256(data / "inputs.json"):
        raise ValueError("baseline binding changed")
    result, hashes = {}, {}
    for row in rows:
        oid = row["object_id"]
        path = baseline / f"object-{oid:02d}-geometry.npz"
        hashes[path.name] = pilot.sha256(path)
        with np.load(path, allow_pickle=False) as saved:
            values = {k: torch.from_numpy(saved[k].copy()) for k in saved.files}
        if (
            values["features"].shape != (1, 64, 1024)
            or values["contacts"].shape != (6, 3)
            or not all(torch.isfinite(v).all() for v in values.values())
        ):
            raise ValueError("invalid bounded geometry cache")
        result[oid] = values
    return rows, result, hashes


def teacher_audio(path, sha):
    if pilot.sha256(path) != sha:
        raise ValueError("teacher hash mismatch")
    rate, wave = wavfile.read(path)
    if wave.dtype == np.int16:
        wave = wave.astype(np.float32) / 32768
    elif wave.dtype != np.float32:
        raise ValueError("unsupported teacher format")
    if wave.ndim == 1:
        wave = np.stack([wave, wave], axis=1)
    if (
        wave.ndim != 2
        or wave.shape[1] != 2
        or not np.isfinite(wave).all()
        or not 8000 <= rate <= 96000
        or not 0 < len(wave) <= 4 * rate
    ):
        raise ValueError("invalid teacher waveform")
    # Match author 2.98 s training window, preserve channels and absolute level.
    wave = wave[: int(rate * 2.98)]
    divisor = np.gcd(rate, 44100)
    wave = resample_poly(wave, 44100 // divisor, rate // divisor, axis=0)
    length = int(44100 * 2.98)
    return np.pad(wave[:length], ((0, max(0, length - len(wave))), (0, 0))).T.astype(
        np.float32
    )


def flow_loss(model, latent, fused):
    batch, length, _ = latent.shape
    duration = model.duration_emebdder(torch.full((batch,), 2.98, device=latent.device))
    pooled = model.fc(fused.mean(1))
    states = torch.cat([fused, duration], dim=1)
    # Author logit-normal sampling, explicit discrete scheduler indices.
    indices = (torch.randn(batch).sigmoid() * 1000).long().clamp(max=999)
    sigma = model.noise_scheduler.sigmas[indices].to(latent.device)
    time = model.noise_scheduler.timesteps[indices].to(latent.device) / 1000
    noise = torch.randn_like(latent)
    noisy = (1 - sigma[:, None, None]) * latent + sigma[:, None, None] * noise
    predicted = model.transformer(
        hidden_states=noisy,
        timestep=time,
        guidance=None,
        pooled_projections=pooled,
        encoder_hidden_states=states,
        txt_ids=torch.zeros(batch, states.shape[1], 3, device=latent.device),
        img_ids=torch.arange(length, device=latent.device)[None, :, None].repeat(
            batch, 1, 3
        ),
        return_dict=False,
    )[0]
    return F.mse_loss(predicted.float(), noise - latent)


def fit(args, models):
    model, _, position, fusion, vae, _ = models
    rows, geometry, hashes = caches(args.data, args.baseline)
    corpus = json.loads((args.data / "corpus.json").read_text())
    if validate_inputs(corpus) != rows:
        # Corpus contacts additionally carry reference provenance: compare only inputs.
        for full, thin in zip(corpus["objects"], rows, strict=True):
            if [
                {k: c[k] for k in ("index", "contact")} for c in full["contacts"]
            ] != thin["contacts"]:
                raise ValueError("corpus contacts differ from render input")
    args.output.mkdir(parents=True)
    teachers, features, positions, admitted = [], [], [], []
    with torch.no_grad():
        for row in corpus["objects"]:
            oid = row["object_id"]
            if oid not in cohort.FIT_TRAIN:
                continue  # DEV waveforms are never opened by fit.
            for contact in row["contacts"]:
                ref = contact["reference"]
                audio = teacher_audio(Path(ref["path"]), ref["sha256"])
                encoded = (
                    vae.encode(torch.from_numpy(audio)[None].cuda())
                    .latent_dist.mode()
                    .transpose(1, 2)
                )
                if encoded.shape != (1, 64, 64) or not torch.isfinite(encoded).all():
                    raise ValueError("unexpected teacher latent")
                teachers.append(encoded)
                features.append(geometry[oid]["features"].cuda())
                positions.append(
                    position(geometry[oid]["contacts"][contact["index"]].cuda())[None]
                )
                admitted.append(
                    {
                        "object_id": oid,
                        "contact_index": contact["index"],
                        "reference_sha256": ref["sha256"],
                    }
                )
            print("encoded TRAIN", oid, flush=True)
    latent, gs, pos = [torch.cat(x) for x in (teachers, features, positions)]
    torch.manual_seed(42)
    adapter = ContactResidual().cuda()
    adapted = AdaptedFusion(fusion, adapter)
    with torch.no_grad():
        if not torch.equal(adapted(gs, pos), fusion(gs, pos)):
            raise ValueError("zero-init changed baseline")
    versions = [(p, p._version) for module in models[:5] for p in module.parameters()]
    optimizer = torch.optim.Adam(adapter.parameters(), lr=1e-4)
    losses = []
    for step in range(120):
        indices = torch.randperm(len(latent))[:4].cuda()
        optimizer.zero_grad(set_to_none=True)
        loss = flow_loss(model, latent[indices], adapted(gs[indices], pos[indices]))
        if not torch.isfinite(loss):
            raise ValueError("nonfinite fit")
        loss.backward()
        norm = torch.nn.utils.clip_grad_norm_(
            adapter.parameters(), 1.0, error_if_nonfinite=True
        )
        optimizer.step()
        losses.append(
            {"step": step + 1, "loss": loss.item(), "gradient_norm": norm.item()}
        )
        if (step + 1) % 10 == 0:
            print("fit", losses[-1], flush=True)
    if any(
        p._version != version or p.grad is not None or p.requires_grad
        for p, version in versions
    ):
        raise ValueError("frozen model changed or received gradients")
    save_file(
        {k: v.detach().cpu().contiguous() for k, v in adapter.state_dict().items()},
        args.output / "adapter.safetensors",
    )
    cohort.save(
        args.output / "fit.json",
        {
            "status": "REPORT_ONLY_SHARED_CONTACT_FIT",
            "steps": 120,
            "batch_size": 4,
            "seed": 42,
            "learning_rate": 1e-4,
            "parameters": sum(p.numel() for p in adapter.parameters()),
            "teacher": "2.98s stereo/no level normalization; SciPy resampling; VAE posterior mode (author samples posterior)",
            "losses": losses,
            "train_rows": admitted,
            "geometry_hashes": hashes,
            "input_sha256": pilot.sha256(args.data / "inputs.json"),
            "weights": pilot.WEIGHTS,
            "adapter_sha256": pilot.sha256(args.output / "adapter.safetensors"),
            "script_sha256": pilot.sha256(Path(__file__)),
            "frozen_parameter_versions_unchanged": True,
            "scope": "one final checkpoint, no DEV selection; all objects known to pretraining",
        },
    )


def render(args, ns, models):
    rows, geometry, hashes = caches(args.data, args.baseline)
    receipt = json.loads((args.fit / "fit.json").read_text())
    if (
        receipt["geometry_hashes"] != hashes
        or receipt["input_sha256"] != pilot.sha256(args.data / "inputs.json")
        or receipt["adapter_sha256"] != pilot.sha256(args.fit / "adapter.safetensors")
    ):
        raise ValueError("fit/input binding mismatch")
    adapter = ContactResidual().cuda()
    adapter.load_state_dict(load_file(args.fit / "adapter.safetensors"), strict=True)
    adapter.eval().requires_grad_(False)
    adapted = AdaptedFusion(models[3], adapter)
    args.output.mkdir(parents=True)
    records, waves = [], []
    for row in rows:
        oid = row["object_id"]
        cache = geometry[oid]
        ge = condition.CachedGaussian(
            cache["features"].cuda(), (cache["cpu_rng"], cache["cuda_rng"])
        )
        for index in range(6):
            pair = []
            for label, fusion in [("baseline", models[3]), ("candidate", adapted)]:
                result = condition.render(
                    ns,
                    models,
                    ge,
                    condition.FusionProbe(fusion),
                    {},
                    cache["contacts"][index].cuda(),
                )
                if not all(np.isfinite(v).all() for v in result.values()):
                    raise ValueError("nonfinite generation")
                pair.append(result)
                name = f"object-{oid:02d}-contact-{index}-{label}"
                np.savez(
                    args.output / f"{name}.npz",
                    latent=result["latent"],
                    wave=result["wave"],
                )
                waves.append(result["wave"])
                records.append(
                    {
                        "object_id": oid,
                        "contact_index": index,
                        "variant": label,
                        "name": name,
                        "local_fit_role": row["local_fit_role"],
                    }
                )
            if not np.array_equal(pair[0]["initial_noise"], pair[1]["initial_noise"]):
                raise ValueError("paired diffusion noise differs")
            if index < 2:
                with np.load(
                    args.baseline / f"object-{oid:02d}-contact-{index}.npz",
                    allow_pickle=False,
                ) as old:
                    if not np.array_equal(old["wave"], pair[0]["wave"]):
                        raise ValueError("baseline cached replay differs; no retry")
            print("rendered", oid, index, flush=True)
    gain = min(1.0, 0.98 / max(float(abs(w).max()) for w in waves))
    for row, wave in zip(records, waves, strict=True):
        path = args.output / f"{row['name']}.wav"
        wavfile.write(path, 44100, (wave.T * gain).astype(np.float32))
        row.update(wav=str(path), wav_sha256=pilot.sha256(path))
    cohort.save(
        args.output / "result.json",
        {
            "rows": records,
            "shared_gain": gain,
            "fit_sha256": pilot.sha256(args.fit / "fit.json"),
            "input_sha256": receipt["input_sha256"],
            "target_audio_read": False,
            "baseline_cached_replay_exact": True,
        },
    )


def nonregression_decision(rows):
    """Reject observed DEV regressions; a pass would not establish realism."""
    dev = [r for r in rows if r["local_fit_role"] == "development"]
    expected = {
        (o, c) for o in set(cohort.OBJECTS) - cohort.FIT_TRAIN for c in range(6)
    }
    if (
        len(dev) != len(expected)
        or {(r["object_id"], r["contact_index"]) for r in dev} != expected
    ):
        raise ValueError("incomplete development evidence")
    if not all(
        np.isfinite(r[k]) and r[k] >= 0 for r in dev for k in ("baseline", "candidate")
    ):
        raise ValueError("invalid development metric")
    regressed = []
    for oid in sorted(set(cohort.OBJECTS) - cohort.FIT_TRAIN):
        subset = [r for r in dev if r["object_id"] == oid]
        if np.mean([r["candidate"] for r in subset]) > np.mean(
            [r["baseline"] for r in subset]
        ):
            regressed.append(oid)
    improved = np.mean([r["candidate"] for r in dev]) < np.mean(
        [r["baseline"] for r in dev]
    )
    return {
        "decision": "REJECT"
        if regressed or not improved
        else "REPORT_ONLY_NONREGRESSION_PASS",
        "regressed_development_objects": regressed,
        "physical_quality_validated": False,
        "rule": "DEV mean must improve with no per-object mean regression; no physical acceptance or runtime promotion",
    }


def assess(args):
    corpus = json.loads((args.data / "corpus.json").read_text())
    validate_inputs(corpus)
    generated = json.loads((args.generated / "result.json").read_text())
    if generated["input_sha256"] != pilot.sha256(args.data / "inputs.json"):
        raise ValueError("assessment input binding mismatch")
    by_key = {
        (r["object_id"], r["contact_index"], r["variant"]): r for r in generated["rows"]
    }
    if len(by_key) != 120:
        raise ValueError("incomplete paired cohort")
    args.output.mkdir(parents=True)
    results, auditions = [], []
    for row in corpus["objects"]:
        oid = row["object_id"]
        references = [
            cohort.audio(Path(c["reference"]["path"]), c["reference"]["sha256"])
            for c in row["contacts"]
        ]
        for index, reference in enumerate(references):
            item = {
                "object_id": oid,
                "contact_index": index,
                "local_fit_role": row["local_fit_role"],
            }
            parts = [reference]
            for kind in ("baseline", "candidate"):
                record = by_key[(oid, index, kind)]
                wave = (
                    cohort.audio(Path(record["wav"]), record["wav_sha256"])
                    / generated["shared_gain"]
                )
                item[kind] = cohort.magnitude_distance(reference, wave)
                item[kind + "_swapped"] = cohort.magnitude_distance(
                    references[(index + 1) % 6], wave
                )
                item[kind + "_rms"] = float(np.sqrt(np.mean(wave.astype(float) ** 2)))
                parts.append(wave)
            results.append(item)
            auditions.append(
                (f"object-{oid:02d}-contact-{index}-comparison.wav", parts)
            )
    gain = min(
        1.0, 0.98 / max(float(abs(w).max()) for _, parts in auditions for w in parts)
    )
    for name, parts in auditions:
        wavfile.write(
            args.output / name,
            44100,
            np.concatenate(
                [v for w in parts for v in [w * gain, np.zeros(22050)]]
            ).astype(np.float32),
        )
    summary = {}
    for role in ("train", "development"):
        subset = [r for r in results if r["local_fit_role"] == role]
        summary[role] = {
            "count": len(subset),
            "wins": sum(r["candidate"] < r["baseline"] for r in subset),
            **{
                kind: float(np.mean([r[kind] for r in subset]))
                for kind in ("baseline", "candidate")
            },
            **{
                kind + "_matched_beats_swapped": sum(
                    r[kind] < r[kind + "_swapped"] for r in subset
                )
                for kind in ("baseline", "candidate")
            },
        }
    cohort.save(
        args.output / "assessment.json",
        {
            "summary": summary,
            "decision": nonregression_decision(results),
            "rows": results,
            "shared_gain": gain,
            "metric": "full-waveform relative MRSTFT magnitude L1; not perceptual/physical acceptance",
            "claim": "fine-tuning transfer only; known pretraining TRAIN, no human approval required",
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


def diagnose(args, models):
    """Paired fixed-noise flow objective; no optimizer/checkpoint selection."""
    model, _, position, original, vae, _ = models
    _, geometry, hashes = caches(args.data, args.baseline)
    receipt = json.loads((args.fit / "fit.json").read_text())
    if (
        receipt["geometry_hashes"] != hashes
        or receipt["input_sha256"] != pilot.sha256(args.data / "inputs.json")
        or receipt["adapter_sha256"] != pilot.sha256(args.fit / "adapter.safetensors")
    ):
        raise ValueError("diagnostic fit binding mismatch")
    adapter = ContactResidual().cuda()
    adapter.load_state_dict(load_file(args.fit / "adapter.safetensors"), strict=True)
    adapter.eval().requires_grad_(False)
    candidate = AdaptedFusion(original, adapter)
    corpus = json.loads((args.data / "corpus.json").read_text())
    validate_inputs(corpus)
    args.output.mkdir(parents=True)
    rows = []
    with torch.no_grad():
        for obj in corpus["objects"]:
            oid = obj["object_id"]
            gs = geometry[oid]["features"].cuda()
            for contact in obj["contacts"]:
                index = contact["index"]
                ref = contact["reference"]
                audio = teacher_audio(Path(ref["path"]), ref["sha256"])
                latent = (
                    vae.encode(torch.from_numpy(audio)[None].cuda())
                    .latent_dist.mode()
                    .transpose(1, 2)
                )
                pos = position(geometry[oid]["contacts"][index].cuda())[None]
                base, changed = original(gs, pos), candidate(gs, pos)
                record = {
                    "object_id": oid,
                    "contact_index": index,
                    "local_fit_role": obj["local_fit_role"],
                    "relative_condition_change": float(
                        (changed - base).norm() / base.norm()
                    ),
                }
                for label, fused in (("baseline", base), ("candidate", changed)):
                    losses = []
                    for seed in (101, 102, 103, 104):
                        torch.manual_seed(seed + index * 1000 + oid * 10000)
                        losses.append(float(flow_loss(model, latent, fused)))
                    record[label] = float(np.mean(losses))
                rows.append(record)
            print("diagnosed", oid, flush=True)
    summary = {}
    for role in ("train", "development"):
        selected = [r for r in rows if r["local_fit_role"] == role]
        summary[role] = {
            "count": len(selected),
            "wins": sum(r["candidate"] < r["baseline"] for r in selected),
            **{
                k: float(np.mean([r[k] for r in selected]))
                for k in ("baseline", "candidate")
            },
        }
    cohort.save(
        args.output / "diagnostic.json",
        {
            "rows": rows,
            "summary": summary,
            "noise_draws_per_contact": 4,
            "paired_randomness": True,
            "training_performed": False,
            "scope": "post-fit diagnostic, not checkpoint selection or physical acceptance",
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=["fit", "render", "assess", "diagnose"])
    for name in ("source", "assets", "data", "baseline", "fit", "generated", "output"):
        parser.add_argument("--" + name, type=Path, required=name in ("data", "output"))
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    required = {
        "fit": ("source", "assets", "baseline"),
        "render": ("source", "assets", "baseline", "fit"),
        "assess": ("generated",),
        "diagnose": ("source", "assets", "baseline", "fit"),
    }[args.stage]
    if any(getattr(args, name) is None for name in required):
        parser.error("missing stage arguments: " + ", ".join(required))
    torch.set_num_threads(4)
    if args.stage == "assess":
        assess(args)
        return
    torch.manual_seed(0)
    ns = pilot.load_definitions(args.source)
    models = pilot.build_models(args.source, args.assets, ns)
    if args.stage == "fit":
        fit(args, models)
    elif args.stage == "diagnose":
        diagnose(args, models)
    else:
        render(args, ns, models)


if __name__ == "__main__":
    main()
