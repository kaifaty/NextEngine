# Agent instructions

## Admission

- Treat `docs/architecture/` as the normative source of truth.
- Run `cargo run -p xtask -- host-check` before handing off a change.
- Use public engine-owned contracts; do not leak OS, vendor, ECS-backend, or importer types into `crates/contracts`.
- Do not add CI workflows or remotes during local bootstrap.

## Data and repository boundaries

- Never commit game installations, imported or protected assets, datasets, checkpoints, training runs, model outputs, captures, or secrets.
- `incubator/gothic-importer/` is an ignored nested Git repository. It must not become a Cargo member, path dependency, or parent-repository tracked path.
- Runtime state changes only through production command boundaries; test-only mutable backdoors are forbidden.

## Physical and evidence claims

- Mac MPS/CPU smoke proves only the bounded train/export/inference toolchain.
- Do not claim `PhysicalCertified` until `TRAIN-RTX-01` and all required POLICY/PHYS/TRAIN gates pass. Use `PrototypeFallback` or `AwaitingCapability` honestly.
- Observable changes require the scenario/capture/evidence workflow from SPEC-15. An agent cannot create human approval or waive an automatic gate.
