# SPEC-09: Current tooling and observability

| Поле | Значение |
|---|---|
| ID | SPEC-09 |
| Статус | Accepted |
| Версия | 5.4 |
| Последняя проверка | 2026-08-21 |
| Нормативные зависимости | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-036](adr/036-thoth-reference-performance-profile.md), [ADR-038](adr/038-versioned-production-worker-handoff-diagnostic.md), [ADR-045](adr/045-low-overhead-hard-performance-evidence.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-049](adr/049-performance-evidence-without-allocator-instrumentation.md), [ADR-060](adr/060-relaxed-thoth-performance-preflight.md), [ADR-061](adr/061-forty-percent-thoth-load-preflight.md), [ADR-062](adr/062-r5-physx-humanoid-performance-authority.md), [ADR-063](adr/063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md), [ADR-082](adr/082-linux-first-development-and-deferred-windows-host.md), [ADR-083](adr/083-public-creator-project-cli-vertical.md), [ADR-084](adr/084-public-creator-run-and-project-package-vertical.md), [ADR-085](adr/085-public-creator-project-inspect-and-diff-vertical.md), [ADR-086](adr/086-public-creator-rpg-starter-template.md) |
| Дополнительные зависимости V4.7 | [ADR-087](adr/087-public-creator-runtime-scenario-and-prefix-minimization.md) |
| Дополнительные зависимости V4.8 | [ADR-088](adr/088-public-replay-first-divergence-and-domain-inspection.md) |
| Дополнительные зависимости V4.9 | [ADR-089](adr/089-governed-external-creator-sdk-workflow.md) |
| Дополнительные зависимости V5.0 | [ADR-090](adr/090-linux-only-v1-and-indefinitely-deferred-windows.md) |
| Дополнительные зависимости V5.1 | [ADR-091](adr/091-linux-release-performance-authority.md) |
| Дополнительные зависимости V5.3 | [ADR-092](adr/092-dimensional-relative-performance-comparison.md) |
| Дополнительные зависимости V5.4 | [ADR-093](adr/093-deterministic-r5-worker-placement.md) |
| Заменяет | SPEC-09 5.3; adds deterministic physical-core placement for the R5 performance workload under ADR-093 |

## Scope and authority

Current tooling has two distinct surfaces: repository-owned `xtask` plus
standard Cargo tests/lints, and the bounded public creator binary `next` from
ADR-083/084/085/086/087/088. Both run production application/content/persistence paths and emit
bounded structured reports. Tool output, telemetry, captures and profiles are
diagnostics; they never become gameplay authority.

Subsystems own the semantics of their diagnostics and immutable projections.
Tooling owns command parsing, stable machine-readable report envelopes, local
output publication and performance evidence. Test/verification crates are not
production dependencies and receive no mutation backdoor.

The current public `next` commands are project create, validate, cook, run,
package, inspect and diff, bounded scenario validate/run/minimize and Replay
V10 validate/inspect below.
The canonical source-build workflow is [Creator SDK beta](../creator-sdk.md).
There is no current MCP tool protocol, public capture command, live
inspector API, `AuthoringContextBundle`, `AgentChangeSet` or agent policy
contract. Those remain later consumer-driven R6 possibilities and cannot be
required by current runtime.

## Current command surface

Repository commands are non-interactive, accept explicit options/outputs and
return nonzero on failure. The governing product commands include:

- `cargo run -p xtask -- host-check`;
- `cargo run --locked --release -p xtask -- animation-lod`;
- `cargo run --locked --release -p xtask -- animation-root-motion`;
- `cargo run --locked -p xtask -- physical-character`;
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

`animation-root-motion` is the fixed milestone-specific
`ANIM-ROOT-MOTION-P1` check. It runs exactly 10,000 production-path cycles in
1,000 isolated deterministic generations and emits one strict V1 command
report. It is not folded into every `host-check`: the workspace carries a
30-cycle production smoke, while the full release-mode matrix runs when the
root-motion gate or its implementation is changed.

