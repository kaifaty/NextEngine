# NSR3-B4E2D7R20R63ZP revision-5 independent review evidence

Status: `NO_GO / DIRECT_PRODUCT_UNADMITTED / ONE_BATCHED_REPAIR_REMAINS`.

## Verdict

The first formal review rejected the revision-5 serialized package. The fixed
numerical apparatus remains reproducible, but candidate/checker causal order,
partial-route authority, exact-work accounting and the named control matrix do
not meet the frozen contract. No direct product, recurrence, certificate,
representation or production authority is granted.

## Identity and reproduction closure

The reviewer independently confirmed source snapshot
`1fe8ee4feed42f135c4cfeef6e1222971d367df7`, tree
`1098bb647442551020066ed26e170a3671d3df2c`, base `38096294`, diff
`004c5cc2...d9e9e` and all frozen source/contract/input hashes.

Two clean Release builds reproduced producer `ddb2c7e8...575e5`, checker
`a1b9b250...9727`, candidate `82fc6740...f9f3`, audit
`21dffa88...a0a` and controls report `f7710c48...903f`. ASan/UBSan baseline,
the exact product root `271facfd...272f2f`, `102/102` containment, positive
curvature, contained step and `102/102` late update all reproduced.

## Load-bearing findings

1. **Oracle/future-state work crosses the candidate seal.** Checker
   `derive_replay` executes exact dyadic work and decodes all `102` cached
   `x1` values before candidate body verification. A downstream-resealed body
   mutation produced candidate `aa45a45c...84c78`, audit
   `e26cc780...792e8`, route 11, yet still reported `64673` dyadic decodes,
   `64566` multiplies, `64672` additions and `102` post-seal `x1` decodes.
2. **Routes 1--5 are not independently reconstructible.** Replay hardcodes
   only final route 6/7; invalid parents become checker route 13 instead of a
   verified producer route 1. Producer curvature/step failures do not seal the
   available product body.
3. **Fixed comparison work is overstated on short-circuit paths.** Interval
   and division checks use `&&` expressions but publish the full planned count
   even when later predicates did not execute.
4. **The nominal 174-control count does not cover the frozen named matrix.**
   Missing executable branches include zero-bound escape, negative/zero/
   nonfinite and exponent/capacity exact cases, denominator zero/negative,
   division miss, update rejection, role-2 causal consumption and explicit
   oracle/future-state-before-seal assertions.

## Minimum repair and budget

One coherent repair must:

- verify the candidate body and owning event before exact-oracle work, and a
  product-stage seal before `x1`;
- independently construct and verify partial receipts/routes 1 through 7;
- evaluate fixed predicates into temporaries before result folding and count
  actual execution; and
- replace nominal mutation volume with the frozen named arithmetic, causal,
  routing and early-stop controls, including zero forbidden work assertions.

Only one batched author repair and one formal re-review remain. A remaining
load-bearing finding closes R63ZP `INCONCLUSIVE`.
