# Nonlocal corrected CUDA term correspondence — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NCGA0` |
| Status | `FROZEN / IMPLEMENTATION_PENDING / REPORT_ONLY` |
| Architecture snapshot | `9b5021aa29b763c4db3d7ed8c975ab9f99a861fc`; SPEC-38 and ADR-076 remain `Proposed`; ADR-081 remains `Accepted` |
| Engineering consumer | Decide whether a corrected Nonlocal CUDA lineage may proceed from scalar/pair terms to a separately frozen neighborhood/linearization audit |
| Claim class | Finite profile-bound implementation correspondence |
| Claim status target | `SUPPORTED_BOUNDED`, `REFUTED`, or `INCONCLUSIVE` |
| Budget | One small executable, nine fixed fixtures, ten cold GPU repetitions, one initial independent review plus at most one batched-repair re-review |

## Exact claim

On the current Linux x86-64 host with NVIDIA RTX 3080 (`sm_86`), CUDA 13.3,
strict binary32 device arithmetic (`--fmad=false --prec-div=true
--prec-sqrt=true --ftz=false`) and the immutable profile below, a separately
implemented corrected CUDA evaluator matches an independent host
`long double` evaluator on all nine fixed term fixtures.

For every nonzero scalar or force component, correspondence requires

```text
abs(candidate - reference) <= 2e-5 * max(1, abs(reference)).
```

Every reference zero requires candidate absolute value `<= 2e-7`. Every
candidate value must be finite. Pair endpoint forces must close under the same
mixed bound. Ten cold CUDA allocations/executions must produce byte-identical
binary32 result payloads.

The historical source-shaped gradient control must fail the corrected bound
for `kernel_inner`, `kernel_outer` and `compression_above_rest`. A harness that
cannot observe those failures is rejected.

## Exact negation

The claim is negative if any corrected fixture is nonfinite, exceeds its
predeclared bound, violates pair closure, changes bytes across cold repeats, or
if the source-shaped negative control does not reject all three named cases.

## Fixed definitions and model

- Coordinate system: right-handed MKS; term checks depend only on relative
  vectors and do not publish engine state.
- Profile: `h=0.15 m`, `dx=0.05 m`, `m=0.125 kg`, `dt=1/240 s`,
  `rho0=1000 kg/m^3`, `kappa=9196.875`, `lambda=2.5`, `mu=1.7`,
  `gamma=3.5`.
- Kernel: the FCR1 cubic with `q=2r/h` and
  `dW/dr=(dW/dq)*(2/h)`.
- Compression: `max(rho/rho0-1,0)^2`; fixtures select ratios `1.1` and
  `0.9` away from the nondifferentiable threshold.
- Viscosity influence: `omega=-dW/dr`; directed-pair bulk coefficient is
  `lambda/2`, shear coefficient is `mu`.
- Surface: `C(r)=dx*C_hat(r/dx)` with fixed repulsive `q=0.8`, attractive
  `q=1.7`, and outside-support `q=3.2` fixtures.
- Reference arithmetic: independent host `long double`, contraction disabled
  by the C++ target flags.
- Candidate arithmetic: CUDA binary32, one thread per fixture, no atomics,
  reductions, neighbor search, solver update or shared formula implementation.
- Frozen parent identities:
  - formula contract SHA-256 `8d693724d6b32d4aa899f57551b45248d645a1ecdcec3ec20cd66572b4a1c5ab`;
  - corrected CPU source SHA-256 `beb331df7b0321fc3235e6b085756cd5a10c5effe9854104203901203436fbe0`;
  - historical CUDA source SHA-256 `ccf6266125f27ababb57e3c9538c13d5a7af3bc6f6cbdc283185b4800b02b4d5`;
  - NPR1-B evidence SHA-256 `44c2af688225b7b779cddac53ea472485d0ea896fa0bc65f6716a6233b60946c`.

## Fixture order

1. `kernel_inner`, `r=0.03 m`;
2. `kernel_outer`, `r=0.105 m`;
3. `compression_above_rest`;
4. `compression_below_rest`;
5. `bulk_viscosity`;
6. `shear_viscosity`;
7. `surface_repulsive`, `q=0.8`;
8. `surface_attractive`, `q=1.7`;
9. `surface_outside`, `q=3.2`.

Compression and viscosity use the exact pair inputs from FCR0. Surface uses
the FCR0 normalized axis and the named `q`. The input table is constructed in
the host front end and copied unchanged to both evaluators.

## Assumptions

| Assumption | Status | How checked or bounded |
| --- | --- | --- |
| CUDA device and toolchain match the selected host | given | executable reports device, driver, runtime, architecture and compile profile |
| Host and device consume identical fixture bytes | derived | one fixture-root field covers the ordered serialized input table |
| Host oracle is independent of the candidate formulas | required | separate C++ and CUDA translation units; shared DTO/profile only |
| Nine fixtures are nonempty and avoid undefined normalization | derived | fixed nonzero pair distances except the explicit outside-support zero result |
| Surface normalization is the selected FCR1 convention | given | this experiment does not adjudicate another paper convention |

## Resolution and near-miss firewall

- Positive: every corrected fixture, closure, repeatability and negative
  control gate passes exactly as frozen.
- Negative: first failed gate is retained; no tolerance, input or arithmetic
  profile changes after observation.
- Does not count: source-shaped CPU/GPU equality, a successful build, aggregate
  trajectory appearance, a different device, hidden double arithmetic or a
  relaxed tolerance.
- Claim ceiling after success: corrected scalar/pair term correspondence only.
  No neighborhood/indexing, local matrix, `3x3` solve, SISSM, recurrence,
  multi-step physics, performance, canonical authority, runtime or production
  claim.

## Competing hypotheses

| ID | Hypothesis | Prediction | Falsifier |
| --- | --- | --- | --- |
| H1 | Corrected term formulas translate faithfully to strict CUDA `f32` | all nine corrected fixtures pass and repeat exactly | any corrected term fails before neighborhood/solver work |
| H2 | A CUDA coefficient/sign/branch defect survives independently of the old source identity | one or more corrected fixtures fail while the host oracle and negative control behave as expected | all corrected fixtures pass |
| H3 | The prior decisive failure is shared source mathematics, not a GPU-only translation defect | corrected path passes while the source-gradient control rejects the three named cases | source control passes or corrected path reproduces the same ratios |
| H4 | The apparatus shares a defect or cannot distinguish identities | negative control does not reject the known missing-chain-factor cases | all three named negative cases reject |

## Evidence plan

- Build only the new target plus the immutable source/corrected controls in a
  fresh directory outside Git.
- Run corrected and source-shaped modes twice at process level; each corrected
  process owns ten cold GPU repetitions.
- Re-run the historical retained GPU `11/11`, NPR1-B expected failure and FCR0
  corrected formula pass as non-regression controls.
- Freeze source, binary, stdout and fixture/result roots before independent
  review.
- Reviewer receives this contract, the exact diff and raw hashes without the
  intended verdict. Candidate source is read-only during review.

## Stop and reconsider

- Stop `INCONCLUSIVE` if the current GPU/toolchain cannot execute, the oracle
  independence cannot be established, or a load-bearing defect survives the
  one allowed batched repair.
- Do not patch the historical `cuda_baseline.cu` or relabel its old roots.
- Reconsider the claim only through revision 2 if the fixture set, numeric
  profile, term semantics or observable changes materially.
- Even a positive result leaves GPU correspondence-only under SPEC-38 and
  ADR-076/081.
