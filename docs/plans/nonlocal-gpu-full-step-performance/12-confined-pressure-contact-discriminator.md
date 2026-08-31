# NCGP14 — Confined pressure/contact geometry and work discriminator

Status: `FROZEN_REVISION_1 / CPU_ONLY / PRE_IMPLEMENTATION`

Date: `2026-09-01`

## Decision under test

The independently reviewed NCGP13 implementation establishes two exact facts:

- its 512-particle pressure/contact step passes density, contact, balance,
  topology and permutation gates in two projection rounds;
- its nominal 128-particle “hydrostatic” trajectory commits step 1 and rejects
  private trial 2 at velocity RMS `0.076470122842192428 m/s`.

The second fixture is not a laterally confined water column. Its dynamic
centres occupy `x,y=0.275..0.425 m` inside a `3.0 x 2.5 m` basin, so every
lateral wall is outside the `0.15 m` support horizon. Its failing value equals
the ballistic prediction for 112 falling particles and 16 bottom-clamped
particles:

```text
sqrt(112/128) * 2 * 9.81 / 240 = 0.076470122842192...
```

NCGP14 asks the smallest next question:

> Does the same reviewed explicit-pressure plus frictionless-contact step pass
> two steps when the same 128 particles actually fill a side-supported tank,
> and is any contrary result physical or merely the frozen QP work ceiling?

This is a CPU `long double` discriminator. It changes no pressure, kernel,
contact, tolerance or physical gate. It does not enable surface or viscosity,
does not call CUDA, does not measure performance and does not change the
product fallback.

## Frozen parent identity

The retained parent is the reviewed NCGP13 repair:

- commit `b1fdcd59fa105e48039bd4c1b5dda72bf7401a64`;
- tree `c541413c73fee10186c1f9652264b7b64dd473cf`;
- NCGP13 source SHA-256
  `ca62174997aaeee928efef9622c9869f7e4f3cf0909fd6f562005684d17e3873`;
- source aggregate
  `fe71ebecca1bbac5232927931a65186d145d6f2b093ee48d28f24c505ae28af7`;
- contract-chain root
  `3c26ed0b806308113384e54198935828bf61fb4a5447cc7311b78285e6b3c904`;
- Release binary
  `877c0e99f6984772cb8c74d2105a90e4f15162fd00df8d774c10c69b516662a4`;
- stdout
  `496c450fe80b5bfd3f20b042a4874e69afef8d2604286447ad35f50f6e72246e`;
- final result root
  `053a6a924a331153a72673d9a5d3809b7a0ea014799c36a3c6cba77ef088c6aa`.

The exact OPEN-512 and OPEN-128 legacy input roots remain respectively:

- `82cf83cbfc5839026c8dff77c55b81be2e5597982c31493a7016a9f5fca2a8fa`;
- `f9dbf235d5e176efe29468eba26551a1958106a5eb9d6cffbcb74ee57b4b4d1e`.

Before NCGP14 evidence is admitted, the exact parent commit is rebuilt and
run in a fresh detached worktree and must reproduce the parent binary/stdout
hashes above. That immutable replay is a verification prerequisite, not an
NCGP14-binary identity claim.

The NCGP14 executable separately reproduces the reviewed OPEN-128 physical,
work and child state/result/trajectory roots plus all 11 NCGP13 controls before
its new lanes can influence classification. That retained route executes the
exact old Phase-B checker schedule, including no centred `Jv` in its trajectory
rounds. New OPEN-512 and TIGHT-128 lanes execute the stronger round-path `Jv`
check below and therefore do not claim parent work-root identity. The final
NCGP14 report binds both the expected detached replay hashes and the exact raw
and normalized embedded replay checks below, but never substitutes successor
source/binary roots for the immutable parent values.

## Frozen hypotheses

| ID | Hypothesis | Distinguishing prediction |
| --- | --- | --- |
| H14A | NCGP13 failed because the “hydrostatic” fixture lacked lateral support | OPEN-128 and OPEN-512 remain unstable while TIGHT-128 passes at the converged work cap |
| H14B | The open 128-particle fixture is only too small to form a supported core | OPEN-128 fails but OPEN-512 passes without lateral walls |
| H14C | A confined step is viable but the old `4096`-sweep QP ceiling is too small | TIGHT-128 fails only by work exhaustion at `4096` and passes unchanged equations/tolerances at `16384` |
| H14D | The pressure/contact operator or cold initial state remains invalid even with real wall support | TIGHT-128 reaches converged QPs at `16384` but violates a frozen physical gate |
| H14E | The omitted corrected-surface term has an initial-state RMS magnitude comparable to the OPEN-128 velocity excess | `2*dt*a_rms >= max(v_rms_OPEN128_trial2-0.05 m/s,0)` in the non-integrated census |
| H14F | Per-round QPs converge, but eight pressure/contact projection rounds are insufficient | R8 has apparatus-valid rounds and converged QPs but its physical `ROUND_CLOSED` remains false; the unchanged R16 restart passes |

H14E is magnitude-only and diagnostic. It cannot overcome the separately
frozen zero-net mean-velocity bound below, establish causal repair or
authorize surface as a repair. NCGP14 does not distinguish a fully coupled
nonlinear surface trajectory. Surface may re-enter only through a later
frozen exact-`gamma` versus zero-`gamma` trajectory after a pressure/contact
baseline is supported.

## Frozen arithmetic profile

Every integrated lane starts from `nonlocal_water_corrected_profile()`:

- `dt=1/240 s`, `spacing=0.05 m`, `horizon=0.15 m`;
- `mass=0.125 kg`, `rho0=1000 kg/m^3`;
- `kernel_scale=7.985668078772472`;
- gravity `(0,0,-9.81) m/s^2`;
- retain `kappa=1226.25 J` in profile and input identity, but execute exactly
  zero penalty energy/gradient/HVP work;
- set viscosity coefficients `profile.lambda=0` and `profile.mu=0`;
- set `profile.gamma=0` for every integrated lane;
- retain `ghost_layers=3`, maximum dynamic samples `50000` and maximum
  directed neighbors `256`.

