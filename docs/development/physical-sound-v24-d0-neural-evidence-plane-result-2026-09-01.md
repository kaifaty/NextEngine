# Physical sound V24 D0 — neural evidence-plane result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `REPEAT_EXACT_PASS / CONTRACT_ONLY / T0_X0_PROTOCOLS_AUTHORIZED / DATA_MODEL_AND_RUNTIME_UNAUTHORIZED` |
| Protocol | [V24 D0 neural evidence-plane protocol](physical-sound-v24-d0-neural-evidence-plane-protocol-2026-09-01.md) |
| Implementation | `f97778b91f4785ea59b3d7ea94274ca8a562dcec` |
| Product effect | None; external research records only, with authored clips still authoritative |

## Result

D0 passes its bounded contract question. The existing Rust
`physical-sound-registry neural-data-plane` owner now accepts V2 unchanged and
adds a V3 projection for three non-interchangeable evidence lanes:

- `synthetic_teacher`, with complete physical axes and a hash-checked modal
  teacher target;
- `exact_real_transfer`, with transfer semantics and no synthetic target;
- `identified_real_recording`, with recorded-waveform semantics and no
  fabricated transfer claim.

The full owning `run_cli` entry point ran twice inside each of two clean focused
executions. All four emitted files were byte-identical within A/B, every five
role and three lane was traversed, and protected method-holdout/admission-shadow
row identities and hashes remained absent from materialized projections and
commitments.

This is a data-contract result only. It does not establish teacher fidelity,
real-source sufficiency, model quality, validator calibration, material
admission or runtime eligibility.

## Exact fixture observations

| Observation | Result |
| --- | --- |
| Focused execution A | `10 passed / 0 failed` |
| Focused execution B | `10 passed / 0 failed` |
| Full `run_cli` fixture inside each execution | A/B emits exactly four byte-identical files |
| V3 contract rows | `10`: synthetic `2`, exact transfer `2`, identified recording `6` |
| Teacher targets | `2`, only on synthetic rows |
| Protected roles | commitments/counts only; rows, audio hashes and materialized values absent |
| Authority flags | model training `false`; method holdout materialized `false`; admission shadow materialized `false` |
| V2 compatibility | existing V2 tests pass; V3 fields remain absent from V2 projection/report output |
| Repository artifacts | none; temporary fixtures and outputs removed |

The negative suite rejects all three lane/semantics mismatches, missing teacher
or synthetic physical axes, teacher targets on either real lane, missing whole
lanes, invalid representation/mode bounds, stale target hashes, all six
cross-role leakage classes, unknown schema/fields, non-empty targets and an
interrupted atomic publication.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | `PASS` |
| `cargo test -p xtask physical_sound_registry_command:: --no-fail-fast` | `PASS`, `161 passed / 0 failed` |
| Focused neural-data-plane command, two post-commit executions | `PASS`, `10/10` then `10/10` |
| `cargo clippy -p xtask --all-targets -- -D warnings` | `PASS` |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known unrelated `SOURCE_LAYOUT_ESCAPE_HATCH` in `realimpact_transfer_fixture.rs` |

The boundary-scan failure predates D0 and names no changed D0 file. Current D0
module files are individually below the repository's 1,000-line limit. It is
reported as remaining workspace risk, not hidden or treated as a D0 pass.

## Decision and next boundary

D0 is complete. It authorizes freezing T0 and X0 protocols against the V3
record, then generating only their declared external evidence. It does not
authorize opening T0/X0 values before those protocols, training M0, calibrating
V0 from candidate outputs, or changing the authored-clip fallback.

The smallest next actions are:

1. freeze T0's modal-target binary schema, units, deterministic teacher,
   analytic controls, object/contact/remesh partitions and resource ceiling;
2. freeze X0's exact internet-object identity and the precise axes it can
   claim without inference;
3. execute T0 and X0 independently into external V3 manifests; stop before M0
   if either record is invalid or insufficient.
