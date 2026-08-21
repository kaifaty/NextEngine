# ADR-091: Linux release performance authority

| Поле | Значение |
|---|---|
| ID | ADR-091 |
| Статус | Accepted |
| Версия | 1.2 |
| Дата решения | 2026-08-21 |
| Последняя проверка | 2026-08-21 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-04](../04-rendering-and-platform.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-29](../29-platform-host-and-application-session.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-045](045-low-overhead-hard-performance-evidence.md), [ADR-049](049-performance-evidence-without-allocator-instrumentation.md), [ADR-062](062-r5-physx-humanoid-performance-authority.md), [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-090](090-linux-only-v1-and-indefinitely-deferred-windows.md) |
| Заменяет | Linux `REPORT_ONLY` clauses of ADR-036/062/063/090 for the exact profile below; historical Windows/THOTH authority remains outside current scope. [ADR-092](092-dimensional-relative-performance-comparison.md) narrowly supersedes generic relative comparison for normalized R5 ratios and advances methodology to v10. |

## Контекст

ADR-090 сделал Linux единственной v1/R7 release target, но намеренно не
перенёс на неё THOTH fingerprint, preflight или budgets. R7c поэтому не может
закрыться существующим Linux report либо переименованием Windows baseline.

Диагностический аудит 2026-08-21 подтвердил, что текущий host probe стабильно
видит Ryzen 9 3950X, RTX 3080 10 GiB, 32 GiB-class RAM, performance governor,
Ubuntu 26.04/kernel 7.0.0-29 и NVIDIA 610.43.02. R3 и R5 завершили exact
workloads с чистым environment evidence. R4 сохранил exact roots и zero
drop/starvation, но превысил canonical ADR-016 navigation/integrated rows; это
optimization blocker, а не основание расширить budgets. R2 fail-closed до
измерения, потому что текущий GNOME session не публиковал ни одного display.

## Решение

### Exact profile

Current Linux release performance profile имеет ID
`ref-linux-b550i-3950x-rtx3080-v1` и обязан совпасть со следующей границей:

| Field | Accepted value |
|---|---|
| Hostname | `kaifaty-B550I-AORUS-PRO-AX` |
| CPU | `AMD Ryzen 9 3950X 16-Core Processor`, 16 physical / 32 logical |
| GPU | `NVIDIA GeForce RTX 3080`, 10,240 MiB |
| RAM | не меньше 30 GiB visible physical memory |
| Root storage | `SAMSUNG MZVL2512HCJQ-00BH1`, не меньше 512,000,000,000 bytes |
| OS | `Linux (Ubuntu 26.04 LTS)` |
| OS build | `26.04; kernel 7.0.0-29-generic` |
| BIOS | `F16e` |
| NVIDIA driver | `610.43.02` |
| CPU policy | `linux-governor:performance` |
| Rust host | pinned `rustc 1.97.1`, `x86_64-unknown-linux-gnu` |

Baseline и gate additionally require byte-for-byte equal fingerprints, so an
otherwise admitted lower-bound field cannot drift inside one evidence set.
Kernel, BIOS, driver, governor, storage or hostname change invalidates
compatibility and requires a fresh Accepted profile revision before release
evidence is recollected.

### Linux preflight

The Linux profile independently accepts these start thresholds:

- CPU and GPU load each strictly below 40%;
- at least 10 GiB available physical RAM;
- observed CPU clock at least 80% of the maximum;
- NVIDIA thermal slowdown explicitly inactive.

Postflight rechecks RAM, clock and thermal state without idle-gating utilization
created by the workload itself. Missing evidence is `NOT_RUN`. The numeric
values intentionally match the final historical THOTH thresholds, but this ADR
accepts them from Linux diagnostics and workload footprint; they are not
inherited authority.

### Representative numeric policy

All durations are integer microseconds and use nearest-rank per independent run
with all outliers retained.

| Workload / hard metric | p95 max | p99 max |
|---|---:|---:|
| R2 primary 1080p critical path, each exploration/combat/UI window | 14,000 | 16,670 |
| R2 safe 720p30 critical path, each exploration/combat/UI window | none | 33,330 |
| R3 1,000-transition total | 1,500,000 | 1,500,000 |
| R4 navigation due work | 1,250 | 1,500 |
| R4 tier-cognition due work | 1,250 | 1,500 |
| R4 integrated world-services tick | 8,000 | 12,000 |

R2 deadlines remain player-visible presentation constraints from SPEC-04.
R4 retains the canonical 30 Hz compositional matrix from ADR-016: the Linux
implementation must be optimized without changing authoritative outcomes.
R3's complete 1,000-transition ceiling bounds the production two-worker
packaged-I/O route to 1.5 ms average per transition while retaining a single
whole-workload observation per independent run.

