# Nonlocal NPR0 static-boundary evidence — 2026-08-20

Status: REPORT_ONLY / SPLIT_STATIC_BOUNDARY_SELECTED / TINY_CORPUS_AUTHORIZED

## Full product-support preflight

The exact SPEC-38 outer extent generates 24,704 fixed two-layer support
samples. Together with 48,000 fluid samples, both candidate profiles execute
72,704 solver participants and reserve 2,399,232 directed pairs.

| Profile | Profile SHA-256 | Exact GPU result | Output / CSR |
| --- | --- | --- | --- |
| nuv-basin-48k-static-support-control.v3 | 7107e963b787bed48c900a062d201a3c8201837af3b43a62e905aa2b233a7a35 | PASS; P1/P2 exact; finite; 23,472,394 bytes | ba2f3671487b4efe6ca5b20502152d4a8c00f2d21504cb263f26cfcf99bc05ac / d609289eaf5865677d6ffb907a1e663bf132284ed7f6b564c1a992dbe81b2c8b |
| nuv-basin-48k-static-support-derived.v3 | 5fdd10c5ac3c9a0eb631cb333c0f39ed7cbe980b37f73e21dfa44030919045e1 | PASS; P1/P2 exact; finite; 23,472,394 bytes | c70dfd359a29d9014cc3cf126bafb0a77e57e22017c36bde88c43f6341917146 / d609289eaf5865677d6ffb907a1e663bf132284ed7f6b564c1a992dbe81b2c8b |

Both reports set compact_eligible=false, fallback_u32=true and
selected_neighbor_id_bytes=4. This is the required checked fallback, not a
retained compact-P2 performance result. The single-call totals are diagnostic
and award no percentile or integrated-budget credit.

The updated machine-audit SHA-256 is
c7e6a0ee26d913c3862446af6444a18dd8b021af1fe66e1bcdbfe217bd663a5f.
Its semantic status remains PROFILE_RECLOSURE_REQUIRED.

## Negative contact discriminator

The independent tiny fixture generates exactly 208 fixed support samples
around one fluid sample. Four derived-coefficient SISSM iterations are finite,
all fixed samples move by exactly zero, and the ghost-only tentative position
crosses the bottom face:

- tentative centre Y: -0.046861773697794817 m;
- allowed minimum centre Y: 0.025 m;
- directed pairs / maximum degree: 4,547 / 30.

The separate bottom-face swept-sphere counterfactual activates feature 2 at
time fraction 0.41030093755237035, accepts centre Y=0.025 m and closes
fluid impulse plus boundary reaction with absolute residual zero.

The stable bounded result root is
ee92af43f9b510afe245b82c15a589471dd303583609e39c3539ed968bdfc2b6;
the raw JSON SHA-256 for this run is
841211c137671d44dc78a61260e1db7d2c292a18fef4bc28862839c1f5c60579.
The command passes with semantic result SPLIT_BOUNDARY_REQUIRED.

## Consequence

The two-layer complement is admitted as static density support. It is
explicitly insufficient for non-penetration. The tiny physical corpus must
evaluate the selected split schedule and may not treat successful ghost
execution as a sealed-boundary claim.

The loss of global u16 indexing becomes a later performance candidate only
after NPR0 selects a physical profile. A typed split fluid/static index space
is the obvious hypothesis; changing it now would optimize two still-unselected
coefficient profiles.

