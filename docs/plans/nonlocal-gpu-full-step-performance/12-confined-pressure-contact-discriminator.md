# NCGP14 — Confined pressure/contact geometry and work discriminator

Status: `FROZEN_REVISION_5 / CPU_ONLY / APPARATUS_REPAIR_AUTHORIZED`

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

Revision 3 supersedes only two omitted root schemas, one control-lineage
identifier and one already-required permutation-comparison ownership/order in
frozen revision 2 (file SHA-256
`eec43f25e592791fed5c551a3200530c9f16f51e88814eabe556809e2f3a7b41`).
Revision 2 required a Control-2 analytic-velocity root and exact preserved-
state root equality in Controls 5 and 7, while simultaneously declaring its
root bullets the sole authority, but did not define those two payloads. It
also did not name the base lane root used by Control-2 computational data
roots or state how a final canonical/permuted lane-root comparison could stop
the suffix without resealing either lane. Revision 3 defines only those bytes,
that lineage and the existing Control-4 comparison owner/order. It changes no
equation, fixture byte, profile, tolerance, lane cap, physical gate,
precomputed profile/fixture/lane root, numerical lane schedule, work field,
fixed hash count or claim ceiling. Only the explicitly named apparatus
comparison/seal interleaving changes; no numerical lane computation is moved
or reordered.

Revision 4 supersedes only one contradictory helper-scheduling sentence in
frozen revision 3 (file SHA-256
`41c622fbd92600a622617463c94215f2fa58b64b7bd71e0377bfa733cc3898a7`).
Revision 3 required Control 6 to execute the production route helper on seven
synthetic truth-table rows in execution step 3, but also prohibited that
helper name without qualification until the later Control-4 seal. Revision 4
distinguishes those already-required side-effect-free synthetic calls from the
single real-orchestration dispatch on actual lane categories. It changes no
equation, fixture byte, profile, tolerance, lane cap, physical gate,
precomputed profile/fixture/lane root, numerical lane schedule, work field,
root schema, fixed hash count or claim ceiling.

Revision 5 supersedes only under-specified apparatus, control-evidence,
transaction and work-accounting semantics in
frozen revision 4 (file SHA-256
`ab70fa64dc4bc0aa2eb064c2009edfc95de6ac8765b52dd73384f80a78241003`).
Static review found that revision 4 required Control 7 to bind each mutated
admission input without defining canonical bytes for that evidence, and that
the surface-force root did not identify the zero-gamma/formula variant. The
same audit also found that revision 4 did not uniquely freeze early graph/
nonfinite termination, pre-commit work rollback, stored-graph topology work,
or the ownership of several already-required predicates. Revision 5 defines
only those apparatus boundaries and makes the already-required independent
expected-work construction executable. It changes no equation, integrated
fixture, physical coefficient, tolerance, lane cap, physical gate or claim
ceiling. The numerical lane order is unchanged; only fail-closed apparatus
checks, receipt sealing and transaction publication move to their explicitly
named pre-commit boundaries. Revision 5 necessarily versions the surface,
lane-result and top-level report roots and changes Control-3/7 content-root and
portable-work counts; all affected downstream roots are recomputed rather than
compared with revision-4 candidate output.

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
binary profile and cannot be compared under the same root. The target sets
`CXX_STANDARD=17`, `CXX_STANDARD_REQUIRED=YES` and `CXX_EXTENSIONS=NO` as
target properties. Configure accepts an unset `CMAKE_CXX_STANDARD` or exact
decimal `17` only and fails on every other nonempty ambient value; an ambient
minimum may not silently select C++20 or later while the report claims the
frozen C++17 profile. The effective compile command is captured from the
generated build and must contain exactly one effective language-standard
selection, the compiler-family spelling of strict C++17, with no earlier or
later conflicting `-std`/`/std` option. The translation unit also rejects at
compile time unless `__cplusplus == 201703L`; GNU extensions and C++20-or-later
profiles cannot self-report as the frozen profile.

The only successor invocation is:

```text
nonlocal-corrected-cpu-confined-pressure-contact \
  --confined-pressure-contact-discriminator
```

It emits exactly one JSON object with schema
`nextengine.nonlocal.ncgp14.result.v3`. Exit `0` means a valid supported,
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

Within every lane, the first rejected trial ends that trajectory. If trial 1
ends by an exact physical, QP or projection-budget result, the next sequential
trial is a typed `NOT_RUN_PRIOR_TRIAL_REJECTED` receipt with zero work. If
trial 1 or its lane prefix ends by apparatus, invariant, permutation or work
precedence, trial 2 is instead `NOT_RUN_BY_PRECEDENCE`. Neither skip evaluates
the retained prior state merely to reach the nominal maximum of two. A
supported lane necessarily commits both steps.

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

The predictor contact is the first successor computational child and remains
before every round graph build. It executes all six plane tests for every
admitted dynamic record without short-circuit. Its actual work and the
independently reconstructed expected prefix contain the exact inherited
`plane_tests`, observed `plane_hits` and `contact_projections`, followed by one
counted `predictor_valid` predicate. If that predicate is false, the typed cause
is `PREDICTOR_CONTACT_INVALID`: no predictor-contact semantic root, graph or
later child is derived. If it is true, the inherited predictor-contact root is
derived with its one inherited hash derivation and is immediately frozen as the
trial-level reached child `TRIAL_PREDICTOR_CONTACT`; only then are the counted
lower-contact-present and top-contact-zero witness predicates evaluated. The
expected predictor prefix is recomputed from the immutable admitted state and
profile; it may not copy the candidate masks, hits or work counters.

Every successor graph consumer is then fail-closed at the first unavailable
stage. An owner row that reaches the `257`th inclusive entry returns
`ROW_NEIGHBOR_CAPACITY_EXCEEDED` immediately. It derives no graph semantic
root, density, Jacobian, matrix, Jv, QP, contact, topology, round-extra or
trial-observable root. Its actual and independently expected work contain the
complete already reached valid predictor prefix above, the records/ghost faces
admitted before graph construction, one `graph_builds`, candidates and
inclusive hits actually visited through the failing entry, one row-capacity
lane predicate at the failing hit, and one `row_degree_check` plus one
successful row-capacity predicate for each earlier fully admitted owner row;
the overflowing row itself is not credited as a completed row. Apart from the
already frozen predictor-contact root, all downstream portable fields and hash
derivations are zero. A lane-result array represents
every unavailable child root by absence through its counted array length,
never by hashing a partial CSR or a finite zero placeholder.

The exact declared early apparatus causes are
`PREDICTOR_CONTACT_INVALID`, `ROW_NEIGHBOR_CAPACITY_EXCEEDED`,
`NONFINITE_ASSEMBLY`, `NONFINITE_QP`, `NONFINITE_CONTACT`,
`NONFINITE_DENSITY` and `NONFINITE_OBSERVABLE`. Before a
finite-only inherited NCGP13 serializer begins, the complete payload required
by that serializer is preflighted. If the preflight fails, that child root is
absent and only previously completed finite child roots plus the reached
top-level lane work prefix are sealed. A finite QP-cap or projection-cap route
may retain the inherited finite partial NCGP13 receipt allowed by its frozen
schema; a nonfinite or capacity route may not fabricate such a receipt when
its required payload is unavailable. These declared causes are handled inside
successor orchestration, roll back the private transaction and produce typed
versioned exit-2 evidence. Only the two executable self-read failures and an
unexpected logic/system exception use the exit-3 route.

The cause boundary and root-presence schedule is exact:

