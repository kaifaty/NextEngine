from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from nextengine_speech_timeline.profile import ProfileError, load_profile


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


if __name__ == "__main__":
    unittest.main()
