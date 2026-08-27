# NSR3-B4E2D7R20R63ZC operator/input factorial research

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN`.

## Question

R63ZB rejects after replacing the earlier binary128 tangent-product lineage
with portable common-block/input arithmetic. Which already-admitted
representation change is sufficient to lose the R63X state-2 certificate:
the operator action, projection of the RHS/inverse scale, or only their
interaction?

The exported factor is not a free variable. R63S, R63ZA and the R63ZB
binary128 comparator all use the same binary64 upper factor at root
`a7a85789364f5af91aaf71e1531759713c95ec085e48b7ca614daa524c3a4f85`
and the same permutation. Varying it would add an unnecessary third cause.

## Existing endpoints

The two diagonal cells of the required comparison already have frozen
evidence:

| Operator | Inputs | Existing result |
|---|---|---|
| binary128 tangent `sigma*T*T^T` | original binary128 RHS/scale | R63X exported state 2 passes R63Y at about `1.01e-3` |
| binary128 replay of R63Z K2 common block | recomposed projected RHS/scale | R63ZB comparator rejects all states at `12+/24-/66?` |

The second cell shares R63Y and is localization-only. It nevertheless fixes
the endpoint that R63ZC must reproduce before interpreting the two new hybrid
cells.

## Minimal factorial

Run exactly four binary128 recurrence lanes through states `x0,x1,x2`:

```text
                         input representation
                    original              projected K2
operator        +-------------------+---------------------+
tangent         | baseline          | input-only          |
                | R63X endpoint     | new discriminator   |
                +-------------------+---------------------+
common K2 block | operator-only     | combined endpoint   |
                | new discriminator| R63ZB comparator    |
                +-------------------+---------------------+
```

Every lane uses the same exported upper factor, permutation, two PCG updates,
fixed dot/update order and R63Y twofold certificate. The tangent operator
keeps its frozen original `sigma`; projected inverse scale affects only the
factor solve. Thus the input axis represents precisely the changes admitted
by the finite lane: projected RHS and projected solve scale, not a different
physical operator.

## Hypotheses and exhaustive interpretation

Let `T/O` be the required passing baseline, `C/P` the required rejecting
combined endpoint, `T/P` input-only and `C/O` operator-only.

| `T/P` | `C/O` | Interpretation |
|---|---|---|
| pass | reject | common-block representation is sufficient |
| reject | pass | input projection is sufficient |
| reject | reject | both perturbations are independently sufficient |
| pass | pass | only their interaction is sufficient |

No outcome selects a repair automatically. The report must also identify the
first unequal state/product/solve root and maximum coordinate differences.
R63ZB finite K2 versus the `C/P` binary128 cell remains a separate arithmetic
comparison; it cannot be folded into either factorial axis.

## Why this precedes another solver change

The exact common operator has an extreme weak direction. R63Q already showed
that tiny representation differences distinguish stored dense and tangent-
Gram forms, and R63S showed an arithmetic fixed point with 66 unresolved
signs under common semantics. Adding iterations, a third word, residual
replacement or another Krylov method now would confound cause with repair.

The factorial changes one admitted representation axis at a time while
holding recurrence, factor and certificate fixed. It can therefore falsify
operator-only, input-only and interaction explanations without a tolerance or
elapsed-time measurement.

## Boundaries

R63ZC is observational localization, not a portable producer. Binary128
candidate vectors may enter its R63Y certificates because no runtime candidate
is selected. Exact rational values remain containment/correspondence oracles
and cannot decide a hybrid outcome.

No result authorizes a new factor, more iterations, adaptive precision,
changed Krylov method, nonlinear state commit, corpus, timing, GPU, runtime or
production work. The final executable evidence must pass the repository's
fresh independent code/evidence review gate before interpretation.
