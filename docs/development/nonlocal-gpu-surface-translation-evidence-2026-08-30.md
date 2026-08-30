# NCGP2 surface translation-floor evidence — 2026-08-30

## Result and claim ceiling

Final result after the single review repair batch and the only permitted
re-review: `VERIFIED H1 SUPPORTED / GO`.

The unchanged global-position `f32` route still fails the retained translated
pair, and promoting only the surface operator to `f64` does not change that
failure. A real device-resident canonical `(hi, lo)` binary32 position state
passes the original pair and all nine translations without changing the FCR0
physics, solver tolerance, active set or HVP ceiling.

This is a tiny, boundary-free arithmetic discriminator. It does not establish
scalable graph transitions, contact semantics, 4k/50k trajectories or full
water performance. Those remain `NOT_RUN`; CPU DFSPH remains the product
fallback.

## Review history and repaired apparatus

The initial candidate at `2a0c38a2` was rejected as `H4 / INCONCLUSIVE`.
Although its local-coordinate execution passed numerically, it implemented a
host shared-origin specialization rather than the frozen per-position
canonical `(hi, lo)` representation. Its binary identity, endpoint evidence,
active-set signature and representation receipts were also incomplete.

The only permitted repair batch at `ecb888c6` replaced that shortcut with:

- explicit device `hi` and `lo` buffers for input, reference, current,
  predicted, trial, velocity and transaction state;
- fixed-order `TwoSum` error-free transforms for prediction and trial updates;
- both-part pair differences and inertia, runtime canonical/non-overlap checks,
  accepted/rejected rollback and one final binary32 publication;
- exact active-pressure ID roots and representation/work/result closure;
- `/proc/self/exe` binary hashing that fails closed, including a missing-path
  negative control;
- per-endpoint Phase-A displacement/ULP/once-round evidence and the corrected
  piecewise independent FCR0 `c(q)` diagnostic;
- negative controls for malformed pairs, omitted low parts, broken EFT,
  input/work/transaction/publish/representation roots and permutation identity.

No host anchor or coordinate localization remains in the repaired route. The
independent re-review found no remaining load-bearing defect for the frozen
finite NCGP2 claim.

## Frozen and candidate identities

| Item | Identity |
| --- | --- |
| Contract | `b4479af3ec41dd37aa02bd9f299c05387d416043d4f9701f0e08e8749752e4cb` |
| Repaired candidate commit | `ecb888c664404dcccab6361664259143af519022` |
| Repaired candidate tree | `bac1e17cf0de3485fb5f646b6a59516263d39bf9` |
| Source root | `4ba66a40259d8c1a59874149edc5709e66565fdf8b2fc635e5aab17bc3be677f` |
| Release binary A/B | `ed62e6c7994221746328c7329e6aaf7ae8565315cbd742c902e58f0f0cd51584` |
| GPU | NVIDIA GeForce RTX 3080, SM 8.6 |
| CUDA runtime/driver | `13030 / 13030` |

Two fresh Release directories produced byte-identical binaries and
byte-identical complete outputs:

| Mode | Exit | Output SHA-256 | Result |
| --- | ---: | --- | --- |
| `--phase-a` | `0` | `e18389bbb26d2b3bddb9f7384415e3e3168a6e3f3ae4652f66c7bdc8782ba434` | unchanged failure and controls reproduced |
| `--surface-f64` | `4` | `55f3dbc913112ef2d7c6c364975b21be98edc232538bf7a38843ca1c5fe8f0dc` | counterfactual failed |
| `--compensated-state-f32` | `0` | `e0628be83b190e277c9f93e34dac674730a2574fbca1707fa79ebcdae9839a19` | counterfactual passed |

Compiler controls were `-ffp-contract=off -fno-fast-math` for C++ and
`--fmad=false --prec-div=true --prec-sqrt=true --ftz=false` for CUDA, targeting
SM 8.6.

The reviewer independently rebuilt from a clean detached worktree. Its two
binaries were byte-identical at
`499dd5c01f6fae443d0bfde3af4f11bdabfb170e81f96199efae21d81767e9e6`.
That binary differs from the author binary only because the absolute source
path is embedded by the toolchain; after replacing the self-reported
`binary_root`, every reviewer output exactly matched the corresponding author
output above.

## Phase A: unchanged route

The independent direct/all-pairs CPU solver passed at every center. The
ordinary global-position CUDA route retained the original first-specific
pattern:

| Center (m) | CUDA | `R_x` | HVP | Trials (accepted/rejected) |
| ---: | --- | ---: | ---: | ---: |
| 0.125 | pass | `3.709e-7` | 6 | 3 (3/0) |
| 0.25 | pass | `6.676e-7` | 6 | 3 (3/0) |
| 0.375 | pass | `3.158e-6` | 6 | 3 (3/0) |
| 0.5 | pass | `4.991e-6` | 6 | 3 (3/0) |
| 0.625 | fail | `1.5036265e-5` | 48 | 24 (3/21) |
| 0.75 | fail | `1.5036265e-5` | 48 | 24 (3/21) |
| 1.0 | pass | `2.215e-6` | 6 | 3 (3/0) |
| 1.5 | fail | `1.5311771e-5` | 46 | 23 (2/21) |
| 2.0 | fail | `1.9560920e-5` | 46 | 23 (2/21) |

