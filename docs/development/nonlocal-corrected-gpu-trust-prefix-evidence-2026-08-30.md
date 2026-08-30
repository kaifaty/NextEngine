# Nonlocal corrected CUDA trust-region solve-prefix evidence — 2026-08-30

Status:
`AUTHOR_SUPPORTED_BOUNDED / GPU_ASSEMBLED_TRUST_PREFIX_SUPPORTED / REVIEW_NOT_RUN / REPORT_ONLY`

## Outcome

The frozen NCGA4 revision-1 experiment passed every declared gate. On the
100-particle combined fixture, the independent `long double` evaluator and the
compensated strict-f32 CUDA evaluator drove the same eight-entry
Steihaug--Toint controller signature:

- all `8/8` outer trials stopped at finite `NEGATIVE_CURVATURE` after one HVP;
- all `8/8` trials were accepted with positive actual and predicted reduction;
- the radius sequence was exactly `50 um -> 100 um -> 200 um`, then remained
  at `200 um`;
- there were zero rejects, zero shrinks, no topology/surface-branch crossing
  and no pressure-active mismatch;
- all 100 pressure centers remained active; and
- canonical and permuted CUDA inputs produced an exact common solve result.

The CUDA path assembled objective, analytical gradient and dense exact Hessian
on the RTX 3080. The trust controller and dense CG matvec remained host
binary64. This is therefore a GPU-assembled/host-controlled prefix, not a
GPU-resident nonlinear solver.

## Frozen numerical gates

| Metric | Reference | strict f32 CUDA | Gate/result |
| --- | ---: | ---: | --- |
| initial objective | `14175.010660251486` | `14174.987326634931` | reported |
| final objective | `14112.745971848557` | `14112.722975396551` | both descend |
| accepted/rejected | `8 / 0` | `8 / 0` | exact |
| inner route | `8x NEGATIVE_CURVATURE` | `8x NEGATIVE_CURVATURE` | exact |
| controller signature root | `6515b4c9...d9878` | `6515b4c9...d9878` | exact |
| maximum state drift | — | `0.00067487689053012103 um` | `<=5 um` |
| reduction-fraction difference | — | `5.4150202555864671e-6` | `<=1e-3` |

The state drift is about `7409x` below its frozen screening limit and the
reduction difference about `185x` below its limit. These margins are reported
only for this fixture and prefix; they are not fitted production tolerances.

Reference accepted ratios range from `0.99999762199951669` to
`0.99999985063928509`. CUDA accepted ratios range from
`0.99981845091241561` to `1.0002912229900713`.

Exact result identities:

| Artifact | SHA-256/root |
| --- | --- |
| semantic result | `4af9bde2b8933ff0ae6923e4717a7f0ca1e13179dca1e959a73afa87d9eff4a4` |
| reference final state | `1e40ac69270e6e493c34e25cf1c7631d6035667cda0f338131ed841ca0819612` |
| CUDA final state | `5946e191bb06d41c8a126c02d1686ce188be4185ee717f430cfbb770f7cc427e` |
| reference work | `733416551df600b7475895eee6229b89eaf9947bb311cad05f2ce584fd6588a3` |
| CUDA work | `2bb6059540f7a7f8a072228b5a54cc68f6bf184823db8310d95ab69ede972ddb` |
| raw report | `be6236be48ab4e75cccbfdc53343f7ce41d77ab44524d7927afd04398d447cb0` |

## Negative control and work closure

The deliberate first-HVP sign inversion changed the controller signature from
`6515b4c9...d9878` to `a70164af...12a8d` and was rejected. It later rejoined
the same final binary32 state, which demonstrates why the controller route is
sealed separately from final-state comparison.

Each positive path executed nine complete evaluator calls. The CUDA receipt
seals, among other fields:

- `8` CG matvecs, `8` prediction matvecs and `72` controller dot products;
- `8` boundary intersections, `8` accepts and `2` radius expansions;
- `2,700` continuous-state scalar uploads, `2,400` trial f32 rounds and
  `39,600` topology pair checks;
- `81,000,000` pressure outer products;
- `810,000` dense entries and `810,000` inherited direct-HVP products;
- `173,956,500` compensated additions and `1,894,536` compensation
  initializations.

## Reproducibility and verification

Frozen contract SHA-256:
`9eb63f2a7cf6728062b0989c5b1da51435444a2a525d770dda865e598bc3ce7c`.

Implementation checkpoint:

