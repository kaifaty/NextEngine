# NCGP14 — Confined pressure/contact geometry and work discriminator

Status: `FROZEN_REVISION_2 / CPU_ONLY / APPARATUS_REPAIR_AUTHORIZED`

Date: `2026-09-01`

Revision 2 supersedes only the NCGP14 result/receipt apparatus frozen in
revision 1 (file SHA-256
`9d1cc04dea6a21f31eccbfc8a81a78df89eb37fe6e6373741e42f6cf0b436f43`).
The first implementation reproduced the independent numerical oracle but was
classified `APPARATUS_INCONCLUSIVE`: its v1 lane root omitted trajectory and
round-extra evidence, its alternate R8/R16 routing and transaction control
were incomplete, and its 42-field work receipt did not count actual
comparisons. Revision 2 changes no equation, fixture byte, profile, tolerance,
lane cap, physical gate or precomputed profile/fixture/lane root. It versions
only the affected roots/report and freezes one apparatus-only repair batch.

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

An apparatus-valid embedded-parent envelope owns exactly four new verification derivations:
raw stdout SHA, independently recomputed raw result root, normalized stdout
SHA and independently recomputed normalized result root. It charges the two
stdout byte blobs as two portable-content fields and charges every semantic
field appended by each result-root recomputation under the retained NCGP13
serializer rules. These four are not part of the outer identity constant
`13` and are not added again by the final envelope. Retained OPEN-128 adds no
NCGP14 round-extra or trial-observable derivation to this frozen embedded
envelope; its inherited NCGP13 trajectory root is compared and published.
On a declared embedded validation failure after capture completes, the
envelope owns only the actually executed prefix starting with the captured
raw-stdout SHA, marks every unavailable derived root invalid/zero-length and
emits typed zero-work remaining receipts; it never credits the unexecuted
suffix. If embedded invocation is skipped by earlier precedence, its raw
stdout SHA and every derived embedded root are validity-qualified absent and
the envelope is a sealed zero-work `NOT_RUN_BY_PRECEDENCE` receipt. If
invocation starts but capture itself does not complete for a reason other than
the self-read exception below, the raw stdout SHA is absent and only the
actually completed prefix is credited.
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
`nextengine.nonlocal.ncgp14.result.v2`. Exit `0` means a valid supported,
physical-refutation or solver-work classification; exit `2` means
`APPARATUS_INCONCLUSIVE`; exit `3` is an uncaught fail-closed exception and
exit `64` is usage error. Stderr is empty on every versioned JSON route.
Failure of either physical open/read/hash of `/proc/self/exe`—the outer read or
the retained embedded-parent read—is the explicit pre-report exception and
uses exit `3`: stdout is empty and no versioned JSON claim is made. This is
true even if the outer read completed before the embedded read failed. Every
other declared/handled failure after the first successful outer binary read,
including declared embedded invocation/parser failures, uses exit `2` JSON.
An unexpected uncaught top-level exception retains exit `3`.

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
The trigger must be identical on canonical and permuted R8 receipts; a route
mismatch is apparatus failure. R16 is decisive if and only if that matched
projection-cap trigger actually executed it. For every allowed zero-work R16
skip, R8 remains decisive; a skipped R16 receipt is never interpreted as a
physical or work result.
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

A fully computed successor round is one that reaches post-QP projection,
six-plane contact, candidate and independent density, pressure-balance and
inset evaluation. Only such a round derives `round-extra.v1`. A QP-cap or
earlier apparatus exit publishes only its inherited partial NCGP13 round
receipt and typed cause; it does not fabricate finite zero placeholders for
unevaluated round-extra fields. The retained OPEN-128 route produces empty
NCGP14 round-extra and trial-observable arrays: its exact inherited NCGP13
trajectory root remains the observable authority.

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
typed cause and cannot trigger R16. For every successor/control
apparatus-valid fully computed private trial, sequence observables and their
root are computed after a true `ROUND_CLOSED` private-step computation and all
`TRIAL_INVARIANTS_OK` inputs, but before any trajectory/sequence physical gate,
forced pre-commit injection or transaction disposition. The root is immutable
pre-classification evidence and contains no trajectory/sequence classification
or commit field. It therefore includes the first physically rejected trial.
The nested inherited NCGP13 attempted-step root describes only the completed
private pressure/contact step: its `accepted`, `physical_pass` and `failure`
fields are the local pre-sequence step outcome and never encode NCGP14
`trial_committed`, sequence rejection or forced disposition. Those later
outcomes are bound separately by the lane result. A trial is fully computed
only after a physical `ROUND_CLOSED` state reaches all
`TRIAL_INVARIANTS_OK` inputs and sequence metrics. QP/projection budget or
earlier apparatus exits do not derive a trial-observables root; they publish
the inherited partial round/step receipt and typed lane cause only.