State, graph, density, Jacobian, matrix, QP, contact and observables use
`long double` after the immutable fixture values have been canonicalized once
through IEEE binary32. All ordered reductions use ascending stable sample ID.
The new translation unit may textually include the byte-exact NCGP13 source
using only `#define main ncgp13_embedded_entry` around that include. It may not
macro-override a parent identity or numerical constant or compile a second
modified copy. The embedded source is compiled with the historical
`NCGP13_*` identity macros. Therefore its structured regression may differ
from the reviewed parent JSON only in `binary_root` and the one `result_root`
that transitively binds it; every other field and child root remains exact.
The wrapper captures the raw embedded stdout bytes and requires exactly one
valid JSON object with exactly one top-level `/binary_root` and one top-level
`/result_root`, each a lowercase 64-byte hexadecimal string. It independently
recomputes the NCGP13 result-root payload from the raw fields and actual outer
binary root and requires equality with raw `/result_root`.

Normalization never parses and reserializes JSON. From the parser's recorded
source spans, it copies the raw bytes and replaces only the 64 value bytes
inside top-level `/binary_root` with the frozen parent binary root
`877c0e99f6984772cb8c74d2105a90e4f15162fd00df8d774c10c69b516662a4`
and only the 64 value bytes inside top-level `/result_root` with
`053a6a924a331153a72673d9a5d3809b7a0ea014799c36a3c6cba77ef088c6aa`.
Quotes, whitespace, key order and every other byte are unchanged. The wrapper
independently recomputes the NCGP13 result payload using the frozen parent
binary and requires the normalized result to equal the frozen result above;
it also requires `SHA256(normalized_stdout_bytes)` to equal the frozen parent
stdout `496c450fe80b5bfd3f20b042a4874e69afef8d2604286447ad35f50f6e72246e`.
Any missing/duplicate/wrong-type/wrong-length path or mismatch is immediate
`APPARATUS_INCONCLUSIVE`.

The embedded-parent envelope owns exactly four new verification derivations:
raw stdout SHA, independently recomputed raw result root, normalized stdout
SHA and independently recomputed normalized result root. It charges the two
stdout byte blobs as two portable-content fields and charges every semantic
field appended by each result-root recomputation under the retained NCGP13
serializer rules. These four are not part of the outer identity constant
`13` and are not added again by the final envelope.
The real NCGP14 commit/tree/source aggregate/binary belong only to the outer
NCGP14 report. The source aggregate binds the wrapper and exact included
parent SHA. The retained route calls the unchanged parent schedule; successor
lane orchestration owns its parameterized QP/projection caps and stronger
checks explicitly. It may not call Rust DFSPH or any CUDA/full-step evaluator.

The Release target uses strict C++17 with extensions disabled and the retained
GNU/Clang flags `-O3 -Wall -Wextra -Wpedantic -Werror -ffp-contract=off
-fno-fast-math`. A different compiler family or flag set is a different
binary profile and cannot be compared under the same root.

The only successor invocation is:

```text
nonlocal-corrected-cpu-confined-pressure-contact \
  --confined-pressure-contact-discriminator
```

It emits exactly one JSON object with schema
`nextengine.nonlocal.ncgp14.result.v1`. Exit `0` means a valid supported,
physical-refutation or solver-work classification; exit `2` means
`APPARATUS_INCONCLUSIVE`; exit `3` is an uncaught fail-closed exception and
exit `64` is usage error. Stderr is empty on every versioned JSON route.

## Frozen fixtures

All dynamic sample IDs are `1000 + 17*logical_index`, where logical order is
`x` fastest, then `y`, then `z`. Reference/current position are identical and
velocity is exactly zero. The permuted route uses the retained bijection
`(23449*input_index + 7919) mod count`, remains physically permuted through
admission and canonicalizes only after admission.

### OPEN-128 — retained negative control

- dynamic dimensions `4 x 4 x 8`;
- centres
  `(0.275+0.05*ix, 0.275+0.05*iy, 0.025+0.05*iz)`;
- basin extent `3.0 x 2.5 x 1.5 m`;
- exact legacy input root
  `f9dbf235d5e176efe29468eba26551a1958106a5eb9d6cffbcb74ee57b4b4d1e`;
- QP cap `4096` completed sweeps per projection round;
- a maximum-two-step sequential schedule, with retained trial 1 accepted,
  trial 2 attempted and rejected and step 1 retained.

This lane must reproduce NCGP13 through the first failure. It is a required
negative control and is never required to become physically passing.

### OPEN-512 — size discriminator

- dynamic dimensions `8 x 8 x 8`;
- the same origin, spacing and large-basin profile as OPEN-128;
- exact legacy input root
  `82cf83cbfc5839026c8dff77c55b81be2e5597982c31493a7016a9f5fca2a8fa`;
- QP cap `4096` completed sweeps per projection round;
- at most two sequential trials from the initial bytes; trial 2 runs only if
  trial 1 commits.

This lane distinguishes a tiny-column artefact from absent lateral support.
It is diagnostic: a valid physical failure does not block TIGHT-128.

### TIGHT-128 — side/bottom-supported primary lane

- dynamic dimensions `4 x 4 x 8` with the same IDs and vertical coordinates
  as OPEN-128;
- centres
  `(0.025+0.05*ix, 0.025+0.05*iy, 0.025+0.05*iz)`;
- basin extent `0.2 x 0.2 x 0.6 m`;
- particle-centre inset
  `[0.025,0.175] x [0.025,0.175] x [0.025,0.575] m`;
- exactly `1608` canonical three-layer ghost samples generated by the frozen
  basin constructor, giving `1736` total dynamic-plus-ghost samples;
- at the initial state the inclusive graph has exactly `6988` directed
  dynamic-owner/ghost pairs; `6480` unique pairs touch at least one lateral
  face, with overlapping face-membership counts
  `x-low/x-high/y-low/y-high = 1809/1865/1809/1865`; exactly `885` unique
  pairs touch the bottom support;
- the initial graph also has exactly `6560` directed dynamic-owner/dynamic
  pairs including self, `13548` total directed pairs and maximum owner-row
  degree `117 < 256`;
- no top-wall ghost may form an inclusive `r<=horizon` pair with a dynamic
  centre; the upper dynamic layer is at `z=0.375 m` and the nearest top ghost
  layer is at `z=0.625 m`;
- all four lateral ghost-support faces and the lower face must have at least
  one directed dynamic-owner pair before either step;
- at most two sequential trials from the same immutable initial bytes; trial
  2 runs only if trial 1 commits.

TIGHT-128 runs two primary lanes independently, in fixed order:

1. `TIGHT-128-CAP4096`: at most `4096` complete QP sweeps per projection
   round;
2. `TIGHT-128-CAP16384`: at most `16384` complete QP sweeps per projection
   round.

