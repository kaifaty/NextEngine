# NSR3-B4E2D7R16 sparse precision transaction research

Date: `2026-08-22`

Status: `FROZEN / IMPLEMENTATION_NEXT / SHARED_HOST_PERFORMANCE_STOP`

## Question

Can the D7R15 backend safely execute one aligned nominal Dam AL substep, or
does the complete transaction still contain hidden dense work or incomplete
state coupling?

## Finding 1: accepted-trial long-double audit is still dense

D7R15 correctly removes dense binary128 work from candidate-effect sign
certification. The complete sparse inner, however, still calls
`audit_al_vector_inner_long_double` after every accepted trial. That evaluator
enumerates every fluid-fluid and fluid-support candidate four times: current
and trial, each with naive and compensated accumulation.

For the decoded nominal Dam state:

```text
dense candidates per evaluation  116,301,000
evaluations per accepted audit              4
candidate checks per acceptance   465,204,000
```

The D7R13 tiny transaction has `19` accepted trials. Repeating that acceptance
count at nominal scale would imply `8,838,876,000` long-double candidate
checks before counting the binary64 solve itself. The exact nominal acceptance
count is unknown, so this is a structural risk projection, not a runtime or
speedup claim.

D7R16 must implement a sparse long-double audit over the same canonical
current/trial `0.04h` superset union. It must preserve naive/compensated
energies, half-horizon and horizon margins, membership decisions, accepted
sign fields and complete D7R13 roots.

## Finding 2: the proof helpers are not yet the transaction backend

D7R15 proves an identity-bound AL workspace builder and a sparse binary128
oracle, but `solve_al_sparse_divided_full_private_transaction` still calls the
standalone boundary-vector builder and the dense binary128 function. A nominal
call would therefore recanonicalize static support per workspace and would not
exercise the D7R15 sparse sign oracle in its actual acceptance path.

D7R16 must thread one `JointStaticSupportBinding` through every inner, outer
and holdout workspace and route every accepted precision audit through sparse
pair unions. The complete active/inactive/repeat roots must remain exact at
legacy `dt`; explicit aligned `dt` must remain a parameter.

## Finding 3: the first predictor has a real contact impulse

The decoded Dam frame starts with `400` particles on the lower `y` contact
plane and `5,600` particles above it. At the aligned substep:

```text
dt                         5.341880341880342e-05 s
free-flight displacement  -2.799350756081525e-08 m
free-flight velocity      -5.240384615384615e-04 m/s
lower-y clamped particles  400
predictor contact impulse  0.026201923076923078 N*s
gravity impulse magnitude  0.39302884615384615 N*s
```

The existing Box-KKT predictor clamps the lower layer. An AL shadow must not
attribute the resulting momentum change to pressure or silently omit it.
D7R17 must ledger the clamped-predictor contact impulse separately, then add
the AL pressure correction and fixed-support reaction. Final box penetration
must be a first-class route; a pressure-only success cannot waive contact
feasibility.

## Decision

Do not execute the nominal solve yet. Freeze D7R16 as a transaction-backend
integration stage:

1. sparse long-double audit with exact dense correspondence;
2. identity-bound static support in every sparse AL workspace;
3. sparse binary128 audit in the real candidate-effect path;
4. complete D7R13 active/inactive/repeat root preservation;
5. zero candidate all-pair calls and exact workspace/index lifecycle;
6. read-only nominal predictor/contact accounting only.

A PASS advances the first aligned nominal substep to D7R17. It does not
authorize a macro frame, trajectory, timing lane, public state or production
path.

## Rejected alternatives

- Run the current nominal solver and tolerate the dense audit: this would
  confound solver convergence with billions of avoidable candidate checks.
- Disable long-double evidence: D7R13 inner roots and sign classification
  depend on it.
- Count predictor clamp as pressure: this breaks the momentum ledger and
  support-reaction interpretation.
- Add projected AL contacts in the same stage: contact projection is required
  only if the bounded D7R17 shadow demonstrates final penetration; pre-adding
  it would change solver mechanics before observing the boundary.