`animation-lod` is the fixed milestone-specific `ANIM-LOD-P1` check for the
current bounded R5 profile. It runs exactly 10,000 production-path
LOD/cadence/resource/publication transitions in 1,000 isolated generations and
emits one strict V1 report. The workspace carries a deterministic one-block
smoke; the full release-mode matrix runs when the LOD planner, animation
projection or atomic character publication boundary changes.

`physical-character` is the fixed milestone-specific procedural `PHYS-P6`
check. It uses the production reference catalog and gameplay route to cover a
named low-riser trip contact/traversal, a compound carried-load clearance
contact with positive capsule clearance, exact checkpoint continuation while
blocked, and the existing committed-contact → Mechanics → RPG melee outcome.
It executes two complete generations and emits one strict V1 report with exact
roots and a matrix digest. It does not benchmark or promote PhysX Stage 0,
articulation, learned control, general grab/drop or Windows evidence.

The active desktop performance route is:

```text
cargo run --locked --release -p xtask --features desktop-sdl-ash -- performance --scenario r2-alpha-render --mode report
```

Windows/THOTH commands remain implemented as dormant utilities, but ADR-090
places them outside current v1/R7 scope indefinitely. They are not part of the
Linux handoff or release loop and create no active backlog.

The current public creator commands are:

```text
next project create --template rpg-starter --project-id <namespaced-id> --output <new-project-directory>
next project validate --project <project-directory>
next project cook --project <project-directory> --output <content-store-directory>
next project run --project <project-directory>
next project run --package <creator-package-directory>
next project package --project <project-directory> --output <new-directory>
next project inspect --project <project-directory>
next project inspect --package <creator-package-directory>
next project diff (--base-project <directory> | --base-package <directory>) (--candidate-project <directory> | --candidate-package <directory>)
next scenario validate --scenario <file> (--project <directory> | --package <directory>)
next scenario run --scenario <file> (--project <directory> | --package <directory>)
next scenario minimize --scenario <file> (--project <directory> | --package <directory>) --output <absent-file>
next replay validate --replay <file> (--project <directory> | --package <directory> | --content-store <directory>)
next replay inspect --replay <file> (--project <directory> | --package <directory> | --content-store <directory>) --tick <u64> --domain <runtime|world-services|physics|owners>
```

The workspace source-build spelling is `cargo run --locked -p next_cli --`
followed by the same arguments. Create renders the one built-in current-only
RPG starter into an absent directory, namespaces its project-owned identifiers,
and validates/cooks it before atomic publication. All subsequent commands use the project-declared identity
and the production V7 loader/cooker. Cook additionally publishes through
`ContentStore` and reopens through production activation. Run crosses generic
Runtime/Application Session plus final save-on-close for one bounded headless
tick. Package publishes a current-only content distribution envelope, validates
its exact inventory/NOTICE/roots/run proof and can be launched again with the
package run spelling. Inspect produces one source-neutral immutable projection
of exact roots, schemas, stable-ID assets/dependencies/world chunks and locked
mechanics capabilities. Diff compares base→candidate keyed projections and
reports valid drift as `PASS` with `different = true`. Project-ID override,
implicit migration and reference-game fixture construction are not CLI paths.

Operational JSON stdout must decode as exactly one versioned command/report
object where the command promises JSON. Progress belongs on stderr. Stable
codes/typed fields are the oracle, not rendered text or unordered logs.
Creator Command Report V1 success exposes only exact project hashes and bounded
counts. Creator Run Report V1 and Creator Package Report V1 are separate
contracts carrying project identity plus runtime/final-save or package proof.
Creator Inspect Report V1 and Creator Diff Report V1 are separately versioned,
path-free read-only contracts over `CreatorProjectProjectionV1`. All failures
expose stable code/subsystem/message key without raw paths.
Creator Scenario Report V1 is a sixth independent family carrying exact
scenario/project identity, final runtime proof or preserved prefix-minimization
failure. It does not add fields to earlier report families.
Creator Replay Report V1 is a seventh independent family. Validate binds the
current canonical Replay V10 to an exact activated project without execution;
inspect completes production replay before exposing exactly one requested
tick/domain. Divergence reports the first tick, stable stage and owner.

## Creator SDK beta workflow