The second lane restarts from the original fixture; it never continues a
partial or accepted state from the first. Both use maximum eight
pressure/contact projection rounds and the same convergence/physical
tolerances. If and only if CAP16384 terminates step 1 or 2 with typed
`PROJECTION_ROUND_CAP_EXHAUSTED` after exactly eight completed projection
rounds whose QPs all converged and whose apparatus checks passed, while only
the physical `ROUND_CLOSED` predicate remains false, a predeclared third lane
`TIGHT-128-CAP16384-R16` restarts from the original fixture and changes only
the projection-round cap from `8` to `16`.
Otherwise it publishes one exact zero-work skip cause:
`NOT_RUN_R8_SUPPORTED`, `NOT_RUN_R8_PHYSICAL_GATE_REJECTED`,
`NOT_RUN_R8_QP_CAP_EXHAUSTED` or `NOT_RUN_APPARATUS_PRECEDENCE`. This prevents
the observed round-8 saturation from being misclassified as physics without
turning R16 into an unconditional retry-to-green. Any work exhaustion is a
typed result, not permission to change tolerance.

Within every lane, the first rejected trial ends that trajectory. The next
sequential trial is a typed `NOT_RUN_PRIOR_TRIAL_REJECTED` receipt with zero
work; it is never evaluated from the retained prior state merely to reach the
nominal maximum of two. A supported lane necessarily commits both steps.

The new length-prefixed roots, independently reconstructed before candidate
code, are:

| Object | Root |
| --- | --- |
| OPEN profile | `3d9b1349c5dd80e94b6ac147d25e86982ba237eed995174ab2420dfa1abad882` |
| TIGHT profile | `d44f72c90122aa7a626e855dfee7424280e9524fa3eff151ec69e3a62cfdeff8` |
| OPEN corrected-surface census profile | `bb8c13cde7f47f687c6c7fc074a19ba0fdb3e3976194551c244267c2eac514c6` |
| TIGHT corrected-surface census profile | `afaac597db6668e448af2b5302084c32572687c28608736cedcb7eb4bbfb5857` |
| OPEN-128 fixture | `e97e2d738f096abeb33918f7ac3d3b6a648cc94f4b13d68c49dd8df5fa0f4bba` |
| OPEN-512 fixture | `465986cb5bd54e921337a12f5221b86fd935fa0a63d91ba9653a2dcc977b5ffa` |
| TIGHT-128 fixture | `6dbaddf563b825e28e37aee58606f25cf50b17479cc3260e919103be79f7c2b2` |
| OPEN-128-CAP4096 lane | `9e46870992b526c2e4764dae8d2d0a37d11cc6832fd9bba74ce4507b790f7c58` |
| OPEN-512-CAP4096 lane | `a6ff5502e46f3f8bc41aa55c11169e956f875399e250b8cdf8cce1095f593393` |
| TIGHT-128-CAP4096 lane | `7bac382c7edf33ca4db698d29f95b5ce62e485f1a0378e8e983f0ef274cc875c` |
| TIGHT-128-CAP16384 lane | `8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6` |
| TIGHT-128-CAP16384-R16 lane | `0646ecf9468815f35f6968c408e1e7489d59e28ff61dfc9fe3baff9e924bfcb2` |

The profile domain is the length-prefixed string
`nextengine.nonlocal.ncgp14.profile.v1`, followed by the length-prefixed exact
profile ID `nonlocal-water-50k-v1` and the exact 19 binary64 fields in this
order: `dt, spacing, horizon, mass, rest_density,
kernel_scale, kappa, lambda, mu, gamma, gravity.x/y/z, basin.x/y/z,
ghost_layers, maximum_dynamic_samples, maximum_neighbors`; the final three
integer-valued profile fields are represented as exact binary64 as in NCGP13.
The fixture domain is the length-prefixed string
`nextengine.nonlocal.ncgp14.fixture.v1`, fixture name, profile root, `u64`
canonical sample count and records `(u32 ID, 9 binary32 values)` in exact
`reference.x/y/z, current.x/y/z, velocity.x/y/z` order, then `u64` ghost count
and records `(u32 ID, position.x/y/z binary32)`. The lane domain is the
length-prefixed string `nextengine.nonlocal.ncgp14.lane.v1`, lane name,
fixture root, `u32 maximum_steps=2`, `u32 projection_cap=8`, `u64 QP_cap`, then
these 16 ordered binary64 tolerance slots:

1. `jv_relative_l2=2e-7`;
2. `symmetry_relative_l2=2e-12`;
3. `primal=1e-8`;
4. `projected_kkt=1e-8`;
5. `complementarity_j=1e-10`;
6. `pressure_balance=1e-8`;
7. `maximum_positive_strain=1e-3`;
8. `rms_positive_strain=2.5e-4`;
9. `penetration_m=0`;
10. `step_balance=1e-8`;
11. `position_rmse_m=0.0025`;
12. `position_maximum_m=0.005`;
13. `velocity_rms_mps=0.05`;
14. `maximum_speed_mps=0.10`;
15. `energy_positive_excess=0.01`;
16. `momentum_residual=0.01`.

The last field is `u32 components=1`.
Integers are little-endian. Strings are prefixed by a little-endian `u64`
byte length. The root constructor independently reproduced both retained
legacy roots before producing this table. No result-derived root may be
inserted later.

The two census profile roots differ from their integrated profile only in
the frozen corrected binary64 `gamma`; they never enter an integrated lane.

The R16 lane uses the same serialization with lane name
`TIGHT-128-CAP16384-R16` and `u32 projection_cap=16`; every other byte is
identical to TIGHT-128-CAP16384.

## Frozen pressure/contact step

The candidate is exactly the reviewed NCGP13 composition. For accepted state
`(x,v)`, first compute and contact-project the free predictor:

```text
d = dt*v + dt^2*g
y_0 = P_box(x + d)
```

`P_box` is the frictionless componentwise endpoint projection onto the
particle-centre inset. It evaluates all six planes, preserves every tangential
endpoint component, records first-hit and complete clamp masks and returns
normal impulse `mass*(P_box(b)-b)/dt`.

For at most the lane's frozen `8` or conditional `16` rounds, independently
rebuild the inclusive current graph at admitted `y_k`, assemble corrected
density and dynamic Jacobian, and solve:

```text
c_k = rho(y_k)/rho0 - 1
A_k = (dt^2/mass) J_k J_k^T

minimize  0.5*lambda_k^T*A_k*lambda_k - c_k^T*lambda_k
subject to lambda_k >= 0

delta_p = -(dt^2/mass) J_k^T*lambda_k
y_(k+1) = P_box(y_k + delta_p)
```

