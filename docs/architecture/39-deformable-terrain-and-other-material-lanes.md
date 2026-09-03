# SPEC-39: Deformable terrain and other material lanes

| Поле | Значение |
|---|---|
| ID | SPEC-39 |
| Статус | Proposed |
| Версия | 0.1 |
| Последняя проверка | 2026-09-03 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-38](38-continuum-material-physics.md), [ADR-076](adr/076-continuum-material-physics-track.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md), [ADR-104](adr/104-water-v1-authority-is-the-exact-table-and-flow-network.md) |
| Заменяет | the "Other material lanes" section and the `CONTINUUM-TERRAIN-P1` row of SPEC-38 2.4, moved here unchanged when SPEC-38 3.0 was accepted for water (ADR-104) |

## Status and scope

This SPEC holds the candidate semantics and promotion gates of the
non-water continuum lanes: dry deformable terrain, wet material saturation
and drainage, and the later free-water/terrain flux. It authorizes no
runtime schema, changes no production world and claims nothing shipped.
The one-pass composite step, the exchange tuple and the rigid-writer
authority of SPEC-38 and ADR-076/081 apply unchanged; water itself is
Accepted in SPEC-38 3.0.

## Material lanes

Dry deformable terrain begins only with one calibrated Drucker-Prager sand
profile implemented through APIC/MLS-MPM. The MPM lane exclusively owns a
declared deformable wheel/terrain or foot/terrain contact pair; the matching
PhysX ground contact is disabled, and MPM emits the sole bounded reaction
batch. The first consumer is an instrumented prescribed single-wheel rig, not
a complete vehicle.

Exact active terrain persistence follows dry-sand/contact evidence and
precedes saturation. Wet material then advances through saturation/drainage,
mechanical response, and only later a closed atomic free-water/terrain flux
batch. Cross-region transfer, lossy sleep and two-phase poromechanics are
separately gated later work.

## Product checks before promotion

| Check | Required result |
|---|---|
| `CONTINUUM-TERRAIN-P1` | One calibrated dry-sand profile and prescribed wheel/terrain contact pass declared conservation and reference curves. |

## Promotion boundary

No public contract is added by this SPEC. A terrain consumer requires its
own frozen plan with gates, an Accepted ADR narrowing ADR-058 for the
terrain contact pair, and synchronized SPEC-26, routing, traceability and
roadmap updates.