Before physical trial classification, `TRIAL_INVARIANTS_OK` requires:

- complete algebraic step-balance residual `<=1e-8`;
- exact count, stable IDs and total mass;
- canonical/permuted state, velocity, pressure, contact and work exact before
  disposition.

Those failures are typed respectively
`STEP_BALANCE_CORRESPONDENCE_INVALID`, `STATE_ID_MASS_INVARIANT_INVALID` and
`PERMUTATION_CORRESPONDENCE_INVALID`; they are apparatus/invariant failures,
preserve the prior accepted transaction and cannot select H14D or trigger
R16. After `TRIAL_INVARIANTS_OK`, the physical trial gates retain the
round-closed maximum/RMS density bounds and require one connected component
with no stable-ID satellite.

After both routes execute the same physical or forced disposition, their
result-root equality is checked as apparatus correspondence. A pre-disposition
gate may not depend on the result root whose outcome it is about to select.

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

The positive-multiplier and lower-contact requirements are global witnesses
for every lane, including OPEN-512. For OPEN lanes, trial 1 may commit after
its local and cumulative numerical gates; immediately before trial-2 commit,
the cumulative OR across both fully computed trials must contain both a
positive multiplier and lower-wall contact. Their absence rejects trial 2 and
retains accepted trial 1. TIGHT requires both witnesses in each
apparatus-valid fully computed trial that reaches physical classification;
QP/projection exhaustion cannot be relabelled as a missing-witness physical
failure. `both_steps_supported` is true only when two trials commit, every
cumulative physical gate passes and the applicable witnesses are present. It
is not an alias for `committed_trial_count==2`.

For roots and gates, `positive_multiplier_count` is the sum over completed
projection rounds of `count(lambda_i>0)`. The lower-contact witness is the OR
of z-low predictor/projection first-hit or clamp-mask occurrences. Every face
count counts set-bit occurrences, not unique particles. `stable_id_count` is
the number of unique stable IDs. `satellite_count` is `dynamic_count` minus
the size of the component containing the minimum stable ID. Position and
velocity metrics use the current private trial state versus the immutable
initial state. Energy positive excess is the cumulative NCGP13 maximum from
the initial state through this trial; momentum residual uses cumulative
gravity, pressure and contact impulses through this trial.

TIGHT-128 additionally requires nonzero lateral ghost support on all four
side faces at every assembly, zero top ghost pairs and zero top contact at
every assembly/projection, and at least one pressure multiplier and bottom
contact during each apparatus-valid fully computed TIGHT trial that reaches
physical classification. Per-face lateral contact positivity and all exact
per-round pair, multiplier, contact-mask and first-hit counts are
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
   witness displacement. The separately sealed mutation witness binds every
   affected stable ID, original integer cell triple and old/new binary32
   position. Its `mutated_record_count` is exactly `100` (the frozen `10 x 10`
   `z_index==12` layer). It creates exactly `144` top pairs and changes the census root.
   The mutation is control-only and never enters a trajectory.
4. **Permutation:** before canonicalization, the canonical and permuted raw
   storage-order roots differ for each fixture. Every lane then reproduces
   stable-ID semantic, physical, work and result roots from those physically
   permuted bytes.
