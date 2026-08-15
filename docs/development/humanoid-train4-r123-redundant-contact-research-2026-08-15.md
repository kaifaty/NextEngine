# TRAIN-4 R123 redundant-contact research — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Optimizer-free causal research after the sole R123 execution stopped before its first local solve |
| Status | `R123_INVALID / R127_INVALID / R130_INVALID / R133_PASS / R134_COMPLETE / R135_PASS / R136_EXECUTION_NEXT` |
| Primary cause | `Environment / stage-2 rigid-contact formulation` |
| Claim ceiling | Research and generated-test design only; no R123 retry, feasibility claim, candidate, scene or training |

## Observed boundary

The one authorized R123 process ran from clean commit
`7e9e93c6160afee45b70a649f94a058f4eb80628`. All six pre-execution
validations passed, including `278/278` lab tests, `56/56` motor tests and full
`host-check`. The first frozen collocation then returned
`SCALED_LOCAL_SYSTEM_ILL_CONDITIONED` with condition number
`6.874956301874059e16`, above the frozen `1e12` ceiling. R123 therefore closed
as `INVALID / STOP_INVALID_EVIDENCE_WITHOUT_RESTART` after one SVD and zero
local-system solves.

The canonical/file/profile SHA-256 identities are
`3436d95d492586570cdd27fa685f2a517e1ac81ab9350fbd4f2c42f7bb5ab6c7` /
`543513bf4f51797b515b718684123a9f5aeabe394bf4ea73fe20194a9d65acb2` /
`492ce5da3852aa68811ce8afc6f0c5b57205ce8f2fc2ddf32dd71279b4ecda30`.
The report's canonical digest was recomputed independently and matches. It
records one execution process/thread, `1.528 GiB` maximum RSS, no restart,
resume or intervention, no solver-private cache, and zero kinodynamic solves,
candidates, PhysX scenes, optimizer steps or training runs.

This result does **not** say whether the fixed-PD trajectory is dynamically
feasible. It says the frozen equality-only force decomposition is not a valid
numerical discriminator for that question.

## Exact structural explanation

The first collocation has modes `[FLIGHT, FLAT_STICKING]` and active point
ordinals `[2, 3]`. R113 binds both to the same rigid body
`body.right-ankle-roll`:

| Point | Local translation, µm |
| --- | --- |
| right heel (`2`) | `[0, -18635, -35000]` |
| right forefoot (`3`) | `[0, -18635, 180000]` |

Their nonzero separation is `d=[0,0,215000] µm`. Choose equal and opposite
forces along this line: `f_heel=d`, `f_forefoot=-d`, expressed in the common
foot frame and rotated together into world coordinates. Their resultant force
is zero, while their resultant moment is

`r_heel × d + r_forefoot × (-d) = (r_heel-r_forefoot) × d = -d × d = 0`.

Thus the two three-component point forces contain a nonzero internal-force
direction that produces exactly zero generalized wrench. Dually, two complete
3D point-position constraints on one rigid body have at most five independent
rows: rotation about the line through the points remains free. The R123 saddle
matrix embeds this exact null vector with zero acceleration, zero effort and
zero inactive-point force. Row/column scaling cannot remove it; floating-point
roundoff merely turns exact singularity into the observed `6.87e16` condition
number. Clean report-only R123-RC1 at commit `e8fa20f` verifies all of these
facts from the immutable JSON and integer geometry without importing NumPy or
the R123 execution module.

## Competing hypotheses

| Hypothesis | Evidence | Decision |
| --- | --- | --- |
| Corrupt artifact or stale model lineage | Clean commit, exact source hashes, canonical digest and all validations close | Falsified |
| Incorrect M/h/J/Jdot-v implementation | R122's independent conformance remains PASS; the null vector exists for any mass matrix and configuration | Falsified as first cause |
| Numeric scaling or too-strict `1e12` threshold | The same-body two-point wrench map has an exact algebraic nullspace | Falsified; raising the limit would hide invalid force selection |
| Redundant flat-foot multipliers make the square KKT singular | First active set is exactly the same-body heel/forefoot pair and the equal/opposite line-force wrench is identically zero | Confirmed and hash-closed by R123-RC1 |
| Fixed-PD dynamics are feasible or infeasible | No local solve was attempted and no cone margin exists | Still unknown |

## Primary-source research

