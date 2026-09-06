# NCGP1 corrected CUDA full-step author evidence

| Field | Value |
| --- | --- |
| Research ID | `NCGP1` revisions 1--4 |
| Author result | `PHYSICS_REFUTED / PERFORMANCE_NOT_RUN` |
| Candidate commit | `3bc5e81d366a0f1e55bd6a3babb967ca26907db2` |
| Candidate tree | `c65469fb4ba601a637757e6b88d72d28f3cf7221` |
| Architecture parent | `62e8bd1f922ce8b8a2d03dd25a0695ac8f212c92` |
| Parent-to-candidate diff SHA-256 | `04b57ae4370b7783ac402015f336bb41ffe47ce0c77405c7f444ee18814aee3e` |
| Review state | `VERIFIED_PHYSICS_REFUTED`; single repair/re-review exhausted |

## Result

The initial independent review found load-bearing apparatus defects, so its
verdict was `INCONCLUSIVE / ONE BATCHED REPAIR REQUIRED` (report SHA-256
`8e19f05dcc6a60ebed4308d7c9e4ba372443402caa24b915d015c57cde1d08e9`).
Revision 4 freezes and implements that one repair: active-set pressure HVP,
direct tiny all-pairs oracle, common binary32 input bytes, exact full-HVP
diagonal accounting, fail-closed admission, full rollback, swept boundary
receipts, valid advected admission and typed result precedence.

The repaired scalable implementation closes the exact graph and matrix-free operator
boundaries, and its GPU-resident projected Newton--CG controller passes
analytic free fall/contact/rollback controls. It still does not pass the first frozen tiny
solver gate. On the translated retained compressed pair, the strict-f32 GPU
path accepts three trials, rejects 21 and reaches minimum trust radius at
`R_x=1.50362650553e-5`, above the frozen `1e-5` terminal. The independent
long-double direct/all-pairs CPU solver succeeds in six HVP. Canonical and
permuted GPU inputs have identical route, work and result roots.

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
| inactive/mixed pressure controls | inactive CUDA vs analytic inertia HVP `1.52446172052e-7`; oracle exact; mixed CUDA vs oracle `4.27521750692e-7`; independent energy FD error `2.2572944105e-9` |
| analytic free fall | GPU error `7.77244568706e-9 m`; Jacobi/unpreconditioned identical |
| boundary/admission/rollback controls | swept face receipt exact; disabled boundary rejected; post-finalize injection restored all state; invalid profile, ghost and f64-to-f32 overflow failed before mutation |
| first 4k GPU/CPU step diagnostic | position RMSE `7.28544554644e-8 m`, max `2.35288370002e-7 m`; density RMSE `1.11118525785e-7 rho0`, max `4.13315272283e-7 rho0` |

The 4k result is a diagnostic after the formal tiny failure, not authority to
skip it. A separate GPU-only hydrostatic replay further shows the selected
work limit becoming material: step 11 succeeds at 110 HVP, while step 12
exhausts the 128 ceiling at 127 completed HVP with
`R_x=1.28564251464e-5`. The step remains finite, has zero penetration and
rolls back transactionally. Its output SHA-256 is
`ec7aa8176d8b20ae473d5949c1907c0641e9eb77f42c3fb73ad65be0a466f0d8`.

## Identity and repeatability

Two fresh Release build directories used GCC 15.2.0, NVCC 13.3.73,
`sm_86`, `--fmad=false`, precise division/sqrt and FTZ disabled. Both produced
the same already-stripped executable and the same four-record report:

| Artifact | SHA-256 |
| --- | --- |
| candidate executable A/B | `bf5df6352a717c2ef869f287bd6f30c4866b7aafbc09677501f0b2cfb927563f` |
| graph/operator/solver/tiny stdout A/B | `0cf043845710d8014b9c4f138d3c9c27c1569becdafa69273549e127b1dc53f1` |
| expected tiny diagnostic stderr A/B | `9accafc0c1cd589ecbe2e9ebd7fdadc9c03bb40bf07420078001d1911d01c881` |

