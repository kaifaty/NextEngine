# NSR3-B4EP0 -- nominal cost-attribution contract

Status: `FROZEN / EXTERNAL_PROFILE_AUTHORIZED / NO_OPTIMIZATION`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep0-gprof-attribution|v1|parent=0cdc26e1d0b406fecc39c64cebfee080be32804b993e6d433fa0a74658efb4cc:54d42af619dbd48ae400ce4656ba155a8726dbd07b00754ecfc137ee864c111d:b9601aaad292c43201a5ab054192de4131478eb27e568e1eb78e6b613b07eecc|implementation=45111bd9662eeb931c80dfd601e9ef08190c18ab|sources=353d35eca5867ae6dcd5b2d87f062eed2bc3417054124562614f209b9a653b90:ed599037dfe01a262bfbaadddbe31024d7164f83eaed2595563e755f76a735e6:29cf1319afe2320aed47226ea7c5abb911c9c9b1ccac33d04bc201ace3133593|toolchain=gcc15.2;release=-O3,-DNDEBUG,-g,-pg;gprof=2.46|run=one-b4e1m;watchdog=900s|gate=stdout-byte-exact;gmon-nonempty;flat-self-samples>0|perf-event=unavailable-paranoid4;sysctl-unchanged|timing=external|credit=b4ep1-design-only
```

Identity SHA-256:
`bf65f79d98c6c9802da5a853d57f77b5cdf8d8fac175da9c9703efde7e667b60`.

## Build and run

Create an external clean CMake Release build of implementation commit
`45111bd...c18ab` with:

```text
CMAKE_CXX_FLAGS_RELEASE=-O3 -DNDEBUG -g -pg
CMAKE_EXE_LINKER_FLAGS=-pg
```

Use `/usr/bin/c++` GCC 15.2.0 and GNU gprof 2.46. Build only
`nonlocal-formula-reclosure`. Run exactly:

```text
nonlocal-formula-reclosure --nominal-hydro-macro-probe
```

from an empty external run directory under a 900-second watchdog and external
`/usr/bin/time -v`. Do not modify `perf_event_paranoid` or any other host
setting.

## Admission and artifacts

Require:

- process exit zero;
- stdout exactly 6,151 bytes with SHA-256
  `b9601aaad292c43201a5ab054192de4131478eb27e568e1eb78e6b613b07eecc`;
- semantic result
  `54d42af619dbd48ae400ce4656ba155a8726dbd07b00754ecfc137ee864c111d`;
- nonempty regular `gmon.out` and at least one nonzero flat-profile sample;
- generated flat/call-graph text plus their hashes, kept outside Git;
- compiler/link command evidence and instrumented executable hash/Build ID.

Report wall/RSS and gprof sample shares as external evidence. They are not
deterministic gates except for existence/nonzero sampling. Do not compare the
instrumented wall time numerically with the Release PASS threshold.

## Exit

PASS ranks observed leaf functions by self time and maps them to hashing,
neighborhood/evaluation/tape, HVP or solver bookkeeping. Freeze exactly one
B4EP1 Release ablation from that attribution. A stdout mismatch is
`PROFILER_CORRESPONDENCE_FAIL`; an empty profile is `PROFILER_NO_SAMPLES`.
Neither failure authorizes optimization by assumption.

Keep B4E2 execution, reference decode, runtime/CUDA integration and production
claims blocked.
