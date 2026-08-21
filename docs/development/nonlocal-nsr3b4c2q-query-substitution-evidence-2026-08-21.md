# NSR3-B4C2Q solver-query substitution evidence -- 2026-08-21

Status: `PASS / JOINT_PRESSURE_KKT_QUERY_CANDIDATE / B4C2T_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --joint-pressure-query-self-test
```

Three reports are byte-identical:

```text
raw JSON plus LF  c2f0d8a4d19ab7682692022099f75ac0fe393c3fada6ea92deb046c25830479f
JSON without LF   97ed02babe0a08972f7fb0f0d149903c2365480227d5d5266ac66fe2a76bb2c6
semantic result   683f3ba1ee6f51814e75779e063e7e594a37b16812a97c855df7b2a3a096105e
```

The exact parent reports remain unchanged, including their final LF:

```text
B4C1  PASS  54173d3b15ace2d86827abf8e372d6e098c9964b5fa63a9de5c2b8909e75f030
B4C0R PASS  79a313b5e258b9da3d776486deade086e609f8b031194e18ac2d4951ad069dfa
B4C0  FAIL  3151787a6d3c2b7a064a5292b3eb70b8bdbab2a6fe2b366ba61c5e39cc355f97
```

## One-substep result

Every candidate result is fed only by joint evaluation and taped HVP output.
All-pairs calls are made only by the separately counted audit oracle. Complete
position, velocity, displacement, KKT/contact state, fluid/support reaction,
ledger, objective and iteration counters match the legacy solve bit-for-bit.

| Case | Outer | Accepted | Rejected | Taped HVP | Final active centres | Workspaces | Promotions |
|---|---:|---:|---:|---:|---:|---:|---:|
| P1 initial `frame/21` | 4 | 3 | 0 | 7 | 16 | 4 | 3 |
| P1 forecast `frame/42` | 4 | 3 | 0 | 6 | 16 | 4 | 3 |
| P2 detached `frame` | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| P1 compressed `frame/48` | 4 | 3 | 0 | 6 | 12 | 4 | 3 |

Thus the selected corpus independently covers active multi-outer solves and
the exact inactive zero-HVP path. Every active trial rebuilds membership and
tape; an accepted trial promotes the complete workspace. Maximum live
workspace count is two and returns to zero in every case.

## Forecast and rejection ownership

P1's read-only macro forecast has 16 active centres and executes 48 taped
Lanczos HVPs. Its maximum eigenvalue is exactly
`70346.222687210058`. P2's forecast remains inactive with zero HVPs. Fixture
positions and velocities remain exact in both cases.

The forced-reject control constructs two distinct valid states:

```text
current  b51a0b8ee0f0c715b29c5b79508be7005cc3e4e8e943ec6f81d06ad6a352ab55
trial    7370615af04c204e436cd5f6dc7ef754c7b90f74f1bbbeb76ee8d5e5516e6faa
```

After destroying the trial, current positions, Evaluation, pair list, CSR,
radii, compression and digest remain bit-for-bit unchanged.

## Spatial work and oracle accounting

Candidate all-pairs evaluation/HVP counters are zero in every trace. Audit
oracle calls are reported independently and cannot feed the candidate result.
The independent legacy spectrum replay makes the active forecast's audit HVP
count 96: 48 per-call image comparisons plus 48 for the final spectrum replay.

| Path | Workspaces | Cell distance tests | All-pairs candidate checks |
|---|---:|---:|---:|
| P1 initial solve | 4 | 72,480 | 108,960 |
| P1 forecast solve | 4 | 72,480 | 108,960 |
| P2 detached solve | 1 | 7,863 | 33,183 |
| P1 compressed solve | 4 | 72,480 | 108,960 |
| P1 active forecast | 1 | 18,120 | 27,240 |
| P2 inactive forecast | 1 | 7,863 | 33,183 |

Every workspace satisfies the inherited strict cell-work reduction. These are
executed structural counters, not elapsed-time, cache, SIMD or production
performance evidence.

## Decision

Select `JOINT_PRESSURE_KKT_QUERY_CANDIDATE`. Authorize only B4C2T contract
design for substituting the complete B4B2 adaptive and fixed-reference
controllers. Canonical continuation, nominal corpus, CUDA, runtime and
production remain blocked.