The exact budget/profile-bound tiny failure work root is
`2d4622c5541157848031d815157830522b4845af0f730f6b977bad8d55fb77ed`;
the result root is
`e84f05cc6d0c17c53b21582a2bb67279880d55cff2021893966865d13eb7a5b9`.
The corrected and permuted roots match. The healthy CPU-oracle result root is
`9af96c2a5c256f8e98cbcff7ad83ae5f5d1fb98432900d26f1e21378e3086e75`.
The full four-record stdout contains three `PASS` records followed by the
expected `PHYSICS_REFUTED` record and exits 37 only on the tiny command.

## Independent re-review

The single authorized re-review used source commit `3bc5e81d`, tree
`c65469fb4ba601a637757e6b88d72d28f3cf7221` and evidence commit `0bd26ad6`.
Its verdict is `VERIFIED_PHYSICS_REFUTED`; no load-bearing finding remains.
Two fresh reviewer Release binaries were byte-identical at
`593d4c1c4a7868d33fc59b7c18ea210c71f026de533e23b55ce9a12d6fba4352`.
Both reviewer closures passed graph/operator/solver controls and reproduced
tiny exit 37 with GPU/permuted failure `9`, 48 HVP, 24 outer trials, `3/21`
accepted/rejected trials and
`R_x=1.50362650553e-5`; the CPU long-double direct oracle succeeded in six
HVP. Corrected/permuted work root
`2d4622c5541157848031d815157830522b4845af0f730f6b977bad8d55fb77ed`
and result root
`e84f05cc6d0c17c53b21582a2bb67279880d55cff2021893966865d13eb7a5b9`
were exact.

The reviewer specifically verified the repaired active-set HVP, direct tiny
oracle, common input identity, charged Jacobi probes, full transaction
rollback, swept boundary receipts, fail-closed admission and typed failure
precedence. The result remains bounded to this strict-f32
solver/profile/terminal package; it does not refute corrected FCR formulas.

## Frozen and source hashes

| File | SHA-256 |
| --- | --- |
| revision-1 contract | `334418de01d0d8a1964331693e7d6846daef2cfb78fc93cc02d7c1bcf3ef32d7` |
| revision-2 terminal | `dce467e0f17e9140592bbb8f01c6b595ebd42f65faf292f857c941a44fe28d57` |
| revision-3 corpus | `64967d65787f3c94bf03c9d13c1112503be0b7a0259d4584f0707ef81585ecea` |
| revision-4 repair closure | `65f17900542e98488e15cc2ab6868680bf0eea8974252122a21e7a6302fbcf4d` |
| public tool-only header | `7cd865217779e987b49920e555a6e23b725610cf35f0194223869872e8c8a47c` |
| CUDA candidate | `3816ccff536a3f79b54cad8270d1392971a4e13d4907f4c0d81d83d3db69413f` |
| independent formula reference | `b77cd047630fce53469ecf35daee4b0771c3a4422a58caa6c88b2653c96ee5e3` |
| independent tiny-direct / 4k-CSR CPU solver | `65c31e4b80811ba1dda45f6cdc4cdc7be1cf9186b61771240fd37d7ca83583a2` |
| harness | `e6e8d83e4a7078d8850a5cb9dbd35198a9453895fb21ea7328918bfd1c6320b0` |
| CMake target | `eddcf5842d43439c4c5d153f93d60b9bf1483f7cc80ef06a8832b0712fd2c41e` |

## Sanitizers and immutable regressions

Compute Sanitizer `memcheck`, `initcheck` and `synccheck` each report
`ERROR SUMMARY: 0 errors` on the positive solver control. Their complete
stdout files are byte-identical at
`68b8b4c8dc8fdfe622b41f2197c7cc98c2e34b7e062949d9f6efea89721f936c`.
Retained outputs are
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
