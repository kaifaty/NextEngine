# NSR3-B4C2T full-controller substitution evidence -- 2026-08-21

Status: `PASS / JOINT_PRESSURE_B4B2_CONTROLLER_CANDIDATE / B4C3_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --joint-pressure-controller-self-test
```

Two reports are byte-identical:

```text
raw JSON plus LF  adfcce42be3edb43be54b6e5c68aa7d57171979167f52c10e88143623361e625
JSON without LF   f33093415620deb819d93c52fec9f969e802cf1341bc1d0a17cbfbaf46512d0a
semantic result   00b67a5506dd2a69fa477c14db0c730140e1747e26fc66736d734ee8cbb44606
```

The two whole harness runs took `60.58 s` and `62.91 s` of wall time and
approximately the same user CPU time. Each includes the complete parent chain,
an independent legacy controller and the joint candidate. These timings are a
test-cost observation, not a solver speed comparison.

## Exact controller result

Both cases pass byte-identical `append_b4b_case`, forecast-work and KKT-work
serialization plus exact adaptive/fixed final states.

| Case | Accepted | Executed | Discarded | Spectral HVP | Nonlinear HVP | Outer |
|---|---:|---:|---:|---:|---:|---:|
| P1 supported | 296 | 444 | 148 | 384 | 3,007 | 1,881 |
| P2 released | 82 | 124 | 42 | 96 | 218 | 226 |

P1 retains frame-zero `FORECAST_ACTIVE 21/42`. P2 retains
`INACTIVE_EXACT` through frame 13, `FORECAST_ACTIVE` on frame 14 and
`START_ACTIVE` on frame 15. Positions, velocities, discarded-level isolation,
contacts, face multipliers/impulses, reactions, ledger, energy/density/speed,
precontact controls, fixed `48/96/192` trajectories and all B4B2 gates match
the legacy all-pairs result bit-for-bit.

Exact serialized-case roots are:

```text
P1  de77effe19213a8cd649655e4d11c076a07673b91e88ec0edb8d46cc881cbe45
P2  21358c5779d42aafe1a1b55c2d27ff255037a0c203513dc191e168cf2f0ae35c
```

## Joint query trace

Candidate and audit all-pairs counters are zero. At most two workspaces are
live; all exits return to zero.

| Counter | P1 | P2 |
|---|---:|---:|
| Joint evaluations/neighborhoods/tapes | 14,149 | 11,860 |
| Taped HVP | 16,153 | 1,474 |
| Trial builds/promotions | 7,802 | 682 |
| Rejected workspace destructions | 0 | 0 |
| Total unique-pair records processed | 52,848,394 | 9,640,564 |
| Total directed records processed | 64,607,616 | 13,599,192 |
| Cell distance tests | 256,379,880 | 93,255,180 |
| Equivalent all-pairs candidate checks | 385,418,760 | 393,550,380 |
| Maximum tape bytes | 49,620 | 13,520 |

Query-chain roots are:

```text
P1  6c3256ab14d05e16333b16eaef52137c9f24b52f3a43967841bc0f781b529c3d
P2  aeaa45a6eb5319864fabecfa6efebae5571630cee22493d4aeb7e4468fd58fa6
```

## New performance finding

The result closes query correctness but exposes excessive workspace rebuilds.
The solver releases its accepted final workspace, then read-only mechanical,
density, frame-start and aggregate diagnostics rebuild the same state. This is
especially visible for detached P2: `11,860` evaluations versus only `1,474`
HVPs.

Do not interpret this as a formula failure. B4C4 should retain a committed
workspace across post-step diagnostics and the next frame-start query, split a
topology/radius build from cheap materialized views, reuse the immutable static
support cell index and eliminate validation-only nested rows. That optimization
must preserve the B4C2T query-chain semantics under a separately frozen gate.

## Decision

Select `JOINT_PRESSURE_B4B2_CONTROLLER_CANDIDATE`. Authorize only B4C3
canonical publish/decode transaction design. B4C4 packaging remains recorded
before B4D nominal execution. CUDA, runtime and production remain blocked.
