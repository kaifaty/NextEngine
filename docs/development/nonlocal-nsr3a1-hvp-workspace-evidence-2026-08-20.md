# Nonlocal NSR3-A1 HVP workspace evidence -- 2026-08-20

Status: `PASS / HVP_WORKSPACE_STREAM_V1_SELECTED / REPORT_ONLY`

## Outcome

`hvp-workspace-stream-v1` is bit-exact and passes every frozen performance
gate. It reuses result/density/neighbor-Jacobian storage and streams the center
through sorted adjacency without copied participant arrays, sorting or binary
searches.

| Particles | Baseline HVP | Candidate HVP | HVP speedup | Baseline total | Candidate total | Total speedup |
|---:|---:|---:|---:|---:|---:|---:|
| 512 | `78.677 ms` | `60.896 ms` | `1.292x` | `118.440 ms` | `100.039 ms` | `1.184x` |
| 1000 | `173.142 ms` | `130.263 ms` | `1.329x` | `280.397 ms` | `237.637 ms` | `1.180x` |
| 1728 | `358.510 ms` | `267.417 ms` | `1.341x` | `591.205 ms` | `500.762 ms` | `1.181x` |
| 4096 | `1280.378 ms` | `944.962 ms` | `1.355x` | `2156.803 ms` | `1817.915 ms` | `1.186x` |

This table is the clean single-process campaign pinned to logical CPU 4. One
earlier diagnostic campaign overlapped another process and is intentionally
excluded. The retained campaign used one warmup per implementation and seven
alternating A/B pairs.

## Correctness

- candidate HVP matches the baseline bit-for-bit on all seven NSR2-B controls;
- candidate and baseline final state, objective, stop, operations and capacity
  match bit-for-bit on 512/1000/1728/4096;
- every timing-independent NSR0--NSR2-C2 hash remains unchanged;
- timing-independent tournament result SHA-256 is
  `63e49c2f785b6eed8b6015dd6c19b8846208c21cb575a9bde958169ab244281b`.

The speedup grows with size, consistent with removing per-center allocations,
sorts and searches rather than exploiting a small-case artifact.

## Decision

Select `hvp-workspace-stream-v1` for subsequent research. Retain the original
HVP as the exact A/B oracle. The remaining largest opportunity is to stop
recomputing state-dependent Hessian coefficients for every Krylov product:
positions, active set and adjacency are immutable throughout an outer trust
state, so their radial/Jacobian coefficients can be assembled once and reused.

Authorize a separately frozen NSR3-A2 Hessian coefficient-tape experiment.
This still grants no multi-step physics, GPU or runtime authority.

