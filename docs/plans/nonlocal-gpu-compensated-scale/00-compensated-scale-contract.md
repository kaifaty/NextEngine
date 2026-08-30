# NCGP3 scalable compensated-state correspondence contract

| Field | Value |
| --- | --- |
| Research ID | `NCGP3` revision 1 |
| Status | `FROZEN / CORRECTNESS_FIRST / REPORT_ONLY` |
| Frozen parent | NCGP2 `VERIFIED H1 SUPPORTED / GO`, candidate `ecb888c664404dcccab6361664259143af519022` |
| Mathematical parent | FCR0 corrected pressure, viscosity and surface objective |
| Engineering consumer | Scalable 4k CUDA state, graph, analytic boundary and transaction semantics |
| Claim class | 240-step CPU/GPU correspondence for a bounded 4k corpus; no 50k or performance claim |

## Question and claim ceiling

NCGP2 proved that the retained interior pair fails because accepted sub-ULP
updates are lost in one global binary32 position vector, and that a canonical
device `(hi, lo)` binary32 representation closes that finite fixture. NCGP2
explicitly did not establish scalable graph membership, ghost/contact
boundaries, injected transaction failure, multi-step state or cost.

NCGP3 asks whether the same physical model and representation remain correct
for dynamic CSR graphs, analytic basin contacts and 240 accepted steps with
4,000 dynamic particles. It cannot change FCR coefficients, solver rules,
tolerances, HVP ceilings or the retained NCGP1/NCGP2 results. It cannot run or
claim 50k timing. SPEC-38 and ADR-076 remain Proposed; CPU DFSPH remains the
product fallback.

## Frozen profile and state representation

The profile remains `nonlocal-water-50k-v1`:

```text
dt = 1/240 s       spacing = 0.05 m      horizon = 0.15 m
mass = 0.125 kg    ghost_layers = 3      maximum_neighbors = 256
```

Pressure, viscosity, surface, trust-region and acceptance coefficients are the
unchanged corrected-FCR/NCGP1 values. Inputs are round-tripped once through
binary32 before both CPU and GPU routes.

Every GPU reference, current, predicted, trial and transaction position is a
canonical pair of binary32 vectors `(hi, lo)` satisfying, per component:

```text
finite(hi) && finite(lo)
round_f32(hi + lo) == hi
abs(lo) <= 0.5 * ulp_toward_positive_infinity(hi)
```

Velocity has the same representation while a step is live. Prediction and
boundary-free trial updates use the NCGP2 fixed `TwoSum` order. Pair
differences and inertia consume both parts. Corrected pressure, viscosity and
surface formulas remain binary32; energy, dot products, norms, `ared`, `pred`
and `rho` remain binary64. Final public state is rounded once to binary32 only
after an accepted step.

## Compensated dynamic graph

The graph remains canonical ID-ordered CSR with at most 256 directed neighbors
per owner. It is rebuilt for every accepted current point and fixed during one
energy/gradient/HVP evaluation.

Cell membership must consume both state parts. The only permitted graph
reconstruction is:

```text
coordinate64 = double(hi) + double(lo)
coordinate_um = llrint(coordinate64 * 1_000_000)
```

Cell keys and the integer-micrometre `distance_squared <= horizon_um^2`
predicate use that value. This binary64 addition is graph-address arithmetic,
not a formula-precision promotion. Owner/candidate IDs, cell traversal, row
sort and CSR compaction remain deterministic. High-only membership, silent
neighbor truncation and floating atomics are forbidden. Ghost coordinates use
the same pair/quantization rule.

## Analytic basin and transaction semantics

The inset basin is `[spacing/2, extent-spacing/2]`. A trial uses the full
logical origin `(hi+lo)` and proposal to find the earliest swept-segment plane
hit. Simultaneous faces are accumulated in the existing mask. A contacted
component is set to the exact plane value and canonicalized; unaffected
components keep the NCGP2 EFT result. Contact impulse, face mask and boundary
work remain sealed.

Before a step, the workspace saves reference/current/predicted/velocity high
and low parts. Rejected trials restore both current parts. Any typed failure,
including an injected post-finalize failure, restores every saved high and low
part byte-for-byte. A successful final publication rounds current position and
velocity once and clears the public low parts before the next uploaded/public
state root is emitted.

## Work, memory and identity closure

The result seals profile/input/source/binary/environment identities, solver
profile, HVP budget/used, active-pressure ID signature, state and these exact
work classes:

- high/low input decomposition and canonical checks;
- high/low graph quantizations, cell probes, distance predicates and emitted
  pairs;
- formula both-part differences, inertia and EFT updates;
- boundary plane tests/hits/masks and contacted-component canonicalizations;
- high/low transaction save, trial publish, rollback, final reconstruction and
  publication;
- host/device transfers and allocated device bytes.

No allocation, full vector transfer or unsealed validation is allowed inside a
hot accepted step. Any nonfinite value, invalid pair, capacity overflow,
profile mismatch or CUDA failure is typed and fail-closed with prior-state
preservation.

## Frozen corpus and gates

Correctness is sequential and stops at the first failure:

1. retained NCGP2 nine-center sweep, `x=0.75,gamma=0`, omitted-low and
   broken-EFT controls;
2. graph boundary fixtures in which identical high parts but different low
   parts change the exact `<= horizon` membership result;
3. swept single-face, edge and corner contacts plus disabled ghost support;
4. actual injected post-finalize failure followed by exact high/low state-root
   recovery and a successful retry;
5. free fall, translation/rotation invariance, viscosity decay and surface
   relaxation;
6. 4k hydrostatic hold, dam-break and orifice trajectories for 240 steps,
   coherent and ID-permuted.

The 4k CPU oracle is independently written in binary64/long double and does
not call CUDA formula, graph, boundary, scheduler, validator or root helpers.
CPU and GPU start from one sealed binary32 byte corpus.

The frozen numerical gates are:

- position RMSE `<= 2.5 mm`, maximum `<= 5 mm` at every reported checkpoint;
- exact particle count and mass;
- density RMSE `<= 5%`, maximum `<= 10%`;
- normalized momentum residual `<= 1%`;
- positive energy excess and reversible energy drift `<= 1%`;
- center penetration `<= 2.5 mm`;
- identical active-pressure ID signature at tiny/operator gates;
- no NaN, capacity overflow, state-root drift or hidden work.

The work ceiling remains the smallest successful member of `{32,64,128}`
total HVP, selected on the complete 4k corpus without retry-to-green. If none
passes, NCGP3 is `PHYSICS_REFUTED` and later stages remain stopped.

## Mandatory negative controls

- high-only graph quantization;
- strict radius `<` instead of `<=`;
- omitted low part in formula difference;
- broken trial EFT;
- high-only boundary origin;
- disabled ghost support independently of contact;
- injected post-finalize failure with corrupted high and low parts;
- changed active-ID order, input root, work count, result root and binary
  identity;
- ID permutation loss and capacity overflow.

Each control must fail through the first relevant typed route or exact
comparator. Merely binding a control variant ID into the expected result root
is not sufficient evidence.

## Stop, review and successor rule

Performance, 16k/50k states and roadmap changes are forbidden until every
NCGP3 gate passes. Two clean Release build/runs and
`compute-sanitizer` memcheck/initcheck/synccheck are required before one
independent read-only review.

NCGP3 inherits no repair allowance from the exhausted NCGP2 package. A
remaining load-bearing defect yields `INCONCLUSIVE`; any repair requires a new
frozen successor revision. A GO authorizes only a separate 16k/50k physics and
performance package. It does not change product status or the roadmap by
itself.
