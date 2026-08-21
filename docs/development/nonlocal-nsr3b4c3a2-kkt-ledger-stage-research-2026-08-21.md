# B4C3A2 KKT-scale stage-ledger revalidation design

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Purpose

B4C3L selected a corrected ledger admission rule, but it was deliberately
read-only. B4C3A2 must prove that using this rule in the canonical stage
transaction preserves everything B4C3A1 established:

```text
fine-only atomic commit
canonical frame identity
publication impulse and closure
energy decomposition
rollback
order/repeat determinism
```

The only intended difference is the ledger policy record and which normalized
compensated residual is an admission predicate.

## Representation versus evidence identity

The canonical representation profile stays
`f57d88c...79c`; changing it would falsely imply different quantized samples.
The B4C3L policy hash `b1136c2c...e2f` identifies ledger evidence semantics.
Therefore candidate canonical trajectory roots must equal B4C3A1, while a new
policy-ledger hash binds both residuals, both scales, correspondence and the
policy identity.

The legacy ledger hash also remains computed and exact. This gives two useful
statements simultaneously:

```text
physical/canonical data did not change
new admission evidence is complete and independently identifiable
```

## Decision

Freeze B4C3A2 as a one-frame P1/P2 revalidation. PASS may authorize only a
new complete adaptive recovery contract. It cannot retroactively turn B4C3TAR
into PASS or authorize fixed-reference/nominal/runtime work.
