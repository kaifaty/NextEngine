# Nonlocal GPU engine water-preview evidence — 2026-09-01

## Result and claim ceiling

`ENGINE_VULKAN_STATIC_SURFACE_SUPPORTED_BOUNDED` for both accepted NGQ5 final
meshes.

The new `xtask water-preview` command imports a bounded triangular OBJ through
the neutral render-content/project cooker, creates an immutable
`PresentationSnapshotV3`, validates a B0 frame plan and drives the production
SDL3/Ash Vulkan desktop backend. The runtime report must contain the planned
one visible object and one indexed draw.

This is a developer presentation tool, not gameplay authority. The rendered
mesh is static, no value feeds back into physics, and the CPU NGQ5 extractor is
not yet a GPU/renderer compute implementation. SPEC-38 and ADR-076 remain
`Proposed`; CPU DFSPH remains the fallback.

## Frozen inputs and source boundary

```text
branch                         codex/water-research
base before change             5ceb2f0ac62ee39c456dc342af02865962cca3f2
4k OBJ SHA-256                 23ba876522dae24ba6e7fe0051d33c12032e1ffb279a8f84fc36736756e358e0
16k OBJ SHA-256                0a20f71161acfc8108d1953a7a8ef4dd21ab83693ed7b457e6ce773d24c3632d
water_preview.rs SHA-256       e6c0799bb46e743f38f0196878e8ab2292484283e73c1fe2d97900e0c963c2ad
xtask main.rs SHA-256          3f9cd5354e5f965be752159218eca57689c88d33322735e740bf60ced889da03
xtask Cargo.toml SHA-256       2792d7b88f33f873ba38d97f635386fcdd0c32c29a695c9f80b9614aa2783645
release xtask SHA-256          e11fb89d27c4595d3eff8440ac6a0e95df34f95f836609efad71bbb48d4e6b97
```

The two OBJ hashes and vertex/triangle counts exactly match
`nonlocal-gpu-presentation-surface-evidence-2026-09-01.md`. Generated OBJ and
run JSON remain under `/tmp` and are not committed.

The importer accepts only UTF-8 `v` and positive-index triangular `f` records,
with explicit byte/vertex/triangle limits, finite micrometre quantization,
in-range indices, nondegenerate triangles and finite reconstructed normals.
It generates planar UVs and area-weighted smooth normals. Unreferenced sparse
height-field vertices receive a finite up normal but remain outside every
drawn triangle.

## Exact invocation

```text
cargo build --release -p xtask --features desktop-sdl-ash

target/release/xtask water-preview \
  --mesh /tmp/nonlocal-edge-surface-final-falling-dam-4k-surface.obj \
  --frames 600 --extent 1280x720 \
  > /tmp/nonlocal-engine-water-preview-4k-final.json

target/release/xtask water-preview \
  --mesh /tmp/nonlocal-edge-surface-final-falling-dam-16k-surface.obj \
  --frames 600 --extent 1280x720 \
  > /tmp/nonlocal-engine-water-preview-16k-final.json
```

The SDL/libdecor installation emitted a non-fatal `libdecor-gtk` warning and
fell back to an undecorated window. Vulkan rendering and timing completed.

## Release results on RTX 3080

| Metric | 4k accepted mesh | 16k accepted mesh |
| --- | ---: | ---: |
| vertices / triangles | `10,071 / 19,708` | `33,634 / 66,510` |
| rendered frames | `600` | `600` |
| visible objects / indexed draws per frame | `1 / 1` | `1 / 1` |
| CPU extract+submit p95 | `88 us` | `88 us` |
| GPU duration p95 | `70 us` | `193 us` |
| critical-path p95 / p99 | `88 / 98 us` | `193 / 199 us` |
| Vulkan timestamp queries / dropped samples | `1,200 / 0` | `1,200 / 0` |
| engine-owned device allocation | `29,122,384 B` | `30,343,776 B` |

Exact presentation roots:

```text
4k catalog       597dd35a5bf8f08ec9a768831231bfbde6c394a6b5cc87f95df811cfaa856468
4k snapshot      24a9afa9247b9767266209789f08fed70091dafdf404ffed7116fc2eb9b6a66a
4k frame plan    8eaa50facbf3c16b057183705387e52f6b946ad1c32a65b7c8d52fae8de9c33f
16k catalog      7dd5d84a4bf9976f0349ae66b50b4b5e2adb4075fb4a80d1ba39a29c417974f4
16k snapshot     e4ed73deb0f8d7891016ff66acc97dd05ee67d27ee883c4ae3972d21246f7ec1
16k frame plan   aad92e8a50ba2f5b497b8a505c9524857a82cbf7e050e9f048b96115a1f5d305
```

Raw JSON SHA-256:

```text
4k  d6a1c5fabe1984a3dd7bac7ddafe793e5c4b3d3e79d15a8e874bc0961092c730
16k 0aacd03771b8be756af3cb83e562d13708e547baca837037a0f8f44b041bfb3d
```

These timings isolate an already-extracted static mesh and the existing B0
draw path. They are not additive proof that live physics plus surface compute
fits one frame; the selected physics remains approximately `3.23 ms` p95 and
the CPU NGQ5 extraction remains too expensive for per-frame use.

## Checks and next boundary

- no-feature OBJ/parser tests: PASS (`3` tests);
- desktop-feature OBJ/project/frame-plan tests: PASS (`4` tests);
- `cargo check -p xtask --features desktop-sdl-ash`: PASS;
- `cargo run -p xtask -- host-check`: PASS on Rust `1.97.1`;
- `cargo run -p xtask --features desktop-sdl-ash -- platform`: PASS;
- two 600-frame Release Vulkan runs: PASS, one indexed draw per frame;
- timing samples: `0` dropped in both runs;
- screenshot/capture: `NOT_RUN`; the backend currently has no readback path;
- `git diff --check`: PASS.

Decision: retain this as the first real engine bridge. The next smallest step
is a live presentation-only upload path that refreshes the surface from the
GPU water state without rebuilding the content catalog. Port close/bilateral
extraction to GPU compute only after that ownership/lifetime boundary is
explicit, and keep its timing separate from both physics and rasterization.
