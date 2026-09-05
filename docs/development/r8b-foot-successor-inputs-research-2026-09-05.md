# Articulated foot input audit — contract revision 1

Research ID: `FOOT-INPUT-01`. Architecture: `a8036086`, Accepted
SPEC-35 / ADR-069/114/115/117. No body or training profile changes here.
Consumer: choose source data and geometry for an actual two-segment foot,
without importing an impossible inertia or hiding a mass-envelope mismatch.

## Frozen questions and method

1. Compare original2015 and pinned upstream2016/2023 calcaneus/toe mass,
   COM, inertia and MTP metadata on both sides. Does the newer source remove
   the original toe principal-moment triangle violation? Negation: a negative
   triangle margin remains at engine micro-unit precision.
2. Does the current V7 foot box contain the source toe COM at zero MTP angle?
   Negation: one exact coordinate lies outside the box. This is a geometric
   incompatibility with that intended split, not a COM error of the merged foot.
3. Measure the right calcaneus/toe visual mesh bounds and their neutral union.
   Bone/visual bounds inform a proxy, not a measured soft-tissue envelope.

Use exact rational arithmetic for source decimal literals, round ties-to-even
to micro kg m² for engine-scale inertia comparison. Report raw margins too:
serialization noise must not be silently called a physical thickness.
OpenSim axes forward/up/right map to engine right/up/forward. MTP parent and
child frame orientations agree at zero, so neutral toe translation is the
parent anchor plus its local COM. All lengths metres, masses kg, inertia kg m².

Finite domain: three named XML files, four foot bodies per file, two MTP
joints, two ASCII VTP point arrays. Source commit
`d9b05d470b1a481c222372c85b75772faf8f7792`, resolved by `git ls-remote`.
No parameter sweep, simulator or optimizer. Controls: realizable finite box
diagonal, original invalid diagonal, planar equality and a deliberately
out-of-box point. Exact equality/inclusion tests have no epsilon.
One independent source/code/output review is required before using results.
Stop after this finite audit; it cannot certify articulated dynamic balance.

## Sources

