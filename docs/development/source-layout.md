# Rust source layout

Next Engine keeps crate boundaries product-driven and uses modules to control
source-file size. A large file is not a reason to create another crate: new
crates still require an architectural ownership or dependency boundary.

## Limits

- A Rust source file under `apps/`, `crates/`, or `tools/` may contain at most
  1,000 physical lines.
- Prefer cohesive modules in the 300–700 line range.
- Prefer functions below 150 lines. Functions above 200 lines should be split
  into named phases unless a compact state machine or table is clearer intact.
- Tests count toward the file limit. Move a large `#[cfg(test)]` block into a
  sibling `tests` module instead of treating test code as exempt.

`cargo run -p xtask -- boundary-scan` enforces the hard limit. During the
existing-file cleanup, the scanner contains an explicit path-and-ceiling
exemption for each legacy oversized file. Exempt files cannot grow. An
exemption also fails once its file reaches the hard limit, so the refactor and
exemption removal land together.

## Module shape

Keep established public module paths stable. When `foo.rs` grows, retain it as
the facade and place implementation modules below `foo/`:

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
visibility is insufficient. Do not use `include!` or `#[path]` to hide physical
size, and do not expose new public paths solely to make a split compile.

Pure moves should be committed before any behavioral decomposition. After each
split, run focused crate tests, `git diff --check`, and the product checks
required by the affected subsystem.