| First unavailable boundary | Cause | Newly present finite roots |
| --- | --- | --- |
| free predictor or predictor six-plane contact | `PREDICTOR_CONTACT_INVALID` | none; the invalid predictor-contact payload is not hashed |
| inclusive graph row overflow after a valid predictor | `ROW_NEIGHBOR_CAPACITY_EXCEEDED` | trial-level `TRIAL_PREDICTOR_CONTACT`; none for the failing round |
| pre-QP density, Jacobian, matrix, diagonal, Jv or symmetry payload | `NONFINITE_ASSEMBLY` | `ROUND_INPUT_STATE` only |
| solve multiplier, gradient or residual payload | `NONFINITE_QP` | input, graph, density, Jacobian and matrix roots as reached-child records |
| projected endpoint or six-plane contact payload | `NONFINITE_CONTACT` | the preceding assembly roots plus multiplier root |
| post-projection candidate or independent density payload | `NONFINITE_DENSITY` | the preceding roots plus contact and post-graph roots; neither post-density root is derived |
| sequence metric or trial-observable payload | `NONFINITE_OBSERVABLE` | complete inherited round and attempted-step roots; no trial-observables root |

The predictor-contact root is the only successor child permitted before graph
capacity succeeds. The input-state root is delayed until graph capacity
succeeds; all assembly
content roots except that input root are derived only after the complete
pre-QP payload is finite. The ordered reached-child array in `lane-result.v3`
binds every table root not already transitive through an inherited round/step
receipt. A finite QP-cap inherited round receipt and a fully computed
balance/inset failure follow their own finite schemas instead of this table.

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
earlier apparatus exit publishes its typed cause and only an inherited partial
NCGP13 round receipt whose exact finite payload is actually available under
the rule above; it does not fabricate a child root or finite zero placeholders
for unavailable fields. The retained OPEN-128 route produces empty
NCGP14 round-extra and trial-observable arrays: its exact inherited NCGP13
trajectory root remains the observable authority.

Only after the QP is valid and converged within its sweep cap and
`ROUND_APPARATUS_OK` is true is the physical transition predicate
`ROUND_CLOSED` evaluated:

```text
maximum_positive_strain <= 1e-3
and rms_positive_strain <= 2.5e-4
```

The implementation first derives and seals every reached apparatus flag and
the aggregate `ROUND_APPARATUS_OK`. If it is false, the round stops with its
first typed apparatus cause. Neither density-strain predicate nor the
positive-multiplier witness predicate is evaluated or credited on that path.
Only the true-apparatus branch evaluates those two strain predicates and the
round's positive-multiplier witness, all without short-circuit. The
independently constructed expected work follows the same reached-stage split;
it may not budget a successful physical suffix merely because a round-extra
object exists.

The first true `ROUND_CLOSED` accepts the pressure/contact projection for that
private trial. `PROJECTION_ROUND_CAP_EXHAUSTED` is possible only after eight
fully computed rounds for which every QP converged, every
`ROUND_APPARATUS_OK` was true and only the physical `ROUND_CLOSED` predicate
was false. It is the sole trigger for R16. A QP cap or any invalid
assembly/Jv/symmetry/oracle/contact/correspondence/nonfinite value has its own
typed cause and cannot trigger R16.

For every successor/control apparatus-valid fully computed private trial, the
step is computed and sealed in this exact two-phase order. After a true
`ROUND_CLOSED`, each route derives its finite state, velocity, pressure and
contact semantic roots and freezes its reached work snapshot without yet
sealing the inherited attempted-step result. Those existing roots and the
42-field snapshots are the inputs to canonical/permuted
`TRIAL_INVARIANTS_OK` correspondence; no provisional attempted-step result
root exists. Only after that correspondence succeeds may the stored-graph
topology and local physical trial predicates run. Each route then seals its
inherited attempted-step root exactly once, with its final local
`accepted/physical_pass/failure` fields, and the two final step roots are
compared before sequence gates or disposition. A route that fails the
pre-seal correspondence derives no attempted-step root and cannot be resealed
under a later cause.

Sequence observables and their root are computed after that local step-root
equality, but before any trajectory/sequence physical gate, forced pre-commit
injection or transaction disposition. The root is immutable pre-sequence
evidence and contains no trajectory/sequence classification or commit field.
It therefore includes the first physically rejected sequence trial. The
nested inherited NCGP13 attempted-step root describes only the completed
private pressure/contact step: its `accepted`, `physical_pass` and `failure`
fields are the local pre-sequence step outcome and never encode NCGP14
`trial_committed`, sequence rejection or forced disposition. Those later
outcomes are bound separately by the lane result. A trial is fully computed
only after a physical `ROUND_CLOSED` state reaches all
`TRIAL_INVARIANTS_OK` inputs, stored-graph topology and sequence metrics.
QP/projection budget or earlier apparatus exits do not derive a
trial-observables root; they publish only actually available finite inherited
children and the typed lane cause.

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

After both routes execute the same physical or forced disposition, each lane
envelope owns three non-short canonical/permuted predicates in this order:
`route_category`, `failure_cause`, then `trial_committed`. Both envelopes
evaluate the same three predicates, six comparisons per pair. There is no
second per-trial post-disposition result root. The only result-root comparisons
are the final local attempted-step equality before sequence disposition and
the later Control-4 equality of the two fully sealed lane-result roots. A
pre-disposition gate may not depend on a result root whose outcome it is about
to select.

Each canonical/permuted successor pair is one apparatus transaction rooted at
the immutable lane-initial accepted state. A successfully checked private
trial may advance a private accepted state so trial 2 can be computed, but no
accepted state, committed count or trajectory becomes authoritative in the
lane result until the reached-prefix work verifier and all correspondence for
that trial pass. The final lane-envelope 42-field verifier also runs before
authoritative publication. Any apparatus, invariant, permutation or work
mismatch inside that pair through its final lane-envelope verifier—including a
mismatch first discovered by that verifier—publishes the immutable
lane-initial state/velocity, a zero-step trajectory,
`committed_trial_count=0` and both commit selectors false. It still binds every
actually computed private/rejected child and its reached work as evidence. An
actually attempted trial 2 remains `EXECUTED_REJECTED`; an unstarted trial 2
is `NOT_RUN_BY_PRECEDENCE`. In contrast, an exact physical, QP or projection
rejection may publish the valid prior private commit(s), so a trial-2 physical
rejection retains trial 1. A later Control-4 comparison failure does not mutate
or reseal either already-final lane result; it invalidates only the enclosing
apparatus/final claim. `lane-result.v3` only serializes an already settled
transaction; its self-seal may not discover or overwrite the first work
failure.

Topology is computed only from the final accepted round's already stored
post-projection candidate-density CSR at `y_(k+1)`, identified by that round's
`post_graph_root`. The pre-correction assembly graph at `y_k`, an independently
rebuilt accepted-state graph and the independent-density oracle's private
enumeration are forbidden topology authorities. No position, distance or
radius predicate is reevaluated. The traversal inspects every stored directed
dynamic edge of that exact CSR once in ascending owner/neighbor stable-ID
order, including self, and increments
`topology_distance_tests` once per inspected stored edge; in successor
receipts that legacy field therefore means a stored edge-membership
inspection, not a second distance calculation. Self edges are ignored for
union, every non-self directed edge contributes the equivalent undirected
union attempt, and ghosts never connect two dynamic components. After all
edges are inspected, component labels are assigned in ascending stable-ID
order and `topology_discoveries` increments exactly once per dynamic ID. Thus a
complete topology pass has `topology_discoveries=N`; no visited-pruned scan,
second graph build or uncounted adjacency reconstruction is permitted.

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

Every surface data root references the frozen OPEN or TIGHT corrected-surface
census-profile root and carries one exact length-prefixed variant string.
Candidate and independent oracle both use `CORRECT`, so their roots must be
identical. `ZERO_GAMMA` means that only `gamma` is overridden to exact
binary64 zero before evaluation; `WRONG_BRANCH`, `WRONG_SIGN` and
`HALF_FORCE` mean exactly the mutations named below. The referenced census
profile itself is never mutated or relabelled, and an integrated profile root
is forbidden in every surface root. This variant field is the authority for a
control override without deriving an extra profile root.

