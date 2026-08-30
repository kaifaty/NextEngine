# NCGP3 compensated-scale evidence — 2026-08-30

| Field | Value |
| --- | --- |
| Status | `FINAL INCONCLUSIVE / INDEPENDENT RE-REVIEW NO-GO` |
| Candidate commit | `b74688b296cc6adfda1e975cbed45f31a0d9d983` |
| Candidate tree | `780f4378745d8d8a4c888133384014bf28f2f842` |
| Frozen contract aggregate | `d0e6875c9c894a89f9f78f3e220aff9b555fb0235486e5b3899413cec835477e` |
| Candidate source root | `c6306d51c974706d1665596a167e6abc16fa7d37df8d33855fa8f3b17009e78a` |
| Clean Release binary | `a6a94a1c897bab00e71a489d1702e13a36e20de6e42b40315af8c6668ff06e85` |
| Environment | RTX 3080, SM 8.6, CUDA runtime/driver `13030` |
| Claim ceiling | corrected compensated FCR2 correctness only; 50k/performance `NOT_RUN` |

## Result

The single revision-5 repair batch closed several initial review apparatus
defects:

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

All implemented pre-trajectory gates and the 16-step hydrostatic route pass.
The author run then observed a deterministic 16-step dam-break failure at step
6. The independent revision-5 re-review rejected the claimed first-specific
result: the frozen corpus requires 240 steps per 4k scenario, and the omitted
`hydrostatic-hold 240` route fails earlier, at step 39. The re-review also found
load-bearing gaps in the reversible-energy control, root closure and
fail-closed CUDA rollback.

The one repair and one re-review allowance is exhausted. NCGP3 therefore
closes `INCONCLUSIVE`; neither hydro nor dam is promoted as a verified physical
refutation. Their deterministic work-ceiling failures remain useful successor
evidence.

The author package incorrectly omitted the mandatory 240-step hydro route.
The reviewer ran it and correctly stopped before 50k/performance. The earlier
`~1.0–1.18 ms p95` result remains neighbor-builder timing only and is not a
full-water result.

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

### Author-observed 4k dam-break failure

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

This route is reproducible but is not the first frozen failure because the
author ran only 16 hydro steps.

### Independent frozen 4k hydrostatic result

The reviewer built exact source commit `b74688b2` twice and ran
`--correspondence-4k hydrostatic-hold 240 128`. Both runs were exact:

| Observable | Value |
| --- | --- |
| Status / completed steps | `PHYSICS_REFUTED / 38`; failure at step 39 |
| Corrected / permuted failure | `WorkBudgetExceeded / WorkBudgetExceeded` |
| CPU failure | `None` |
| Corrected / permuted HVP | `126 / 126` of 128 |
| Shared work root | `0687e23dc524a83858a04d58b9cfd524a0e639c3ba282bf4cb7429a383c2d128` |
| Restored public state root | `eeab4153c3f7ff783853350d506b8878523f2fb0ab44d85e6d3761cc72d3763f` |
| Stdout SHA-256 | `313d5eb78babfd34b407876248c90881dca0a5c75f63308e707e781e8184c834` |

This invalidates only the author's first-failure ordering claim. It does not
promote hydro as verified physics-refuted because the remaining control/root
defects are load-bearing.

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

## Independent re-review verdict

The single revision-5 re-review returned `NO-GO / INCONCLUSIVE` with four
load-bearing findings:

1. the author changed the frozen 4k trajectory length from 240 to 16 and thus
   missed the earlier hydro step-39 ceiling;
2. `reversible_energy_drift` contains only a forward free-fall positive-excess
   calculation and no reverse leg; the viscosity threshold is absolute rather
   than the frozen relative `1e-6`;
3. physics and trajectory roots omit gated metrics, identities and work fields,
   while the failure-restored root covers rounded public state rather than the
   full compensated transaction state;
4. CUDA transaction restoration ignores errors from its own copies and final
   synchronization, so a restoration failure is not fail-closed.

Positive independent closure: exact source/contract identities, two clean
builds, all implemented/retained controls and three sanitizers reproduced. The
reviewer binaries were identical to each other (`c7f2bcc8b1135ae4440ed2f31796876ea9287575a7a7dd633664f45275d5d27b`);
their difference from the author binary was traced to an anonymous-namespace
CUDA source-path token, with normalized SASS identical.

No further repair is allowed in NCGP3. A successor may reuse these observations
only under a new frozen contract that restores the 240-step corpus, implements
a real reversible leg, seals every gate/work/result field and makes rollback
failure typed and fail-closed. Full-solver performance remains unknown.
