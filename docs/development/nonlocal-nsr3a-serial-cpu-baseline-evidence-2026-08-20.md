# Nonlocal NSR3-A serial CPU baseline evidence -- 2026-08-20

Status: `PASS / HVP_DOMINANT / NSR3A1_AUTHORIZED / REPORT_ONLY`

## Outcome

The selected solver is exactly repeatable through 4096 particles and remains
inside neighborhood/conservation capacity. HVP allocation and traversal is the
largest measured bucket on every size and is selected as the first CPU
optimization target.

Host: Linux x86-64, AMD Ryzen 9 3950X, GCC 15.2.0, one process pinned to
logical CPU 4. The target uses `-O3 -ffp-contract=off -fno-fast-math`.
Hardware performance counters were unavailable because the host has
`perf_event_paranoid=4`; no inferred counter values replace them.

| Particles | Operations outer/eval/HVP | Campaign A total/HVP/control median | Campaign B total/HVP/control median |
|---:|---:|---:|---:|
| 512 | `12 / 12 / 46` | `116.291 / 77.460 / 10.191 ms` | `117.941 / 78.383 / 10.337 ms` |
| 1000 | `13 / 14 / 45` | `289.783 / 175.609 / 39.535 ms` | `279.539 / 171.661 / 37.595 ms` |
| 1728 | `13 / 14 / 49` | `624.521 / 379.568 / 112.217 ms` | `583.808 / 352.833 / 105.671 ms` |
| 4096 | `14 / 14 / 65` | `2214.861 / 1311.842 / 554.320 ms` | `2237.036 / 1321.038 / 553.416 ms` |

Each campaign used one warmup and seven measured solves. All measured results
matched their untimed reference exactly. The common timing-independent result
SHA-256 is
`32ec0f90615713ab8b9f9ef5feeecc475d547494ede2de522745df6e7e836066`.

The 512/1000 operations, stops and capacity maxima match NSR2-C2. State roots:

| Particles | State SHA-256 |
|---:|---|
| 512 | `a403336eb5f09d67a25abf580c70ab89f84979cd92a8d66b4c46770ae7d8a0b0` |
| 1000 | `d029671ec131a76098bb9eced9bd3814635996c9c60fd8bd816db78cd3b2e7f0` |
| 1728 | `41d08b3ddd6288a52370510df99ea11d6c1bb15195e982960306832aadccac4c` |
| 4096 | `8d5e2afeb771ccdd696b342889cefda5491c2a7f40b52cea1a1c3e5360708e1d` |

## Bottleneck diagnosis

The reference HVP performs avoidable work for every Krylov call and active
density center:

- allocates output and density arrays per HVP;
- allocates `neighbor_jacobian` and `participants` per active center;
- copies and sorts already sorted adjacency after inserting the center;
- repeats binary searches to recover adjacency slots during both reduction and
  scatter.

At 4096 particles HVP is about 59% of total median time and control/vector work
another 25%. The first optimization therefore targets only reusable HVP
storage and allocation-free ordered traversal. Krylov vector storage remains a
separate later candidate so attribution is not mixed.

## Decision

Authorize NSR3-A1 `hvp-workspace-stream-v1`. No performance target or result is
claimed for GPU/runtime. Physical-corpus execution remains blocked by NSR3-B0
profile derivation and NSR3-B2 boundary reclosure.

