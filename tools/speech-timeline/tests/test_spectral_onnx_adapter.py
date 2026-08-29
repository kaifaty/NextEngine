from __future__ import annotations

from pathlib import Path
import unittest

import numpy as np

from nextengine_speech_timeline.adapters.base import AdapterError
from nextengine_speech_timeline.adapters.spectral_onnx import (
    CompositeAudioPreprocessor,
    GTCRN_SPEC,
    UL_UNAS_SPEC,
    StreamingSpectralOnnxAudioPreprocessor,
)
from nextengine_speech_timeline.audio import float32_to_pcm16le, pcm16le_to_float32


class _IdentitySession:
    def __init__(self, spec: object) -> None:
        self.spec = spec

    def run(self, output_names: list[str], inputs: dict[str, np.ndarray]) -> list[np.ndarray]:
        del output_names
        return [
            inputs["mix"],
            *[inputs[name] for name, _ in self.spec.state_inputs],
        ]


class _InvalidSession(_IdentitySession):
    def run(self, output_names: list[str], inputs: dict[str, np.ndarray]) -> list[np.ndarray]:
        values = super().run(output_names, inputs)
        values[0] = np.zeros((1, 1), dtype=np.float32)
        return values


class _RouteProcessor:
    def __init__(self, route: str) -> None:
        self.route = route
        self.calls: list[tuple[str, object]] = []
        self.load_count = 0

    def load(self) -> dict[str, int]:
        self.load_count += 1
        return {"load_count": self.load_count, "elapsed_ms": 0}

    def warmup(self) -> dict[str, int]:
        return {"elapsed_ms": 0}

    def capabilities(self) -> dict[str, object]:
        return {
            "adapter_id": f"fake-{self.route}/1",
            "asr_audio_routes": [self.route],
            "route_details": {self.route: {"stages": [self.route]}},
        }

    def reset(self, route: str, *, noise_floor_dbfs: float | None = None) -> None:
        self.calls.append(("reset", (route, noise_floor_dbfs)))

    def process_pcm(self, pcm: bytes) -> bytes:
        self.calls.append(("process", pcm))
        return pcm + self.route.encode("ascii")

    def flush(self) -> bytes:
        self.calls.append(("flush", None))
        return self.route.encode("ascii")

    def close(self) -> None:
        self.calls.append(("close", None))


class StreamingSpectralOnnxTests(unittest.TestCase):
    def test_identity_graph_preserves_exact_sample_clock_across_odd_chunks(self) -> None:
        source = (
            np.random.default_rng(17).standard_normal(4_001).astype(np.float32) * 0.05
        )
        source_pcm = float32_to_pcm16le(source)
        for spec in (GTCRN_SPEC, UL_UNAS_SPEC):
            with self.subTest(adapter=spec.adapter_id):
                adapter = StreamingSpectralOnnxAudioPreprocessor(
                    spec=spec,
                    model_id="test/model",
                    model_revision="revision",
                    onnx_path=Path("/external/model.onnx"),
                )
                adapter._session = _IdentitySession(spec)
                adapter.reset(spec.route)
                output = bytearray()
                for offset in range(0, len(source_pcm), 666):
                    output.extend(adapter.process_pcm(source_pcm[offset : offset + 666]))
                output.extend(adapter.flush())
                self.assertEqual(len(output), len(source_pcm))
                np.testing.assert_array_equal(
                    pcm16le_to_float32(bytes(output)),
                    pcm16le_to_float32(source_pcm),
                )
                self.assertEqual(adapter.flush(), b"")

    def test_invalid_model_output_fails_closed(self) -> None:
        adapter = StreamingSpectralOnnxAudioPreprocessor(
            spec=GTCRN_SPEC,
            model_id="test/model",
            model_revision="revision",
            onnx_path=Path("/external/model.onnx"),
        )
        adapter._session = _InvalidSession(GTCRN_SPEC)
        adapter.reset(GTCRN_SPEC.route)
        with self.assertRaisesRegex(AdapterError, "invalid enhanced spectrum"):
            adapter.process_pcm(b"\x00\x00" * 512)

    def test_composite_dispatches_only_the_selected_resident_route(self) -> None:
        left = _RouteProcessor("left")
        right = _RouteProcessor("right")
        adapter = CompositeAudioPreprocessor([left, right])
        loaded = adapter.load()
        self.assertEqual(loaded["load_count"], 1)
        adapter.warmup()
        adapter.reset("right", noise_floor_dbfs=-61.0)
        self.assertEqual(adapter.process_pcm(b"pcm"), b"pcmright")
        self.assertEqual(adapter.flush(), b"right")
        self.assertFalse(any(call[0] == "process" for call in left.calls))
        self.assertIn(("reset", ("right", -61.0)), right.calls)
        self.assertEqual(adapter.capabilities()["asr_audio_routes"], ["left", "right"])


if __name__ == "__main__":
    unittest.main()
