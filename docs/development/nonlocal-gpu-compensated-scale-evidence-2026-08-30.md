# NCGP3 compensated-scale evidence — 2026-08-30

| Field | Value |
| --- | --- |
| Status | `CANDIDATE PHYSICS_REFUTED / INDEPENDENT RE-REVIEW PENDING` |
| Candidate commit | `b74688b296cc6adfda1e975cbed45f31a0d9d983` |
| Candidate tree | `780f4378745d8d8a4c888133384014bf28f2f842` |
| Frozen contract aggregate | `d0e6875c9c894a89f9f78f3e220aff9b555fb0235486e5b3899413cec835477e` |
| Candidate source root | `c6306d51c974706d1665596a167e6abc16fa7d37df8d33855fa8f3b17009e78a` |
| Clean Release binary | `a6a94a1c897bab00e71a489d1702e13a36e20de6e42b40315af8c6668ff06e85` |
| Environment | RTX 3080, SM 8.6, CUDA runtime/driver `13030` |
| Claim ceiling | corrected compensated FCR2 correctness only; 50k/performance `NOT_RUN` |

## Result

The single revision-5 repair batch closes the initial review apparatus defects:

- accepted steps rebuild `predicted` as a canonical device `(hi, lo)` pair;
- an ordinary binary32 predictor is a reachable negative control;
- the CPU long-double solver owns a local admission validator;
- the hot GPU `step()` transfers only scalar decisions; full state, density and
  active IDs use a separately sealed diagnostic snapshot;
- work receipts seal predictor EFT work and allocated device bytes;
- trajectory reports seal state/work/result/failure roots, source/binary/build
  identity, transfers, graph size, active counts, momentum, energy and
  penetration;
- free fall, translation/rotation, tangential `mu=0`, pressure-inactive surface
  relaxation and identical-state HVP controls run before every 4k route.

All pre-trajectory gates and the 16-step hydrostatic route pass. The next
sequential gate, 16-step dam break, fails at step 6 in both canonical and
permuted GPU routes with `WorkBudgetExceeded`. Both routes use 127 of the
frozen 128 total-HVP budget, publish identical work roots and restore the exact
prior public state. The independent CPU oracle succeeds. This is therefore a
candidate first-specific `PHYSICS_REFUTED` result, subject to the one permitted
independent re-review.

Per the frozen stop rule, orifice, 240-step, 50k and performance measurement
were not run. The earlier `~1.0–1.18 ms p95` result remains neighbor-builder
timing only and is not a full-water result.

## Numerical gates

### Pre-trajectory physics suite

The two clean builds emitted byte-identical physics JSON, SHA-256
`c69c843b57108d88f0400c0a3e2d125bb23497db23387112a58c38f7ff5993dc`:

| Gate | Result |
| --- | --- |
| 32-step free fall | PASS; position max `1.0251695 um`, velocity max `1.8501282e-7 m/s` |
| Reversible energy drift | `0` |
| Free-fall momentum residual | `1.5417735e-8` |
| 8-step translation | PASS; max `0.2731428 um` |
| 8-step +90-degree rotation | PASS; max `0.0119209 um` |
| Corrected/permuted identity | PASS; zero mismatches |
| Tangential viscosity, selected `mu=0` | PASS; final kinetic change `8.1227813e-19 J` |
| Pressure-inactive surface relaxation | PASS; GPU/CPU max `0.0457458 um`, energy decrease `9.3749394e-9 J` |
| Identical-state FCR2 HVP | PASS; relative L2 `8.8377346e-7`, cosine loss `3.8715981e-13`, exact mixed active set |
| Ordinary predictor negative | PASS; corrected EFT components `6`, ordinary `0`, distinct state roots |

### 4k hydrostatic hold

Both clean builds completed 16/16 steps and emitted byte-identical JSON,
SHA-256 `a37869a9aaa1b868ec223ddb1b41f8e68dd1748fd8d6a1120e2a4803d13d0056`:

- status `PASS`;
- maximum CPU/GPU position error `8.2191719 um`;
- maximum normalized momentum residual `0.0008997864` (`0.090%`);
- maximum positive mechanical-energy excess `0`;
- maximum penetration `0`;
- maximum compression-density error `0.001322876`;
- corrected/permuted state and active-ID mismatches `0`;
- maximum 105 HVP, 415,248 directed pairs and degree 123.

