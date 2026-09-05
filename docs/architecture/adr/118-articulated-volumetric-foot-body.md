# ADR-118: Articulated volumetric foot body

| Field | Value |
| --- | --- |
| ID | ADR-118 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-069](069-biomechanics-body-schema-v2-and-solver-projection.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-115](115-full-principal-inertia-body-successor.md), [ADR-117](117-quiet-upright-body-and-standing-reference.md) |
| Supersedes | V7's rigid merged foot only when selecting BodySchema V8; ADR-071's two-sole material count only for the exact canonical V8 factory. No environment, standing reference, contact safety profile or runtime route changes. |
| Superseded by | none |

## Decision

Add opt-in `nextengine.body.humanoid-biomechanics-raja-1700.v8`, revision8,
26 bodies,25 actuators and21 colliders. Each foot gains one MTP hinge and a
separate physical forefoot. Preserve every non-foot body, existing joint and
actuator, total mass75.337 kg, all old body factories and their compiled hashes.
This is a native anatomy/kinematics diagnostic, not a dynamically admitted
standing/training body. Rollback selects V7; never rewrite V8's identity.

The [reviewed input audit](../../development/r8b-foot-successor-inputs-research-2026-09-05.md)
rejects the impossible original toe inertia and identifies the newer source's
planar limiting tensor. Use a declared finite-volume toe proxy instead.
Preserve original source segment mass and COM at engine precision:

| Engine right/up/forward; local segment frame | Rear foot | Toe |
| --- | --- | --- |
| Mass, micro kg | 1250000 | 216600 |
| COM, micrometres | [0,30000,100000] | [-17500,6000,34600] right; mirror X left |
| Diagonal inertia, micro kg m² | [4100,3900,1400] | [100,199,132] |

Rear inertia is the source calcaneus tensor, not the previous merged foot.
It admits a homogeneous cuboid realization contained by the new rear collider.
Toe inertia is a homogeneous80x30x68 mm cuboid about its source COM,
`I_i=m*(L_j²+L_k²)/12`, rounded to nearest micro kg m². Its three principal
triangle margins are strictly positive; its volume fits inside the toe collider.
These are engineering collision/mass proxies, not measured skin or tissue density.
Both use identity principal frames and zero tensor projection error. The actual
change from the old aggregate inertia is explicit, not labeled source-conserving.

Retain each ankle projection group's mass and COM. Add its toe member and
replace its aggregate inertia by the rounded parallel-axis sum of talus,
source calcaneus and new toe. Right group tensor in xx,xy,xz,yy,yz,zz order:
`[8155,-71,309,7958,645,2733]`; left reverses xy/xz. Toe member origin in the
ankle group is `[9000,-43950,130030] um` right, mirror X left. Existing first-
moment/inertia reconstruction tolerances are unchanged; no artificial carrier.
Source provenance domain is `nextengine.source.raja-1700.volumetric-forefoot.v8`;
solver projection domain is
`nextengine.solver-projection.humanoid-biomechanics-raja-1700.v4`.

## Geometry, articulation and materials

Rear collider: half extents `[55000,47500,102500] um`, centre
`[0,28865,87500] um`. Toe hinge origin in rear frame:
`[1080,-2000,178800] um` right, mirror X left. Toe collider half extents
`[60000,20000,40000] um`, local centre `[-1080,3365,40000] um` right,
mirror X left. Both neutral soles remain at rear Y=-18635 um (world floor).
The combined forward envelope is[-15000,258800] um; toe width120 mm, rear110 mm.
Rear/toe overlap near the joint is intentionally excluded for that adjacent pair;
no additional non-adjacent exclusions are introduced.

MTP axis is engine-X negative on both sides: positive extension lifts the
forward segment. Hard ROM[-349066,1221730] urad, soft[-174533,1047198] urad,
neutral0, velocity limit8000000 urad/s. These symmetric-side engineering limits
are not the frozen/oblique source MTP. Existing ankle axes/limits remain.
The new diagnostic actuator has stiffness25 Nm/rad, damping1311/65536 Nm s/rad,
effort +/-12 Nm, effort-rate120 Nm/s, power40 W, positive-work limit666667 uJ
per motor tick, residual100000 urad and target delta +/-50000 urad/tick.
No passive-control or balance guarantee is attached to those initial settings;
paper gains from continuous compliant-contact simulation are not transferred.

Heel effector stays on the rear body at `[0,-18635,0] um`. Forefoot effector
moves to the toe body at `[-1080,-16635,40000] um` right, mirror X left.
Add preserved-value bilateral body/joint/actuator symmetry pairs.
The material compiler permits17 body-material plus4 sole-material shapes only
when the supplied schema equals the entire canonical V8 factory. Otherwise its
old17+2 closure remains. Material coefficients, combine rules and native ABI
are unchanged. Actual assignment counts already participate in material lineage.

## Validation and admission boundary

Require exact unchanged non-foot bodies/joints/actuators and total mass;
positive/contained volume inertia, group moment closure, neutral floor/stature
and no non-excluded overlap; bilateral native MTP extension, rear independence,
coupled ankle/MTP orientation, exact neutral restoration and native creation/
substep. Check exact four-sole material closure, altered/renamed rejection and
old hashes. Run native motor suite, all-target Clippy/format, boundary-scan and
content-package; this package-local diagnostic does not require a broad host
or gameplay run absent a changed consumer.

The inspection export advertises no standing reference, environment or Isaac
admission. Existing standing V2 correctly rejects V8. Before any standing or
learning selection, implement explicitly identified controller/contact consumers:
preserve a6 Ns anatomical-foot impact ceiling across both segments, not6 Ns
per segment; check sole support/continuity and independent reset identity.
Then perform loaded flat support, heel-rise/re-contact and disturbance tests.
Native pose import alone is not evidence of load-bearing balance or walking.