The retained `x=0.25` control passed. The `x=0.75, gamma=0` causal negative
also passed at `R_x=1.3775e-7`. Per-endpoint binary32 bits, ULP-normalized CPU
displacements and once-round errors are present in every version-2 center
record.

## Phase B discriminators

### Surface-only binary64

Binary64 surface subtraction, distance, spline, energy, gradient and HVP did
not change the failing centers or routes. The primary remained `failure=9`,
48 HVP, 24 trials and `R_x=1.50362650553e-5`. H2 is falsified for this fixture.

### Canonical binary32 `(hi, lo)` state

All nine translations passed. Corrected and permuted executions had identical
published state, active-pressure IDs, work roots and result roots. The primary
`x=0.75` route used 6 HVP and 3 accepted trials with no rejection, reaching
`R_x=1.74840291341e-6`. Its active-pressure root exactly matched CPU:
`029d295c3f3f85f9fd82e6d04241a3366c9fe9da4539b7eb02e7509e92462c9a`.

The maximum final position difference from the independent CPU result over
the sweep was `3.84540017606e-8 m`, below the frozen `5e-6 m` gate.

The sealed aggregate representation work root is
`a5cc08ee17adfdd0d8ff36429828c3447ce25dfb9a5c09bb2510bb8d95b40e2a`:

| Work item | Count |
| --- | ---: |
| input components | 324 |
| decompositions | 324 |
| final reconstructions | 108 |
| canonical/non-overlap checks | 648 |
| both-part differences | 5,724 |
| inertia components | 756 |
| predicted/trial EFT components | 540 |
| transaction components | 1,512 |
| published components | 216 |

The independent analytic surface control passed:

- gradient relative L2: `7.57043481411e-9`;
- HVP relative L2: `8.83071361321e-8`;
- directional relative error: `7.57043479309e-9`;
- equal/opposite closure: exactly `0`.

Both semantic mutation controls returned the frozen physics failure route:
omitted-low `failure=9`, broken-EFT `failure=9`. All identity and receipt
mutation controls were rejected.

## Independent verdict and bounded limitations

The re-review independently closed commit/tree/contract/source identities,
two clean builds, all three NCGP2 routes, retained graph/operator/solver
controls, PATH-based executable identity and all three CUDA sanitizers. It
accepted first-specific H1 and rejected H2/H3 for this retained fixture.

The following limitations are explicitly non-load-bearing only because the
frozen contract limits NCGP2 to a finite interior boundary-free pair and
defers scalable semantics:

- transaction/publish negative controls prove sealed-root sensitivity but do
  not inject a compensated post-finalize device failure;
- the malformed-pair control is a host-side canonicality predicate;
- omit-low and broken-EFT controls have typed physics failures, but their
  variants are also bound into the result root;
- graph cell membership remains based on the high position part.

These limitations must become load-bearing requirements in the scalable
successor. NCGP2 may not be cited as proof of dynamic graph transitions,
boundaries, multi-step rollback or production performance. The review/repair
allowance for this package is exhausted.

## Retained controls and sanitizers

The clean A/B NCGP1 graph, operator and solver self-tests each returned `PASS`
and were byte-identical between builds. Their output SHA-256 values were:

- graph: `cc83da2054479959408e2489a7bf42c2b6cbea2cf39e182b7e63a1ba04c1314d`;
- operator: `eda1e69f8265bb27d373e08b9589b7b14df15475ccadf8048ca8aa8d5c0ba380`;
- solver: `602a73f6b5c6eee0232c3f907cefbd5d83b960fa443d7cb5a5a9729c8fbc061e`.

The original tiny full-step route remained the expected
`PHYSICS_REFUTED`/exit 37 in both builds, with byte-identical output SHA-256
`dda70ded7341fa64739323f18ad3892da34d2a92c9ce38fc59b62d711786ff28`.

`compute-sanitizer` memcheck, initcheck and synccheck each returned exit 0 and
`ERROR SUMMARY: 0 errors`. Their captured outputs were byte-identical at
`e4834c94b3f3db055e75aef566d8da711098708878d53ca38a837ac96eca778e`.

## Decision and next action

The verified apparatus selects H1: sub-ULP accepted updates must remain in the
GPU state representation. The next package must freeze scalable `hi/lo` graph
cell transitions, ghost/contact boundaries, injected transaction failures,
multi-step state, memory cost and timing before reopening 4k correctness.
Performance remains `NOT_RUN`, and `docs/roadmap.md` remains unchanged.
