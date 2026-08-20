# FCR3-B2 — pressure-activated Chebyshev remediation

Status: `FROZEN FOR IMPLEMENTATION / REPORT_ONLY`

Predecessor: FCR3-B1 `PASS / isolated-P`.

## Candidate

Keep the exact corrected SISSM v1 map and all FCR2 energy/gradient semantics.
If and only if `kappa>0`, transform its raw fixed-point output `f_k` with the
SISPH Chebyshev recurrence:

```text
spectral_radius = 0.9
omega_1 = 1
omega_2 = 2 / (2 - spectral_radius^2)
omega_(k+1) = 4 / (4 - spectral_radius^2 * omega_k)
y_candidate = omega_(k+1)*(f_k-y_(k-1)) + y_(k-1)
```

The first iteration uses the raw map. `y_(k-1)` is the previous accepted
iterate. The exact FCR2 gradient and Armijo rule evaluate
`y_candidate-y_k`; non-descent or exhausted search is a typed failure. There
is no silent fallback to v1, coefficient change, spectral-radius estimation
or parameter sweep.

When `kappa=0`, do not execute the recurrence. This preserves the passing
`V`, `S` and `VS` controls and keeps the remediation inside the FCR3-B1
selected scope.

## Corpus and gate

Run the unchanged FCR3-B cases and the `P, V, S, PV, PS, VS, PVS` masks with
the same 80-iteration cap.

For every case:

- finite, monotonic and internal momentum residual `<=1e-12`;
- physical direction preserved;
- final objective no worse than FCR2 by relative `1e-10`;
- final gradient no worse than `2x` FCR2 with a `1e-8` absolute floor;
- no non-descent direction or exhausted line search.

Every pressure-containing tetrahedron mask must pass the quality gate; the
non-pressure mask report fragments must equal v1 exactly. Iterations,
backtracks and objective evaluations are reported against v1 but do not gate
this pressure-quality remediation. In particular, the already observed
repulsive-surface backtracking cost cannot be repaired inside an
`isolated-P` change. Two complete reports must be byte-identical, and
FCR0-FCR2 must retain their frozen raw hashes.

Passing authorizes one separately frozen FCR3-B3 cost-localization stage;
profile reclosure remains blocked until the original aggregate objective
evaluation gate is closed. Failure closes the current fast-SISSM branch as
`FORMULA_RECLOSURE_STOP`; it does not authorize tuning `spectral_radius`, the
iteration cap, coefficients or tolerances.
