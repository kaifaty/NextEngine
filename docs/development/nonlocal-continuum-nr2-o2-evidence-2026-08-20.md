# Nonlocal continuum NR2-O2 term-specialization evidence — 2026-08-20

Status: `O2_RETAINED_TERM_SPECIALIZATION / O3_NEXT / REPORT_ONLY`

## Scope and identity

This report executes the ordered gates in the
[NR2-O2 specification](../plans/nonlocal-continuum/05-nr2-o2-term-specialization.md).
It compares two explicit term-kernel identities on the retained gather/swap
path:

```text
baseline  = nuv-gather-directed-r0 + pointer-swap-o1 + nuv-terms-runtime-v0
candidate = nuv-gather-directed-r0 + pointer-swap-o1 + nuv-terms-specialized-o2
```

O2 changes only the viscosity kernel's bulk/shear mask from runtime arguments
to three closed template instantiations. It does not fuse pair passes, remove
clears, change the full `3x3` local system, alter equations, reorder terms or
add device storage. The no-viscosity case still issues no viscosity launch.
Commands without `--term-kernels` retain the runtime identity.

| Item | Exact identity |
|---|---|
| O2 specification checkpoint | `e3f6af58ef8162a15dc188f7e2200b7df63ae747` |
| O2 implementation checkpoint | `053e1e739352d64940729809ee7f126141372ae1` |
| Standalone binary SHA-256 | `ae4442743a27a486b1f880bdbbdb6125356edade3e74a83009c661e698f4c389` |
| Standalone binary size | `1,834,696 B` |
| Host | Linux x86-64; NVIDIA GeForce RTX 3080 10 GB; compute capability `8.6` |
| Device | 68 SMs; `10,351,214,592 B` global memory |
| Toolchain reported by binary | CUDA compiler/runtime `13.3.73` / `13.3`; CCCL/CUB `3.3.4`; driver API `13.3` |
| CUDA compile contract | target property `sm_86`; `-O3 --fmad=false --prec-div=true --prec-sqrt=true --ftz=false` |

The build and all reports were external to the repository. Raw profiler files
remain under `/tmp` and are identified by hash below.

## Ordered correctness gates

No retention timing ran before the complete correctness matrix passed on the
final binary.

| Gate | Result |
|---|---|
| `O2-BUILD` | PASS; final CMake/Ninja rebuild reported no pending work and reproduced the binary hash above |
| `O2-LEGACY` | default atomic/copy/runtime and gather/swap/runtime tiny controls each PASS 11/11; atomic+specialized and gather/copy+specialized are both rejected |
| `O2-TINY-MASKS` | specialised tiny PASS 11/11 including bulk-only, shear-only and bulk+shear; runtime correspondence and candidate reuse are exact; no CSR or memory change |
| `O2-SURFACE-I2` | ten cold repeats PASS and exact; runtime/specialized correspondence and CSR exact |
| `O2-SURFACE-I20` | ten cold repeats PASS and exact; runtime/specialized correspondence and CSR exact |
| `O2-FULL` | water-16k/48k at five iterations and viscous-16k at twenty pass finite, topology, capacity, local-solve and momentum gates; reused specialised executions and runtime correspondence are exact |

The surface outputs remain
`52a3d852c05b9cc7931a3133816e0ddb1445695b88ea25193d0981022080b65e`
at two iterations and
`0f16c58f72fd3e31e2bd13bf12899359dbafdb90d06fdeef947371671ead7dbd`
at twenty. Their shared CSR SHA-256 is
`a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698`.

### Full fixed-work controls

| Profile | Profile / input SHA-256 | Samples / pairs / max degree | Ordered output SHA-256 | Momentum residual | Memory | Result |
|---|---|---:|---|---:|---:|---|
| water-16k, i5 | `8445417063a80ac1626632de49d74f349f6690f7ed135bc20090106534f74dcb` / `f6b63269db1555ec4d4ad8be8041da088c7d9bd9380fa15e47504c4e0c4bb975` | `16,000 / 1,699,688 / 123` | `675efc7f729e0e0e496f8000b809858573a87315834a909c3035f2e2424d68eb` | `7.4952794e-8` | `10,914,054 B` | PASS |
| water-48k, i5 | `cb1868b86b4d9d40e996ebfa8e529982f73e647e358dd9f0a7c9269fce1211e8` / `1be1a98e586bae70a2ffa72eb56d4b79071e404a546616811eea1764c1e1ce51` | `48,000 / 5,190,588 / 123` | `03671a4fccf72958ff772dfde7870ad24e8a8678c669f2a1286a178676203d1b` | `5.3916989e-8` | `32,725,766 B` | PASS |
| viscous-16k, i20 | `c91b60682a8722085f6d244eeec3588812e0f9d55eb0cd29b27c14cf6432cc24` / `9b6297eb6e923d822588a1d900fb16b55e8bae2d0ac577ed9897bd77f7afd11c` | `16,000 / 1,699,688 / 123` | `6f8e97c8f4db11cd759628a078fefbc0177d34110b494c99f79fb03ca2d9fdb6` | `1.7105814e-7` | `10,914,054 B` | PASS |