- [Pinned OpenSim2016](https://github.com/opensim-org/opensim-models/blob/d9b05d470b1a481c222372c85b75772faf8f7792/Models/Rajagopal/Rajagopal2016.osim)
  and [2023](https://github.com/opensim-org/opensim-models/blob/d9b05d470b1a481c222372c85b75772faf8f7792/Models/Rajagopal/RajagopalLaiUhlrich2023.osim), read2026-09-05.
- [Upstream issue185](https://github.com/opensim-org/opensim-models/issues/185),
  opened2025-06-18, still open on read: reports cross-model toe inertia
  discrepancy; not an accepted instruction to divide a value by10.
- [Falisse et al., 2022](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0256311):
  a two-segment foot improved several stance mechanics in their predictive
  simulations, but worsened ankle kinematic error. Their passive MTP used
  stiffness25 Nm/rad and damping2 Nm s/rad with smooth compliant contacts and
  direct collocation. These are not validated explicit-240Hz PhysX gains.

Raw sources and audit apparatus stay outside Git in
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/foot-source-01`.
## Reviewed result

`SUPPORTED_BOUNDED` for the finite input audit, with independent source/code
correspondence and byte-exact rerun. No load-bearing review defect.

| Observable | Original2015 | Upstream2016 / 2023 |
| --- | --- | --- |
| Toe diagonal, source XYZ, micro kg m² | [100,200,1000] | [100,1100,1000] |
| Minimum micro triangle margin | -700 | 0 |
| Minimum exact XML triangle margin, kg m² | -0.0007 | -1e-20 |
| Toe mass at micro kg resolution | 216600 | 216600 |
| Calcaneus mass at micro kg resolution | 1250000 | 1250000 |

Thus the newer source changes Iyy, not the reported Izz. It removes the
material negative margin at engine precision, but yields planar equality:
`integral(y² dm) = (Ixx + Izz - Iyy)/2 = 0` about the toe COM. A positive-
thickness mass distribution is not established. The tiny negative decimal
serialization residual is explicit, not silently discarded.

At neutral MTP the2015 right toe COM in the rear-foot engine frame is
`[-0.01642,0.004,0.2134] m`; the left mirrors X. The V7 box reaches only
Z=0.210 m, so the intended toe COM is3.4 mm beyond it. The merged V7 COM
remains inside its box: this does not invalidate the existing rigid body's
COM. It rules out blindly partitioning its envelope around the source MTP
while claiming to preserve the source toe segment.

Independent extrema-first mesh check gives the neutral right-foot union:

- minimum XYZ `[-0.039235,-0.012135,-0.009800] m`;
- maximum XYZ `[0.055278,0.043035,0.255718] m`;
- dimensions `[0.094513,0.055170,0.265518] m`.

The current box is260 mm long versus source visual union265.518 mm, but its
rear face is40.2 mm farther back and its front face45.718 mm farther back.
Length alone concealed placement. This source comparison does not prove the
correct soft-tissue envelope or explain the old policy's toe-edge behavior.
Both newer models still lock/clamp MTP;2023 also changes its lower ROM bound
from about-30° to-45°. Do not describe the entire MTP metadata as unchanged.

Evidence SHA-256:

| Artifact | Hash |
| --- | --- |
| `audit.py` | `8bee005e1ec0757a0cf6c9dea406ff37b4d3d77aa1fb291adb238036c22cd053` |
| `audit.json` | `7352683f86ea10a5e5ec6615cac85e65604778dd8a53d38bce3aee2db1b9c8c8` |
| `Rajagopal2016.osim` | `3f5c5f23e486073f2ad2aa4a4967ffe2fcdd582b1e355512bc54f70c36376bf4` |
| `RajagopalLaiUhlrich2023.osim` | `8f30d0b64750b87eb7f705907862590535212b4afd7e919faa3fd7d1683d22ec` |
| `r_foot.vtp` | `dcb305246405ac136bc9c5efa302899cc492222f11f2dafd6191623b4fbd84c5` |
| `r_bofoot.vtp` | `6fd358e6d12b28a3376dbae09c55ddcb4e4570bfce1657e4d6db7551984e9430` |

Original2015 hash and descriptor hash are closed in `audit.json`. Python
runtime: `/home/kaifaty/NextEngine-training/isaaclab-2.3.2/env/bin/python`.
Command: that Python plus the absolute `foot-source-01/audit.py` path;
stdout is the report. Five manufactured assertions pass. Reviewer independently
checked frame parentage/scaling and extrema with an alternate formulation;
upstream was not refetched by the reviewer. Main retrieved immutable raw URLs
at the `git ls-remote` commit; GitHub API discovery returned403, not used as
source evidence. `preview.py` renders `foot-side-comparison.png`, inspected
visually; it is presentation only and is not the arithmetic oracle.

## Implementation decision / next action

Do not copy the2015 toe inertia, treat2023 as volumetric, divide Izz by10 on
the authority of issue185, or split the old box without changing its placement.
Do not repeat the source audit unless a new source identity changes these rows.

Next implement an explicit finite-volume two-segment proxy candidate with the
source MTP anchor and a separately documented mass/inertia projection. Preserve
the75.337 kg body total, bilateral symmetry and original body versions; keep
source mass/COM where physically feasible and declare every necessary deviation.
Verify positive triangle margins and neutral combined first/second moments.
Source mesh bounds are input to that engineering choice, not a skin contract.
Use the rigid-foot V7 as the native control, then test loaded flat support,
heel rise, release/re-contact and meaningful standing/disturbance behavior.

Adjacent implementation checks from source inspection (not dynamic results):
`BiomechanicsProceduralStandingControllerV2` deliberately rejects non-V7 bodies,
so the successor needs its own controller identity. Existing contact impact
aggregation is per actor/shape pair with6 Ns foot ceiling: adding a second foot
link must not silently turn that into a12 Ns per-foot allowance. Audit anatomical
foot aggregation, sole effectors and sibling/self exclusions with the successor.
The paper's continuous/compliant-contact passive gains are not selected here.

PASS: exact controls, independent review, visual inspection, diff whitespace and
local documentation references. Cargo/ProductChecks:
`NotRun(NoExecutableRepositoryChange)`. No actual foot body, learning profile,
checkpoint or running training process was changed. Goal remains open.
