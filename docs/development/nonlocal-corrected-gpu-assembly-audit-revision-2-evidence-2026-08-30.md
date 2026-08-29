# Nonlocal corrected GPU objective assembly audit — revision-2 evidence

| Field | Value |
| --- | --- |
| Research ID | `NCGA2` revision 2 |
| Result | `REFUTED / COMPENSATED_F32_INSUFFICIENT` |
| Contract SHA-256 | `14690d3bfeb71b36742bdc0f7c1c6532c801318a70d4e353c34dac5f41424a20` |
| Parent result | Revision 1 `REFUTED / NAIVE_F32_PRESSURE_CANCELLATION` |
| Product status | `REPORT_ONLY`; no solver, performance, runtime or product-water authority |
| Host | Linux x86-64, RTX 3080 (`sm_86`), CUDA compiler/runtime 13.3 |

## Outcome

The frozen Kahan-style binary32 recurrence repairs the symmetric dense-pressure
case but does not close the unchanged combined pressure/viscosity/surface
fixture. Revision 2 stops at the first positive mismatch with every fixture,
formula, oracle, product, branch, output precision and tolerance unchanged.

This discriminates the cause more sharply:

- naive summation was a real contributor — the active-pressure maximum Hessian
  error fell from `3.4332275e-3` to `4.5776367e-5` and now passes;
- stable compensation is not sufficient — the combined Hessian error is
  `2.6247279e-4` against the frozen `2e-4` mixed bound; and
- the remaining discrepancy therefore enters before or outside the final
  reduction, through strict-binary32 product/normal/radial-coefficient
  evaluation or their composition. This is a bounded inference, not a proof of
  which individual product dominates.

NCGA2 is `REFUTED`. The contract authorizes no third accumulation identity,
tolerance change or full solver port.

## Revision-1 versus revision-2 discriminator

| Fixture/field | Naive f32 | Compensated f32 | Frozen gate | Result |
| --- | ---: | ---: | ---: | --- |
| active pressure maximum gradient mixed error | `2.82288e-4` | `1.38872e-6` | `<=2e-4` | repaired |
| active pressure maximum Hessian mixed error | `3.43323e-3` | `4.57764e-5` | `<=2e-4` | repaired |
| active pressure Hessian zero residue | `3.43323e-3` | `4.57764e-5` | `<=2e-5` for exact zero, main maximum is a near-zero reference | bounded positive still passes the common comparator |
| combined maximum gradient mixed error | `6.15208e-5` | `1.62916e-5` | `<=2e-4` | improved/pass |
| combined maximum Hessian mixed error | `2.27420e-4` | `2.62473e-4` | `<=2e-4` | FAIL |
| combined direct-HVP consistency | `1.12831e-4` | `9.05657e-6` | `<=2e-4` | improved/pass |

The failing combined scalar is dense Hessian index `51472`:

```text
host       = 20.023981996858438
candidate  = 20.029237747192383
mixed error = 0.00026247278562120636
```

The analytical host Hessian remains independently supported by the energy-only
second directional derivative (`5.90299e-7` relative error), far inside the
unchanged `2e-5` host gate. Candidate dense-Hessian/HVP consistency also passes,
so accepting the candidate by relabelling the oracle or dropping the dense
matrix would erase the discriminating evidence.

## Gates reached before stop

- All current/reference graphs and pressure-active flags are exact.
- Isolated inertia, inactive pressure, support crossing, oblique viscosity,
  both surface branches and active pressure pass.
- Combined and its input permutation fail at the same scalar/error; their
  payload and work roots remain exact under permutation.
- Host first/second energy derivatives pass in all eight fixtures.
- All seven negatives reject: the six revision-1 wrong identities plus exact
  naive-f32 pressure.
- Admission controls and ten cold executions are exact inside both fresh
  processes.

Sanitizers and NCGA0/NCGA1/historical regressions were not run after the
unchanged positive gate failed. They cannot promote a numerically refuted
candidate and the ordered contract stops before those costs.

## Exact artifacts

Two independent clean Release builds produced byte-identical stripped binaries
and byte-identical failing reports:

| Artifact | SHA-256 |
| --- | --- |
| Release executable A/B | `913c7424f7f73cab2b1534290c2879f281620acbef482ecbae7fa406745d5e73` |
| Failing report A/B | `5ac5b3e7990ac5cadd2b711119fac8bdc3ae27c2d1b1bafeb7296fe587377878` |
| Fixture root | `a91fbb102bfe51f3827af70084a7e279b689f62b099b096417c52a3573c30f0d` |
| Candidate payload-set root | `97f11b13640032116df73cf5c2973d3956663f0188d791f77cbd576b1f3a36a7` |
| Candidate work-set root | `84ad025671f097595b003bf5b4f74d1be06297f476646df3360ad1b6cac576b9` |
| CMake target source | `04506e0f6d0a44516bb4aa36ae8c274e37256f2a985c8433548f3a5c068650c8` |
| DTO/header | `c461461061e99ecb215f2854a861115d54fdb52c8e5275394cc20f1af85b7ee8` |
| Host analytical oracle | `c91e93e98e4f681aad41d28cddabc5586cdfa9d66e8df9422099f3a0fdd9652d` |
| Host energy-only oracle | `7bcd31347d758cfcfe0482f43d6e54edf77131306fc4bdbe3153cd2d55d4f9b9` |
| CUDA candidate | `6e0b3606cf655beeba73495f921329b0c600c5551bb181caf0291ff575af8493` |
| Harness/fixtures | `cfe798141287b84786115bb447c6a76624a1fb0c88f1b259c2a917bda7f40a96` |

Fresh directories:

- `/tmp/nextengine-ncga2-r2-a-ZdXB7t`;
- `/tmp/nextengine-ncga2-r2-b-VzsMXL`.

Both processes exited `1`, preserving fail-closed terminal behavior.

## Decision and next boundary

Close NCGA2 revision 2 and the current strict-f32 assembly question as
`REFUTED`. Do not begin a full corrected GPU solver from this candidate.

A future GPU experiment needs a genuinely new numerical question, not another
summation tweak: for example, a frozen product-error decomposition followed by
either mixed-precision coefficients/products, strict binary64 assembly, or a
solver-level error budget derived independently from trajectory/physics
acceptance. None is authorized here. The fast exact neighborhood result remains
valid and separate.

