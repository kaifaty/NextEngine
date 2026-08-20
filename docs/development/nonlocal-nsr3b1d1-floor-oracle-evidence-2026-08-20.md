# Nonlocal NSR3-B1D1 floor-oracle evidence -- 2026-08-20

Status: `PASS / TEMPORAL_STIFFNESS_CONFIRMED / REPORT_ONLY`

## Outcome

B1D1 validates the B1D acoustic-Courant trend without disabling the arithmetic
floor. The five ordinary main levels overlap B1D exactly. Both floor-limited
oracle replicas complete, and their state changes are negligible relative to
the D3--D4 temporal difference. The B1 compression failure is therefore
attributed to underresolved temporal stiffness, not to the selected nonlinear
stopping tolerance.

This does not retroactively pass B1 and does not select a production time
step. It authorizes a separate substep-policy design.

## Exact main overlap

All five final phase-state hashes and every frozen work counter match B1D.
The unchanged main ladder retains:

```text
q_x = 1.558982416226432 / 1.744334569200635
q_v = 1.553864208427801 / 1.740846357077156.
```

The final radius, speed and kinetic-energy differences also decrease over the
last two refinements. Thus the parent asymptotic-shape rule is satisfied once
nonlinear sensitivity is independently bounded.

## Floor-limited sensitivity

The oracle suppresses only the early scale stop. The numerical-energy-floor
stop remains enabled and terminates all steps without trust-radius collapse.

| Oracle | Position difference | Velocity difference | Floor stops | Max outer / reject / HVP per step |
|---|---:|---:|---:|---:|
| D3-floor | `1.2651e-7 m` | `2.5049e-6 m/s` | 384 | `4 / 0 / 9` |
| D4-floor | `0` | `0` | 768 | `3 / 0 / 6` |

The frozen D3--D4 temporal differences are `0.0022880 m` and
`0.0467877 m/s`. D3 oracle sensitivity is only about `5.53e-5` and `5.35e-5`
of those values; D4 is bit-exact. Both are far below the predeclared `0.1`
limit.

The large counts of floor stops are expected because this diagnostic
deliberately disables ordinary scale convergence even on inactive later
steps. Its work is oracle overhead and cannot be used as a runtime strategy.

## Interpretation

For the B1 coefficient anchor,
`c=sqrt(K_eff/rho0)=99.0454 m/s`. The original B1 steps had acoustic Courant
`8.25 / 4.13 / 2.06` and did not resolve the transient. Self-convergence
appears once the continuation crosses approximately Courant one and is clear
at `0.516 / 0.258 / 0.129`.

The evidence supports using acoustic Courant as a substep-control variable.
It does not yet choose the accuracy target: `C<=1` begins convergence, while a
tighter value such as `C<=0.5` may be needed for a robust policy. That choice
must be frozen and tested against more than this single relaxation fixture.

## Repeatability and lineage

- B1D1 semantic result SHA-256:
  `64592cd38c3371decd14baee6517e85bd0407e9654255e2eaab03fba8fd770c1`;
- two byte-identical raw reports:
  `2a9ddd605cc1ec89a8cb71fd24c239d08416ab398d09dfa3c37c80a9913adcc1`;
- B1D remains byte-identical INVALID at
  `fb5817bf75a31c09268998ec94345f5d55d1b5d8e60042bab99a17e3fbaa41c3`;
- B1 remains byte-identical FAIL at
  `c0f4a8362028323ea3281851c5a06a6cc96eb282e4bcec6e9460efa5920d917b`.

## Decision

Select `TEMPORAL_STIFFNESS_CONFIRMED` as a research disposition. Freeze B1S to
derive and test an acoustic-Courant substep policy over multiple manufactured
amplitudes/coefficient scales. B2 boundaries, CUDA and runtime integration
remain blocked.

