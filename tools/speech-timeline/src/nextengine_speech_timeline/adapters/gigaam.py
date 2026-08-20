from __future__ import annotations

from collections.abc import Callable, Sequence
import gc
import importlib.util
import json
from pathlib import Path
import sys
import time
from types import ModuleType
from typing import Any, Protocol

import numpy as np

from ..capabilities import ModelLoadEvidence, TranscriberCapabilities, WarmupEvidence
from .base import AdapterError, TranscriberConfig, TranscriptRevision


ADAPTER_ID = "gigaam-pytorch/1"
MODEL_ROUTE_ID = "gigaam-v3-e2e-rnnt"
SAMPLE_RATE_HZ = 16_000
MAX_AUDIO_DURATION_MS = 25_000
MAX_AUDIO_SAMPLES = SAMPLE_RATE_HZ * MAX_AUDIO_DURATION_MS // 1_000
_REMOTE_MODULE_NAME = "modeling_gigaam"


class _Backend(Protocol):
    device: str

    def transcribe(self, samples: np.ndarray) -> str: ...

    def close(self) -> None: ...


class _PinnedGigaAmBackend:
    def __init__(self, model: Any, torch_module: ModuleType, device: str) -> None:
        self._model = model
        self._torch = torch_module
        self.device = device

    def transcribe(self, samples: np.ndarray) -> str:
        if samples.ndim != 1 or not 0 < samples.size <= MAX_AUDIO_SAMPLES:
            raise AdapterError("GigaAM input must contain between 1 sample and 25 seconds")
        torch = self._torch
        waveform = torch.from_numpy(np.ascontiguousarray(samples, dtype=np.float32))
        waveform = waveform.to(self.device).unsqueeze(0)
        length = torch.full(
            [1],
            waveform.shape[-1],
            dtype=torch.long,
            device=self.device,
        )
        try:
            with torch.inference_mode():
                encoded, encoded_length = self._model.forward(waveform, length)
                text = self._model.decoding.decode(
                    self._model.head,
                    encoded,
                    encoded_length,
                )[0]
        except Exception as error:
            raise AdapterError(
                f"GigaAM inference failed: {type(error).__name__}: {error}"
            ) from error
        return str(text).strip()

    def close(self) -> None:
        model = self._model
        self._model = None
        del model
        gc.collect()
        if self.device.startswith("cuda") and self._torch.cuda.is_available():
            self._torch.cuda.empty_cache()


def _load_pinned_backend(snapshot: Path, device: str) -> _Backend:
    try:
        import torch
        from hydra.utils import instantiate
        from omegaconf import OmegaConf
    except Exception as error:
        raise AdapterError(f"GigaAM runtime dependency is unavailable: {error}") from error

    resolved_device = device
    if device == "auto":
        resolved_device = "cuda" if torch.cuda.is_available() else "cpu"
    if resolved_device == "cuda" and not torch.cuda.is_available():
        raise AdapterError("GigaAM CUDA was requested but is unavailable")

    code_path = snapshot / "modeling_gigaam.py"
    previous = sys.modules.get(_REMOTE_MODULE_NAME)
    if previous is not None:
        previous_path = Path(str(getattr(previous, "__file__", ""))).resolve()
        if previous_path != code_path.resolve():
            raise AdapterError("another GigaAM remote-code revision is already loaded")
        module = previous
    else:
        specification = importlib.util.spec_from_file_location(_REMOTE_MODULE_NAME, code_path)
        if specification is None or specification.loader is None:
            raise AdapterError("cannot load pinned GigaAM model code")
        module = importlib.util.module_from_spec(specification)
        sys.modules[_REMOTE_MODULE_NAME] = module
        try:
            specification.loader.exec_module(module)
        except Exception as error:
            sys.modules.pop(_REMOTE_MODULE_NAME, None)
            raise AdapterError(
                f"failed to load pinned GigaAM model code: {type(error).__name__}: {error}"
            ) from error

    try:
        raw_config = json.loads((snapshot / "config.json").read_text(encoding="utf-8"))
        model_config = raw_config["cfg"]["model"]
        model_config["cfg"]["decoding"]["model_path"] = str(snapshot / "tokenizer.model")
        model = instantiate(OmegaConf.create(model_config), _recursive_=False)
        state = torch.load(
            snapshot / "pytorch_model.bin",
            map_location="cpu",
            weights_only=True,
        )
        model.load_state_dict({key.removeprefix("model."): value for key, value in state.items()})
        model.eval().to(resolved_device)
    except Exception as error:
        raise AdapterError(
            f"failed to construct pinned GigaAM model: {type(error).__name__}: {error}"
        ) from error
    return _PinnedGigaAmBackend(model, torch, str(resolved_device))


