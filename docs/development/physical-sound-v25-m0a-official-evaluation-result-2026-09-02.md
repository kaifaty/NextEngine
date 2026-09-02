# Physical sound V25 M0a-E — official evaluation result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `IMPLEMENTATION_CONFORMANCE_REJECT / PREPROCESSING_ONLY / NO_MODEL_VALUES / RUN_B_NOT_STARTED / M0A_SPENT` |
| Protocol | [V25 M0a protocol](physical-sound-v25-m0a-causal-material-neural-student-protocol-2026-09-02.md) |
| Frozen implementation | commit `6b04dc5fe6e60e7dc40e51b196d29e74a4e64ff4`, root `9df63e614ff1cd4e46531b4157379306d9bc128da6f92e2886ea0810790f2e46` |
| Official manifest | external SHA-256 `37e3418a2de4bb7bc70eede12ae91e0d0120642dbd0861cbae2d600f8ecbd7e2` |
| Product effect | None; authored clips remain authoritative and every downstream gate stays blocked |

## Result

Official run A rejected during deterministic preprocessing with:

```text
M0 contact does not bind an exact mesh vertex
```

The failure occurred before ridge fitting, neural construction, optimization,
checkpoint creation, candidate comparison, development, method holdout or the
disclosed real query. No model-derived weight, loss, metric or sound value was
opened. The owning CLI removed its staging directory and published no output.

The protocol states that one failed official execution spends M0a. Run B was
therefore not started and the frozen implementation was not repaired in place.
This is an implementation-conformance result, not evidence for or against the
neural representation or sound quality.

## First failing boundary

| Observation | Exact value |
| --- | --- |
| Row | `t0-train-t-beam-a-c00` |
| Phase | synthetic train-context preprocessing |
| Refined mesh | `825` vertices; query binds exactly |
| Coarse mesh | `221` vertices; query is not a vertex |
| Nearest coarse-vertex distance | `0.0016666666666666635 m` |
| Frozen exact-binding tolerance | `3.0265491900843113e-10 m` |
| Cause | official contact uses an eighth-fraction coordinate while the coarse transverse grid has denominator `12`; the surface point is valid but off-vertex |

The contract fixture used quarter-fraction contacts on `5×5` and `9×9` grids,
so every fixture contact happened to be a vertex in both meshes. It proved the
exact-vertex code path but did not cover the official grid/contact denominator
combination. Increasing the tolerance to millimetres would silently select a
different spatial query and is not a valid repair.

## Execution evidence

| Observation | Result |
| --- | --- |
| Manifest contract | `PASS`: profile, protocol, implementation and official T0/X0/combined hashes match |
| Combined rows | `151` |
| T0 aggregate verification | `PASS`, `204` artifacts |
| Network boundary | cgroup `IPAddressDeny=any` plus offline/frozen/no-sync environment |
| Resource boundary | cgroup `MemoryMax=4 GiB`, `MemorySwapMax=0`; external `1,800 s` timeout |
| Wall time | `3.22 s` |
| Peak RSS | `797,824 KiB` |
| Exit | status `1`, preprocessing conformance reject |
| Resource record | external SHA-256 `8a37eb51b929c31bd35dbd1108ca071e915729472a534d495a96800664b06a7d` |
| Canonical output | none |
| MLflow/output/staging directories | none after failure |
| Run B | `NOT_STARTED_BY_STOP_RULE` |

## Decision

M0a closes as `IMPLEMENTATION_CONFORMANCE_REJECT`; its quality hypothesis
remains unobserved. A successor may not reinterpret the failure as a model
signal or tune architecture, capacity, seed, loss, thresholds or role access.

The smallest admissible successor is a newly frozen execution-equivalent
protocol that replaces exact-vertex remesh sampling with deterministic
triangle location and barycentric interpolation, and adds an official-profile
structural fixture that exercises every contact/grid combination before any
model value. See the [bounded research](physical-sound-v26-off-vertex-field-transfer-research-2026-09-02.md)
and [Roadmap V26](../plans/physical-sound-synthesis-roadmap-v26.md).
