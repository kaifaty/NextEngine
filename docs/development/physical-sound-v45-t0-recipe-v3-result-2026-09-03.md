# Physical sound V45 T0 — Recipe V3 and masked-target result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `COMPLETE / REPEAT_EXACT / VALUE_FREE / NO_EXTERNAL_PAYLOAD_AUTHORITY` |
| Decision | `T0_RECIPE_V3_MASKED_TARGET_CONTRACT_REPEATABLE` |
| Claim | Experimental schema/projection evidence only; no external numeric payload, training, validator, admission, cooker, demo, public-contract or runtime authority |
| Next | C0 external Clatter prior/control decoder |

## Frozen implementation

- profile: [`physical-sound-v45-t0-recipe-v3.v1.json`](../../lab/profiles/physical-sound-v45-t0-recipe-v3.v1.json),
  `12,512` bytes, SHA-256
  `6cf7e1fe6c141ac324b9896defcadc033047fcbd7d33bc8f6b245d99cb0c9994`;
- owner: [`physical_sound_v45_t0_recipe_v3_v1.py`](../../lab/scripts/physical_sound_v45_t0_recipe_v3_v1.py),
  `29,993` bytes, SHA-256
  `6c6cd0ed708f98fa30fd862fdc8b29f27c24e6a17302512bf3d5fe8a0cedb316`;
- focused tests:
  [`test_physical_sound_v45_t0_recipe_v3_v1.py`](../../lab/tests/test_physical_sound_v45_t0_recipe_v3_v1.py),
  `8,034` bytes, SHA-256
  `a5d41164ab07ec150d6729ce0334cd8b2eff8af88452d6f3d24ba67b4659ae77`.

The profile hash-binds its owner, the V45 evidence rebaseline and the completed
R0 result. It does not bind the living roadmap or task-state file.

## Contract frozen by T0

Recipe V3 is a fixed `189`-coordinate integer vector with four explicit heads:

- modal body: mode count, base frequency, up to `32` frequency ratios, positive
  RT60 values, participation and uncertainty;
- excitation/transfer: onset, contact duration, impact gain, eight location
  modifiers, four support modifiers and uncertainty;
- radiation/residual: `24` band-energy values, spectral tilt, a `16`-element
  residual envelope and uncertainty;
- support: explicit OOD score plus a separate descriptor-presence mask.

Each neutral target carries a binary coordinate mask, exact source lane,
active mode count and descriptor mask. A missing coordinate has value and mask
both zero and contributes no loss. `validator_calibration` and
`protected_admission` lanes cannot construct generator targets. Empirical,
synthetic-modal, structural-transfer and real-acoustic lanes can expose only
their frozen field sets; the owner rejects lane escalation.

The sole deterministic projector:

- uses integers only and emits canonical JSON;
- sorts modes by raw ratio and input ordinal, then makes ratios strictly
  increasing;
- clamps all modal frequencies below the `48 kHz` Nyquist boundary;
- makes every RT60 positive and bounds onset/contact/gain/uncertainty fields;
- closes modal-participation, band-energy and residual-envelope budgets at
  most `1,000,000 ppm` with integer floor plus stable largest remainder;
- rejects malformed shape, unknown descriptors, over-capacity mode banks and
  an ordered mode set that cannot fit below Nyquist;
- zero-pads inactive modal coordinates in the fixed vector.

These are representation and safety semantics, not learned values. The two
tracked fixtures are artificial contract controls and contain no decoded
Clatter, NISR, VibraVerse, structural, microphone, validator or protected
payload.

## Repeat-exact execution

Two fresh external output directories have no `diff -qr` difference:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access.json` | 642 | `c76fd7c3fecbc0ba25ccd5786c5b1cde8ec5aa97ddc9e77ced3eef2732129f5c` |
| `contract.json` | 5,927 | `ffd9410cf11152f63064f2a61f5a9df099f132ccad9816b5380b124859419f38` |
| `fixtures.json` | 16,968 | `72a71bebfe4c1a5708bc3b4fef989f80e3d481e1ce618da01683bb934a0f9f1e` |
| `report.json` | 1,277 | `c24a084977c0a205a86dd83a2955536d28a02b389688dedf22f21d6dd3abbada` |

All six report gates pass. All eight forbidden access counters are zero. The
stress fixture observes `73` real-acoustic-lane coordinates; the missing-label
fixture observes zero coordinates and returns exactly zero masked loss for an
arbitrary prediction.

## Failure coverage

Nine focused tests pass. They cover dependency closure, two-run identity,
fixed-vector roundtrip, zero padding, ordered modes, positive decay, Nyquist,
energy/resource bounds, missing-label loss, validator/protected isolation,
lane escalation, target-mask/value corruption, non-integer values, unknown
descriptors, unknown profile fields and external-output/import guards.

## Consequence

T0 closes the value-free representation boundary and authorizes C0 to define
an external Clatter decoder against these exact fields. It does not authorize
opening NISR/VibraVerse/structural/acoustic payloads outside their later
preflights, model fitting, independent-validator calibration, protected
admission, deterministic clip cooking or a demo/runtime consumer. Real support
remains `71/105`, deficit `34`; PSEL and real training remain blocked, and the
authored clip fallback remains mandatory.

## Verification

- `python3 -m py_compile` for the owner and focused test: `PASS`;
- `python3 -m unittest lab.tests.test_physical_sound_v45_t0_recipe_v3_v1`:
  `PASS`, nine tests;
- two fresh owner runs plus `diff -qr`: `PASS`, byte-identical;
- `git diff --check` and direct changed-link/path validation: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL` only on the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
  T0 does not modify or import that path, adds no source-layout escape hatch
  and receives no boundary-scan credit.