5. **Transactional failure:** a manufactured TIGHT first-round solve with
   `QP_cap=1` must return typed `QP_SWEEP_CAP_EXHAUSTED` independently of the
   observed CAP4096 result. A second injection
   `FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL` fires after all private trial
   observables and their evidence root are sealed, immediately before commit.
   They run as two independent subroutes from the same immutable prior, each
   through the exact production transaction object and rollback routine.
   Immediately before rollback, each derives a nonempty scratch root from that
   actual object with `owner_count=128`, `csr_entry_count>0`,
   `qp_record_count=128` and `contact_record_count=128`. Each then preserves
   every prior position/velocity value and accepted-state root, publishes its
   own rejected-trial receipt, clears the actual graph/CSR, QP multiplier/
   gradient and contact scratch, and derives its own post-reject scratch root
   with all four counts zero. One separately constructed canonical empty
   scratch is compared field-by-field to both actual post-reject workspaces;
   hashing fresh empty temporaries in place of either actual workspace is
   forbidden. Each subroute publishes its own exact zero-work trial-2 skip
   receipt with cause `NOT_RUN_PRIOR_TRIAL_REJECTED`.

   Outer control 5 owns exactly four fixed derivations: shared prior state,
   QP-rejected state, forced-rejected state and independently constructed
   canonical-empty scratch. Each transaction computational child owns its own
   pre- and post-scratch semantic-root derivations and portable fields; those
   child counts aggregate into the control but are not part of its fixed four.
   Its `control.v1` scalar records are ordered exactly as
   `(field_name="QP_CAP_1_SUBROUTE", type=bool, value=true)`,
   `(field_name="effective_qp_cap", type=u64, value=1)` and
   `(field_name="FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL", type=bool,
   value=true)`. Its evidence roots are ordered exactly: shared prior state;
   QP pre-scratch, post-scratch, rejected-state, rejected-trial-result and
   trial-2-skip; forced pre-scratch, post-scratch, rejected-state,
   rejected-trial-result and trial-2-skip; canonical-empty scratch. Equal
   zero-work skip roots remain in both distinct ordered slots. These fields
   bind each child to its policy without a transaction-policy content root, so
   the fixed derivation vector remains unchanged.
6. **Work-cap identity:** changing `4096 -> 16384` changes only the declared
   policy input, lane name/root and QP sweep ceiling. Profile, fixture,
   equations, tolerances, ordering and projection cap remain byte-exact. Any
   round, work, rejected-trial, state, trajectory and result roots may differ
   only as downstream consequences of actually executed sweeps. One counted
   sweep, `Jv` product, graph build, contact plane test and hash derivation
   mutation each changes the owning work and final roots. The same control
   evaluates a fixed route-selection truth table without invoking a solver:
   R8 `SUPPORTED`, `PHYSICAL_GATE_REJECTED` and `QP_SWEEP_CAP_EXHAUSTED`
   select R8 as decisive and require a typed zero-work R16 skip; only R8
   `PROJECTION_ROUND_CAP_EXHAUSTED` selects an actually executed R16. For that
   row, R16 `SUPPORTED`, `PHYSICAL_GATE_REJECTED`,
   `QP_SWEEP_CAP_EXHAUSTED` and `PROJECTION_ROUND_CAP_EXHAUSTED` map
   respectively to supported, physical-refuted, solver-work-ceiling and
   solver-work-ceiling classification. One side-effect-free production helper
   `select_decisive(r8_category, optional_r16_category)` is used by both real
   orchestration and this control; the expected seven-row table is enumerated
   independently and may not call that helper. Route categories are one-hot
   after apparatus closure: `SUPPORTED` means `both_steps_supported`,
   `PHYSICAL_GATE_REJECTED` means any typed physical trial/sequence cause and
   QP/projection categories mean their respective exclusive flags.
   Canonical/permuted mismatch is apparatus failure before helper invocation.
   The control publishes exactly seven row-pass booleans. Each row charges one
   production dispatch predicate and one independent expected-equality
   predicate, exactly 14 `lane_scalar_comparisons`; real orchestration charges
   one production dispatch predicate.

   The same production lane-acceptance helper is exercised by a pure witness
   truth table with all numerical gates and `committed_trial_count=2` fixed
   true. OPEN cumulative witness pairs `00/01/10` reject and `11` supports.
   Five TIGHT rows test four cases where either step lacks pressure or lower
   contact plus the all-present supported case. The nine row-pass booleans are
   scalar evidence; each charges one production acceptance predicate and one
   independent expected-equality predicate, exactly 18 comparisons. Both
   truth tables add no separate content-root derivation.
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
  evaluated, including every gate, route-table row, permutation field,
  selector input and other substantive semantic predicate. Comparison batches use an
  operation-faithful helper that evaluates the predicate first, increments
  exactly once and only then combines its boolean result; `&&`/`||`
  short-circuit expressions over uncounted predicates are forbidden;
- transaction/scratch counters increment once per scalar, ID, CSR offset/
  entry, multiplier, gradient or contact value compared;
