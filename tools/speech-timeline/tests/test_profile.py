from __future__ import annotations

import hashlib
import json
from dataclasses import replace
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from nextengine_speech_timeline.profile import ProfileError, load_profile, validate_profile_artifacts


class ProfileTests(unittest.TestCase):
    def test_exact_external_artifacts_are_validated_before_model_load(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            model = root / "model.gguf"
            model.write_bytes(b"voxtral")
            library = root / "libtranscribe.so"
            library.write_bytes(b"runtime")
            transcribe_root = root / "transcribe.cpp"
            transcribe_root.mkdir()
            cache = root / "emotion-cache"
            cache.mkdir()
            profile_path = root / "profile.json"
            revision = "a" * 40
            profile_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "voxtral": {
                            "model_path": str(model),
                            "model_size_bytes": model.stat().st_size,
                            "model_sha256": f"sha256:{hashlib.sha256(model.read_bytes()).hexdigest()}",
                            "transcribe_root": str(transcribe_root),
                            "library": str(library),
                            "runtime_revision": revision,
                            "backend": "cuda",
                            "delay_ms": 480,
                            "partial_decode_interval_ms": 480,
                        },
                        "emotion": {
                            "model_id": "emotion/model",
                            "model_revision": "b" * 40,
                            "cache_dir": str(cache),
                            "device": "cpu",
                            "classification": "unclassified_local_only",
                        },
                        "service": {
                            "port": 0,
                            "ready_file": str(root / "ready.json"),
                            "max_frame_bytes": 32_000,
                            "max_turn_bytes": 960_000,
                        },
                    }
                ),
                encoding="utf-8",
            )
            completed = subprocess.CompletedProcess(
                ["git"], 0, stdout=f"{revision}\n", stderr=""
            )
            with patch(
                "nextengine_speech_timeline.profile.subprocess.run",
                return_value=completed,
            ):
                profile = load_profile(profile_path)
        self.assertEqual(profile.voxtral.model_sha256, f"sha256:{hashlib.sha256(b'voxtral').hexdigest()}")
        self.assertEqual(profile.service.port, 0)
        self.assertEqual(profile.voxtral.partial_decode_interval_ms, 480)

    def test_legacy_v1_profile_defaults_partial_decode_interval(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            model = root / "model.gguf"
            model.write_bytes(b"voxtral")
            library = root / "libtranscribe.so"
            library.write_bytes(b"runtime")
            transcribe_root = root / "transcribe.cpp"
            transcribe_root.mkdir()
            cache = root / "emotion-cache"
            cache.mkdir()
            revision = "a" * 40
            profile_path = root / "profile.json"
            profile_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "voxtral": {
                            "model_path": str(model),
                            "model_size_bytes": model.stat().st_size,
                            "model_sha256": f"sha256:{hashlib.sha256(model.read_bytes()).hexdigest()}",
                            "transcribe_root": str(transcribe_root),
                            "library": str(library),
                            "runtime_revision": revision,
                            "backend": "cuda",
                            "delay_ms": 480,
                        },
                        "emotion": {
                            "model_id": "emotion/model",
                            "model_revision": "b" * 40,
                            "cache_dir": str(cache),
                            "device": "cpu",
                            "classification": "unclassified_local_only",
                        },
                        "service": {
                            "port": 0,
                            "ready_file": str(root / "ready.json"),
                            "max_frame_bytes": 32_000,
                            "max_turn_bytes": 960_000,
                        },
                    }
                ),
                encoding="utf-8",
            )
            completed = subprocess.CompletedProcess(
                ["git"], 0, stdout=f"{revision}\n", stderr=""
            )
            with patch(
                "nextengine_speech_timeline.profile.subprocess.run",
                return_value=completed,
            ):
                profile = load_profile(profile_path)
        self.assertEqual(profile.voxtral.partial_decode_interval_ms, 240)

    def test_hash_mismatch_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            profile = root / "profile.json"
            profile.write_text("{}", encoding="utf-8")
            with self.assertRaises(ProfileError):
                load_profile(profile)

    def test_optional_gigaam_snapshot_is_hash_closed_and_selectable(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            voxtral = root / "model.gguf"
            voxtral.write_bytes(b"voxtral")
            library = root / "libtranscribe.so"
            library.write_bytes(b"runtime")
            transcribe_root = root / "transcribe.cpp"
            transcribe_root.mkdir()
            cache = root / "emotion-cache"
            cache.mkdir()
            snapshot = root / "gigaam-snapshot"
            snapshot.mkdir()
            artifacts = {
                "pytorch_model.bin": b"weights",
                "modeling_gigaam.py": b"modeling",
                "config.json": b"{}",
                "tokenizer.model": b"tokenizer",
            }
            for filename, payload in artifacts.items():
                (snapshot / filename).write_bytes(payload)
            revision = "a" * 40
            profile_path = root / "profile.json"
            profile_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "default_asr_model": "gigaam-v3-e2e-rnnt",
                        "voxtral": {
                            "model_path": str(voxtral),
                            "model_size_bytes": voxtral.stat().st_size,
                            "model_sha256": f"sha256:{hashlib.sha256(voxtral.read_bytes()).hexdigest()}",
                            "transcribe_root": str(transcribe_root),
                            "library": str(library),
                            "runtime_revision": revision,
                            "backend": "cuda",
                            "delay_ms": 480,
                        },
                        "gigaam": {
                            "model_id": "ai-sage/GigaAM-v3",
                            "model_revision": "b" * 40,
                            "snapshot_path": str(snapshot),
                            "device": "cuda",
                            "classification": "unclassified_local_only",
                            "weights_sha256": f"sha256:{hashlib.sha256(artifacts['pytorch_model.bin']).hexdigest()}",
                            "modeling_sha256": f"sha256:{hashlib.sha256(artifacts['modeling_gigaam.py']).hexdigest()}",
                            "config_sha256": f"sha256:{hashlib.sha256(artifacts['config.json']).hexdigest()}",
                            "tokenizer_sha256": f"sha256:{hashlib.sha256(artifacts['tokenizer.model']).hexdigest()}",
                        },
                        "emotion": {
                            "model_id": "emotion/model",
                            "model_revision": "c" * 40,
                            "cache_dir": str(cache),
                            "device": "cpu",
                            "classification": "unclassified_local_only",
                        },
                        "service": {
                            "port": 0,
                            "ready_file": str(root / "ready.json"),
                            "max_frame_bytes": 32_000,
                            "max_turn_bytes": 960_000,
                        },
                    }
                ),
                encoding="utf-8",
            )
            completed = subprocess.CompletedProcess(
                ["git"], 0, stdout=f"{revision}\n", stderr=""
            )
            with patch(
                "nextengine_speech_timeline.profile.subprocess.run",
                return_value=completed,
            ):
                profile = load_profile(profile_path)
            self.assertEqual(profile.default_asr_model, "gigaam-v3-e2e-rnnt")
            self.assertIsNotNone(profile.gigaam)
            (snapshot / "modeling_gigaam.py").write_bytes(b"changed")
            with self.assertRaisesRegex(ProfileError, "modeling_gigaam.py SHA-256"):
                validate_profile_artifacts(profile)

    def test_optional_nemotron_runtime_and_model_are_hash_closed_and_selectable(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            voxtral = root / "voxtral.gguf"
            voxtral.write_bytes(b"voxtral")
            transcribe_library = root / "libtranscribe.so"
            transcribe_library.write_bytes(b"transcribe")
            transcribe_root = root / "transcribe.cpp"
            transcribe_root.mkdir()
            cache = root / "emotion-cache"
            cache.mkdir()
            nemotron_model = root / "nemotron.gguf"
            nemotron_model.write_bytes(b"nemotron")
            runtime_root = root / "nemo-speech.cpp"
            runtime_root.mkdir()
            implementation = root / "libnemo_speech_asr.so"
            implementation.write_bytes(b"implementation")
            abi = root / "libnemo_speech_asr_c.so"
            abi.write_bytes(b"abi")
            revision = "a" * 40
            profile_path = root / "profile.json"
            profile_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "default_asr_model": "nemotron-3.5-streaming",
                        "voxtral": {
                            "model_path": str(voxtral),
                            "model_size_bytes": voxtral.stat().st_size,
                            "model_sha256": f"sha256:{hashlib.sha256(voxtral.read_bytes()).hexdigest()}",
                            "transcribe_root": str(transcribe_root),
                            "library": str(transcribe_library),
                            "runtime_revision": revision,
                            "backend": "cuda",
                            "delay_ms": 480,
                        },
                        "nemotron": {
                            "model_id": "nvidia/nemotron-3.5-asr-streaming-0.6b",
                            "model_revision": "b" * 40,
                            "model_path": str(nemotron_model),
                            "model_size_bytes": nemotron_model.stat().st_size,
                            "model_sha256": f"sha256:{hashlib.sha256(nemotron_model.read_bytes()).hexdigest()}",
                            "runtime_root": str(runtime_root),
                            "runtime_revision": revision,
                            "implementation_library": str(implementation),
                            "implementation_library_sha256": f"sha256:{hashlib.sha256(implementation.read_bytes()).hexdigest()}",
                            "abi_library": str(abi),
                            "abi_library_sha256": f"sha256:{hashlib.sha256(abi.read_bytes()).hexdigest()}",
                            "gpu": 0,
                            "right_context": 1,
                            "classification": "unclassified_local_only",
                        },
                        "emotion": {
                            "model_id": "emotion/model",
                            "model_revision": "c" * 40,
                            "cache_dir": str(cache),
                            "device": "cpu",
                            "classification": "unclassified_local_only",
                        },
                        "service": {
                            "port": 0,
                            "ready_file": str(root / "ready.json"),
                            "max_frame_bytes": 32_000,
                            "max_turn_bytes": 960_000,
                        },
                    }
                ),
                encoding="utf-8",
            )
            completed = subprocess.CompletedProcess(
                ["git"], 0, stdout=f"{revision}\n", stderr=""
            )
            with patch(
                "nextengine_speech_timeline.profile.subprocess.run",
                return_value=completed,
            ):
                profile = load_profile(profile_path)
            self.assertEqual(profile.default_asr_model, "nemotron-3.5-streaming")
            self.assertIsNotNone(profile.nemotron)
            abi.write_bytes(b"changed")
            with (
                patch(
                    "nextengine_speech_timeline.profile.subprocess.run",
                    return_value=completed,
                ),
                self.assertRaisesRegex(ProfileError, "C ABI library SHA-256"),
            ):
                validate_profile_artifacts(profile)

    def test_optional_audio_preprocessor_requires_pinned_external_onnx(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            model = root / "model.gguf"
            model.write_bytes(b"voxtral")
            library = root / "libtranscribe.so"
            library.write_bytes(b"runtime")
            transcribe_root = root / "transcribe.cpp"
            transcribe_root.mkdir()
            cache = root / "emotion-cache"
            cache.mkdir()
            preprocessor_model = root / "dpdfnet2.onnx"
            preprocessor_model.write_bytes(b"pinned-onnx")
            revision = "a" * 40
            profile_path = root / "profile.json"
            profile_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "voxtral": {
                            "model_path": str(model),
                            "model_size_bytes": model.stat().st_size,
                            "model_sha256": f"sha256:{hashlib.sha256(model.read_bytes()).hexdigest()}",
                            "transcribe_root": str(transcribe_root),
                            "library": str(library),
                            "runtime_revision": revision,
                            "backend": "cuda",
                            "delay_ms": 480,
                        },
                        "emotion": {
                            "model_id": "emotion/model",
                            "model_revision": "b" * 40,
                            "cache_dir": str(cache),
                            "device": "cpu",
                            "classification": "unclassified_local_only",
                        },
                        "audio_preprocessor": {
                            "adapter_id": "dpdfnet-streaming/1",
                            "model_id": "Ceva-IP/DPDFNet",
                            "model_revision": "c" * 40,
                            "model_name": "dpdfnet2",
                            "model_path": str(preprocessor_model),
                            "model_size_bytes": preprocessor_model.stat().st_size,
                            "model_sha256": f"sha256:{hashlib.sha256(preprocessor_model.read_bytes()).hexdigest()}",
                            "routing": "asr_only",
                        },
                        "service": {
                            "port": 0,
                            "ready_file": str(root / "ready.json"),
                            "max_frame_bytes": 32_000,
                            "max_turn_bytes": 960_000,
                        },
                    }
                ),
                encoding="utf-8",
            )
            completed = subprocess.CompletedProcess(
                ["git"], 0, stdout=f"{revision}\n", stderr=""
            )
            with patch(
                "nextengine_speech_timeline.profile.subprocess.run",
                return_value=completed,
            ):
                profile = load_profile(profile_path)
            self.assertIsNotNone(profile.audio_preprocessor)
            assert profile.audio_preprocessor is not None
            self.assertEqual(profile.audio_preprocessor.model_name, "dpdfnet2")
            self.assertEqual(profile.audio_preprocessor.gain_placement, "post_denoise")
            self.assertEqual(
                profile.audio_preprocessor.whisper_attenuation_limit_db,
                12.0,
            )
            damaged = preprocessor_model.write_bytes(b"altered.onx")
            self.assertGreater(damaged, 0)
            with self.assertRaisesRegex(ProfileError, "SHA-256"):
                validate_profile_artifacts(profile)

    def test_wavlm_profile_requires_the_pinned_external_weights(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            model = root / "model.gguf"
            model.write_bytes(b"voxtral")
            library = root / "libtranscribe.so"
            library.write_bytes(b"runtime")
            transcribe_root = root / "transcribe.cpp"
            transcribe_root.mkdir()
            cache = root / "emotion-cache"
            weights = (
                cache
                / "models--Aniemore--wavlm-emotion-russian-resd"
                / "snapshots"
                / ("b" * 40)
                / "model.safetensors"
            )
            weights.parent.mkdir(parents=True)
            weights.write_bytes(b"wavlm")
            revision = "a" * 40
            profile_path = root / "profile.json"
            profile_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "voxtral": {
                            "model_path": str(model),
                            "model_size_bytes": model.stat().st_size,
                            "model_sha256": f"sha256:{hashlib.sha256(model.read_bytes()).hexdigest()}",
                            "transcribe_root": str(transcribe_root),
                            "library": str(library),
                            "runtime_revision": revision,
                            "backend": "cuda",
                            "delay_ms": 480,
                        },
                        "emotion": {
                            "adapter_id": "transformers-wavlm-russian-ser/1",
                            "model_id": "Aniemore/wavlm-emotion-russian-resd",
                            "model_revision": "b" * 40,
                            "cache_dir": str(cache),
                            "device": "cpu",
                            "classification": "unclassified_local_only",
                            "weights_sha256": f"sha256:{hashlib.sha256(weights.read_bytes()).hexdigest()}",
                        },
                        "service": {
                            "port": 0,
                            "ready_file": str(root / "ready.json"),
                            "max_frame_bytes": 32_000,
                            "max_turn_bytes": 960_000,
                        },
                    }
                ),
                encoding="utf-8",
            )
            completed = subprocess.CompletedProcess(
                ["git"], 0, stdout=f"{revision}\n", stderr=""
            )
            with patch(
                "nextengine_speech_timeline.profile.subprocess.run",
                return_value=completed,
            ):
                profile = load_profile(profile_path)
            generic_without_map = replace(
                profile,
                emotion=replace(
                    profile.emotion,
                    adapter_id="transformers-wavlm-audio-classification/1",
                    label_map=None,
                ),
            )
            with self.assertRaisesRegex(ProfileError, "requires emotion.label_map"):
                validate_profile_artifacts(generic_without_map)
        self.assertEqual(profile.emotion.adapter_id, "transformers-wavlm-russian-ser/1")
        self.assertEqual(profile.emotion.weights_sha256, f"sha256:{hashlib.sha256(b'wavlm').hexdigest()}")


if __name__ == "__main__":
    unittest.main()
