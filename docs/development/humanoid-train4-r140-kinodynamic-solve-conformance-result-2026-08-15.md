# TRAIN-4 R140 kinodynamic solve conformance result — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Sole clean report-only implementation/numerical conformance for the R139 solve method |
| Status | `R140_PASS_R141_ROADMAP_DECISION_ONLY` |
| Gate decision | `PERMIT_EXPLICIT_ROADMAP_DECISION_FOR_ONE_BOUNDED_R141_ONLY` |
| Claim ceiling | Synthetic and metadata-only method conformance; no source payload, real graph, QP solve, candidate, PhysX or training |

## Immutable result

The sole R140 process ran from clean commit `0dfdccf` and passed all six frozen
validations. Canonical/file/profile SHA-256 is
`68cab54fccef71b3a30b28159740272547c289b3a04866f6d59979217b35901c` /
`5315eaf27d4ba3756cd119ff7260fa5dfaf664d2d7c5ff65a9d985f916a61f92` /
`dd2dd64f0f37e700256eaa1668967bcfd97560371193340dbcffae4adf769b6a`.
The canonical hash independently reproduces exactly.

R140 reads only the R139 JSON report and its tracked identities. Source array
payload reads are zero, and the import discriminator proves that NumPy, SciPy,
OSQP, Pinocchio and the R139 formulation module are not loaded. Every real
reconstruction, controller graph, dynamics/Jacobian/residual, assembly, QP,
factorization, exact trial, optimizer, cache/candidate, PhysX and training
counter is zero.

## Conformed implementation boundary

R140 passes 43 synthetic cases and 32 fixed cone half-space coefficient rows:

- the exact ten-category source hash set passes before graph call zero, while a
  changed cache hash and a premature graph call both fail closed;
- the three sparse `l <= A x <= u` QP schemas preserve maximum and then mean
  optima within `1e-8`; elastic state has no output or acceptance authority;
- seven trust/funnel cases cover exact-PASS priority, maximum- and mean-level
  lexicographic progress, ratio rejection, contract/hold/expand, exact-anchor
  restoration and minimum-radius research stop;
- two independent ties-to-even implementations agree on six half-integer cases;
  the continuous surrogate differs from the exact integer in all six, while
  lower/upper branch equalities record ambiguity and use zero derivative;
- 32 float64 coefficient rows are hash-bound; exact circular inside/outside/
  negative-normal cases pass, and three violated cones add at most one rational
  separation direction in graph-row/cone-type/point order;
- ranks `0/3/5`, ordered bounds and an exact `solved` residual case pass; rank
  gap, unexpected rank, non-finite singular/coefficient, unordered bound,
  cone-classification disagreement and ambiguous solver status/residual cases
  all stop invalid;
- the formulas close exactly to six major iterations, 18 QPs and 42 exact
  audits, with ten pre-graph, eight accepted-anchor, 14 output hash categories
  and 20 required counters.

The ordered source-identity/QP/trust hashes are
`f0ff3ed2a22747e40890cd45177744aa9ac9ccfa0a0ebe97080a4848a64e49bb` /
`1b735a3870ab093521e99ad161cb8171026aeb8a7f733841bca856c7ac58c263` /
`6bf8bc669712e89d352add0dc1a0f8212ca13c87133c3ab99b159b4bab85157f`.
The 32-row cone SHA-256 is
`d03a7f8afef7ddc8678903ed07d4e3a33ead40dab14979b810a0bc54b5e78943`.

## Explicit R141 roadmap decision

R140 satisfies the first frozen precondition for a real execution. This result
and the matching roadmap update supply the second: authorize exactly one clean,
bounded R141 fixed-mode kinodynamic execution under the unmodified R139 profile.
R141 must use one process/thread, seed zero, at most six major iterations, 18
OSQP QPs at 100000 iterations each, 42 exact trial audits, four hours and
`16 GiB`, with no restart or manual intervention.

R141 must reproduce all ten source hash categories before its first graph call,
start from exact R120 q0/R133 v0, preserve the fixed R131/R138 graph and exact
R133-order controller, and use the exact nonlinear/circular-cone/descriptor
oracle as sole acceptance authority. R136 cache/forces, tolerance changes,
contact changes and alternative warm starts remain forbidden.

An exact all-hard-constraint PASS may authorize only a later report-only R142
candidate-formulation decision; it does not itself publish a candidate or
permit PhysX/corpus/training. Budget/minimum-radius stop is valid complete
research evidence; identity, rank, non-finite, cone or solver ambiguity is
invalid. Either non-PASS outcome stops without restart.

Candidate publication, PhysX, all-17/V19, corpus admission, visual/exhaustive
gate and every learned optimizer or training action remain unauthorized.