| Field | Value |
| --- | --- |
| commit | `c6d5db9c0acb8a3e32c27e4256abc411ebac7017` |
| tree | `3e81f1ecc9913518c7a82968d4de27cfc4fe9ae0` |
| CMake SHA-256 | `677c7dd94ab35d80a1b779cef68dc1ad5e210f541dda7ac285453e1723a487cf` |
| assembly header SHA-256 | `51091a1c3d1d86d13e7fdca9106e1b2352338dbd2c9b0dc37fec9e51d4d7c809` |
| CUDA assembly SHA-256 | `1d99f003e09f1211566efc77279f1bb7f4ef99af19d9be183576f3658cf8b41c` |
| host evaluator SHA-256 | `7bd0462b36b5f83cdae88872551e067fef17faf9a98325716646d7d29cde40da` |
| trust driver SHA-256 | `e6aba1f91a4b52afd5bedfe31decea7be9b94419d6e42aa4c5db3dff69b5616c` |

Two fresh Release directories used CMake 3.28+, Ninja, GCC 15.2 and NVCC
13.3.73 with SM86, `--fmad=false`, precise divide/sqrt and FTZ disabled. Both
stripped binaries are byte-identical at
`32f0cd69cfe6732f18e21244d29f538e458314b94a45bab5185cc161daabb407`;
both reports are byte-identical at `be6236...447cb0`.

CUDA Compute Sanitizer on that exact Release executable:

| Tool | Result |
| --- | --- |
| `memcheck` | PASS / `ERROR SUMMARY: 0 errors` |
| `initcheck` | PASS / `ERROR SUMMARY: 0 errors` |
| `synccheck` | PASS / `ERROR SUMMARY: 0 errors` |

Focused retained regressions:

| Check | Exit/result | stdout SHA-256 |
| --- | --- | --- |
| NCGA0 corrected terms `--self-test` | `0 / PASS` | `5342fb400c07d429645f66ebb1f596d3e1a2fcd69e16dc5f8d6640c700cfbcd7` |
| NCGA1 neighborhoods `--self-test` | `0 / PASS` | `0a89d92ceff561f6e8f128299ead356abaa7820cd05025fd2c1f6b3d85906b51` |
| NCGA2 strict assembly | expected `1 / FAIL` | `5ac5b3e7990ac5cadd2b711119fac8bdc3ae27c2d1b1bafeb7296fe587377878` |
| NCGA3 consequence screen | `0 / PASS` | `0329f8b5848ba06b46f90af4028406af1b7b8ab68d2426a16f82912a6d4fc44a` |

No workspace ProductCheck was run: the target is an isolated report-only C++/
CUDA research harness and changes no Rust runtime, public contract or product
profile. Independent review is `NOT_RUN`; the user did not request a reviewer
for NCGA4, so this evidence remains author-only.

## Interpretation and next boundary

The result closes the immediate f32 question for one globalized prefix: the
known Hessian scalar miss did not change negative-curvature detection, trust
radius policy, acceptance or local state to a material degree. Mixed pressure
products are therefore not selected by NCGA4.

It does **not** demonstrate full nonlinear convergence. Every inner solve
terminated on negative curvature after one HVP, and the gradient norm rose as
the objective descended. The next valid experiment is not a physical water
trajectory yet: first run the same CUDA evaluator/controller to convergence on
the two frozen NSR1 static cases, including a positive-curvature/residual CG
path. Only after that correspondence closes should a boundary-free short
trajectory reopen.

The algorithm choice itself is retained prior art rather than a new method:
Steihaug's original paper establishes truncated/preconditioned CG for trust
regions and explicitly treats negative curvature
([SIAM DOI 10.1137/0720042](https://epubs.siam.org/doi/10.1137/0720042));
PETSc's official optimization manual describes the same quadratic-model,
radius and actual-reduction transaction used by production trust-region
solvers
([PETSc TAO manual](https://petsc.org/main/manual/tao/)). These sources support
the method family, not the correctness of this implementation; that comes only
from the frozen local evidence above.

## Claim ceiling

NCGA4 supports only one eight-trial, 100-particle, fixed-graph,
GPU-assembled/host-controlled correspondence result. It grants no full-solver
convergence, dynamic-neighborhood, physical trajectory, contact/boundary,
50k/full-frame timing, GPU-resident Newton--CG, runtime, gameplay or
game-ready-water authority. SPEC-38 remains Proposed and CPU DFSPH remains the
current V1 product candidate.