The three reported CPU/GPU active-ID mismatch steps begin only after the CPU
and GPU accepted states differ numerically; revision 5 freezes these as a
diagnostic, not a trajectory rejection. Corrected/permuted GPU active IDs are
exact at every step.

### First failing gate: 4k dam break

Both clean builds emitted byte-identical JSON, SHA-256
`620a80cc6fe8c6eac0dfd226e4b1fb7b67c115949d151f2ed3241e969dc83cf5`:

| Observable | Value |
| --- | --- |
| Status / process exit | `PHYSICS_REFUTED` / `37` |
| Completed steps | `5`; first failure is step `6` |
| Corrected / permuted failure | `WorkBudgetExceeded` / `WorkBudgetExceeded` |
| CPU failure | `None` |
| Corrected / permuted HVP used | `127 / 127` of 128 |
| Shared failure work root | `cf626e52f75c919e0fd2d180b99e268df77a4b4635c2528504dbac6f0e2dc140` |
| Restored/final state root | `e445483f6919f409f94576e54b9190ae8b62e60e71bccda36c9da9a145070b25` |
| Result root | `b8d21fef298ee3d75d956eee325766a1ac45becf2567c4de38ce92bd52bf15b1` |
| Position error before failure | `1.6595623 um` |
| Momentum residual before failure | `0.0002518165` |
| Positive energy excess / penetration | `0 / 0` |
| Allocated device bytes per workspace | `136251397` |
| Hot H2D / D2H for two routes | `0 / 188328` bytes across attempted steps |
| Diagnostic snapshot D2H | `1536000` bytes, separately sealed and untimed |
| Maximum directed pairs / degree | `401248 / 123` |

## Build, retained controls and sanitizers

Two fresh Release directories produced byte-identical stripped binaries. Each
ran `physics -> hydrostatic-hold 16 128 -> dam-break 16 128` with exit sequence
`0, 0, 37` and byte-identical outputs.

Retained controls were also byte-identical across both builds:

| Control | Exit / result | Output SHA-256 |
| --- | --- | --- |
| NCGP1 graph | `0 / PASS` | `85667e2b238b7b355c88407c8685b6ea095ed10b5d356df3b56b343ddac31c88` |
| NCGP1 operator | `0 / PASS` | `7a2fc16a2eb7133df7cc4867dd70d5084183996ff9e1dc1203b7c602e169781b` |
| NCGP1 solver | `0 / PASS` | `602a73f6b5c6eee0232c3f907cefbd5d83b960fa443d7cb5a5a9729c8fbc061e` |
| NCGP2 phase A | `0 / PHASE_A_PASS` | `412e6496fd455adf5b3b8c74aab184f139b010e4c5cde3a096a3d54e6f6c691d` |
| NCGP2 surface-f64 discriminator | `4 / expected FAIL` | `054e94cfe29b1a53744633e55c87650079463b8ae6b1626969b7f80ae8576add` |
| NCGP2 compensated state | `0 / PASS` | `db0913cbcbb3f25b21cae2b96b0b48d86254eec8924d5f366f7bcff81c274a2b` |

`compute-sanitizer` memcheck, initcheck and synccheck each ran the complete
physics suite, exited 0 and reported `ERROR SUMMARY: 0 errors`. Their stdout
was identical, SHA-256
`477fd745715ba2b863c4b71be63a7e0a8f593de10bca4932dfd09b1888448d6a`.

## Exact commands

```text
cmake -S crates/continuum-water/tools/nonlocal-feasibility -B <fresh> -DCMAKE_BUILD_TYPE=Release
cmake --build <fresh> --target nonlocal-corrected-cuda-compensated-scale -j2
<fresh>/nonlocal-corrected-cuda-compensated-scale --physics-self-test
<fresh>/nonlocal-corrected-cuda-compensated-scale --correspondence-4k hydrostatic-hold 16 128
<fresh>/nonlocal-corrected-cuda-compensated-scale --correspondence-4k dam-break 16 128
compute-sanitizer --tool {memcheck,initcheck,synccheck} --error-exitcode=<nonzero> <binary> --physics-self-test
```

## Remaining uncertainty

The independent re-review must confirm that the repaired predictor, hot-step
boundary, work roots, trajectory metrics and first-failure classification
match the frozen revision-5 contract. Until that verdict, this document does
not promote the candidate to a verified result. Even a verified refutation
would say only that the frozen 128-HVP primary profile cannot complete the dam
gate; it would not establish full-solver frame time or production readiness.
