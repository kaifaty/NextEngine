from pathlib import Path
import hashlib
import unittest

from next_lab.motor_mirror import load_json, validate_biomechanics_descriptor
from next_lab.safety_contact_mirror import validate_safety_contact_golden


FIXTURES = Path(__file__).parent / "fixtures"
BIOMECHANICS = FIXTURES / "biomechanics_motor_mirror_v1.json"
SAFETY_CONTACT = FIXTURES / "biomechanics_safety_contact_mirror_v1.json"
SAFETY_CONTACT_SHA256 = (
    "dca80a314f0b23f2520b906e47bb0ed73fd2884474f46a12ee78d5fc422a5fe1"
)


class SafetyContactMirrorTests(unittest.TestCase):
    def test_rust_golden_matches_independent_python_safety_contact_and_done(self) -> None:
        self.assertEqual(
            hashlib.sha256(SAFETY_CONTACT.read_bytes()).hexdigest(),
            SAFETY_CONTACT_SHA256,
        )
        descriptor = load_json(BIOMECHANICS)
        validate_biomechanics_descriptor(descriptor)
        validate_safety_contact_golden(load_json(SAFETY_CONTACT), descriptor)

    def test_profile_or_body_remap_is_rejected(self) -> None:
        descriptor = load_json(BIOMECHANICS)
        golden = load_json(SAFETY_CONTACT)
        golden["body_schema_hash"] = "00" * 32
        with self.assertRaisesRegex(ValueError, "body_schema_hash"):
            validate_safety_contact_golden(golden, descriptor)


if __name__ == "__main__":
    unittest.main()
