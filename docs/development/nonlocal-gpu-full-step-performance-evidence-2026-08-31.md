# NCGP4 corrected GPU full-step evidence — 2026-08-31

## Verdict

`REFUTED_BOUNDED / PERFORMANCE NOT_RUN` for the frozen NCGP4 claim.

Removing the scalar Jacobi preconditioner repairs the prior deterministic
step-39 work-ceiling failure, but the maximum allowed 128-HVP profile still
fails the first 240-step 4k trajectory. Hydrostatic hold crosses the frozen
CPU/GPU maximum-position gate on step 92: `5.211209258 mm` against a `5 mm`
limit. The complete 4k corpus therefore has no passing member of
`{32,64,128}` and later correctness/performance stages must not run.

This result does not show an unstable or escaping fluid. At the first failure,
position RMSE is `0.238779 mm`, density correspondence max is `1.01934%`,
compression max is `1.44937%`, normalized momentum residual is `0.238287%`,
positive energy excess and penetration are zero, basin bounds are closed, and
coherent/permuted GPU compensated state roots are identical. The bounded
failure is one trajectory-correspondence outlier relative to the independent
CPU oracle.

## Frozen identity

- source commit: `08036770046f3f937221712f7ba554be38b2b454`
- source tree: `0c7972870b4db5ed82b1ed577d3b659639d11618`
- aggregate NCGP4 contract root:
  `1e63c02acf1bfb77b9ffcbdede07882e8b2a2e22d4a3804d2d9412e5fd957a7d`
- aggregate source root:
  `b9d94a2a1dc0e7f688d35a948120116d8684ec0e29e43a344c66823e7ed7231d`
- clean Release binary SHA-256:
  `55db74591b893b71fd8d329d5a28505ae890c676143181896e0c4248c7344f6c`
- host: NVIDIA GeForce RTX 3080, SM 8.6, CUDA runtime/driver 13.3
- flags: C++ `-O3 -Wall -Wextra -Wpedantic -Werror -ffp-contract=off
  -fno-fast-math`; CUDA `-O3 --fmad=false --prec-div=true --prec-sqrt=true
  --ftz=false`; `SM=86`

## Solver diagnosis and selected repair

The exact pre-step-39 compensated state root is
`df0fe7a85f061e734e632c7ee3f4bbf10bacf477187afdf6a3d9297215f53824`.
On that state, scalar Jacobi fails at 126 HVP/28 outer trials while the retained
unpreconditioned profile succeeds at 69 HVP/19 outer trials. Same-state GPU
versus long-double HVP relative L2 is `1.4262815435566996e-6`, cosine loss is
`9.8629643948550116e-13`, and the active signature is exact. Raw diagnostic
stdout SHA-256 is
`78b40392e0981eec77e37c514ce39cae78172c642edfa9ead3de0d737e342f47`;
result root is
`2f376da73d4edb8a09d5a223b43870415cfdd3e685601cb5e639b4fff52e4cce`.

NCGP4 revision 2 therefore selects the unchanged unpreconditioned
Steihaug--Toint path. No coefficient, tolerance, radius rule, acceptance rule,
arithmetic profile or 128-HVP ceiling changed.

## Repaired apparatus and retained controls

The final binary passes profile, pair-aware graph, swept boundary, complete
hi/lo transaction rollback and the repaired physics suite:

| Control | Raw stdout SHA-256 | Result |
| --- | --- | --- |
| profile | `df63e96f22df47a9167666682d1575b89e9e53a15b02d7dcf0c1d6c0c5f900cb` | PASS |
| graph | `21b66386d98d2ed860385a03039b3331c286e3e06f9a82aeb23182bdfc64eff2` | PASS |
| boundary | `fa9a3386e70f6b7f44d2e8002bf2ced1383e72276c642d799055e46b50a29f7a` | PASS |
| transaction | `ad05d715bd5953460fa5812a2586e04aedd5a57bf797b9e0d7562dffd2654965` | PASS |
| physics | `9e11354ec1ffab3ccc16387286cef2327e3dd80efaeba7b59828206c00585e9a` | PASS |

The repaired free-fall control executes equal forward/reverse legs. Its
reconstructed position error is zero, velocity error is
`1.4068186282578665e-7 m/s`, and reversible energy drift is
`8.9605810875784662e-10`. The viscosity gate uses the frozen relative
denominator. The transaction control restores the complete compensated state
root exactly and a retry matches a fresh workspace.

## Exact work-budget ladder

All commands use `--correspondence-4k-unpreconditioned hydrostatic-hold 240
BUDGET` with the final binary.

| HVP budget | Completed steps | First result | Raw stdout SHA-256 | Result root |
| ---: | ---: | --- | --- | --- |
| 32 | 0 | `WorkBudgetExceeded`, 32 HVP | `7fba32df885748cd8d489795e36bf77fae4ccdc43098eaaab539b03e72f7a4be` | `a19abe9631259c6c19f9d64d2b9983fc276ae52d423104b805b45175e10b5a8c` |
| 64 | 11 | `WorkBudgetExceeded`, 63 HVP | `3ae6257bdefc63e7b1360f43085dec11d7a9269f0624e7e499231b5a6aa5abd4` | `27adb09a1e1a2921688e8c2f451cb7cf681ff6a5df21a2a2c303b76ab357b7ea` |
| 128 | 92 | `position_max`, `5.211209258 mm` | `a3aca468e8eb5bef283db65e4891dc280970f50fa35596e57da6afd2ee6b4d05` | `e9f888f0a611e5985b8d3d2d79323d1699186af7e2ff8362c42f5eb6f963cc16` |

At budget 128 both GPU routes and the CPU solver return `None`, maximum GPU
and CPU work is 106 HVP, and coherent/permuted final compensated state roots
are both
`0a308959b3f5fda9e9c6146e597dbd72e26256d8bf1f7da73032ed8e5ffb011f`.
The status is `REFUTED_BOUNDED`; timing is exactly `NOT_RUN`.

## Stopped stages and remaining uncertainty

Per the frozen first-failure order, dam-break, orifice, 16k/50k capacity,
240-step 50k, performance samples, Compute Sanitizer and independent review
were not run. The old neighbor-only `~1.0--1.18 ms p95` result remains a graph
component measurement and is not a full-solver estimate.

The failure distribution is not yet localized. The next bounded research must
identify the exact outlier sample/component, distinguish accumulated
active-set sensitivity from boundary/contact or CPU/GPU semantic mismatch,
and compare one synchronized pre-failure state. It must not loosen the 5 mm
gate or start 50k timing under the refuted NCGP4 claim.
