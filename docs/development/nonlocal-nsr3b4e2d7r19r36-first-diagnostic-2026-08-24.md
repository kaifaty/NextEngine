# NSR3-B4E2D7R19R36 first diagnostic -- 2026-08-24

Status: `PASS / CLEAN PROOF RUNS PENDING`.

The first strict-f64 diagnostic build executes the frozen equal-work hybrid
without changing the contract. The R35 parent stdout SHA-256 remains exact at
`8f4161e6d4cf9ac24d1ee828dec7019ae0179bd5979c97ea8fec3f2910446fe9`,
and the independently recomputed first three curvature records reproduce the
three frozen R35 roots.

The six unchanged projected exact-line polish steps all pass strict descent,
line KKT and trust-feasibility checks. Fresh terminal evaluation gives:

| Metric | R34 first-order reference | R36 hybrid | Ratio |
|---|---:|---:|---:|
| hinge objective | `1.6477454905319591e-16` | `3.5527997925597766e-17` | `0.21561581038906613` |
| violation norm | `1.8153487216135412e-8` | `8.4294718607511539e-9` | `0.46434449537931005` |
| projected mapping norm | `1.4988794937300478e-9` | `7.4709090069224081e-10` | `0.49843293194509064` |

Strict three-metric dominance therefore selects
`EQUAL_WORK_CURVATURE_POLISH_HYBRID_CANDIDATE` in this diagnostic. The work
ledger closes exactly at `51` new pair passes and `15` generalized HVPs;
workspace lifecycle, route corpus and rollback all pass. No nonlinear moved
state is evaluated and no correction or state is applied.

Diagnostic stdout-with-LF SHA-256 is
`13566a2dc1ed03336e906ba68a6a2c80af84cb98a38981e61e22595b50c07550`;
semantic result SHA-256 is
`6efda6f07993f4e4cf067c894bd2ffee7b0bf3f10585ad043aba6054008bae99`.
These are observations only until two clean Release builds and byte-exact
fresh runs close the frozen contract. No wall-time or production credit is
claimed.