- one receipt aggregation per child receipt added componentwise;
- one portable-content field per typed value appended to a semantic/data
  content root. The owning receipt's own work/result payload serialization,
  derivation of its own `work_root`/`result_root`, and an enclosing aggregate
  seal are unmetered seal operations, preventing recursive work.
  The single dispatch between the predeclared finalization `PASS` and
  `NOT_RUN_BY_PRECEDENCE` constructors is also an unmetered receipt-routing
  operation: it reads only the already sealed prior outcome, evaluates or
  revalidates no semantic predicate and cannot change the first failure.
  A baseline or mutated work root deliberately derived as evidence by control
  6 is not the owning receipt's self-seal: it is charged to that control;
- an apparatus-valid complete run performs exactly two physical reads of
  `/proc/self/exe`: one for outer NCGP14 identity and one in the byte-exact
  embedded NCGP13 replay. Therefore that route has `binary_file_reads=2` and
  `binary_bytes_hashed` exactly twice the executable byte length. A versioned
  exit-2 route that stops before embedded invocation has exactly one completed
  read and one executable length. Either read failure uses the unversioned
  exit-3 rule above. The embedded child owns its original binary-root
  derivation; the outer identity owns the one new executable-root derivation.
  Terminal JSON transport formatting is outside solver work; every semantic
  field it emits belongs either to a counted semantic/data-root serializer or
  to the explicitly unmetered owning receipt self-seal.

Actual work is incremented only at the operation site. Independently of those
counters, each owning NCGP14 envelope in the scope below constructs an expected
`Ncgp14WorkV1` from its root-bound trace lengths, loop extents and the fixed
ownership rules above; it may not copy any actual counter into expected after
execution. A
`same_work14` verifier evaluates all 42 fields without short-circuit but is
unmetered verifier work and does not mutate either receipt being compared. It
is required only for the NCGP14 outer identity and admission envelopes, the
embedded-parent envelope, every trial-skip receipt, every top-level successor
lane and control envelope, the finalization envelope and the final aggregate.
Its pass boolean and the actual/expected work roots are bound by that exact
envelope/result schema.
Inherited round, step and trajectory descendants retain their frozen NCGP13
20-field work verification. New NCGP14 operations performed by those children
aggregate into the owning top-level lane/control work; apart from the
canonical zero-work trial-skip receipt, no per-descendant `Ncgp14WorkV1`
expected/actual root or exact claim is published. Fully computed
round-extra and trial-observable objects are counted semantic/data roots, not
work receipts. Partial QP/early-exit descendants have no fabricated NCGP14
child-work object. The side-removal control has exactly two executed step
children and therefore exactly two `receipt_children_aggregated` additions.
Every typed value appended to graph, round-extra, trial-observable and
mutation-witness semantic content—including nested graph roots—is charged to
`portable_content_fields_serialized` at its owning serializer; receipt
self-payloads follow the unmetered rule above.

`Ncgp14WorkV1` seals the length-prefixed domain
`nextengine.nonlocal.ncgp14.work.v1` followed by the 42 little-endian `u64`
values. Final aggregation uses only the ordered top-level receipts from the
execution list below: outer identity/admission, the embedded retained-parent
envelope, each canonical/permuted successor-lane envelope (or zero-work skip)
and controls 1--8, followed by one finalization envelope. Each such envelope
already contains the componentwise sum
of its direct work and all immediate descendants; no descendant is added to
the final aggregate a second time. Every envelope asserts its own
componentwise child sum, and the final receipt asserts the componentwise sum
of those top-level envelopes. Skipped receipts contain 42 zeroes. QP
sweeps/updates and downstream pair work are data-dependent, but each is
independently bounded by its lane policy and must equal the independently
recomputed trace extents and inherited child receipts inside its one owning
envelope.

The root-bound finalization envelope owns the production decisive-route call,
all selector and primary-status predicates and no content-hash derivation. The
execution order is exact: close every preceding envelope; perform the single
unmetered predeclared receipt-constructor dispatch from the already sealed
apparatus outcome. An apparatus-valid prefix executes and counts
the route, selector and provisional-status predicates. A prior apparatus
failure executes none of those predicates and constructs zero expected/actual
finalization work for a `NOT_RUN_BY_PRECEDENCE` receipt. Then construct and
verify finalization work, aggregate and verify total work, and on any prior
apparatus, finalization-work or total-work failure set published status to
`APPARATUS_INCONCLUSIVE` and all seven published selectors false without
rerunning a predicate. Only then seal the finalization semantic result and the
top-level v2 result. A total-work mismatch overrides the zero-work NOT_RUN
outcome according to the precedence below. The finalization envelope's own
result seal, the final v2 result payload/derivation and both 42-field verifier
calls are unmetered seal/verifier operations.

