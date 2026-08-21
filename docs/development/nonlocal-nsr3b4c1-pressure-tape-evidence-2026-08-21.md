# NSR3-B4C1 compact pressure-tape evidence -- 2026-08-21

Status: `PASS / JOINT_PRESSURE_RADIUS_TAPE_CANDIDATE / B4C2_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --joint-pressure-tape-self-test
```

Three reports are byte-identical:

```text
raw JSON plus LF  54173d3b15ace2d86827abf8e372d6e098c9964b5fa63a9de5c2b8909e75f030
JSON without LF   8a2c27833953297e1bdffabe480e05d849a39271e6e147d0a01d79e831218182
semantic result   b7b05aa735f2b8153013cb02f7eab220e575df187061a5fd6663ec63d136c1d7
```

The exact parent reports remain unchanged, including their final LF:

```text
B4C0R PASS  79a313b5e258b9da3d776486deade086e609f8b031194e18ac2d4951ad069dfa
B4C0  FAIL  3151787a6d3c2b7a064a5292b3eb70b8bdbab2a6fe2b366ba61c5e39cc355f97
```

## Exact correspondence

All five controls pass exact CSR-row, binary64 radius, compression and HVP
correspondence. Each HVP comparison covers all fluid and static-support output
rows for three deterministic joint directions and one fluid-only direction.
Identity, reverse and coprime-affine input orders produce the same tape digest
and HVP. Inactive controls return exact zero pressure response.

| Case | Pairs | Directed | Active centres | Active directed | Tape bytes |
|---|---:|---:|---:|---:|---:|
| P1 initial | 3,274 | 4,036 | 0 | 0 | 42,916 |
| P1 forecast | 3,626 | 4,452 | 16 | 1,552 | 47,396 |
| P2 detached | 746 | 1,069 | 0 | 0 | 10,572 |
| signed cutoff | 4 | 5 | 0 | 0 | 92 |
| P1 compressed 0.99 | 3,682 | 4,548 | 12 | 1,272 | 48,228 |

The three failure controls return `PRESSURE_TAPE_PAIR_INDEX`,
`PRESSURE_TAPE_OFFSET_CAPACITY` and `PRESSURE_TAPE_CAPACITY` respectively.
Every returned offsets/index/radius/compression array remains empty.

## Work discriminator

The frozen count is radial norm/sqrt evaluations for three HVPs over one
immutable outer state:

| Active case | Untaped | Tape build + three taped HVPs | Ratio |
|---|---:|---:|---:|
| P1 forecast | 24,846 | 3,626 | 0.146x |
| P1 compressed 0.99 | 22,494 | 3,682 | 0.164x |

Inactive cases report counts but make no ratio claim. This is an exact
algorithmic-work result, not elapsed-time, cache, SIMD or production evidence.

## Payload ceiling

At the frozen `50,000`-fluid admission bound:

```text
existing pair list               64,000,000 bytes
binary64 radii                   64,000,000 bytes
directed pair indices            32,000,000 bytes
CSR offsets                         200,004 bytes
binary64 compression                400,000 bytes
combined                        160,600,004 bytes
```

The ceiling is exact capacity evidence, not a product memory budget.

## Decision

Select `JOINT_PRESSURE_RADIUS_TAPE_CANDIDATE`. Authorize only a B4C2 contract
that substitutes every current, trial and feasible-forecast pressure query
inside one substep and compares it to the frozen all-pairs operator. Full
trajectory, canonical continuation, nominal corpus, CUDA, runtime and
production claims remain blocked.
