# R8b foot gain correction — contract revision 1

Research ID `FOOT-REPAIR-01`, initial status `NOT_TESTED`.
Consumer: select one bounded native corrective experiment from the independently
reviewed [sampled-control result](r8b-sampled-foot-control-research-2026-09-05.md).
This is a successor experiment; FOOT-CONTROL-01 did not fit or promote gains.

Candidate A preserves every stiffness, anatomical/body/mass value and every
safety bound. Change only bilateral ankle-pitch damping /16 (40->2.5Nm s/rad),
ankle-roll damping /32 (30->0.9375), MTP damping ×4 (1311->5244Q16, about0.080017).
This reduces the two high-damping sampled foot channels while increasing the
weakly damped light toe. Simple fixed factors are candidate values, not a fitted
human muscle law or a claim of native stability. Hypothesis: the strongest
sampled foot mode is removed without modifying source masses or joint geometry.

First evaluate this exact Q16 candidate against both retained response matrices
with the previously independently checked map. If either spectral radius>=1,
do not run a full native rollout for this candidate and do not tweak it under
this ID. Record dominant remaining mode and reject it as a full linear repair.
If both are<1, it is eligible for a separately identified native diagnostic body
under the architecture workflow, not an existing identity/default overwrite.

Any native candidate must preserve complete V8 geometry/mass/K/safety, change
only the six declared D values and carry a new body/source/compiled identity.
Use unchanged standing-reference equations and anatomical-foot contact rules
with explicit compatible admission, never spoof the old compiled descriptor.
Before promotion compare original30s nominal stance, bilateral loaded transfer
(both v1/v2 inputs and frozen oracle) and the six FOOT-SERVO-01 worlds. Original
V8 outputs are controls and must remain byte-exact. Keep every failure; a stable
unloaded run or lower mode radius cannot replace loaded support/re-contact.

Budget for this preflight: one exact gain vector × two matrices, no gain sweep.
Candidate selection report alone does not change Accepted semantics or fulfill
the user goal. If rejected, use the remaining mode to choose a materially
different correction; no arbitrary mass change or softer safety criterion.

## Candidate A preflight result

Using the frozen exact six damping changes and both reviewed operators gives
spectral radius3.455757827 /3.455923702. The dominant remaining negative mode
is mostly bilateral elbows, with shoulder participation. Candidate A therefore
fails its full-model preflight; no native body or rollout is created for it.
This does not claim the foot changes have no benefit; they are insufficient
for the stated all-joint linear criterion. Revisit only if the consumer changes
to a justified narrower domain, never relabel this as a complete correction.
