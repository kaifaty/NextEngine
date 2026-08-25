# NSR3-B4E2D7R20R11 v3 generalization evidence

Status: `REFUTED / V3_GENERALIZATION_SOLVER_REJECTED` on the frozen v3 finite
profile.

Clean implementation `8104f1b2` executed the frozen command once:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v3-solver
```

The command exited `1` with semantic result:

```text
d24dd8da48a1e1d9c15b91bc67241fff22e504fad4c01ef4fe82f56e13399b35
```

Parent preflight `c2328bfb...b577`, all four exact problem roots, the frozen
binary128 profile, controls and workspace lifecycle pass. The result is
therefore a solver counterexample, not an input, platform or harness failure.

## Decisive result

| holdout | result | accepted | mask changes | principal solves | terminal primal |
|---|---|---:|---:|---:|---:|
| lower-x/upper-z edge | certified | 11 | 30 | 850 | `9.060e-33` |
| lower-x/lower-y/upper-z corner | active-set rejected | 6 | 59 | 531 | `2.699e-4` |
| upper-y shear layer | active-set rejected | 10 | 58 | 810 | `4.015e-5` |
| radial compression | certified | 11 | 91 | 646 | `2.179e-33` |

The two certified cases satisfy the complete `2^-70` KKT tuple. The rejected
cases did make substantial monotone progress, but neither is a near-success:
their terminal primal/dual-mapping residuals remain many orders of magnitude
above the certificate threshold. Neither failure performed a verified-inverse
audit, so the previously repaired passive-solve enclosure is not yet implicated.

## Claim ledger

| claim | status | evidence | ceiling |
|---|---|---|---|
| unchanged R8 certifies all four frozen v3 holdouts within 32 accepted steps | `REFUTED` | deterministic binary128 numerical/correspondence evidence on four hash-bound problems | says nothing universal about other inputs or a revised method |
| R8 can certify previously unseen v3 topologies | `SUPPORTED_BOUNDED` | two of four blind cases certify strictly | not production readiness or a success rate estimate |

No parameters changed after observation, no development solver was rerun by
the command, and no timing was admitted. These cases are no longer blind and
may only be reused as named development counterexamples. A fresh future
generalization claim requires a new source-only holdout family.

The next bounded step is failure-mechanism localization, not a solver change.

