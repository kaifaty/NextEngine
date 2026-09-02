# Physical sound V36 A0 typed-owner contract result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Decision | `REPEAT_EXACT_PASS / ZERO_OFFICIAL_VALUES` |
| Owner commit | `afbf22fe5a6cb2a98138f7c258ec057b37839e34` |
| Contract owner SHA-256 | `a93d22f4432a29cdb54bcad241b68099f9f0a56c7601e374f5a64a96a20eaac8` |
| Conformance owner SHA-256 | `9837f7ea4332d26fce654a0e3c32b2bd0ea26eefae2c481ad5856c99279474a1` |
| Claim | `ZERO_OFFICIAL_VALUE_TYPED_OWNER_PROVIDER_AND_LIFECYCLE_CONFORMANCE_ONLY / NO_FRESH_ROLE_TARGET_MODEL_HOLDOUT_REAL_VALIDATOR_RELEASE_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

A0 implements the contract that every later V36 D0/H0 owner must import. One
immutable `RoleBatch` now carries role rows, typed feature matrices, targets,
local/geometry/contact keys, partitions, groups, case spans and strata. It has
an explicit `row_count` property and intentionally implements no `__len__`.
The exact V35 failure expression is therefore rejected both at runtime and by
the pinned strict type checker.

One `OwnerLifecycle` owns all stage transitions and all target-provider calls.
A role stage cannot be advanced through a test-only marker: it must call
`materialize_role` with a provider whose kind, namespace, capability, allowed
role and returned concrete batch agree. D0 and H0 have separate frozen stage
sequences but share the same container, provider protocol, terminal vocabulary
and atomic-publication stage.

Official capability construction requires a two-run repeat-exact
`ExecutionSeal` binding owner/profile/environment, both rehearsal roots and
both exact topology hashes. A0 constructs no official provider and opens no
fresh V36 target or model value.

## Frozen topology

| Pipeline | Stages including terminal | Topology SHA-256 |
| --- | ---: | --- |
| D0 | `15` | `03f704eb262dfeb34fcfc763ac3d43bff7fb75cb87270c9f1e773a668eb34df6` |
| H0 | `10` | `7a4a9b90c077dc5d4a0a0320354bc5c3902bfef624fb5a58c8fb12a2a51d1cee` |

D0 explicitly traverses train materialization, local cross-fit, four distinct
model fits, ridge controls, development materialization/support/prediction,
evaluation, hard/resource gates and terminal publication. H0 traverses train
reconstruction, frozen candidate load, holdout materialization/support/
prediction/evaluation, hard/resource gates and terminal publication. Callable
IDs form part of each topology hash; provider kind and target values do not.

## Contract and mutation evidence

Both full miniature surrogate paths pass. Ten negative mutations are detected:

- stage skip and direct role-stage bypass;
- provider namespace/kind mismatch and wrong returned role;
- implicit `RoleBatch` length access;
- early scientific terminal;
- pre-access `OwnerFault` and post-access `ContractReject` inversion;
- non-discarded surrogate namespace;
- one-run/incomplete execution seal.

The focused suite additionally rejects writable/misshaped arrays, invalid
strata/case closure, pipeline/provider crossing and incomplete terminal use.
Every declared `TerminalDecision` has an exhaustive stable exit code.

The strict positive check covers both A0 source files with mypy `2.0.0`, NumPy
`2.5.2`, `--strict`, `--disallow-any-explicit`,
`--disallow-any-generics` and `--warn-unused-ignores`. The committed negative
fixture returns exactly one expected error: `RoleBatch` is incompatible with
`Sized` at `len(batch)`.

## Repeatability and access

| Observation | Run A | Run B |
| --- | ---: | ---: |
| Exit code | `0` | `0` |
| Artifact files / bytes | `4 / 10,573` | `4 / 10,573` |
| Tree-manifest SHA-256 | `403afdb220cf5095cc78d046d6bb3091d24d9cf90d38e02c0fcca76c7dc5752e` | same |
| stdout bytes / SHA-256 | `1,041 / 1fe04919…0ef3` | same |
| Outer wall / max RSS | `0.14 s / 32,976 KiB` | `0.13 s / 33,012 KiB` |
| D0 surrogate train/development rows | `2 / 2` | same |
| H0 surrogate train/method-holdout rows | `2 / 2` | same |
| Fresh/official target rows | `0` | `0` |
| Official model parameters initialized | `0` | `0` |
| Real/protected/network/prior-value access | `0 / 0 / 0 / 0` | same |

Artifact identities:

| Artifact | SHA-256 |
| --- | --- |
| `contract.json` | `aa60436f72da2f3197592756e68ab9c55e7925edaedbb206e3c287591a6259a1` |
| `conformance.json` | `24e52463e59513ff8ddaa11abb58341aed0b48475779f71ddde641f5c058f9ef` |
| `evidence.json` | `7fa18b6cfc8e5577bc620c9572e05c75c683dc06481a7321889f6199a85bf25d` |
| `report.json` | `1fe0491914e9415f0a19e17e863b76487279adef35e925044e0692abf6660ef3` |

## Consequence and next boundary

A0 closes the interface/lifecycle prerequisite only. It does not prove the
future numeric owner calls these stages, does not perform full-count/full-step
training and creates no execution seal. F0 may now copy V35 scientific fields
unchanged into a fresh V36 role/truth namespace and prove disjoint identities
with zero target access. C0/X0/E0 must still bind the actual numeric D0 and H0
owner to this contract and execute both full paths before any official
capability exists.

SPEC-45 remains `Proposed`. The authored clip stays authoritative; no public
schema, real-material quality, validator, cooker, demo, runtime or ProductCheck
credit follows.

## Verification

- Ruff `0.16.3` format/check and Python compile: `PASS`;
- strict mypy `2.0.0` positive owner check: `PASS`, `2/2` files;
- negative implicit-length fixture: `PASS`, exactly one expected `arg-type`
  diagnostic and no additional diagnostic;
- focused V36 A0/contract suites: `PASS`, `15/15` tests;
- official A/B conformance: `PASS / REPEAT_EXACT`, four artifacts and stdout;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because no production consumer or promoting ADR
  exists.
