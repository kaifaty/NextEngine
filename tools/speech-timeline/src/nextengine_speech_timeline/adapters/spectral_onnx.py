from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import time
from typing import Any

import numpy as np

from ..audio import float32_to_pcm16le, pcm16le_to_float32
from ..protocol import ASR_AUDIO_ROUTE_GTCRN, ASR_AUDIO_ROUTE_UL_UNAS
from .base import AdapterError


IMPLEMENTATION_VERSION = "1.0.0"
COMPOSITE_ADAPTER_ID = "composite-audio-preprocessor/1"
GTCRN_ADAPTER_ID = "gtcrn-onnx-streaming/1"
UL_UNAS_ADAPTER_ID = "ul-unas-onnx-streaming/1"
SAMPLE_RATE_HZ = 16_000
FFT_SAMPLES = 512
HOP_SAMPLES = 256
ALGORITHMIC_LATENCY_MS = FFT_SAMPLES * 1_000 // SAMPLE_RATE_HZ


@dataclass(frozen=True)
class SpectralModelSpec:
    adapter_id: str
    route: str
    window: str
    state_inputs: tuple[tuple[str, tuple[int, ...]], ...]
    state_outputs: tuple[str, ...]


GTCRN_SPEC = SpectralModelSpec(
    adapter_id=GTCRN_ADAPTER_ID,
    route=ASR_AUDIO_ROUTE_GTCRN,
    window="sqrt_hann",
    state_inputs=(
        ("conv_cache", (2, 1, 16, 16, 33)),
        ("tra_cache", (2, 3, 1, 1, 16)),
        ("inter_cache", (2, 1, 33, 16)),
    ),
    state_outputs=("conv_cache_out", "tra_cache_out", "inter_cache_out"),
)

UL_UNAS_SPEC = SpectralModelSpec(
    adapter_id=UL_UNAS_ADAPTER_ID,
    route=ASR_AUDIO_ROUTE_UL_UNAS,
    window="hann",
    state_inputs=(
        ("conv_cache", (1, 5358)),
        ("tfa_cache", (1, 402)),
        ("inter_cache", (1, 1056)),
    ),
    state_outputs=("conv_cache_out", "tfa_cache_out", "inter_cache_out"),
)

SPECS_BY_ADAPTER_ID = {
    GTCRN_ADAPTER_ID: GTCRN_SPEC,
    UL_UNAS_ADAPTER_ID: UL_UNAS_SPEC,
}


