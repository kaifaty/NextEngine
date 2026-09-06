# NCGP10 pressure-f64 complete 4k corpus

| Field | Value |
| --- | --- |
| Research ID | `NCGP10` revision 1 |
| Status | `FROZEN / CORRECTNESS_ONLY / PERFORMANCE_BLOCKING` |
| Frozen before implementation | 2026-08-31 |
| Authorized parent | NCGP9 exact diagnostic commit `42ded232` |
| Claim class | finite/profile-bound mixed-precision 4k correctness; no 16k/50k or timing claim |

## Frozen observation and question

The exact primary NCGP9 route fails before step 1 because the binary32 density
sum places 1,362 centres above the pressure kink while the independent
long-double route places 978 centres above it. Density correspondence itself
is close (`2.40139231e-7` relative RMSE), but the discontinuous active set makes
the HVP relative error `0.3275595`.

The one pressure-only discriminator authorized by the original implementation
plan closes that same-state witness: 978/978 active IDs, HVP relative L2
`5.45037230e-7` and cosine loss `1.45875282e-13`. NCGP10 asks whether that
single arithmetic change is sufficient for the full frozen NCGP9 corpus.

NCGP9 remains `PHYSICS_REFUTED_BOUNDED`; NCGP10 does not reinterpret or erase
the primary-f32 result.

## Exact arithmetic change

Use `CompensatedScalePressureF64` for every GPU operator and solver evaluation.
Only the following pressure path is binary64:

- density accumulation used to classify the pressure active set;
- pressure excess and corrected `kappa`, `mass`, `rho0`, horizon and kernel
  coefficient products;
- pressure `q = Jv`, `J^T q` and the pressure geometric HVP term.

Reference/current/predicted/trial positions, velocity, gradient, HVP storage,
surface, viscosity, inertia, graph representation and final publication remain
the reviewed compensated binary32 `(hi,lo)` implementation. The public density
snapshot remains binary32 and is compared to the unchanged independent
binary64/long-double CPU oracle. No pressure active mask or scalar is copied
from CPU to GPU.

The variant and all additional binary64 pressure work are part of each work,
step, scenario and corpus root. The HVP ceiling remains 128, the selected
solver remains unpreconditioned Steihaug--Toint and every coefficient,
tolerance, scenario, boundary rule and observer threshold remains exactly the
NCGP9 revision-1 value.

## Admission and ordered corpus

Before any trajectory step:

1. reproduce the exact NCGP9 primary-f32 pressure-kink failure;
2. pass the pressure-f64 discriminator with active IDs exact, HVP relative L2
   `<=1e-3` and cosine loss `<=1e-6`;
3. pass all retained NCGP9 graph/operator/solver, pressure-active, physical,
   boundary/rollback, product, bulk and visible-observer controls.

Then run the unchanged ordered corpus:

1. hydrostatic hold, 4,000 samples, 240 steps;
2. dam break, 4,000 samples, 240 steps;
3. orifice jet, 4,000 samples, 240 steps.

Every per-step physical, permutation and visible-surface gate and every
60/120/180/240 retained bulk checkpoint is inherited verbatim from
`07-complete-4k-corpus.md`. The first failure stops the corpus. Stable-ID
trajectory errors remain diagnostic after the strict first-step gate.

## First-specific result

- input, CPU oracle, identity, receipt or non-matching corrected/permuted
  failure: `APPARATUS_INCONCLUSIVE`;
- matching admitted GPU work/capacity failure, or any same-state, physical,
  visible or bulk gate failure: `PHYSICS_REFUTED_BOUNDED`;
- all 720 steps and all 24 retained bulk field pairs pass: `CORPUS_PASS`.

No partial or prefix pass exists. Performance remains `NOT_RUN` in every
NCGP10 result. A corpus pass authorizes only a separately frozen 16k/50k
capacity and correctness successor.

## Evidence and stop rule

The report publishes the exact primary and pressure discriminator receipts,
arithmetic variant, all NCGP9 roots/work/metrics, source/contract/binary/CUDA
identity and the exact command. A passing candidate requires two fresh Release
builds, two byte-identical corpus processes, retained controls, Compute
Sanitizer memcheck/initcheck/synccheck and one independent read-only review.
One repair batch and one re-review are allowed. Threshold, coefficient, HVP
budget or scenario changes after observing the result are forbidden.
