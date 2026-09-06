# Physical sound V33 I0 — mode-local spectral owner result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / REPEAT_EXACT_PASS / OFFICIAL_VALUES_UNOPENED` |
| Frozen protocol | [F0 fresh-role spectral freeze](physical-sound-v33-f0-fresh-role-spectral-freeze-protocol-2026-09-02.md) |
| Profile | [`physical-sound-v33-f0-mode-local-spectral-residual.v1.json`](../../lab/profiles/physical-sound-v33-f0-mode-local-spectral-residual.v1.json) |
| Owner | [`physical_sound_v33_i0_mode_local_spectral_owner_v1.py`](../../lab/scripts/physical_sound_v33_i0_mode_local_spectral_owner_v1.py) |
| Product effect | None; D0 sequencing only, authored clips remain authoritative |

## Result

The complete I0 owner passes twice byte-identically on discarded non-official
fixtures. It executes the P1 plate, cantilever and simply-supported beam
families, builds the exact `11 / 13 / 32` candidate branches and `12`-field raw
contact control, initializes both frozen MLP topologies, exercises ridge and
nearest controls and completes three disposable gradient steps per MLP.

Decision: `I0_OWNER_CONFORMANCE_PASS`.

The conformance corpus contains 18 cases and 180 modal rows using one material,
one geometry cell and six contacts that are disjoint from both V32 and V33
official roles. It proves:

- exact candidate/raw-MLP parameter counts `1,811 / 1,491`;
- all 32 contact features in frozen order, including 16 Fourier and four local
  P1 participation samples;
- four clamped-edge probes and 72 direct P1 stencil-sample comparisons;
- lexicographic nearest tie-break and finite identity/raw-ridge/spectral-ridge/
  nearest/raw-MLP mechanics;
- exact branch isolation when contact-lift or material inputs are changed;
- exact P1 frequency/order passthrough, 74 contact-node zeros, gain signs,
  remesh identity and `0.5 / 1 / 2` impulse scaling;
- positive decay, finite output, correction bounds and maximum corrected peak
  `0.03937939314093168`, below the frozen strict `0.95` bound;
- canonical external publication and cleanup after a late pre-publication
  failure.

Official train, development and method-holdout target rows, official training
steps, real/protected signal values and network requests all remain exactly
zero. The non-official three-step losses are finite-gradient probes only and
cannot select a model, threshold or D0 outcome.

## Exact evidence

Both external runs publish the same three files, `8,816` bytes in total, and
identical stdout.

| Artifact | SHA-256 |
| --- | --- |
| F0 profile | `4c1869101af3ec2264771e17af5c2e652e5861e36c3a8ab61333e588db88fd52` |
| I0 owner | `8a12af9417feae0e92da6950ffc7b86327902849647e959a1ad01b4e94a6f5b0` |
| `conformance.json` | `1b6cad06e6863c88175a399c02dcd0559eb88c80ad1e142019f2756648182057` |
| `evidence.json` | `4330185ab2240bb0bf0048a55efef4d2d9dc249b940d7387e885a3fd36017203` |
| `report.json` / stdout | `08dd17215f1007ee11201e265da637e54ca13ef1c6e5946f19190716a7f9f719` |
| Canonical comparison closure | `11a4170bfe973d9cf5f61412b1f4f7da5927f30bcbcefd42ef0b04fd3ab5ac17` |

The runs complete in `4.54 / 4.52 s` and peak at `865,740 / 865,716 KiB` RSS,
inside the frozen `300 s / 1 GiB / 64 MiB output` envelope. The memory margin
is only about `178 MiB`; D0 must keep one CPU process and return a resource
reject rather than raising the bound after values open.

## Verification

| Check | Result |
| --- | --- |
| I0 focused suite | `PASS`, `5/5` |
| F0 + inherited V31 P1 and V32 M0/M1 suites | `PASS`, `21/21` |
| `cargo test -p xtask physical_sound_registry` | `PASS`, `161/161` matched tests |
| Ruff format/check on I0 owner and tests | `PASS` |
| Two complete external publications | `PASS`, exact files/stdout |
| Canonical profile, dependencies and relative links | `PASS` |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known `SOURCE_LAYOUT_ESCAPE_HATCH` in unchanged `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`; no I0 path is named |

The boundary diagnostic remains unchanged external debt. It is reported as a
failed mapped check, does not invalidate the isolated I0 evidence and grants
no ProductCheck or runtime credit.

## Consequence

I0 is closed and the exact owner identity above authorizes one D0 development
tournament under the frozen F0 profile. D0 may materialize train and
development roles in two fresh processes; method holdout must remain at exact
zero unless every development and hard gate passes. I0 grants no synthetic
quality pass, real-material or naturalness claim, validator release, admission,
cooker, demo, runtime or ProductCheck authority.
