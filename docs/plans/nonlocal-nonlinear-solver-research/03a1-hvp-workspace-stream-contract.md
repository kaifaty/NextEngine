# NSR3-A1 -- HVP workspace/stream traversal contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Candidate: `hvp-workspace-stream-v1`

## Single change

Keep the exact HVP formula and accumulation order, but:

- allocate result, density and maximum-degree neighbor-Jacobian scratch once
  per solve;
- fill/reuse those arrays for every HVP;
- merge the center into already sorted adjacency by streaming index order;
- eliminate copied/sorted participant arrays and `lower_bound` slot recovery.

Do not change pair membership, arithmetic precision, compiler flags, term
order, CG forcing, trust policy, convergence, coefficients or any non-HVP
vector allocation. The original HVP remains callable as the A/B baseline.

## Correctness gates

- candidate HVP is bit-exact to the baseline on all seven NSR2-B controls;
- final candidate state/operations/capacity are bit-exact on 512/1000/1728/
  4096;
- every NSR0--NSR3-A timing-independent result hash remains unchanged;
- all candidate repeats are exact and momentum/capacity gates remain valid.

## Same-process performance tournament

On logical CPU 4, run one warmup per implementation and seven alternating
measured baseline/candidate pairs for every NSR3-A size. Fixture construction
is outside timing. Report raw and median total/HVP buckets.

The candidate passes performance only if:

- HVP median improves by at least `1.20x` on every size;
- total median improves by at least `1.10x` on 1728 and 4096;
- candidate total median regresses by no more than 2% on 512 and 1000.

No retry, factor sweep or second bundled optimization is allowed. PASS selects
the candidate and profiles the remaining buckets. FAIL retains the baseline
and records the exact failed gate.

