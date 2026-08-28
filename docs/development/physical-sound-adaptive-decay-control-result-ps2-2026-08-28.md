# Adaptive modal-decay synthetic control result — PS-2 — 2026-08-28

## Outcome

The frozen adaptive-decay control supports
`spatial-adaptive-rms-envelope-v1` on its delayed/noisy known-truth fixture.
Two independent runs emit byte-identical report
`9fabc2bdcd7d174aafb28d672d4f8e55df96817f8bcf02131f9f1fafb6b0297f`
with decision `AdaptiveDecaySyntheticControlSupported`. Every preregistered
check passes without changing implementation, fixture or threshold.

This proves only that the source-derived adaptive interval recovers known
post-excitation decay in the declared stress counterexample. It does not prove
that Ceramic contains the same delayed structure or grant observation,
quality, admission, mechanics or runtime credit.

## Exact result

- manifest `926921e2…cedf`;
- runner `fe59b3d6…9c20`;
- repeated preflight `f51e513a…7b49`;
- repeated run `9fabc2bd…0297f`;
- generated fixture `cd55a7c4…852b`, `46,080,000` bytes; and
- zero network, real-payload, physics and Planter counts.

| Measurement | Observed | Frozen criterion | Result |
| --- | ---: | ---: | --- |
| selected modes | `16` | `= 16` | pass |
| valid adaptive fits | `1.0` | `>= 0.75` | pass |
| median frequency error | `0.06444 cents` | `<= 40 cents` | pass |
| median adaptive decay error | `0.68081 dB/s` | `<= 3 dB/s` | pass |
| median fit `R²` | `0.999812` | `>= 0.95` | pass |
| fixed-window truth association | complete | required | pass |
| fixed-window median decay error | `58.95314 dB/s` | `>= 6 dB/s` | pass |
| adaptive error reduction | `58.27233 dB/s` | `>= 5 dB/s` | pass |

All mode-specific envelope peaks occur at `0.621..0.930 s`, while the fixed V2
fit ends at `0.900 s`. The control therefore directly discriminates a
per-mode post-peak fit from a universal early interval.

## Next action

Freeze a separate read-only Ceramic adaptive counterfactual before touching
the existing payload again. It must reuse rows `0..14`, unchanged frequency
selection and reference lineage; require valid dynamic ranges, negative decay,
fit linearity and improvement over the rejected `0.1875` fixed-window fraction;
and publish failure without tuning. No new data, mechanics or Planter access is
authorized.

