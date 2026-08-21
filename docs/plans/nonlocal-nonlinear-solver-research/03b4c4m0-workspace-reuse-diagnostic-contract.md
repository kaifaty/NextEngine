# NSR3-B4C4M0 -- workspace-reuse diagnostic

Status: `FROZEN / MEASUREMENT_AUTHORIZED / B4D_BLOCKED`

Parent B4C3MC1 passes with JSON-without-final-LF SHA-256
`d19d2f451a48bdc400de03edb157356e3eec8b7ef20e79d3fb2a808b69c44d7c`
and semantic SHA-256
`91d6eae1e9789276797785d0672fe748dcaf99178635f322385cb9e6c95dae24`.

## Identity

```text
sha256  5dc38b6fc6e5dae57954357d53c1960bb5fe7444a70f558ee0f963ce216ff6f1
text    nextengine.nonlocal.workspace-reuse-diagnostic|v1|scope=macro-p1,p2|evidence=query-state-kind-static-support-nested-rows|thresholds=none
```

## Execution

Run one selected canonical-topology macro-adaptive transaction for each exact
P1/P2 fixture and scenario identity. Enable the already existing query-record
path only for this diagnostic. Solver equations, workspace construction,
adaptive selection, KKT/contact, canonical publication and roots are unchanged.

The P1/P2 transaction results and roots must equal independent record-disabled
controls. Query recording must not alter the existing query-chain root.

## Classification

Classify every recorded request into exactly one stable category:

```text
FRAME_START
FORECAST
INTERVAL_INITIAL
CURRENT
TRIAL
SUBSTEP_DIAGNOSTIC
CANONICAL_LEDGER
OTHER
```

Report total builds, unique full workspace state hashes, duplicate builds,
consecutive equal-state transitions and per-category counts. Report current
static-support rebuild count and the counterfactual lower bound of one index
per fixture. Report nested adjacency rows and directed records constructed in
addition to the flat pressure-tape CSR.

## Gate and authority

Gate only complete classification, exact recorded/unrecorded transaction
correspondence, exact roots/query-chain and finite bounded counters. No
performance threshold or optimization selection exists.

Two full reports must be byte-identical and reproduce B4C3MC1 at its exact
parent hash. PASS authorizes only a separately frozen B4C4 packaging design.
B4D reference execution, nominal corpus, CUDA, runtime/schema and production
remain blocked.
