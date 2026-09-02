# ADR-101: Presentation-only dynamic surface ring and bounded frame capture

| Field | Value |
|---|---|
| ID | ADR-101 |
| Status | Proposed |
| Version | 0.1 |
| Proposal date | 2026-09-02 |
| Last verified | 2026-09-02 |
| Normative dependencies | [SPEC-04](../04-rendering-and-platform.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-29](../29-platform-host-and-application-session.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [ADR-003](003-vulkan-renderer-and-shader-toolchain.md), [ADR-028](028-platform-session-and-presentation-authority.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-100](100-authoritative-water-volume-and-presentation-only-gpu-water.md) |
| Supersedes | none |
| Superseded by | none |

## Context

SPEC-30 describes two vertex paths in the B0 renderer: immutable
indexed-indirect geometry cooked into `RenderContentCatalogV1`, and the
per-frame-slot skinned vertex ring driven by exact skinning records in
`PresentationSnapshotV3`. A free water surface changes topology every frame;
re-cooking the catalog or republishing a snapshot per frame is neither
possible within the frame budget nor meaningful as content identity.

The SDL3/Ash adapter now carries a third path, used by the developer water
bridge: a composition root declares one exact catalog mesh revision with a
fixed vertex/index capacity, the adapter allocates one ring per frame slot
once, and the frame source publishes immutable vertex/index updates next to
(or instead of) a new snapshot. Measured on the RTX 3080 witness with a 16k
surface: one catalog/snapshot/frame plan per run, `0.34 ms` render critical
path with device-local residency, real-time surface cadence at 60 Hz. The
same adapter gained a bounded one-frame capture to host memory so live
presentation can be judged as human evidence.

## Decision

### Declared dynamic surfaces are renderer-private caches

- A run may declare at most eight `DynamicSurfaceProfileV1` entries. Each
  names one exact mesh revision that exists in the active catalog with a
  single triangle primitive, a vertex and index capacity bounded by
  `1,048,576` and `3,145,728`, and a residency (`HostVisible` or
  `DeviceLocal`). Capacity never grows at runtime.
- The catalog mesh remains the stable identity, material binding and
  declared bounds. A `DynamicSurfaceUpdateV1` replaces only the vertex/index
  payload for one frame slot; it must lie inside the catalog mesh bounds,
  within capacity, carry strictly increasing sequences and a content-only
  canonical hash. Undeclared revision, capacity excess, out-of-bounds vertex
  or sequence regression fail closed with typed diagnostics before exposure;
  a publication batch commits atomically.
- Updates never enter `PresentationSnapshotV3`, the frame-plan hash, any
  content, save, replay or gameplay root. Device loss or swapchain rebuild
  reconstructs the rings from the declared profiles; the next frame republishes.
- The world and shadow passes bind the slot ring for a dynamic draw and
  rebind the immutable stream afterwards; the closed B0 shader interface,
  material contract and descriptor layouts are unchanged.

### Bounded developer frame capture

`DesktopRunOptions::frame_capture` names one rendered frame. The swapchain
is then created with transfer-source usage or the run fails closed before the
first frame; after that frame the image is copied to host memory and
reported once as tightly packed sRGB RGBA8. This is SPEC-04 developer
diagnostics: it never becomes a correctness oracle, a gameplay input or a
release artifact, and captures stay outside the repository.

### Boundary

No type from this decision enters `crates/contracts`; the declaration and
update types are adapter-owned presentation API consumed by composition
roots and developer tools. A runtime consumer that needs the surface inside
the presentation snapshot, a compute-written ring without staging, or a
displayless capture target requires its own decision.

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `RENDER-DYNSURF-P1` | Publish bounded updates for one declared surface over a bounded run with frame capture. | One catalog/snapshot/frame plan; ring refreshes equal publications times frame slots; rejected batches leave the prior state; capture read once after idle. | Draw the catalog placeholder mesh when no update was published. |

## Considered alternatives

- Re-cooking the catalog per frame — rejected: identity churn and cost.
- Reusing the skinning stream through a fake skeleton record — rejected: it
  would abuse an exact animation contract for non-animation data.
- A public contract type in `crates/contracts` — rejected until a runtime
  consumer exists under ADR-046.
