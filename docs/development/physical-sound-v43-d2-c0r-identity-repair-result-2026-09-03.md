# Physical sound V43 D2/C0R identity repair result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / CORRECTED_CORPUS / B0R_R0R_AND_C1_AUTHORIZED / V0_STILL_BLOCKED / NO_AUDIO_OR_FEATURE_DECODE`

Planning authority: [Roadmap V43](../plans/physical-sound-synthesis-roadmap-v43.md)

## Outcome

D2/C0R applies the exact
[C1A repair plan](physical-sound-v43-c1a-lineage-audit-result-2026-09-03.md)
without editing D1 or C0 in place. It publishes a new immutable disclosed
roster, source-component graph, parent aliases, material quarantine, manifest,
role projections and a self-contained copy of every hash-verified C0 content
object.

The terminal decision is
`C0R_CORRECTED_CORPUS_REPEATABLE_B0R_R0R_C1_AUTHORIZED`. C0R may feed the
corrected baseline/domain audit and descriptor-source growth. It does not
authorize candidate training or validator calibration: only one independent
validator project/parent remains after repair.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v43_d2_c0r_identity_repair_v1.py`,
  `41,652 bytes`,
  `sha256=20264958c374d3a819d90c6076408ece19b19ebe36626414f7d25175f83e3ef9`;
- profile: `lab/profiles/physical-sound-v43-d2-c0r-identity-repair.v1.json`,
  `6,063 bytes`,
  `sha256=f817d51ff415a47afb1978f6511dff2b65629d34160c85e40a3a0507672323cc`;
- tests: `lab/tests/test_physical_sound_v43_d2_c0r_identity_repair_v1.py`,
  `18,745 bytes`,
  `sha256=a25491ca3cda08093c19329e8763dc4f5c854cfe7f97312b066d412c8ba098ac`.

The profile binds SPEC-45, the immutable C0 and C1A result records, the C1A
profile/owner, the D2/C0R owner itself, exact D1 roster, all eight C0 metadata
artifacts and all six C1A artifacts. Each PCM/feature object is additionally
bound by the source C0 manifest before byte copying. The living roadmap is not
a hash dependency.

## Applied repair

| Layer | Before | C0R |
| --- | --- | --- |
| Source component | AV-MSF separate from ObjectFolder/RealImpact | All three families share `source-component.realimpact-objectfolder-av-msf.v1` in train. |
| AV-MSF role | `20` validator-calibration records | All `20` are generator-train records. |
| Object `6` | Two current parent IDs across train/validator | One `realimpact-6-bowl--objectfolder-real-object-6` parent. |
| Object `80` | Two parents and conflicting `Wood`/`Ceramic` labels | One `objectfolder-real-object-80--av-msf-object-80` parent; all five labels become `Unknown`. |
| Object `80` material axis | Five observed `material_identity=true` rows | Five quarantined `material_identity=false` rows. |

The operation changes only role, source-component, physical-parent and the
explicit object-80 material quarantine. Acoustic target and PCM bindings are
unchanged.

## Corrected corpus shape

| Surface | C0 | C0R |
| --- | ---: | ---: |
| generator-train records / parents | `44 / 26` | `64 / 34` |
| generator-development records / parents | `70 / 30` | `70 / 30` |
| validator-calibration records / parents | `25 / 11` | `5 / 1` |
| all records / physical parents | `139 / 67` | `139 / 65` |
| material-identity-observed records | `139` | `134` |
| validator-calibration projects | `2` | `1` |

Material counts change only through quarantine: `Wood 18 -> 15`,
`Ceramic 12 -> 10`, and `Unknown 0 -> 5`. All other material counts remain
unchanged. C0R retains `135` identified recordings and `4` transfer rows.

## Content preservation and access

The source C0 and both C0R runs contain the same `278` content objects and
`40,746,497` content bytes. Their canonical sorted content inventory root is
identical:

`7e51b5ba3a647e2e8d63139eb71e5f2f1f4b1ff90b8b602aaffeef0f3f980a29`

D2/C0R verifies and copies bytes but never decodes them:

| Counter | Value |
| --- | ---: |
| content objects/bytes copied | `278 / 40,746,497` |
| C0 metadata bytes read | `488,797` |
| C1A bytes read | `12,812` |
| D1 bytes read | `14,236` |
| acoustic feature objects decoded | `0` |
| audio samples decoded | `0` |
| candidate model bytes | `0` |
| validator candidate outputs | `0` |
| protected payload bytes | `0` |
| network requests | `0` |

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v43-d2-c0r-2026-09-03`

