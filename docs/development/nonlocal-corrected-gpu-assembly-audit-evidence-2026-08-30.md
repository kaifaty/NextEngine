# Nonlocal corrected GPU objective assembly audit — revision-1 evidence

| Field | Value |
| --- | --- |
| Research ID | `NCGA2` revision 1 |
| Result | `REFUTED / NAIVE_F32_PRESSURE_CANCELLATION` |
| Contract SHA-256 | `47c996a7ee40d38b422cee3512942c4c76f997ba3e09e4047a49dc260e5eb00c` |
| Product status | `REPORT_ONLY`; no solver, performance, runtime or product-water authority |
| Host | Linux x86-64, RTX 3080 (`sm_86`), CUDA compiler/runtime 13.3 |

## Outcome

The first valid Release execution stopped at the frozen positive gate. Five
isolated/boundary fixtures passed, including independent energy derivatives,
but both 100-sample pressure-bearing fixtures exceeded the predeclared strict
binary32 mixed bound. No tolerance, fixture, term or expected value was
changed.

The failure is localized to pressure accumulation rather than graph identity,
viscosity/surface formulas or the analytical host oracle:

- current/reference CSR and pressure-active flags are exact;
- density and energy remain well inside the bound;
- the independent energy-only oracle agrees with the analytical host gradient
  and Hessian directional products;
- dense CUDA Hessian and the separately accumulated CUDA HVP remain mutually
  consistent under the frozen bound; and
- all six deliberately wrong identities are rejected.

Revision 1 therefore grants no NCGA2 correspondence claim. The smallest next
experiment is a separately frozen strict-f32 compensated summation revision
with every input and tolerance unchanged.

## First mismatch

| Fixture/field | Host reference | CUDA candidate | Frozen normalized error | Gate |
| --- | ---: | ---: | ---: | --- |
| active pressure, gradient scalar 184 | `-3.2265856653168612e-16` | `2.8228759765625e-4` | `2.8228759765657266e-4` | FAIL `>2e-4` |
| active pressure, Hessian scalar 12340 | `7.7715611723760958e-16` | `3.4332275390625e-3` | `3.4332275390617228e-3` | FAIL |
| combined, gradient scalar 171 | `3.2629599657379704` | `3.2631607055664062` | `6.1520775781392326e-5` | PASS |
| combined, Hessian scalar 51472 | `20.023981996858438` | `20.028535842895508` | `2.2741960304317957e-4` | FAIL `>2e-4` |

The symmetric pressure fixture makes the mechanism observable: large pair and
center terms cancel analytically to approximately zero, while stable naive
binary32 addition leaves a milliscale Hessian residue. This is evidence for
roundoff cancellation, not yet proof that one specific compensation recurrence
will close the complete corpus.

## Passing boundaries

| Fixture | Maximum mixed error | First derivative error | Second derivative error | Result |
| --- | ---: | ---: | ---: | --- |
| isolated inertia | `3.74185e-7` | `1.86e-16` | `3.04e-16` | PASS |
| inactive pressure | `1.38996e-7` | `0` | `0` | PASS |
| reference/current support crossing | `4.18172e-7` | `3.25e-15` | `7.24e-8` | PASS |
| oblique viscosity | `2.23450e-6` | `3.02e-14` | `2.08e-14` | PASS |
| two surface branches | `4.76837e-7` | `1.22e-13` | `3.59e-8` | PASS |

The corrected and permuted combined fixtures have identical payload and work
roots. The negative controls rejected missing `2/h`, half viscosity,
owner-only pressure, pressure Gauss-Newton-only curvature, the stopped local
SISSM matrix and current-graph viscosity.

## Reproduction and identities

Two independent clean Release directories produced byte-identical stripped
binaries and byte-identical failing reports:

| Artifact | SHA-256 |
| --- | --- |
| Release executable A/B | `5e4c1826c055597a22dc9e3fcf54868beea344bf941b499185ac127f9cf0c020` |
| Failing report A/B | `9b000701b9282a4a8e8ff8846fad34676d26872e09497e72159608676d006353` |
| CMake target source | `04506e0f6d0a44516bb4aa36ae8c274e37256f2a985c8433548f3a5c068650c8` |
| DTO/header | `45f73373ad91bcd87024a690ee72c75708a9326b89edbe2227c85e5257f29ad0` |
| Host analytical oracle | `c91e93e98e4f681aad41d28cddabc5586cdfa9d66e8df9422099f3a0fdd9652d` |
| Host energy-only oracle | `7bcd31347d758cfcfe0482f43d6e54edf77131306fc4bdbe3153cd2d55d4f9b9` |
| CUDA candidate | `4df58a2199657bfeb0a27888fca240c12341226d2627db2c332c16e20d0939da` |
| Harness/fixtures | `80b41cd12021d79e877ac604e2237fb25c4aaf1671a404c2c9910605de971522` |

Fresh directories:

- `/tmp/nextengine-ncga2-r1-a-wmdd2K`;
- `/tmp/nextengine-ncga2-r1-b-NGyLH8`.

Both processes exited `1` as required for the failed mandatory positive. The
fixture, candidate payload-set and work-set roots in the report are
`a91fbb10...3c30f0d`, `8721205c...f858e` and `16de275f...7f968`.

Sanitizers and old regressions were not run after this positive gate failed;
the frozen stop rule makes them incapable of promoting revision 1.

## Decision

Close NCGA2 revision 1 as `REFUTED`. Do not widen the mixed/zero bounds or
weaken the symmetric fixture. A revision 2 may change only the declared
binary32 accumulation recurrence and its work receipt, must retain the naive
path as a rejected control, and must reproduce every revision-1 input and
independent oracle result exactly.

