# Contributing

The repository is local-first during bootstrap. Keep changes small,
product-driven and inside the technical boundaries in `docs/architecture/`.

Before handing off a code change, run:

```text
cargo run -p xtask -- host-check
```

No DCO trailer, CLA or separate contribution ceremony is required. Submitted
engine work is offered under Apache-2.0 unless a file records compatible
third-party provenance.

Do not commit local artifacts, generated model weights, imported
installations/output, or anything under `incubator/gothic-importer/`. Do not add
native/vendor types to `next_contracts`; introduce adapters behind engine-owned
contracts instead.
