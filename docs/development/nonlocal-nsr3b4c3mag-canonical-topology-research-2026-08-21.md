# B4C3MAG canonical topology research

Status: `COMPLETE / PASS / FULL ADAPTIVE MACRO DESIGN AUTHORIZED`

Date: `2026-08-21`

## Finding

B4C3MA exposed two coordinate identities at the macro publication boundary:

```text
binary solver geometry     0.2 - 0.025 -> 0.17500000000000002
canonical durable geometry 175000 um   -> 0.17499999999999999...
```

They denote the same canonical coordinate but compare unequal as binary64.
Using an epsilon would make topology hardware/policy dependent. Snapping the
decoded state back to the solver value would make committed state disagree with
the hashed canonical frame.

## Selected topology identity

Topology at a durable boundary is a relation over canonical integers:

```text
particle feature (sample, face) is active geometrically
iff quantize_position(sample[axis]) == quantize_position(face[axis])
```

Both sides use the already selected exact nearest-even canonical conversion.
This is equality, not tolerance. Binary64 equality remains diagnostic for
private solver state; exact KKT terminal features must equal both private and
decoded canonical feature sets after conversion.

Canonical geometry is valid only when each quantized low face is strictly less
than its high face. Every decoded position integer must lie inclusively between
those integers. Raw penetration stays zero in the current discriminator.

## Scope

B4C3MAG changes only topology admission and report identity. Candidate levels,
solver coordinates, balanced publication, mixed budgets, ledger policy,
failure grammar, work accounting and rollback remain B4C3MA-exact. The old
raw-equality B4C3MA report must reproduce as a FAIL control.

## Decision

Freeze the [B4C3MAG contract](../plans/nonlocal-nonlinear-solver-research/03b4c3mag-canonical-topology-contract.md).
PASS can select the one-frame adaptive macro transaction and authorize only a
complete adaptive macro replay design.

B4C3MAG passes twice byte-identically; see the
[dated evidence](nonlocal-nsr3b4c3mag-canonical-topology-evidence-2026-08-21.md).
It preserves the raw B4C3MA mismatch as a discriminator while recovering exact
64-feature P1 topology and KKT identity in canonical integer coordinates.