Both executions on each reused candidate instance have the displayed digest.
The separately constructed runtime baseline has the same digest, CSR and
device-memory byte count. Density, energy, source, matrix, position and
velocity correspondence are bit-exact in all three controls.

## Adjacent retention timing

Each table is the required adjacent `runtime-v0` then `specialized-o2` pair
from the same final binary and device state. Each command used five warm-ups
and 50 measured runs. Values are CUDA device-timeline milliseconds in
`minimum / median / p95 / p99 / mean` order. Totals include neighbor
construction. Every command passed tiny preflight and exact repeated output.

### Water 16k, five fixed iterations

| Stage | `runtime-v0` | `specialized-o2` |
|---|---:|---:|
| prediction | `0.006144 / 0.007232 / 0.009536 / 0.010688 / 0.007615` | `0.005888 / 0.007104 / 0.010848 / 0.011776 / 0.007588` |
| neighbor construction | `0.279552 / 0.280576 / 0.284672 / 0.287744 / 0.281356` | `0.246784 / 0.247808 / 0.249856 / 0.249856 / 0.248157` |
| density | `0.519168 / 0.521216 / 0.523264 / 0.524288 / 0.521080` | `0.474112 / 0.476384 / 0.479232 / 0.480256 / 0.476660` |
| buffer reset | `0.032768 / 0.033792 / 0.034848 / 0.035840 / 0.033748` | `0.029856 / 0.032768 / 0.034816 / 0.035840 / 0.032874` |
| incompressibility | `0.532480 / 0.536576 / 0.538624 / 0.539648 / 0.536199` | `0.481280 / 0.484352 / 0.486400 / 0.487424 / 0.484014` |
| viscosity | `0.622624 / 0.624672 / 0.626688 / 0.627712 / 0.624923` | `0.541824 / 0.544768 / 0.546816 / 0.547840 / 0.544954` |
| surface tension | `0 / 0 / 0 / 0 / 0` | `0 / 0 / 0 / 0 / 0` |
| local update | `0.039040 / 0.040960 / 0.043008 / 0.044032 / 0.041076` | `0.034816 / 0.037888 / 0.038912 / 0.039936 / 0.037622` |
| state handoff + velocity | `0.010240 / 0.013312 / 0.015360 / 0.015360 / 0.013190` | `0.009216 / 0.012288 / 0.014336 / 0.014336 / 0.012168` |
| total | `2.112544 / 2.118144 / 2.125920 / 2.129056 / 2.118580` | `1.895168 / 1.899872 / 1.903392 / 1.904320 / 1.900004` |

Viscosity p95 decreases `12.745%`; total p95 decreases `10.467%`.

### Water 48k, five fixed iterations

| Stage | `runtime-v0` | `specialized-o2` |
|---|---:|---:|
| prediction | `0.006944 / 0.010272 / 0.015136 / 0.161792 / 0.013676` | `0.006592 / 0.011232 / 0.012576 / 0.021920 / 0.010298` |
| neighbor construction | `0.442368 / 0.490496 / 0.496640 / 0.498688 / 0.487423` | `0.440320 / 0.446464 / 0.454656 / 0.463872 / 0.447696` |
| density | `0.816128 / 0.832544 / 0.836800 / 0.839680 / 0.832253` | `0.816128 / 0.821248 / 0.828416 / 0.834560 / 0.822366` |
| buffer reset | `0.034944 / 0.038912 / 0.039968 / 0.040960 / 0.038777` | `0.033792 / 0.035840 / 0.037888 / 0.039936 / 0.036231` |
| incompressibility | `0.885760 / 0.901120 / 0.910336 / 0.916480 / 0.900849` | `0.883712 / 0.894976 / 0.908288 / 0.910336 / 0.895700` |
| viscosity | `1.226752 / 1.256448 / 1.280000 / 1.290304 / 1.257611` | `1.209344 / 1.238016 / 1.251328 / 1.255488 / 1.235412` |
| surface tension | `0 / 0 / 0 / 0 / 0` | `0 / 0 / 0 / 0 / 0` |
| local update | `0.052224 / 0.056320 / 0.058368 / 0.059392 / 0.055997` | `0.051200 / 0.053248 / 0.055296 / 0.055328 / 0.053302` |
| state handoff + velocity | `0.011264 / 0.014336 / 0.015360 / 0.016384 / 0.014036` | `0.011264 / 0.013312 / 0.014400 / 0.015360 / 0.013219` |
| total | `3.553248 / 3.664480 / 3.697216 / 3.799040 / 3.659182` | `3.535808 / 3.573120 / 3.594816 / 3.603040 / 3.570622` |

