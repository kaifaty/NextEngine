# NSR3-B4EP4 cached residual-attribution evidence -- 2026-08-22

Status: `PASS / HVP_RESIDUAL_DOMINANT / B4EP5_HVP_DESIGN_AUTHORIZED`

## Result

The exact-output cached profile records 1,112 ten-millisecond CPU samples.
Exact HVP application owns 6.91 s, or 62.14% inclusive sampled time. Complete
cached workspace construction owns 3.95 s, or 35.52%; residual nonlinear and
publication bookkeeping accounts for about 0.26 s, or 2.34%.

HVP leads complete workspace by `1.749367x`, above the frozen `1.20x`
selection threshold. B4EP4 therefore selects only HVP design for B4EP5. It
does not authorize simultaneous workspace work, a solver-policy change,
parallelism, GPU work or B4E2 execution.

## Build and correspondence

| Field | Value |
|---|---|
| B4EP4 identity | `4240148cadb765b33d6b72b55b84d523cbcce5456774dad1d3918faecdeee42e` |
| source implementation commit | `65ce739e17def07266f0d2f72b00178451a1fb3e` |
| compiler | GCC 15.2.0 |
| compile/link flags | `-O3 -DNDEBUG -g -pg`; link `-pg` |
| compile commands | 14,601 bytes; `74d029a3205b8630f844f987c86a7c7c4a223b66d0565a6056b8cfa2f8f237b5` |
| instrumented executable | 36,918,368 bytes; `201236a8b7aeaf23e73adfad6d9ddfffc10f98176dab26be2ffd25d363b142ae` |
| GNU Build ID | `e17f833fc1dc8069c6bec690f738e35ce294af13` |

The process exits zero with empty stderr. Its stdout is exactly 6,462 bytes
with SHA-256
`b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055`
and semantic result
`99a7e4b183f844390dca81beba0d83585bf938cb08e0dd6bd6794c82079fd3fe`.
It matches the selected B4EP3I candidate byte-for-byte.

Instrumented wall/RSS are 17.91 s, 62,904 KiB and 99% CPU. They demonstrate
normal completion only and are not Release throughput evidence.

## Profile artifacts

| Artifact | Size | SHA-256 |
|---|---:|---|
| `gmon.out` | 1,397,307 bytes | `875bba17fec9bf0a7b2edaa60d9a1674c09dcbe7e8e20eb5f0ae9bc27a20a8d9` |
| flat profile | 50,719 bytes | `9fa81d9c7f6c3d4e84fdbd01e911e049ac660f4737a9b377b9b7747189312b32` |
| call graph | 340,016 bytes | `430b1d7f3f6c30872b109ddb4945b32a51db8631a7538dd3b929ad1b39fc18b4` |

B4EP4 result attestation root:
`171f1544bfbd65a999f393e88d7c32afd6f36df4fdf7fc360fb5527acf614779`.

External artifacts remain outside Git under:

- build: `/home/kaifaty/.cache/nextengine/external/build-nonlocal-b4ep4.NSSOjh`;
- run: `/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep4.OyJSao`.

## Attribution

| Category | Sampled CPU | Share of 11.12 s |
|---|---:|---:|
| exact HVP application inclusive | 6.91 s | 62.140288% |
| complete cached workspace inclusive | 3.95 s | 35.521583% |
| superset/filter/CSR within workspace | 1.14 s | 10.251799% |
| evaluation/tape within workspace | 2.74 s | 24.640288% |
| residual bookkeeping | 0.26 s | 2.338129% |

The HVP total consists of 4.53 s direct `apply_joint_pressure_tape` work plus
2.38 s of its `kernel_scale` child over 459 calls. Evaluation contributes
1.84 s direct plus 0.53 s of the same kernel child over 227 calls; pressure
tape refresh adds 0.37 s.

Filtered topology contributes 0.54 s direct plus 0.50 s of adjacency and
canonicalization children over 226 calls. The one superset build contributes
0.07 s and the single parent canonical topology contributes about 0.03 s.
Evaluation/tape therefore leads filter/CSR by `2.403509x`, but the frozen
top-level rule selects HVP first.

As in B4EP2, one optimized topology symbol is folded under the displayed
`run_joint_pressure_tape_controls()` name. Its single call from
`build_joint_query_workspace`, cell/adjacency children and exact parent count
establish the mapping; the label is not interpreted literally.

## Decision

Select `HVP_RESIDUAL_DOMINANT` and authorize only B4EP5 HVP research/design.
The next discriminator should first separate mechanical output allocation,
invariant tape inputs and arithmetic/reduction traversal while retaining the
exact HVP result and call schedule. B4E2, further topology/evaluation changes,
runtime, CUDA and production remain blocked.
