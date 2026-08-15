from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.projected_fixed_pd_inverse_dynamics_execution import (
    R133_ARRAY_SHAPES,
    _validate_profile,
    r135_target_shape_compatibility_audit,
    validate_r133_reconstruction_hashes,
)
from next_lab.projected_inverse_dynamics_composition import (
    verify_r133_array_hashes as frozen_r135_hash_guard,
)


def _hash(value: np.ndarray) -> str:
    return hashlib.sha256(np.ascontiguousarray(value).tobytes()).hexdigest()


class ProjectedFixedPdInverseDynamicsExecutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.arrays = {
            name: np.zeros(shape, dtype=np.float64)
            for name, shape in R133_ARRAY_SHAPES.items()
        }
        self.hashes = {name: _hash(value) for name, value in self.arrays.items()}

    def test_real_r133_shapes_and_hashes_pass_before_first_id_system(self) -> None:
        actual = validate_r133_reconstruction_hashes(
            arrays=self.arrays,
            expected_sha256=self.hashes,
            inverse_dynamics_system_assemblies=0,
        )
        self.assertEqual(actual, self.hashes)
        self.assertEqual(
            self.arrays["applied_target_microradians"].shape,
            (800, 23),
        )

    def test_hash_guard_rejects_one_scalar_change(self) -> None:
        mutated = {name: value.copy() for name, value in self.arrays.items()}
        mutated["applied_effort_micronewton_metres"][19, 7] = 1.0
        with self.assertRaisesRegex(ValueError, "applied_effort.*hash differs"):
            validate_r133_reconstruction_hashes(
                arrays=mutated,
                expected_sha256=self.hashes,
                inverse_dynamics_system_assemblies=0,
            )

    def test_hash_guard_cannot_run_after_id_assembly(self) -> None:
        with self.assertRaisesRegex(ValueError, "before inverse dynamics"):
            validate_r133_reconstruction_hashes(
                arrays=self.arrays,
                expected_sha256=self.hashes,
                inverse_dynamics_system_assemblies=1,
            )

    def test_r135_synthetic_target_shape_defect_is_reproduced_and_bounded(self) -> None:
        audit = r135_target_shape_compatibility_audit()
        self.assertEqual(audit["status"], "PASS_LOCAL_INPUT_GUARD_REPAIRED")
        self.assertEqual(audit["r135_synthetic_declared_shape"], [3200, 23])
        self.assertEqual(audit["r133_real_schedule_shape"], [800, 23])
        self.assertEqual(audit["dynamics_input_role"], "NONE_TARGET_IS_PROVENANCE_ONLY")
        with self.assertRaisesRegex(ValueError, "applied_target.*differs"):
            frozen_r135_hash_guard(self.arrays, self.hashes)

    def test_frozen_profile_contract_is_accepted(self) -> None:
        repository = Path(__file__).resolve().parents[2]
        profile = json.loads(
            (
                repository
                / "lab/profiles/humanoid-projected-fixed-pd-inverse-dynamics-r136.v1.json"
            ).read_bytes()
        )
        _validate_profile(profile)


if __name__ == "__main__":
    unittest.main()
