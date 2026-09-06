# BODY-SUPPORT-EFFORT-01.r1 — frozen experiment

Base5ac24c0f; Accepted ADR-125 additive diagnostic, no default selection.
Full calibration requirements in BODY-GAIN-NATIVE-01 remain unchanged.
Prior static LP does not prove native torque insufficiency because passive
joint friction was omitted. This experiment tests a different controller
mechanism empirically; it does not claim that objection is resolved.

## Fixed controller and observations

Exact V11 body, original authored ground reset/free pelvis/gravity, native
CompiledV4/240Hz and60Hz standingV6 reference. No K/D, geometry, reset, contact,
friction, solver or safety change. Add a25-channel support-effort input before
the complete fixed-point safety intersection. Evaluate it once each motor
tick from its pre-tick canonical snapshot, hold four substeps. Keep all tick
link poses, support points/weights/COM and requested support vector, and all
substep joints, applied efforts/flags and raw contacts, including first failure.

F64 diagnostic model: normalize observed xyzw orientations; transform every
body COM and joint anchor/axis from the engine-owned descriptor. World +Y up,
g=9.81m/s², units SI. For each of the four rear-foot/MTP box colliders,
enumerate8 corners in fixed x/y/z sign order; retain abs(worldY)<=0.002m.
These are modeled near-ground support candidates, not asserted real contacts.
At most32 points. Compute whole-body COM using all26 masses including carriers.

For each ordered triple i<j<k whose XZ area determinant has magnitude>1e-10m²,
compute the COM's barycentric coordinates. Accept if all>=-1e-9; clamp negative
roundoff to0 and normalize. Average the resulting point weights over EVERY
accepted triple, not a selected triangle. This provides a symmetric bounded
static force allocator without a contact optimization dependency. Set modeled
point force to [0,total_mass*9.81*weight,0]. If no triangle encloses COM, emit
zero support effort with explicit unsupported flag; do not invent root support,
change the reference or suppress subsequent physical safety failures.

For each joint, form its complete child subtree and compute
tau_support = axis dot(sum_descendants((COM-anchor) cross [0,m*9.81,0])
                       - sum_subtree_points((point-anchor) cross F_support)).
Parent joint-frame rotation is included. No Coriolis/inertial/friction term is
claimed. Quantize each finite Nm value to i64 microNm with ties-even; reject
overflow/malformed source before safety mutation. Complexity O(N³+J*(B+N)*H),
N<=32,J25,B26, tree height H<=26. Only resulting joint efforts go through production safety and
actuation; modeled normal forces are never applied to the native world.

## Hypotheses, gate and budget

H1: added support term lets softened joints retain quiet loaded standing.
H2: allocation/reference/coupled dynamics still fail; preserve earliest native
failure. H3: wrong transforms/signs/rounding or bypassed safety invalidate the
test; analytic sign/rotation/force balance and independent reconstruction
must distinguish H3 from native insufficiency.

Gate unchanged:7200 substeps, timeout only; final10s root tilt<=2deg,
torso<=1deg, both feet loaded, neither MTP RMS speed above exact V8 control.
A shorter failure is not standing PASS. All25 loaded responses, heel-rise/
recontact and disturbance recovery remain required after this gate.

Budget: candidate+repeat, V11 support-zero, exact original V11 and V8 controls.
No gain/allocation threshold sweep. Unit tests cover profile/body rejection,
failure atomicity, zero equivalence, positive/negative support before effort/
rate/power/work safety and unchanged observed limits. Helper tests cover
analytic torque sign, quaternion rotation, symmetric weight/moment balance,
no-support and degenerate triangles. Freeze source/diff/raw hashes; one fresh
independent executable review, at most one batched apparatus repair/re-review.
If standing fails, stop later trials sharing its prefix. Next mechanism must
be selected from the measured failure, not another unbounded parameter sweep.

## Implementation and sealed native evidence

ADR-125 adds one opt-in safety constructor and explicit per-substep vector.
The complete V11 compiler value is checked, not merely its hash label.
The support checkpoint domain binds its CompiledV4 hash; this hash and safety
payload are subject-independent like the old safety profile. Subject-specific
compiled admission remains complete. Old reference/contact/terminal laws and
old safety roots are untouched. Four focused boundary tests cover profile,
tamper, reset, zero correspondence, failure atomicity and both signed i64
extremes through all four effort/rate/power/work bounds. Three helper tests
cover descriptor dimensions, quaternion/sign and support allocation.

External store: `/home/kaifaty/NextEngine-training/body-support-effort-r7BzKQ`.
Frozen contract, source diff/new helper, native binary, both descriptors and
all five raw worlds are in `manifest.json`, SHA256
`ee4ee7242e199b9c51b546957a9a7d25349f1f07c10a3b986ca1e1992596d470`.
Rust1.97.1, pinned SDK path recorded in manifest. Reproduce from base5ac24c0f
plus source diff/helper, `cargo build -p next_motor --features physx-sdk
--example probe_biomechanics_body_standing` with the recorded SDK environment.
The manifest records exact CLI inputs, whose stdout is each raw JSON file.
`measure.py` computes the declared tail gate; no additional native trials ran.