class GigaAmTranscriberAdapter:
    """Resident finalized-utterance adapter for pinned GigaAM-v3 e2e RNNT."""

    def __init__(
        self,
        snapshot: Path,
        *,
        model_id: str,
        model_revision: str,
        device: str = "cuda",
        loader: Callable[[Path, str], _Backend] = _load_pinned_backend,
    ) -> None:
        self.snapshot = snapshot.expanduser().resolve()
        self.model_id = model_id
        self.model_revision = model_revision
        self.requested_device = device
        self._loader = loader
        self._backend: _Backend | None = None
        self._active = False
        self._load_count = 0
        self._warmup_count = 0

    @property
    def load_count(self) -> int:
        return self._load_count

    def load(self) -> ModelLoadEvidence:
        if self._backend is not None:
            return ModelLoadEvidence(elapsed_ms=0, load_count=self._load_count)
        started = time.perf_counter()
        self._backend = self._loader(self.snapshot, self.requested_device)
        self._load_count += 1
        return ModelLoadEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1_000),
            load_count=self._load_count,
        )

    def warmup(self) -> WarmupEvidence:
        started = time.perf_counter()
        self.load()
        self._warmup_count += 1
        return WarmupEvidence(
            elapsed_ms=round((time.perf_counter() - started) * 1_000),
            warmup_count=self._warmup_count,
        )

    def capabilities(self) -> TranscriberCapabilities:
        if self._backend is None:
            raise AdapterError("GigaAM model is not loaded")
        return TranscriberCapabilities(
            adapter_id=ADAPTER_ID,
            model_id=self.model_id,
            runtime_id="pytorch-pinned-remote-code",
            backend=self._backend.device,
            sample_rate_hz=SAMPLE_RATE_HZ,
            encoding="pcm_f32",
            channels=1,
            supports_streaming=False,
            timing_precision="utterance",
            resolution_samples=None,
            supported_delay_ms=(),
            configured_delay_ms=0,
            partial_decode_interval_ms=0,
            max_audio_duration_ms=MAX_AUDIO_DURATION_MS,
        )

    def start(self, config: TranscriberConfig | None = None) -> GigaAmTranscriberSession:
        self.load()
        language = (config or TranscriberConfig()).language
        if language is not None and language.lower().replace("_", "-") not in {
            "ru",
            "ru-ru",
        }:
            raise AdapterError("the selected GigaAM-v3 profile accepts Russian audio only")
        if self._active:
            raise AdapterError("GigaAM adapter already owns an active session")
        self._active = True
        return GigaAmTranscriberSession(self)

    def _session_closed(self) -> None:
        self._active = False

    def close(self) -> None:
        if self._active:
            raise AdapterError("cannot close GigaAM with an active session")
        if self._backend is not None:
            self._backend.close()
        self._backend = None


class GigaAmTranscriberSession:
    def __init__(self, adapter: GigaAmTranscriberAdapter) -> None:
        self._adapter = adapter
        self._chunks: list[np.ndarray] = []
        self._samples = 0
        self._closed = False
        self._finished = False
        self._final_revision: TranscriptRevision | None = None

    def push_pcm(self, samples: Sequence[float]) -> None:
        if self._closed or self._finished:
            raise AdapterError("GigaAM session is not active")
        chunk = np.asarray(samples, dtype=np.float32)
        if chunk.ndim != 1 or chunk.size == 0 or not np.isfinite(chunk).all():
            raise AdapterError("GigaAM received invalid mono PCM")
        if self._samples + chunk.size > MAX_AUDIO_SAMPLES:
            raise AdapterError("GigaAM utterance exceeds the 25 second model limit")
        self._chunks.append(chunk.copy())
        self._samples += int(chunk.size)
        return None

    def finish(self) -> TranscriptRevision:
        if self._closed and self._final_revision is None:
            raise AdapterError("GigaAM session was cancelled")
        if self._final_revision is not None:
            return self._final_revision
        if not self._chunks:
            raise AdapterError("GigaAM cannot finalize an empty utterance")
        assert self._adapter._backend is not None
        samples = np.concatenate(self._chunks)
        text = self._adapter._backend.transcribe(samples)
        self._finished = True
        self._final_revision = TranscriptRevision(
            revision=1,
            full_text=text,
            committed_text=text,
            tentative_text="",
            final=True,
        )
        return self._final_revision

    def cancel(self) -> None:
        self.close()

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        self._chunks.clear()
        self._adapter._session_closed()
