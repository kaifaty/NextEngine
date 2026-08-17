# R5d BodySchema projection — completed task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-17 |
| Task key | `r5d-body-schema-projection` |
| Scope | Bind the frozen Stage 0 `BodySchemaV1` to the exact reference project and deterministically derive player/NPC physics descriptors, tensor layouts and actuator-safety roots without creating a second mutable owner |
| Definition of done | The cooker publishes one exact body-schema root; production bootstrap compiles default instance projections for both reference characters; focused and restore/replay checks prove stable roots or fail closed |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in schemas and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5d is complete. Authoring V7/activation V8 bind one exact frozen `BodySchemaV1`; production bootstrap derives neutral player/NPC projections with shared schema/tensor/safety roots and distinct instance/physics roots.
- **Why:** The Stage 0 V1 generation is already the accepted public schema. Its compiled descriptors are deterministic projections used by the shared physical-animation binding, not another transform or save owner.
- **Next action:** Start R5e with one bounded production procedural motor/safety/recovery route consuming these exact roots while capsule Physics remains the sole active transform owner.
- **Current blocker:** None inside R5d. Active articulation cutover, non-default overlays, real skinning and learned control remain out of scope.
- **Do not retry:** Treating `BodySchemaV2` as the runtime successor, persisting compiled projections as mutable state, or adding non-default morphology/topology mutation without an admitted consumer.
- **Reconsider when:** Accepted architecture replaces the frozen Stage 0 generation or an actual consumer requires non-default overlays/topology remap.

## Delivered boundary

1. `BodySchemaAssetV1` canonically wraps the frozen V1 schema, exact asset revision and admitted compiler-profile identity with bounded fail-closed decode.
2. `BodyInstanceProjectionV1::validate_against` binds exact schema ID, revision and hash. Neutral owner hashes are explicit and non-zero; default overlay revisions remain zero and topology revision remains one.
3. `CompiledBodySchemaV1::compile_projection` atomically derives canonical body/shape/joint/actuator descriptors, observation/action layouts, full actuator-safety facts and `BodyProjectionRootsV1`.
4. Authoring V7, cook and `ActivatedProjectV8` publish one DomainRelevant body-schema root. The CharacterDefinition requires it; wrong profile, missing dependency/root, malformed bytes or hash mismatch rejects the complete candidate.
5. Reference bootstrap compiles the exact activated asset for stable player/NPC IDs. Shared schema/layout/safety roots match; subject, instance, physics-descriptor and complete projection roots differ.
6. Physical-animation bindings require the matching projection, but active gameplay collision and transform authority remain with the existing capsule Physics path.
7. Compiled projections are rederived after project/session rehydration. Replay V10 remains a ten-owner closure with no projection segment or format successor.

## Decisions

### D-001 — Preserve the accepted schema generations

- **Decision:** R5d production instances compile the exact V1 schema. The separate biomechanics/training `BodySchemaV2` lineage is unchanged.
- **Reason:** Reinterpreting V1 as V2 would violate generation identity and ADR-067/069 compatibility.
- **Consequence:** R5d claims only the bounded V1 production projection.

### D-002 — Compiled projections are derived evidence, not an owner

- **Decision:** Project content owns the immutable schema; runtime derives instance roots from exact schema plus owner revision/hash inputs and rederives them after restore.
- **Reason:** Morphology, equipment, stats, damage, fatigue and attachment facts remain owned by their domains.
- **Consequence:** Existing save/replay owner arity and transform ownership are unchanged.

### D-003 — Preserve the frozen offline nominal subject

- **Decision:** `BodyInstanceProjectionV1` continues to accept the all-zero nominal subject used by the tracked Stage 0 offline mirror. The production reference bootstrap separately rejects zero or duplicate player/NPC IDs.
- **Reason:** Strengthening the frozen V1 wire contract broke the byte-exact Python/Isaac mirror fixture; production identity validation belongs at the consumer boundary.
- **Consequence:** Historical training compatibility remains exact while live reference subjects fail closed.

### D-004 — No runtime-profile or training-manifest successor

- **Decision:** Compiler semantics are bound by the exact body asset compiler-profile ID/hash and project/content closure. `core_r5c`, Replay V10 and frozen training manifests remain unchanged.
- **Reason:** R5d adds no command, schedule, numeric rule, saved owner or training consumer.
- **Consequence:** Project authoring/activation alone advance to V7/V8; learned/training lineages gain no promotion authority.

## Exact evidence

| Evidence | Result |
| --- | --- |
| `cargo test -p next_contracts body --no-fail-fast` | `PASS`: 21 body/reference/compatibility tests |
| `cargo test -p next_motor compiler --no-fail-fast` | `PASS`: 9 compiler tests, including source-order and two-subject root separation |
| tracked Stage 0 mirror exact test | `PASS`: zero-subject offline fixture remains byte-identical |
| `cargo test -p next_project --test content_pipeline --no-fail-fast` | `PASS`: 15 cook/activation tests, including body profile/root/dependency failures |
| `cargo test -p next_reference_game --test physical_animation --no-fail-fast` | `PASS`: direct/rehydrated projection roots and live owner binding |
| `cargo test -p next_world --lib --no-fail-fast` | `PASS`: 12 tests; V7 CharacterDefinition dependency updates exact staging bytes `6714 -> 6730` |
| `cargo run -p xtask -- content-package` | `PASS`: 36 roots, 122 entries, 64 chunks; composition root `33952d1df2dfa1aaec0a188778d31a465bb1f846f87c16fefba20ab37d1e0af7` |
| `cargo run -p xtask -- play` | `PASS`: 32 ticks, 52 events, state root `0d8f846843a6972e0af92d6237a310552307763a7f640208d6496cbf8138c3b2` |
| `cargo run -p xtask -- persistence-replay` | `PASS`: 20 ticks, two generations, final state root `a40ec7f39ab88b2f5d150b0431e3e4f6373fe8680272e6c25842a264a6389662` |
| full `host-check` Rust/doc phases | `PASS`: workspace compilation, Clippy, all Rust tests and all doc tests completed; the command then reported only the 1116-line `body.rs` structural limit |
| post-fix `cargo run -p xtask -- boundary-scan` | `PASS`: tests moved to `body/tests.rs`; `body.rs` is 942 lines and all six boundary checks pass |
| post-split body/format/diff checks | `PASS`: 21 body tests, `cargo fmt --all -- --check` and `git diff --check` |

## Known boundaries

- The articulated descriptor graph is compiled and consumer-bound but is not inserted into the active reference gameplay scene.
- No non-default equipment, damage, fatigue, attachment or topology transaction is admitted.
- No real skinning, general graph/non-identity retarget, physical IK, learned policy or full `ANIM-ROOT-MOTION-P1` claim is made.
- Performance is `NotRun(not triggered)`: compilation occurs at bootstrap and this cut does not materially change the measured hot path.

## Handoff

- **Next bounded cut:** R5e production procedural motor/safety/recovery route over the exact R5d projection roots.
- **Promotion:** Roadmap and current Accepted SPEC/glossary/traceability references advance R5d to complete without declaring full R5.
- **Recovery note:** A concurrent functional-anatomy workspace transfer stashed this complete diff before resetting the primary worktree. R5d was finalized first on an isolated branch, then reapplied over the completed anatomy commit without overwriting it.