Viscosity p95 decreases `2.240%`; total p95 decreases `2.770%`.

### Viscous 16k, twenty fixed iterations

| Stage | `runtime-v0` | `specialized-o2` |
|---|---:|---:|
| prediction | `0.006304 / 0.008640 / 0.012800 / 0.013888 / 0.009382` | `0.006848 / 0.008544 / 0.013184 / 0.017664 / 0.009523` |
| neighbor construction | `0.246784 / 0.249856 / 0.281600 / 0.282624 / 0.262084` | `0.247808 / 0.249888 / 0.252928 / 0.258048 / 0.250413` |
| density | `1.638592 / 1.646656 / 1.793024 / 1.796096 / 1.703521` | `1.645568 / 1.651808 / 1.654976 / 1.656032 / 1.651460` |
| buffer reset | `0.128544 / 0.134560 / 0.142944 / 0.144384 / 0.136014` | `0.126272 / 0.130688 / 0.134560 / 0.138240 / 0.131016` |
| incompressibility | `1.933312 / 1.941824 / 2.139136 / 2.362464 / 2.022787` | `1.941504 / 1.946752 / 1.950720 / 1.952768 / 1.947279` |
| viscosity | `2.238464 / 2.247680 / 2.495488 / 2.496512 / 2.343674` | `2.214304 / 2.222080 / 2.226336 / 2.228224 / 2.221700` |
| surface tension | `0 / 0 / 0 / 0 / 0` | `0 / 0 / 0 / 0 / 0` |
| local update | `0.148480 / 0.155648 / 0.167936 / 0.169024 / 0.158251` | `0.147456 / 0.152608 / 0.156896 / 0.157888 / 0.152502` |
| state handoff + velocity | `0.031744 / 0.036864 / 0.040960 / 0.040960 / 0.037089` | `0.029696 / 0.036864 / 0.041984 / 0.044032 / 0.037097` |
| total | `6.591456 / 6.610816 / 7.267360 / 7.490368 / 6.877369` | `6.593184 / 6.601408 / 6.608064 / 6.613984 / 6.601540` |

Viscosity p95 decreases `10.786%`; total p95 decreases `9.072%`.

## Ordering sensitivity control

The first adjacent pair showed movement in stages O2 cannot change. A
non-gating reverse-order run and a second control with 100 warm-ups were
therefore recorded before attribution. The reverse-order water-16k pair
flipped direction (`0.606208` versus `0.566272 ms` viscosity p95), confirming
that a short process can sample GPU clock-state movement. With 100 warm-ups,
specialised viscosity p95 was lower on all three profiles; total medians were
also lower, while water-48k total p95 was effectively flat but slightly worse
(`3.597504` versus `3.592288 ms`, `+0.145%`).

These extra runs do not replace the frozen 5+50 retention pair. They limit the
interpretation: O2 is retained because the required gate passes and the
same-process profiler below independently attributes lower viscosity-kernel
time. Unchanged-stage movement and the full adjacent total reduction are not
credited to specialization. A future benchmark harness should hold both
identities in one process or establish a longer device preconditioning rule.

## Post-O2 profiler attribution

Nsight Systems `2026.1.3.425-261338342291v0` captured the runtime comparator
and two exact candidate executions inside each full-control process. The CUDA
kernel summary therefore provides same-process per-launch attribution:

