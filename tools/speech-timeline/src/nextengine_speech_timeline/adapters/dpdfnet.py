from __future__ import annotations

from dataclasses import asdict, dataclass
import math
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
GAIN_FRAME_SAMPLES = 320


@dataclass(frozen=True)
class SpeechAwareGainConfig:
    """Bounded gain for denoised speech, never an unrestricted AGC."""

    enabled: bool = False
    activation_threshold_dbfs: float = -75.0
    target_dbfs: float = -26.0
    max_gain_db: float = 32.0
    attack_ms: int = 40
    release_ms: int = 160
    limiter_peak_dbfs: float = -1.0

    def __post_init__(self) -> None:
        if not -90.0 <= self.activation_threshold_dbfs <= -25.0:
            raise ValueError("gain activation threshold must be between -90 and -25 dBFS")
        if not -45.0 <= self.target_dbfs <= -6.0:
            raise ValueError("gain target must be between -45 and -6 dBFS")
        if not 0.0 <= self.max_gain_db <= 40.0:
            raise ValueError("gain maximum must be between 0 and 40 dB")
        if not 10 <= self.attack_ms <= 1_000 or not 10 <= self.release_ms <= 2_000:
            raise ValueError("gain attack/release must be bounded positive milliseconds")
        if not -6.0 <= self.limiter_peak_dbfs <= -0.1:
            raise ValueError("gain limiter peak must be between -6 and -0.1 dBFS")

    def as_dict(self) -> dict[str, bool | float | int]:
        return asdict(self)


class SpeechAwareGain:
    """Frame-continuous, denoised-signal gain with bounded residual-noise lift."""

    def __init__(self, config: SpeechAwareGainConfig) -> None:
        self.config = config
        self.reset()

    def reset(self) -> None:
        self._pending = np.zeros(0, dtype=np.float32)
        self._current_gain_db = 0.0

    def process(self, samples: np.ndarray) -> np.ndarray:
        values = np.asarray(samples, dtype=np.float32)
        if values.ndim != 1:
            raise AdapterError("speech gain requires a mono sample array")
        if not self.config.enabled:
            return values
        if len(values) == 0:
            return np.zeros(0, dtype=np.float32)
        self._pending = np.concatenate((self._pending, values))
        frames: list[np.ndarray] = []
        while len(self._pending) >= GAIN_FRAME_SAMPLES:
            frames.append(self._apply_frame(self._pending[:GAIN_FRAME_SAMPLES]))
            self._pending = self._pending[GAIN_FRAME_SAMPLES:]
        return np.concatenate(frames) if frames else np.zeros(0, dtype=np.float32)

    def flush(self) -> np.ndarray:
        if not self.config.enabled or len(self._pending) == 0:
            pending = self._pending
            self._pending = np.zeros(0, dtype=np.float32)
            return pending
        original = len(self._pending)
        padded = np.pad(self._pending, (0, GAIN_FRAME_SAMPLES - original))
        self._pending = np.zeros(0, dtype=np.float32)
        return self._apply_frame(padded)[:original]

    def _apply_frame(self, frame: np.ndarray) -> np.ndarray:
        rms = float(np.sqrt(np.mean(frame * frame)))
        level_dbfs = 20.0 * math.log10(max(rms, 1e-8))
        desired_gain = (
            min(self.config.max_gain_db, max(0.0, self.config.target_dbfs - level_dbfs))
            if level_dbfs >= self.config.activation_threshold_dbfs
            else 0.0
        )
        interval_ms = GAIN_FRAME_SAMPLES * 1_000 / SAMPLE_RATE_HZ
        time_constant_ms = self.config.attack_ms if desired_gain > self._current_gain_db else self.config.release_ms
        interpolation = 1.0 - math.exp(-interval_ms / time_constant_ms)
        self._current_gain_db += (desired_gain - self._current_gain_db) * interpolation
        output = frame * (10.0 ** (self._current_gain_db / 20.0))
        limit = 10.0 ** (self.config.limiter_peak_dbfs / 20.0)
        peak = float(np.max(np.abs(output))) if len(output) else 0.0
        if peak > limit:
            output = output * (limit / peak)
        return output.astype(np.float32, copy=False)


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
        gain_config: SpeechAwareGainConfig | None = None,
    ) -> None:
        if model_name not in SUPPORTED_MODELS:
            raise AdapterError(f"unsupported DPDFNet model name: {model_name}")
        self.model_id = model_id
        self.model_revision = model_revision
        self.model_name = model_name
        self.onnx_path = onnx_path.expanduser().resolve()
        self.gain_config = gain_config or SpeechAwareGainConfig()
        self._gain = SpeechAwareGain(self.gain_config)
        self._enhancer: Any | None = None
        self.load_count = 0
        self._model_input_samples = 0
        self._model_output_samples = 0

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
            self._gain.reset()
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
            "algorithmic_latency_ms": 20 + (20 if self.gain_config.enabled else 0),
            "routes": ["asr"],
            "affect_input": "raw_pcm",
            "vad_input": "raw_pcm",
            "runtime": "onnxruntime_cpu",
            "gain": self.gain_config.as_dict(),
        }

    def reset(self) -> None:
        self._require_loaded().reset()
        self._gain.reset()
        self._model_input_samples = 0
        self._model_output_samples = 0

    def process_pcm(self, pcm: bytes) -> bytes:
        samples = pcm16le_to_float32(pcm)
        try:
            output = self._require_loaded().process(samples, sample_rate=SAMPLE_RATE_HZ)
            self._model_input_samples += len(samples)
            self._model_output_samples += len(output)
            return float32_to_pcm16le(self._gain.process(output))
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
            missing = self._model_input_samples - self._model_output_samples - produced
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
            output = output[: max(0, self._model_input_samples - self._model_output_samples)]
            self._model_output_samples += len(output)
            gained = self._gain.process(output)
            return float32_to_pcm16le(
                np.concatenate((gained, self._gain.flush()))
                if len(gained)
                else self._gain.flush()
            )
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
