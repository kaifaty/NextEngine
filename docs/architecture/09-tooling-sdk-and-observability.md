# SPEC-09: Current tooling and observability

| Поле | Значение |
|---|---|
| ID | SPEC-09 |
| Статус | Accepted |
| Версия | 3.7 |
| Последняя проверка | 2026-08-16 |
| Нормативные зависимости | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-036](adr/036-thoth-reference-performance-profile.md), [ADR-038](adr/038-versioned-production-worker-handoff-diagnostic.md), [ADR-045](adr/045-low-overhead-hard-performance-evidence.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-049](adr/049-performance-evidence-without-allocator-instrumentation.md), [ADR-060](adr/060-relaxed-thoth-performance-preflight.md), [ADR-061](adr/061-forty-percent-thoth-load-preflight.md), [ADR-062](adr/062-r5-physx-humanoid-performance-authority.md), [ADR-063](adr/063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md) |
| Заменяет | SPEC-09 3.6; admits the bounded current Decision Trace diagnostic projection |

## Scope and authority

Current tooling is the repository-owned `xtask` command surface plus standard
Cargo tests/lints. It runs production application/content/persistence paths and
emits bounded structured reports. Tool output, telemetry, captures and profiles
are diagnostics; they never become gameplay authority.

Subsystems own the semantics of their diagnostics and immutable projections.
Tooling owns command parsing, stable machine-readable report envelopes, local
output publication and performance evidence. Test/verification crates are not
production dependencies and receive no mutation backdoor.

There is no current public `next` CLI, MCP tool protocol, live inspector API,
`AuthoringContextBundle`, `AgentChangeSet` or agent policy contract. Those are
R6 possibilities and cannot be required by current development or runtime.

## Current command surface

Repository commands are non-interactive, accept explicit options/outputs and
return nonzero on failure. The governing product commands include:

- `cargo run -p xtask -- host-check`;
- `cargo run -p xtask -- play`;
- `cargo run -p xtask -- persistence-replay`;
- `cargo run -p xtask -- content-package`;
- `cargo run -p xtask -- platform`;
- `cargo run -p xtask -- v1-closure` and `v1-package`;
- `cargo run -p xtask -- physics-collision`/backend parity commands;
- `cargo run --release -p xtask -- performance ...`;
- `cargo run --release -p xtask -- performance-baseline ...`.

`host-check` is the canonical fast local handoff check and combines formatting,
clippy, workspace tests and boundary scanning. Product-specific commands select
their production scenario; they do not prove unrelated roadmap stages.

Operational JSON stdout must decode as exactly one versioned command/report
object where the command promises JSON. Progress belongs on stderr. Stable
codes/typed fields are the oracle, not rendered text or unordered logs.

## Diagnostics and output safety

Diagnostics carry a version, stable code, severity, subsystem, message key,
build/process identity and relevant tick/stage/stable IDs, expected/actual
hashes or first divergence. Native/vendor details may be an optional debug
field but cannot replace the engine-owned code.

Outputs use private staging followed by validation and atomic publication.
Crash, disk-full, hash mismatch or unsupported input leaves the previous
complete output untouched. Generated reports, captures, profiles, caches and
machine-local paths are not committed. Secrets, prompts, voices, protected
assets and raw user paths are redacted or excluded.

Pre-v1 reports and tool-owned formats are current-only unless an ADR names a
public support promise. A retired version returns a typed unsupported result;
tooling does not mutate or delete the input.

## Observability boundary

Profiling and telemetry are optional and bounded. CPU spans, Vulkan timestamps,
process/resource counters and diagnostics may observe a run but cannot block a
fixed tick or choose command/world outcome. Running with profiling disabled or
enabled must preserve authoritative state and ledger roots.

External profilers and OS APIs remain private adapters. Overflow, dropped
required samples or an unowned gameplay span invalidates the affected evidence
rather than hiding loss. Screenshots/video/audio are optional human debugging
evidence, not correctness or release authority.

## Performance V5

Current tooling serializes `PerformanceRunV5`,
`PerformanceResourceCountersV4`, `PerformanceMetricV1`,
`PerformanceBaselineV5` and the closed verdict. The methodology ID is
`nextengine-performance-v8`; V2/V3/V4 readers and allocator instrumentation
are removed.