Logical content-root derivations use the retained nonrecursive convention.
The fixed outer identity receipt owns `13` derivations: one executable-binary,
four profile, three fixture and five lane roots. New controls 1--8 own fixed
outer derivations `[0,2,4,6,4,10,10,12]`. The meanings are: side-removal
fixture and analytic-velocity roots; candidate/oracle/mutated geometry plus
the mutation-witness root; six raw-order roots;
prior/QP-rejected/forced-rejected/empty-scratch roots;
five baseline/mutated work-root pairs; one shared prior plus nine
rejected-state roots; and three candidate, three oracle, three zero-gamma plus
wrong-branch, wrong-sign and half-force surface roots. In particular, the ten
control-6 work-root derivations are semantic mutation evidence and are counted
even though each baseline receipt's own self-seal is unmetered.
Every fully computed successor round owns one additional `round-extra.v1`
content-root derivation, and every successor/control apparatus-valid fully
computed private trial owns one `trial-observables.v1` derivation. These data
roots are counted inside their lane's computational envelope. The v2
lane-result self-seal and the final v2 result self-seal remain unmetered.

Each top-level lane/control envelope reports the componentwise aggregate of
its executed computational descendants, while its own work/result seals are
unmetered. Parent and successor NCGP13 computational descendants retain their
published expected counts. The final value is calculated exactly once from
the same top-level-envelope list used for all 42 work fields, and every
apparatus-valid final receipt asserts:

```text
hash_derivations = identity.hash_derivations /* 13 */
                 + admission_envelope.hash_derivations /* 0 */
                 + embedded_parent_envelope.hash_derivations
                 + sum(executed successor_lane_envelope.hash_derivations)
                 + sum(control[1..8].hash_derivations)
                 + finalization_envelope.hash_derivations /* 0 */

control[i].hash_derivations =
    sum(its executed computational-child hash_derivations)
  + [0,2,4,6,4,10,10,12][i]
```

The fixed outer control contribution is therefore `48`, but it is already
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

The exact bullets below are the sole serialization authority; there are no
implicit trailing fields. NCGP14 owning receipts/envelopes explicitly named by
their schemas—identity, admission, embedded, trial-skip, control, lane and
finalization—end with the same verifier triplet in exact order: one-byte
`work_exact`, expected work root and actual work root. The 42 comparisons
producing `work_exact` are the unmetered verifier operation defined above.
`round-extra.v1` and `trial-observables.v1` are counted semantic/data roots,
not owning receipts, and have no verifier triplet. Raw-order, surface-force and
scratch contain only the fields in their exact bullets and no work root;
geometry-census and geometry-mutation contain the work root explicitly listed
in their bullets. The owning top-level envelope binds aggregate expected and
actual work for all of these data-root computations.

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
  and once to every matching face-membership field. This census root is the
  authority for the separately enumerated all-pairs oracle; the report omits
  a `graph_root` for that oracle rather than publishing an empty placeholder;
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
- `nextengine.nonlocal.ncgp14.geometry-mutation.v1`: original fixture root,
  `u64 mutated_record_count`, then records in ascending stable ID
  `(u32 ID, i32 x_index, i32 y_index, i32 z_index, old_position.x/y/z
  binary32, new_position.x/y/z binary32)`. Signed indices use little-endian
  two's-complement. Every record has `z_index==12`, unchanged `x/y`, old
  `z=0.625`, new `z=0.5` and displacement `-0.125 m`. The final fields are
  original census root, mutated census root and work root;
- `nextengine.nonlocal.ncgp14.round-extra.v1`: lane root, one-based
  `u32 trial_index`, one-based `u32 projection_round_index`, inherited NCGP13
  round root, then these one-byte apparatus flags in exact order:
  `finite_capacity_valid, assembly_valid, jv_valid, symmetry_valid,
  qp_valid, qp_converged, contact_valid, candidate_density_valid,
  independent_density_valid, density_correspondence_valid,
  pressure_balance_valid, inset_valid, round_apparatus_ok, round_closed`.
  Next are binary64
  `jv_relative_l2, symmetry_relative_l2, pressure_balance,
  density_correspondence, maximum_positive_strain, rms_positive_strain,
  penetration_m`; `u64 positive_multiplier_count`; six ordered `u64`
  dynamic-owner/ghost membership counts and six first-hit plus six clamp-mask
  occurrence counts in `x_low,x_high,y_low,y_high,z_low,z_high` order;
  `u64 maximum_owner_row_degree`;
