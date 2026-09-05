# R8b sampled foot control — contract revision 1

Research ID `FOOT-CONTROL-01`, status `SUPPORTED_BOUNDED / outside_margin`
for the frozen local response and unclipped linear model, not native stability.
Consumer: decide whether the current ankle/toe gains need a discrete-time
correction, retaining human-like geometry and mass. Snapshot `f64a9a4d`;
Accepted SPEC-35/ADR-118/119 diagnostic scope, exact V8/CompiledV4/native SDK.

## Native local-response measurement

Use102 fresh native worlds: zero twice, then25 DOFs × both signs × moments
0.01/0.02Nm, exactly one240-Hz physics step each. Root lifted10m; both knees
and elbows initially0.1rad, all other joint coordinates/velocities zero.
This moves the four zero-minimum joints into their ROM interior. Use production
state import; all initial snapshots must match. These are bounded open-loop
plant inputs, not a new safety/reference controller. Check torque and per-step
rate bounds explicitly; initial velocities zero make initial power/work zero.
Retain production observed-state validation and all resulting joint/link/contact
data. Stop each world after that single step, including failed observations.

The zero runs must be exact duplicates and keep relative joint positions
within1urad of reset and velocities within1urad/s. No ground endpoints or nonzero
canonical contact impulses in any case. Raw zero-impulse self-contact records
are retained, not called absent constraints. Require every observed state within
unchanged ROM/velocity bounds. Any failure invalidates the linear model fit;
preserve the complete finite trial census, no retry-to-green.

For each amplitude construct B from central velocity response:
B_ij = (v_i(+tau_j)-v_i(-tau_j))/(2*tau*h), units1/(kg m²), h=1/240s.
This is a measured response operator, not automatically an inverse mass matrix.
Use nativef32 and canonical integers; diagnostic calculations IEEEf64.
Frozen adequacy checks: both amplitudes' operators differ by no more than1%
in max-entry norm relative to the larger operator norm; reciprocity error of
each operator is at most1% in the same norm. Symmetric-part eigenvalues must
all be positive. Retain the full operator, errors and eigenvalues; no trimming
or matrix repair. These checks support a local approximation only, not a
uniform error certificate or contact/velocity-limit behavior.

## Sampled-controller model

Only after the native controls pass, evaluate the linear zero-reference PD
model with actual descriptor diagonal K,D, constant B and constant effort over
one outer step. For16 equal kick-then-drift substeps, alpha=17/32 and:

v_next = v + h*B*(-K*q-D*v)
q_next = q + h*v + alpha*h²*B*(-K*q-D*v).

This follows by summing16 equal velocity increments, not by claiming one
semi-implicit Euler step. The pinned TGS implementation applies forces and
records motion per position iteration; actual constraints, varying articulated
inertia, canonical rounding, effort rate/power/work clipping and standing
feedback are excluded from this frozen linear model. A model eigenvalue outside
the unit circle is not by itself proof of a native nonlinear instability.

Quantified finite claim: do the two measured operators with current K,D give
spectral radius>1.1, or do both give<0.99? Otherwise INCONCLUSIVE under this
decision margin. Report all eigenvalues and dominant mode participation.
An outside witness from this model motivates a bounded control correction
experiment, not a mass change or automatic gain promotion. No fitted gains or
controller replacement under this contract. Keep coupled whole-body modes;
do not infer full stability from three independent ankle/toe scalar inertias.

Controls for analysis: independent manufactured one-DOF stable and unstable
maps with known eigenvalues; compare direct16-step integration against the
closed update; test rejected malformed/incomplete/constraint-contaminated native
inputs. Independent reviewer must check actual matrix order, indices, units,
alpha, controls and bounds before interpreting the model. At most one native
repeat and two review passes (one initial, one batched repair/re-review).

Budget:102 one-step worlds plus one exact repeat; no amplitude/gain sweep or
changed physics under this ID. If controls/model adequacy fail, keep the result
inconclusive and name the failed assumption. No changing thresholds to fit a
matrix. This is the new V8 interior-pose unloaded response, not reuse of the
rejected loaded V6 inverse-response calibration.

## Result and independent review

All102 trials satisfy the declared controls: identical zero outputs, exact
zero relative motion, all observed safety checks valid, and816 retained contact
records with no ground endpoint and zero canonical impulse. One repeat is
byte-exact. The two response operators differ by2.99355e-6 in the frozen norm;
reciprocity errors are5.98709e-6 /1.49678e-6. Minimum symmetric-part eigenvalues
are0.341370 /0.336220. Both operators pass the frozen adequacy checks.

The actual-gain model has spectral radius53.852480 /53.849456, dominated by
a negative real mode. Bilateral ankle roll contributes about93.44% and MTP6.45%
in the stated [q,hv] coordinate metric; this is not a physical energy fraction.
The local model strongly rejects these gains as a stable unclipped sampled
controller in this pose. Clipping, contact, posture and varying inertia still
prevent transferring that conclusion directly to native standing behavior.

One fresh independent reviewer recomputed all response entries in SI units
(maximum difference5.69e-14), propagated all50 basis vectors through16 kick/
drift steps in [q,v] and transformed to[q,hv] (matrix difference2.14e-14).
Native reset/census/safety/contacts, K/D order, force-schedule source and compiled
outer identity were independently checked. No load-bearing defect or second
review. B condition numbers11854/12037 and dominant-eigenvalue condition~1.139
were independently measured. The global adequacy norm is not per-channel
accuracy or a uniform certificate; canonical rounding alone contributes up to
0.012/0.006 per response entry. Native/model error is separate. No inferred mass
change or promoted gain set follows from these observations.

Smallest next action: a separately frozen gain-correction candidate focused
on the coupled foot channels, checked in this model then in the original native
standing/loaded/unloaded tests. Preserve anatomy/mass and original safety.

External root:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/sampled-foot-control-01`.
Native/repeat SHA `8bf3eb8a08eb4ac032c7c736377d60c9826f9a62e3aa7f554f09ef30a359d12f`;
report SHA `0a542b68f59a544157326c45de3b17bffbd0eb44627b459448481509e5f80d90`.
Native source SHA `3a37ac573549b7c516097f93adb57747b64e93c721ab272cdc3edb1635598838`;
audit SHA `42391434ca23c678a516b26fda4430b73c75adb75b64e1cc19f2c1fc0c62ae33`;
tests SHA `cc064666c2a024c6767e49efe98ec9017f963bc13f4bcce484098967ca67e1d1`.
Frozen contract before result/status additions SHA
`29fbccba4120c759bbb0fda03e6f950743031e99515192ea431078eb1a442587`.
Descriptor and tool hashes are in report.json; NumPy1.26.4/Python3.11.15,
Rust1.97.1 and SDKf259d3da… unchanged. Native source commit517a0073… inspected.

Reproduce with `cargo run -p next_motor --features physx-sdk --example
probe_foot_small_response`, then `python -m lab.scripts.audit_sampled_foot_control
DESCRIPTOR TRACE OUTPUT`. Three Python manufactured tests, Ruff, focused native
Clippy and the exact native repeat pass. Body/library code remains unchanged;
no native closed-loop gain-correction trial, training or stability admission
has happened under this contract.
