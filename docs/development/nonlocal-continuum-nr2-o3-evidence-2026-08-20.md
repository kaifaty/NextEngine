# Nonlocal continuum NR2-O3 accumulation-layout evidence — 2026-08-20

Status: `O3_SEGMENTED_NUMERIC_MISMATCH / RETAIN_GATHER_DIRECTED / REPORT_ONLY`

## Scope and identity

This report executes the ordered stop gates in the
[NR2-O3 specification](../plans/nonlocal-continuum/06-nr2-o3-accumulation-layout-tournament.md).
It compares the retained O2 stack with one separately selectable layout:

```text
retained  = nuv-gather-directed-r0
            + pointer-swap-o1
            + nuv-terms-specialized-o2

candidate = nuv-unique-pair-segmented-o3
            + pointer-swap-o1
            + nuv-terms-specialized-o2
```

The candidate builds an immutable symmetric reverse-slot map over the frozen
CSR, evaluates only `i < j`, writes one 48-byte source/full-matrix fragment to
each endpoint slot, then reduces fragments in the original owner CSR order.
It neither changes equations nor fuses passes, clears or launches.

| Item | Exact identity |
|---|---|
| O3 specification checkpoint | `b2d03391f70473414ed89e6a56e7ab04688b0407` |
| O3 implementation checkpoint | `ffa3b02f0e9c839dcc7c7387fe0d64460394e987` |
| Standalone binary SHA-256 | `2bf9a4b0991868a925e686930406dbee51546cb2733166dc525e8009d7aa8732` |
| Standalone binary size | `1,979,992 B` |
| Surface profile / input SHA-256 | `9ad78788bee7dc17dcc06877bd6d2fbe7ffbee77dfc76d001f8c6ceefea1c038` / `a97e0e5be388b3a124843fc386746e4e51ffe68c5dfd1273c8f3d2fb9bc0ed9a` |
| Host | Linux x86-64; NVIDIA GeForce RTX 3080 10 GB; compute capability `8.6` |
| Device | 68 SMs; `10,351,214,592 B` global memory |
| Toolchain reported by binary | CUDA compiler/runtime `13.3.73` / `13.3`; CCCL/CUB `3.3.4`; driver API `13.3` |
| CUDA compile contract | target property `sm_86`; `-O3 --fmad=false --prec-div=true --prec-sqrt=true --ftz=false` |

Build trees, binaries and raw JSON reports remain outside Git.

## Ordered gate result

The final binary stopped at the first candidate failure, before retention
timing, as required by the O3 contract.

| Gate | Result |
|---|---|
| `O3-BUILD` | PASS; clean external CMake/Ninja rebuild |
| `O3-LEGACY` | PASS; default atomic/copy/runtime and retained gather/swap/specialized are both 11/11; segmented with copy, runtime terms or defaults rejects |
| `O3-TINY` | PASS 11/11 against the CPU oracle; every case corresponds to retained gather, three reused executions are exact, CSR/reverse mapping passes; bulk-only smoke plus shear-only and bulk+shear fixtures pass |
| `O3-SURFACE-I2` repeatability | candidate output exact 10/10; CSR exact 10/10; reverse map, topology, momentum, finiteness and local solve pass |
| `O3-SURFACE-I2` gather correspondence | **FAIL**; stop boundary selected before `O3-SURFACE-I20`, `O3-FULL` or an admissible tournament |

The candidate is deterministic. Its failure is correspondence to the retained
numerical path, not repeated-output spread:

| Surface-16k i2 field | Maximum absolute difference | Frozen field result |
|---|---:|---|
| density | `27.0174560546875` | FAIL |
| energy | `3.9294152064361004e-4` | FAIL |
| source | `5.9934458934785356e-2` | FAIL |
| local matrix | `7.3865547875922422e-2` | FAIL |
| position | `1.4183437451720238e-4 m` | FAIL |
| velocity | `1.4183425903320312e-1 m/s` | FAIL |

The exact retained gather digest is
`52a3d852c05b9cc7931a3133816e0ddb1445695b88ea25193d0981022080b65e`.
All ten cold segmented repeats and both reused check executions produce
`d37d7ecda4fae7a8343bd6697ec2d6f2119f3ed48ce94b847ce4c354acc4aa98`.
The segmented normalized momentum residual is `1.3269132e-8`, below the
unchanged bound.

