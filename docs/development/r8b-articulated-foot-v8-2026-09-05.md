# R8b articulated foot V8 — implementation evidence

Status: `KINEMATICS_IMPLEMENTED / LOADED_SUPPORT_NOT_TESTED`.
Authority: [ADR-118](../architecture/adr/118-articulated-volumetric-foot-body.md).
Input selection: [reviewed foot audit](r8b-foot-successor-inputs-research-2026-09-05.md).

## Change and engineering rationale

V8 replaces each rigid merged foot with a source-mass rear segment and a
finite-volume forefoot at the source toe COM. Two MTP hinges provide independent
forefoot extension. Total mass remains 75.337 kg; all non-foot bodies, existing
joints and actuators remain exact. The previous torso alignment is retained.
There are 26 bodies, 25 actuators and 21 collision shapes.

The toe uses the declared 80 x 30 x 68 mm homogeneous mass proxy, giving rounded
principal inertia [100,199,132] micro kg m². Its volume fits in its collider.
Source rear inertia [4100,3900,1400] admits a homogeneous cuboid; testing its
containment exposed a real draft geometry error. The original 90 mm rear box
provided only 41.365 mm above the COM, while the equivalent cuboid requires
sqrt(0.00192) m = approximately 43.818 mm. Increase the top by 5 mm, retain the
sole at rear Y=-18.635 mm and preserve mass/COM/inertia. Final height is 95 mm.
This is an explicit engineering proxy, not measured skin/tissue geometry.

Initial native test output (`native-tests.log`) has 143 passes and this one
failure. The repaired suite (`native-tests-r2.log`) has 144 passes. The integer
containment test is unchanged; the repair changes physical geometry, not its
success criterion. The four native sole-corner checks now include the complete
lateral quaternion term and actual mirrored collider offset.

An earlier compile rejected four sole shapes because material closure was
hard-coded to two. The exact canonical V8 factory now permits four, with full
schema equality validation. Renamed and geometrically modified V8 inputs fail.
Old factories, material coefficients and material lineage semantics stay exact.

## What the native tests establish

- Positive 30-degree MTP extension raises all forefoot sole corners, on both
  sides, without changing the rear link's native state.
- Combined ankle/MTP poses pitch the rear while keeping the forefoot level.
- Restoring the neutral raw checkpoint is exact; native creation and a
  25-effort substep succeed.
- Neutral stature, floor alignment and non-excluded collider clearance pass
  alongside the preserved earlier profiles.
- The old V2 standing controller rejects the new body. Inspection does not
  advertise a standing reference, training environment or mirror admission.

These are production native pose-import and geometry tests, **not loaded heel
rise, re-contact, disturbance recovery or learned standing**. No optimizer ran.
No gameplay/default environment changed. Initial MTP gains are diagnostic
settings, not a validated passive-foot law.

## Exact artifacts and checks

External root:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-v8-01`.
The earlier `descriptor.json` is the rejected geometry draft; use
`descriptor-r2.json` for the implementation reported here.

- Body hash: `0424903f748c0922bafaf77c2312c2ba8b820fbe9b3aad0bf276e2f37f995533`.
- Inspection compiled V3 hash: `5b7bc412c7eb13eff17f2a6195e9d70323d0144b91357a420f1912942975961c`.
- Descriptor SHA-256: `0fe0252fe8e1c63caaf323bc4c555b4efacf81765db0a542ac851e116e07cf7f`.

Commands (native SDK environment points to existing cache profile
`f259d3da157cc6120b378b53ee14c10805be89698242b03d7417f699ca711c3b`):

```sh
cargo test -p next_motor --features physx-sdk --lib
cargo clippy -p next_motor --features physx-sdk --all-targets -- -D warnings
cargo run -p next_motor --example export_biomechanics_body_v8
cargo run -p xtask -- boundary-scan
cargo run -p xtask -- content-package
cargo fmt --all -- --check
git diff --check
```

PASS: 144 native library tests, native all-target Clippy, descriptor export,
all six boundary checks, content-package (123 records / 64 chunks), format,
diff whitespace and local documentation links. Logs use the `-r2` suffix.
Broad host-check, play, persistence-replay and performance: not run for this
package-local opt-in body diagnostic; native regression tests remain included.

## Next action and rollback

Implement explicitly identified V8 contact, terminal and standing consumers.
The existing 6 Ns impact limit is per contact pair; do not accidentally allow
12 Ns per anatomical foot by splitting it. Preserve the anatomical-foot ceiling
across both segments and verify support continuity before a loaded trial.
Then test flat standing, loaded heel rise, release/re-contact and bounded
disturbances. Old learned weights remain incompatible. Reverting selection to
V7 is the rollback; do not mutate old profiles or declare the full body goal
complete from these kinematic checks.
