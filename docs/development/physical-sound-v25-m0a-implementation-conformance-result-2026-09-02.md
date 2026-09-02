# Physical sound V25 M0a-I — implementation conformance result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `REPEAT_EXACT_CONTRACT_PASS / IMPLEMENTATION_FROZEN / OFFICIAL_MODEL_VALUES_UNOPENED / M0A_E_AUTHORIZED` |
| Protocol | [V25 M0a causal-material neural-student protocol](physical-sound-v25-m0a-causal-material-neural-student-protocol-2026-09-02.md), SHA-256 `2311f2410b44c89617dbaf4a6f4b9f7d30fd0a183f5fd262853c85f5be956e68` |
| Implementation commit | `6b04dc5fe6e60e7dc40e51b196d29e74a4e64ff4` |
| Implementation root | `9df63e614ff1cd4e46531b4157379306d9bc128da6f92e2886ea0810790f2e46` |
| Product effect | None; external research execution only, with authored clips still authoritative |

## Result

M0a-I passes its value-independent implementation-conformance boundary. The
complete owning entry point now implements deterministic parsing,
preprocessing, training, evaluation, role-gated evidence access, local MLflow
diagnostics and atomic publication for the frozen causal-material student.

The contract fixture executes the CLI twice and obtains byte-identical
canonical weights, predictions and all eight published files. It proves that
isolated elasticity and density mutations reach distinct physical-material
coordinates, while X0 Glass keeps unknown physical constants and support as
zero-valued masked inputs. It also exercises the actual D0 T0/X0 assembler,
strict binary parsers and the complete protected-role order.

This is not a model-quality result. No official checkpoint, loss, development,
method-holdout or disclosed-real query value was opened. The fixture's short
schedule deliberately carries no quality claim. M0a-E may now run once under
the frozen official schedule without changing architecture, seed, thresholds,
roles or losses.

## Frozen implementation identity

| File | SHA-256 |
| --- | --- |
| `physical_sound_v25_m0a_common.py` | `592b38137eb088e4ffc8087c3c68c0c19b0a8ceed8d049e7e9b2f6a002573e89` |
| `physical_sound_v25_m0a_evaluate.py` | `e5c48a74f5df89d60433d2ef1605bce648250836809e04721a2aa1fefe3e552a` |
| `physical_sound_v25_m0a_model.py` | `6075b4121c5eb45aca2934308fb4e73c078c467669e458c0d36b65b066f4009b` |
| `physical_sound_v25_m0a_train.py` | `86ada58126ad901021cdf297cef2fc46059dc33763e960dbdc21ebf3972630b5` |

The root is SHA-256 over the canonical sorted filename-to-hash map. The owning
CLI rejects any manifest whose implementation root, protocol hash, official
T0/X0/combined evidence hashes or lineage bindings differ.

## Contract observations

| Observation | Result |
| --- | --- |
| Model | fixed CPU float32 `m0a-contact-modal-field-v1`, seed `3101`, `23,142` parameters |
| Object input | `38` values: geometry `24`, semantic material `5`, physical material `6`, support `3` |
| Contact input | deterministic local/contact descriptor with eight-point farthest-point sampling |
| Canonical artifacts | eight files; two full fixture executions byte-identical |
| Protected access | candidate freezes before one-shot method holdout; admission shadow and REALIMPACT row `2407` never open |
| Controls | ridge plus no-geometry, no-contact and no-residual ablations |
| Causal mutations | elasticity, density, thickness, scale, force and remesh paths are independently observable |
| Failure behavior | malformed tensors/binaries, stale hashes, rebound lineage, authority changes, wrong seed/thread/profile, occupied output and interrupted publication reject without partial output |
| MLflow | external local `file://` store, autolog disabled, diagnostic only; no registry, serving or remote tracking |
| Repository artifacts | no datasets, weights, generated audio, run outputs or MLflow store committed |

## Verification

| Check | Result |
| --- | --- |
| Python bytecode compilation for four implementation modules and test module | `PASS` |
| Ruff format and static analysis for all five M0a files | `PASS` |
| T0 + X0 + V3 assembler + M0a focused suite | `PASS`, `19 passed / 0 failed` |
| M0a full-entry and mutation suite after final audit | `PASS`, `6 passed / 0 failed` |
| Canonical A/B fixture | `PASS`, all eight output files byte-identical |
| Module-size policy | `PASS`; largest module is `999` lines |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known unrelated `SOURCE_LAYOUT_ESCAPE_HATCH` in `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs` |

The boundary-scan failure predates M0a and names no M0a file. It remains a
workspace-level risk and is not reported as an M0a pass.

## Decision and next boundary

M0a-I is complete. Commit `6b04dc5f` and implementation root `9df63e…e46`
are the rollback boundary for this model family. M0a-E is authorized to build
one exact official manifest from the frozen external T0/X0/combined roots and
execute official A/B under the protocol resource ceilings.

After any official model-derived value opens, this implementation may not be
tuned against that value. The official result must publish the first terminal
decision exactly as observed: `PASS`, `REPRESENTATION_REJECT`,
`DOMAIN_GAP_REJECT`, or a conformance/resource result. No outcome authorizes
runtime inference, public contracts, Metal admission or removal of the authored
fallback.