V5 retains:

- all raw samples, explicit independent-run lengths and nearest-rank
  per-run p50/p95/p99;
- compatible methodology/workload/fingerprint hashes;
- canonical logical charges;
- process peak working set and process I/O;
- conservative engine-owned device allocation ceiling;
- Vulkan timestamp accounting where relevant;
- profiler integrity and exact authoritative roots.

Unavailable required counters are explicit `NOT_RUN`; report-only evidence
cannot become a hard PASS. A baseline consists of exactly ten clean release
reports with one independent run each. One hard-gate command executes a fixed
three-run batch and publishes one aggregate V5 report. Relative statistics use
per-run p95 observations; raw frames are never treated as independent runs.
Only complete THOTH evidence with a compatible V5 baseline may produce hard
timing PASS. B-12 remains open.

THOTH preflight admits CPU and GPU load strictly below 40% and at least 10 GiB
free physical RAM before every independent run. CPU clock and GPU thermal
checks remain unchanged. Postflight checks free RAM, clock and thermal state,
while retaining but not idle-gating utilization caused by the workload itself.
Missing boundary evidence, 40% start load or less than 10 GiB free RAM
invalidates hard evidence under ADR-061/063.

The seven current scenario families are:

1. smoke;
2. long-session;
3. interactive-frame;
4. production-worker;
5. `r2-alpha-render`;
6. `r3-multiregion-streaming`;
7. `r5-physics-16`.

Each uses its current V5 methodology hash and existing workload semantics.
`r2-alpha-render` runs exploration, combat and UI/dialogue for primary 1080p
and fallback 720p profiles through the production project/Vulkan path. Report
mode remains `REPORT_ONLY` without clean THOTH preflight/baseline.

`r3-multiregion-streaming` runs 1,000 canonical transitions across the current
four-region/64-chunk route, reports the `streaming_world` span and logical
staging charge, and remains `REPORT_ONLY`. It does not imply a generic
scheduler/resource framework or close B-12.

`r5-physics-16.v1` executes sixteen independent production PhysX 5.9.0
23-DoF humanoids at 240 Hz physics / 60 Hz motor with fixed standing control.
It reports direct substeps/s and motor-frames/s plus lower-is-better reciprocal
cost metrics, lockstep frame p95/p99, 1/4/8-worker scaling, checkpoint/restore,
replay-prefix overhead, process peak memory, logical bytes/slot and exact
worker/profiler root parity. CPU PhysX reports zero engine-owned device
residency and does not fabricate Vulkan queries. Exact workload and budgets are
ADR-062 authority. Until ten compatible clean runs and a hard gate exist, its
clean calibration remains `REPORT_ONLY` and does not close B-12.
Calibration automation uses `--require-ready-preflight`: if the exact internal
host probe is not ready, the command publishes typed `NOT_RUN` before starting
the representative workload. The flag does not wait, relax thresholds or
retry a completed workload; it only prevents known-invalid busy-host samples.

## Failure semantics and checks

Unknown command/input format, corrupt report, incompatible baseline, atomic
write failure or internal tool crash returns a stable failure and publishes no
partial output. Telemetry exporter absence disables/drops bounded optional data
without affecting gameplay. A deterministic retry mismatch is
`NONDETERMINISTIC_RESULT`, never retry-to-green.

Focused tooling tests cover command parsing, exact report schemas, atomic
output, current-only rejection and boundary scan. `host-check` covers the
workspace. The `performance` command covers V4 reports/baselines and all seven
scenario routes; platform/GPU availability may legitimately yield typed
`NOT_RUN` without claiming success for that scenario.

## Strategic Agent explainability

SPEC-32/ADR-073 admits bounded immutable `DecisionTraceV1` containing candidate
goal scores, switch reason, cited belief IDs, selected semantic plan, planning
failure and intent ID. The production reference scenario exposes it as a
reproducible diagnostic projection, but it is not persisted, hashed as an
owner, accepted as gameplay input or promoted to mutable inspector/generic UI
authority. Prompts, secrets, generated transcripts and unbounded memory
payloads remain excluded. R4d may extend diagnostic reasons only with its own
consumer and bounded contract.