Each round cold-starts `lambda_k=0` and uses deterministic cyclic projected
coordinate descent in ascending owner ID, complete-gradient update after each
changed coordinate and the lane-specific completed-sweep cap. Stop only after
a complete sweep and the exact NCGP13 primal, projected-KKT and
complementarity gates. Graph, assembly and QP state never carry between
rounds or steps.

On a physically accepted trial:

```text
x_new = y
v_new = (y-x)/dt
```

All state and observables are private until every trial gate passes. A work,
physical, admission or apparatus failure retains the complete prior accepted
`(x,v)` state/root. The rejected trial state, work, observables and typed cause
are sealed separately.

## Frozen convergence and physical gates

Every newly assembled OPEN-512 or TIGHT-128 round requires:

- centred finite-difference `Jv` relative L2 `<=2e-7` in the stable-ID
  direction retained by NCGP13;
- matrix symmetry relative error `<=2e-12`;
- primal and projected-KKT residual each `<=1e-8`;
- complementarity `<=1e-10 J`;
- pressure-correction balance `<=1e-8`;
- finite values, positive matrix diagonal and no capacity overflow.

The retained OPEN-128 control instead follows the exact reviewed NCGP13
Phase-B schedule and work receipt. It is admitted only by exact parent child-
root reproduction, not by silently adding work and comparing a changed route
to the old root.

For successor lanes, apparatus, solver and physical closure are separate.
After projection of a converged QP candidate, `ROUND_APPARATUS_OK` is the
conjunction of finite/capacity admission, valid graph/assembly, valid centred
`Jv`, symmetry correspondence, valid six-plane contact, valid candidate and
independent density evaluations and their `<=2e-12` correspondence, finite
pressure-correction balance `<=1e-8`, and an independent exact check that the
projected endpoint has componentwise inset penetration `0`. The execution
order is fixed: pre-QP apparatus checks; QP solve and its typed
validity/convergence result; projection; post-QP contact/density/balance
apparatus checks; then the physical predicate. Balance failure is typed
`PRESSURE_BALANCE_CORRESPONDENCE_INVALID`; inset failure is typed
`CONTACT_INSET_CORRESPONDENCE_INVALID`. Failure of any apparatus member is
`APPARATUS_INCONCLUSIVE` and cannot be relabelled as a physical, solver or
projection-budget result.

Only after the QP is valid and converged within its sweep cap and
`ROUND_APPARATUS_OK` is true is the physical transition predicate
`ROUND_CLOSED` evaluated:

```text
maximum_positive_strain <= 1e-3
and rms_positive_strain <= 2.5e-4
```

The first true `ROUND_CLOSED` accepts the pressure/contact projection for that
private trial. `PROJECTION_ROUND_CAP_EXHAUSTED` is possible only after eight
fully computed rounds for which every QP converged, every
`ROUND_APPARATUS_OK` was true and only the physical `ROUND_CLOSED` predicate
was false. It is the sole trigger for R16. A QP cap or any invalid
assembly/Jv/symmetry/oracle/contact/correspondence/nonfinite value has its own
typed cause and cannot trigger R16. Sequence metrics include every fully
computed private trial, including the first rejected trial. QP/projection
exhaustion seals diagnostic partial observables but those observables never
classify physics.

Before physical trial classification, `TRIAL_INVARIANTS_OK` requires:

- complete algebraic step-balance residual `<=1e-8`;
- exact count, stable IDs and total mass;
- canonical/permuted state, velocity, pressure, contact, work and result roots
  exact.

Those failures are typed respectively
`STEP_BALANCE_CORRESPONDENCE_INVALID`, `STATE_ID_MASS_INVARIANT_INVALID` and
`PERMUTATION_CORRESPONDENCE_INVALID`; they are apparatus/invariant failures,
preserve the prior accepted transaction and cannot select H14D or trigger
R16. After `TRIAL_INVARIANTS_OK`, the physical trial gates retain the
round-closed maximum/RMS density bounds and require one connected component
with no stable-ID satellite.

Topology is computed only from the inclusive dynamic-owner/dynamic-neighbor
graph. Self edges are present and counted in graph receipts but ignored by
component traversal; every non-self directed pair contributes the equivalent
undirected adjacency in ascending stable-ID order. Ghosts never connect two
dynamic components.

Across the at-most-two executed trials, using the exact NCGP13 definitions:

- position RMSE from initial `<=2.5 mm` and maximum particle displacement
  `<=5 mm`;
- velocity RMS `<=0.05 m/s` and maximum speed `<=0.10 m/s`;
- positive energy excess `<=1%`;
- normalized momentum residual `<=1%`;
- at least one positive pressure multiplier and lower-wall contact.

TIGHT-128 additionally requires nonzero lateral ghost support on all four
side faces at every assembly, zero top ghost pairs and zero top contact at
every assembly/projection, and at least one pressure multiplier and bottom
contact during each attempted step. Per-face lateral contact positivity and
all exact per-round pair, multiplier, contact-mask and first-hit counts are
diagnostic and root-bound rather than physical gates: fixed side ghosts can
supply a pressure reaction without requiring an endpoint clamp on every face.
Active-set and rounding changes must remain visible without turning one
observed count into a fitted acceptance criterion.

## Surface-force census

A separate non-integrated diagnostic evaluates the frozen corrected FCR
surface pair force on each initial lane using
`gamma=0.010664424039285813 m/(kg s^2)` and the exact piecewise spline:

```text
q = r/spacing
c(q) = q^2-1                    for 0<=q<=1
     = 1-(q-2)^2                for 1<q<3
     = 0                        for q>=3

grad_first U_pair = 2*gamma*mass^2*c(q)*normalized(x_first-x_second)
force_first = -2*gamma*mass^2*c(q)*normalized(x_first-x_second)
force_second = -force_first
```

It uses every unordered dynamic pair with `0<r<3*spacing` once, applies
equal-and-opposite forces in ascending ID-pair order and excludes ghosts. It
reports exact active pair counts `2976/17764/3216` for
OPEN-128/OPEN-512/TIGHT-128, force
root, per-particle acceleration RMS/maximum, net-force residual and the
first-order two-step velocity scale `2*dt*a_rms`. A separately written oracle
enumerates its own `i<j` pairs from positions and recomputes the same
spline/pair sum without a candidate pair list or helper. `gamma=0` must
produce the exact zero-vector root. The corrected census must close
`||F_candidate-F_oracle||_2 /
max(||F_candidate||_2,||F_oracle||_2,1e-30 N) <=2e-12` and normalized net
force `||sum_i F_i||_2 / max(sum_i ||F_i||_2,1e-30 N) <=1e-15`.