## Work and capacity proof at the stop boundary

| Item | Observed | Required relationship |
|---|---:|---|
| samples / self slots | `16,000 / 16,000` | one self slot per sample |
| directed CSR samples | `1,699,688` | frozen profile topology |
| non-self directed samples | `1,683,688` | directed minus self |
| unique pairs | `841,844` | non-self directed divided by two |
| candidate pair evaluations per active term | `841,844` | one per unique pair |
| endpoint fragments per active term | `1,683,688` | one per physical non-self endpoint slot |
| candidate layout allocation | `102,336,004 B` | exactly `1,968,000 * 52 + 4` |
| complete candidate allocation | `113,250,058 B` | exact 16k ceiling |
| reverse-map setup | `1.204224 ms` | reported outside execution total |

The reverse validator checks slot range, owner/neighbor identity and
involution after every reconstructed CSR. No second fragment copy, owner-key
sort or O5 fusion was introduced.

## Diagnosis

The tiny CPU and gather comparisons, exact cold/reused candidate output,
valid reverse involution, exact work identities and low momentum residual do
not support a race, missing endpoint or asymmetric-pair diagnosis. The first
large stiff-surface mismatch instead occurs with a stable but different
floating association: O3 pre-adds the two named endpoint contributions inside
one fragment before owner reduction, while retained gather adds them
sequentially to the owner accumulator. Strong surface tension amplifies that
difference across the fixed-point iterations.

This does not make the segmented trajectory invalid physics in isolation. It
does make it invalid as the frozen O3 replacement, because O3 required full
field correspondence to the already reclosed gather identity. The contract
explicitly forbids widening tolerances or changing association again after
this boundary.

## Timing and retention

No final-binary O3 retention tournament is admissible: the ordered surface
gate failed first. An implementation-time water-16k diagnostic was run before
the surface boundary was discovered, but it is excluded from evidence and
cannot select or reject a retained layout. Water-48k and viscous tournament
runs were not started.

| Retention condition | Result |
|---|---|
| all correctness gates pass | FAIL at surface-16k i2 correspondence |
| memory/capacity bounds pass at reached boundary | PASS |
| p95 improvement and `1.10x` denominator geometric mean | NOT RUN / INELIGIBLE |
| water-16k no-regression rule | NOT RUN / INELIGIBLE |

O3 exits `O3_SEGMENTED_NUMERIC_MISMATCH`. The selectable failed identity and
its exact boundary remain in the standalone tool for diagnosis. The retained
O4 input remains:

```text
nuv-gather-directed-r0 + pointer-swap-o1 + nuv-terms-specialized-o2
```

No O3 speedup, aggregate NR2 result, NR4 decision, W2 credit, runtime
integration, GPU authority or production claim is created.

## External raw report hashes

| Raw report | SHA-256 |
|---|---|
| final default atomic tiny | `a8009bdde91e08c078361efef61ad8e416553026b02c1eee185e9fccdf011e49` |
| final retained gather tiny | `5112fb19b08af10f0fa27ef76fd6156e4bec6b453049858d161e8e785bcc3ca1` |
| final segmented tiny | `f0aa4282dfb9d99c7fa42bed126f6d9208b6c3c5ba330326c74c209c214df3ba` |
| final segmented surface i2 cold repeatability | `ce7ed5bc80dea34d07c6954191a590031a8825e1708cd5c279f3b4f9a17f6fe1` |
| final segmented surface i2 reused check | `2a84bfbf6bec89ea82a79d4f04fdfec493a59a14059a4656eb14bdfe761a18e1` |

## Commands

```text
cmake --build /tmp/nextengine-nonlocal-feasibility-build --clean-first --parallel
nonlocal-feasibility --self-test
nonlocal-feasibility --self-test --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
nonlocal-feasibility --self-test --accumulation nuv-unique-pair-segmented-o3 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
nonlocal-feasibility --repeatability nuv-surface-16k.v0 --iterations 2 --runs 10 --accumulation nuv-unique-pair-segmented-o3 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
nonlocal-feasibility --check nuv-surface-16k.v0 --iterations 2 --accumulation nuv-unique-pair-segmented-o3 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
```
