# Nonlocal corrected GPU objective assembly audit — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / NCGA3_AUTHOR_SUPPORTED_BOUNDED / REVIEW_NOT_RUN / NCGA2_IMMUTABLE` |
| Updated | `2026-08-30` |
| Task key | `nonlocal-corrected-gpu-assembly-audit` |
| Scope | Determine whether the tiny strict-f32 corrected Nonlocal Hessian mismatch materially changes bounded local GPU operator/step behavior |
| Definition of done | NCGA3 produces one hash-closed author result without changing NCGA2, or stops at the first invalid consequence boundary |
| Authority | Working context only; SPEC-38, ADR-076/081, FCR0 and the frozen NCGA2 contract outrank this file |

## Resume in 60 seconds

- **Current state:** NCGA3 revision 2 is hash-closed
  `AUTHOR_SUPPORTED_BOUNDED / F32_CONSEQUENCE_NEGLIGIBLE_BOUNDED`. NCGA2 remains
  honestly refuted at `2.6247e-4`, but strict f32 reaches only `1.0643e-6` HVP
  and `2.0004e-6` regularized-step relative error; eight integer steps end with
  zero micrometre drift and identical objective at published precision.
- **Decision:** NCGA2 assembles objective energy, analytical gradient, exact
  dense Hessian/diagonal blocks and HVP on tiny immutable graphs. It does not
  port the stopped SISSM local matrix.
- **Why:** FCR3-B2 already rejected the pressure-bearing SISSM/Chebyshev
  recurrence. The nonlinear objective and its derivatives remain the valid
  mathematical boundary for a future separately selected solver.
- **Next action:** if authorized, freeze a corrected nonlinear/operator solver
  experiment that starts with strict f32 and keeps f64 pressure products as a
  named fallback. Request independent review before treating NCGA3 as reviewed
  evidence.
- **Current blocker:** no corrected solver, physical trajectory or full
  assembly/solve timing exists; NCGA3 independent review is `NOT_RUN`.
- **Revision-1 discriminator:** all HVP probes completed and strict f32 reached
  only `1.0643e-6` maximum relative L2 error; reduction-only stayed outside the
  old element gate while f64 pressure products reached `4.7195e-5`. The fixed
  `H+7200I` reference system was not positive definite, so step/sequence fields
  were invalid and the run is `INCONCLUSIVE`.
- **Single apparatus repair:** revision 2 uses the reference symmetric
  infinity-norm bound plus `7200` as one common, guaranteed-positive shift.
  No threshold, fixture, arithmetic result or second repair is authorized.
- **Claim ceiling:** tiny objective assembly correspondence only; no solve,
  trajectory, performance, runtime or product-water claim.

## Competing hypotheses

| Hypothesis | Prediction | Discriminator | Status |
| --- | --- | --- | --- |
| H1 corrected GPU assembly corresponds | graphs, density, energies, gradient and Hessian pass the frozen mixed bounds | independent long-double oracle plus energy-only derivatives | refuted for frozen strict f32 |
| H2 historical mismatch was in accumulation/composition | isolated NCGA0 terms pass but active-pressure or combined assembly differs | active pressure and combined clusters | supported for naive f32 |
| H3 a wrong matrix can hide behind matching forces | gradient passes while exact Hessian or second derivative fails | dense matrix, HVP and Gauss-Newton/SISSM negatives | supported as a real audit risk |
| H4 reference/current graph roles are mixed | support-crossing viscosity or current terms differ | `reference_current_support_crossing` | refuted on frozen fixture |
| H5 compensated strict-f32 closes cancellation | exact same corpus passes without changing bounds | frozen Kahan-style recurrence plus naive control | refuted; symmetric closes, combined does not |
| H6 the remaining element miss changes local behavior materially | HVP, regularized step or short sequence exceeds the product screen | thirteen probes, common norm-bounded solve and eight steps | refuted on the frozen fixture |
| H7 binary64 reduction alone closes the miss | f32 products accumulated in f64 meet the old matrix gate | reduction-only arithmetic variant | refuted: `2.59425e-4` |
| H8 binary64 pressure products close the miss | promoted pressure coefficients/products meet the old gate | mixed-product arithmetic variant | supported: `4.71951e-5`, but not selected yet |

## Decisions

### D-001 — Do not port the stopped SISSM split

- **Observation:** FCR3-B2 closed the existing SISSM/Chebyshev lineage after
  reproducible pressure non-descent/quality failures.
- **Evidence:** `docs/development/nonlocal-continuum-fcr3b2-chebyshev-evidence-2026-08-20.md`.
- **Conclusion:** CPU/GPU equality for that local split would not select a
  corrected solver.
- **Decision:** assemble derivatives of `nuv-variational-fcr1` directly and
  include the historical SISSM matrix only as a mandatory rejected identity.
- **Rejected alternatives:** port old `source/local_matrix`, or jump directly
  to a full GPU solve.
- **Consequences:** NCGA2 can feed a future matrix-free operator audit but
  cannot claim a solver step.
