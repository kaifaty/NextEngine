from pathlib import Path
import unittest

from next_lab.isaac_profile import IsaacProfile


class IsaacProfileTests(unittest.TestCase):
    def test_stage0_profile_is_explicit_and_pinned(self) -> None:
        profile = IsaacProfile.load(
            Path(__file__).parents[1] / "profiles/isaac-lab-physx-stage0.v1.json"
        )
        self.assertEqual(profile.isaac_lab_version, "2.3.2")
        self.assertEqual(profile.isaac_lab_distribution_version, "0.54.2")
        self.assertEqual(profile.isaac_sim_version, "5.1.0")
        self.assertEqual(profile.isaac_sim_distribution_version, "5.1.0.0")
        self.assertEqual(profile.engine_physx_version, "5.9.0")
        self.assertEqual(profile.environments, 4_096)


if __name__ == "__main__":
    unittest.main()
