# NSR3-B4E2D7R20R63P finite-projector idempotence evidence

Status: `PASS / PROJECTOR_RANK_ONE_EXPLANATION_REJECTED`.

Claim status: finite-projector non-idempotence is `SUPPORTED_EXACT`, but its
causal explanation of the dense/direct inverse split is `REFUTED_EXACT`.
Evidence classes: `EXACT_CERTIFICATE`, `NUMERICAL`, `CORRESPONDENCE`.

## Reproducible result

Implementation commit: `841301a9`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-projector-idempotence
```

Two independent executions are byte-identical:

```text
stdout sha256   e4cffa7dfa4e56c3a7360895b95d5725c75db8dc485d250f4db775a372f271f0
semantic sha256 b7cae5137a2e9c3cd9acdd678044dab5fcf820172db15ea13981b443c765e3e1
route           PROJECTOR_RANK_ONE_EXPLANATION_REJECTED
controls root   806d258c6882829c89eb023aa0e85288540efd05b65f80347e7f7dd5756def05
```

Platform, controls, exact R63O reconstruction, dyadic conversions, arithmetic
work and lifecycle gates pass. The negative route is scientific, not an
apparatus failure.

## The projector is non-idempotent, but by far too little

For all 174 frozen free coordinates, exact dyadic summation gives:

```text
s = exact y^T y       3.906250000000000000000000000000000000e-3
stored d              3.906249999999999999999999999999999624e-3
s-d                   3.790292054128948475267202486643231859e-38
```

Thus `P_d=I-yy^T/d` is exactly non-idempotent. The independently constructed
rational numerators satisfy, without rounded division,

```text
P_d^2-P_d = ((s-d)/d^2) y y^T.
```

However, on the immutable R63E weak witness the induced ideal operator defect
is only:

```text
delta = alpha^T(K*-H*)alpha
      = 8.727499537028996667178514639164530729e-68.
```

The exact observed gap between the two stored representations is:

```text
k_observed-h_observed
    = 3.483204074806985512040028117159384070e-34.
```

After subtracting the derived rank-one term, the residual remains
`3.483204074806985512040028117159382957e-34`. It is not smaller than both the
observed and predicted gaps, so the frozen dominance gate fails. The projector
norm-rounding term cannot explain R63O.

## The structured operator is much closer to the common oracle

The common exact rational source/JVP/source oracle is

```text
H* = sigma A P_d A^T.
```

Exact cross-product comparisons establish both local model associations:
stored dense `H` is infinitesimally closer to `H*` than to `K*`, and stored
tangent Gram is closer to `K*` than to `H*`. The important absolute result is
different:

```text
representation                  exact weak-form error to H*
stored dense H                  3.483204074806985492979103879731054283e-34
stored tangent sigma T T^T      1.906092423742832992158038209594096499e-51
```

The structured tangent representation is about `1.8274e17` times closer on
this frozen weak direction. Its ideal `P_d^2` deviation is negligible here;
the dense matrix's per-entry rounding is amplified by cancellation in the
near-null quadratic form.

This also explains why R63O can be simultaneously correct and misleading if
read too broadly: direct PCG does not solve the *stored dense* system, but the
stored dense system is not established as the most accurate finite
representation of the original derivative.

## Exact work and roots

```text
source-combination products  32,130
dense quadratic terms        10,404
tangent-combination products 32,130
tangent norm terms              315
observation root             a8e4311989c3c8d1e80d6f7d8b0fb840d2e35775de4f9198fad7e24ab124e5a5
arbitration root             c535acfe3323cdb83b5dfe30d572136138388d3efda2e32b3c108f3775d66196
source root                  37b6732c452875321961d757b66fc36cb652e2dec5b2b3501cb4b19e6af75f66
tangent-image root           3ad94d01d2cfb0fc286bed027b5d68d39058f1824e064592164d5dc98adbd930
```

There are zero RHS/factor solves, PCG updates, rank-one corrections, sparse
constructions, state updates or timing samples.

## Meaning and next discriminator

Do not repair `K` toward stored dense `H`, and do not yet grant `K` operator
authority from one weak witness. R63Q should construct the complete exact
rational `102 x 102` common oracle `H*`, compare both materializations over all
entries and deterministic sensitive directions, and establish an operator
error norm before any RHS solve. Only after this full-matrix arbitration may a
structured RHS/PCG certificate replace the dense-system certificate.

The direct source/JVP/source form remains a useful independent implementation,
but R63P does not select it over tangent Gram: their exact semantic difference
on the witness is only order `1e-68`.

R63B--R63O stdout files remain byte-identical to all frozen hashes. Build
passed. No CPU/wall performance comparison, runtime state, GPU or production
inference occurred.