- `nextengine.nonlocal.ncgp14.trial-observables.v1`: lane root, one-based
  `u32 trial_index`, inherited attempted-step root, state root, velocity root,
  then binary64 `position_rmse_m, position_maximum_m, velocity_rms_mps,
  maximum_speed_mps, energy_positive_excess, momentum_residual`; `u64`
  `dynamic_count, stable_id_count, component_count, satellite_count,
  positive_multiplier_count`; binary64 total mass; one-byte lower-contact
  witness; then one-byte `trial_invariants_ok`; six predictor first-hit and six
  predictor clamp-mask `u64` counts in the face order above;
  `u64 round_extra_root_count` and ordered
  round-extra roots. It is derived for
  every successor/control apparatus-valid fully computed private trial before
  physical classification, whether that trial later commits or rejects. It
  contains no `physical_gates_passed`, disposition or commit byte;
- `nextengine.nonlocal.ncgp14.trial-skip.v1`: base lane root,
  one-based `u32 trial_index`, exact length-prefixed cause and the canonical
  zero-work verifier triplet. Its receipt self-seal is unmetered. A trial-1 physical, QP
  or projection rejection maps trial 2 to
  `NOT_RUN_PRIOR_TRIAL_REJECTED`; whole-lane/apparatus precedence maps it to
  `NOT_RUN_BY_PRECEDENCE`; an attempted trial 2 instead uses the lane v2
  `EXECUTED_COMMITTED` or `EXECUTED_REJECTED` disposition and has no skip
  receipt. Both manufactured transaction-control children use the unchanged
  TIGHT-128-CAP16384 base lane root
  `8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6`.
  Their enclosing control payload separately binds either effective
  `QP_cap=1` or the exact hook name
  `FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL`; scratch and skip roots prove workspace
  lineage, not a derived mutated-policy identity. No transaction-policy root
  or content-hash derivation exists;
- `nextengine.nonlocal.ncgp14.identity-envelope.v1`: length-prefixed typed
  outcome; contract, source aggregate and binary roots; compiler family,
  version, flags and invocation; four profile, three fixture and five lane
  roots in their frozen order; then the verifier triplet;
- `nextengine.nonlocal.ncgp14.admission-envelope.v1`: length-prefixed typed
  outcome, `u64 child_count`, ordered admission child result roots, then
  the verifier triplet;
- `nextengine.nonlocal.ncgp14.embedded-envelope.v1`: length-prefixed typed
  outcome, a validity-qualified optional raw stdout SHA, then the three
  validity-qualified optional roots in this order: raw embedded result,
  normalized embedded stdout and normalized embedded result; a
  validity-qualified inherited parent-envelope result root; then the verifier
  triplet. Every optional uses `(u8 present, length-prefixed root)`, where
  `present=0` requires zero length;
- `nextengine.nonlocal.ncgp14.control.v1`: `u32 control_index`,
  length-prefixed control name and typed outcome, `u64 evidence_root_count`
  and ordered root strings, then `u64 scalar_count`. Each scalar is
  `(length-prefixed field_name, u8 type, value)`, where type `0/1/2/3` means
  respectively `bool/u32/u64/binary64`; a bool value is one byte and the
  numeric encodings are little-endian. The final fields are the verifier
  triplet;
- `nextengine.nonlocal.ncgp14.lane-result.v2`: lane root, length-prefixed typed
  route, `u32 attempted_trial_count`, `u32 committed_trial_count`, then six
  separately counted arrays in this order: inherited NCGP13 round roots,
  NCGP14 round-extra roots, attempted-step roots, rejected-step roots and
  trial-observable roots, then trial-skip roots. Next come the trajectory root, accepted state root,
  accepted velocity root, failing state root, failing velocity root and a
  length-prefixed `trial_2_execution` string. Its exact values are
  `EXECUTED_COMMITTED`, `EXECUTED_REJECTED`,
  `NOT_RUN_PRIOR_TRIAL_REJECTED` or `NOT_RUN_BY_PRECEDENCE`; an executed trial
  may never publish a NOT_RUN value. A nonexistent state/velocity/trajectory
  root is a zero-length string. The next nine one-byte selectors are exactly
  `apparatus_valid, both_steps_supported, physical_gate_rejected,
  qp_sweep_cap_exhausted, projection_round_cap_exhausted,
  trial_1_committed, trial_2_attempted, trial_2_committed,
  projection_cap_saturated`; the verifier triplet is last;