It uses every unordered dynamic pair with `0<r<3*spacing` once, applies
equal-and-opposite forces in ascending ID-pair order and excludes ghosts. It
reports exact active pair counts `2976/17764/3216` for
OPEN-128/OPEN-512/TIGHT-128, force
root, per-particle acceleration RMS/maximum, net-force residual and the
first-order two-step velocity scale `2*dt*a_rms`. A separately written oracle
enumerates its own `i<j` pairs from positions and recomputes the same
spline/pair sum without a candidate pair list or helper. `gamma=0` must
produce the exact `ZERO_GAMMA` zero-vector root. The corrected census must close
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
   output. Every NCGP14 round-extra and trial-observable data root produced by
   this control uses the frozen TIGHT-128-CAP16384 base lane root
   `8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6`.
   The modified side-removal fixture root remains separate and is the first
   Control-2 evidence root; the analytic-velocity root below also binds it.
   Inherited NCGP13 child roots retain their own frozen domains.
   On the PASS path, the Control-2 `control.v1` evidence-root array is exactly
   `[side_removal_fixture, analytic_velocity, attempted_step_1,
   attempted_step_2, trial_observables_1, trial_observables_2]` in that order.
   Each trial-observables root transitively binds its ordered round-extra
   roots; direct duplicate round-extra entries are forbidden. This array makes
   every printed Control-2 child root transitively part of `result.v3` without
   adding a content-root derivation. Its scalar array is exact:
   `(ghost_count,u64,348)`, followed by binary64
   `step1_velocity_rms, step1_analytic_velocity_rms, step2_velocity_rms,
   step2_analytic_velocity_rms` in that order.
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
   permuted bytes. Control 4 is one ordered, prefix-aware top-level receipt.
   Its work accumulator begins before the six raw-order roots are serialized
   and derived, although its receipt is sealed only after its final reached
   prefix is complete. After those six roots exist, it evaluates exactly three
   non-short raw-root inequality predicates in `OPEN128`, `OPEN512`, `TIGHT128`
   order immediately before the first lane-result equality. As soon as each
   canonical/permuted lane pair has been sealed exactly once, Control 4
   evaluates and counts one final result-root equality predicate; neither lane
   may then be modified or resealed. A true intermediate predicate only
   authorizes the next frozen execution prefix and cannot set final status
   before Control 4 itself is sealed.

   The TIGHT-CAP16384-R8 result-root equality must pass before Control 4
   evaluates and counts exactly one non-short `r16_triggered` predicate. That
   predicate is true exactly when the matched R8 category is
   `PROJECTION_ROUND_CAP_EXHAUSTED`; its boolean is the next ordered Control-4
   scalar after the R8 equality. A true value authorizes the frozen R16 pair;
   a false value authorizes its two exact typed lane skips. R16's executed
   roots or its two sealed skip roots are then compared by the fifth lane-root
   equality before Control 4 closes. Only the real-orchestration invocation of
   `select_decisive` on the actual R8/R16 lane categories, owned by
   finalization, is forbidden until the final Control-4 receipt has been
   sealed. The synthetic Control-6 truth-table invocations below are the sole
   exception and cannot observe or affect actual lane or Control-4 state.

   On the PASS path, the `control.v1` evidence-root array is exactly the six
   raw-order roots in fixture/canonical-permuted order
   `[OPEN128_c, OPEN128_p, OPEN512_c, OPEN512_p, TIGHT128_c, TIGHT128_p]`,
   followed by five canonical/permuted lane-result pairs in this order:
   `OPEN128`, `OPEN512`, `TIGHT_CAP4096`, `TIGHT_CAP16384_R8`,
   `TIGHT_CAP16384_R16`. The last pair contains the two exact skip roots when
   R16 is not run. The exact Control-4 scalar order is
   `open128_raw_different, open512_raw_different, tight128_raw_different,
   open128_result_exact, open512_result_exact, tight4096_result_exact,
   tight_r8_result_exact, r16_triggered, tight_r16_result_exact`. A full PASS
   therefore owns exactly nine non-short comparisons: three raw-root
   inequalities, five final lane-result equalities and the one R16-trigger
   predicate. All nine scalar records have `type=bool`; on a prefix route the
   scalar array contains exactly the reached prefix in this declared order.
   PASS requires all three `*_raw_different` and all five `*_result_exact`
   validation booleans to be true, but does not require
   `r16_triggered=true`. Both trigger values are valid: true executes R16 and
   false creates its two typed skips before the final equality. Control 4's
   fixed six content-root derivations remain only the six raw-order roots; all
   lane/skip roots are existing evidence, not rederived by the control.

   Control 4 has this exact prefix state machine:

   - if no Control-4 serializer, derivation or predicate has started, an
     earlier apparatus failure produces the existing zero-work
     `NOT_RUN_BY_PRECEDENCE` receipt;
   - if any of the three `*_raw_different` validation predicates or any of the
     five `*_result_exact` validation predicates is false, all remaining lane
     slots first receive their canonical typed zero-work skip receipts, then
     Control 4 seals `CONTROL_INVALID`; Control 4 is the first failure. The
     `r16_triggered` selector is excluded from this failure rule;
   - if an unrelated embedded, control or lane apparatus failure occurs after
     any Control-4 work, no further Control-4 predicate runs. All remaining
     lane slots first receive their canonical typed zero-work skip receipts,
     then Control 4 seals `PREFIX_CLOSED_BY_PRECEDENCE` with the nonzero
     independently expected and actual prefix work. The earlier failure stage
     and cause remain the first failure;
   - Control 4 has a specialized verifier seal and may never emit a generic
     `FAIL` outcome. Its own expected/actual prefix-work mismatch with no prior
     failure seals `CONTROL_INVALID` and makes Control 4 the first failure. If
     an unrelated earlier failure already selected the prefix-close branch,
     the verifier mismatch remains visible in its triplet but the outcome stays
     `PREFIX_CLOSED_BY_PRECEDENCE` and preserves that earlier first failure;
   - only a route reaching all nine predicates with exact work seals Control 4
     `PASS`.

   Every started Control-4 route binds one full sixteen-slot evidence array:
   the six raw roots followed by the ten lane-result or lane-skip roots in the
   PASS order above. A partially executed pair uses every already sealed actual
   lane root followed by the counterpart skip if that counterpart did not
   start; every future pair uses two skips. All referenced skip receipts are
   sealed before the Control-4 result root. They remain zero-work top-level
   lane owners; Control 4 only references their existing roots. Prefix expected
   work is derived from the frozen reached-stage schedule, never copied from
   actual work.
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
   every prior position/velocity value and accepted-state root, then clears the
   actual graph/CSR, QP multiplier/gradient and contact scratch. After rollback,
   it derives its actual post-rejection accepted-state root and its own
   post-reject scratch root with all four counts zero. One separately constructed canonical empty
   scratch is compared field-by-field to both actual post-reject workspaces;
   hashing fresh empty temporaries in place of either actual workspace is
   forbidden. Each subroute next seals its own exact zero-work trial-2 skip
   receipt with cause `NOT_RUN_PRIOR_TRIAL_REJECTED`, verifies its complete
   child work, and only then seals its control-only
   `transaction-rejection.v1` receipt. After both child receipts exist, outer
   Control 5 performs its declared state/scratch/hook comparisons, verifies its
   aggregate work and seals exactly once. Placeholders and resealing at any of
   these boundaries are forbidden.

   The forced subroute arms an explicit production transaction hook before the
   private trial. Only that same production disposition routine may set the
   observed `hook_fired` byte, exactly after the observable root and all
   pre-disposition gates are sealed and immediately before rollback. Control 5
   evaluates and owns exactly one non-short `lane_scalar_comparisons`
   predicate `hook_fired==true`; a false value is `CONTROL_INVALID`. The JSON
   scalar and control payload use this observed byte, never a literal `true`.
   This adds no content-root derivation and does not change the forced child's
   existing pre-disposition computational schedule.

   Outer control 5 owns exactly four fixed derivations: shared prior state,
   QP-rejected state, forced-rejected state and independently constructed
   canonical-empty scratch. Each transaction computational child owns its own
   pre- and post-scratch semantic-root derivations and portable fields; those
   child counts aggregate into the control but are not part of its fixed four.
   Its `control.v1` scalar records are ordered exactly as
   `(field_name="QP_CAP_1_SUBROUTE", type=bool, value=true)`,
   `(field_name="effective_qp_cap", type=u64, value=1)` and
   `(field_name="FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL", type=bool,
   value=observed_hook_fired)`, which must be true on PASS. Its evidence roots
   are ordered exactly: shared prior state;
   QP pre-scratch, post-scratch, rejected-state, rejected-trial-result and
   trial-2-skip; forced pre-scratch, post-scratch, rejected-state,
   rejected-trial-result and trial-2-skip; canonical-empty scratch. Equal
   zero-work skip roots remain in both distinct ordered slots. These fields
   bind each child to its policy without a transaction-policy content root, so
   the fixed derivation vector remains unchanged.

   In both `rejected-trial-result` slots, the referenced object is the
   control-only `transaction-rejection.v1` receipt defined below, never a
   `lane-result.v3`. The QP child has subroute `QP_CAP_1_SUBROUTE` and outcome
   `QP_SWEEP_CAP_EXHAUSTED`; the forced child has subroute
   `FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL` and outcome
   `FORCED_ROLLBACK_OBSERVED`. Both serialize attempted count `1`, committed
   count `0`, the same preserved post-rejection accepted-state root and their
   own pre/post scratch plus trial-2-skip roots. The QP receipt binds exactly
   two ordered children: the already-derived predictor-contact root followed
   by its reached finite partial-round result root. It has no attempted-step,
   trial-observables, private-state or private-velocity child. The forced
   receipt binds, in order, the inherited attempted-step, trial-observables,
   private-state and private-velocity roots and has `hook_fired=true`. These
   receipts are admissible only as Control-5 evidence; they never occupy a
   production lane slot, enter `select_decisive` or extend the
   `lane-result.v3` route/cause vocabulary.

   The shared prior, QP-rejected and forced-rejected state roots all use the
   single `state.v1` schema below with the same TIGHT-128-CAP16384 base lane
   root. Exact state preservation therefore requires all three root strings to
   be equal in addition to the field-by-field comparisons. Exactly two
   non-short-circuit `lane_scalar_comparisons` evaluate
   `qp_rejected_state_root == prior_state_root` and
   `forced_rejected_state_root == prior_state_root`. Each rejected root is
   serialized from that subroute's actual authoritative post-rejection
   accepted-state object after rollback, never by rehashing the saved prior
   object as a substitute.
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
   For the real-orchestration invocation, any canonical/permuted mismatch is
   apparatus failure before that invocation.
   The control publishes exactly seven row-pass booleans. Each row charges one
   production dispatch predicate and one independent expected-equality
   predicate, exactly 14 `lane_scalar_comparisons`; real orchestration charges
   one production dispatch predicate. All seven production-helper calls in
   this truth table execute and Control 6 seals in frozen execution step 3.
   They consume only the independently enumerated synthetic categories and
   cannot inspect any actual lane category, mutate Control-4 state or select
   the actual R8/R16 execution path.

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
   QP or contact. It derives no graph root and credits only the exact prefix
   defined above. When mutations overlap, precedence is fixed and independent
   of storage order: `INVALID_PROFILE`, `CAPACITY_EXCEEDED`, `DUPLICATE_ID`,
   `NONFINITE`, `NON_BINARY32`, then `ROW_NEIGHBOR_CAPACITY_EXCEEDED`.

   Control admission carries two immutable policy scalars inside every
   `admission-mutation.v1` input: `maximum_total_records=100000` and
   `expected_ghost_count`. Mutations 1--7 use
   `expected_ghost_count=1608`; mutation 8 uses `100000`; mutation 9 uses `0`.
   The production admission path compares the actual ghost count against that
   rooted policy value, not against an implicit TIGHT fixture constant. Thus
   mutation 2 alone reaches the ghost-count `INVALID_PROFILE` result, mutation
   8 reaches total-record `CAPACITY_EXCEEDED` at `1+100000=100001`, and
   mutation 9 is admitted through record checks before its row overflow. This
   control-only expected-shape policy does not change any integrated fixture or
   precomputed profile/lane root; the same production validators and frozen
   precedence still execute.

   Each mutation is an exact deterministic transform of the frozen
   TIGHT-128-CAP16384 profile/fixture and has this one-based identity and
   expected outcome:

   1. `WRONG_BASIN_X`: replace only `basin_extent.x` by binary64 bits
      `0x3fd0000000000000` (`0.25`); `INVALID_PROFILE`;
   2. `MISSING_LAST_GHOST`: require original ghost count `1608` and remove
      only canonical ghost index `1607`; `INVALID_PROFILE`;
   3. `WRONG_GHOST_LAYERS`: replace only `ghost_layers` by `u32 2`;
      `INVALID_PROFILE`;
   4. `NON_BINARY32_CURRENT_X`: replace only canonical dynamic index `0`
      `current.x` by binary64 bits `0x3fb999999999999a` (`0.1`);
      `NON_BINARY32`;
   5. `DUPLICATE_LAST_ID`: replace only canonical dynamic index `127` ID by
      the ID at index `0`; `DUPLICATE_ID`;
   6. `NONFINITE_CURRENT_Z`: replace only canonical dynamic index `0`
      `current.z` by positive-infinity binary64 bits
      `0x7ff0000000000000`; `NONFINITE`;
   7. `DYNAMIC_CAPACITY_50001`: preserve the original `128` dynamic records,
      then append copies of canonical dynamic record `0` until the exact count
      is `50001`; ghosts remain unchanged; `CAPACITY_EXCEEDED`;
   8. `TOTAL_CAPACITY_100001`: retain only canonical dynamic record `0` and
      replace the ghost vector by exactly `100000` records with
      `ID=1000000+i`, `i=0..99999`, and position bits
      `(0x0000000000000000,0x0000000000000000,
      0xbf9999999999999a)` (`0,0,-0.025`); `CAPACITY_EXCEEDED`;
   9. `ROW_CAPACITY_257`: replace dynamics by exactly `257` records with
      `ID=1000+17*i`, `i=0..256`, identical reference/current position bits
      `(0x3fb3333340000000,0x3fb3333340000000,
      0x3fb3333340000000)`, zero velocity bits and no ghosts;
      `ROW_NEIGHBOR_CAPACITY_EXCEEDED`.

   Before any production admission call, Control 7 freezes in memory the exact
   base-lane root, mutation index/name and mutation-specific transform payload
   below. That immutable prefix is the sole input authority used by the
   independent expected-work/input reconstruction; it cannot inspect the later
   observed outcome or rejected-state root. After the production route returns,
   one `admission-mutation.v1` data root serializes those frozen input fields
   together with the observations in the authoritative order below. This
   remains one counted derivation per mutation and does not introduce a
   separate input root.

   The shared prior and all nine rejected-state roots use the same `state.v1`
   schema and TIGHT-128-CAP16384 base lane root; every rejected-state root must
   equal the shared prior root. Each mutation derives one
   `admission-mutation.v1` data root from the exact compact transform payload
   below, observed outcome and actual post-rejection state root. Mutation
   identity is not encoded by changing the preserved-state root domain.
   Exactly nine non-short-circuit
   `lane_scalar_comparisons` evaluate one
   `rejected_state_root[i] == shared_prior_state_root` predicate per mutation.
   Nine more non-short-circuit comparisons evaluate the observed outcome
   against the exact expected outcome above. The Control-7 evidence-root array
   is exactly `[shared_prior_state_root, mutation_1_root,
   rejected_state_1_root, ..., mutation_9_root, rejected_state_9_root]`.
   Its scalar array is exactly the eighteen bool records
   `mutation_1_outcome_exact, mutation_1_state_preserved, ...,
   mutation_9_outcome_exact, mutation_9_state_preserved` in that order.
   Every rejected root is serialized from the actual authoritative
   post-rejection accepted-state object returned by that production admission
   control, never by rehashing the shared prior object as a substitute.
