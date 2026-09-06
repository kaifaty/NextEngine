---
name: using-rust-engineering
description: Use when working in Rust — routes to specialist sheets for borrow-checker errors (E0502/E0597/E0382), trait bounds (E0277), async Send/Sync, clippy warnings, unsafe/FFI soundness, performance profiling, or PyO3/candle interop
---

# Using Rust Engineering

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Implement or repair the requested Rust behavior in the existing crate. Inspect
the affected source, compiler diagnostic and nearby callers before choosing a
specialist reference. A straightforward fix can proceed directly; routing is
an aid for unresolved language or runtime questions.

## Work from the actual boundary

- Use the repository-pinned toolchain, dependencies and lint policy. In
  NextEngine, preserve its unsafe-code boundary and production command paths.
- Read ownership guidance for borrow/lifetime problems, traits guidance for
  bounds/dispatch, and async guidance for executor or Send/Sync issues. Read a
  second reference only when that boundary is involved.
- For an unsafe/FFI change, establish the accepted boundary and its invariants
  before editing; a working build does not establish soundness.
- For performance work, identify the affected workload and measure the disputed
  cost before choosing an optimization. Avoid speculative redesign.
- For workspace-wide dependency, resolver, lint or publication changes, use
  [using-rust-workspaces](../using-rust-workspaces/SKILL.md).
- Recover missing diagnostics from the workspace where possible. Ask a focused
  question only when the target or required behavior remains materially unclear.

## Verify and finish

Run repository-required formatting/static analysis and focused tests for the
affected package. Use broader checks when public contracts, workspace settings
or uncertain dependencies warrant them. Coverage campaigns, Miri, nightly
toolchains and extra frameworks are conditional on the actual change and
repository policy; examples in topic references do not require installing them.

Report what changed, the relevant check results and remaining risk. Do not
require a new specification, interview or specialist handoff for ordinary Rust
work. Upstream slash commands and named agents are optional capabilities; use
available tools and the local references when those integrations are absent.

## Topic references

Read only the reference relevant to the next decision. Paths are relative to this skill.

- [project-structure-and-tooling.md](project-structure-and-tooling.md) — workspace layout, `build.rs`, feature flags, `links=` metadata
- [unsafe-ffi-and-low-level.md](unsafe-ffi-and-low-level.md) — bindgen/cbindgen, ABI contracts, safe wrapper patterns
- [testing-and-quality.md](testing-and-quality.md) — Miri, integration tests across the FFI boundary
- [systematic-delinting.md](systematic-delinting.md) — staged clippy reduction by category
- [performance-and-profiling.md](performance-and-profiling.md) — measure first (flamegraph, criterion, heaptrack)
- [ownership-borrowing-lifetimes.md](ownership-borrowing-lifetimes.md) — fix the Rust-side lifetime model before crossing languages
- [ai-ml-and-interop.md](ai-ml-and-interop.md) — PyO3 patterns (`Python<'py>`, GIL, NumPy buffer protocol)
- [modern-rust-and-editions.md](modern-rust-and-editions.md) — capture rule changes, `cargo fix --edition`, resolver differences
- [traits-generics-and-dispatch.md](traits-generics-and-dispatch.md) — dyn-compatibility changes, `impl Trait` precise capture
- [async-and-concurrency.md](async-and-concurrency.md) — async trait changes (`async fn in trait`, RPITIT interactions)
- [error-handling-patterns.md](error-handling-patterns.md) — anyhow, thiserror, custom error types, `?` operator, error context
