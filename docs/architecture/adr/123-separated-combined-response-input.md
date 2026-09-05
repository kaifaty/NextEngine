# ADR-123: Separated combined-response input diagnostic

| Field | Value |
| --- | --- |
| ID | ADR-123 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-06 |
| Dependencies | SPEC-35, ADR-122 |
| Supersedes | ADR-122 diagnostic CLI input set only, additively; original census unchanged |
| Superseded by | ADR-124 additive two-second-return input; original case27 unchanged |

Add explicit `abduction-step` mode to `probe_native_joint_bandwidth` for exact
body8 or11. It emits BODY-COMBINED-01.r1 case27 as one separate world, with both
hip-roll step signs reversed and every other input/fixture/safety path unchanged.
No-flag invocations retain the original27-world BODY-BANDWIDTH-01.r2 bytes.
No arbitrary gains, input vectors, collision filters or standing admission.

The [frozen contract](../../development/r8b-combined-input-calibration-2026-09-06.md)
requires all eligible collider-pair sampled path clearance before native motion.
This is a finite input preflight, not a continuous/native collision guarantee.
The original all-positive step is retained as a failed regression, not relabeled.
Successful combined response does not select a training body or prove loaded
support; those consumers remain subject to explicit compatible admission.

Require exact two-sign mapping and strict CLI tests, original no-flag byte
regression, native V8/V11 controls and repeat, independent executable review,
native example tests/Clippy, format/diff and boundary/content checks. Library,
game, standing and physics sources are unchanged; previous156 native library
tests remain exact-base evidence. Host/play/replay/performance are not triggered
by this isolated input-only example extension.
