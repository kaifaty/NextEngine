# NSR3-B1D1 -- floor-limited nonlinear-sensitivity oracle

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / BOUNDARY_EXECUTION_BLOCKED`

Parent invalid diagnostic: `NSR3B1D_LEVEL_VALIDITY`, semantic SHA-256
`9efcae39dcb56868457ca2105796c8cbab661a7e62d5d33ede1f01dc36fadf28`.

Objective/solver identity remains `nuv-variational-fcr2` /
`nuv-newton-krylov-r0 + outer-state-hessian-tape-v1`.

## Question and one authorized oracle change

B1D's main ladder exhibits the frozen asymptotic trend, but its strict oracle
was invalid because it disabled the binary64 numerical-energy-floor stop.
B1D1 asks whether the ordinary `1e-8` scale stop materially contaminates the
D3--D4 temporal difference.

Repeat the B1D main ladder byte-exactly. Then repeat only D3 and D4 with:

```text
scaled_displacement_limit = 0
numerical_energy_floor_stop = enabled.
```

Raw-gradient convergence remains enabled. No formula, coefficient, geometry,
time step, trust policy, CG forcing, capacity or terminal time may change.
The oracle is expected to continue past the ordinary scale stop and terminate
only at raw-gradient or the already selected arithmetic floor.

## Exact parent overlap

All main levels must reproduce these phase-state hashes:

| Level | Final phase-state SHA-256 | Outer / accepted / HVP total |
|---|---|---:|
| D0 | `7f667eb41a86e1c840630ff8d81da5a258a759a6106442c2407f28ac5069028e` | `59 / 59 / 120` |
| D1 | `3611e20b08b5b400599b4a46edfa765ef1783d033456d9221bb505b1f3eda1b5` | `14 / 14 / 30` |
| D2 | `1ea70858651d4d548a49934be6cb3054945cc35af38b0d58e87807be33cf1bed` | `20 / 20 / 40` |
| D3 | `7078e43076b2209bd99fdb879e9294869223fe0792aceaa3b31b668620bf7442` | `24 / 24 / 48` |
| D4 | `3f320beae99be233c5675ff58656f38d6d533638e3104503410b10d7b0ed4c04` | `46 / 46 / 92` |

Rejected totals remain zero. Objective evaluations, pair builds, maximum
per-step work, pair/tape capacities and every published B1D observable must
also match its raw parent report.

## Sensitivity gates

Let `e_x=0.0022880058867231971 m` and
`e_v=0.04678768432527124 m/s` be the frozen D3--D4 temporal differences.
For normal versus floor-limited oracle states at both D3 and D4 require:

```text
s_x <= 0.1 e_x
s_v <= 0.1 e_v.
```

Both oracle runs must complete all steps under the B1D conservation,
density, work and capacity gates. At least one numerical-energy-floor stop
must occur across the pair; `MINIMUM_TRUST_RADIUS`, an outer limit or any
unrecognized stop invalidates the oracle.

## Decision

After exact overlap and oracle validity:

- select `TEMPORAL_STIFFNESS_CONFIRMED` when the unchanged B1D main ladder
  satisfies its frozen final-two ratio and scalar-change rule;
- otherwise select `NO_ASYMPTOTIC_REGIME_AT_C0P129`.

A diagnostic PASS requires one of those bounded answers and two byte-identical
reports. B1 and B1D remain negative/invalid historical results.

## Exit

`TEMPORAL_STIFFNESS_CONFIRMED` authorizes only B1S substep-policy design.
The other disposition requires integration reclosure. Neither authorizes B2,
hydrostatics, CUDA, runtime, public schemas, save/replay or production use.

