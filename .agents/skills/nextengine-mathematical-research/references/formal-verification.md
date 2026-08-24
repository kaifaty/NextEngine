# Formal and machine-verification boundary

Use formal verification only when it reduces a named uncertainty more cheaply
or more reliably than another independent derivation or test.

## Select the verifier by claim

| Claim shape | Useful verifier | Required caveat |
| --- | --- | --- |
| Algebraic identity or symbolic derivative/integral | CAS plus independent substitution or second derivation | Record assumptions, branches, singularities and simplification domain |
| Finite combinatorial/property claim | Exhaustive checker, SAT/SMT/ILP certificate or decidable Lean theorem | Prove the encoded domain and certificate/range correspondence |
| Numeric inequality or rounding bound | Exact rational, arbitrary precision or interval/enclosure method | State rounding mode and enclosure validity over the full domain |
| Reusable theorem with subtle logical dependencies | Lean or another proof assistant | Formalize the actual statement and audit imports/axioms |
| Model-to-code behavior | Property tests, independent reference code and correspondence evidence | A theorem about an ideal model does not execute the implementation |

Do not use theorem-prover effort to compensate for an unstable question,
unknown boundary condition, uncalibrated physical model or missing empirical
consumer.

## Audit a CAS result

1. State the input expression and variable domain.
2. Record all generated conditions and branch choices.
3. Test singular, equality and boundary cases separately.
4. Substitute the result back or differentiate/recompose it.
5. Recheck load-bearing identities with another method or tool.

CAS output without its assumptions is a conjecture generator, not a proof.

## Audit an exact computational certificate

1. Define the finite domain and why it covers the claim.
2. Separate certificate generation from a small checker when practical.
3. Keep the checker simpler and independently reviewable.
4. Use exact arithmetic and canonical ordering.
5. Re-run a successful and deliberately invalid certificate control.
6. Record tool version, command, input/certificate/checker hashes and result.

An exhaustive result is universal only over the proved finite encoding.

## Audit a Lean result

1. Freeze a faithful statement before proving it; document the notation and
   units mapping from the research contract.
2. Search existing declarations before inventing a duplicate lemma.
3. Use the repository/project-pinned Lean and Mathlib environment when one
   exists. Treat official Lean skills and declaration search as optional
   backends, not as research authority.
4. Compile the target and dependency cone with no `sorry`, `admit`, unexpected
   `axiom` or hidden native oracle. Record `#print axioms` or equivalent.
5. Have a second pass compare the compiled statement—not only the proof—to the
   frozen claim and every assumption.
6. Record source files, imports, toolchain revision and exact successful
   command.

If Lean or a declaration-search backend is unavailable, report formal
verification as `NOT_TESTED`. Do not imply installation, API access or proof.

## Close the correspondence gap

Formal proof closes only the encoded statement. Before making an engine claim,
separately map:

- mathematical variables to contract fields and units;
- real/integer/fixed-point arithmetic to the executed finite-precision model;
- abstract update/order to actual Rust/PhysX/GPU stages and reductions;
- theorem preconditions to validation and runtime invariants;
- proved output to immutable probes and ProductCheck observables.

Classify that bridge as `CORRESPONDENCE`. A missing bridge caps the result at a
model theorem, regardless of how strong the proof is.
