# NSR3-B4E2D7R20R63ZJ product-precision localization evidence

Status: `AUTHOR_PASS / INDEPENDENT_RE_REVIEW_NO_GO / INCONCLUSIVE`.

## Outcome

The repaired executable deterministically reproduces its author JSON, callback
traces and reject/reject/pass certificate counts. That mechanical fact is not
enough to admit the intended localization claim. The single independent
re-review found two load-bearing resealed-drift paths in the public validator:

1. recurrence states do not bind the complete certificate semantics back to
   the state solution and frozen verifier; a state solution or route-bearing
   certificate field can change while a fully resealed hybrid remains valid;
2. the frozen recurrence-work ledger is not validated beyond
   `work.certificates == 3`, and callback-validation replay performs additional
   unsealed work that affects apparatus, controls and route.

Under the one-re-review rule, R63ZJ is closed as `INCONCLUSIVE`. The route
`DENSE_TWOFOLD_OPERATOR_PRODUCT_PRECISION_INSUFFICIENT`, preservation of the
rest of the K2 recurrence, and promotion to a portable representation
discriminator are not supported.

The full review record is preserved in the
[independent re-review evidence](nonlocal-nsr3b4e2d7r20r63zj-independent-rereview-evidence-2026-08-28.md).

## Frozen identities and reproducibility

| Artifact | Identity |
|---|---|
| Initial reviewed snapshot | `caaa16b4f91a622dd36bf55026529c0853fd6d16` |
| Repaired candidate snapshot | `9d3482bfc557cd20367c8082a769f36884bc20fd` |
| Hash-closed re-review packet | `5161432c4e74342ef8a1bced33622147f977a4f4` |
| Repair diff SHA-256 | `e644da04a688fa797a3698bed3ff1efd98d0b926dc9352e29a013c70d495b505` |
| Frozen contract blob | `713f9ab948528e5f8db7fc1db2b0daa211d5957425b951a25862e545ca263696` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Author and reviewer stdout | `b9ded7d71e0c19aed85ee51f8ba9d923569aeee9c11b02a7060a267672b27f8a` |
| Clean reviewer Release binary | `a468e64c932721234a5a3e8192dd0282b2a7e109867ff33daaff9489c63fd0e2` |
| Author result semantic | `35a57274e548076908588c3a3e6c9bc808d3a2a314b1aed54a01777070d4d0d2` |

Dev, both author Release runs and both detached reviewer Release runs are
byte-identical. The reviewer also reproduced R63ZI, R63ZG and R63ZH stdout at
`98736993...080a1e`, `ca2a0f80...029c9` and `181ac246...96722`.

## Author observation retained without causal authority

The repaired snapshot publishes:

- three complete callback traces at `Kx0`, `Kp0` and `Kp1`;
- hybrid recurrence root `381cfa95...a7cd`, callback identity
  `a8330d4b...44c1` and callback aggregate `c8cdbf61...1564`;
- state-2 certificate `24+/78-/0?` with sign root `89b2908b...6094`;
- exact product audit `306/306`, root `36433384...b7e6`;
- candidate work `3/6`, dots `1890/612`, terms `192780/192780`, scale and
  input components `612/612`, projections/additions `612/306`;
- exact frozen rejecting dense/common endpoints and their certificate roots.

The independent review confirmed the callback identity, input mapping,
separate high/low binary128 tangent products, K2 projection/addition and
bit-exact product replay. These are apparatus facts only. Because the final
state, certificate and work correspondence is not closed, the observed ladder
cannot carry the R63ZJ causal interpretation.

## Load-bearing counterexamples

### State and certificate drift

The state root seals only `certificate.root`; the certificate-set root hashes
the mutable certificate fields beside that unchanged root; the recurrence
validator never recomputes a certificate from the state solution and verifier;
and `ladder()` consumes those mutable fields.

An external temporary harness changed one state-2 solution component and
resealed the public roots, then separately changed
`state2.certificate.passed` and resealed all dependent roots:

```text
original_valid=1 state_solution_valid=1 certificate_valid=1 original_ladder=1 certificate_ladder=0
```

- stdout SHA-256:
  `dd0bb518863dd992d94f0c7dc18fdd0c614db615235afa1aa14bedd7d04e068e`;
- harness binary SHA-256:
  `985db9e6247180fb9e6cedcf2f38d242937fa2c9c572d47f4bfe077cf9f1cb51`.

### Recurrence-work drift

The same style of harness changed `recurrence.work.factor_solves` from `3` to
`4` and resealed recurrence, callback and hybrid roots:

```text
original_valid=1 recurrence_factor_solves=4 resealed_work_valid=1
```

- stdout SHA-256:
  `54a23054d31b997616a526892dfd83dbba61879a5128019666d16fe01e038437`;
- harness binary SHA-256:
  `3736a94d83db88abeb9c6f86721e2af0a958945e5f9a3eab285e850fe76d2227`.

Callback validation also performs eight additional kernel calls in the frozen
main control path, totalling `2520/816` dots and `257040/257040` terms. This
replay affects validation outcomes but has neither an independent sealed work
ledger nor ownership in candidate `hybrid_work`.

## Decision and smallest next action

Do not repair, review or promote R63ZJ again. Preserve its raw deterministic
observation as untrusted input only. The next experiment must be a separately
frozen verifier-first boundary that:

- derives every state transition and every certificate from the frozen
  fixture rather than trusting author DTO fields or author route;
- compares complete state/product/certificate traces, not opaque nested roots;
- freezes candidate and validation-replay work separately;
- rejects state, certificate, transition and both work-ledger mutations before
  classifying the numerical observation.

Until that succeeds and receives fresh review, no product-precision
localization, representation selection, dynamic builder, corpus, timing,
runtime, GPU or production authority exists. SPEC-38 and ADR-076 remain
`Proposed`; later continuum ProductChecks remain `NOT_RUN`.
