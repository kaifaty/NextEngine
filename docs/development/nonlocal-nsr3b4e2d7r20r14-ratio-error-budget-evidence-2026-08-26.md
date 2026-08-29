# NSR3-B4E2D7R20R14 ratio error-budget evidence

Status: `PASS / CANDIDATE_REFINEMENT_RESOLVES_SUBSET`.

Implementation `efcd8e60` reproduces the complete R13 semantic root and emits:

```text
8707531a56963b5ce0a456e33b27d0b831a76061d354188b9a1232d9f959d30c
```

Both final principal inverse audits certify with `rho < 1`:

| case | current error | candidate old → refined | result |
|---|---:|---:|---|
| corner | `9.325e-4` | `2.943e-3 → 2.242e-23` | still ambiguous; pair bounds `6.688e-3`, `6.709e-3` |
| shear | `5.198e-8` | `2.427e-3 → 6.210e-22` | strict; best/competitor bounds `2.277e-6`, `4.853e-8` |

The arithmetic gamma components are about `1.3e-31`; input enclosures, not raw
binary128 rounding, dominate both original collisions. E1 and E3 are refuted.
E2 is supported: verified candidate refinement is sufficient for shear, while
the accumulated current-direction enclosure independently blocks corner.

No counterfactual state update occurred. The result supports a possible
on-demand candidate-inverse fallback, but does not authorize it; corner still
requires a separately justified current-error remedy.