- Carpentier, Le Lidec and Montaut,
  [*From Compliant to Rigid Contact Simulation: a Unified and Efficient Approach*](https://www.roboticsproceedings.org/rss20/p108.pdf),
  RSS 2024, derives the same rigid-contact KKT form and states that it is often
  non-invertible when the contact Jacobian is rank deficient; it also identifies
  compliance/proximal regularization as a distinct physical/numerical model.
- MIT Underactuated Robotics,
  [*Multi-Body Dynamics*](https://underactuated.mit.edu/multibody.html), uses a
  pseudoinverse when the Delassus matrix drops rank and explicitly notes that
  this selects a minimum-force solution among multiple multipliers.
- Olsen and Kamrin,
  [*Resolving Force Indeterminacy in Contact Dynamics Using Compatibility Conditions*](https://arxiv.org/abs/1805.07437),
  2018, explains that perfectly rigid contact networks can have non-unique
  forces and that selecting one distribution requires an additional
  compatibility rule.
- Caron, Pham and Nakamura,
  [*Stability of Surface Contacts for Humanoid Robots*](https://arxiv.org/abs/1501.04719),
  2015, shows that humanoid surface contact can be represented and tested via a
  resultant contact-wrench cone rather than an unnecessarily large set of
  individual point forces.

These sources support the general mechanics. The exact same-body line-force
null vector above is the repository-specific causal proof.

## Repair alternatives

| Alternative | Assessment |
| --- | --- |
| Raise the condition limit or call the ordinary square solve | Reject: produces an arbitrary roundoff-selected force split and violates the frozen invalid-result contract |
| Use only a Moore-Penrose/minimum-norm force split | Insufficient for the original claim: a minimum-norm witness may violate an individual cone even when another value of the internal force is cone-feasible |
| Delete heel/forefoot or silently drop a kinematic row | Reject: changes the frozen contact geometry or leaves an undocumented force gauge |
| Add compliance/Tikhonov/proximal regularization | Potential future model, but changes rigid-contact physics and needs an explicit parameter/identity decision; not the smallest discriminator |
| Preserve both points and test existence over the internal-force nullspace | Selected direction: reduce to an independent sticking basis and solve the remaining one-dimensional-per-flat-foot cone-feasibility problem, or an equivalent deterministic SOCP/contact-wrench formulation |

If a deterministic force witness is needed after feasibility is established,
its secondary selection rule must be explicit and separately audited. It must
not change the existential cone classification.

## R123-RC1 result

The exact audit returns
`COMPLETE / CONFIRMED_REDUNDANT_FLAT_FOOT_FORCE_GAUGE`. All nine discriminators
pass. It measures separation `[0,0,215000] µm`, exact first/second moments
`[-4006525000,0,0]` / `[4006525000,0,0]`, zero resultant force/moment,
constraint-rank upper bound `5` and minimum force nullity `1`. The observed
condition is `68749.56×` the frozen maximum.

Canonical/file/profile SHA-256 is
`ebf257991c36970e9ccf9501fe4175fc0efa0efed2e3b1a8ac1e176acef045cf` /
`a35d408a901ea2c439ac387287fa03aa77c669863211fb6b0d7634ce3b0836f9` /
`4ee1fd77701e638a3087cbeaa6498482e073c33133bbb89a1a0b6d134d2ee8b7`.
Research-module/tool SHA-256 is
`ddeb9c81f1a4d6488b972b6fcfa55a9f77f068ac516b1efe0fc0917c61942bb3` /
`e3f0538aebcadaaf7e75881b7ce6ee52f2feaceca0ee50137c169c4717f67b91`.
All six validations pass, including solver-free import, `283/283` lab tests,
`56/56` motor tests and full `host-check`. The audit records one research audit
and zero local-system reconstructions, SVDs, solves, caches or downstream work.

## Decision and next gate

Freeze R123 as invalid and never retry or reinterpret it. The only next action
is one separate report-only R125 redundant-contact feasibility formulation. It
must preserve both point locations, rigid sticking and individual friction
cones; define an independent constraint basis and existential gauge treatment;
and pre-register deterministic conformance/resource guards. It may not
reconstruct, factor or solve a frozen local dynamics system. R125 itself has no
execution authority.

R124 remains unauthorized because its prerequisite was a *valid* R123
completion. Every R123 retry, additional KTO/ID/kinodynamic solve, candidate,
PhysX/all-17 run, corpus admission and learned optimization remains forbidden.

## R125 formulation result

Clean report-only R125 at commit `a80ed0e` freezes the selected repair without
assembling a real system. Fixed effort and inactive forces are eliminated from
the algebra but remain exact audited/output identities. Local layouts become
`29` variables in flight, `32` for one active point and `35` for flat foot.
Across the unchanged schedule this is `107668` algebraic rows/variables with
independent rank `105352`; the difference is exactly the `2316` predeclared
flat-foot gauge scalars. All original `4956` point cones remain.

R125 freezes one SVD-derived equality-consistent particular solution, an
analytic same-foot force gauge and exact-rational one-dimensional line-cone
interval intersection. Feasibility depends on whether that complete interval
is empty. A minimum normalized-force alpha is selected only afterward for
deterministic cache bytes, so it cannot turn a feasible gauge family into an
infeasible minimum-norm witness.

Canonical/file/profile SHA-256 is
`ddf443610315680d0326b478212105c846a0cfda2b8bcda95567556ea1773080` /
`bdc5cd5388005bbb549df7bfb723dda49a1535d2a6a39c4e31da58340e3ac0db` /
`ca9cc4019e45ea316072378422e7aab304664b24034f204e41b3ccbe30e7da05`.
Formulation-module/tool SHA-256 is
`fc18c0e112bf2dad7000cf49a9ff173e598bddbb0634bb0b4f121a49c00b295e` /
`a93b7a12b4ede4b6336578dce54c28a88a701427916a3bac10b5751774b180dd`.
All six validations pass, including `288/288` lab tests and full `host-check`;
every SVD/particular/gauge classification and downstream work count is zero.

## R126 conformance result

Clean report-only R126 at commit `e963599` passes all seven frozen real
`29/32/35` rank/nullspace anchors, five synthetic SVD discriminators and four
independent decimal-oracle line-cone cases. Maximum analytic gauge residual is
`9.056e-16`; maximum analytic/SVD projector error is `1.037e-12`. Canonical/
file/profile SHA-256 is
`2a500b6e6514e3a5cc8cec453756089f235d66d8684718a56678c471202f3e8f` /
`4931ff4c96e8bb062bed64a45681097ae70b23c615c022ba267c0ae6edb6ffd3` /
`23007c0455fef7cf84da411528f9f6162cd04d97e8be86baa7be2c19ae7e87cb`.
All six validations pass (`295/295` lab, motor and full `host-check`). R126
computes zero real particular solutions or gauge intervals and authorizes
exactly one bounded R127 execution. No retry, candidate, PhysX or training is
authorized.

## R127 invalid boundary

The sole clean R127 at commit `44b536b` passes all six validations, then stops
at collocation `0`. The reduced system has the expected rank `34`, nullity `1`,
analytic gauge residual `1.11e-16` and analytic/SVD projector error `8.35e-13`.
Its RHS is nevertheless incompatible: particular scaled residual
`6.2044653e-5` exceeds `1e-9`, so no gauge interval is classified. Canonical/
file/profile SHA-256 is
`255f2dd900f7ca67381fd6853aa42e47a991680f723b17539e9505651d6e7a4e` /
`0bdf21b2e995a3ea16f7670666a10b73379d6f6eda9dede04dea5c5e788e1a2b` /
`0e3537dd36e1a148788d6e4634e4409a01d10136033bf957f7a5d684699976c2`.

Clean report-only R127-RC1 at `9d5cdbf` gives right-foot line length `0.215 m`,
perpendicular angular speed `0.131848744 rad/s` and projected heel/forefoot
`Jdot-v` difference `-0.003737579615 m/s²`. It matches the rigid centripetal
identity `-L||omega×d||²` within `4.34e-19`: the frozen velocity is not tangent
to the two-point FlatSticking acceleration manifold. Canonical/file/profile
SHA-256 is
`a1028728e50050747c2167b45d76726aceebd24a7c881bd11bc9eb1e2c8dcc62` /
`a32f84719b72bf826255287209c596c4faba99646e16112f970acc3c4a5b63ba` /
`1e0137b3c1ba37cedd62da9f20e71616b8cc3a52d2f3e36df49ed3613549ec70`.
This is not a remaining force gauge defect. R127 cannot retry. Clean R128 at
`0220493` selects mass-metric tangent-velocity projection as the smallest
pointwise diagnostic, with no position/integration claim. Its canonical/file/
profile SHA-256 is
`32a9e278f0c1afe74b646daaf9ef2e446e04e1e56c101caa4e8221cab76cd3d0` /
`15cba3557e487b3af33cb9b9b281501f90aea2fa662151dcbf99277bab46f56e` /
`4a6caece3113ad622ad8a2481bb07849b139eab626015f68511c66cabfe74035`.
Clean R129 at `b81270f` passes all seven frozen projection anchors. The exact
rank/nullity cases pass, maximum active-point velocity is `9.281e-16 m/s`,
maximum scaled KKT residual is `1.469e-14`, and the projected flat rigid-line
incompatibility is at most `2.277e-17 m/s²`. It performs six projections and
zero inverse-dynamics/downstream work. Canonical/file/profile SHA-256 is
`b34eb4727165e9b16ef82f597138fde33993c12efa8e3177776254237c6fbb99` /
`490b32f67c98d07d9daf0fe9c301372d69b8b85774227658b942b05210531829` /
`d25557c5a07cc243570c1a2b57d9ecb6c8c9ac12bdd31c1ec950f4cfbfc4a376`.
The sole R130 is consumed and immutable `INVALID`: all projection rows pass,
but its fixed-PD schedule violates right-ankle-roll bounds before inverse
dynamics. Clean [R130-RC1](humanoid-train4-r130-projected-schedule-research-2026-08-15.md)
confirms repeated contact-exit hotspots. Clean R131 selects a nine-exit
mode-owned lift; clean R132 passes `36/36`, lowers all `27` changed corrections
and finds zero local speed violations. Clean R133 passes all `3200` projections
and the complete schedule with zero unsafe actuator categories. Clean R134
freezes the exact projected q/v/effort plus gauge-aware inverse-dynamics
composition with zero numeric systems. Clean R135 passes its report-only
composition conformance. Exactly one bounded R136 pointwise ID execution is
permitted; kinodynamic execution and every downstream action remain blocked.
