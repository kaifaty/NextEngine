# NSR3-B4C0R one-pass neighborhood evidence -- 2026-08-21

Status: `PASS / JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE / B4C1_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --joint-neighborhood-one-pass-self-test
```

Three reports are byte-identical:

```text
raw JSON plus LF  79a313b5e258b9da3d776486deade086e609f8b031194e18ac2d4951ad069dfa
JSON without LF   5ecaa4d5356611863dbd80fd8e261f16aa3410907e4a15739875521fb578d493
semantic result   bc60400325bfa0e7a3109fe8df037ff8ff2bd8e2362f378347e61c7514ad8047
```

The exact B4C0 failed report remains
`3151787a6d3c2b7a064a5292b3eb70b8bdbab2a6fe2b366ba61c5e39cc355f97`
including its final LF. B4C0R requires and observes the exact parent failure;
the repair does not relabel it.

## Result

The one-pass builder retains every B4C0 mathematical root and exact gate:

| Case | Pair root | all-pairs checks | one-pass cell checks | Ratio |
|---|---|---:|---:|---:|
| P1 initial | `61799f26...e7122` | `27,240` | `18,120` | `0.665x` |
| P1 forecast | `80736a12...c1b7` | `27,240` | `18,120` | `0.665x` |
| P2 detached | `fca4d222...5cc` | `33,183` | `7,863` | `0.237x` |
| signed cutoff | `0720a9c0...963` | `12` | `7` | `0.583x` |

Pair membership/order, density, pressure energy, active set, complete
fluid/support gradient, joint and fluid-only HVP, repeat digest and both
storage permutations are exact. The six inherited failure controls still
return their required typed error with zero pair and adjacency sizes.

## Workspace boundary

Both indices in a pair are `u32`. At the frozen maximum:

```text
pair payload reservation       64,000,000 bytes
degree-bounded adjacency       32,000,000 bytes
nested-row header diagnostic    1,200,000 bytes on this ABI
```

The first two are hard payload bounds. The row-header number is diagnostic;
compact CSR remains mandatory before nominal memory or performance credit.

## Decision

Select `JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE`. Authorize B4C1 compact CSR and
pressure-coefficient-tape contract design only. The one-pass result has not
replaced any trajectory query, published canonical state, executed a nominal
corpus or established runtime/production performance.
