---
name: using-rust-workspaces
description: Use for workspace-level Cargo topology, shared dependency/resolver behavior, workspace lint or supply-chain configuration, feature unification, crate visibility, or multi-crate release policy. Do not use for ordinary implementation inside a member crate merely because the repository is a Cargo workspace.
---

# Using Rust Workspaces

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Address the specific workspace-level problem with one relevant reference and a
working Cargo/configuration result. A multi-crate repository does not by itself
trigger a workspace architecture audit.

## Scope gate

Use this skill only when the requested change crosses crate manifests or changes
workspace composition/policy. Borrow checking, traits, async, tests and
performance within one crate belong to `using-rust-engineering`.

Read only the reference matching the immediate issue:

| Immediate issue | Reference |
| --- | --- |
| Add/remove/split crates or dependency direction | [workspace-structure-patterns.md](workspace-structure-patterns.md) |
| Shared versions or resolver behavior | [workspace-dependencies-and-resolver.md](workspace-dependencies-and-resolver.md) |
| Workspace lint policy | [workspace-lints-and-clippy-config.md](workspace-lints-and-clippy-config.md) |
| Advisory/licence/source policy | [workspace-deny-config.md](workspace-deny-config.md) |
| Feature unification surprise | [feature-unification-gotchas.md](feature-unification-gotchas.md) |
| Public versus internal crates | [crate-visibility-and-internal-traits.md](crate-visibility-and-internal-traits.md) |
| Miri across selected crates | [miri-on-workspace-subset.md](miri-on-workspace-subset.md) |
| Cross-crate test placement | [test-organisation-at-workspace-scope.md](test-organisation-at-workspace-scope.md) |
| Workspace documentation | [documentation-architecture.md](documentation-architecture.md) |
| Multi-crate release | [release-flow-for-workspaces.md](release-flow-for-workspaces.md) |
| Task runner/CI symmetry | [task-runner-patterns.md](task-runner-patterns.md) |
| Cross-crate coverage | [coverage-at-workspace-scope.md](coverage-at-workspace-scope.md) |
| Focused health audit | [workspace-anti-patterns.md](workspace-anti-patterns.md) |

## Implement the smallest correction

- Inspect root and affected member manifests before choosing a policy.
- Change only the manifests/configuration needed by the observed issue.
- Validate with the narrowest Cargo metadata/build/test command that exercises
  the affected graph.
- Do not generate workspace governance documents, a numbered artifact suite,
  CI scaffolding, deny policy or publication rules unless requested.
- Adding a crate requires a clear dependency boundary, but not a full workspace
  redesign when existing architecture already governs it.

A comprehensive workspace review may load several references only when the user
explicitly requests that audit or the workspace is being created/restructured
as the primary task. Even then, findings and working configuration are primary;
do not emit one document per reference unless those documents were requested.