The census does no state update and cannot turn a pressure/contact failure
into PASS or establish surface causality. For OPEN-128,
`|mean(v_trial2)|=(112/128)*2*g*dt=0.07153125 m/s >0.05 m/s`. Because the
pairwise surface force has exact zero total force, its direct first-order
impulse preserves that mean and therefore still obeys
`velocity_rms>=|mean(v)|`. Surface cannot directly close the frozen velocity
gate while pressure/contact impulses remain unchanged; a later coupled test
could only examine whether surface nonlinearly changes those other impulses.
The report also publishes
`delta_mean_bound=2*dt*||sum_i F_i||_2/(N*mass)` and requires
`|mean(v_trial2)|-delta_mean_bound>0.05 m/s`.

## Mandatory controls

The 11 reviewed NCGP13 controls run first and retain their exact outcomes,
logical hash-derivation vector `[7,1,4,10,3,16,2,9,1,2,16]` and aggregate
work closure. NCGP14 adds separately sealed controls:

1. **OPEN-128 retained route:** exact NCGP13 trial-1 commit, trial-2 rejection,
   physical observables and child work/state/result/trajectory roots.
2. **Side support removal:** index tight ghosts by the integer cell triple
   used by the frozen generator. Retain every ghost with `z_index<0`, including
   bottom edge/corner ghosts. For `z_index>=0`, remove ghosts satisfying
   `x_index<0 || x_index>=4 || y_index<0 || y_index>=4`; disable only `x/y`
   contact. This leaves exactly `348` ghosts. Preserve the bottom
   support/contact, IDs, vertical coordinates, pressure equations and
   maximum-two-step schedule. On both attempts the independent positive-
   density strain is exactly zero, every pressure multiplier is exactly zero,
   only the 16 `z-low` particles contact and all `x/y` velocities remain zero.
   On attempted step 1 each bottom particle has
   `v_z=(0.5L*spacing-widen(binary32(0.025)))/dt`; on attempted step 2 each
   bottom particle has `v_z=0`. Each non-bottom particle has
   `v_z=-n*9.81*dt` for attempted step `n=1,2`, evaluated in `long double`.
   Step-1 RMS must match
   `sqrt((16*v_bottom,1^2+112*(9.81*dt)^2)/128)` and trial-2 RMS must match
   `sqrt(112/128)*2*9.81*dt`, each within `2e-12` relative. Trial 2 fails the
   velocity gate. This is a diagnostic control and cannot reuse OPEN-128
   output.
3. **Independent geometry census:** a separately written all-pairs scan
   reproduces the initial `6988/6480/1809/1865/1809/1865/885` total,
   side-unique, per-side-membership and bottom-unique counts and returns
   exactly zero dynamic/top-ghost pairs at `r<=horizon`. Moving the top ghost
   layer by changing every ghost with integer `z_index==12` from `z=0.625` to
   `0.5 m`, preserving IDs and `x/y`, by one exactly binary32 `0.125 m`
   witness displacement. It creates exactly `144` top pairs and changes the
   census root. The mutation is control-only and never enters a trajectory.
4. **Permutation:** before canonicalization, the canonical and permuted raw
   storage-order roots differ for each fixture. Every lane then reproduces
   stable-ID semantic, physical, work and result roots from those physically
   permuted bytes.
5. **Transactional failure:** a manufactured TIGHT first-round solve with
   `QP_cap=1` must return typed `QP_SWEEP_CAP_EXHAUSTED` independently of the
   observed CAP4096 result. A second injection
   `FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL` fires after all private trial
   observables and their evidence root are sealed, immediately before commit.
   Both preserve every prior position/velocity value and accepted-state root,
   publish a distinct rejected-trial receipt and leave graph, QP multiplier/
   gradient and contact scratch at the exact canonical empty-scratch root.
   The following sequential trial is
   `NOT_RUN_PRIOR_TRIAL_REJECTED` with zero work.
6. **Work-cap identity:** changing `4096 -> 16384` changes only the declared
   policy input, lane name/root and QP sweep ceiling. Profile, fixture,
   equations, tolerances, ordering and projection cap remain byte-exact. Any
   round, work, rejected-trial, state, trajectory and result roots may differ
   only as downstream consequences of actually executed sweeps. One counted
   sweep, `Jv` product, graph build, contact plane test and hash derivation
   mutation each changes the owning work and final roots.
7. **Tight profile admission:** wrong basin extent, ghost count, ghost layer,
   non-binary32 coordinate, duplicate ID, nonfinite value, dynamic-capacity
   excess and total-capacity excess are eight separate mutations and fail
   before graph/density/QP/contact work while preserving the prior root. A
   ninth mutation manufactures a row of `257` distinct IDs at one
   admitted binary32 position exceeds `maximum_neighbors=256` with exactly
   `257` accepted entries including self and returns
   `ROW_NEIGHBOR_CAPACITY_EXCEEDED` after graph admission but before density,
   QP or contact. When mutations overlap, precedence is fixed and independent
   of storage order: `INVALID_PROFILE`, `CAPACITY_EXCEEDED`, `DUPLICATE_ID`,
   `NONFINITE`, `NON_BINARY32`, then `ROW_NEIGHBOR_CAPACITY_EXCEEDED`.
8. **Surface census:** corrected candidate/oracle roots agree, zero-gamma is
   exactly zero, the wrong upper branch `q*q-1` for all `q<3`, wrong force sign
   and omitted factor two are rejected, and no census value affects integrated
   state or primary lane classification.

Every control has its own typed outcome, raw work, work root and result root.
A skipped control uses a sealed `NOT_RUN_BY_PRECEDENCE` receipt with zero work.

## Work, roots and report closure

The new CMake target is
`nonlocal-corrected-cpu-confined-pressure-contact`. Its source aggregate hashes
the following ordered `path:sha256\n` leaves:

1. `CMakeLists.txt`;
2. `src/corrected_cpu_confined_pressure_contact_main.cpp`;
3. `src/corrected_cpu_pressure_contact_main.cpp` at the exact parent SHA;
4. `src/corrected_cuda_full_step.hpp`;
5. `src/corrected_cuda_full_step_reference.cpp`;
6. `src/sha256.cpp`;
7. `src/sha256.hpp`.

