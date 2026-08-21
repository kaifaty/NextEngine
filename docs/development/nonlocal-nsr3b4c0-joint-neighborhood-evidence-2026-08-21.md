# NSR3-B4C0 joint neighborhood evidence -- 2026-08-21

Status: `FAIL / P1_WORK_GATE / EXACT_MATH_PRESERVED`

## Reproduction

```text
nonlocal-formula-reclosure --joint-neighborhood-self-test
```

The command exits `1` as required for the frozen failure. Three reports are
byte-identical:

```text
raw JSON plus LF  3151787a6d3c2b7a064a5292b3eb70b8bdbab2a6fe2b366ba61c5e39cc355f97
JSON without LF   d811c8d55fd3ed1dad803d489b70eb9f2b1ca690296d4b4420c42ad44908f761
semantic result   44aec304e5fd7d3a54a3d74d9512630d4a76595b5d437fda42751fc4187e8a89
```

The first failure is `p1-initial:CORRESPONDENCE_GATE`; more precisely, only
its frozen work predicate fails.

## Exactness result

All four cases pass exact pair membership/order, density, pressure energy,
active set, full fluid/support gradient, joint-direction HVP, fluid-only HVP,
repeat digest and identity/reverse/coprime-affine storage permutations.

| Case | fluid / support | fluid / support pairs | max degree | Pair root |
|---|---:|---:|---:|---|
| P1 initial | `48 / 544` | `762 / 2512` | `94` | `61799f26...e7122` |
| P1 feasible forecast | `48 / 544` | `826 / 2800` | `99` | `80736a12...c1b7` |
| P2 detached | `27 / 1216` | `323 / 423` | `60` | `fca4d222...5cc` |
| signed cutoff | `3 / 3` | `1 / 3` | `3` | `0720a9c0...963` |

The signed cutoff includes binary64 distances immediately below, exactly at
and immediately above `H`; its membership remains exact.

All six failure controls return their exact typed error with zero partial pair
or adjacency output, including the `161+` neighbor overflow.

## Work failure

The two-pass design deliberately performs the same cell candidate scan once
to count exact membership and once to fill exact storage:

| Case | all-pairs checks | two-pass cell checks | Ratio | Gate |
|---|---:|---:|---:|---|
| P1 initial | `27,240` | `36,240` | `1.330x` | FAIL |
| P1 forecast | `27,240` | `36,240` | `1.330x` | FAIL |
| P2 detached | `33,183` | `15,726` | `0.474x` | PASS |

One P1 cell pass uses `18,120` distance tests, so the cell broad phase itself
does reduce candidates by `33.5%`. The exact-count replay erases that saving
for the dense small box. This is an allocation/construction-policy failure,
not a membership or pressure-formula failure.

## Decision

Preserve B4C0 FAIL and do not authorize the pressure tape. Research a separate
one-pass builder that writes into a capacity admitted before the query, clears
all private output on overflow and retains the exact pair/order/math gates.
Do not weaken the work gate or reinterpret one-pass counts as the executed
two-pass cost.