The external workflow begins with an absent directory and the built-in
`rpg-starter`. Its generated Project Authoring V7 contains the minimum current
NPC/ability/quest and three-chunk closure. A creator edits only the public JSON
source, then uses public validate, cook, run, package, inspect and diff; no
engine crate, fixture constructor or identity override participates.

The governing cold exercise changes one NPC role, ability ID, quest
entry/source and streamed chunk/region, requires the project lock to change,
and then requires public validate/cook to agree with the production cooker.
The edited authoring and exact package must run and inspect identically, with
an empty authoring-to-package diff. `content-package` executes this complete
sequence in scratch storage.

The exact Luau scripted-melee and Wasm Component sources executed by the same
governing check live under `examples/creator-sdk/`; their bytes and package
identities are unchanged from the prior embedded fixtures. Current WIT V3
remains the host interface. Arbitrary project-local Luau/Wasm ingestion, GUI,
MCP, live mutation, replay capture and alpha migration remain outside this
bounded workflow until a concrete consumer admits them.

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

Creator project references are confined to the explicitly selected project
root after symlink resolution. Creator cook accepts only a new/empty or
recognizable ContentStore root; symlink and unrelated nonempty outputs fail
with `CREATOR_OUTPUT_INVALID` before publication.

Creator template output must also be absent below an existing real parent. The
fixed four-file starter is rendered into a private sibling, loaded and cooked
through Project Authoring V7, then renamed once. It never accepts an arbitrary
template path, follows a remote source, merges with caller files or overrides
the identity of existing authoring.

Creator package output must be absent. The command stages beside the resolved
destination, validates and runs the staged bytes, then publishes one complete
directory. It never merges or overwrites. Package inventory paths, sizes and
hashes are exact and bounded; links, unknown files, traversal, missing/empty
root `NOTICE`, retired format or changed bytes fail closed. This envelope is
for a compatible installed `next` runtime and is not the native R7 game bundle.

Inspect never writes caller output. Package operands first pass the complete
inventory/NOTICE/activation check and recorded-run rerun; authoring operands
pass the current loader/cooker. The projection excludes paths, source spans,
content property values and private generation layout. Diff compares only
validated complete operands, is stable-ID keyed and never emits a partial
comparison after one operand fails.

Creator scenario source is one regular non-link file bounded to 1 MiB and
current format `nextengine.creator-runtime-scenario.v1`. It binds a complete
project identity, 1–256 ordered tick actions, an explicit tick budget and 1–32
sorted exact read-only assertions. Run uses isolated headless state and ordinary
save-on-close; minimize writes only an absent regular output after a private
sibling candidate validates and reproduces the same assertion category, ID and
probe. It never weakens assertions, overwrites output, emits private paths or
accepts arbitrary command, fault or capture payloads.

Public replay source is one regular non-link canonical Replay V10 file bounded
to 16 MiB and 1–4,096 ticks. The exact project compatibility block is checked
before restore. Inspect returns only hashes, stable IDs and bounded counts for
one tick in `runtime`, `world-services`, `physics` or `owners`; it never emits
canonical owner bytes, private snapshots, paths or a partial projection after
divergence. Replay V9 and earlier are rejected before nested decode.

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

## Performance V6

Current tooling serializes `PerformanceRunV6`,
`PerformanceResourceCountersV4`, `PerformanceMetricV1`,
`PerformanceBaselineV6` with budget-bearing baseline metric V2 and the closed
verdict. The methodology ID is `nextengine-performance-v10`; v9 and older
readers plus allocator instrumentation are removed from the current path.

V6 retains:

- all raw samples, explicit independent-run lengths and nearest-rank
  per-run p50/p95/p99;
- compatible methodology/workload/fingerprint hashes;
- canonical logical charges;
- process peak working set and process I/O;
- conservative engine-owned device allocation ceiling;
- Vulkan timestamp accounting where relevant;
- profiler integrity and exact authoritative roots.

Unavailable required counters are explicit `NOT_RUN`; report-only evidence
cannot become a hard PASS. Only metrics enumerated by the canonical scenario
policy carry an absolute budget and participate in absolute/relative verdicts;
diagnostic counters remain strict evidence without accidental
lower-is-better gating. A baseline consists of exactly ten clean release
reports with one independent run each. One hard-gate command executes a fixed
three-run batch and publishes one aggregate V6 report. Relative statistics use
per-run p95 observations; raw frames are never treated as independent runs.
Only complete evidence from `ref-linux-b550i-3950x-rtx3080-v1` with a
compatible V6 baseline may produce the current hard verdict. Historical
THOTH/V5 evidence remains outside current v1/R7 authority.

