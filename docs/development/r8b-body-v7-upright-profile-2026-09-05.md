# V7 body and upright reference implementation

Implements [ADR-117](../architecture/adr/117-quiet-upright-body-and-standing-reference.md)
from the reviewed [quiet nominal candidate](r8b-hip-rate-feedback-discriminator-2026-09-05.md).
This is opt-in reusable motor code, not a new learned policy or existing
environment switch. Previous source body versions and controller V1 remain.

## Identity and correspondence

- BodySchema V7: `nextengine.body.humanoid-biomechanics-raja-1700.v7`,
  hash `43d9f3e1f8291fc054767c104ef19a2712b3dbb69016c975168e960a4417e989`.
- CompiledBodySchemaV4 outer hash for subject zero:
  `5bc1bd8536cec8c9879889c2ea54840b24c07cd01b22f1b3a250ed58f496ae0b`.
- Reference: `nextengine.motor.procedural-standing-reference.v2`;
  nominal-reset state root
  `cec3d35b0f1d810f83b3c8d12310e8b4a8b1cade78d65048a3c954ac69af4047`.

V7 changes exactly ten actuator records versus V6: two shoulder-yaw stiffness
and damping pairs divided by16, eight declared coupled damping values divided
by4. The exact-body-delta test restores only identity/gains and compares the
entire schema. All masses, COM, tensors/frames, lengths, anchors, axes, physical
colliders, exclusions, material and safety remain unchanged. The example test
also compares with the original independently reviewed diagnostic construction.

The V2 controller validates the complete compiled input at construction, then
reuses V1 ankle/knee behavior and adds the exact integer hip position term.
No native velocity is replaced or filtered. Its state identity binds the
compiled profile, subject PersistentId and immutable reset anchoring. Full recompilation validation
is reset-time work, not per-substep work; there is no throughput claim.

All1801 recorded motor samples and7200 physical substeps in the new native
run equal the diagnostic candidate, including references, applied targets,
joint efforts, link poses/velocities, joint states and raw contacts. The only
output changes are explicit body/profile identity and labels. Timeout is
unchanged. Therefore the prior bounded nominal measurements transfer to this
exact native run, not to perturbations or another environment.

External root:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-v7-01/`.

| Artifact | SHA-256 |
| --- | --- |
| `standing.json` | `ffd2fb17abde39c4c69cc1382da90ef35111fc9d09ff228bf128fdea0e663941` |
| Final `standing-subject-bound.json` | `6eef44ec589dbb1b28069165fc1784e55bd5c6c81fbdf06c3c3ae5fac465c720` |
| `descriptor.json` | `18de43f49c91f69042ebfe14bfce50e626b3e02fa7d96594841da044eb3475ef` |
| Reference candidate `../hip-rate-feedback-discriminator-01/position-only.json` | `49cb7fbffe6762f47e6ae00e530f6b7d5e892d5ae5d7470a7eae5aaace4054df` |
| `unchanged-candidate.json` | `49cb7fbffe6762f47e6ae00e530f6b7d5e892d5ae5d7470a7eae5aaace4054df` |
| `unchanged-old-baseline.json` | `899de64b39cc4fbb2e5fea897100970f30e04742b72ba933f2b9e1c1a81ceef2` |

A late subject-distinction test initially failed: compiled hashes intentionally
identify reusable body profiles, not subjects. V2 now binds subject separately.
The final subject-bound trace differs from `standing.json` only in the new
reference state root; all physical data remains exact. The failing test was
not weakened and now passes. Historical pre-fix state hashdf6479c6… is not
the selected V2 state identity.

Commands (native runs require the explicit pinned SDK in task-state):

```sh
cargo run -p next_motor --features physx-sdk --example probe_biomechanics_body_standing -- 7 0 0 upright-v2 per-iteration unchanged
cargo run -p next_motor --example export_biomechanics_body_v7
```

The inspection descriptor uses `nextengine.canonical-upright-body-inspection.v1`,
includes the compiled-V4 hash, preserved legacy descriptor hash, force-schedule
and standing profile IDs. Admission remains `native-body-diagnostic-only`;
no environment profiles or Isaac admission are fabricated.

## Validation and remaining work

The final native motor suite passed all138 tests after the subject-binding
repair (`final-native-tests.log`). Five native example tests pass. Integer reference
vectors include both signs, fractional truncation, varying angular velocity,
unchanged non-hip channels, reset-state hash distinction and rejected missing /
duplicate roots. Wrong body and tampered compiled gains fail before use.
Native all-target Clippy, workspace formatting, diff whitespace and changed
documentation links pass. `play`, `persistence-replay` and `content-package`
pass; their JSON reports are in `play.log`, `replay.log` and `content.log`.

The broad `host-check` invocation completed workspace Clippy and tests but
failed its final boundary scan on the diagnostic's pre-existing `#[path]`
module declaration (`host-check.log`, exit1). Replaced it with conventional
nested `mod support { pub mod effort_response; }` resolution, without moving
or changing the helper. All six boundary-scan checks then passed; the five
native example tests, native all-target Clippy and workspace formatting were
rerun and passed after this repair. The broad wrapper was not rerun: its
recorded status remains failed/repaired with focused revalidation, not a
fabricated wrapper PASS. It started before the final subject-binding repair;
the final138 native tests cover that change. No check policy was weakened.

Performance, desktop SDL/ash and new training checks were not run: this change
does not select their runtime paths or make a throughput/learning claim.

Remaining: controlled disturbance recovery, realistic foot successor with valid
source/proxy inertia and heel-rise/re-contact checks, and separately admitted
compatible learning. No training is launched or old checkpoint reused here.