8. **Surface census:** corrected candidate/oracle roots agree, zero-gamma is
   exactly zero, the wrong upper branch `q*q-1` for all `q<3`, wrong force sign
   and omitted factor two are rejected, and no census value affects integrated
   state or primary lane classification. `WRONG_BRANCH`, `WRONG_SIGN` and
   `HALF_FORCE` each use only the OPEN-128 fixture, the OPEN corrected-surface
   census-profile root and its respective variant string. The OPEN-512 and
   TIGHT fixtures execute only `CORRECT` candidate/oracle and `ZERO_GAMMA`
   variants.

The eight `control.v1` names are exactly, in index order,
`open128-retained-route`, `side-support-removal`,
`independent-geometry-census`, `canonical-permutation`,
`transactional-failure`, `work-cap-route-witness`,
`tight-profile-admission`, `surface-force-census`. Except for Control 4's
prefix outcomes frozen above, a started control outcome is exactly `PASS` or
`CONTROL_INVALID`; an unstarted control is `NOT_RUN_BY_PRECEDENCE`. The literal
outcome `FAIL` is forbidden.

The payloads not already enumerated above are exact:

- Control 1 evidence roots are `[embedded_parent_envelope,
  open128_canonical_lane, open128_permuted_lane, trial1_attempted_step,
  trial2_rejected_step, retained_trajectory]`. Its bool scalars are
  `parent_envelope_exact, trial1_committed_exact, trial2_rejected_exact,
  trial1_state_root_exact, trial2_state_root_exact, trajectory_root_exact,
  work_root_exact, canonical_permuted_exact` in that order;
