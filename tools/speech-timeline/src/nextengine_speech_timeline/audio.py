from __future__ import annotations

import array
import sys

import numpy as np


def pcm16le_to_float32_array(data: bytes) -> array.array:
    if not data or len(data) % 2:
        raise ValueError("PCM must contain aligned signed 16-bit samples")
    pcm = array.array("h")
    pcm.frombytes(data)
    if sys.byteorder == "big":
        pcm.byteswap()
    return array.array("f", (sample / 32768.0 for sample in pcm))


def pcm16le_to_float32(data: bytes) -> np.ndarray:
    samples = pcm16le_to_float32_array(data)
    return np.frombuffer(samples, dtype=np.float32).copy()


def float32_to_pcm16le(samples: np.ndarray) -> bytes:
    """Encode finite mono float samples as canonical little-endian PCM16.

    This is deliberately a bounded conversion at the model seam: an optional
    audio preprocessor cannot hand invalid values or a platform-native endian
    representation to ASR.
    """
    values = np.asarray(samples, dtype=np.float32)
    if values.ndim != 1:
        raise ValueError("audio samples must be a one-dimensional mono array")
    if not np.isfinite(values).all():
        raise ValueError("audio samples must be finite")
    pcm = np.clip(np.rint(values * 32768.0), -32768, 32767).astype("<i2", copy=False)
    return pcm.tobytes()