ADR-091 Linux preflight admits CPU and GPU load strictly below 40% and at least
10 GiB free physical RAM before every independent run. CPU clock must be at
least 80% of maximum and GPU thermal slowdown must be false. Postflight checks
free RAM, clock and thermal state while retaining but not idle-gating workload
utilization. Missing boundary evidence invalidates hard evidence.

The eight current scenario families are:

1. smoke;
2. long-session;
3. interactive-frame;
4. production-worker;
5. `r2-alpha-render`;
6. `r3-multiregion-streaming`;
7. `r4-100npc`;
8. `r5-physics-16`.

Each uses its current V6/v10 methodology hash and existing workload semantics.
`r2-alpha-render.v3` runs exploration, combat and UI/dialogue for primary 1080p
and fallback 720p profiles through the production project/Vulkan path on the
active Linux desktop host. Report mode remains `REPORT_ONLY`; it records
Linux-native peak RSS/process I/O plus Vulkan device/timestamp evidence but
does not require or synthesize a THOTH baseline.

`r3-multiregion-streaming` runs 1,000 canonical transitions across the current
four-region/64-chunk route, reports the `streaming_world` span and logical
staging charge, and has a hard 1,500,000 us whole-workload p95/p99 ceiling. It
does not imply a generic scheduler/resource framework.

`r4-100npc.v1` runs 1,000 warm-up and 10,000 measured production ticks over
the exact 16/32/52 population, one graph query per due record and the four
tier-cognition work kinds. It publishes due/work counters, queue bounds,
zero-fabrication evidence and exact activity/Agent/application/ledger roots.
Report mode remains diagnostic. Gate mode enforces the unchanged ADR-016
navigation `1,250/1,500 us`, cognition `1,250/1,500 us` and integrated
`8,000/12,000 us` p95/p99 rows on the exact ADR-091 Linux host.

`r5-physics-16.v1` executes sixteen independent production PhysX 5.9.0
23-DoF humanoids at 240 Hz physics / 60 Hz motor with fixed standing control.
It reports direct substeps/s and motor-frames/s plus lower-is-better reciprocal
cost metrics, lockstep frame p95/p99, 1/4/8-worker scaling, checkpoint/restore,
replay-prefix overhead, process peak memory, logical bytes/slot and exact
worker/profiler root parity. CPU PhysX reports zero engine-owned device
residency and does not fabricate Vulkan queries. Exact workload and budgets are
ADR-062 workload identity and rows are accepted independently for the ADR-091
Linux profile. Windows execution is neither required nor scheduled.
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
output, current-only rejection, creator project-root/output confinement,
deterministic fresh RPG starter creation, location-independent repeated
inspect, an edited cold starter through public validate/cook/run/package,
authoring/package empty diff, localized record drift, repeated
three-tick scenario proof, assertion failure, shortest-prefix minimization and
boundary scan. `content-package` reopens both the reference project and the independent
creator fixture, generates and edits another namespaced project, executes its
complete public lifecycle, and compares source/package projections; it also runs/minimizes the tracked creator scenario
from authoring/package bytes. `host-check` covers the workspace. The
`performance` command covers V6 reports/baselines and all eight
scenario routes; platform/GPU availability may legitimately yield typed
`NOT_RUN` without claiming success for that scenario.

## Strategic Agent explainability

SPEC-32/ADR-073/074 admits bounded immutable `DecisionTraceV1` containing candidate
goal scores, switch reason, cited belief IDs, selected semantic plan, planning
failure and intent ID. The production reference scenario exposes it as a
reproducible diagnostic projection, but it is not persisted, hashed as an
owner, accepted as gameplay input or promoted to mutable inspector/generic UI
authority. Prompts, secrets, generated transcripts and unbounded memory
payloads remain excluded. R4d systemic intent/failure reasons are current only
for the bounded ADR-074 consumer.
