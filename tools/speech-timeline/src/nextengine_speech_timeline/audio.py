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
