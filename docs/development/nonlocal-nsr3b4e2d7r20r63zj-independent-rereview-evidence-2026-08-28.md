# NSR3-B4E2D7R20R63ZJ independent re-review evidence

Status: `NO_GO / R63ZJ_INCONCLUSIVE`.

## Verdict

The single batched-repair re-review returned `NO-GO`. The repaired snapshot is
reproducible and closes the initial callback/product and frozen-endpoint
findings, but two remaining load-bearing validation defects admit fully
resealed semantic drift. Per the review budget, the candidate was not edited
or repaired and R63ZJ ends as `INCONCLUSIVE`.

## Verified snapshot

The reviewer verified:

- packet `5161432c4e74342ef8a1bced33622147f977a4f4`;
- repair `9d3482bfc557cd20367c8082a769f36884bc20fd` and parent
  `caaa16b4f91a622dd36bf55026529c0853fd6d16`;
- repair diff
  `e644da04a688fa797a3698bed3ff1efd98d0b926dc9352e29a013c70d495b505`;
- contract and four source blob identities from the
  [re-review request](nonlocal-nsr3b4e2d7r20r63zj-independent-rereview-request-2026-08-28.md);
- `generalization_v5_twofold_recurrence.inc` blob
  `e0197c475c60aa09c7b37ffd6accd32359da19f40a94efe2e3f2d1b4e4cb68e7`;
- `git diff --check`: pass.

One detached clean Release build and two focused runs produced identical
stdout
`b9ded7d71e0c19aed85ee51f8ba9d923569aeee9c11b02a7060a267672b27f8a`
and binary
`a468e64c932721234a5a3e8192dd0282b2a7e109867ff33daaff9489c63fd0e2`.
The cache was exact at
`23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84`.

Focused regressions remained byte-exact:

| Probe | stdout SHA-256 |
|---|---|
| R63ZI | `98736993d50ae29deebdb48361c0079e0aa3443ca3bfd1775ecbddbcd0080a1e` |
| R63ZG | `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9` |
| R63ZH | `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722` |

## Finding P1: state/certificate correspondence is resealable

`FormulaProbeCertificate` exposes route-bearing mutable fields. The state root
binds only its nested `certificate.root`; the certificate-set root binds the
mutable fields alongside that root; and recurrence validation does not derive
the certificate again from `state.solution_components` and the frozen
verifier. Callback replay checks operator products but not recurrence
transitions or certificates. The final `ladder()` trusts the mutable fields.

The reviewer demonstrated both a resealed state-solution mutation that remains
valid with a stale certificate and a resealed certificate mutation that
changes `ladder()` while full validation remains true:

```text
original_valid=1 state_solution_valid=1 certificate_valid=1 original_ladder=1 certificate_ladder=0
```

Harness stdout/binary SHA-256 are
`dd0bb518863dd992d94f0c7dc18fdd0c614db615235afa1aa14bedd7d04e068e`
and
`985db9e6247180fb9e6cedcf2f38d242937fa2c9c572d47f4bfe077cf9f1cb51`.

Exact source locations at the reviewed snapshot:

- mutable certificate fields: `formula_probe_api.hpp:38`;
- certificate-set root: `boundary_reference.cpp:80392`;
- state root: `boundary_reference.cpp:80679`;
- callback replay boundary: `boundary_reference.cpp:80837`;
- recurrence certificate validation: `boundary_reference.cpp:81621`;
- route predicate: `formula_product_precision_localization_main.cpp:87`.

## Finding P1: recurrence and replay work are not closed

The frozen recurrence work is produced in
`generalization_v5_twofold_recurrence.inc:616`, but the public recurrence
validator checks only `work.certificates == 3` at
`boundary_reference.cpp:81610`. The hybrid validator checks `hybrid_work`, not
the recurrence ledger, at `boundary_reference.cpp:82021`; the main negative
control mutates only `hybrid_work.output_projections` at
`formula_product_precision_localization_main.cpp:295`.

Changing `recurrence.work.factor_solves` from `3` to `4` and resealing every
dependent public root still validates:

```text
original_valid=1 recurrence_factor_solves=4 resealed_work_valid=1
```

Harness stdout/binary SHA-256 are
`54a23054d31b997616a526892dfd83dbba61879a5128019666d16fe01e038437`
and
`3736a94d83db88abeb9c6f86721e2af0a958945e5f9a3eab285e850fe76d2227`.

The callback validator also performs unsealed replay work at
`boundary_reference.cpp:80863`. In the frozen main path the additional eight
kernel calls total `2520/816` dots and `257040/257040` terms. They influence
apparatus, controls and route but belong to neither candidate `hybrid_work`
nor a separate sealed replay ledger.

## Positive audit facts and claim ceiling

Callback identity, the three state-input mappings, separate high/low
binary128 products, K2 projection/addition, bit-exact product replay,
dense/common endpoint roots, six endpoint certificate roots, route precedence
and the author candidate-work arithmetic were correct. The old self-derived
product radius no longer admits finite product drift by itself.

Only this mechanical statement is permitted: on the frozen host and cache,
the author executable deterministically reproduces the recorded JSON and
roots. The scientific route, preservation of the remaining K2 recurrence and
any portable-representation successor are not admitted by R63ZJ. Timing,
runtime, GPU, corpus, generalization and production authority remain false.

The reviewer changed no files or commits. Assigned and detached review
worktrees remained clean. ProductChecks were `NOT_RUN` as required for this
localized offline C++ discriminator.
