# NCGP1 corrected CUDA full-step author evidence

| Field | Value |
| --- | --- |
| Research ID | `NCGP1` revisions 1--3 |
| Author result | `PHYSICS_REFUTED / PERFORMANCE_NOT_RUN` |
| Candidate commit | `c7c33e9bdc1be027ed61d24deb590f44b93ab44a` |
| Candidate tree | `eb7c037550e54588607c007abf90a3fe091bc20f` |
| Architecture parent | `62e8bd1f922ce8b8a2d03dd25a0695ac8f212c92` |
| Parent-to-candidate diff SHA-256 | `b8abea9996e635a2deb44fdbd990595c0787c5bab224d913cb392372d807f754` |
| Review state | independent review pending |

## Result

The scalable implementation closes the exact graph and matrix-free operator
boundaries, and its GPU-resident projected Newton--CG controller passes
analytic free fall/contact controls. It does not pass the first frozen tiny
solver gate. On the translated retained compressed pair, the strict-f32 GPU
path accepts three trials, rejects 21 and reaches minimum trust radius at
`R_x=1.50362650553e-5`, above the frozen `1e-5` terminal. The independent
long-double CPU solver succeeds in six HVP. Canonical and permuted GPU inputs
take the same rejected route.

This is `PHYSICS_REFUTED` under the revision-2 resolution firewall. The
performance window, single-pass graph candidate, fusion cycles, 240-step 50k
run and roadmap update are therefore `NOT_RUN`, rather than being timed after
a failed physical admission. The historical NCGA2 result
`2.624727856e-4 > 2e-4` remains unchanged.

## Passed boundaries before the first failure

| Gate | Result |
| --- | --- |
| exact scalable graph | PASS; 50k graph root `9f1ef440f7ca1c760c8f4586b1445d87908d8c5d05ef07ecf5372c96a344aed3`, 5,711,868 directed pairs, max degree 123 |
| graph permutation/capacity/strict-radius controls | PASS |
| matrix-free CPU vs dense HVP | relative L2 `3.8620648814e-16` |
| CUDA vs CPU HVP | relative L2 `9.33383431823e-7`, cosine loss `2.14939177567e-13`, active count exact |
| formula/operator controls | missing `2/h`, half viscosity, surface sign, HVP sign, graph role and missing neighboring pressure center all rejected |
| analytic free fall | GPU error `7.77244568706e-9 m`; Jacobi/unpreconditioned identical |
| boundary control | corrected lower-face state held exactly; disabled boundary returned `PhysicsGateFailed` |
| first 4k GPU/CPU step diagnostic | position RMSE `7.28544554644e-8 m`, max `2.35288370002e-7 m`; density RMSE `1.11118525785e-7 rho0`, max `4.13315272283e-7 rho0` |

The 4k result is a diagnostic after the formal tiny failure, not authority to
skip it. A separate GPU-only hydrostatic replay further shows the selected
work limit becoming material: step 11 succeeds at 110 HVP, while step 12
exhausts the 128 ceiling at 127 completed HVP with
`R_x=1.28564251464e-5`. The step remains finite, has zero penetration and
rolls back transactionally. Its output SHA-256 is
`ec7aa8176d8b20ae473d5949c1907c0641e9eb77f42c3fb73ad65be0a466f0d8`.

## Identity and repeatability

Two fresh Release/Ninja build directories used GCC 15.2.0, NVCC 13.3.73,
`sm_86`, `--fmad=false`, precise division/sqrt and FTZ disabled. Both produced
the same already-stripped executable and the same four-record report:

| Artifact | SHA-256 |
| --- | --- |
| candidate executable A/B | `163098d8d98d0b449a5b6a64b4e5c9d8c250e12fbf703cfe41518867a6de4a47` |
| graph/operator/solver/tiny stdout A/B | `315632ff7683dead595019179cddc542085a317035f609acefb30f2783f53b55` |
| expected tiny diagnostic stderr A/B | `9accafc0c1cd589ecbe2e9ebd7fdadc9c03bb40bf07420078001d1911d01c881` |

The exact tiny failure work root is
`47d9924b4e6f93203c063093456af081af39cdaf8bc8f58bb09b159804c25626`.
The full four-record stdout contains three `PASS` records followed by the
expected `PHYSICS_REFUTED` record and exits 37 only on the tiny command.

## Frozen and source hashes

| File | SHA-256 |
| --- | --- |
| revision-1 contract | `334418de01d0d8a1964331693e7d6846daef2cfb78fc93cc02d7c1bcf3ef32d7` |
| revision-2 terminal | `dce467e0f17e9140592bbb8f01c6b595ebd42f65faf292f857c941a44fe28d57` |
| revision-3 corpus | `64967d65787f3c94bf03c9d13c1112503be0b7a0259d4584f0707ef81585ecea` |
| public tool-only header | `10be3ea2385b52e7d841ffb178e295b1deb69075e227f46b4ac57634b19299d1` |
| CUDA candidate | `61f67481476217838f763ee1122ac0b688f260cff4339166407f5a6b62e107ea` |
| independent formula reference | `c14a7921034514c0f7d205cd32a01cf5279b78a2ebbaaa8fdd918ef5a049da62` |
| independent cached-CSR CPU solver | `040a13a844d82a52f3567c9c630efc92eb36c2d3caeaecfcefe11c731a93f5a4` |
| harness | `b79131b7d986ebc1bcc88a66b5b5aa985232107ee40eeecb80f0224cb9dd7d9d` |
| CMake target | `f4f62f211e43cc40db5b5bc0ec3b9f20481b076a2ba6397037d63eeb0e10fe40` |

## Sanitizers and immutable regressions

Compute Sanitizer `memcheck`, `initcheck` and `synccheck` each report
`ERROR SUMMARY: 0 errors` on the positive solver control. Retained outputs are
byte-exact:

| Regression | Result / stdout SHA-256 |
| --- | --- |
| NCGA0 terms | PASS / `5342fb400c07d429645f66ebb1f596d3e1a2fcd69e16dc5f8d6640c700cfbcd7` |
| NCGA1 graph | PASS / `0a89d92ceff561f6e8f128299ead356abaa7820cd05025fd2c1f6b3d85906b51` |
| NCGA2 assembly | expected exit 1 / `5ac5b3e7990ac5cadd2b711119fac8bdc3ae27c2d1b1bafeb7296fe587377878` |
| NCGA3 consequence | PASS / `0329f8b5848ba06b46f90af4028406af1b7b8ab68d2426a16f82912a6d4fc44a` |
| NCGA4 trust prefix | PASS / `be6236be48ab4e75cccbfdc53343f7ce41d77ab44524d7927afd04398d447cb0` |
| NCGA5 strict solve | retained classification / `fa1e5f059fd60aeed6e5ad67094c9c5aa9c17dc1b7444c23491bf990068bc03c` |
| NCGA6 mixed energy | retained classification / `87eeefc6ea53be8c81cb5e3a764f339e4332bf4a518bee5aa2b28b1c7c66fc82` |
| NCGA7 mixed pressure | retained classification / `a6bdbe0fb1a78ab091cdfb0e93566a1d71c20207b148732f2c899c43516000c4` |

## Product consequence

This result does not establish standalone Nonlocal water performance and says
nothing about integrated game frame time. It also does not invalidate the
corrected FCR formula/operator correspondence. It refutes this exact strict-f32
solver/profile/terminal/work package. CPU DFSPH remains the current product
fallback; SPEC-38 and ADR-076 remain Proposed and no public Rust/engine API or
roadmap fact changes.
