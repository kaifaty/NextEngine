# Nonlocal NSR3-B1D temporal-stiffness evidence -- 2026-08-20

Status: `FAIL / INVALID_DIAGNOSTIC / REPORT_ONLY`

## Outcome

The acoustic-Courant continuation produces a strong but non-gating temporal
trend. The diagnostic itself fails because its predeclared strict nonlinear
oracle disables the numerical-energy-floor stop; both strict replicas then
reach `MINIMUM_TRUST_RADIUS` at the double-precision energy floor. B1D cannot
select either temporal-stiffness disposition.

The result is preserved rather than repairing the oracle after observing it.
B1 remains FAIL and boundary work remains blocked.

## Main ladder

Every main-ladder level completes with finite state, zero rejected trials,
bounded work, zero final pressure-active centers and unchanged maximum density
ratio `1.0308294231753403`.

| Level | Courant | RMS radius | RMS speed | Kinetic energy | Pressure exit |
|---|---:|---:|---:|---:|---:|
| D0 | `2.0634` | `0.182830` | `0.433537` | `4.02927` | `0.003125 s` |
| D1 | `1.0317` | `0.186892` | `0.567984` | `6.91587` | `0.002083 s` |
| D2 | `0.5159` | `0.190679` | `0.693799` | `10.3191` | `0.001823 s` |
| D3 | `0.2579` | `0.193183` | `0.774956` | `12.8744` | `0.001823 s` |
| D4 | `0.1290` | `0.194635` | `0.821589` | `14.4705` | `0.001758 s` |

The D0 phase state and all frozen work counters exactly overlap B1.

Adjacent final-position errors are
`0.006711 / 0.006222 / 0.003991 / 0.002288 m`; final-velocity errors are
`0.135764 / 0.126563 / 0.081450 / 0.046788 m/s`. The final two
self-convergence ratios are:

```text
q_x = 1.558982 / 1.744335
q_v = 1.553864 / 1.740846.
```

Those ratios and the last radius/speed/kinetic changes satisfy the frozen
asymptotic-shape rule. They are evidence worth retaining, but the failed
solver-sensitivity control prevents publication as a valid conclusion.

## Invalid strict oracle

The strict replicas set scaled-displacement tolerance `1e-10` and disable the
numerical-floor stop exactly as frozen:

| Replica | Completed | Failure | Max outer / rejects / HVP |
|---|---:|---|---:|
| D3-strict | 2 / 384 steps | `MINIMUM_TRUST_RADIUS` | `27 / 22 / 54` |
| D4-strict | 7 / 768 steps | `MINIMUM_TRUST_RADIUS` | `24 / 21 / 48` |

Both remain finite and preserve rejected state, but cannot complete. Their
partial-state differences therefore cannot bound nonlinear stopping error.

This reproduces the arithmetic failure isolated in NSR2-C1/C2: once predicted
reduction is at the floating-point energy floor, trust-radius contraction
cannot manufacture a distinguishable actual reduction. Disabling the selected
floor stop was not a viable higher-accuracy oracle in binary64.

## Repeatability and lineage

- B1D semantic result SHA-256:
  `9efcae39dcb56868457ca2105796c8cbab661a7e62d5d33ede1f01dc36fadf28`;
- two byte-identical raw B1D reports:
  `fb5817bf75a31c09268998ec94345f5d55d1b5d8e60042bab99a17e3fbaa41c3`;
- B1 remains byte-identical at
  `c0f4a8362028323ea3281851c5a06a6cc96eb282e4bcec6e9460efa5920d917b`
  with unchanged semantic FAIL root;
- B0R and the eight frozen FCR1/NSR hashes remain required before any later
  selection.

## Decision

Preserve `NSR3B1D_LEVEL_VALIDITY`. Design B1D1 as a separately rooted
nonlinear-sensitivity oracle: suppress the early `1e-8` scaled-displacement
stop but retain the already selected numerical-energy-floor stop. It must
exactly overlap the B1D main ladder and show that extra floor-limited solves
are small relative to the D3--D4 temporal difference. No B1/B2 status changes.

