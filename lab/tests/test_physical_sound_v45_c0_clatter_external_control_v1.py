from __future__ import annotations

import copy
import hashlib
import json
import struct
import sys
import tempfile
import unittest
import wave
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v45_c0_clatter_external_control_v1 as c0  # noqa: E402

PROFILE = ROOT / c0.PROFILE_PATH
OWNER = ROOT / c0.OWNER_PATH
PROTOCOL = ROOT / c0.PROTOCOL_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_profile() -> dict[str, object]:
    return json.loads(PROFILE.read_text())


def payload(
    cf: list[float] | None = None,
    op: list[float] | None = None,
    rt: list[float] | None = None,
) -> bytes:
    cf = cf or [200.0 + 100.0 * index for index in range(10)]
    op = op or [-40.0 + index for index in range(10)]
    rt = rt or [0.1 + 0.01 * index for index in range(10)]
    return (
        struct.pack("<iii", len(cf), len(op), len(rt))
        + struct.pack(f"<{len(cf)}d", *cf)
        + struct.pack(f"<{len(op)}d", *op)
        + struct.pack(f"<{len(rt)}d", *rt)
    )


class ClatterExternalControlTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = load_profile()

    def test_preaccess_profile_is_canonical_and_binds_owner_protocol_t0(self) -> None:
        profile_bytes, profile = c0.read_canonical_json(PROFILE, "profile")
        c0.validate_profile(profile)
        self.assertEqual(profile_bytes, c0.canonical_json(profile))
        bindings = {item["path"]: item for item in profile["dependency_bindings"]}
        for path in (c0.OWNER_PATH, c0.PROTOCOL_PATH, c0.t0.PROFILE_PATH, c0.t0.OWNER_PATH):
            self.assertIn(path, bindings)
            data = (ROOT / path).read_bytes()
            self.assertEqual(bindings[path]["bytes"], len(data))
            self.assertEqual(bindings[path]["sha256"], sha256(data))
        self.assertEqual(bindings[c0.PROTOCOL_PATH]["sha256"], sha256(PROTOCOL.read_bytes()))
        self.assertEqual(bindings[c0.OWNER_PATH]["sha256"], sha256(OWNER.read_bytes()))

    def test_expected_inventory_is_exactly_fourteen_by_six(self) -> None:
        names = c0.expected_filenames()
        self.assertEqual(len(names), 84)
        self.assertEqual(names, sorted(set(names)))
        self.assertEqual(names[0], "cardboard_0_mm.bytes")
        self.assertEqual(names[-1], "wood_soft_5_mm.bytes")

    def test_artificial_payload_roundtrips_three_arrays(self) -> None:
        raw = payload()
        cf, op, rt = c0.decode_payload(raw, "artificial")
        self.assertEqual(cf, [200.0 + 100.0 * index for index in range(10)])
        self.assertEqual(op, [-40.0 + index for index in range(10)])
        self.assertEqual(rt, [0.1 + 0.01 * index for index in range(10)])

    def test_decoder_rejects_count_shape_trailing_and_nonfinite_corruption(self) -> None:
        with self.assertRaisesRegex(c0.ClatterControlError, "array counts differ"):
            c0.decode_payload(payload(op=[-40.0] * 11), "unequal")
        with self.assertRaisesRegex(c0.ClatterControlError, "byte count"):
            c0.decode_payload(payload() + b"x", "trailing")
        changed = [200.0] * 10
        changed[4] = float("nan")
        with self.assertRaisesRegex(c0.ClatterControlError, "non-finite"):
            c0.decode_payload(payload(cf=changed), "nan")
        changed = [0.1] * 10
        changed[4] = 0.0
        with self.assertRaisesRegex(c0.ClatterControlError, "not positive"):
            c0.decode_payload(payload(rt=changed), "zero")

    def test_artificial_prior_uses_only_empirical_modal_mask(self) -> None:
        cf, op, rt = c0.decode_payload(payload(), "artificial")
        recipe, target = c0.project_prior("glass_0_mm.bytes", cf, op, rt)
        self.assertEqual(recipe["source_lane"], "empirical_prior")
        self.assertEqual(target["observed_fields"], list(c0.OBSERVED_FIELDS))
        self.assertEqual(sum(recipe["heads"]["modal"]["participation_ppm"]), 1_000_000)
        self.assertEqual(
            c0.t0.masked_squared_error(c0.t0.flatten_recipe(recipe), target)["squared_error"],
            0,
        )
        self.assertTrue(all(target["mask"][offset] == 0 for offset in range(130, 189)))

    def test_seeded_renderer_is_byte_exact_and_canonical_wav(self) -> None:
        cf, op, rt = c0.decode_payload(payload(), "artificial")
        renderer = copy.deepcopy(self.profile["renderer"])
        renderer["render_frames"] = 512
        renderer["contact_pulse_samples"] = 8
        left = c0.render_control("glass_0_mm.bytes", cf, op, rt, renderer)
        right = c0.render_control("glass_0_mm.bytes", cf, op, rt, renderer)
        self.assertEqual(left, right)
        self.assertEqual(len(left), 44 + 2 * renderer["render_frames"])
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "control.wav"
            path.write_bytes(left)
            with wave.open(str(path), "rb") as reader:
                self.assertEqual(reader.getnchannels(), 1)
                self.assertEqual(reader.getsampwidth(), 2)
                self.assertEqual(reader.getframerate(), 48_000)
                self.assertEqual(reader.getnframes(), 512)
                samples = np.frombuffer(reader.readframes(512), dtype="<i2")
        self.assertGreater(int(np.max(np.abs(samples.astype(np.int32)))), 0)

    def test_profile_semantics_and_unknown_fields_fail_closed(self) -> None:
        changed = copy.deepcopy(self.profile)
        changed["renderer"]["root_seed"] += 1
        with self.assertRaisesRegex(c0.ClatterControlError, "renderer semantics"):
            c0.validate_profile(changed)
        changed = copy.deepcopy(self.profile)
        changed["unexpected"] = True
        with self.assertRaisesRegex(c0.ClatterControlError, "profile fields"):
            c0.validate_profile(changed)

    def test_output_and_source_must_remain_external(self) -> None:
        with self.assertRaisesRegex(c0.ClatterControlError, "outside"):
            c0.prepare_output(ROOT / "forbidden-v45-c0-output")
        with self.assertRaisesRegex(c0.ClatterControlError, "outside"):
            c0.validate_external_source(ROOT, self.profile)

    def test_owner_has_no_network_client_or_runtime_import(self) -> None:
        source = OWNER.read_text()
        for forbidden in (
            "import requests",
            "import urllib",
            "import httpx",
            "from requests",
            "from urllib",
            "from next_runtime",
        ):
            self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
