# NSR3-B3D1 displacement-ownership evidence -- 2026-08-21

Status: `FAIL / REACTION_BELOW_ENERGY_RESOLUTION / NO_B3_RETRY`

The frozen
[B3D1 contract](../plans/nonlocal-nonlinear-solver-research/03b3d1-displacement-ownership-contract.md)
was executed twice byte-identically. The candidate does remove the diagnosed
world-position cancellation, but it cannot complete any fixed trajectory
under the unchanged reaction-aware energy-floor rule. B3 remains failed.

## What the candidate fixed

The report-only oracle now owns one transient displacement per fluid sample:

```text
delta = h*(v+h*g) + accepted trust corrections
position = x + delta
velocity = delta/h
```

Contact edits the same `delta`, and both position and velocity consistency are
bit-exact on every completed step. Storage is exactly `24N` bytes: `1800`
bytes for the 75-sample face fixture and `1080` bytes for the 45-sample corner
fixture.

On the completed prefixes, the displacement-based forward bound no longer
contains `|x|/h`:

| Fixture | substeps/frame | completed steps | cumulative `B_fp` | max inactive defect/bound |
|---|---:|---:|---:|---:|
| face | 96 | 168 | `2.90e-12` | `0` |
| face | 192 | 336 | `5.80e-12` | `0` |
| face | 384 | 676 | `1.17e-11` | `0` |
| corner | 96 | 332 | `3.39e-12` | `0.06295` |
| corner | 192 | 664 | `6.79e-12` | `0.06296` |
| corner | 384 | 1331 | `1.36e-11` | `0.06299` |

This is roughly four orders below the old world-position cumulative bounds at
the fine levels. It validates the arithmetic premise, not the whole candidate:
the trajectories terminate before their frozen final/correspondence gates.

## Exact stopping boundary

Every candidate stops at the first active state that needs a correction below
the inherited absolute energy floor:

| Fixture | substeps/frame | failing substep | reaction defect / limit | predicted decrease | floor |
|---|---:|---:|---:|---:|---:|
| face | 96 | 168 | `5.22e-11 / 4.01e-12` | `5.51e-21` | `2.27e-13` |
| face | 192 | 336 | `2.39e-12 / 2.01e-12` | `1.21e-23` | `2.27e-13` |
| face | 384 | 676 | `1.12e-12 / 1.02e-12` | `5.61e-24` | `2.27e-13` |
| corner | 96 | 332 | `5.66e-11 / 2.40e-12` | `1.25e-20` | `2.27e-13` |
| corner | 192 | 664 | `2.10e-12 / 1.20e-12` | `1.76e-23` | `2.27e-13` |
| corner | 384 | 1331 | `8.25e-13 / 6.06e-13` | `4.55e-24` | `2.27e-13` |

The predicted decreases are `1.8e7--5.0e10` times smaller than the frozen
floor. This is not permission to accept the state: reaction residual remains
`1.10x--23.6x` above its mixed limit at the failed steps. The exact first gate
is therefore `NSR3B3D1_DISPLACEMENT_CERTIFICATE` with disposition
`DISPLACEMENT_CERTIFICATE_REJECTED`.

The two inherited active captures still pass under displacement ownership.
The face replay uses `2/2` ordinary/reaction HVPs; the corner replay uses
`0/2`, reduces the reaction defect to `9.09e-14 kg m/s`, and changes state by
only `4.01e-9 dx` and `9.33e-8 c`.

## Architectural conclusion

Displacement ownership should be retained as a proven numerical
representation candidate, but it is not sufficient to authorize B3R. The new
blocker is the trust acceptance scalar, not boundary/contact semantics:
`current_total-trial_total` and the inherited `max(|E|,1)` floor cannot resolve
the tiny decrement still required by the impulse stationarity gate.

The next discriminator must evaluate the same objective change directly from
per-term differences, with an analytical binary64 error bound. It may not
weaken the reaction limit, alter the objective, or claim that an unresolved
energy decrement is positive.

## Repeatability and regression

```text
B3D1 raw SHA-256 (two identical runs):
f500187b0fb02c36d4383e2e9fe1003677afc5b2744c02b587fe9f9d86b8587a

B3D1 JSON-without-newline SHA-256:
6dede55270ca21c583ba83f5eebb740b0c0adf0e4b2a61cd00eb941906e4d259

B3D1 semantic SHA-256:
f9ed6a83170eff4c965b8c573eac0dd6f308beb41a6f7fcc365f8ad2b11a2ec7

B3D raw preserved:
66474768533403f41e7ba7becf3ca03ddcab0cd57a99edf16ed18017735aec35

B3 raw preserved:
c64ad0b8d73f7ada62364a3daa2d3bed66a8bfc5a1fe6c0148b7c1f8f2e5fb2f

B2 raw preserved:
d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9
```

No physical corpus, CUDA, performance, runtime or production authority is
granted.
