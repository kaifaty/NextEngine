from __future__ import annotations

from pathlib import Path
import time
from typing import Any

import numpy as np

from ..audio import float32_to_pcm16le, pcm16le_to_float32
from .base import AdapterError


ADAPTER_ID = "dpdfnet-streaming/1"
IMPLEMENTATION_VERSION = "0.6.0"
SAMPLE_RATE_HZ = 16_000
SUPPORTED_MODELS = {
    "baseline",
    "dpdfnet2",
    "dpdfnet4",
    "dpdfnet8",
}
HOP_SAMPLES = 160


class DpdfNetAudioPreprocessor:
    """CPU-only, causal DPDFNet adapter for the optional ASR branch.

    ``StreamEnhancer`` owns recurrent state and must remain resident.  The
    service resets that state once per utterance and serializes calls on a
    dedicated CPU worker.  Passing an explicit ONNX path prevents upstream's
    convenience auto-download path from running in the service process.
    """

    def __init__(
        self,
        *,
        model_id: str,
        model_revision: str,
        model_name: str,
        onnx_path: Path,
    ) -> None:
        if model_name not in SUPPORTED_MODELS:
            raise AdapterError(f"unsupported DPDFNet model name: {model_name}")
        self.model_id = model_id
        self.model_revision = model_revision
        self.model_name = model_name
        self.onnx_path = onnx_path.expanduser().resolve()
        self._enhancer: Any | None = None
        self.load_count = 0
        self._input_samples = 0
        self._output_samples = 0

    def load(self) -> dict[str, int | str]:
        if self._enhancer is not None:
            return {"load_count": self.load_count, "elapsed_ms": 0}
        started = time.perf_counter_ns()
        try:
            from dpdfnet import StreamEnhancer

            self._enhancer = StreamEnhancer(
                model=self.model_name,
                onnx_path=self.onnx_path,
                verbose=False,
            )
        except Exception as error:
            raise AdapterError(f"cannot load pinned DPDFNet ONNX: {type(error).__name__}") from error
        self.load_count += 1
        return {
            "load_count": self.load_count,
            "elapsed_ms": round((time.perf_counter_ns() - started) / 1_000_000),
        }

    def warmup(self) -> dict[str, int]:
        enhancer = self._require_loaded()
        started = time.perf_counter_ns()
        try:
            # One causal window initializes the ONNX path. Reset afterwards so
            # no artificial zero audio or recurrent state reaches a user turn.
            enhancer.process(np.zeros(320, dtype=np.float32), sample_rate=SAMPLE_RATE_HZ)
            enhancer.reset()
        except Exception as error:
            raise AdapterError(f"DPDFNet warmup failed: {type(error).__name__}") from error
        return {"elapsed_ms": round((time.perf_counter_ns() - started) / 1_000_000)}

    def capabilities(self) -> dict[str, object]:
        return {
            "adapter_id": ADAPTER_ID,
            "implementation_version": IMPLEMENTATION_VERSION,
            "model_id": self.model_id,
            "model_revision": self.model_revision,
            "model_name": self.model_name,
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "streaming": True,
            "causal": True,
            "algorithmic_latency_ms": 20,
            "routes": ["asr"],
            "affect_input": "raw_pcm",
            "vad_input": "raw_pcm",
            "runtime": "onnxruntime_cpu",
        }

    def reset(self) -> None:
        self._require_loaded().reset()
        self._input_samples = 0
        self._output_samples = 0

    def process_pcm(self, pcm: bytes) -> bytes:
        samples = pcm16le_to_float32(pcm)
        try:
            output = self._require_loaded().process(samples, sample_rate=SAMPLE_RATE_HZ)
            self._input_samples += len(samples)
            self._output_samples += len(output)
            return float32_to_pcm16le(output)
        except AdapterError:
            raise
        except Exception as error:
            raise AdapterError(f"DPDFNet streaming inference failed: {type(error).__name__}") from error

    def flush(self) -> bytes:
        try:
            enhancer = self._require_loaded()
            pieces = [enhancer.flush()]
            produced = sum(len(piece) for piece in pieces)
            # DPDFNet's public flush pads one causal frame.  If a turn ends
            # between hops, the last fractional hop remains in its internal
            # overlap buffer. Feed bounded zero context to drain that acoustic
            # tail, then truncate it to the exact captured sample clock.
            missing = self._input_samples - self._output_samples - produced
            while missing > 0:
                extra = enhancer.process(
                    np.zeros(HOP_SAMPLES, dtype=np.float32),
                    sample_rate=SAMPLE_RATE_HZ,
                )
                if len(extra) == 0:
                    raise AdapterError("DPDFNet did not drain its causal tail")
                piece = extra[:missing]
                pieces.append(piece)
                produced += len(piece)
                missing -= len(piece)
            output = np.concatenate(pieces) if pieces else np.zeros(0, dtype=np.float32)
            output = output[: max(0, self._input_samples - self._output_samples)]
            self._output_samples += len(output)
            return float32_to_pcm16le(output)
        except AdapterError:
            raise
        except Exception as error:
            raise AdapterError(f"DPDFNet flush failed: {type(error).__name__}") from error

    def close(self) -> None:
        self._enhancer = None

    def _require_loaded(self) -> Any:
        if self._enhancer is None:
            raise AdapterError("DPDFNet adapter was used before load")
        return self._enhancer
