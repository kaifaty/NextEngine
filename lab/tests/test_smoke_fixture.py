import unittest

import numpy as np

from next_lab.smoke import (
    ENVIRONMENTS,
    PARITY_OBSERVATIONS,
    SEEDS,
    STEPS,
    generate_corpus,
    generate_parity_observations,
)


class SmokeFixtureTests(unittest.TestCase):
    def test_corpus_is_repeatable_and_bounded(self) -> None:
        first_observations, first_actions = generate_corpus()
        second_observations, second_actions = generate_corpus()
        expected_rows = len(SEEDS) * ENVIRONMENTS * STEPS
        self.assertEqual(first_observations.shape, (expected_rows, 6))
        self.assertEqual(first_actions.shape, (expected_rows, 2))
        np.testing.assert_array_equal(first_observations, second_observations)
        np.testing.assert_array_equal(first_actions, second_actions)
        self.assertTrue(np.isfinite(first_observations).all())
        self.assertTrue(np.isfinite(first_actions).all())

    def test_parity_inputs_are_repeatable(self) -> None:
        first = generate_parity_observations()
        second = generate_parity_observations()
        self.assertEqual(first.shape, (PARITY_OBSERVATIONS, 6))
        np.testing.assert_array_equal(first, second)


if __name__ == "__main__":
    unittest.main()