- Control 3 evidence roots are `[candidate_graph, candidate_census,
  oracle_census, mutated_census, geometry_mutation]`; every census and mutation
  root transitively contains its own counted work root. The production graph
  root is an already-derived child reference and adds no Control-3 content-root
  derivation. Candidate and oracle census roots are expected to differ because
  their operation-faithful work roots differ; full census-root equality is
  neither evaluated nor published. In schema order, every one of the 13 census
  fields evaluates two non-short predicates: candidate equals oracle, then
  oracle equals the corresponding frozen constant in the Independent Geometry
  Census control above. The published `*_exact` scalar for that field is the
  conjunction of those two already evaluated booleans and adds no hidden third
  predicate. `mutation_record_count_exact` owns one predicate. The next scalar,
  `mutation_records_exact`, is the conjunction of exactly `100*10=1000`
  non-short field predicates in ascending stable-ID and record-schema order:
  ID; x/y/z integer indices; old x/y/z; new x/y/z. It adds no aggregate
  predicate. `mutated_top_pairs_exact` and `mutation_changes_census_root` each
  own one predicate. Thus Control 3 owns exactly
  `13*2 + 1 + 1000 + 1 + 1 = 1029` semantic comparisons. Its bool scalar array
  remains one `*_exact` scalar for each of the 13 geometry-census `u64` fields,
  followed by `mutation_record_count_exact, mutation_records_exact,
  mutated_top_pairs_exact, mutation_changes_census_root`;
- Control 6 evidence roots are five baseline/mutated `work.v1` pairs in this
  mutation order: `graph_builds, qp_sweeps, plane_tests,
  analytic_jv_multiply_adds, hash_derivations`. Its bool scalars are the five
  corresponding `*_root_changed` fields; the seven route rows
  `r8_supported, r8_physical, r8_qp_cap, r16_supported, r16_physical,
  r16_qp_cap, r16_projection_cap`; then the nine witness rows
  `open_00, open_01, open_10, open_11, tight_step1_no_pressure,
  tight_step1_no_contact, tight_step2_no_pressure, tight_step2_no_contact,
  tight_all_present`. No truth-table evidence root exists;
- Control 8 evidence roots are `[open128_correct, open128_oracle,
  open128_zero_gamma, open512_correct, open512_oracle,
  open512_zero_gamma, tight128_correct, tight128_oracle,
  tight128_zero_gamma, open128_wrong_branch, open128_wrong_sign,
  open128_half_force]`. For `open128, open512, tight128` in that order it emits
  bool scalars `candidate_pair_count_exact, oracle_pair_count_exact,
  zero_pair_count_exact, candidate_oracle_root_exact, relative_l2_pass,
  net_residual_pass, zero_force_exact`; then
  `open128_wrong_branch_rejected, open128_wrong_sign_rejected,
  open128_half_force_rejected, open128_mean_bound_pass`; finally binary64
  `open128_two_step_velocity_scale` and `open128_delta_mean_bound`.

Every control has its own typed outcome, raw work, work root and result root.
A skipped control uses a sealed `NOT_RUN_BY_PRECEDENCE` receipt with zero work.
An ordinary started control evaluates its complete declared non-short schedule
even after one expected-equality boolean is false, then seals
`CONTROL_INVALID`. If a child itself ends by apparatus before its promised
evidence exists, the control stops and serializes exactly the reached prefix of
the declared evidence/scalar arrays; it never invents a placeholder or copies
an expected suffix. Control 4 alone uses its specialized interleaved prefix
state machine.

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
translation unit. An evidence build requires the wrapper, CMake file, included
parent and contract to be tracked at one clean Git commit; an untracked or
relevant dirty source is non-admissible. Configure resolves `HEAD`, its
symbolic-ref file when present, the index and every source/contract leaf above
as configure dependencies, so a later commit or staged/source change forces
identity regeneration before build. The embedded commit/tree/source and flag
strings are generated only after these checks; they may not be stale literals
from a prior configure.

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
- every profile admission compares the exact length-prefixed profile ID once,
  then for each of 19 serialized fields evaluates one binary64-finite and one
  finite-after-binary32-narrowing predicate, followed—when an expected profile
  is supplied—by 19 field-equality predicates; it evaluates one rooted
  expected-ghost-count predicate when supplied, then exactly 15 physical/
  policy predicates in the frozen field order, one aggregate-invalid-profile
  predicate, and `empty`, dynamic-capacity and total-capacity predicates in
  that order. All 78 predicates on the ordinary expected-profile/ghost-count
  path increment `lane_scalar_comparisons` without short-circuit, and exactly
  19 field visits increment `profile_fields_checked`;
- after profile/capacity admission, every dynamic record owns exactly seven
  ordered predicates: duplicate ID, then finite and binary32-exact for
  reference, current and velocity. The first ghost owns duplicate ID, finite
  and binary32-exact; each later ghost additionally owns the ascending-ID
  predicate between duplicate-ID and finite. After all records, the three
  aggregate predicates are duplicate, nonfinite and nonbinary in precedence
  order. Every reached record increments its admitted-record counter once;
- a row-capacity scan increments `graph_builds` once on builder entry. Pair
  membership is represented by `graph_candidates`/`accepted_pairs` and is not
  additionally a lane-scalar predicate. Each fully admitted owner row owns one
  `row_degree_checks` and one non-short `degree<=maximum_neighbors` lane
  predicate. The failing owner instead stops at the first `257`th inclusive
  hit, owns no completed-row check, and evaluates exactly one
  `degree>maximum_neighbors` lane predicate. Mutation 9 therefore has
  `graph_builds=1`, `graph_candidates=257`, `accepted_pairs=257`,
  `row_degree_checks=0`, one overflow lane predicate and zero graph-root hash/
  portable fields;
