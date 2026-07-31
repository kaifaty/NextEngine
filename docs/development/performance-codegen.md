# Opt-in Thin LTO и PGO

Этот workflow проверяет code-generation candidate после алгоритмических
оптимизаций. Он не меняет default `[profile.release]`, не создаёт shipping
verdict и не закрывает B-12. Единственный fallback при неполной или неуспешной
проверке — обычный `release`.

Ограничения следуют ADR-036:

- сравнение выполняется на одном clean commit и ровно pinned Rust 1.93.0 из
  `rustc -vV`; stable 1.94.0 тоже отклоняется;
- нужны все четыре representative workload: `r2-alpha-render`,
  `r3-multiregion-streaming`, `r4-100npc`, `r5-physics-16`;
- для baseline и candidate собираются по десять clean THOTH run каждого
  scenario с profiler enabled, полными counters и одинаковыми authoritative
  roots;
- candidate считается только пригодным для отдельного shipping-решения, если
  combined p95 improvement статистически значим, а ни одна hard metric не
  ухудшилась на 2% или больше;
- `target-cpu=native`, nightly, unsafe intrinsics и BOLT-флаги запрещены.

Пока хотя бы один R2–R5 workload возвращает `NOT_RUN`, codegen experiment
останавливается. Smoke и `long-session-soak` полезны для диагностики, но не
подменяют representative comparison.

## Профили

`release-thin-lto` и `release-pgo` наследуют обычный `release`, включают Thin
LTO и один codegen unit. `release-pgo` становится PGO build только при явных
stable-rustc flags `-Cprofile-generate` или `-Cprofile-use`; без них build stamp
его отклоняет. Обычный `cargo build --release` остаётся неизменным.

Provenance сохраняет exact effective `CARGO_ENCODED_RUSTFLAGS`, `OPT_LEVEL` и
отсортированный список всех `CARGO_PROFILE_*` environment overrides. Для
baseline и Thin LTO effective rustflags обязаны быть пустыми; любой
`CARGO_PROFILE_*` override запрещён. PGO candidate допускает ровно один compact
flag `-Cprofile-use=<merged.profdata>` и связывает SHA-256 указанного файла;
`-Copt-level=0`, `-Clto=off`, `target-feature`, `target-cpu=native`, второй flag
или profile override закрывают эксперимент fail-closed. Запускайте workflow из
чистого shell без унаследованных `RUSTFLAGS`/`CARGO_PROFILE_*`.

SHA-256 merged PGO profile вычисляется и embed-ится во время build; exact profile
path также становится `rerun-if-changed` input. `stamp` и `collect` повторно
хэшируют файл и требуют равенство build-time hash, поэтому замена profdata после
сборки отклоняется до публикации provenance.

Все результаты хранятся только под `target/performance-codegen/`.

## Калибровочный layout

Baseline и candidate используют одинаковую структуру:

```text
<set>/
  r2-alpha-render/run-01/performance-report-v1.json
  ...
  r2-alpha-render/run-10/performance-report-v1.json
  r3-multiregion-streaming/run-01/performance-report-v1.json
  ...
  r4-100npc/run-10/performance-report-v1.json
  r5-physics-16/run-10/performance-report-v1.json
```

Каждый root — baseline и candidate — дополнительно содержит
`performance-codegen-build-v1.json`, созданный тем же exact binary.
Build stamp связывает SHA-256 exact executable с встроенными во время сборки
git HEAD, clean-worktree fact и полным `rustc -vV`; при stamp/collect текущий
clean checkout обязан совпасть с этим build-time identity. Поэтому старый или
заменённый binary нельзя переименовать и выдать за baseline/candidate другого
commit или toolchain.

Build-script подписывается через `cargo:rerun-if-changed` на полный NUL-safe
список tracked inputs из `git ls-files`, а не только на Git HEAD/index. Поэтому
цикл clean build → dirty tracked source build → возврат source к HEAD не может
оставить executable с устаревшим embedded clean identity.

Для каждого набора включите profiler и выполните каждый scenario десять раз:

```powershell
$env:NEXTENGINE_PERFORMANCE_PROFILER = "on"
$scenarios = @("r2-alpha-render", "r3-multiregion-streaming", "r4-100npc", "r5-physics-16")
$baseline = (Resolve-Path "target").Path + "\performance-codegen\baseline"
cargo build --release -p xtask
.\target\release\xtask.exe performance-codegen stamp --candidate-kind baseline --output $baseline
foreach ($scenario in $scenarios) {
    1..10 | ForEach-Object {
        $run = "run-{0:D2}" -f $_
        .\target\release\xtask.exe performance-codegen collect --candidate-root $baseline --candidate-kind baseline --scenario $scenario --output "$baseline\$scenario\$run"
    }
}
```

Raw baseline `performance-report-v1.json` без build stamp и hash-bound sidecar
не является допустимым входом comparator. Baseline обязан иметь профиль
`release`, plain codegen mode, отсутствие PGO profile и запрещённых flags.

