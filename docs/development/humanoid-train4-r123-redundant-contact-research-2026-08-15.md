# TRAIN-4 R123 redundant-contact research — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Optimizer-free causal research after the sole R123 execution stopped before its first local solve |
| Status | `R123_INVALID / R123-RC1_COMPLETE / R125_REPORT_ONLY_FORMULATION_NEXT` |
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
