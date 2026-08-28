# NSR3-B4E2D7R20R63ZK independent re-review evidence

Status: `NO_GO / R63ZK_INCONCLUSIVE`.

## Verdict

The single revision-2 re-review returned `NO-GO`. The reviewer found one
remaining load-bearing checker-work omission. Under the frozen one-re-review
budget, R63ZK is closed as `INCONCLUSIVE` and receives no second repair.
R63ZJ remains `INCONCLUSIVE`.

## Load-bearing finding

Each independently replayed product calls the high and low tangent kernel, so
the three sites execute `formula_probe_tangent_product` six times. The wrapper
runs `al_formula_probe_tangent_payload_valid` before every kernel call, and
that targeted validation recomputes the complete tangent payload root.

The published checker receipt counts one fixture admission, three factor
payload validations and three profile payload validations, but has no category
for the six tangent-payload validations or their six payload hashes. Its root
and exact predicate therefore cannot seal all executed validation/hash work.
This directly violates revision-2 work ownership, which forbids invisible
repeated validation and root/hash paths.

Reviewer source trace:

- high/low calls and three sites:
  `formula_independent_recurrence_checker_main.cpp:335-361`, `:601-603`,
  `:626-627`, `:689-690`;
- repeated targeted validation:
  `boundary_reference.cpp:81244-81253`;
- full tangent payload hash:
  `boundary_reference.cpp:80421-80429`;
- omitted receipt category/root/exact constants:
  `formula_independent_recurrence_checker_main.cpp:52-89`, `:262-287`,
  `:1244-1280`.

The minimum mechanical correction would count six tangent validations and six
tangent payload hash derivations, or remove the repeated validation by using a
single immutable admitted kernel context. It cannot be applied to R63ZK. Any
such work requires a new experiment/package and new contract.

## Repairs independently confirmed

No other load-bearing finding remained. The reviewer confirmed:

- exactly one untrusted producer capture and a separately implemented replay;
- all four strict positivity guards in transaction order;
- explicit K2 high/low DTO payloads and bit-exact scalar comparison;
- certificates derived from replayed solutions;
- full product/state/certificate/work semantic comparison;
- first-specific mismatch routes plus unknown fail-safe;
- distinct producer/checker/control receipts;
- all `64` controls, `61` control comparisons, `19` author validations,
  `114` replay kernels and `96` classifier cases;
- structured fail-closed malformed-cache handling and false authority fields.

These facts are mechanical author/reviewer observations only. Because work
separation is incomplete, they do not establish fixed-profile trace
correspondence.

## Identity and clean execution

All frozen identities matched:

| Item | Identity |
|---|---|
| Contract / parent | `3097bc08a31a2a4c017cb696a6990a3138ff056a` |
| Repaired snapshot | `7635d664fe35bbfde725956307e62f6a598009ab` |
| Complete diff | `2e652018528d5d03f8921f970070254587c181fbc3a9f85942ef504c3f3fac63` |
| Neutral request | `a4e552c37805246b4a79db6cd32bcf91c2eb787197909dec5b91125fced5cbae` |
| Contract blob | `0a231bb60646b4c4d655bcba2747251b9cd8a68edd38f3cb85af5ec288cc40ea` |
| Checker main | `02bec9996c754fc7c12ac3b86617ff3350049ccbf57c452018023d6a1e018063` |
| Private API | `13b0c894805d4a57968afbfd0f146c686c475d72b7ff0d78d7838c18f1ce4f77` |
| Boundary primitives | `621300de50157b1e3da1adfc114618990460f16217678aa99787d1c2caa0a899` |
| CMake | `397deaf39a47d3e1a2cdc1a265f741caa0bb092fd682953730ef5138fa3c6dd3` |
| Cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |

The detached tree remained clean. A clean Release binary reproduced
`4ccee9be...d35c`; two executions were byte-identical at
`73df9f1b...2755`. `git diff --check` passed.

Regression stdout remained exact:

- R63ZI: `98736993d50ae29deebdb48361c0079e0aa3443ca3bfd1775ecbddbcd0080a1e`;
- R63ZG: `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9`;
- R63ZH: `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722`;
- R63ZJ: `b9ded7d71e0c19aed85ee51f8ba9d923569aeee9c11b02a7060a267672b27f8a`.

ProductChecks remain `NOT_RUN` under the frozen offline-checker packet.

## Claim ceiling and next rule

Allowed claims are mechanical only: revision 2 builds deterministically,
reproduces its declared JSON and semantic comparisons, and preserves four
regressions. Fixed-profile correspondence is not established.

Portable representation, width three, dynamic building, corpus/generalization,
adaptive stopping, timing, runtime/Rust/GPU integration, ProductChecks and
production promotion remain blocked. Do not retry R63ZK. A successor must be a
new experiment/package with an immutable admitted tangent context or an
explicitly complete compositional validation/hash receipt.
