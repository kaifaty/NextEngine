# Continuum water W0F geometry, capacity and root closure evidence — 2026-08-18

Status: `REPORT_ONLY / SUCCESSOR_PROFILE_ROOTS_FROZEN / W1_AUTHORIZED`.

## Scope and result

W0F closes the profile-level blockers left by the W0E local survivor. The
research oracle now has one exact geometry representation shared by fluid
neighbor visibility, density support, opening classification and swept
contact; a static-boundary capacity that admits every W1 scenario; and a new
domain-separated successor root set.

Production and separately written calculators match exactly for geometry,
all oriented density-support records, four density/gradient fixtures and eight
swept-contact fixtures. The successor retains the exact W0E hydro and
free-fall transcripts, and a 24-step two-chamber orifice preflight completes
with legal transfer, strict particle-radius clearance and bounded momentum
accounting.

This is profile closure, not corpus credit or production promotion. The full
W1 serial corpus and Windows/Linux root comparison have not run.
`CONTINUUM-WATER-REF-P1` remains `NOT_RUN`; SPEC-38 and ADR-076 remain
`Proposed`; the main roadmap remains `PLANNED / NOT_ACTIVE`.

## Closed architectural contracts

| Responsibility | Successor operation | Closure |
| --- | --- | --- |
| Geometry authority | Exact integer outer box plus optional X-plane patch and closed rectangular opening | One canonical manifest and stable feature IDs `0..5`, `16..24` |
| Fluid visibility | Exact rational centre-segment intersection against the solid patch | Neighbors across solid wall are excluded without a float epsilon |
| Density support | Two-layer `REST_VOLUME` lattice complement; internal samples are side-oriented | Opposite chambers cannot see each other's ghost support; opening cells are excluded |
| Contact | Frictionless swept particle-radius constraints after pressure and before integration | Outer faces, internal face and aperture edges use stable feature order; no retry or position repair |
| Incompressibility | Projected diagonally preconditioned active-set PCG, `2..=50` | W0E solver family is now hash-bound in the successor profile |
| Static capacity | `32,768` boundary samples | Product extent count `24,704` is admitted; dynamic rigid capacity remains separate W3 work |
| Canonical state | Stable ID and integer position/velocity after every 240 Hz substep | No pressure, neighbor, contact or warm-start continuation state is added |

The contact schedule admits at most eight constraints per sample and uses an
explicit binary64 direction-rounding guard. Contact impulse is reduced both as
a total and by stable feature ID. Density support remains rebuilt profile data,
not canonical owner state.

## Independent comparisons

| Comparison | Result |
| --- | --- |
| Canonical geometry bytes, root, five classification probes and nearest features | `EXACT_MATCH` |
| Orifice density-support records | `EXACT_MATCH` — `10,880/10,880` |
| Density-support root | `dbbff914c2b6f36fc030fc2c44632cb5e5aced7b26d9a2bfb1b50e604c4f451a` |
| Outer face, internal face, aperture edge and corner density/gradient fixtures | `EXACT_MATCH` |
| Separating, resting, direct, fast, pass, graze, edge and simultaneous-corner contact fixtures | `EXACT_MATCH` |
| Static-boundary thresholds | `32,767 PASS`, `32,768 PASS`, `32,769 EXPECTED_REJECTION` |
| Fluid/boundary row thresholds | `127 PASS`, `128 PASS`, `129 EXPECTED_REJECTION` |

The independent path does not call the production geometry serializer,
support generator, neighbor reconstruction or swept-contact implementation.
Exact agreement therefore discriminates shared fixture inputs from a shared
implementation mistake.

## Frozen successor roots

| Projection | SHA-256/root |
| --- | --- |
| W0F document | `faf655976ca52f2ba49cd1fe786a82771f0df5a4889199f0417f727f4cf62df0` |
| Float profile | `4b08279c5d640b3a550677aa70e4c9798cda354ab0936683e9b7bf3f18d8c91e` |
| Execution manifest | `141448c0a1eac2d110639ba5a70476e9cc10b4536679f900728f3b3a5ae14eac` |
| Corpus | `2170a7117ac7dfa9cca38fa465ac4805ffb8b73693e38e18a1285dbfe077c75d` |
| Fixtures | `6bea54b5223aff6735ecb44d6597326737efecf986665f7ca609259803546c38` |
| Geometry | `575300649a3d0d7f3c0c92f95d3e194d3dacb1602a18ab064959e026118f18bf` |
| Composite execution profile | `617ceec10c0ce2e1c90ec45a3a713696e58313a7432cc9379aadbf4adf48cab3` |

Scenario roots are:

