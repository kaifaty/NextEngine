from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = (
    Path(__file__).resolve().parents[1] / "scripts" / "fish_s2_pro_audio_cpp_demo.py"
)
SPEC = importlib.util.spec_from_file_location("fish_s2_pro_audio_cpp_demo", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class FishS2ProAudioCppDemoTests(unittest.TestCase):
    def test_generated_output_is_rejected_inside_repository(self) -> None:
        with self.assertRaises(MODULE.DemoError):
            MODULE._external_path(MODULE.REPOSITORY_ROOT / "demo.wav", "output")

    def test_cmake_cache_parser_ignores_comments_and_types(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            cache = Path(temporary_directory) / "CMakeCache.txt"
            cache.write_text(
                "// comment\nENGINE_ENABLE_CUDA:BOOL=ON\n"
                "CMAKE_CUDA_ARCHITECTURES:UNINITIALIZED=86\n",
                encoding="utf-8",
            )
            values = MODULE._read_cmake_cache(cache)
        self.assertEqual(values["ENGINE_ENABLE_CUDA"], "ON")
        self.assertEqual(values["CMAKE_CUDA_ARCHITECTURES"], "86")

    def test_metrics_parser_groups_requests(self) -> None:
        parsed = MODULE._parse_metrics(
            "metrics.wall_ms=2900\n"
            "metrics[prewarm].wall_ms=3000\n"
            "metrics[warm_1].wall_ms=2700.5\n"
            "metrics[warm_1].rtf=0.625\n"
        )
        self.assertEqual(parsed["prewarm"]["wall_ms"], 3000.0)
        self.assertEqual(parsed["request"]["wall_ms"], 2900.0)
        self.assertEqual(parsed["warm_1"]["wall_ms"], 2700.5)
        self.assertEqual(parsed["warm_1"]["rtf"], 0.625)

    def test_default_profile_keeps_warm_cache_and_cuda_graphs(self) -> None:
        self.assertEqual(MODULE.MODEL_RELATIVE_PATH.suffix, ".gguf")
        self.assertEqual(MODULE.MODEL_SIZE, 6_317_911_232)
        self.assertEqual(len(MODULE.MODEL_SHA256), 64)


if __name__ == "__main__":
    unittest.main()