- `nextengine.nonlocal.ncgp14.finalization-envelope.v1`: length-prefixed typed
  outcome, then a length-prefixed decisive category that is zero-length on an
  invalid/not-run route; `u64 selector_count=7` and the seven published
  selector bytes in final-result order; length-prefixed published primary
  status; one-byte `total_work_exact`, expected total work root and actual
  total work root; finally the finalization verifier triplet. Its typed outcome
  is exactly one of `PASS`, `NOT_RUN_BY_PRECEDENCE`,
  `FINALIZATION_WORK_MISMATCH` or `TOTAL_WORK_MISMATCH`. A nonempty decisive
  category is exactly one of `SUPPORTED`, `PHYSICAL_GATE_REJECTED`,
  `QP_SWEEP_CAP_EXHAUSTED` or `PROJECTION_ROUND_CAP_EXHAUSTED`. On any prior
  apparatus or either work mismatch the decisive category is empty, all
  selector bytes are false and primary status is
  `APPARATUS_INCONCLUSIVE`. Outcome precedence is
  `FINALIZATION_WORK_MISMATCH`, then `TOTAL_WORK_MISMATCH`, then
  `NOT_RUN_BY_PRECEDENCE` for an earlier apparatus failure, otherwise `PASS`.
  The first-failure fields still preserve any earlier stage. The total-work
  triplet is computed before this unmetered semantic result is sealed;
  serialization order does not rerun its predicates;
- `nextengine.nonlocal.ncgp14.result.v2` starts with these exact
  length-prefixed identity strings in order: schema
`nextengine.nonlocal.ncgp14.result.v2`, NCGP14 contract root, source commit,
  source tree, source aggregate root, binary root, compiler family, compiler
  version, exact compiler flags and exact invocation; then length-prefixed
  `first_failure_stage` and `first_failure_cause`, each empty only on an
  apparatus-valid result. `first_failure_stage` is selected by frozen
  execution order and is exactly one of `IDENTITY`, `ADMISSION`,
  `EMBEDDED_PARENT`, `CONTROL_1`, `CONTROL_2`, `CONTROL_3`, `CONTROL_4`,
  `CONTROL_5`, `CONTROL_6`, `CONTROL_7`, `CONTROL_8`, `OPEN_512`,
  `TIGHT_4096`, `TIGHT_16384_R8`, `TIGHT_16384_R16`, `FINALIZATION` or
  `TOTAL_WORK`. Its matching `first_failure_cause` is respectively one of
  `IDENTITY_INVALID`, `ADMISSION_INVALID`, `EMBEDDED_PARENT_INVALID`,
  `CONTROL_INVALID`, `LANE_APPARATUS_INVALID`,
  `FINALIZATION_WORK_MISMATCH` or `TOTAL_WORK_MISMATCH`; detailed child causes
  remain in that envelope. The mapping is exact: identity, admission and
  embedded stages use their like-named invalid cause; every `CONTROL_n` uses
  `CONTROL_INVALID`; every lane stage uses `LANE_APPARATUS_INVALID`;
  finalization and total use their like-named mismatch cause. Next are one-byte
  `identity_valid, admission_valid, embedded_valid` and the identity, admission and embedded-
  envelope result roots. Each root is encoded as `(u8 present,
  length-prefixed root)`; `present=0` requires a zero-length string. Detached-parent identity
  contains only the explicitly named expected parent commit, tree,
  source-file root, source aggregate, contract root, binary root, stdout root
  and result root, followed by a validity-qualified optional SHA of captured
  raw embedded stdout bytes and three validity-qualified optional roots: raw
  embedded result, normalized embedded stdout and normalized embedded result. A missing,
  duplicate, wrong-type or wrong-length embedded field sets the relevant
  validity byte false and uses the required zero-length root; no fabricated
  64-hex placeholder is permitted. The process may not label a
  copied constant as independently observed. The identity, admission and
  embedded-envelope result roots are always present on every versioned JSON
  route: a later envelope skipped by precedence is sealed with typed
  `NOT_RUN_BY_PRECEDENCE` and zero work. The optional raw stdout SHA is present
  only when capture actually completed; an unstarted/skipped embedded replay
  uses `present=0` and zero length without hashing empty bytes. The external
  evidence report,
  outside this process result, records and binds the actual detached replay
  command and observed hashes before admitting this JSON.

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
followed by `finalization_envelope_result_root`, one-byte `total_work_exact`,
expected total work root and actual total work root.