The NCGP14 contract root is
`SHA256(NCGP13_CONTRACT_ROOT + ":" + SHA256(this complete file))`. Configure
fails if the included parent source SHA differs from the frozen value above.
The target is compiled only from the wrapper, reference and SHA translation
units; the textually included parent is not also passed as an independent
translation unit.

### `Ncgp14WorkV1`

Every receipt stores the following `u64` fields in this exact order. Fields
1--20 preserve the reviewed NCGP13 definitions and order:

```text
graph_builds, graph_candidates, accepted_pairs, density_pairs,
derivative_pairs, matrix_products, qp_sweeps, qp_updates,
gradient_recomputations, finite_difference_candidates,
independent_density_candidates, plane_tests, plane_hits,
contact_projections, projection_rounds, analytic_jv_multiply_adds,
topology_distance_tests, topology_discoveries, hash_derivations, penalty_work,
fixture_lattice_sites_generated, ghost_cells_tested, records_canonicalized,
profile_fields_checked, dynamic_records_admitted, ghost_records_admitted,
raw_order_records_hashed, face_classifications,
ghost_pair_distance_tests, ghost_pairs_accepted, row_degree_checks,
surface_pair_distance_tests, surface_active_pairs,
surface_force_evaluations, surface_reduction_adds,
lane_scalar_comparisons, transaction_values_compared,
scratch_values_compared, receipt_children_aggregated,
portable_content_fields_serialized, binary_file_reads, binary_bytes_hashed
```

The first 20 increment exactly as in NCGP13. The additional rules are:

- one lattice site, extended ghost-grid cell and canonicalized record per
  actual generator/canonicalization visit; repeated passes count again;
- every profile admission compares the exact length-prefixed profile ID once
  and charges that predicate to `lane_scalar_comparisons`, then evaluates all
  19 serialized fields into `profile_fields_checked` without boolean
  short-circuit; after profile/capacity admission, every dynamic/ghost record
  is scanned once and overlapping invalid flags are reduced by the frozen
  precedence above;
- one raw-order record per record appended before canonicalization;
- six face classifications per ghost in a face census;
- one ghost-pair distance test per tested dynamic/ghost pair, one accepted
  pair per inclusive hit and one row-degree check per completed owner row;
- one surface distance test per independently enumerated `i<j` pair, one
  active pair per `0<r<3*spacing`, one force evaluation per active pair in
  each candidate/oracle/control path, and one reduction add per force vector
  actually added to a particle, net, norm or RMS accumulator;
- one lane comparison per scalar, boolean or root predicate actually
  evaluated; comparison batches do not short-circuit;
- transaction/scratch counters increment once per scalar, ID, CSR offset/
  entry, multiplier, gradient or contact value compared;
- one receipt aggregation per child receipt added componentwise;
- one portable-content field per typed value appended to a content root. Only
  the owning receipt's derivation of its own `work_root`, its own `result_root`
  and an enclosing aggregate seal are excluded, preventing recursive work.
  A baseline or mutated work root deliberately derived as evidence by control
  6 is not the owning receipt's self-seal: it is charged to that control;
- exactly two physical reads of `/proc/self/exe`: one in the byte-exact
  embedded NCGP13 replay and one for the outer NCGP14 identity. Therefore
  `binary_file_reads=2` and `binary_bytes_hashed` is exactly twice the
  executable byte length. The embedded child owns its original binary-root
  derivation; the outer identity owns the one new executable-root derivation.
  Terminal JSON transport formatting is outside solver work, but all semantic
  fields it emits already belong to a counted content/root path.

`Ncgp14WorkV1` seals the length-prefixed domain
`nextengine.nonlocal.ncgp14.work.v1` followed by the 42 little-endian `u64`
values. Final aggregation uses only the ordered top-level receipts from the
execution list below: outer identity/admission, the embedded retained-parent
envelope, each canonical/permuted successor-lane envelope (or zero-work skip)
and controls 1--8. Each such envelope already contains the componentwise sum
of its direct work and all immediate descendants; no descendant is added to
the final aggregate a second time. Every envelope asserts its own
componentwise child sum, and the final receipt asserts the componentwise sum
of those top-level envelopes. Skipped receipts contain 42 zeroes. QP
sweeps/updates and downstream pair work are data-dependent, but each is
independently bounded by its lane policy and must equal the sum of the
published child rounds/trials inside its one owning envelope.

Logical content-root derivations use the retained nonrecursive convention.
The fixed outer identity receipt owns `13` derivations: one executable-binary,
four profile, three fixture and five lane roots. New controls 1--8 own fixed
outer derivations `[0,2,3,6,4,10,10,12]`. The meanings are: side-removal
fixture and analytic-velocity roots; candidate/oracle/mutated geometry roots;
six raw-order roots; prior/QP-rejected/forced-rejected/empty-scratch roots;
five baseline/mutated work-root pairs; one shared prior plus nine
rejected-state roots; and three candidate, three oracle, three zero-gamma plus
wrong-branch, wrong-sign and half-force surface roots. In particular, the ten
control-6 work-root derivations are semantic mutation evidence and are counted
even though each baseline receipt's own self-seal is unmetered.

Each top-level lane/control envelope reports the componentwise aggregate of
its executed computational descendants, while its own work/result seals are
unmetered. Parent and successor NCGP13 computational descendants retain their
published expected counts. The final value is calculated exactly once from
the same top-level-envelope list used for all 42 work fields, and every
apparatus-valid final receipt asserts:

```text
hash_derivations = identity.hash_derivations /* 13 */
                 + embedded_parent_envelope.hash_derivations
                 + sum(executed successor_lane_envelope.hash_derivations)
                 + sum(control[1..8].hash_derivations)

control[i].hash_derivations =
    sum(its executed computational-child hash_derivations)
  + [0,2,3,6,4,10,10,12][i]
```

The fixed outer control contribution is therefore `47`, but it is already
inside the eight control envelopes and is not added again. An R16 or
post-failure skip contributes zero. An apparatus failure publishes the
completed identity/child/control prefix and zero-work typed receipts for the
exact remaining order; it cannot set `expected=actual` after execution.

### New root domains

All SHA-256 root fields below are lowercase 64-byte hexadecimal ASCII strings
with their own little-endian `u64` length prefix. Every variable-length array
starts with a little-endian `u64` element count. New NCGP14 typed routes,
outcomes, skip causes and the primary status are hashed as their exact
length-prefixed ASCII JSON strings, never as implementation enum ordinals.
This rule does not alter an embedded NCGP13 child's frozen serialization.

