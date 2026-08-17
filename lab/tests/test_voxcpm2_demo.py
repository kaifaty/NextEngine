import importlib.util
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "voxcpm2_demo.py"
SPEC = importlib.util.spec_from_file_location("voxcpm2_demo", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
DEMO = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = DEMO
SPEC.loader.exec_module(DEMO)


class VoxCPM2DemoTests(unittest.TestCase):
    def test_designed_text_wraps_sanitized_control(self) -> None:
        self.assertEqual(
            DEMO._designed_text("(A calm adult man)", "Привет!"),
            "(A calm adult man)Привет!",
        )
        self.assertEqual(DEMO._designed_text("", "Привет!"), "Привет!")

    def test_control_override_wins_over_preset(self) -> None:
        args = DEMO.argparse.Namespace(voice="adult-male", control="Custom voice")
        self.assertEqual(DEMO._control_for(args), "Custom voice")

    def test_external_path_rejects_repository_output(self) -> None:
        with self.assertRaises(DEMO.DemoError):
            DEMO._external_path(DEMO.REPOSITORY_ROOT / "output.wav", "test output")

    def test_runtime_environment_is_isolated(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            site = root / "venv" / "lib" / "python3.11" / "site-packages"
            (site / "torch" / "lib").mkdir(parents=True)
            (site / "nvidia" / "cublas" / "lib").mkdir(parents=True)
            with mock.patch.dict(os.environ, {"PYTHONPATH": "/host/path"}):
                environment = DEMO._runtime_environment(root, 2)
            self.assertEqual(environment["CUDA_VISIBLE_DEVICES"], "2")
            self.assertEqual(environment["PYTHONPATH"], str(root / "source" / "src"))
            self.assertNotIn("/host/path", environment["PYTHONPATH"])
            self.assertEqual(environment["HF_HUB_OFFLINE"], "1")
            self.assertIn(str(site / "torch" / "lib"), environment["LD_LIBRARY_PATH"])

    def test_validate_installation_checks_pinned_small_closure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            model = root / "model"
            python = root / "venv" / "bin" / "python"
            source.mkdir()
            model.mkdir()
            python.parent.mkdir(parents=True)
            python.write_text("#!/bin/sh\n", encoding="utf-8")
            python.chmod(0o755)
            artifact_path = model / "artifact.bin"
            artifact_path.write_bytes(b"model")
            artifacts = {
                Path("artifact.bin"): DEMO.Artifact(
                    5, DEMO.hashlib.sha256(b"model").hexdigest()
                )
            }
            with (
                mock.patch.object(DEMO, "MODEL_ARTIFACTS", artifacts),
                mock.patch.object(
                    DEMO, "_git_revision", return_value=DEMO.SOURCE_REVISION
                ),
            ):
                result = DEMO.validate_installation(root, verify_model_hash=True)
            self.assertEqual(result["status"], "PASS")
            self.assertTrue(result["model_hashes_checked"])
            self.assertEqual(result["model_bytes"], 5)

    def test_parse_worker_result_uses_last_structured_line(self) -> None:
        output = "noise\n{}{}\n".format(
            DEMO.RESULT_PREFIX, '{"status": "PASS", "value": 7}'
        )
        self.assertEqual(DEMO._parse_worker_result(output)["value"], 7)


if __name__ == "__main__":
    unittest.main()
