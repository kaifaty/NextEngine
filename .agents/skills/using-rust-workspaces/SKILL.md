---
name: using-rust-workspaces
description: Resolve Cargo workspace structure, shared dependency or lint policy, resolver/feature interactions, crate visibility and release organization. Use for changes spanning member composition or workspace configuration; a local Rust fix inside a workspace uses using-rust-engineering instead.
---

# Using Rust Workspaces

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Resolve the requested Cargo workspace problem while preserving its established
structure and policy. Start with the root manifest, affected member manifests,
lockfile and repository instructions. For a local borrow, trait, async or lint
issue, use [using-rust-engineering](../using-rust-engineering/SKILL.md).

## Select the relevant concern

- Structure: identify crate responsibilities and dependency direction before
  adding or moving a crate. Change topology only when the task requires it.
- Dependencies and features: inspect declared inheritance, resolver, feature
  graph and the failing build selection. Preserve pinned versions and edition;
  do not turn a local fix into an unsolicited resolver migration.
- Lints and supply chain: use existing workspace rules and inspect inheritance
  for affected members. Add policies or tooling only within the requested scope.
- Public surface: identify publishable/internal crates and any newly exposed
  types. A public re-export needs the repository's stability decision;
  documentation hiding alone does not establish encapsulation.
- Tests, documentation, release and coverage: load their references only for
  those tasks or when the changed boundary needs them.

## Keep work proportional

Use the repository's existing architecture and change workflow. The numbered
artifact sets, size tiers, compulsory spec campaigns and tooling recipes in
upstream references are examples for an explicitly requested workspace design,
not prerequisites for a patch. Do not generate a parallel specification tree,
add CI, replace an existing task runner or remove a single-member workspace
merely because a generic checklist recommends it.

For mixed dependency versions or differing builds, inspect the actual graph and
reproduce the failure before declaring the workspace invalid or reorganizing it.
State unresolved constraints; continue useful in-scope work. Named upstream
commands or agents may be absent in this installation; use available Cargo
commands and the local topic references instead.

## Verification

Check the affected members and feature combinations. A resolver, shared lint,
build-policy or public-contract change can require workspace checks; a local
edit does not automatically require all targets, coverage, Miri, publication or
a security campaign. In NextEngine, use AGENTS.md and routed ProductChecks.
Reuse successful checks until changed inputs or evidence justify repeating them.

## Topic references

Read only the reference relevant to the next decision. Paths are relative to this skill.

- [workspace-structure-patterns.md](workspace-structure-patterns.md) — Layered vs feature-grouped vs domain-grouped; the `workspace.members` ordering question; when each works and when it breaks; trait-crate pattern for cycle avoidance
- [workspace-dependencies-and-resolver.md](workspace-dependencies-and-resolver.md) — `[workspace.dependencies]`; per-crate `dep = { workspace = true }` inheritance; resolver-1 vs resolver-2 vs resolver-3 (rust ≥ 1.84); the feature-unification math; `cargo build -p A` vs `cargo build` divergence
- [workspace-lints-and-clippy-config.md](workspace-lints-and-clippy-config.md) — `[workspace.lints]` table; `lints.workspace = true` inheritance; workspace-scope `clippy.toml` thresholds (cognitive complexity, line length); per-crate override discipline
- [workspace-deny-config.md](workspace-deny-config.md) — Workspace-scope `deny.toml`; composition with single-crate `axiom-rust-engineering:audit`; advisory database, licence allow-list, source policy, banned crates; waiver lifecycle
- [crate-visibility-and-internal-traits.md](crate-visibility-and-internal-traits.md) — Public vs internal crates; the internal-traits-crate pattern; `doc(hidden)`; sealed traits; semver implications; `publish = false`
- [workspace-anti-patterns.md](workspace-anti-patterns.md) — God-crate, leaky internal-only API, version drift, cyclic features, single-package workspace, accidental publication of internal crate, deny.toml shadowing, clippy.toml shadowing
- [feature-unification-gotchas.md](feature-unification-gotchas.md) — The seven cases the headline rule misleads: `default-features` unanimous-vote, transitive defaults, mutually-exclusive features, dev-dep contamination, `cargo build -p` non-isolation, `dep:` prefix audit, hidden defaults in `[workspace.dependencies]`
- [miri-on-workspace-subset.md](miri-on-workspace-subset.md) — Selective Miri at CI scope; the arena-crate pattern; nightly toolchain split (Pattern A vs B); MIRIFLAGS policy; `cfg(miri)` gating
- [test-organisation-at-workspace-scope.md](test-organisation-at-workspace-scope.md) — Per-crate unit + integration tests; the workspace integration-tests crate; `*-test-fixtures` (data) vs `*-test-helpers` (infrastructure); `cargo nextest` vs `cargo test`
- [documentation-architecture.md](documentation-architecture.md) — Rustdoc per crate + mdbook for the workspace; the "book sits next to the crates" pattern; intra-doc links; `[package.metadata.docs.rs]`
- [release-flow-for-workspaces.md](release-flow-for-workspaces.md) — Synchronised vs independent versioning; the dep-graph publish order; `cargo-release` vs `release-plz`; tag schemes; pre/post-publish verification; yank-as-rollback
- [task-runner-patterns.md](task-runner-patterns.md) — `justfile` (recommended) vs cargo aliases vs `xtask` vs scripts; the CI-symmetry rule; recipe-naming conventions; `pre-commit` vs `ci` granularity
- [coverage-at-workspace-scope.md](coverage-at-workspace-scope.md) — `cargo-llvm-cov` vs `cargo-tarpaulin`; per-crate thresholds (not one workspace number); doc-test inclusion; Codecov flags; the gaming trap
