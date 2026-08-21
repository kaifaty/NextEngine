# NSR3-B4EP8 -- fused residual-attribution contract

Status: `CLOSED / PASS / B4EP9_PHASE_TIMING_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep8-fused-residual-attribution|v1|parent=441ac76483748631f94ae4f2d092b97ba4d271fb2c380d2b6e889245527821c9:de432459df752b4dd8d1b8ffc186718178fc68e65f19d503d1dffd0137f965bd:e3453dc158a8eace69b468ac84641cccdcad221dcc05695d06b6f400039e775c:8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095|implementation=a3aa054217cfd9d21193f8943effe10b7ebce7bf|sources=fb603f7729b1a653e0badec3dd06ef47a4f4458214c49b5678d63da79b474e67:11df6d9de8f6c924ab080ca8bd852c52b43cedbba0a3e67b1ecc22539e9a8fd0:fc9e3e05d6e235d2546755d0918cabbc229d6d342654c4cc0328519df3e9f58e|toolchain=gcc15.2;release=-O3,-DNDEBUG,-g,-pg,-ffp-contract=off,-fno-fast-math;gprof=2.46|run=one-b4ep7i;watchdog=900s|gate=stdout-byte-exact;gmon-nonempty;flat-self-samples>0|categories=hvp;fused-workspace;topology-filter-csr;fused-pair;fused-center;control|selection=top-leader>=1.20x-runner-up;workspace-subleader>=1.20x;otherwise=phase-timing|timing=external|decision=one-next-design|credit=b4ep9-design-only
```

Identity SHA-256:
`d8c6584ee93f8b3232441c061bacfb6ddb8f50d5a80805a57c6b53b0968a65e1`.

## Build and admission

Use:

```text
CMAKE_CXX_FLAGS_RELEASE=-O3 -DNDEBUG -g -pg -ffp-contract=off -fno-fast-math
CMAKE_EXE_LINKER_FLAGS=-pg
CMAKE_EXPORT_COMPILE_COMMANDS=ON
```

Build only `nonlocal-formula-reclosure`, then run the fused command once from
an empty external directory under `/usr/bin/time -v` and the 900-second
watchdog. Require exit zero, empty stderr, stdout exactly 6,714 bytes and
SHA-256
`8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095`,
semantic result
`e3453dc158a8eace69b468ac84641cccdcad221dcc05695d06b6f400039e775c`,
nonempty `gmon.out` and nonzero flat samples.

Record compiler commands, executable hash/Build ID and flat/call-graph hashes.
Instrumented wall/RSS are descriptive only.

## Attribution and exit

Report HVP, complete fused workspace and control. Split workspace into
topology/filter/CSR, fused pair and fused center work when symbols permit.
Select one next design only when the leader is at least `1.20x` runner-up;
otherwise select `SCOPED_INTERNAL_PHASE_TIMING` and no optimization.

B4E2, references, runtime/CUDA, parallelism, solver-policy and production
remain blocked.

Observed PASS: workspace 3.61 s and HVP 3.59 s differ by only `1.0056x`;
gprof also cannot split the inlined fused pair/center loops. See the
[dated evidence](../../development/nonlocal-nsr3b4ep8-fused-residual-attribution-evidence-2026-08-22.md).
