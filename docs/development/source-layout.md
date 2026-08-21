# Rust source layout

Next Engine keeps crate boundaries product-driven and uses modules to control
source organization. A large file is not a reason to create another crate:
new crates still require an architectural ownership or dependency boundary.

## Guidance

- There is no hard per-file line-count limit.
- Prefer cohesive modules in the 300–700 line range.
- Prefer functions below 150 lines. Functions above 200 lines should be split
  into named phases unless a compact state machine or table is clearer intact.
- Move a large `#[cfg(test)]` block into a sibling `tests` module when that
  makes the tests easier to navigate.

`cargo run -p xtask -- boundary-scan` does not enforce source-file size. It
rejects source-including `include!` invocations and aliases, plus direct or
conditional `#[path]` module escape hatches, under the scanned source roots.

## Module shape

Keep established public module paths stable. When splitting `foo.rs`, retain it
as the facade and place implementation modules below `foo/`:

```text
src/
├── foo.rs
└── foo/
    ├── command.rs
    ├── codec.rs
    ├── error.rs
    └── tests/
        ├── mod.rs
        └── roundtrip.rs
```

The facade declares private submodules and re-exports only the existing public
surface. Use `pub(super)` for internal cross-module collaboration where private
visibility is insufficient. Do not use `include!` or `#[path]` to obscure the
source inventory, and do not expose new public paths solely to make a split
compile.

Pure moves should be committed before any behavioral decomposition. After each
split, run focused crate tests, `git diff --check`, and the product checks
required by the affected subsystem.
