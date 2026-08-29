# Nonlocal corrected GPU term audit — revision-2 repair evidence

| Field | Value |
| --- | --- |
| Research ID | `NCGA0` revision 2 |
| Contract | `docs/plans/nonlocal-corrected-gpu-audit/01-full-pair-force-correspondence-contract.md` |
| Contract SHA-256 | `10303b06727c703a2a958b3c6e49663a9b1b6ee7ad462e8f15cba267474d3e6f` |
| Author result | `CORRECTED_FULL_PAIR_TERM_CORRESPONDENCE_SUPPORTED_BOUNDED` |
| Review status | `SINGLE_REREVIEW_PENDING` |
| Product status | `REPORT_ONLY`; SPEC-38 and ADR-076 remain `Proposed` |
| Host | Linux x86-64, NVIDIA GeForce RTX 3080, compute capability 8.6, driver `610.43.02`, CUDA compiler/runtime `13.3.73` / `13030` |
| Fresh build | `/tmp/nextengine-corrected-gpu-repair-jqlKHr` (outside Git) |

## Repair outcome

The revision-1 full-force implementation remains unchanged numerically, but it
is now checked against a separate `long double` energy translation unit. That
oracle uses the frozen energy coefficients `lambda/2` and `mu`, perturbs only
the first endpoint, and takes the frozen central directional derivative.

The corrected CUDA full-force projection passed both energy derivatives. A
deliberate directed-edge/half-force CUDA mode failed both the component and
energy gates, closing the factor-of-two apparatus gap from the initial review.
All original nine fixtures, three source-gradient negatives, bounds and ten
cold repetitions remain unchanged.

## Frozen author artifacts

| Artifact | SHA-256 |
| --- | --- |
| Executable | `2e3f5427c5253cdd7a0daa637064cbd7ef2e34595321a546a62343ac81332edf` |
| Header/DTO | `0a64494129e7f9596b71882c9118e1104adaee80f4e03e13550745082b8c81cf` |
| Host force reference | `150121df18c537252eacac0012e7bf12eb7be12d0e0eeac9e43b12b28cdd6b3f` |
| Host energy derivative | `2d2d810b647cb03ba623f1bafa47046c5689363799cce1768b1d4593cb3f9a66` |
| CUDA evaluator/controls | `c43fe7e74de04d2fe1702ba647991a155bd3e12d92b1d332fad6ff1602674900` |
| Harness/fixtures | `53e4363208e8b0893d22fd05cc5bacbbc3959a1826efa6df2a59d61d9e1caa68` |
| Process stdout | `5342fb400c07d429645f66ebb1f596d3e1a2fcd69e16dc5f8d6640c700cfbcd7` |
| Ordered fixture root | `046e111e760788e67d5a1ae04448a4a9d606c07974f386d9b046fe874c089a87` |
| Corrected CUDA payload root | `a915ddbcddfd0cc607db83b734402e3744784e30014be20b0d16aae78f9b198b` |
| Energy-reference root | `199335b485e68f18dd76472372177b47f4364c108947a4b2b2d7357c4245bac9` |
| Source-shaped control root | `f92b85b6fef687c2ef8e39578ba99736546a1585d657e4f365bb60e15a3df4a2` |
| Directed-edge control root | `defab98e3bbd1363487bba0aca483f9b129a4dc9ce6df6f244294365c8c3ad7d` |

Two fresh process invocations produced the exact stdout hash above. Every
process performed ten cold corrected CUDA allocations/executions.

## Independent energy discriminator

| Fixture | `dE/depsilon` | Corrected `F_i dot d` | Corrected error | Half-force error | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| `bulk_viscosity` | `-0.080691255889494856` | `0.080691177862044522` | `7.8027450334250403e-08` | `0.040345666958472595` | corrected PASS, half rejected |
| `shear_viscosity` | `0.032353476585516294` | `-0.032353606608413341` | `1.3002289704633352e-07` | `0.016176673281309624` | corrected PASS, half rejected |

The frozen derivative bound is `2e-5*max(1,abs(dE/depsilon))`. The deliberate
half-force errors exceed it by factors of about 2017 and 809 respectively.

## Other numeric gates

- Nine corrected fixtures: PASS; largest component mixed error
  `3.6787060029602878e-07` against the frozen `2e-5` bound.
- Endpoint closure: PASS under the frozen `2e-7` zero bound.
- Source-shaped negatives: all three mandatory kernel/kernel/compression cases
  rejected.
- Directed-edge negatives: both mandatory viscosity cases rejected.
- Ten cold repeats per process: byte-identical.

## Commands and checks

Fresh Release configure/build and two primary runs: PASS. The executable was
compiled for `sm_86` with `--fmad=false --prec-div=true --prec-sqrt=true
--ftz=false`; both host oracles used `-ffp-contract=off -fno-fast-math` and
warnings-as-errors.

Compute Sanitizer `memcheck`, `initcheck` and `synccheck` each exited 0 with
`ERROR SUMMARY: 0 errors`.

Same-build non-regression controls:

- retained source-shaped CUDA gather/pointer-swap/specialized path: `11/11`
  PASS and exact reused-instance output in all 11 cases;
- NPR1-B: expected `KERNEL_SOURCE_GRADIENT_MISMATCH`, result root
  `069aff09f7919fae33f86b2c86cb4a39d654188a15bd8c14868a9dfd5237e5c9`;
- FCR0: PASS, result root
  `ea5c4b423ce47b89699150dd17086ac63ccc4e4a59ec234a829a066022b54412`.

## Claim ceiling and next action

This remains corrected scalar/full-pair-force term correspondence on one frozen
profile and GPU only. Neighborhood/indexing, matrix assembly, local solve,
SISSM, trajectories, performance, runtime authority and product-ready water are
not established.

Freeze the repair commit and revision-1-to-repair diff, then give the original
reviewer the single allowed read-only re-review. If it is clean, NCGA0 can close
and the next separately frozen neighborhood/linearization audit may be drafted.