New successor round, private-trial, rejected-trial and trajectory children use
the exact reviewed NCGP13 domains/payloads; the enclosing lane root supplies
the new policy identity. Each fully computed successor round additionally
produces its NCGP14 `round-extra.v1` root, and each apparatus-valid fully
computed trial produces its `trial-observables.v1` root. The v2 lane root
binds those arrays and the trajectory root, so no printed round/trial/
trajectory field is outside final hash closure. Inherited NCGP13 child work
and roots remain published; additional NCGP14 work is published and verified
only as the owning top-level lane aggregate.
Strings and domains are length-prefixed; vectors bind stable IDs; booleans are
one byte `0/1`; binary64 values are finite checked little-endian bits. Every
control and lane receipt, including a skip, has one unmetered result seal under
its listed domain.

The versioned JSON report publishes and the final result root binds:

- contract/source commit/tree/aggregate, binary, compiler flags and exact
  invocation;
- first-failure stage/cause, identity/admission/embedded validity flags and
  their envelope result roots; raw embedded stdout SHA and every
  validity-qualified optional derived root;
- complete per-lane profile and profile root;
- canonical dynamic/ghost bytes, geometry/fixture root, lane root and both
  legacy parent input roots where applicable;
- step/projection/QP caps and all 16 convergence/physical tolerances for every
  lane;
- for every round, attempted step and lane: raw NCGP13 graph, density,
  Jacobian, matrix, QP, independent-density, contact, topology and root/hash
  work reached by that typed stage, plus all inherited work/result roots; for
  every fully computed successor round only, the complete round-extra flags/
  metrics/face counts and its data root. QP-cap/earlier exits have no
  round-extra object or placeholder;
- separate fixture generation, face classification, ghost-pair census,
  surface-pair/candidate/oracle/reduction and lane-comparison work;
- every trial-observable data root, committed and rejected-trial state/velocity/
  observable/result roots, the inherited full trajectory receipt/work/root and
  the truthful `trial_2_execution` value;
- predictor plus per-round per-face side/bottom/top ghost-pair, first-hit and
  clamp-mask counts;
- every NCGP13 and NCGP14 control leaf, including admission outcomes, actual
  transaction scratch records, all five work mutations, geometry mutation
  records and wrong-branch/wrong-sign/half-force surface leaves, with every
  work/result root and typed outcome;
- selector flags, primary route and final result root;
- finalization raw/expected/actual work, finalization-envelope result root,
  componentwise expected/actual total-work roots and the final result root.

Logical semantic/data-root derivations are charged to the receipt that owns
their content. A receipt's own work/result seal and an enclosing aggregate seal
are unmetered; mutation derivations belong to the mutation control, never the
baseline. The report states exact expected counters and asserts componentwise
work aggregation. No validation, graph traversal, pair reduction, serializer
for a semantic/data root, or content-root derivation may be hidden behind a
boolean; only the named receipt self-seal/verifier operations and the single
predeclared finalization receipt-routing dispatch are unmetered.
Selectors are evaluated only after all preceding envelope closures. Their
predicate work is then closed in finalization, total work is aggregated and
verified, and only afterward are the finalization and result roots sealed. If
any prior apparatus, finalization-work or total-work check fails, all seven
selectors are serialized as false and cannot survive from a partially
evaluated successful prefix; predicates are not rerun.

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
receipt order. All declared embedded JSON validation failures—including
missing, duplicate, wrong-type or wrong-length fields—are caught inside the
successor orchestration and produce the versioned v2 JSON,
`APPARATUS_INCONCLUSIVE` and exit `2`; they never escape to the generic exit
`3` exception route. The sole exception is failure of either physical
`/proc/self/exe` read, which always uses exit `3` with empty stdout as frozen
above. Before embedded invocation, its typed envelope and optional roots are
sealed as zero-work/absent on precedence; after completed capture, only the
actually executed embedded prefix is credited.

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
