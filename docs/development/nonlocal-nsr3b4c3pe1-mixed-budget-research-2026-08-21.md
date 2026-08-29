# B4C3PE1 mixed stability budget design

Status: `COMPLETE / PASS / ADAPTIVE MACRO DESIGN AUTHORIZED`

Date: `2026-08-21`

## Selected rule

B4C3PE rejects a scalar macro-map gain and shows why neither a purely relative
nor a broad physical gate is sufficient. B4C3PE1 assigns the durable
representation a small explicit share of two already independent error scales:

```text
temporal branch  e <= 0.5 * D_binary
absolute branch  e <= 0.01 * A_physical
```

The temporal branch is evaluated first when `D_binary` exceeds its computed
binary64 floor. Otherwise, or when it fails, the absolute branch may admit the
frame. Every admitted field reports its branch and utilization.

## Adjacent estimator mapping

For a first-order ladder, `|B_h-B_h/2|` estimates the fine solution's temporal
error. Freeze:

```text
candidate 48   estimator |B48-B96|
candidate 96   estimator |B48-B96|
candidate 192  estimator |B96-B192|
```

The 48 lane deliberately uses the available adjacent difference as a strict
diagnostic scale; the absolute branch prevents division by a physically
irrelevant near-zero estimator.

Physical scales remain the B4B comparison values `0.05dx` for position and
`0.001c` for velocity. One percent is reserved for durable representation, so
the rule does not consume the main adaptive/reference accuracy budget.

## Decision

Replay the complete B4C3P candidate with observed first-order convergence
required for both fields, exact events/contacts, all non-tube physical gates,
macro transaction/ledger roots and rollback. PASS may select macro-boundary
canonical fixed reference and authorize only adaptive macro-transaction
design. The old B4C3P FAIL and B4C3TR FAIL remain evidence.

Execution passed twice byte-identically. The selected policy admits 89 fields
through the temporal branch and 55 through the absolute branch, with zero
rejections. The
[dated evidence](nonlocal-nsr3b4c3pe1-mixed-stability-budget-evidence-2026-08-21.md)
selects the fixed macro-boundary reference candidate and authorizes only the
adaptive macro-transaction design.
