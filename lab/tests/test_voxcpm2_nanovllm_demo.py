import importlib.util
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "voxcpm2_nanovllm_demo.py"
SPEC = importlib.util.spec_from_file_location("voxcpm2_nanovllm_demo", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
DEMO = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = DEMO
SPEC.loader.exec_module(DEMO)


class VoxCPM2NanoVllmDemoTests(unittest.TestCase):
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
            extra = root / "extra-site"
            (site / "torch" / "lib").mkdir(parents=True)
            (site / "nvidia" / "cublas" / "lib").mkdir(parents=True)
            extra.mkdir()
            with mock.patch.dict(os.environ, {"PYTHONPATH": "/host/path"}):
                environment = DEMO._runtime_environment(root, 2, extra)
            self.assertEqual(environment["CUDA_VISIBLE_DEVICES"], "2")
            self.assertEqual(
                environment["PYTHONPATH"],
                os.pathsep.join((str(extra), str(root / "source"))),
            )
            self.assertNotIn("/host/path", environment["PYTHONPATH"])
            self.assertEqual(environment["HF_HUB_OFFLINE"], "1")
            self.assertIn(str(site / "torch" / "lib"), environment["LD_LIBRARY_PATH"])

    def test_validate_installation_checks_source_and_model_sizes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "runtime"
            model = Path(directory) / "model"
            source = root / "source"
            python = root / "venv" / "bin" / "python"
            source.mkdir(parents=True)
            model.mkdir()
            python.parent.mkdir(parents=True)
            python.write_text("#!/bin/sh\n", encoding="utf-8")
            python.chmod(0o755)
            artifact = model / "artifact.bin"
            artifact.write_bytes(b"model")
            with (
                mock.patch.object(DEMO, "MODEL_FILES", {Path("artifact.bin"): 5}),
                mock.patch.object(
                    DEMO, "_git_revision", return_value=DEMO.NANO_SOURCE_REVISION
                ),
            ):
                result = DEMO.validate_installation(root, model, None)
            self.assertEqual(result["status"], "PASS")
            self.assertEqual(result["model_files_checked"], 1)

    def test_parse_worker_result_uses_last_structured_line(self) -> None:
        output = "noise\n{}{}\n".format(
            DEMO.RESULT_PREFIX, '{"status": "PASS", "value": 7}'
        )
        self.assertEqual(DEMO._parse_worker_result(output)["value"], 7)


if __name__ == "__main__":
    unittest.main()
