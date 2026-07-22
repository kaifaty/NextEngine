# Contributing

The repository is local-first during bootstrap. Changes must preserve the architecture ownership and backend boundaries in `docs/architecture/`.

Before committing, run:

```text
cargo run -p xtask -- host-check
```

Contributions use DCO sign-off and an inbound-equals-outbound policy: submitted engine work is offered under Apache-2.0 unless a file records compatible third-party provenance. Use `git commit -s` when contribution history becomes shared. See [DCO.md](DCO.md) and [GOVERNANCE.md](GOVERNANCE.md).

Do not commit local artifacts, generated model weights, imported installations/output, or anything under `incubator/gothic-importer/`. Do not add native/vendor types to `next_contracts`; introduce adapters behind engine-owned contracts instead.