Run A consumes D1/C0/C1A `run-a`; run B consumes their `run-b` counterparts.
Recursive comparison is empty. Each output contains `288` files and
`41,252,445` bytes.

| Inventory | Files | Bytes | SHA-256 root |
| --- | ---: | ---: | --- |
| complete C0R tree | `288` | `41,252,445` | `76c81b0f53349f5cebf4f2698e804c08052a58c92d2f80a4e15f8501b7460bca` |
| content objects | `278` | `40,746,497` | `7e51b5ba3a647e2e8d63139eb71e5f2f1f4b1ff90b8b602aaffeef0f3f980a29` |
| metadata | `10` | `505,948` | `448f1212e3aab007cfa4a95f8768893074ff011065e3d564190250a115a301c8` |

| Metadata artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access-ledger.json` | 1,399 | `a4d3aee105135ee923278cc6bd19e17c39d2f18092d52aad663585ee6e1136fe` |
| `corpus-card.json` | 1,575 | `582e57a6f13a0f794354f353fe8c166fa8190d7f591e7a5076ec9b056802a1ad` |
| `disclosed-roster.json` | 12,922 | `6ddb4df98a3f02ab12f94a9fa401614b9d370f620141bd6476b4020cc98ccbdc` |
| `lineage-repair.json` | 3,164 | `cde3359d9451f833fc4973bbbdfead4e82c7f4bcfac4618e717dcc0fc35655d4` |
| `manifest.json` | 297,789 | `5a3b8f76d8382a0855162146909fffb22aeb9a2ff325352ce4c41e57f8052efa` |
| `profile.json` | 6,063 | `f817d51ff415a47afb1978f6511dff2b65629d34160c85e40a3a0507672323cc` |
| `projections/generator_development.json` | 90,451 | `d58b5f236fef911089b49b596a45ddc10ae511db60cb38ab8bb8bd1a50e76c08` |
| `projections/generator_train.json` | 82,647 | `93d397c18c3c8df49f7f687caf79d8e346b7691c0ab1726dd3488bcd8f2bc00e` |
| `projections/validator_calibration.json` | 7,046 | `51ecfc0ecf41daf54efa95f370de9ed03689bef059368e0e13dbd434930909d2` |
| `report.json` | 2,892 | `28769b43a00a3a435b058daeaecbc77d79a47a85ebed7c0231c5b8cf71c12097` |

The D2 roster root is
`d82fe8d321ba8806d0331451d5e26ade8979ddbbb9dea01e0f654f1c72860742`;
the C0R manifest root is
`2539d5ac14b6b52616aaca9dcba0557e799afdd237b215581adb5f1447656cdd`.

## Verification and authority

- focused D2/C0R suite: `PASS`, `7/7` tests;
- combined D0/D1/C0/C1A/D2 lineage suite under pinned NumPy `2.5.2` and SciPy
  `1.18.0`: `PASS`, `39/39` tests;
- Ruff `0.14.1` format/check, Python compile and canonical profile validation:
  `PASS`;
- official external A/B, recursive byte comparison and source/C0R content-root
  comparison: `PASS`;
- content mutation, C1A-plan mutation, projection mismatch, output overwrite,
  repository-path, symlink and injected late-write paths reject before partial
  publication;
- `cargo run -p xtask -- boundary-scan`: `FAIL` on the pre-existing tracked
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  D2/C0R adds no source-layout escape hatch and receives no boundary-scan credit;
- runtime ProductChecks: `NOT_RUN`, because D2/C0R is external research tooling
  under SPEC-45 `Proposed` and changes no production consumer.

C0R authorizes only corrected B0R/R0R and C1 descriptor/disclosed-source growth.
V0 remains blocked on fresh independent validator-calibration power, and neural
training remains blocked on the later descriptor-signal gate.