- one raw-order record per record appended before canonicalization;
- six face classifications per ghost in a face census;
- one ghost-pair distance test per tested dynamic/ghost pair, one accepted
  pair per inclusive hit and one row-degree check per completed owner row;
- successor topology reuses the sealed inclusive graph: one
  `topology_distance_tests` increment per stored directed dynamic edge
  inspected, including self, and one `topology_discoveries` increment per
  dynamic ID assigned a component label. It performs no additional
  `graph_builds`, `graph_candidates`, radius test or graph-root derivation;
- one surface distance test per independently enumerated `i<j` pair, one
  active pair per `0<r<3*spacing`, one force evaluation per active pair in
  each candidate/oracle/control path, and one reduction add per force vector
  actually added to a particle, net, norm or RMS accumulator;
  the three candidate/oracle relative-L2 norm reductions over
  OPEN-128/OPEN-512/TIGHT-128 therefore own exactly
  `3*(128+512+128)=2304` additional `surface_reduction_adds`;
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
- the new revision-3 data-root serializers have exact portable-field counts.
  At `N=128`, each `state.v1` root owns `3 + 10*N = 1283` fields, so the three
  Control-5 state roots own `3849` and the ten Control-7 state roots own
  `12830`. The two-step `side-removal-velocity.v1` root owns
  `3 + 2*(1 + 4*128 + 2) = 1033` fields. These schema fields change expected
  portable work only. Revision 5 leaves Control-2/5 at `2/4` fixed content-root
  derivations and changes Control 7 to the explicitly listed `19`;
- Control 3's three `geometry-census.v1` roots own `3*16=48` portable fields,
  its 100-record `geometry-mutation.v1` root owns
  `6 + 10*100 = 1006`, and the four explicitly embedded `work.v1` roots own
  `4*43=172`. Its fixed data-root serialization is therefore `1226` portable
  fields and eight derivations. These embedded work roots are mutation/control
  evidence, not owning-receipt self-seals;
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
execution. Work returned by a child is always added componentwise to the
already accumulated owner work; assigning a child receipt over earlier
generation, canonicalization or validation work is forbidden. Admission
expected graph candidates are derived from input sizes
and the independently frozen loop extents; expected accepted pairs and row
degrees are recomputed by a separately written inclusive-radius enumeration
over the root-bound baseline or the immutable mutation-specific prefix that is
later sealed by `admission-mutation.v1`. Reading
production `row_degrees`, accepted-hit booleans or accumulated counters to
construct expected admission work is forbidden. Embedded-parent expected
validation work is derived only from the captured byte length and the fixed
schedule below; it may not read a production
`validation_predicates_completed` counter. Let `B` be the exact captured stdout
byte count. The production and independently written expected validator both
execute this non-short schedule:

1. a total strict-JSON byte-step recognizer consumes all `B` bytes, including
   bytes after the first error by remaining in an explicit rejecting sink
   state. Every consumed byte owns exactly one `lane_scalar_comparisons`
   predicate, followed by exactly one EOF/complete-single-object predicate.
   Thus lexical/grammar work is always exactly `B+1`; no parser-dependent first
   rejection offset exists. Here strict JSON is UTF-8 RFC-8259 syntax with no
   BOM, no non-JSON `NaN`/`Infinity`, no unescaped control characters, valid
   escape and surrogate-pair structure, exactly one root value and only JSON
   whitespace after it; the required root-object shape is checked in step 2;
2. if and only if that final grammar predicate is true, exactly `135` further
   predicates run: one top-level-object predicate, then for `/binary_root` and
   `/result_root` in that order one exact top-level occurrence-count predicate,
   one string-type predicate, one byte-length-64 predicate and 64 lowercase-hex
   predicates. Missing or wrong-typed values use a fixed 64-byte `0xff` sentinel
   for the character predicates, so all 135 execute without invalid access or
   short-circuit;
3. if and only if all 135 predicates pass, one predicate first requires the raw
   `/binary_root` value to equal the actual outer executable root from the
   already sealed identity envelope. A false result stops the embedded suffix.
   On true, the two source spans are normalized byte-for-byte as frozen above,
   the normalized stdout SHA is derived and one predicate compares it with the
   frozen parent stdout SHA. A false comparison stops the embedded suffix;
4. normalized-stdout equality proves every byte other than the two substituted
   64-byte value spans is the reviewed parent JSON. Only on that true branch,
   the independent retained NCGP13 result-payload serializer visits and appends
   every payload field twice in its frozen payload order—once with the observed
   outer binary and once with the frozen parent binary. Those typed appends own
   the already declared `portable_content_fields_serialized` work; no separate
   per-JSON-path validation predicate exists. It derives the raw and normalized
   result roots, then evaluates exactly two predicates in order: raw root equals
   observed `/result_root`, normalized root equals the frozen parent result.

The raw captured-stdout SHA precedes this schedule and remains the first of the
four embedded verification derivations. The normalized stdout SHA and two
result-root derivations are reached only as specified above. This executable
does not claim a separate semantic walk over every nested JSON member: exact
normalized stdout bytes close all 11 retained controls and nested evidence,
while the independent payload serializer checks the parent result-root
semantics. JSON token dispatch, stack maintenance, decoded-tree construction
and source-span movement are the implementation of the one-per-byte recognizer
unit, not additional semantic predicates. The complete successful validator
therefore owns exactly `B+1+135+1+1+2 = B+140` lane comparisons. A malformed
document therefore has
the unambiguous full `B+1` grammar prefix and no later work; a syntactically
valid mismatch has the exact fixed suffix reached above. A
`same_work14` verifier evaluates all 42 fields without short-circuit but is
unmetered verifier work and does not mutate either receipt being compared. It
is required only for the NCGP14 outer identity and admission envelopes, the
embedded-parent envelope, every transaction-rejection and trial-skip receipt,
every top-level successor lane and control envelope, the finalization envelope
and the final aggregate.
Its pass boolean and the actual/expected work roots are bound by that exact
envelope/result schema.
Inherited round, step and trajectory descendants retain their frozen NCGP13
20-field work verification. New NCGP14 operations performed by those children
aggregate into the owning top-level lane/control work; apart from the
transaction-rejection receipts and the canonical zero-work trial-skip receipt,
no other per-descendant `Ncgp14WorkV1`
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
top-level v3 result. A total-work mismatch overrides the zero-work NOT_RUN
outcome according to the precedence below. The finalization envelope's own
result seal, the final v3 result payload/derivation and both 42-field verifier
calls are unmetered seal/verifier operations.

Logical content-root derivations use the retained nonrecursive convention.
The fixed outer identity receipt owns `13` derivations: one executable-binary,
four profile, three fixture and five lane roots. New controls 1--8 own fixed
outer derivations `[0,2,8,6,4,10,19,12]`. The meanings are: side-removal
fixture and analytic-velocity roots; candidate/oracle/mutated geometry plus
the mutation-witness root and the four work roots embedded by those data
roots; six raw-order roots;
prior/QP-rejected/forced-rejected/empty-scratch roots;
five baseline/mutated work-root pairs; one shared prior, nine
rejected-state roots and nine admission-mutation roots; and three candidate,
three oracle, three zero-gamma plus
wrong-branch, wrong-sign and half-force surface roots. In particular, the ten
control-6 work-root derivations are semantic mutation evidence and are counted
even though each baseline receipt's own self-seal is unmetered.
The nine admission-mutation roots serialize exactly
`[11,11,11,12,11,12,12,16,19]` portable fields, total `115`; the ten state roots
remain `10 * (3 + 10*128) = 12830` portable fields. Control 7 therefore owns
`19` fixed content-root derivations and `12945` fixed portable fields in
addition to its executed admission children.
Every fully computed successor round owns one additional `round-extra.v1`
content-root derivation, and every successor/control apparatus-valid fully
computed private trial owns one `trial-observables.v1` derivation. These data
roots are counted inside their lane's computational envelope. The v3
lane-result self-seal and the final v3 result self-seal remain unmetered.

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
  + [0,2,8,6,4,10,19,12][i]