- hydro `8df5c03be5f09ffecc9d3787712b138b463d9a21f5fd42c259e0f348799e7373`;
- free-fall `84af378ac392983b88318e9b63a727a41c97e9bf2e9b22c13091988141cd26fe`;
- dam-break `de787aca267b8c0c7b892f14639ef011911d28750d66acb47b09f42adc6c873e`;
- still tank `272043a7d9ae428908867d7092bfdbe8dedf36021e2664dd1c78701e1ddf7756`;
- orifice `a10a731187bdc66a41a5f9ac3407103a43c3ea1ffd58760d76cd9d5ed5717839`;
- sealed product extent `047e9ff6e2fc1f0b806caa7e96ea8fc571b1334fe213d5c5248dcae2ac629556`;
- storage order `e26be82f99726a763016958daea8cedb8c1cec77c6efe2b3dee5e2e7d61ba25f`.

Every in-code scenario definition is compared byte-for-byte with its rooted
document projection before the closure command can run.

## Capacity and regression observations

Static support counts are hydro `5,824`, free-fall `9,344`, dam-break
`16,384`, still tank `5,824`, orifice `10,880`, sealed product extent
`24,704`, and storage order `2,368`. All are within the successor capacity.

The hydro regression completes `1200/1200`, reaches at most 48 density
iterations and `99,998 ppb`, and preserves the W0E roots
`399705af374393503581dd9f3d03bc0bba8e44af732544c5ea3c19d7dbd90b4f`
at step 24 and
`026d26585edbda74aff93ef126810b0ced0a7c9f5d623b4dbf60260b48554b18`
at step 1200. Free-fall compares all 97 frames, activates zero contact
constraints and retains root
`cbe47b57dbb819e44eabe53049a1b9cb44c6a94626d560421c73deb6b1db4011`.

The orifice preflight completes `24/24`: chamber counts change from
`6000 + 0` to `5976 + 24`, with 24 strict left-to-right crossings and none in
reverse. Minimum clearance squared is exactly `625,000,000 µm²`, equal to the
squared `25,000 µm` radius. Density reaches at most 42 iterations and
`99,519 ppb`; feature impulse reduction differs by `0 ppb`, total momentum
residual is `23 ppb`, and the final frame root is
`a03f787a09626fcc1d9b46e54275c8ad9914c389ffbf9396928054025e7afa36`.

## Clean repeatability evidence

Implementation and clean-report checkpoint:
`a56e57a24ae78607949e09396287de641bf6a8a9`.

Two clean runs used the frozen `water-oracle` target/profile and recorded
`tool_tree_state=CLEAN`. Their complete report SHA-256 values differ because
wall-clock nanoseconds are explicitly diagnostic:

- run 1: `12b91c095c32b05254683dd7d9eec6bc9662a4f78c45b4a8516b0b59e59170d2`;
- run 2: `7c44d40002e9cf1fede08921bfcd2cfa94fdbbf310a1c6a964221cade3a91a9c`.

After removing only `details.wall_clock_nanoseconds`, both bounded JSON
projections have SHA-256
`7a1a11ef9ea73769d03ca400de506007f03231ae8d7f08b60f289392459452ba`.
All roots, fixtures, counts, regressions and dispositions are identical.
Raw reports remain outside Git at
`/tmp/nextengine-w0f-clean-{1,2}-a56e57a.json`.

The exact command for each unique output path was:

```bash
CARGO_ENCODED_RUSTFLAGS=$'-Ctarget-cpu=x86-64\x1f-Ctarget-feature=-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,-fma\x1f-Cllvm-args=-fp-contract=off' \
cargo run --locked --profile water-oracle \
  --target x86_64-unknown-linux-gnu -p xtask -- \
  continuum water close-successor-profile \
  --output /tmp/nextengine-w0f-clean-1-a56e57a.json
```

## Verification and remaining risk

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` and `git diff --check` | `PASS` |
| Strict all-target Clippy for `next_continuum_water` | `PASS` |
| `next_continuum_water` tests | `PASS` — 64/64 |
| `xtask` tests | `PASS` — 99/99 library and 44/44 binary |
| Strict all-target Clippy for `xtask` | `PASS` |
| `cargo run --locked -p xtask -- boundary-scan` | `PASS` — all six checks |
| Two clean successor-profile reports | `EXACT_MATCH` after excluding diagnostic wall clock |
| `CONTINUUM-WATER-REF-P1` | `NOT_RUN` |

W0F authorizes the existing W1 serial oracle under the successor roots. It
does not prove dam-break, 7200-step still-tank, full orifice, sealed-48k,
storage-order, cross-target, 50k performance, PhysX coupling, persistence or
runtime/public-contract readiness. W2 and integration remain blocked until W1
passes.
