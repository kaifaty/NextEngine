# Linux-first host policy — task state

| Field | Value |
| --- | --- |
| Status | `SUPERSEDED / ADR-090` |
| Updated | 2026-08-20 |
| Task key | `linux-first-host-policy` |
| Scope | Historical ADR-082 transition that made Linux active while retaining a deferred Windows v1 target; the retained-Windows boundary is superseded by ADR-090 |
| Definition of done | Accepted policy, specs, routing, roadmap and developer guidance agree; R2 executes through SDL3/ash on Linux with native process counters; mapped Linux checks pass; deferred Windows work has one bounded backlog |
| Authority | Historical working context only; ADR-090, current Accepted SPECs, `docs/roadmap.md` and exact check outputs outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Superseded by ADR-090. Linux remains the active host
  and becomes the only v1 shipping target; Windows/THOTH/paired work is outside
  current scope indefinitely rather than queued for pre-R7 bring-up.
- **Implementation:** `r2-alpha-render.v3` runs six real Vulkan windows on
  Linux when xtask is built with `desktop-sdl-ash`; Linux peak RSS and process
  I/O come from procfs. The outer report may pass while its timing verdict
  remains `REPORT_ONLY`.
- **Boundary:** Historical results below retain their exact-run meaning, but
  this file no longer defines current platform or R7 policy.
- **Next action:** Use
  [linux-only v1 task state](linux-only-v1-release-policy.md) and ADR-090; do
  not reactivate the former Windows backlog from this historical state.

## Decisions

1. Linux checks execute inline with every affected work package.
2. The former [Windows validation backlog](../windows-validation-backlog.md) is
   archived outside current scope; do not add new actions to it.
3. Historical Windows and Linux target reports retain exact-commit meaning
   only.
4. Linux performance remains development evidence until R7c accepts a Linux
   hard profile; it does not inherit THOTH budgets or close B-12.
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