- `nextengine.nonlocal.ncgp14.raw-order.v1`: length-prefixed fixture name and
  profile root; `u64 dynamic_count` followed by the exact dynamic fixture
  records above in physical storage order; then `u64 ghost_count` and exact
  ghost records in physical storage order;
- `nextengine.nonlocal.ncgp14.geometry-census.v1`: fixture root followed by
  this exact ordered `u64` list:
  `dynamic_count, ghost_count, dynamic_dynamic_pairs_including_self,
  dynamic_ghost_pairs, total_directed_pairs,
  lateral_unique_dynamic_ghost_pairs, x_low_dynamic_ghost_memberships,
  x_high_dynamic_ghost_memberships, y_low_dynamic_ghost_memberships,
  y_high_dynamic_ghost_memberships, z_low_dynamic_ghost_memberships,
  z_high_dynamic_ghost_memberships, maximum_owner_row_degree`; then the work
  root. A pair touching an edge/corner contributes once to the unique field
  and once to every matching face-membership field;
- `nextengine.nonlocal.ncgp14.surface-force.v1`: census-profile root, fixture
  root, `u64 active_pair_count`, `u64 force_record_count` and records
  `(u32 stable_ID, force.x/y/z binary64)` in ascending stable-ID order;
- `nextengine.nonlocal.ncgp14.scratch.v1`: lane root, `u64 owner_count` and
  owner records `(u32 owner_ID, u64 row_begin, u64 row_end)` in ascending
  owner ID; `u64 csr_entry_count` and entries
  `(u8 neighbor_kind, u32 neighbor_ID)` in canonical row order, where kind `0`
  is dynamic and `1` is ghost; `u64 qp_record_count` and records
  `(u32 owner_ID, lambda binary64, gradient binary64)`; then
  `u64 contact_record_count` and records `(u32 owner_ID, u8 first_hit_mask,
  u8 clamp_mask, correction.x/y/z binary64, accumulated_impulse.x/y/z
  binary64)`. No padding byte is serialized. The canonical empty-scratch root
  contains the lane root followed by four zero counts and no records;
- `nextengine.nonlocal.ncgp14.control.v1`: `u32 control_index`,
  length-prefixed control name and typed outcome, `u64 evidence_root_count`
  and ordered root strings, then `u64 scalar_count`. Each scalar is
  `(length-prefixed field_name, u8 type, value)`, where type `0/1/2/3` means
  respectively `bool/u32/u64/binary64`; a bool value is one byte and the
  numeric encodings are little-endian. The final field is the work root;
- `nextengine.nonlocal.ncgp14.lane-result.v1`: lane root, length-prefixed typed
  route, `u32 attempted_trial_count`, `u32 committed_trial_count`, then three
  separately counted arrays in this order: NCGP13 round roots, attempted-step
  roots and rejected-step roots. Next come accepted state root, accepted
  velocity root, failing state root, failing velocity root and lane work root;
  a nonexistent state/velocity root is a zero-length string. The final nine
  one-byte selectors are exactly
  `apparatus_valid, both_steps_supported, physical_gate_rejected,
  qp_sweep_cap_exhausted, projection_round_cap_exhausted,
  trial_1_committed, trial_2_attempted, trial_2_committed,
  projection_cap_saturated`;
- `nextengine.nonlocal.ncgp14.result.v1` starts with these exact
  length-prefixed identity strings in order: schema
  `nextengine.nonlocal.ncgp14.result.v1`, NCGP14 contract root, source commit,
  source tree, source aggregate root, binary root, compiler family, compiler
  version, exact compiler flags and exact invocation. Detached-parent identity
  then contains expected parent commit, tree, source-file root, source
  aggregate, contract root, binary root, stdout root and result root, followed
  by the independently observed detached binary/stdout/result roots and the
  raw embedded stdout SHA/result root, then the normalized embedded stdout
  SHA/result root.

The final result then binds four separately counted ordered root arrays:

1. profile roots: OPEN, TIGHT, OPEN corrected-surface, TIGHT
   corrected-surface;
2. fixture and lane-input roots: OPEN-128 fixture, OPEN-512 fixture,
   TIGHT-128 fixture, then the five lane roots in the table above;
3. lane-result roots: OPEN-128 canonical/permuted, OPEN-512
   canonical/permuted, TIGHT-CAP4096 canonical/permuted, TIGHT-CAP16384-R8
   canonical/permuted and TIGHT-CAP16384-R16 canonical/permuted (typed skips
   occupy the last two slots when R16 is not run);
4. NCGP14 control result roots 1--8 in numeric order.

After those arrays it serializes `u64 selector_count=7` and the seven one-byte
selectors in this exact order:
`invalid_hydrostatic_fixture_supported,
tiny_open_support_artifact_supported, qp_budget_limited,
projection_budget_limited, projection_cap_saturated, cause_not_unique,
surface_direct_norm_insufficient`; finally the length-prefixed primary status
and total work root.

New successor round, private-trial, rejected-trial and trajectory children use
the exact reviewed NCGP13 domains/payloads; the enclosing lane root supplies
the new policy identity. Their raw work and roots remain fully published.
Strings and domains are length-prefixed; vectors bind stable IDs; booleans are
one byte `0/1`; binary64 values are finite checked little-endian bits. Every
control and lane receipt, including a skip, has one unmetered result seal under
its listed domain.

The versioned JSON report publishes and the final result root binds:

- contract/source commit/tree/aggregate, binary, compiler flags and exact
  invocation;
- complete per-lane profile and profile root;
- canonical dynamic/ghost bytes, geometry/fixture root, lane root and both
  legacy parent input roots where applicable;
- step/projection/QP caps and every convergence/physical tolerance;
- for every round, attempted step and lane: raw NCGP13 graph, density,
  Jacobian, matrix, QP, independent-density, contact, topology and root/hash
  work, plus all work/result roots;
- separate fixture generation, face classification, ghost-pair census,
  surface-pair/candidate/oracle/reduction and lane-comparison work;
- committed and rejected-trial state/velocity/observable/work/result roots;
- per-face side/bottom/top ghost-pair and contact-mask counts;
- every NCGP13 and NCGP14 control leaf, work/result root and typed outcome;
- selector flags, primary route and final result root.

