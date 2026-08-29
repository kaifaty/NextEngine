# NSR3-B4C3MAR complete adaptive macro replay evidence

Status: `PASS / COMPLETE_CANONICAL_TOPOLOGY_ADAPTIVE_MACRO_CONTROLLER_CANDIDATE`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --macro-adaptive-replay-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / COMPLETE_CONTROLLER_RESEARCH_ONLY
raw JSON + LF          e73bf94388ca82fded7aa92f1434bfb0696791f798011a2e10cd5a4da7bf22e3
raw JSON without LF    b1549cc6929f86256d81330a7a6d70cdb8df0e8db81f1d9d3e3fbf5d684b9dff
semantic result        0da1f9eab67f0fd3288f06f2de1baf482a24067393675ef34b4abadbcb734cf5
wall time              71.13 s / 70.95 s
CPU utilization        294% / 291%
maximum RSS            15,156 KiB / 13,440 KiB
parent B4C3MAG exact    true
```

The isolated replay passes in `4.23 s` at raw-with-LF
`60e44aaff80d5df8a274306b8846cc6435591ae940f5fdb9b63d413a30f2b554`,
raw-without-LF
`9e186b1ec0fd904807180e6ffec6bab714f9363d8f53082eb74f62d36c77d4ff`
and the same semantic result.

## Complete work and schedule

| Case | Frames | Accepted / attempted / discarded | Nonlinear / spectral HVP | Recovery |
|---|---:|---:|---:|---:|
| P1 supported | 8 | `296 / 444 / 148` | `3013 / 384` | `0` |
| P2 released | 16 | `82 / 124 / 42` | `218 / 96` | `0` |

P1 selects `42,42,32,32,42,42,32,32` substeps. Macro-only publication removes
the two exact reject-limit recoveries required by the old per-substep canonical
controller and reduces its P1 attempted work from 563 to 444 substeps and
nonlinear HVP work from 3810 to 3013. This is a controller/cadence result, not
yet a fixed-reference accuracy claim.

P2 retains the exact independent onset schedule: frames 0--13 are
`INACTIVE_EXACT`, frame 14 is `FORECAST_ACTIVE`, and frame 15 is
`START_ACTIVE`. Its accepted/attempted and HVP work remain `82/124` and `218`.

## Representation, topology and physics

All 16 P1 field admissions use the temporal branch; their maximum selected
utilization is `0.735765`. P2 uses 16 temporal position admissions and velocity
uses 3 temporal plus 13 exact/free-flight absolute admissions. Its maximum
selected utilization is `0.361003`. No field is rejected.

Every committed frame passes canonical integer topology and box feasibility;
maximum penetration is zero. P1 maximum density strain is `6.42205e-4` and
speed `0.216212 m/s`. P2 has 22 precontact private steps, zero premature
pressure/support reaction, position error zero, velocity error `6.94e-18 m/s`
and spread `1.67e-16 m/s`.

Publication pressure/mechanical budget utilizations are only
`0.000298/0.000340` for P1 and `0.001021/0.001166` for P2. Maximum KKT-scale
macro ledger residuals are `5.64003e-10` and `1.89807e-10`.

## Durable identity and rollback

| Case | Trajectory root | Legacy ledger root | Macro-policy root |
|---|---|---|---|
| P1 | `852759dc...50d5` | `78bda3a7...74ad` | `bab793e5...eb23` |
| P2 | `de782684...4883` | `9273756e...ba1c` | `d03a5985...63cd` |

Steps are contiguous macro indices and final decoded state equals the last
frame. A forced failure after selecting the second P1 frame but before
publication preserves the first committed state, counts, all roots and
cumulative totals exactly. Recovery and topology negatives also pass.

## Decision

Select `COMPLETE_CANONICAL_TOPOLOGY_ADAPTIVE_MACRO_CONTROLLER_CANDIDATE`.
Authorize only design of an adaptive-versus-fixed macro comparison. This PASS
does not authorize nominal corpus, B4C4/B4D, CUDA, runtime/schema or production.