class StreamingSpectralOnnxAudioPreprocessor:
    """Causal frame-by-frame ONNX enhancer with bounded STFT/OLA state.

    Both supported upstream graphs consume one 257-bin complex spectrum at a
    time and return their recurrent caches explicitly.  This adapter owns the
    512/256 STFT clock, keeps the graphs resident on CPU, and returns exactly
    one output sample for every captured input sample after ``flush``.
    """

    def __init__(
        self,
        *,
        spec: SpectralModelSpec,
        model_id: str,
        model_revision: str,
        onnx_path: Path,
    ) -> None:
        self.spec = spec
        self.model_id = model_id
        self.model_revision = model_revision
        self.onnx_path = onnx_path.expanduser().resolve()
        periodic_hann = np.hanning(FFT_SAMPLES + 1)[:-1].astype(np.float32)
        self._window = (
            np.sqrt(periodic_hann).astype(np.float32)
            if spec.window == "sqrt_hann"
            else periodic_hann
        )
        self._window_power = self._window * self._window
        self._session: Any | None = None
        self.load_count = 0
        self._reset_stream_state()

    def load(self) -> dict[str, int | str]:
        if self._session is not None:
            return {"load_count": self.load_count, "elapsed_ms": 0}
        started = time.perf_counter_ns()
        try:
            import onnxruntime as ort

            options = ort.SessionOptions()
            options.intra_op_num_threads = 1
            options.inter_op_num_threads = 1
            self._session = ort.InferenceSession(
                str(self.onnx_path),
                sess_options=options,
                providers=["CPUExecutionProvider"],
            )
            self._validate_contract()
        except AdapterError:
            self._session = None
            raise
        except Exception as error:
            self._session = None
            raise AdapterError(
                f"cannot load pinned {self.spec.adapter_id} ONNX: {type(error).__name__}"
            ) from error
        self.load_count += 1
        return {
            "load_count": self.load_count,
            "elapsed_ms": round((time.perf_counter_ns() - started) / 1_000_000),
        }

    def warmup(self) -> dict[str, int]:
        self._require_loaded()
        started = time.perf_counter_ns()
        try:
            self.reset(self.spec.route)
            self.process_pcm(b"\x00\x00" * FFT_SAMPLES)
            self.flush()
            self.reset(self.spec.route)
        except Exception as error:
            raise AdapterError(
                f"{self.spec.adapter_id} warmup failed: {type(error).__name__}"
            ) from error
        return {"elapsed_ms": round((time.perf_counter_ns() - started) / 1_000_000)}

    def capabilities(self) -> dict[str, object]:
        return {
            "adapter_id": self.spec.adapter_id,
            "implementation_version": IMPLEMENTATION_VERSION,
            "model_id": self.model_id,
            "model_revision": self.model_revision,
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "streaming": True,
            "causal": True,
            "algorithmic_latency_ms": ALGORITHMIC_LATENCY_MS,
            "frame_ms": FFT_SAMPLES * 1_000 // SAMPLE_RATE_HZ,
            "hop_ms": HOP_SAMPLES * 1_000 // SAMPLE_RATE_HZ,
            "routes": ["asr"],
            "asr_audio_routes": [self.spec.route],
            "route_details": {
                self.spec.route: {
                    "adapter_id": self.spec.adapter_id,
                    "model_id": self.model_id,
                    "model_revision": self.model_revision,
                    "stages": ["stft", self.spec.route, "overlap_add"],
                    "algorithmic_latency_ms": ALGORITHMIC_LATENCY_MS,
                    "attenuation_limit_db": None,
                }
            },
            "affect_input": "raw_pcm",
            "vad_input": "raw_pcm",
            "runtime": "onnxruntime_cpu",
            "stft": {
                "n_fft": FFT_SAMPLES,
                "window_samples": FFT_SAMPLES,
                "hop_samples": HOP_SAMPLES,
                "window": self.spec.window,
                "center_padding": "causal_zero",
            },
        }

    def reset(self, route: str, *, noise_floor_dbfs: float | None = None) -> None:
        del noise_floor_dbfs
        if route != self.spec.route:
            raise AdapterError(f"unsupported {self.spec.adapter_id} ASR route: {route}")
        self._require_loaded()
        self._reset_stream_state()

    def process_pcm(self, pcm: bytes) -> bytes:
        if self._finished:
            raise AdapterError(f"{self.spec.adapter_id} stream was already flushed")
        if not pcm:
            return b""
        try:
            samples = pcm16le_to_float32(pcm)
            self._input_samples += len(samples)
            self._analysis = np.concatenate((self._analysis, samples))
            return float32_to_pcm16le(self._process_available())
        except (AdapterError, ValueError):
            raise
        except Exception as error:
            raise AdapterError(
                f"{self.spec.adapter_id} streaming inference failed: {type(error).__name__}"
            ) from error

    def flush(self) -> bytes:
        if self._finished:
            return b""
        self._finished = True
        pieces: list[np.ndarray] = []
        try:
            while self._output_samples < self._input_samples:
                if len(self._analysis) < FFT_SAMPLES:
                    self._analysis = np.pad(
                        self._analysis,
                        (0, FFT_SAMPLES - len(self._analysis)),
                    ).astype(np.float32, copy=False)
                output = self._process_frame()
                accepted = self._accept_output(output)
                if len(accepted):
                    pieces.append(accepted)
            result = np.concatenate(pieces) if pieces else np.zeros(0, dtype=np.float32)
            expected = self._input_samples - (self._output_samples - len(result))
            if len(result) != expected or self._output_samples != self._input_samples:
                raise AdapterError(f"{self.spec.adapter_id} did not preserve the sample clock")
            return float32_to_pcm16le(result)
        except (AdapterError, ValueError):
            raise
        except Exception as error:
            raise AdapterError(f"{self.spec.adapter_id} flush failed: {type(error).__name__}") from error

    def close(self) -> None:
        self._session = None

    def _reset_stream_state(self) -> None:
        self._analysis = np.zeros(HOP_SAMPLES, dtype=np.float32)
        self._overlap = np.zeros(FFT_SAMPLES, dtype=np.float32)
        self._normalization = np.zeros(FFT_SAMPLES, dtype=np.float32)
        self._states = {
            name: np.zeros(shape, dtype=np.float32) for name, shape in self.spec.state_inputs
        }
        self._discard_samples = HOP_SAMPLES
        self._input_samples = 0
        self._output_samples = 0
        self._finished = False

    def _process_available(self) -> np.ndarray:
        pieces: list[np.ndarray] = []
        while len(self._analysis) >= FFT_SAMPLES:
            accepted = self._accept_output(self._process_frame())
            if len(accepted):
                pieces.append(accepted)
        return np.concatenate(pieces) if pieces else np.zeros(0, dtype=np.float32)

    def _process_frame(self) -> np.ndarray:
        frame = self._analysis[:FFT_SAMPLES]
        self._analysis = self._analysis[HOP_SAMPLES:]
        spectrum = np.fft.rfft(frame * self._window, n=FFT_SAMPLES)
        mix = np.empty((1, FFT_SAMPLES // 2 + 1, 1, 2), dtype=np.float32)
        mix[0, :, 0, 0] = spectrum.real
        mix[0, :, 0, 1] = spectrum.imag
        inputs = {"mix": mix, **self._states}
        output_names = ["enh", *self.spec.state_outputs]
        values = self._require_loaded().run(output_names, inputs)
        if len(values) != len(output_names):
            raise AdapterError(f"{self.spec.adapter_id} returned an invalid output count")
        enhanced = np.asarray(values[0], dtype=np.float32)
        if enhanced.shape != mix.shape or not np.isfinite(enhanced).all():
            raise AdapterError(f"{self.spec.adapter_id} returned an invalid enhanced spectrum")
        next_states: dict[str, np.ndarray] = {}
        for (input_name, shape), value in zip(self.spec.state_inputs, values[1:], strict=True):
            state = np.asarray(value, dtype=np.float32)
            if state.shape != shape or not np.isfinite(state).all():
                raise AdapterError(f"{self.spec.adapter_id} returned invalid state: {input_name}")
            next_states[input_name] = state
        self._states = next_states
        complex_spectrum = enhanced[0, :, 0, 0] + 1j * enhanced[0, :, 0, 1]
        enhanced_frame = np.fft.irfft(complex_spectrum, n=FFT_SAMPLES).astype(np.float32)
        self._overlap += enhanced_frame * self._window
        self._normalization += self._window_power
        output = np.divide(
            self._overlap[:HOP_SAMPLES],
            self._normalization[:HOP_SAMPLES],
            out=np.zeros(HOP_SAMPLES, dtype=np.float32),
            where=self._normalization[:HOP_SAMPLES] > 1e-8,
        )
        self._overlap[:-HOP_SAMPLES] = self._overlap[HOP_SAMPLES:]
        self._overlap[-HOP_SAMPLES:] = 0.0
        self._normalization[:-HOP_SAMPLES] = self._normalization[HOP_SAMPLES:]
        self._normalization[-HOP_SAMPLES:] = 0.0
        return output

    def _accept_output(self, output: np.ndarray) -> np.ndarray:
        if self._discard_samples:
            discarded = min(self._discard_samples, len(output))
            output = output[discarded:]
            self._discard_samples -= discarded
        remaining = self._input_samples - self._output_samples
        accepted = output[: max(0, remaining)]
        self._output_samples += len(accepted)
        return accepted

    def _validate_contract(self) -> None:
        session = self._require_loaded()
        expected_inputs = {"mix": (1, 257, 1, 2), **dict(self.spec.state_inputs)}
        expected_outputs = {"enh": (1, 257, 1, 2)}
        expected_outputs.update(
            {
                output_name: shape
                for output_name, (_, shape) in zip(
                    self.spec.state_outputs, self.spec.state_inputs, strict=True
                )
            }
        )
        actual_inputs = {item.name: tuple(item.shape) for item in session.get_inputs()}
        actual_outputs = {item.name: tuple(item.shape) for item in session.get_outputs()}
        if actual_inputs != expected_inputs or actual_outputs != expected_outputs:
            raise AdapterError(f"{self.spec.adapter_id} ONNX tensor contract does not match")
        if any(item.type != "tensor(float)" for item in (*session.get_inputs(), *session.get_outputs())):
            raise AdapterError(f"{self.spec.adapter_id} ONNX tensors must be float32")

    def _require_loaded(self) -> Any:
        if self._session is None:
            raise AdapterError(f"{self.spec.adapter_id} adapter was used before load")
        return self._session


class CompositeAudioPreprocessor:
    """One façade over multiple resident route-owning preprocessors."""

    def __init__(self, processors: list[Any]) -> None:
        if not processors:
            raise ValueError("composite audio preprocessor requires at least one delegate")
        self._processors = tuple(processors)
        self._routes: dict[str, Any] = {}
        for processor in self._processors:
            routes = processor.capabilities().get("asr_audio_routes")
            if not isinstance(routes, list) or not routes:
                raise ValueError("audio preprocessor delegate must advertise routes")
            for route in routes:
                if not isinstance(route, str) or route in self._routes:
                    raise ValueError(f"duplicate or invalid audio preprocessor route: {route}")
                self._routes[route] = processor
        self._active: Any | None = None
        self.load_count = 0

    def load(self) -> dict[str, object]:
        started = time.perf_counter_ns()
        delegates = {
            processor.capabilities()["adapter_id"]: dict(processor.load())
            for processor in self._processors
        }
        self.load_count += 1
        return {
            "load_count": self.load_count,
            "elapsed_ms": round((time.perf_counter_ns() - started) / 1_000_000),
            "delegates": delegates,
        }

    def warmup(self) -> dict[str, object]:
        started = time.perf_counter_ns()
        delegates = {
            processor.capabilities()["adapter_id"]: dict(processor.warmup())
            for processor in self._processors
        }
        return {
            "elapsed_ms": round((time.perf_counter_ns() - started) / 1_000_000),
            "delegates": delegates,
        }

    def capabilities(self) -> dict[str, object]:
        delegates = [dict(processor.capabilities()) for processor in self._processors]
        route_details: dict[str, object] = {}
        for capabilities in delegates:
            details = capabilities.get("route_details")
            if isinstance(details, dict):
                route_details.update(details)
        return {
            "adapter_id": COMPOSITE_ADAPTER_ID,
            "implementation_version": IMPLEMENTATION_VERSION,
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "streaming": True,
            "causal": True,
            "routes": ["asr"],
            "asr_audio_routes": list(self._routes),
            "route_details": route_details,
            "models": {
                str(capabilities["adapter_id"]): capabilities for capabilities in delegates
            },
            "affect_input": "raw_pcm",
            "vad_input": "raw_pcm",
        }

    def reset(self, route: str, *, noise_floor_dbfs: float | None = None) -> None:
        processor = self._routes.get(route)
        if processor is None:
            raise AdapterError(f"unsupported composite ASR route: {route}")
        self._active = processor
        processor.reset(route, noise_floor_dbfs=noise_floor_dbfs)

    def process_pcm(self, pcm: bytes) -> bytes:
        if self._active is None:
            raise AdapterError("composite audio preprocessor was used before reset")
        return self._active.process_pcm(pcm)

    def flush(self) -> bytes:
        if self._active is None:
            raise AdapterError("composite audio preprocessor was used before reset")
        return self._active.flush()

    def close(self) -> None:
        self._active = None
        for processor in reversed(self._processors):
            processor.close()
