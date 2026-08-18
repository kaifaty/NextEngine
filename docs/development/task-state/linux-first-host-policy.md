# Linux-first host policy — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-18 |
| Task key | `linux-first-host-policy` |
| Scope | Make native Linux x86_64 the active development host, execute R2 renderer evidence there, and defer Windows/THOTH work without removing Windows from v1 shipping targets |
| Definition of done | Accepted policy, specs, routing, roadmap and developer guidance agree; R2 executes through SDL3/ash on Linux with native process counters; mapped Linux checks pass; deferred Windows work has one bounded backlog |
| Authority | Working context only; ADR-082, Accepted SPECs, `docs/roadmap.md` and exact check outputs outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Linux is the only active development host. Windows,
  THOTH hard timing and same-commit compare are deferred until an explicit
  pre-R7 bring-up.
- **Implementation:** `r2-alpha-render.v3` runs six real Vulkan windows on
  Linux when xtask is built with `desktop-sdl-ash`; Linux peak RSS and process
  I/O come from procfs. The outer report may pass while its timing verdict
  remains `REPORT_ONLY`.
- **Boundary:** Windows remains a v1 shipping target. B-01, B-12, paired native
  evidence and PhysX Stage 0 readiness stay open release claims, not current
  R5/R6 handoff blockers.
- **Next action:** return to the R5 completion audit and select its next bounded
  Linux work package. Keep the Windows backlog dormant until an explicit
  pre-R7 bring-up ADR.

## Decisions

1. Linux checks execute inline with every affected work package.
2. Windows-specific actions accumulate in
   [Windows validation backlog](../windows-validation-backlog.md) and do not run
   until a superseding bring-up ADR.
3. Historical Windows and Linux target reports retain exact-commit meaning
   only.
4. Linux performance remains development evidence; it does not inherit THOTH
   budgets or close B-12.
5. Only the R2 scenario identity advances from v2 to v3; Performance V5 wire
   and methodology v8 remain unchanged.

## Validation evidence

| Check | Result | Boundary proved |
| --- | --- | --- |
| `cargo test --locked -p xtask performance --lib` | `PASS`: `45` tests | Linux procfs parsers, R2 v3 identity and methodology note are covered. |
| `cargo fmt --all -- --check`; xtask performance command tests; strict desktop-feature Clippy | `PASS`: `24` command tests, no formatting or warning failures | Both library and CLI branches remain buildable with the Linux desktop feature. |
| `cargo run --locked -p xtask --features desktop-sdl-ash -- platform` | `PASS`: Wayland/NVIDIA RTX 3080, four normalized events, eight rendered objects; state root `4fbc6f43c4e58e46d26f844f16a466cf762dbd41793ab6e1ddf7cd34ebcd9f8b` | Current Linux desktop adapter consumes the production snapshot without authority drift. |
| `cargo run --locked --release -p xtask --features desktop-sdl-ash -- performance --scenario r2-alpha-render --mode report --output target/performance-r2-linux-first-20260818` | Outer `PASS`, inner `REPORT_ONLY`; `3,600` warm-up, `21,600` measured samples, `50,400` Vulkan queries, zero deadline misses, peak RSS `198,701,056` bytes, read I/O `397,312` bytes, no unavailable resource fields; report SHA-256 `4370267ae4506e3383079d7d8b16c51bfa5b0fb63ccea070647bc9c845de3789` | R2 v3 executes all six production Vulkan windows on Linux and publishes complete process/GPU evidence. The report was intentionally run on the policy changeset worktree (`worktree_clean = false`), so it is development evidence, not a baseline or B-12 claim. |
| Changed-document direct-reference validation and `git diff --check` | `PASS` | New ADR/backlog/task-state links resolve and the patch has no whitespace defects. |
| `cargo run --locked -p xtask -- host-check` | `PASS`: `x86_64-unknown-linux-gnu`, Rust `1.97.1` | Formatting, strict Clippy, all workspace/doc tests and repository boundary checks remain green. |

The generated report remains an ignored local artifact; only its exact
path/hash/result is recorded here.