```

The fixed outer control contribution is therefore `61`, but it is already
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
their schemas—identity, admission, embedded, transaction-rejection,
trial-skip, control, lane and finalization—end with the same verifier triplet
in exact order: one-byte
`work_exact`, expected work root and actual work root. The 42 comparisons
producing `work_exact` are the unmetered verifier operation defined above.
`round-extra.v1` and `trial-observables.v1` are counted semantic/data roots,
not owning receipts, and have no verifier triplet. Raw-order, surface-force,
scratch, state and side-removal-velocity contain only the fields in their exact
bullets and no work root; geometry-census and geometry-mutation contain the
work root explicitly listed in their bullets. The owning top-level envelope
binds aggregate expected and actual work for all of these data-root
computations.

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
  a `graph_root` for that oracle rather than publishing an empty placeholder.
  The production candidate census uses the same semantic field order but binds
  its own distinct operation-faithful work root, so candidate/oracle census-root
  equality is neither expected nor a control predicate; Control 3 separately
  publishes the production candidate graph root and compares all 13 semantic
  `u64` census fields;
- `nextengine.nonlocal.ncgp14.surface-force.v2`: census-profile root,
  length-prefixed variant string, fixture root, `u64 active_pair_count`,
  `u64 force_record_count` and records
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
- `nextengine.nonlocal.ncgp14.state.v1`: the frozen TIGHT-128-CAP16384 base
  lane root, `u64 dynamic_count`, then records in ascending stable-ID order:
  `(u32 ID, reference.x/y/z binary64, position.x/y/z binary64,
  velocity.x/y/z binary64)`. This schema is used only for the preserved-state
  evidence in Controls 5 and 7. Equal state values under the same base lane
  root must produce byte-identical roots; subroute or mutation names are not
  serialized into this state root;
- `nextengine.nonlocal.ncgp14.transaction-rejection.v1`: the frozen
  TIGHT-128-CAP16384 base lane root; exact length-prefixed subroute and outcome
  strings; `u32 attempted_trial_count=1`, `u32 committed_trial_count=0`; then
  `u64 child_root_count` and ordered child roots; post-rejection accepted-state
  root, pre-reject scratch root, post-reject scratch root and trial-2-skip root;
  one-byte `hook_fired`; finally the verifier triplet for that computational
  child. For `QP_CAP_1_SUBROUTE/QP_SWEEP_CAP_EXHAUSTED`, `child_root_count=2`
  and the children are the predictor-contact root and reached finite
  partial-round result root in that order;
  `hook_fired=false`. For
  `FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL/FORCED_ROLLBACK_OBSERVED`,
  `child_root_count=4` and the children are inherited attempted-step,
  trial-observables, private-state and private-velocity roots in that order;
  `hook_fired=true`. Both receipts bind the actual post-rollback accepted-state
  object and actual scratch objects, use the same exact zero-work trial-2 skip
  cause `NOT_RUN_PRIOR_TRIAL_REJECTED`, and have no production-lane selectors.
  Their receipt self-seals and 42-field verifier calls are unmetered; all child
  computation and semantic/data-root work remains owned once by Control 5;
- `nextengine.nonlocal.ncgp14.admission-mutation.v1`: the frozen
  TIGHT-128-CAP16384 base lane root, one-based `u32 mutation_index`, exact
  length-prefixed mutation name, `u64 maximum_total_records=100000`, `u64
  expected_ghost_count`, exact expected and observed outcome strings, then the
  actual post-rejection `state.v1` root followed by the mutation-specific
  payload below. All binary64 values in this pre-admission
  payload are serialized as their exact little-endian `u64` IEEE-754 bits;
  this is the sole exception that permits the declared positive-infinity
  control value and it never enters an admitted state.

  The mutation-specific payloads are exact and contain no implicit fields:

  1. length-prefixed field name `basin_extent.x`, then its `u64` bits;
  2. `u64 original_ghost_count=1608`, `u64 removed_index=1607`;
  3. length-prefixed field name `ghost_layers`, `u32 value=2`;
  4. `u64 dynamic_index=0`, length-prefixed field name `current.x`, then its
     `u64` bits;
  5. `u64 destination_dynamic_index=127`, `u64 source_dynamic_index=0`;
  6. `u64 dynamic_index=0`, length-prefixed field name `current.z`, then its
     `u64` bits;
  7. `u64 original_dynamic_count=128`, `u64 resulting_dynamic_count=50001`,
     `u64 append_source_dynamic_index=0`;
  8. `u64 retained_dynamic_index=0`, `u64 generated_ghost_count=100000`,
     `u32 ghost_id_start=1000000`, `u32 ghost_id_stride=1`, then three `u64`
     position-bit fields in xyz order;
  9. `u64 dynamic_count=257`, `u32 id_start=1000`, `u32 id_stride=17`, three
     `u64` reference/current position-bit fields (the same triplet is applied
     to both vectors), three `u64` zero-velocity fields and
     `u64 ghost_count=0`.

  The base lane/fixture bytes plus this transform are the complete mutated
  input identity. The root is a counted semantic/data derivation owned by
  Control 7, has no verifier triplet and must be published with all payload
  fields so an independent reader can reconstruct it;
- `nextengine.nonlocal.ncgp14.side-removal-velocity.v1`: the frozen
  TIGHT-128-CAP16384 base lane root, the Control-2 side-removal fixture root,
  then exactly two attempted-step slots in one-based order. Each slot is
  `u64 velocity_count=128`, followed by records
  `(u32 stable_ID, velocity.x/y/z binary64)` in ascending stable-ID order,
  then binary64 actual velocity RMS and binary64 analytic expected velocity
  RMS. It is a pure data root with no implicit work root; the Control-2
  envelope binds its derivation/serialization work and all per-particle
  analytic equality predicates;
- `nextengine.nonlocal.ncgp14.geometry-mutation.v1`: original fixture root,
  `u64 mutated_record_count`, then records in ascending stable ID
  `(u32 ID, i32 x_index, i32 y_index, i32 z_index, old_position.x/y/z
  binary32, new_position.x/y/z binary32)`. Signed indices use little-endian
  two's-complement. Every record has `z_index==12`, unchanged `x/y`, old
  `z=0.625`, new `z=0.5` and displacement `-0.125 m`. The final fields are
  original production candidate census root, mutated production candidate
  census root and work root;
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
  `NOT_RUN_BY_PRECEDENCE`; an attempted trial 2 instead uses the lane v3
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
- `nextengine.nonlocal.ncgp14.lane-result.v3`: lane root, length-prefixed
  `route_category`, length-prefixed `failure_cause`, `u32
  attempted_trial_count`, `u32 committed_trial_count`, then six
  separately counted arrays in this order: inherited NCGP13 round roots,
  NCGP14 round-extra roots, attempted-step roots, rejected-step roots and
  trial-observable roots, then trial-skip roots. Next is `u64
  reached_child_count` and only the finite child roots that were derived but
  are not transitively bound by one of those six arrays. Each record is
  `(u32 trial_index, u32 round_index, length-prefixed stage,
  length-prefixed root)`. Trial indices are one-based; round index is
  one-based for a round child and zero for a trial-level child. Records sort by
  trial, round and this exact stage order:
  `TRIAL_PREDICTOR_CONTACT, ROUND_INPUT_STATE, ROUND_GRAPH, ROUND_DENSITY,
  ROUND_JACOBIAN, ROUND_MATRIX,
  ROUND_MULTIPLIER, ROUND_CONTACT, ROUND_POST_GRAPH, ROUND_POST_DENSITY,
  ROUND_INDEPENDENT_DENSITY, ROUND_STATE, TRIAL_STATE, TRIAL_VELOCITY,
  TRIAL_MULTIPLIER, TRIAL_CONTACT_MASK, TRIAL_CONTACT_IMPULSE`. A root already
  bound by an inherited round/step result is forbidden from this array; an
  unavailable or nonfinite root has no record. These records are fields of the
  unmetered lane receipt self-payload and introduce no new content-root
  derivation. Next come the trajectory root, accepted state root,
  accepted velocity root, failing state root, failing velocity root and a
  length-prefixed `trial_2_execution` string. Its exact values are
  `EXECUTED_COMMITTED`, `EXECUTED_REJECTED`,
  `NOT_RUN_PRIOR_TRIAL_REJECTED` or `NOT_RUN_BY_PRECEDENCE`; an executed trial
  may never publish a NOT_RUN value. A nonexistent state/velocity/trajectory
  root is a zero-length string. The next nine one-byte selectors are exactly
  `apparatus_valid, both_steps_supported, physical_gate_rejected,
  qp_sweep_cap_exhausted, projection_round_cap_exhausted,
  trial_1_committed, trial_2_attempted, trial_2_committed,
  projection_cap_saturated`; the verifier triplet is last.

  `route_category` is exactly one of `SUPPORTED`, `PHYSICAL_GATE_REJECTED`,
  `QP_SWEEP_CAP_EXHAUSTED`, `PROJECTION_ROUND_CAP_EXHAUSTED`,
  `APPARATUS_INVALID`, `NOT_RUN_R8_SUPPORTED`,
  `NOT_RUN_R8_PHYSICAL_GATE_REJECTED`, `NOT_RUN_R8_QP_CAP_EXHAUSTED` or
  `NOT_RUN_APPARATUS_PRECEDENCE`. `failure_cause` is empty only for
  `SUPPORTED`; equals the category for QP/projection and all four NOT_RUN
  categories; is `PHYSICAL_TRIAL_GATE_REJECTED` or
  `PHYSICAL_SEQUENCE_GATE_REJECTED` for physical rejection; and for apparatus
  is exactly one of `PREDICTOR_CONTACT_INVALID,
  ROW_NEIGHBOR_CAPACITY_EXCEEDED, NONFINITE_ASSEMBLY,
  GRAPH_ASSEMBLY_INVALID, JACOBIAN_CORRESPONDENCE_INVALID,
  MATRIX_SYMMETRY_CORRESPONDENCE_INVALID, QP_CORRESPONDENCE_INVALID,
  NONFINITE_QP, NONFINITE_CONTACT, CONTACT_CORRESPONDENCE_INVALID,
  NONFINITE_DENSITY, DENSITY_CORRESPONDENCE_INVALID,
  PRESSURE_BALANCE_CORRESPONDENCE_INVALID,
  CONTACT_INSET_CORRESPONDENCE_INVALID,
  STEP_BALANCE_CORRESPONDENCE_INVALID, STATE_ID_MASS_INVARIANT_INVALID,
  TIGHT_SUPPORT_CORRESPONDENCE_INVALID, TOP_CONTACT_CORRESPONDENCE_INVALID,
  PERMUTATION_CORRESPONDENCE_INVALID, NONFINITE_OBSERVABLE,
  TRAJECTORY_HASH_ACCOUNTING_INVALID, LANE_CATEGORY_INVALID,
  LANE_WORK_MISMATCH`. Any other string is invalid. A mismatch first discovered
  by the final lane verifier uses `route_category=APPARATUS_INVALID` and
  `failure_cause=LANE_WORK_MISMATCH` with the lane-initial rollback frozen
  above;
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
- `nextengine.nonlocal.ncgp14.result.v3` starts with these exact
  length-prefixed identity strings in order: schema
`nextengine.nonlocal.ncgp14.result.v3`, NCGP14 contract root, source commit,
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
computed trial produces its `trial-observables.v1` root. The v3 lane root
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
  the truthful `trial_2_execution` value; every lane's exact
  `route_category`, `failure_cause` and ordered reached-child records;
- predictor plus per-round per-face side/bottom/top ghost-pair, first-hit and
  clamp-mask counts;
- every NCGP13 and NCGP14 control leaf, including admission outcomes, actual
  Control-7 mutation indices/names/expected+observed outcomes/compact transform
  payloads/mutation roots/rejected-state roots, both complete
  `transaction-rejection.v1` receipts, actual transaction scratch records, all
  five work mutations, geometry mutation records and every
  surface variant string plus wrong-branch/wrong-sign/half-force leaf, with
  every work/result root and typed outcome;
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
`long double` object bytes are forbidden. The sole nonfinite exception is the
explicit raw binary64-bit field in `admission-mutation.v1` mutation 6; it is
pre-admission evidence and may never reach a solver serializer. Vectors bind stable sample IDs.
Binary identity is read fail-closed from `/proc/self/exe`.

## Frozen execution and aggregation order

The external verification first performs the exact detached NCGP13 replay.
The NCGP14 process then executes and aggregates receipts in this order:

1. NCGP14 executable/contract/source identity, all profile/fixture/lane roots
   and raw admission;
2. embedded NCGP13 controls and retained OPEN-128 canonical/permuted route,
   followed immediately by its Control-4 lane-result comparison;
3. independent geometry census, admission/overflow and manufactured
   transaction/work-mutation controls, including the complete Control-6
   route-selection and lane-acceptance synthetic truth tables and Control-6
   seal;
4. OPEN-512 canonical, then OPEN-512 permuted and its Control-4 comparison;
5. TIGHT-128-CAP4096 canonical, then permuted and its Control-4 comparison;
6. TIGHT-128-CAP16384 R8 canonical, then permuted and its Control-4 comparison;
7. only after that comparison passes, conditional R16 canonical/permuted or
   its exact typed skip receipts, followed by the final Control-4 comparison
   and Control-4 seal;
8. side-support-removal control;
9. OPEN-128, OPEN-512 and TIGHT-128 surface censuses and their formula
   mutations;
10. only the real finalization/selector predicates on actual sealed lane
    categories, followed by final aggregation.

Within a successor lane, a false apparatus/invariant/correspondence predicate
immediately ends the reached prefix. The route/category is assigned
`APPARATUS_INVALID` without evaluating physical, QP/projection classification,
full-lane acceptance, category-closure or later root-equality predicates. The
independent expected receipt includes only actually reached stages. In
particular, a trial-observable correspondence failure evaluates exactly the
same eight ordered predicates in each of the canonical and permuted lane
envelopes: `position_rmse_m`, `position_maximum_m`, `velocity_rms_mps`,
`maximum_speed_mps`, `energy_positive_excess`, `momentum_residual`,
`component_count`, then `trial_observables_root`. This is eight comparisons per
lane and sixteen per pair; both envelopes evaluate all eight without
short-circuit. It then stops: it does not budget or run the seven sequence
gates or the one trial-acceptance predicate, and its first cause remains
`PERMUTATION_CORRESPONDENCE_INVALID` rather than a later work symptom. Every
reached-prefix work comparison occurs before the private commit it guards. No
result/root self-seal is used as a late validator or resealed after route
selection.

A valid physical or solver-work rejection in one independent lane does not
stop later independent lanes. Within a lane, the first rejected trial stops
that lane transactionally. Any identity, admission, oracle, permutation,
transaction, work/root or CUDA-forbidden-path apparatus failure stops all
subsequent work and emits ordered typed zero-work receipts for the remainder.
No arrival order, exception path or boolean short-circuit may change this
receipt order. All declared embedded JSON validation failures—including
missing, duplicate, wrong-type or wrong-length fields—are caught inside the
successor orchestration and produce the versioned v3 JSON,
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