Logical semantic/data-root derivations are charged to the receipt that owns
their content. A receipt's own work/result seal and an enclosing aggregate seal
are unmetered; mutation derivations belong to the mutation control, never the
baseline. The report states exact expected counters and asserts componentwise
work aggregation. No validation, graph traversal, pair reduction, serializer
for a semantic root, or root derivation may be hidden behind a boolean.

Every new root uses a length-prefixed domain and payload. Finite `long double`
values narrow once to checked IEEE binary64 bits before hashing; raw
`long double` object bytes are forbidden. Vectors bind stable sample IDs.
Binary identity is read fail-closed from `/proc/self/exe`.

## Frozen execution and aggregation order

The external verification first performs the exact detached NCGP13 replay.
The NCGP14 process then executes and aggregates receipts in this order:

1. NCGP14 executable/contract/source identity, all profile/fixture/lane roots
   and raw admission;
2. embedded NCGP13 controls and retained OPEN-128 canonical/permuted route;
3. independent geometry census, admission/overflow and manufactured
   transaction/work-mutation controls;
4. OPEN-512 canonical, then OPEN-512 permuted;
5. TIGHT-128-CAP4096 canonical, then permuted;
6. TIGHT-128-CAP16384 R8 canonical, then permuted;
7. conditional R16 canonical/permuted, or its exact typed skip receipts;
8. side-support-removal control;
9. OPEN-128, OPEN-512 and TIGHT-128 surface censuses and their formula
   mutations;
10. permutation/selector comparisons and final aggregation.

A valid physical or solver-work rejection in one independent lane does not
stop later independent lanes. Within a lane, the first rejected trial stops
that lane transactionally. Any identity, admission, oracle, permutation,
transaction, work/root or CUDA-forbidden-path apparatus failure stops all
subsequent work and emits ordered typed zero-work receipts for the remainder.
No arrival order, exception path or boolean short-circuit may change this
receipt order.

## Classification precedence

All lanes start from independent immutable fixtures and run even when another
lane reaches a valid physical or work-budget rejection. Only an identity,
admission, oracle, algebraic balance, contact/inset, state/ID/mass invariant,
permutation, transaction, work/root or retained-parent mismatch stops the
apparatus.

1. `APPARATUS_INCONCLUSIVE` — any required identity, retained NCGP13 route,
   independent oracle, algebraic/contact/invariant, permutation, transaction,
   work or root check fails.
2. `SOLVER_WORK_CEILING_INCONCLUSIVE` — CAP16384-R8 or the triggered R16 lane
   reaches a finite valid QP with `sweeps==16384` and `converged=false`, or R16
   completes 16 apparatus-valid rounds with every QP converged while only the
   physical `ROUND_CLOSED` predicate remains false.
3. `CONFINED_PRESSURE_CONTACT_REFUTED` — the decisive CAP16384 R8/R16 lane
   converges but violates a frozen physical gate. This selects H14D only for
   the frozen cold two-step fixture; the next discriminator is static
   constrained equilibrium, not tolerance fitting.
4. `CONFINED_PRESSURE_CONTACT_SUPPORTED` — the decisive CAP16384 R8/R16 lane
   passes both steps and every control. This authorizes only a separately
   frozen 240-step TIGHT-128 pressure/contact trajectory.

The final report additionally binds non-exclusive selectors:

- `invalid_hydrostatic_fixture_supported`: TIGHT-128 passes and OPEN-128 plus
  OPEN-512 validly fail physical gates;
- `tiny_open_support_artifact_supported`: OPEN-128 fails and OPEN-512 passes;
- `qp_budget_limited`: CAP4096 ends specifically with typed
  `QP_SWEEP_CAP_EXHAUSTED`, `sweeps==4096`, finite valid residuals and
  `converged=false`, while the decisive CAP16384 lane passes;
- `projection_budget_limited`: CAP16384-R8 exhausts only the projection cap
  while the predeclared R16 lane passes;
- `projection_cap_saturated`: the decisive accepted second step first passes
  on its final allowed projection round;
- `cause_not_unique`: both a larger open lane and the confined lane pass;
- `surface_direct_norm_insufficient`: the corrected census satisfies
  `2*dt*a_rms < max(v_rms_OPEN128_trial2-0.05 m/s,0)`; this is secondary to the
  exact zero-net-force mean-velocity argument above.

These selectors are observations, not authority to modify the algorithm. A
different OPEN-512 route does not override the primary TIGHT-128 route.

## Claim ceiling and verification

Even `CONFINED_PRESSURE_CONTACT_SUPPORTED` proves only a two-step CPU
long-double pressure/contact baseline in one exact tight fixture. It is not a
surface/viscosity result, correct-water claim, 4k/16k/50k result, CUDA
correspondence result, performance measurement, runtime integration or R8
completion. CPU DFSPH remains the product fallback; SPEC-38 and ADR-076 remain
Proposed. No public Rust/engine API, renderer, PhysX, save or roadmap status
changes.

After the numerical result, run two clean Release builds/runs, an
ASan+UBSan run, the retained NCGP12/NCGP13 regressions and one independent
read-only review. One batched apparatus repair and one re-review are allowed.
A remaining load-bearing defect closes NCGP14 `INCONCLUSIVE`.

## Pre-freeze pilot disclosure

A read-only external pilot, run before this contract was drafted, motivated
the fixed work discriminator but is not evidence for any route. It observed:

- OPEN-128 trial-2 RMS equal to `0.0764701228 m/s`;
- OPEN-512 trial-2 RMS approximately `0.07399 m/s` despite active pressure;
- TIGHT-128 at cap `4096` stopped near primal `1.36e-7`, above the unchanged
  `1e-8` gate;
- TIGHT-128 at cap `16384` converged with maximum observed `8111` sweeps,
  step-1/2 velocity RMS approximately `0.01754 / 0.02285 m/s`, maximum speed
  approximately `0.06126 m/s` and maximum positive strain approximately
  `0.000788`; step 2 first met the physical gate on projection round `8`,
  exactly saturating the retained R8 cap;
- the corrected surface census implied only about `0.000319 m/s` RMS velocity
  change over two time steps, roughly 83 times below the correction needed to
  bring OPEN-128 under `0.05 m/s`.

The disclosed pilot fixes the two QP caps and the conditional R16 projection
discriminator before candidate code. It does not change equations, tolerances
or physical gates, and its values must not be copied into the candidate as
expected outputs. The frozen implementation and independent reviewer must
recompute every result from the declared bytes.