Не продолжайте после `NOT_RUN`, `FAIL`, dirty-worktree diagnostic или
неполного counter/profiler report.

## Thin LTO candidate

Сначала создайте stamp именно candidate binary, затем соберите десять run:

```powershell
$candidate = (Resolve-Path "target").Path + "\performance-codegen\thin-lto"
cargo run --profile release-thin-lto -p xtask -- performance-codegen stamp --candidate-kind thin-lto --output $candidate
foreach ($scenario in $scenarios) {
    1..10 | ForEach-Object {
        $run = "run-{0:D2}" -f $_
        .\target\release-thin-lto\xtask.exe performance-codegen collect --candidate-root $candidate --candidate-kind thin-lto --scenario $scenario --output "$candidate\$scenario\$run"
    }
}
```

На Linux используется тот же layout и binary
`./target/release-thin-lto/xtask`; timing там остаётся `REPORT_ONLY` и не
заменяет THOTH authority.

## PGO candidate

Установите matching LLVM tool из pinned toolchain:

```powershell
rustup component add llvm-tools-preview --toolchain 1.93.0
```

Создайте instrumented build и прогоните все R2–R5 workload как training input.
Путь профиля должен быть абсолютным:

```powershell
$raw = (Resolve-Path "target").Path + "\performance-codegen\pgo-raw"
New-Item -ItemType Directory -Force $raw | Out-Null
$env:RUSTFLAGS = "-Cprofile-generate=$raw"
cargo build --profile release-pgo -p xtask
foreach ($scenario in $scenarios) {
    .\target\release-pgo\xtask.exe performance --scenario $scenario --mode report --target ref-win-thoth-v1 --output "target\performance-codegen\pgo-training\$scenario"
}
Remove-Item Env:RUSTFLAGS
```

Merge выполняется matching `llvm-profdata` из Rust sysroot. Helper принимает
только direct regular `.profraw` files и публикует output атомарно:

```powershell
$merged = (Resolve-Path "target").Path + "\performance-codegen\pgo\merged.profdata"
cargo run --release -p xtask -- performance-codegen pgo-merge --profiles $raw --output $merged
```

Затем создайте optimized binary, stamp и десять candidate run каждого
scenario:

```powershell
$env:RUSTFLAGS = "-Cprofile-use=$merged"
cargo build --profile release-pgo -p xtask
Remove-Item Env:RUSTFLAGS
$candidate = (Resolve-Path "target").Path + "\performance-codegen\pgo-candidate"
.\target\release-pgo\xtask.exe performance-codegen stamp --candidate-kind pgo --output $candidate
foreach ($scenario in $scenarios) {
    1..10 | ForEach-Object {
        $run = "run-{0:D2}" -f $_
        .\target\release-pgo\xtask.exe performance-codegen collect --candidate-root $candidate --candidate-kind pgo --scenario $scenario --output "$candidate\$scenario\$run"
    }
}
```

Для POSIX shell используются эквивалентные `export RUSTFLAGS=...`,
`unset RUSTFLAGS` и `./target/release-pgo/xtask`.

`collect` запускает `performance` дочерним процессом того же exact baseline или
candidate binary и добавляет hash-bound `performance-codegen-run-v1.json`.
Comparator отклоняет report любой стороны без sidecar, с другим build stamp или
с изменёнными bytes.

## Report-only comparison

```powershell
cargo run --release -p xtask -- performance-codegen compare `
  --baseline target\performance-codegen\baseline `
  --candidate target\performance-codegen\thin-lto `
  --candidate-kind thin-lto `
  --output target\performance-codegen\thin-lto-comparison
```

Для PGO поменяйте candidate path и `--candidate-kind pgo`. Команда проверяет
baseline и candidate build stamps, exact executable/profile, clean commit,
pinned toolchain, THOTH fingerprint, preflight, profiler/counters, methodology,
metric sets и authoritative roots.
Она сохраняет deterministic bootstrap interval и basis-point changes в
`performance-codegen-comparison-v1.json`. Для каждой metric используются все
raw samples при вычислении p95 каждого run, затем сравниваются медианы десяти
независимых run-p95; deterministic bootstrap resamples именно эти десять run,
а не отдельные correlated frame/tick samples. Combined confidence interval не
усредняет marginal metric intervals: каждая bootstrap iteration отдельно
resamples baseline/candidate run indices внутри каждого scenario, применяет один
и тот же index vector ко всем metrics этого scenario, затем вычисляет combined
mean. Так межметрическая корреляция одного run сохраняется.

`eligible_for_separate_shipping_decision = true` — только основание обсудить
перенос candidate-настроек в shipping profile отдельным изменением. Сам
workflow никогда не изменяет `[profile.release]`. Любой `NOT_RUN`, отсутствие
значимого combined improvement или regression `>=2%` оставляет обычный
`release` единственным shipping fallback.