| Profile / kernel | Launches | Mean per launch | Observation |
|---|---:|---:|---|
| water-48k runtime viscosity `<bulk runtime, shear runtime>` | 5 | `262.1568 us` | comparator |
| water-48k specialised viscosity `<true, false>` | 10 | `246.7264 us` | `5.886%` lower mean |
| viscous-16k runtime viscosity `<bulk runtime, shear runtime>` | 20 | `122.0049 us` | comparator |
| viscous-16k specialised viscosity `<true, true>` | 40 | `120.1273 us` | `1.539%` lower mean |

The specialised viscosity launch remains a material part of device time;
specialization does not move the overall bottleneck away from repeated
neighbor-based density/incompressibility/viscosity work. It removes a small
dispatch/arithmetic cost and does not reduce pair evaluations or launch count.

| Raw record | SHA-256 |
|---|---|
| `/tmp/nonlocal-o2-water48-nsys.nsys-rep` | `1fc9c5e3d3bcffe2a51ad8e0c75da1c47f60366424825c0c54fd2398ca3dac92` |
| water-48k Nsight Systems command output | `2efaed9c08a1e237ec0be45e2306c8cf8860858ce5157ccd17b54e6e20f4f06d` |
| `/tmp/nonlocal-o2-viscous16-nsys.nsys-rep` | `2d00dd3e5fc91b533107bfb1fa089ca979a30894b0e950ead6ed04e88a50a807` |
| viscous-16k Nsight Systems command output | `85a56d3351627b76bcb79a7dca33549738278e34798f12eb3b11558d49e65519` |

Nsight Compute `2026.2.1.0` was invoked with the Basic set and a filtered
specialised-viscosity launch on both profiles. The host denied access to GPU
performance counters with `ERR_NVGPUCTRPERM`; consequently no `.ncu-rep` was
created and registers, achieved occupancy, SM throughput and DRAM throughput
are deliberately not inferred. This is the specification's explicit
unavailable-counter record rather than a missing silent capture:

| Failed counter-capture log | SHA-256 |
|---|---|
| water-48k command output | `269ca9d3456620c5537de98705a2b5c6481ee6e256c677eaba621522c1577db8` |
| viscous-16k command output | `2c740388beaad9018db69b7a64d3c40b272f577854839a434ae51362b0c3b1ff` |

## Retention decision

| O2 retention condition | Observation | Result |
|---|---|---|
| all correctness gates pass | build, legacy, tiny masks, surface i2/i20 and full controls pass | PASS |
| memory does not increase | exact byte equality at 16k and 48k | PASS |
| viscosity p95 decreases on all profiles | `12.745%`, `2.240%`, `10.786%` | PASS |
| total p95 improves on water-48k and viscous-16k | `2.770%`, `9.072%` | PASS |
| post-O2 profiler record exists | Systems attribution captured; Compute counters explicitly unavailable | PASS WITH UNAVAILABLE COUNTERS |

O2 exits `O2_RETAINED_TERM_SPECIALIZATION`. Its adjacent geometric-mean
total-p95 speedup on the two HN-3 denominator profiles is `1.0635x`.
`nuv-terms-specialized-o2` becomes the O3 input, while
`nuv-terms-runtime-v0` stays selectable as the rollback comparator.

The retained adjacent p95 is `1.903392 ms` water-16k, `3.594816 ms`
water-48k and `6.608064 ms` viscous-16k. These all satisfy the separate
research-only `8 ms` local/48k cutoffs, but O2 does not establish aggregate
NR2 speedup, an NR4 decision, W2 credit, runtime integration, GPU authority or
production readiness. O1 and O2 percentages are not added across
non-adjacent runs.

## Commands

```text
cmake --build /tmp/nextengine-nonlocal-feasibility-build --parallel
nonlocal-feasibility --self-test
nonlocal-feasibility --self-test --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels nuv-terms-runtime-v0
nonlocal-feasibility --self-test --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
nonlocal-feasibility --repeatability nuv-surface-16k.v0 --iterations <2|20> --runs 10 --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
nonlocal-feasibility --check <profile> --iterations <5|20> --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
nonlocal-feasibility --benchmark <profile> --warmup 5 --runs 50 --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels <nuv-terms-runtime-v0|nuv-terms-specialized-o2>
nsys profile --trace=cuda,nvtx --sample=none --force-overwrite=true --output=<raw-prefix> nonlocal-feasibility --check <profile> ... --term-kernels nuv-terms-specialized-o2
ncu --target-processes all --set basic --kernel-name regex:accumulate_viscosity_term_gather_specialized --launch-count 1 --force-overwrite --export <raw-prefix> nonlocal-feasibility --check <profile> ... --term-kernels nuv-terms-specialized-o2
```
