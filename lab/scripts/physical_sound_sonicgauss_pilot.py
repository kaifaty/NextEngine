"""Report-only SonicGauss contact pair, from 3DGS + coordinates, not audio.

Execute only reviewed definitions from pinned external sources. Keep the trained
1024-token PTv3 patches and half-precision QKV; use PyTorch SDPA for the optional
FlashAttention kernel and segment_reduce for torch_scatter's CSR reductions.
These are explicit numerical compatibility substitutions, not bit-exact claims.
No upstream package initializers, tokenizer downloads, pickle or training code.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import importlib.util
import itertools
import json
import math
import sys
import time
from collections import OrderedDict
from functools import partial
from pathlib import Path
from types import ModuleType, SimpleNamespace
from typing import Optional

import numpy as np
import torch
from torch import nn

POINT = "SplatFormer/Pointcept/pointcept/models/"
SOURCES = {
    "stage2/common.py": "d57fc0e5b6bbe54ea4fe7b1d2d9ec409ea60d46c9dfad83cc2518bb110c32650",
    "stage3/common.py": "405379251c0c824f3238eef121861a79597111d449060746161082266cd48179",
    "stage3/infer_3.py": "9cccda8ad6940bc1e4b02491302dd1674191c70ba05855f738d8a2cd154df88b",
    "configs/stage3.yaml": "ea9f5ca57935b5f0abd5a2365878feef9613159e80b68a1739e91e22290b4b96",
    "TangoFlux/tangoflux/model.py": "a8d2a49d16d831e954f8db33cb50807fb1b55eec79793b4799dd7c8498b339e6",
    "SplatFormer/models/feature_predictor.py": "2830f1cd46097907c2b7c5d2fcad47b47bb6927b0fb86db9e49b0f647caa5fd0",
    "SplatFormer/models/pointtransformer_v3.py": "85412febaaa6d0a75314397b32388475f7a4181c457f137ac6576d199678285c",
    "SplatFormer/configs/model/ptv3.gin": "010a8c459a204fd37e69f5693082e40e14d1c4ce1a1e87a30e44b96fadf81e62",
    "SplatFormer/utils/transform_utils.py": "ced86c22947a8050deee9b95b9c159b9bab77da7cb919212993872a6848b7797",
    POINT
    + "modules.py": "b50f5713b2a4a10af225bafdbace8a32c74821e6820fc545fb28ba6d0d16a1ed",
    POINT
    + "utils/misc.py": "23bc152bd98c7a02c8ad16906c17a93f2f0d0f334863f2b803fa19d1938d7605",
    POINT
    + "utils/structure.py": "cebe08382e12a3f4f82d4e6fc0f3ccb0f500b782b6316a417766703f484ab429",
    POINT
    + "point_transformer_v3/point_transformer_v3m1_base.py": "aeee80f3818538ed36f99add950a5d0af778fb522ae833cf86b8e64ecb90649d",
    POINT
    + "utils/serialization/__init__.py": "e6461b37ccb5dcb24725943271259c4f74dac89d27b22becb97e48ce04c8a9b1",
    POINT
    + "utils/serialization/default.py": "886b8f3f0bbfaccb96b629fe2021d0bc9a700887c2ff8ec13b347f62ae60d964",
    POINT
    + "utils/serialization/hilbert.py": "b6c4e6c763d0d1e1583448d0cfe9c1c2ff5600e4535799beed9af80b48179f9e",
    POINT
    + "utils/serialization/z_order.py": "8a3b1f516c35e03ca15cb53b10c09b8b9cbe75289ac58650b6489212e9252c21",
}
WEIGHTS = {
    "model.safetensors": "2131cdb52020b2473707465449d8bdb4f6cca61c93150a947baca02bc58ffd7b",
    "model_1.safetensors": "8a1fea9bb996eb59c2d35ad4d15c7b6fec2926e58e7abdfc9ddf24956746ba0e",
    "model_2.safetensors": "a61f49b8d5df827a9b70087f523d5aa101d99fc0a43a6a09730e6bcf08d4157b",
    "model_3.safetensors": "86db66ffe860fdc25da454d2757fcc80bb7b219d5453a72be31e49eeb48ecaf3",
    "model_4.safetensors": "ca28ac1f753b2e87c5a6058210fcf756e24ec1a230f2f217dc5b86d4469a5328",
}
BOWL_SHA = "92892268d640c54e60482562fb712b0a6b562a15917d6256bb7fe475dd058303"


def sha256(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def verify(root, bindings):
    for name, expected in bindings.items():
        if sha256(root / name) != expected:
            raise ValueError(f"unreviewed or corrupt asset: {name}")


def segment_csr(src, indptr, reduce="sum"):
    """The only torch_scatter operation used by the reviewed encoder."""
    if src.ndim != 2 or indptr.ndim != 1 or reduce not in {"sum", "mean", "min", "max"}:
        raise ValueError("unsupported CSR reduction")
    if indptr[0] != 0 or indptr[-1] != src.shape[0] or torch.any(indptr.diff() <= 0):
        raise ValueError("expected nonempty contiguous CSR segments")
    return torch.segment_reduce(src, reduce, offsets=indptr)


def sdpa_varlen(qkv, cu_seqlens, max_seqlen, dropout_p=0, softmax_scale=None):
    """Same unmasked ragged attention equation and partition, alternate kernel."""
    if qkv.ndim != 4 or qkv.shape[1] != 3 or dropout_p != 0:
        raise ValueError("inference-only packed QKV required")
    offsets = cu_seqlens.tolist()
    lengths = np.diff(offsets)
    if (
        offsets[0] != 0
        or offsets[-1] != qkv.shape[0]
        or min(lengths) <= 0
        or max(lengths) > max_seqlen
    ):
        raise ValueError("invalid attention partition")
    chunks = []
    for start, end in itertools.pairwise(offsets):
        q, k, v = qkv[start:end].permute(1, 2, 0, 3).unbind(0)
        value = nn.functional.scaled_dot_product_attention(
            q.unsqueeze(0),
            k.unsqueeze(0),
            v.unsqueeze(0),
            dropout_p=0,
            is_causal=False,
            scale=softmax_scale,
        )
        chunks.append(value[0].transpose(0, 1))
    return torch.cat(chunks)


def definitions(root, path, names, namespace):
    tree = ast.parse((root / path).read_text())
    selected = []
    found = set()
    for node in tree.body:
        name = getattr(node, "name", None)
        if isinstance(node, ast.Assign) and len(node.targets) == 1:
            name = getattr(node.targets[0], "id", None)
        if name in names:
            selected.append(node)
            found.add(name)
    if found != set(names):
        raise ValueError(f"missing reviewed definitions: {set(names) - found}")
    exec(  # noqa: S102 — selected definitions from hash-verified reviewed sources.
        compile(ast.Module(body=selected, type_ignores=[]), str(root / path), "exec"),
        namespace,
    )


def load_definitions(root):
    import gin
    import spconv.pytorch as spconv
    from addict import Dict
    from plyfile import PlyData
    from timm.layers import DropPath

    verify(root, SOURCES)
    # Import the reviewed four-file serialization package, no model registry.
    serial_path = root / (POINT + "utils/serialization/__init__.py")
    spec = importlib.util.spec_from_file_location("sonic_serialization", serial_path)
    serial = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = serial
    spec.loader.exec_module(serial)
    ns = {
        "__name__": "sonicgauss_reviewed",
        "torch": torch,
        "nn": nn,
        "np": np,
        "math": math,
        "pi": math.pi,
        "sys": sys,
        "OrderedDict": OrderedDict,
        "partial": partial,
        "List": list,
        "Optional": Optional,
        "Dict": Dict,
        "PlyData": PlyData,
        "gin": gin,
        "spconv": spconv,
        "DropPath": DropPath,
        "encode": serial.encode,
        "decode": serial.decode,
        "torch_scatter": SimpleNamespace(segment_csr=segment_csr),
        "flash_attn": SimpleNamespace(flash_attn_varlen_qkvpacked_func=sdpa_varlen),
    }
    definitions(
        root,
        POINT + "utils/misc.py",
        ["offset2batch", "batch2offset", "offset2bincount"],
        ns,
    )
    definitions(root, POINT + "utils/structure.py", ["Point"], ns)
    # Satisfy GaussianEncoder's one local import with the actual reviewed Point.
    for name in [
        "pointcept",
        "pointcept.models",
        "pointcept.models.utils",
        "pointcept.models.utils.structure",
    ]:
        if name in sys.modules:
            raise ValueError("run in a fresh process without another Pointcept import")
        sys.modules[name] = ModuleType(name)
    sys.modules["pointcept.models.utils.structure"].Point = ns["Point"]
    definitions(root, POINT + "modules.py", ["PointModule", "PointSequential"], ns)
    definitions(
        root,
        POINT + "point_transformer_v3/point_transformer_v3m1_base.py",
        [
            "RPE",
            "SerializedAttention",
            "MLP",
            "Block",
            "SerializedPooling",
            "SerializedUnpooling",
            "Embedding",
        ],
        ns,
    )
    definitions(
        root,
        "SplatFormer/models/pointtransformer_v3.py",
        [
            "PointSequential_intermediate_output",
            "PointTransformerV3",
            "PointTransformerV3Model",
        ],
        ns,
    )
    definitions(
        root,
        "SplatFormer/models/feature_predictor.py",
        ["FEATURE2CHANNEL", "ALL_FEATURES", "FeaturePredictor"],
        ns,
    )
    definitions(root, "SplatFormer/utils/transform_utils.py", ["MinMaxScaler"], ns)
    definitions(
        root,
        "stage2/common.py",
        [
            "load_ply",
            "preprocess_gaussian",
            "normalize_position",
            "GaussianEncoder",
            "retrieve_timesteps",
        ],
        ns,
    )
    definitions(root, "stage3/common.py", ["PositionEncoder", "FeatureFusion"], ns)
    definitions(root, "stage3/infer_3.py", ["inference_with_stage3"], ns)
    definitions(
        root,
        "TangoFlux/tangoflux/model.py",
        ["StableAudioPositionalEmbedding", "DurationEmbedder"],
        ns,
    )
    for cls in [nn.Identity, nn.Tanh, nn.Sigmoid]:
        gin.external_configurable(cls)
    gin.parse_config_files_and_bindings(
        [str(root / "SplatFormer/configs/model/ptv3.gin")],
        ["PointTransformerV3Model.stride=(2,2,4,4)"],
    )
    return ns


def load_weights(model, path, ignore_text=False):
    from safetensors import safe_open

    weights, ignored = {}, []
    with safe_open(path, framework="pt", device="cpu") as stream:
        for key in stream.keys():  # noqa: SIM118 — safetensors is not iterable.
            if ignore_text and key.startswith("text_encoder."):
                ignored.append(key)
                continue
            value = stream.get_tensor(key)
            if not torch.isfinite(value).all():
                raise ValueError(f"nonfinite weight: {key}")
            weights[key] = value
    model.load_state_dict(weights, strict=True)
    model.eval().requires_grad_(False)
    return {
        "loaded_keys": len(weights),
        "unused_text_encoder_keys": len(ignored),
        "strict": True,
    }


def build_models(source, assets, ns):
    import yaml
    from diffusers import (
        AutoencoderOobleck,
        FlowMatchEulerDiscreteScheduler,
        FluxTransformer2DModel,
    )

    verify(assets, WEIGHTS)
    cfg = yaml.safe_load((source / "configs/stage3.yaml").read_text())["model"]
    gaussian = ns["GaussianEncoder"](
        ns["FeaturePredictor"](), output_dim=1024, output_seq_len=64, mode="generative"
    )
    position = ns["PositionEncoder"](**cfg["position_encoder"])
    fusion = ns["FeatureFusion"](embed_dim=1024, **cfg["feature_fusion"])
    # Exact inference-reachable TangoFlux modules, excluding unused T5/tokenizer.
    model = nn.Module()
    model.audio_seq_len = cfg["audio_seq_len"]
    model.noise_scheduler = FlowMatchEulerDiscreteScheduler(num_train_timesteps=1000)
    model.fc = nn.Sequential(nn.Linear(1024, 1024), nn.ReLU())
    model.duration_emebdder = ns["DurationEmbedder"](
        1024, min_value=0, max_value=cfg["max_duration"]
    )
    model.transformer = FluxTransformer2DModel(
        in_channels=cfg["in_channels"],
        num_layers=cfg["num_layers"],
        num_single_layers=cfg["num_single_layers"],
        attention_head_dim=cfg["attention_head_dim"],
        num_attention_heads=cfg["num_attention_heads"],
        joint_attention_dim=cfg["joint_attention_dim"],
        pooled_projection_dim=1024,
        guidance_embeds=False,
    )
    vae = AutoencoderOobleck()
    modules = [
        (vae, "model.safetensors"),
        (model, "model_1.safetensors"),
        (gaussian, "model_2.safetensors"),
        (position, "model_3.safetensors"),
        (fusion, "model_4.safetensors"),
    ]
    loaded = {}
    for module, filename in modules:
        loaded[filename] = load_weights(
            module, assets / filename, ignore_text=module is model
        )
        print("loaded", filename, loaded[filename], flush=True)
        module.to("cuda")
    return model, gaussian, position, fusion, vae, loaded


def main():
    from scipy.io import wavfile

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("generated audio belongs outside the repository")
    if args.output.exists():
        raise ValueError("output must be new")
    if sha256(args.input / "bowl-6.ply") != BOWL_SHA:
        raise ValueError("pilot only supports the disclosed TRAIN bowl")
    metadata = json.loads((args.input / "bowl-6.json").read_text())
    expected_contacts = [
        [0.9306724345821787, -0.918678529191214, 0.15392957406368474],
        [-0.6213642203178644, -0.35045376636680214, 0.10162437371200583],
    ]
    if [x["contact"] for x in metadata["contacts"]] != expected_contacts:
        raise ValueError("expected the two preselected author TRAIN contacts")
    torch.set_num_threads(4)
    torch.manual_seed(0)
    ns = load_definitions(args.source)
    model, gaussian, position, fusion, vae, loaded = build_models(
        args.source, args.assets, ns
    )
    args.output.mkdir(parents=True)
    gs = ns["load_ply"](str(args.input / "bowl-6.ply"))
    if any(not torch.isfinite(x).all() for x in gs.values()):
        raise ValueError("nonfinite geometry")
    norm, scaler = ns["preprocess_gaussian"](
        gs, "cuda", scaler_class=ns["MinMaxScaler"], return_scaler=True
    )
    print("geometry", {k: list(v.shape) for k, v in norm.items()}, flush=True)
    waves, records = [], []
    # Same seed for both contacts and replay A: vary only the physical input.
    for index in [0, 1, 0]:
        name = (
            ["contact-a", "contact-b"][index]
            if len(records) < 2
            else "contact-a-replay"
        )
        start = time.monotonic()
        p = ns["normalize_position"](expected_contacts[index], scaler, "cuda")
        with torch.no_grad():
            latents = ns["inference_with_stage3"](
                model,
                gaussian,
                position,
                fusion,
                norm,
                p,
                "cuda",
                duration=3.0,
                num_inference_steps=50,
                guidance_scale=-1,
                seed=0,
            )
            wave = vae.decode(latents.transpose(2, 1)).sample[0].cpu().numpy()
        np.save(args.output / f"{name}-latent.npy", latents.cpu().numpy())
        np.save(args.output / f"{name}-raw.npy", wave)
        if (
            not np.isfinite(wave).all()
            or np.sqrt(np.mean(wave.astype(np.float64) ** 2)) < 1e-7
        ):
            raise ValueError("invalid/silent generated audio; raw preserved")
        record = {
            "name": name,
            "normalized_contact": p.tolist(),
            "seconds": time.monotonic() - start,
            "raw_peak": float(np.abs(wave).max()),
            "samples": wave.shape[1],
        }
        print(record, flush=True)
        records.append(record)
        waves.append(wave)
    gain = min(1.0, 0.98 / max(x["raw_peak"] for x in records))
    rate = vae.config.sampling_rate
    for wave, record in zip(waves, records, strict=True):
        path = args.output / (record["name"] + ".wav")
        wavfile.write(path, rate, (wave.T * gain).astype(np.float32))
        record["wav_sha256"] = sha256(path)
    comparison = np.concatenate(
        [waves[0].T * gain, np.zeros((rate // 2, 2)), waves[1].T * gain]
    ).astype(np.float32)
    wavfile.write(args.output / "comparison.wav", rate, comparison)
    result = {
        "status": "REPORT_ONLY_PRETRAINED_3D_CONTACT_PAIR",
        "source_hashes": SOURCES,
        "weights": WEIGHTS,
        "loaded": loaded,
        "geometry_sha256": BOWL_SHA,
        "geometry_points": len(norm["means"]),
        "seed": 0,
        "steps": 50,
        "guidance": -1,
        "requested_duration": 3.0,
        "full_decode_retained": True,
        "shared_gain": gain,
        "sample_rate": rate,
        "compatibility": [
            "ragged FP16-QKV SDPA instead of flash_attn",
            "torch.segment_reduce instead of segment_csr",
        ],
        "same_process_replay_exact": bool(np.array_equal(waves[0], waves[2])),
        "contact_pair_relative_rms": float(
            np.linalg.norm(waves[0] - waves[1]) / np.linalg.norm(waves[0])
        ),
        "records": records,
        "vae_config": dict(vae.config),
        "target_audio_input": False,
        "limitations": "Known TRAIN object, not new-object realism; no size, striker material or force control. No runtime promotion.",
    }
    (args.output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(
        json.dumps(
            {
                k: v
                for k, v in result.items()
                if k not in {"source_hashes", "weights", "vae_config"}
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
