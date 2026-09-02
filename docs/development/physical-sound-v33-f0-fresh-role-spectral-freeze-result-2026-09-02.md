# Physical sound V33 F0 — fresh-role spectral freeze result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / REPEAT_EXACT_PASS / ZERO_NEW_VALUES` |
| Protocol | [F0 fresh-role spectral freeze protocol](physical-sound-v33-f0-fresh-role-spectral-freeze-protocol-2026-09-02.md) |
| Profile | [`physical-sound-v33-f0-mode-local-spectral-residual.v1.json`](../../lab/profiles/physical-sound-v33-f0-mode-local-spectral-residual.v1.json) |
| Owner | [`physical_sound_v33_f0_profile_freeze_v1.py`](../../lab/scripts/physical_sound_v33_f0_profile_freeze_v1.py) |
| Product effect | None; I0 sequencing only, authored clips remain authoritative |

## Result

The value-independent V33 F0 freeze passes twice byte-identically. It validates
one canonical profile against the V32 evidence closure and proves that the new
material constants, geometry cells and contact sets are fresh without
evaluating any truth expression, target row or model parameter.

Decision: `F0_PROFILE_FREEZE_PASS`.

The frozen experiment contains:

- three new synthetic materials, ten new geometry multiplier cells and 24 new
  contact pairs, all disjoint from V32 M1;
- `648 / 432 / 432` train, development and method-holdout cases, or `1,512`
  cases and `15,120` modal rows in total;
- separate development and holdout `geometry-only`, `contact-only` and `joint`
  strata, while exact case identities remain role-disjoint;
- 32 contact inputs: the 12 lawful V32 inputs, 16 fixed Fourier components and
  four local P1 participation-stencil samples;
- one `1,811`-parameter spectral candidate, the fresh `1,491`-parameter raw-MLP
  control, identity, nearest and raw/spectral ridge controls;
- one seed, optimizer, step count, access order, resource envelope and complete
  development/holdout gate matrix with no selection or retry surface.

The three decay, gain and contact truth expressions are distinct from V32.
Their coefficients and all role memberships are now committed, but their
values remain unmaterialized. Train, development, holdout, real signal and
network access are exactly zero, as are initialized model parameters.

## Exact evidence

Both external runs publish the same three files, `7,473` bytes in total, and
identical stdout.

| Artifact | SHA-256 |
| --- | --- |
| Protocol | `aff11077eb3ec8148c877cde87094d482bf098aa4e6af9f93f8e1e66ff60fa1a` |
| Profile | `4c1869101af3ec2264771e17af5c2e652e5861e36c3a8ab61333e588db88fd52` |
| Owner | `b3c8ee4cceac36361b36ce95269388c4cc039dadd87e86bcf3fbc32f0c77bb2e` |
| `conformance.json` | `1d257598b20fb9b5cc63a2f7cae71db793ff84a081e6dd6977117f7a6e97a4dc` |
| `evidence.json` | `acf201ebfa147f1f51e17cba00db4c1a80dd9e26605880a72df0b686ab001ff2` |
| `report.json` | `b71208d3d6712304517b97c0804fa4ee79f5bf9928169fa0a73c1854577e2560` |
| stdout | `db3c88b42e3777a6f1f2dc6c3b7b99375f074e64b58a3fe6b1316bc8c3cbb5ea` |
| Canonical comparison closure | `cddd0e3cb0345bf32f9b3a9c8e52804a6b796d510a15cda15673bb56be0c460c` |

The runs complete in `0.05 / 0.04 s` and peak at `21,728 / 21,688 KiB` RSS,
inside the frozen `300 s / 1 GiB / 64 MiB output` envelope.

## Verification

| Check | Result |
| --- | --- |
| F0 focused suite | `PASS`, `4/4` |
| Inherited V31 P1 + V32 M0/M1 suites | `PASS`, `17/17` in pinned `lab` environment |
| Ruff format/check on the F0 owner and tests | `PASS` |
| Two complete external publications | `PASS`, exact files/stdout |
| Canonical profile and dependency hashes | `PASS` |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known `SOURCE_LAYOUT_ESCAPE_HATCH` in unchanged `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`; no F0 path is named |

The first inherited-suite invocation used the host Python without NumPy and
failed during import; the authoritative rerun uses the repository-pinned
Python 3.12 `lab` environment and passes all 17 tests. The boundary diagnostic
is unchanged external debt, remains a failed mapped check and grants no
ProductCheck or runtime credit.

## Consequence

F0 is closed without spending any synthetic target or model value. I0 may now
implement the complete feature construction, clamped local stencil, model and
control mechanics only on discarded non-official fixtures. D0 remains blocked
until that owner passes value-independent conformance twice exactly. This
result grants no real-material, naturalness, validator-release, admission,
cooker, demo, runtime or ProductCheck authority.
