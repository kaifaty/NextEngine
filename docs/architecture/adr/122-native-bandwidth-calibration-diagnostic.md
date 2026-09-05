# ADR-122: Native bandwidth calibration diagnostic

| Field | Value |
| --- | --- |
| ID | ADR-122 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-06 |
| Dependencies | SPEC-35, ADR-069/115/116/118/119/120/121 |
| Supersedes | ADR-118/120/121 exact four-sole admission only for the new V11 factory; all prior profiles stay unchanged |
| Superseded by | none |

## Decision and boundary

Add exact opt-in `nextengine.body.humanoid-biomechanics-raja-1700.v11`,
revision11, source domain `nextengine.source.raja-1700.coupled-bandwidth.v11`.
Clone complete V8 and replace only20 stiffness and25 damping coefficients with
the fixed Q16 vector from [BODY-BANDWIDTH-01](../../development/r8b-native-bandwidth-calibration-2026-09-06.md).
Calculation SHA256 `5dece9d3940a2cd8c00e4a61281346a98a47746d117badd590b56aff20b59272`.
The source factory stores that literal vector by semantic joint name. All
bilateral gains are equal. No stiffness increases. No runtime eigensolver,
automatic inertia inference, arbitrary gain setter or model promotion.

CompiledV3 permits four sole shapes only after equality to the complete V11
factory; CompiledV4 retains the exact force schedule. Geometry, mass/COM/full
inertia, joint axes/limits, effort/rate/power/work/target safety and all materials
remain V8. Descriptor export is native diagnostic-only, not mirror/training.

The new `probe_native_joint_bandwidth` accepts exactly8 or11, executing the
frozen27-world unloaded census for that body. It uses production checkpoint
restore to raise the free body100m and initialize four ROM-interior joints;
the captured joint pose initializes the existing safety controller's target
state. Gravity/240Hz physics and full motor safety are unchanged. Every ground
contact rejects the unloaded trial; existing classifier rejects active forbidden/
self/impact contacts. Raw zero-impulse self-contact records are retained, not
called absent constraints. No standing/fall-recovery claim follows from falling
freely without touching the ground. Canonical states, targets, actually applied
efforts/flags, failure boundary and complete trial census are exported as JSONL.

V11 is not admitted to old standing/contactV2/terminal-specific constructors.
Conditional loaded tests require an explicit compatible consumer decision after
the response result, not a profile bypass. V8 is the unchanged nominal-standing
control. Failed V9/V10 remain immutable diagnostics. No optimizer/default change.

## Verification

Require exact gain/identity-only factory delta,20 changed K and25 changed D,
no K increase, full altered/renamed material rejection and native descriptor
coverage. Test exact pulse windows and bounds. Native motor/example tests,
all-target Clippy/format, boundary/content checks, old V8 standing output and
the frozen native corpus/independent executable review are required before
claiming this diagnostic validated. Broad host/performance and game/play/replay
checks are not triggered by this isolated unloaded tool (no standing/game
consumer is changed). Full calibration still requires loaded response and
disturbance recovery; failure never authorizes safety relaxation or a second
gain vector under V11.
