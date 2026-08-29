# NSR3-B4EP4 -- cached residual-attribution contract

Status: `FROZEN / PASS / B4EP5_HVP_DESIGN_AUTHORIZED / NO_OPTIMIZATION`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep4-cached-residual-attribution|v1|parent=e617043b55349894356ce3eda95aca538e99aeec8b127c418d8273bc408a1474:99a7e4b183f844390dca81beba0d83585bf938cb08e0dd6bd6794c82079fd3fe:b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055|implementation=65ce739e17def07266f0d2f72b00178451a1fb3e|sources=6a0438fe298fb462fbb5f1e8cf00c0e2aeb54b009cc60d98874fa38d7d8231f5:edee9a3b1c1998d1324e476a62a9a96012fec290e80864f8162ef7c86b9e293f:ecb36bd8ca5e87af3236b4b5bd9868a0aca1ea39210ab835e74091474379bbbd|toolchain=gcc15.2;release=-O3,-DNDEBUG,-g,-pg;gprof=2.46|run=one-cached;watchdog=900s|gate=stdout-byte-exact;gmon-nonempty;flat-self-samples>0|categories=hvp;workspace-filter-csr;evaluation-tape;nonlinear-bookkeeping|selection=leader>=1.20x-runner-up;otherwise=phase-timing|timing=external|decision=one-next-design|credit=b4ep5-design-only
```

Identity SHA-256:
`4240148cadb765b33d6b72b55b84d523cbcce5456774dad1d3918faecdeee42e`.

## Build and run

Create a separate external clean CMake Release build at implementation commit
`65ce739e17def07266f0d2f72b00178451a1fb3e` with:

```text
CMAKE_CXX_FLAGS_RELEASE=-O3 -DNDEBUG -g -pg
CMAKE_EXE_LINKER_FLAGS=-pg
```

Use `/usr/bin/c++` GCC 15.2.0 and GNU gprof 2.46. Build only
`nonlocal-formula-reclosure`. From an empty external run directory, execute
once under the 900-second watchdog and external `/usr/bin/time -v`:

```text
nonlocal-formula-reclosure --nominal-hydro-cached-topology-ablation
```

Do not change `perf_event_paranoid` or any host execution policy.

## Admission and artifacts

Require:

- identity projection and all three source hashes exactly as frozen;
- process exit zero and empty stderr;
- stdout exactly 6,462 bytes with SHA-256
  `b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055`;
- semantic result
  `99a7e4b183f844390dca81beba0d83585bf938cb08e0dd6bd6794c82079fd3fe`;
- nonempty regular `gmon.out` and at least one nonzero flat-profile sample;
- generated flat/call-graph text, their hashes, compiler commands,
  instrumented executable hash and GNU Build ID.

Instrumented wall/RSS are descriptive only. Preserve exact call counts and
map optimized/folded symbols through callers and children before assigning a
category.

## Attribution and routing

Report inclusive HVP, complete cached workspace and residual bookkeeping
cost. Split workspace into superset/filter/CSR and evaluation/tape. Select one
next design only when the leading comparable category is at least `1.20x` the
runner-up. Otherwise authorize scoped internal phase timing and no
optimization.

A stdout mismatch is `PROFILER_CORRESPONDENCE_FAIL`; empty samples are
`PROFILER_NO_SAMPLES`. B4E2, reference decode, runtime/CUDA integration and
production remain blocked.

## Closed result

The exact-output profile records 1,112 samples. HVP owns 6.91 s (62.14%)
inclusive versus 3.95 s (35.52%) for complete cached workspace, a `1.749x`
lead that clears the frozen selection ratio. This authorizes only B4EP5 HVP
research/design. See the
[dated evidence](../../development/nonlocal-nsr3b4ep4-cached-residual-attribution-evidence-2026-08-22.md).