R5 independently accepts every absolute row currently enumerated by ADR-062
and the executable `r5-physics-16` policy: 1/4/8-worker motor-frame and
reciprocal costs, 4/8-worker scaling inefficiency, fresh-scene restore,
checkpoint bytes, replay-prefix overhead, process peak working set and logical
host bytes per slot. Their values do not change. The 2026-08-21 Linux
diagnostic completed every row with headroom, exact worker roots and clean
environment evidence.

Only metrics with an explicit canonical absolute budget participate in hard
absolute/relative verdicts. Auxiliary phase timings, counters, queue depths and
sizes remain preserved in the strict report/baseline and must retain compatible
shape, but cannot fail or pass a gate through an accidental lower-is-better
interpretation. Exact roots, required resource counters and workload-specific
closure invariants remain mandatory independently of metric verdicts.

### Performance V6 / methodology v10

Current evidence advances to strict `PerformanceRunV6`,
`PerformanceBaselineV6`, `performance-report-v6.json`,
`performance-baseline-v6.json` and methodology
`nextengine-performance-v10`.

V6 retains ADR-063 semantics:

- ten clean single-run release reports form one baseline;
- one gate command executes exactly three independent runs;
- p50 is the median of run p50s; p95/p99 are the worst per-run tails;
- relative evidence uses run-level p95 values and deterministic 2,000-iteration
  bootstrap intervals;
- `>=5%` regression with a `>=5%` lower confidence bound fails, `>=2%` warns;
- workload/content/methodology/toolchain/fingerprint/root/metric-policy drift
  is incompatible rather than comparable;
- no completed calibration or gate run is retried to obtain a greener sample.

V6 additionally stores the canonical absolute budget beside every baseline
metric and validates the scenario's full hard/diagnostic policy before baseline
publication or gate comparison. V5 remains historical evidence only and is not
decoded as current Linux authority.

ADR-092 retains all absolute rows and whole-run relative comparison for direct
durations, reciprocal costs and bytes. The already normalized R5 scaling and
replay-overhead ratios are absolute-only to avoid a second percentage over a
changing denominator; their samples remain mandatory hard evidence.

Each of the three fixed gate members executes in a fresh process of the same
release binary. This makes gate members operationally symmetric with the ten
fresh-process calibration reports and prevents cumulative process high-water,
allocator or vendor-runtime state from leaking across independent evidence
units. The parent process alone aggregates the three strict reports; it cannot
drop, select or rerun a member.

## Failure semantics

- Wrong target ID, fingerprint, target triple, build profile, dirty worktree,
  missing baseline or unavailable preflight is typed `NOT_RUN` before workload
  execution.
- R2 without a real X11/Wayland display and production Vulkan adapter is
  `NOT_RUN`; a virtual/software display cannot close the release criterion.
- Missing PhysX support makes R5 `NOT_RUN`; no reference backend substitutes.
- An absolute or significant relative regression is `FAIL` and remains exact
  negative evidence. Quality may change only through an already declared
  non-authoritative/LOD fallback; authoritative roots may not change.
- Windows/THOTH execution is neither required nor scheduled.

## Product checks

| Check | Expected | Fallback |
|---|---|---|
| Profile/failure focused tests | Exact Linux host accepted; field drift, THOTH ID, old schema, missing/changed budgets and report-only scenarios fail closed | Keep R7c open |
| Ten report runs per R2–R5 + baseline publication | Same clean commit, fingerprint, toolchain, workload/content/methodology, roots and metric policy; all pre/postflight valid | Preserve failed set; fix cause on a new commit/set |
| Fixed three-run gate per R2–R5 | All absolute and relative hard metrics pass, exact roots remain unchanged | Preserve result; optimize or retain blocker |
| R2 desktop prerequisite | One real display and production NVIDIA Vulkan path | `NOT_RUN`; do not use virtual/software rendering |

## Рассмотренные варианты

- **Reuse THOTH V5 unchanged.** Отклонено: target and OS authority differ and
  ADR-090 explicitly forbids relabelling.
- **Relax R4 budgets to observed Linux tails.** Отклонено: the canonical 30 Hz
  product matrix would be weakened to fit the implementation.
- **Use relative-only R3 evidence.** Отклонено: a release gate needs an
  absolute product ceiling and must pass before a baseline can normalize it.
- **Require budgets for every diagnostic counter.** Отклонено: cache hits,
  queue depths and phase counters do not all share lower-is-better semantics;
  accidental numeric comparison would create false gates.

## Последствия

- R7c implementation may now replace THOTH-only current tooling with V6/v10
  Linux authority and collect fresh evidence.
- R4 optimization and a real desktop display remain concrete prerequisites for
  a complete R7c PASS.
- A future hardware/OS/driver upgrade requires a new Accepted profile revision
  and fresh baselines/gates, not an edit to old evidence.
- Windows remains indefinitely out of scope under ADR-090.
