# NSR3-B4E2D7R19R64 sparse row-operator research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / TWO-LAYER SPARSE OPERATOR SELECTED /
CONTRACT NEXT`.

## Problem exposed by R63

The R63 projection model is useful, but the fixed 494-row restoration master
leaves 2,796 positive tangential rows outside its ownership. Expanding the
existing representation naively is not acceptable:

```text
full row gradients:  6000 rows * 6000 Vec3  ~= 864 MB
dense Gram:          6000 * 6000 doubles     ~= 288 MB
```

It would also make every Hildreth coordinate update scan all particles and all
active residuals. The physical Jacobian is local: R59 measured only 44--113
directed slots per row. R64 must make that locality explicit before an all-row
active set is attempted.

## Selected two-layer representation

Each density row owns two complementary views derived from the same validated
flat topology.

### 1. Directed slot view

Retain exact row slot order:

```text
SparseSlot {
    pair_index,
    participant,
    coefficient = SPACING * pair_jacobian
}
```

The row action is evaluated as

```text
sum_slots dot(coefficient,
              direction[row] - direction[participant])
```

with boundary participants contributing zero direction. This is the same
operation order as `al_r29_directed_jvp`, so it can own exact or tightly bounded
row-action equivalence and the same row-local certificate arithmetic.

### 2. Aggregated DOF-entry view

For coordinate updates aggregate the mathematical row gradient by fluid DOF:

```text
SparseEntry { particle, Vec3 coefficient }
```

The row center receives the sum of all directed coefficients; each fluid
neighbor receives its negation; boundary neighbors create no DOF entry.
Entries are sorted by particle and unique. The direct diagonal is
`sum_entries ||coefficient||^2`.

Build the transpose incidence once:

```text
particle -> [(row, entry_index), ...]
```

When Hildreth changes one multiplier, only rows sharing a particle with that
gradient can change. Iterate its entries, traverse the particle incidence and
accumulate overlap dots for touched rows.

## Competitive-programming structures retained

Use a dense scratch array of 6000 doubles plus an integer epoch/tag array and a
`touched_rows` list. An overlap update becomes:

```text
for entry in active_row:
    for incident in particle_to_rows[entry.particle]:
        if tag[incident.row] != epoch:
            tag[incident.row] = epoch
            touched.push_back(incident.row)
            scratch[incident.row] = 0
        scratch[incident.row] += dot(entry.coefficient,
                                     incident.coefficient)
```

Only `touched_rows` is consumed/reset. This timestamp-array technique avoids
`O(N)` clearing per coordinate and is common in graph and offline-query
solutions. Stable row/particle ordering keeps the reference deterministic.

The eventual Hildreth complexity follows local overlap rather than a dense
Gram column. Storage follows total nonzeros, approximately `O(N*K)`, instead
of `O(N^2)`.

## R64 is equivalence, not another solve

Before dynamic expansion is authorized, validate the new owner against all
available references at exact R43 topology:

1. offsets start at zero, are monotone and terminate at exact slot/entry and
   incidence counts; all pair/particle/row indices are valid;
2. directed sparse action on the deterministic R29 probe reproduces the fresh
   directed JVP rowwise;
3. sparse transpose action on the deterministic R29 row probe is compared to
   pair-once VJP under a predeclared floating forward bound;
4. reconstruct every one of the 494 captured R51 full row gradients from
   sparse entries and compare components, roots or bounded differences;
5. compare all 494 direct diagonals with captured diagonals;
6. use the incidence/timestamp overlap algorithm for all 494 captured rows and
   compare every 6000-entry Gram column with R51;
7. mutate row offsets, entry particle, incidence back-reference and epoch
   reuse in dense controls; every corruption must fail closed;
8. exact work and rollback; no Hildreth cycles, projection or nonlinear trial.

Expected classification routes are:

```text
SPARSE_ROW_OPERATOR_EXACT_EQUIVALENCE_CANDIDATE
SPARSE_ROW_OPERATOR_BOUNDED_EQUIVALENCE_CANDIDATE
SPARSE_ROW_OPERATOR_ARITHMETIC_ALIGNMENT_REQUIRED
SPARSE_ROW_OPERATOR_TOPOLOGY_REJECTED
```

Bit-exactness is preferred but not assumed: directed slot action and aggregated
gradient/transpose/Gram use different legal addition groupings. A bounded
candidate must use gamma-style operation-count bounds selected before nominal
execution; observed differences cannot choose a tolerance.

## Production consequence if validated

R65 may then replace the fixed master with a dynamic all-row active set while
retaining R63 as the full-vector result reference. The same structure maps to
GPU CSR/COO kernels and later admits graph coloring or block-Jacobi scheduling.
R64 itself provides no solver, timing, runtime or production authority.

