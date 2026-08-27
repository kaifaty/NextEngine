# NSR3-B4E2D7R20R63ZC operator/input factorial contract -- revision 3

Revision 3 preserves the same four cells, endpoints and classification, but
requires every lane to prove that its actually consumed operator, factor,
permutation, RHS, inverse scale and verifier-profile roots match the declared
cell identity. The consumed verifier profile is revalidated from its live
payload rather than trusted through a cached root. The revision also binds the
complete work ledger into every transaction root and names the first
overflowing operator-product boundary precisely.

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZC` |
| Parent | reviewed R63ZB result `acfe6baf...a709ebf` |
| Claim class | fixed binary128 `2x2` representation-localization factorial |
| Budget | four lanes; states 0..2; no timing |
| Consumer | selection of one later repair discriminator only |

## Frozen factors

Cross exactly these two axes:

1. operator: original binary128 tangent action versus binary128 reconstruction
   of the immutable R63Z K2 common-block components;
2. inputs: original binary128 RHS/inverse scale versus K2 project-and-recompose
   RHS/inverse scale from the immutable R63ZA fixture.

All cells use exported factor root
`a7a85789364f5af91aaf71e1531759713c95ec085e48b7ca614daa524c3a4f85`,
permutation root
`335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f`,
the original frozen tangent `sigma`, dimension 102 and the same recurrence
order. Projected inverse scale changes triangular consumption only.

## Recurrence and certificates

Each lane performs the same complete sequence:

1. factor solve for `x0`;
2. `r0=b-Hx0`, factor solve for `z0`, `p0=z0`, positive `rho0=r0^Tz0`;
3. two fixed PCG updates with positive denominator/rho checks;
4. seal states `x0,x1,x2` before all certificates;
5. run the immutable R63Y twofold affine certificate on every state.

No lane stops on an early certificate. Scalar, vector, product, solve, state
and certificate roots bind operator/input cell identity.

## Endpoint correspondence

- Tangent/original must reproduce all three R63X exported solution roots and
  their sealed solution-set root
  `d0b42562f5ed45efdf2d454f1c1f6ec93f587fe8045943517f9af076d279c98e`,
  plus
  the frozen reject/reject/pass R63Y ladder with state-2
  `24+/78-/0?` and sign root
  `89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094`.
- Common/projected must reproduce the R63ZB binary128 comparator root
  `7db8a84ee9cee6506533b4d634d051ee9dfb8e56b23f8cbd06222ddba3b1c8e3`
  and its three `12+/24-/66?` rejections.

If either endpoint correspondence fails, reject before interpreting the new
hybrid cells.

## Classification

After valid endpoints, classify only from complete R63Y ladders:

1. `OPERATOR_INPUT_FACTORIAL_APPARATUS_REJECTED`;
2. `OPERATOR_INPUT_FACTORIAL_IDENTITY_REJECTED`;
3. `OPERATOR_INPUT_FACTORIAL_WORK_REJECTED`;
4. `TANGENT_ORIGINAL_ENDPOINT_REJECTED`;
5. `COMMON_PROJECTED_ENDPOINT_REJECTED`;
6. `COMMON_OPERATOR_PERTURBATION_SUFFICIENT` when `T/P` passes and `C/O`
   rejects;
7. `INPUT_PROJECTION_PERTURBATION_SUFFICIENT` when `T/P` rejects and `C/O`
   passes;
8. `BOTH_PERTURBATIONS_INDEPENDENTLY_SUFFICIENT` when both hybrid cells
   reject;
9. `JOINT_OPERATOR_INPUT_INTERACTION_REQUIRED` when both hybrid cells pass.

`Pass` means the exact R63Y reject/reject/pass ladder, not merely a smaller
error or distance to another lane. No numerical threshold is fitted.

## Fixed work

```text
lanes                           4
states/certificates            12
operator products              12
factor solves                  12
factor terms              123,624
factor divisions            2,448
Krylov scalar dots             16
Krylov scalar divisions        12
solution updates              816
residual updates              816
direction updates             408
adaptive stops                  0
```

The two tangent lanes execute six two-stage rectangular products; the two
common lanes execute six dense common-block products. Report structural term
counts separately. Certificate- and oracle-isolation replays are controls and
are explicitly excluded from this four-lane work ledger. No elapsed-time
sample is admitted.

## Controls

1. A small sealed `2x2` factorial proves input changes can affect state 0,
   while operator-only changes cannot affect state 0 but do affect the first
   residual/product.
2. All four cells share factor/permutation/recurrence/certificate identities.
   Every accepted lane joins its actually consumed operator, factor,
   permutation, RHS, inverse scale and profile roots back to the declared
   identity; a resealed wrong-payload assignment or relabeling rejects.
3. Mutating original/projected RHS or scale, tangent/common operator, factor,
   state, order, profile or endpoint root rejects.
4. Dimension-invalid input and an actual overflowing `Hx0` product reject at
   `initial_product`. A reachable zero rho and negative denominator at the
   early and later recurrence boundaries reject before partial state
   publication, with every tracked work counter checked exactly. Negative rho
   is not manufactured under the frozen positive-definite preconditioner.
5. Certificate or exact-oracle mutation cannot alter any recurrence root.
6. Tangent/original and common/projected endpoint roots are independently
   reconstructed rather than copied from cached success flags.
7. Every classification branch and first-failure precedence has a negative
   control.
8. The same nested execution captures the actual R63Y, R63Z and R63ZA JSON
   bytes, and the immediate R63ZB JSON bytes; all four newline-terminated
   stdout hashes must remain byte exact.
9. Freeze source/diff/binary/command/stdout hashes and require fresh
   independent code/evidence review before interpreting the result.

## Ceiling

Success selects only the cause class for one subsequent frozen repair
experiment. Failure localizes an apparatus, identity or endpoint mismatch.
Neither authorizes a portable producer, new precision width, tolerance,
iteration count, factor, Krylov method, corpus, timing, GPU/runtime path,
nonlinear commit or production claim.