- **Remaining uncertainty:** whether strict-f32 active-pressure Hessian
  composition stays inside the frozen bound.
- **Reconsider when:** a separately reviewed solver chooses a different
  derivative/operator identity.

### D-002 — Keep product and research profiles separate

- **Observation:** SPEC-38 V1 selects CPU DFSPH with `0.1 m` support and no
  viscosity/surface model; NCGA0/FCR uses the Nonlocal `0.15 m` audit profile.
- **Decision:** NCGA2 stays on the latter and names it report-only. It does not
  inherit SPEC-38 product authority from NCGA1's exact integer cache test.
- **Consequence:** successful assembly is useful GPU-port evidence, not a claim
  that the V1 game water profile is implemented.

### D-003 — Preserve the naive-f32 pressure failure

- **Observation:** the first dense pressure fixture leaves `2.82e-4` gradient
  and `3.43e-3` Hessian residues where the host values are approximately zero;
  combined Hessian error is `2.274e-4` against the `2e-4` bound.
- **Evidence:**
  `docs/development/nonlocal-corrected-gpu-assembly-audit-evidence-2026-08-30.md`.
- **Conclusion:** revision 1 is refuted; graph/formula boundaries outside dense
  pressure remain supported only as local diagnostics.
- **Decision:** no tolerance/fixture change. The only admissible next
  discriminator is an exactly frozen compensated-binary32 accumulation path
  with naive f32 as a negative.
- **Remaining uncertainty:** whether compensation closes every gradient,
  Hessian and direct-HVP reduction, or strict f32 remains insufficient.

### D-004 — Freeze compensated f32 without changing the question

- **Observation:** cancellation, not graph or isolated term translation, is the
  first surviving explanation.
- **Decision:** revision 2 changes only the exact binary32 addition recurrence
  and work receipt. Profile, fixture bytes, bounds, products and oracles remain
  immutable; naive revision 1 becomes a common-comparator negative.
- **Rejected alternatives:** tolerance widening, deleting the symmetric case,
  using device binary64, switching to a new physical profile, or measuring a
  solver before assembly closes.
- **Reconsider when:** revision 2 fails an unchanged gate or independent review
  finds shared/hidden work.

### D-005 — Stop strict-f32 assembly after compensation fails

- **Observation:** compensation reduces the symmetric Hessian residue by about
  `75x`, but the combined scalar at index `51472` remains `2.6247e-4` relative
  error and fails the `2e-4` gate.
- **Evidence:**
  `docs/development/nonlocal-corrected-gpu-assembly-audit-revision-2-evidence-2026-08-30.md`.
- **Conclusion:** final reduction order is not the only source; strict-f32
  product/normal/coefficient composition remains outside the frozen bound.
- **Decision:** close NCGA2 without review or sanitizers because the positive
  candidate is already refuted. No third reduction identity and no full GPU
  solver port are authorized.
- **Rejected alternatives:** widen the bound, remove the combined fixture,
  accept HVP-only equality, or infer product water from neighborhood timing.
- **Reconsider when:** a new contract independently selects arithmetic from a
  solver/physics error budget or decomposes product-level rounding.

### D-006 — Retain f32 for the next solver discriminator

- **Observation:** the strict matrix still misses one elementwise gate, while
  all thirteen HVP probes, the stable regularized step and eight quantized
  steps remain far inside the separately frozen consequence bands.
- **Evidence:**
  `docs/development/nonlocal-corrected-gpu-consequence-evidence-2026-08-30.md`.
- **Conclusion:** the miss is numerically real but negligible for this bounded
  local response. Merely accumulating f32 products in f64 does not close it;
  f64 pressure products do close it but do not improve the observed step.
- **Decision:** begin any separately frozen solver experiment with strict f32;
  preserve mixed pressure products as a fallback/control rather than paying
  their cost before a solver-level failure requires them.
- **Rejected alternatives:** relabel NCGA2 as passing, declare mixed precision
  mandatory from one scalar, or infer full water stability/frame time from
  eight tiny steps.
- **Remaining uncertainty:** nonlinear globalization, long trajectory,
  conditioning across a corpus, 50k assembly/solve cost and independent review.
- **Reconsider when:** a solver/corpus changes descent, convergence or physical
  acceptance under strict f32.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38 and ADR-076/081.
2. FCR0 formula contract and FCR3-B2 stop evidence.
3. NCGA0 and NCGA1 contracts, task states and independent reviews.
4. All four NCGA2/NCGA3 contracts in
   `docs/plans/nonlocal-corrected-gpu-assembly-audit/` and the NCGA3 evidence.

## Do not retry or infer

- do not use `variational_reference.cpp` or historical CPU/CUDA equality as
  the independent oracle;
- do not replace the exact Hessian with a clamped/Gauss-Newton/SISSM block;
- do not hide a host-built candidate graph or floating atomics;
- do not report NCGA2 work counts or the parallel NCGP0 neighborhood benchmark
  as full solver/game throughput.
