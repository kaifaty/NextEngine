# Physical sound V32 V0a — modal-equivalence validator result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / REPEAT_EXACT_PASS / SYNTHETIC_MECHANICS_ONLY` |
| Protocol | [V32 V0a modal-equivalence protocol](physical-sound-v32-v0a-modal-equivalence-validator-protocol-2026-09-02.md) |
| Profile | [`physical-sound-v32-v0a-modal-equivalence-validator.v1.json`](../../lab/profiles/physical-sound-v32-v0a-modal-equivalence-validator.v1.json) |
| Owner | [`physical_sound_v32_v0a_modal_equivalence_validator_v1.py`](../../lab/scripts/physical_sound_v32_v0a_modal_equivalence_validator_v1.py) |
| Rejected predecessor | [V0-v1 clean false positive](physical-sound-v32-v0-validator-mechanics-result-2026-09-02.md) |
| Product effect | None; authored clips remain authoritative and SPEC-45 remains `Proposed` |

## Result

V0a passes its frozen synthetic mechanics contract twice byte-identically:

```text
9 clean -> Pass
7 mutations -> Reject, each with its one frozen reason
2 lawful equal-PCM modal alias pairs -> Pass
plate PCM presented as beam -> Reject(RetrievalCopyDetected)
```

Decision: `V0A_MODAL_EQUIVALENCE_VALIDATOR_MECHANICS_PASS`.

The successor removes only copied clean targets whose complete ordered modal
signature exactly equals the requested target. It does not enumerate observed
record IDs, introduce a tolerance or change another specialist, threshold or
reason precedence. Candidate labels remain outside the feature view and cache.

This closes validator mechanics only. No real signal, material threshold,
quality model, protected role, admission rule, cooker, demo or runtime path was
opened or promoted.

## Exact publication

Both external runs publish the same 35 files and `49,396` bytes. Their complete
path-and-content maps and stdout are equal.

| Artifact | SHA-256 |
| --- | --- |
| Profile | `bb3e6cb2f6f0e52e80dc31df14f7887e9f3fc14ecca6e9a541d670b716118094` |
| V0a owner | `a946f96307dc01bccab6151ef2049796677210edd1ced9d2518e842630f49584` |
| Validator release | `267df19d17588be03812bea306d40e74db77f835239737e55b3b5fc128d828ee` |
| Evidence | `e775046cc8c4f27ef885c8eddf6dad518fac1b9dbc295aceeeb9455abde7ecd9` |
| Report / stdout | `4a20aef5728a603b8e99bee816b64ae8489172ab38009442fccb3ec78115f76f` |

The two runs took `1.77 s / 1.79 s` and reached `52,856 / 52,880 KiB` maximum
RSS, inside the frozen `300 s / 1 GiB` envelope. The evidence reports zero
network requests, model parameters and protected or real signal values.

## Decision matrix

| Candidate class | Result |
| --- | --- |
| Nine clean P1 renders | `9/9 Pass` |
| Density/scale and thickness/Young's alias members | `4/4 Pass`, empty copied-target list |
| Wrong decay | `PhysicalDecayMismatch` |
| Frozen carrier | `TemporalEvolutionFrozen` |
| Shuffled envelope | `EnvelopeOrderMismatch` |
| Mode collapse | `ModalCoverageCollapsed` |
| Plate-to-beam spectral copy | `RetrievalCopyDetected`, copied target `baseline-plate` |
| Clipping | `PcmClipping` |
| Provenance mismatch | `ProvenanceMismatch` |

Additional probes pass for `ModalParameterMismatch`, `IntegrityMismatch`,
`UnsupportedTargetCase`, reason precedence, ignored-label invariance, successor
cache separation, profile/dependency/truth corruption and atomic failure.

## Verification

| Check | Result |
| --- | --- |
| V0a focused suite | `PASS`, `8/8` |
| V24 T0 + V31 P0/P1 + V32 T0/V0/V0a inherited suites | `PASS`, `43/43` |
| Two complete external V0a CLI publications | `PASS`, exact 35-file maps and stdout |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known pre-existing `SOURCE_LAYOUT_ESCAPE_HATCH` in unchanged `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs` |

The boundary scan names no V0a file. Its JSON result remains `ERROR` even
though the command process returns zero; this is reported as a failed mapped
check and grants no ProductCheck credit.

## Consequence

V32 M0 protocol work is now sequenced. Before opening any model value it must
freeze one bounded correction family, controls, losses, causal hard gates,
resource oracle and one-shot tournament rule. V0a remains synthetic mechanics;
real validator qualification stays blocked by the unchanged source-power
frontier of six exact-Steel and 27 non-Metal groups.
