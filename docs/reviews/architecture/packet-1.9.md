# Architecture review snapshot: packet 1.9

| Field | Value |
|---|---|
| Record ID | ARCH-REVIEW-1.9 |
| From packet | 1.8 |
| To packet | 1.9 |
| Status | Historical |
| Outcome | SPEC-31, ADR-029 and ADR-031 accepted through normal repository workflow on 2026-07-26 |
| Authority | Historical snapshot only; current Accepted SPEC/ADR text governs |

## Promotion scope

This packet recorded the autonomous-quest/narrative-director track before its
final consistency remediation:

- SPEC-31 defines the `AgentIntent/world need → candidate → admission → latent
  Quest → disclosure → activation` boundary, four closed disclosure channels,
  system-owned challenge/reward terms, explicit per-quest autonomy, world-local
  graph revisions, causal outcome hooks, bounded director request/candidate
  contracts, deterministic template fallback and replay without regeneration;
- ADR-029 keeps Quest/graph authority in RPG Framework, world facts in World
  Services, private NPC intents and untrusted asynchronous proposals in Agent
  Intelligence, and dialogue/UI at the disclosure boundary;
- ADR-031 adds one dedicated RPG-owned `DivineStanding` aggregate, bounded
  per-god epistemic scopes and interventions, directed pantheon/covenant
  conflicts, independent per-god requests over one pre-decision snapshot and
  one deterministic atomic multi-god resolution;
- `REQ-148`…`REQ-155` and `FAIL-062`…`FAIL-065` are local editorial anchors;
- no external technology changes status and no `VS-16` is created.

The subsequent
[consistency remediation](../../plans/2026-07-26-autonomous-quest-divine-consistency-remediation-implementation-plan.md)
fixed decision timing, cross-context atomicity, offer/covenant lifecycle,
epistemic spillover, reaction lineage and world compatibility before the
documents became Accepted. This historical record is not approval authority,
does not create implementation conformance and accepts no model/provider.

## Automatic checks

| Check | Result | Evidence reference |
|---|---|---|
| Markdown relative-link validation | PASS | local architecture tree check, 2026-07-26 |
| cargo fmt --all -- --check | PASS | `cargo run -p xtask -- host-check`, 2026-07-26 |
| cargo clippy --workspace --all-targets -- -D warnings | PASS | `cargo run -p xtask -- host-check`, 2026-07-26 |
| cargo test --workspace | PASS | `cargo run -p xtask -- host-check`, 2026-07-26 |
| cargo run -p xtask -- boundary-scan | PASS | `cargo run -p xtask -- host-check`, 2026-07-26 |
| git diff --check | PASS | local working-tree check, 2026-07-26 |
| cargo run -p xtask -- host-check | PASS | macOS aarch64 developer host, Rust 1.93.0, 2026-07-26 |

## Review decision

The Repository Owner directly confirmed the architecture direction and
requested the consistency remediation. After synchronized document updates and
successful local checks, SPEC-31, ADR-029 and ADR-031 became Accepted through
the normal repository workflow. The authoritative result is their current
document text; this packet remains a historical snapshot and does not claim
that runtime scenarios are implemented or passing.
