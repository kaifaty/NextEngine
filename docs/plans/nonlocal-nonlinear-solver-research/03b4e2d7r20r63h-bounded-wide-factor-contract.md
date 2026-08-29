# NSR3-B4E2D7R20R63H bounded-wide-factor contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63H` |
| Architecture snapshot | R63G semantic `d1b09efa...25e1eb`; binary64 storage survives; strict-binary64 QR factor loses weak signal |
| Engineering consumer | Locate whether wider precision is required only during factor construction or through factor consumption |
| Claim class | Stored-binary64/promoted-factor correspondence plus original/stored weak-signal separation |
| Claim status target | wide factor rejected, wide state required, binary64 export candidate or exact apparatus boundary |
| Budget | One capture-only parent replay; exact R63G reconstruction; one additional binary128 factor of stored coefficients; one cast-only export; no RHS, triangular solve, iteration, rank action, trajectory or timing |

## Exact claim

The complete R63G subject, factors, audits, signals, semantic inputs and route
repeat. Its once-rounded binary64 input is promoted exactly to binary128 and
factored without changing permutation or Householder construction.

The promoted factor closes orthogonality, stored-input reconstruction and the
stored-operator Gram. Its inherited weak-direction image lies strictly inside
both the stored-image and original-image signal balls. A one-time binary64
export of the completed factor is independently classified by the same two
gates.

## Exact negation

The first platform, parent, capture, R63G root, input-promotion, stored-Gram,
permutation, control, factor, orthogonality, reconstruction, Gram, signal,
export, work or lifecycle gate fails. A global small residual cannot override
a failed weak-direction gate.

## Frozen parent roots

Require:

```text
R63G semantic     d1b09efa20c088b60796c83b33d5870b9db49b26fa8e0f44477ec22c3825e1eb
transpose         8b5db34c925973b26d0761ea2b3e050ca9e665c42a8879716fe3b3fa17588061
permutation       335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f
q128 factor       fde9aef5ee0233c920ab76c855c612cc722422c67f44d18598211ae901082ec4
f64 factor        7cfe979b17c53cf81f4e69fe4ecb51223bb87e97d0b149dcc86df31723818d5d
q128 audit        b4a89421e503e954c8fffb219e2178f99ca9fb07f5375269114a6cf5201ca9bf
f64 audit         e0f5fd36c405f0e563053fa9b43bfe4e561e0b66b48725ae1c58b32128b80b21
```

Also require the four R63G squared signal errors and reference norm to repeat
bit-exactly. The command is not called recursively; the one parent replay
reconstructs these private artifacts in the same process.

## Stored-operator construction

- Round the complete unpermuted `315 x 102` transpose once to binary64, then
  promote each stored value exactly to binary128. Bind its root.
- Build `G64=C64^T C64` independently with the binary128 accumulator and bind
  all 10,404 entries.
- Promote the already permuted `C64 P` factor input exactly. Its value order
  must equal applying the immutable permutation to the promoted unpermuted
  storage.
- No source may read the original binary128 coefficient after this promotion
  boundary except the original-image audit.

## Promoted factor and export

- Call the unchanged binary128 R63G Householder routine once on promoted
  `C64 P`; require 102 finite nonzero diagonals and exact frozen work counts.
- Run the unchanged binary128 factor audit against `G64` with full containment.
- Export `Q` and `R` by one `static_cast<double>` per scalar, then promote those
  stored exports back to binary128 solely for audit.
- The export factor audit must be finite and publish all three global
  residuals. It has no gamma-based pass authority over the signal gates.
- Bind promoted input, stored Gram, wide factor, wide audit, exported `Q/R`
  and export audit roots separately.

## Weak-direction gates

Reuse exact `alpha`, original `u` and stored `u64` from the parent
reconstruction. Compute:

```text
u_wide   = Qwide (Rwide P^T alpha)
u_export = Qexport (Rexport P^T alpha)
```

For each candidate publish and root:

- error vector and squared error versus `u64`;
- error vector and squared error versus `u`;
- maximum component residual for each reference;
- strict stored/original survival booleans.

The wide factor passes only if both squared errors are strictly less than the
corresponding nonzero reference squared norms. Export classification uses the
same rule. Equality fails.

## Controls

1. A literal binary64 tall full-rank input promoted to binary128 closes factor,
   stored Gram and both identical-reference signal gates.
2. A literal weak input whose promoted factor survives but whose deliberately
   perturbed binary64 export leaves the signal ball selects wide-state-required.
3. Exact promotion round-trips every finite literal binary64 bit pattern used
   by the controls.
4. A nonidentity complete permutation closes the promoted-input identity.
5. Mutating one stored coefficient, wide factor scalar or exported scalar
   changes its root.
6. Shrinking either open signal ball to the observed boundary rejects the
   candidate.
7. Classifier precedence covers all four routes.
8. R63B--R63G byte regressions pass. RHS/triangular/iterative/SVD, rank action,
   row drop, regularization, center, replacement, state and timing counts are
   zero.

## Resolution precedence

1. First apparatus/control/parent/correspondence failure.
2. `STORED_OPERATOR_WIDE_FACTOR_REJECTED` if either promoted-factor signal
   obligation fails.
3. `WIDE_FACTOR_STATE_REQUIRED` if both promoted-factor obligations pass and
   either export obligation fails.
4. `WIDE_BUILD_BINARY64_EXPORT_CANDIDATE` if all four obligations pass.

## Does not count

An NNQP RHS or triangular solve; iterative refinement; LSQR/LSMR/SVD;
effective-rank selection; row deletion/replacement; regularization; global or
production stability; default runtime binary128; center, trajectory, timing,
runtime/GPU or production inference.

## Stop and reconsider

- Wide state required: freeze one wide triangular RHS solve, cast only the
  final candidate, and verify the original Gram residual/sign semantics.
- Export candidate: use exported factor as an equal-work RHS control against
  the retained-wide solve.
- Wide factor rejected: research reorthogonalized Golub--Kahan or Jacobi-SVD;
  do not widen the entire runtime or weaken the signal gate.
- R64 and R65 remain blocked for every route.
