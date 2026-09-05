# Contact iteration discriminator — contract revision 1

Research ID `r8b-contact-iteration-discriminator.v1`; baseline `afb1971b`.
Engineering consumer: determine whether a larger TGS position-iteration
budget repairs the cold contact sensitivity and existing standing reference
before changing controller/body parameters. SPEC-26/35 and ADR-059/115/116
remain Accepted and unchanged. This is temporary diagnostic physics only.

## Frozen experiment

Change only the native v2 articulation's position iterations from 16 to 64;
keep velocity iterations 4, 240 Hz, per-iteration external forces, joint and
material friction, V6 shoulder-yaw-gain-/16 schema, masses/inertias, poses,
targets and safety unchanged. TGS position iterations also subdivide solver
time; the result cannot isolate algebraic convergence from that subdivision.

Use the complete existing cold-pair apparatus: grounded and root raised
500000 micrometres, zero initial velocities/joints, 94 fresh one-step worlds
per case, signed 0.01/0.02 N m on 23 DOFs, repeated exact controls. Reuse
`audit_coupled_effort_response.py` for central response in (rad/s)/(N m),
relative Frobenius difference with max-norm denominator <=5%. Require exact
initial translations, positive grounded zero-control support and zero raised
ground records. Retain unilateral-limit and canonical-rounding caveats from
the [cold contract](r8b-cold-contact-response-2026-09-05.md).

H1: insufficient position-solve budget is sufficient to explain this cold
inconsistency; predicts grounded <=5% while raised remains <=5%. H2: contact
active-set/float/constraint behavior remains inconsistent at this budget;
predicts grounded >5%. Failed boundary controls make the outcome INCONCLUSIVE.
An unchanged/failed response refutes sufficiency of this exact increase, not
every possible solver repair. Do not sweep further budgets under v1.

Budget: one cold pair, one 30-second baseline standing; if baseline reaches
the original safe timeout, one existing k=2 hip-feedback standing run. Stop
each standing case at its first unchanged terminal. Report full root/torso
tilt and contact failures, never use a final upright frame as standing success.
There is no promise of differentiability, inverse mass, global convergence,
stable control, realistic feet or learned locomotion from this experiment.

Identify the temporary constructor patch through source hashes and explicit
`native_source_override: r8b-contact-iteration-discriminator.v1` metadata.
Record actual 64/4 counts in diagnostic output. Old body/compiled hashes are
input identities only, insufficient to admit the altered native dynamics.
No training may consume these outputs. No ambient override or permanent
bridge/default change. Revert setter/count metadata after independent review
and reproduce the original cold SHA byte-exactly as non-regression.

Freeze source/contract/raw hashes before one independent review (at most one
batched repair/re-review); reviewer may perform one focused native rerun.
External artifacts live under the existing external body-audit root in
`contact-iteration-discriminator-01/`, never Git.

Primary-source context: the [official stability guide](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/guides/articulation_stability.html)
(updated 2026-08-21, read 2026-09-05) describes TGS position iterations as
reducing effective solver timestep. This motivates the controlled comparison,
not a claim that a native drive formula applies to our external explicit PD.

## Outcome: exact intervention is insufficient

Independent `iteration_review` found no load-bearing defect and reproduced
the native cold output byte-exactly. The frozen sufficiency prediction is
`REFUTED` (empirical/numerical/correspondence): grounded discrepancy grows
from45.4775646455% to87.7159766369%. Raised discrepancy remains0.018519501146%.
Raised response matrices agree, but complete raised snapshots do not all
agree; do not claim full trajectory equivalence. The canonical rounding
bound cannot reverse the ground failure (conservative lower bound87.595%).

All94 reconstructions per case, repeated controls and translation controls
pass. Grounded zero support is2.657697 N s,10 ground records/8 loaded;
raised has no ground records. Metadata counts are supported by source route
inspection, not a native getter. The only behavioral diff was replacing
`setSolverIterationCounts(position_iterations, velocity_iterations)` with
`setSolverIterationCounts(64, velocity_iterations)` in native v2 construction.

The baseline completes7200 substeps/30 s; normalized full-substep root tilt
peaks8.180782 degrees, final5.246096, last10-second maximum6.637551. Sampled
torso maximum10.044066, final5.891679, last10-second maximum8.055108. The latter
was9.905388 at16 iterations: modestly smaller oscillation, not upright balance.
Hip feedback fails at tick1696/substep6784 (28.267 s), bilateral foot impulses
8.833121/14.726648 N s versus unchanged6 N s ceiling. Longer survival is not
passing safety. Reviewer root figures computed without normalization differ
by <0.0004 degree; no decision depends on that rounding distinction.

External root is the existing body-audit directory plus
`contact-iteration-discriminator-01/`:

| Evidence | SHA-256 |
| --- | --- |
| Frozen contract before results | `b54ad6eabc2d4b6243c4ad6b72c1b16f185779854ad0286594a4ccc9cd368098` |
| Temporary native bridge | `74a0de1f51e780866cf52746a049a4e02bedfb0a5552f1b990dd44a057738920` |
| Temporary main | `1d50ad8113b7dccc64d122917e8224ecac4c1989132c340eaafb70e8571bd3e0` |
| Temporary helper | `e164e54f4702e21c59a4f7633e71cf57357bb08d1cea2d6332d0da1a8ea214f4` |
| `cold-response-64.json` | `4dc965bf07ce973081f6a0d9df527c7498e35a24d2858ed5ec75b3b4abbb9387` |
| `cold-analysis-64.json` | `56616f8ffcc537db011aa4b20730c30acda2a4f11cac6ad2ee0207c064dc08cc` |
| `standing-64.json` | `c87ddd2772bb93f84445a2437deb5872b9c964d854e4aecd2499c148ab06879e` |
| `hip-feedback-64.json` | `f14497adf1e90bcca370613bbc559a632421600098d2a78a405554cc699aa286` |
| `restored-cold-response.json` | `04d11922098b283672a024da35c06b8c2d77e7ec51b58c4c13ec5d79355b0e0f` |

Temporary native and metadata edits are reverted. Original cold SHA reproduces
exactly. Broad ProductChecks NOT_RUN for the reverted experiment; no runtime
profile/default selected, no training launched. Do not sweep more iterations
under v1. A smooth infinitesimal contact map is not itself a requirement for
standing or training: investigate actual closed-loop motion next, through the
[coupled damping discriminator](r8b-coupled-damping-discriminator-2026-09-05.md).
