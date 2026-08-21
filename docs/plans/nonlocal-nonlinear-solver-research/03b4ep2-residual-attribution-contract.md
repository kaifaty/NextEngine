# NSR3-B4EP2 -- work-only residual-attribution contract

Status: `FROZEN / PROFILE_AUTHORIZED / NO_OPTIMIZATION`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep2-residual-attribution|v1|parent=470a0f4ec9b51ac57f1ecb69e937bb9d2756db41299a0dc01fa427ae63a78e99:25b1f00c0c7477a03532dca2acb97314853e9b4184962d9695792273280f5e04:4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112|implementation=5e40aa26e6ba0fcf2d0f3c4ca8d6f77857a660f1|sources=0315a2bae59f82cdef3934bc39a9f1ee4b51ef4170944f96b1b4da095004e840:58d6b7b4412b5f8775bd61ad78b7c8372bbeb0dfaf8175de965bc5bc722e5c20:b52802e8bb3119e8744117d92b621573391890462d083b95d61248730305cc8a|toolchain=gcc15.2;release=-O3,-DNDEBUG,-g,-pg;gprof=2.46|run=one-work-only;watchdog=900s|gate=stdout-byte-exact;gmon-nonempty;flat-self-samples>0|timing=external|decision=one-next-ablation|credit=b4ep3-design-only
```

Identity SHA-256:
`3268d59c30c11f892e45c5d781989fb085d54b7d5924069285fa4b56e8d0196d`.

## Build and run

Create a separate external clean CMake Release build of implementation commit
`5e40aa26e6ba0fcf2d0f3c4ca8d6f77857a660f1` with:

```text
CMAKE_CXX_FLAGS_RELEASE=-O3 -DNDEBUG -g -pg
CMAKE_EXE_LINKER_FLAGS=-pg
```

Use `/usr/bin/c++` GCC 15.2.0 and GNU gprof 2.46. Build only
`nonlocal-formula-reclosure`. From an empty external run directory, run once
under the existing 900-second watchdog and external `/usr/bin/time -v`:

```text
nonlocal-formula-reclosure --nominal-hydro-query-evidence-ablation
```

Do not alter `perf_event_paranoid` or any other host setting.

## Admission and artifacts

Require:

- source hashes and identity projection exactly as frozen above;
- process exit zero;
- stdout exactly 5,780 bytes with SHA-256
  `4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112`;
- semantic result
  `25b1f00c0c7477a03532dca2acb97314853e9b4184962d9695792273280f5e04`;
- nonempty regular `gmon.out` and at least one nonzero flat-profile sample;
- generated flat/call-graph text, their hashes, compiler commands,
  instrumented executable hash and GNU Build ID.

Report wall/RSS and category shares externally. They are descriptive and must
not be compared numerically to Release B4EP1 timing. Preserve call counts and
map optimized/folded symbols using call-graph children before assigning a
category.

## Exit

Rank HVP, topology/CSR construction, evaluation/tape refresh and nonlinear
bookkeeping by observed self/inclusive cost. Select exactly one B4EP3 design,
or select scoped timing if attribution is ambiguous. A stdout mismatch is
`PROFILER_CORRESPONDENCE_FAIL`; an empty profile is `PROFILER_NO_SAMPLES`.

B4E2 execution, reference decode, runtime/CUDA integration and production
claims remain blocked.
