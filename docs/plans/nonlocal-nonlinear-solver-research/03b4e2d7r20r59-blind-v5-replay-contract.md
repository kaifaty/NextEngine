# NSR3-B4E2D7R20R59 blind v5 replay contract

Status: `FROZEN / ONE-SHOT COMPOSED POLICY REPLAY AUTHORIZED`.

## Parent

- R58 implementation `fdf5ef07`, semantic `966c7264...bfad`;
- six immutable problem roots listed in R58 evidence;
- composed callbacks/caps exactly as R56; no v5 solver result exists.

## Frozen execution

For each v5 problem in committed order:

1. create fresh empty `ALR20R50Trace` and `ALR20R55SlopeTrace`;
2. install both existing callbacks without case/root selection;
3. execute `al_r20_r35_case` exactly once;
4. release hooks and retain the case and both traces.

Trace accounting accepts:

- a worked R50 entry only with exact/no-underflow frozen depth-16 ledger;
- an unworked R50 entry only as an explicitly counted dimension rejection or
  structural-cap rejection;
- at most one worked R55 inverse audit per case; later calls must cap-reject
  before inverse work;
- exact finite roots/work for every attempted hook.

Do not require 6/6 for apparatus PASS. Classify:

1. `BLIND_V5_PARENT_REJECTED`;
2. `BLIND_V5_APPARATUS_REJECTED`;
3. `BLIND_V5_POLICY_ACCOUNTING_REJECTED`;
4. `BLIND_V5_6_OF_6_CANDIDATE`;
5. `BLIND_V5_BOUNDARY_IDENTIFIED`.

Report all case/problem roots, routes, transition/principal counts, centered
calls/work/rejections and slope calls/audits/caps. Bind aggregate roots and
work into semantic output. R59 adds no source change, retry, mechanism,
tolerance/cap, timing, runtime/GPU or production authority. Regress R58/R57/R56
exactly.