| Native observation | V11 support-v1 | Original V8 control |
| --- | ---: | ---: |
| Duration / reason | 7200 substeps / timeout | 7200 / timeout |
| Final10s root maximum tilt,240Hz | 1.602214deg | 1.325107deg |
| Final10s torso-roll maximum tilt,60Hz | 1.675229deg | 0.149940deg |
| Left MTP RMS reported velocity | 0.035553993rad/s | 0.024579165rad/s |
| Right MTP RMS reported velocity | 0.154938338rad/s | 0.111206245rad/s |
| Minimum upward foot impulse, left/right | 1.527391/1.529874Ns | 1.645005/1.412427Ns |

Candidate and repeat are byte-identical. Original V11 and support-zero both
fail at substep346 with the original bilateral ankle ROM failure; their full
physical/reference/contact records are equal after removing only explicit
new support metadata/flags and normalizing the CLI mode label. Old V8 and V11
outputs match the previous sealed raw JSON byte-for-byte. Candidate support
has zero unsupported ticks, maximum absolute requested support31.678105Nm;
24 applied channel records across17 substeps have nonzero safety flags,
including tail substep6709; independent reconstruction below agrees.

The survival subcriterion improves, but the frozen full standing gate FAILS:
torso tilt exceeds1deg and both MTP reported RMS velocities exceed V8. Do not
convert timeout to quiet-standing PASS or full calibration. Loaded25 response,
heel-rise/recontact and disturbances remain NOT_RUN; this failed prerequisite
does not justify running their shared prefix repeatedly. No training started.

## Exploratory next-boundary observation

Separate `tail_diagnostic.py`/`tail-diagnostic.json`, not the frozen gate or an
independently certified cause: candidate MTP endpoint-angle differences imply
RMS0.000511/0.000430rad/s, much smaller than the SDK-reported0.035554/0.154938.
Angles span only90/68urad during the last10s. Reported velocity AC spectral
peaks are0.3/0.4Hz, with only0.37/0.30% power above100Hz; this result must not
be described as the old V8/V9 near-Nyquist mechanism without new evidence.
All tail support inputs retain16 corners, so point-count switching is not
the observed source of this tail variation. Support magnitudes still vary.

These differences do not prove a readback bug: contact impulses and solver
integration/readback stages may make instantaneous velocity differ from an
endpoint finite difference. Keep the original gate failed. Before another
damping change, discriminate actual joint/link motion, cache readback and the
native contact/force integration boundary with an unloaded successful control.
Torso/root slow motion remains a separate measured balance issue. Do not mask
either issue by replacing velocity in production or loosening the tilt limit.

## Independent review and checks

Initial review was interrupted and resumed, not replaced by a second pass or
candidate repair. External `body-support-review-dZEbzi/review.md` SHA256
`87b860377d624ed8ae07cf4abf363a29481430251d80755edcb3a2222fa64aaa`;
`SHA256SUMS` SHA256
`61014532b093b784adc407df42055b9191f333988d32905f8b6f231eb2f6031c`.
All13 manifest artifacts and6 live sources match. Independent quaternion
cross-product and subtriangle-area formulations reconstruct90,000 support
values,557,300 applied efforts,368,650 flags, all reference/contact/safety roots,
controls and failure precedence. One focused native rerun is byte-identical;
no clean rebuild or ancestor rerun. No load-bearing apparatus defect found.
Executable correspondence is SUPPORTED_BOUNDED; quiet-standing gate REFUTED.
Independent root maximum1.599726deg uses60Hz motor boundaries; the table's
slightly larger1.602214deg uses240Hz root records. Both pass the root criterion;
the observed torso/MTP failures are unchanged. No solver or universal proof.

PASS:161 native motor library tests,15 example tests, native all-target Clippy,
format, eight negative CLI checks, six boundary checks, content-package, play,
persistence-replay and Linux host-check (workspace Clippy/tests/boundary).
The first host wrapper was interrupted after Clippy; its process was confirmed
absent before rerunning, and its partial log is retained. Dev-only serde reuses
the existing pinned workspace dependency; no package version changed.
Full calibration, loaded transfer/disturbance, new environment and training
remain open. No optimizer launched.

## User-directed next work

The user now prioritizes connecting the corrected body to a tested walking
environment. Preserve this calibration failure rather than claiming it passes.
The existing canonical PPO adapter is hard-coded to23 actions; V11 has25.
Next implement explicit new-body/environment identity, descriptor-owned action
width, articulated sole observations and reset/reward/terminal checks. A bounded
pipeline smoke is distinct from selecting a fully calibrated body; do not make
perfect hand-authored walking or this report's1deg tuning criterion a universal
RL requirement. Any new consumer/run needs its explicit architecture boundary.
