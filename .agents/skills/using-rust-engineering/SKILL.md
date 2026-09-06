---
name: using-rust-engineering
description: Diagnose a Rust-specific compiler, ownership, trait, async, unsafe/FFI, tooling, test, or profiling problem when language mechanics are the obstacle. Do not invoke merely because an ordinary feature happens to be implemented in Rust.
---

# Using Rust Engineering

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Use the smallest relevant Rust reference to unblock working code. The primary
artifact is the requested implementation or fix, not a routing report.

## Scope gate

If the task is a domain feature and no Rust-specific obstacle has appeared,
implement it normally under repository conventions. Do not load this pack.

When a concrete Rust issue exists, inspect the exact diagnostic and nearby code
first. Fix an obvious localized issue directly. Read one specialist reference
when the language rule, safety invariant or measured bottleneck is non-trivial:

| Symptom | Reference |
| --- | --- |
| Edition or modern syntax | [modern-rust-and-editions.md](modern-rust-and-editions.md) |
| Move/borrow/lifetime error | [ownership-borrowing-lifetimes.md](ownership-borrowing-lifetimes.md) |
| Trait bound, generics or dispatch | [traits-generics-and-dispatch.md](traits-generics-and-dispatch.md) |
| Error type or `?` conversion | [error-handling-patterns.md](error-handling-patterns.md) |
| Single-crate Cargo/features/build setup | [project-structure-and-tooling.md](project-structure-and-tooling.md) |
| Test layout, mocking or property tests | [testing-and-quality.md](testing-and-quality.md) |
| Large lint cleanup | [systematic-delinting.md](systematic-delinting.md) |
| Tokio, `Send`/`Sync` or blocking async | [async-and-concurrency.md](async-and-concurrency.md) |
| Measured performance bottleneck | [performance-and-profiling.md](performance-and-profiling.md) |
| Unsafe, ABI, FFI or `no_std` | [unsafe-ffi-and-low-level.md](unsafe-ffi-and-low-level.md) |
| PyO3/Candle/tensor interop | [ai-ml-and-interop.md](ai-ml-and-interop.md) |

Workspace topology, dependency unification, workspace lints and crate
publication belong to `using-rust-workspaces`. An ordinary change inside one
member crate does not.

## Work guard

- Load a second reference only after the first diagnosis exposes another
  independent boundary.
- Preserve the repository's existing error, dependency and async conventions.
- Do not introduce `clone`, boxing, `Arc<Mutex<_>>`, lint suppression or
  unsafe merely to silence a diagnostic; establish the actual ownership,
  dispatch, concurrency or safety requirement.
- For performance, profile first. For unsafe, state and test the invariant.
- Finish with focused formatting/static analysis/tests for the affected crate.

Do not create a Rust methodology document unless the user explicitly asks for a
review or policy.
