# Physical sound V44 G0A structured-source increment result

Date: `2026-09-03`

Status: `COMPLETE / REPEAT_EXACT / YCB_26_MASS_EXTENT_PARENTS / 20_PLATE_NUMERICAL_PROSPECTS / DESCRIPTOR_COMPLETE_STILL_ZERO / PSEL_BLOCKED`

## Question

Can published internet metadata enrich the corrected C0R corpus with
runtime-available physical descriptors and provide numerical controls for
Steel, Glass and Wood without reading waveforms, features, models, validator
data or protected data?

## Primary evidence

- The official [YCB Object and Model Set paper](https://www.ri.cmu.edu/pub_files/2015/7/ICAR-FINAL.pdf)
  publishes masses and major dimensions in Table I. The reviewed bridge uses
  `26` unambiguous current YCB model IDs. Current IDs `61`, `68` and `70` map
  explicitly to paper rows `60`, `66` and `69`; current IDs `60`, `65` and
  `71` remain excluded because the paper does not identify one unambiguous
  scalar row for the corpus object.
- Giordano and McAdams publish the controlled plate protocol and Table I in
  [Material identification of real impact sounds](https://www.mcgill.ca/mpcl/files/mpcl/blg_smc_2006_jasa.pdf):
  `2 mm` square plates, five areas, Steel/Glass/Wood/Plexiglass, suspended
  support, central steel-pendulum impact, and eleven numerical physical or
  acoustic columns. This yields `20` numerical prospects, including `15` for
  the three priority materials.
- The plate paper describes `48 kHz / 16-bit` recordings, but no authoritative
  public binding for those twenty waveform files was found. The rows are
  therefore structured numerical controls only, not waveform evidence,
  validator calibration data or candidate-training authority.

The exact external research inputs are:

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| `ycb-object-model-set-icar-2015.pdf` | `1,293,998` | `71aee0b490a5a19a1f885b9b502738241f01af24677973999e1fc307dabf0aa5` |
| `ycb-object-model-set-icar-2015.txt` | `51,165` | `f20d240365ad63aab7a0dc9f90efe8416b15fd4ad5b2ceda0a44a5e4a2a36467` |
| `giordano-mcadams-2006-plates.pdf` | `162,326` | `f30408f7a9b7a38273cf79fe02636c1e3ddfa82b6012dc069c3b7b2de8e0f5a2` |
| `giordano-mcadams-2006-plates.txt` | `86,230` | `42b99fb5c271661c47a3f45d39b790cf97c1617e5c698f7435beecb82e518b67` |

The two text files are deterministic Poppler extractions of the bound PDFs.
The adapter verifies both exact hashes and reviewed table markers. Source
license is deliberately `NOASSERTION`; copied evidence remains
`external_research_only` and is not a distributed content asset.

## Implementation

- [G0A owner](../../lab/scripts/physical_sound_v44_g0_structured_source_v1.py)
  verifies the frozen profile, C0R projections and four metadata artifacts,
  parses the plate table into integer fixed-point values and publishes only to
  a fresh external directory.
- [G0A profile](../../lab/profiles/physical-sound-v44-g0-structured-source.v1.json)
  binds the owner, architecture/roadmap context, corrected corpus projections
  and raw evidence hashes.
- [C1 successor profile](../../lab/profiles/physical-sound-v44-c1-runtime-descriptor-intake-g0-ycb.v1.json)
  binds the generated source manifest and reuses the unchanged frozen C1
  contract and intake owner.
- [Focused tests](../../lab/tests/test_physical_sound_v44_g0_structured_source_v1.py)
  cover canonical bindings, exact table parsing, repeat publication, direct C1
  manifest acceptance, source mutation, parent-set mutation, external-output
  guards and forbidden decoder/network imports.

The owner publishes:

- a C1-compatible source manifest with `26` parent-scoped mass and extent
  observations and two bound evidence artifacts;
- `20` plate prospects with fixed-point descriptors and published acoustic
  targets (`5` each for Glass, Steel, Wood and Plastic);
- source inventory, zero-forbidden-access ledger and fail-closed report.

It does not read C0R PCM or acoustic-feature object bytes. It only reads the
two bound role projections and the four metadata evidence files.

## Exact result

G0A run A and run B are byte-identical. Important roots from run A:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `manifest.json` | `22,258` | `9dddec2311510e40f69aabccc334cb12791404aaccc116661b3e13a8b0dac366` |
| `plate-prospects.json` | `24,973` | `28dffb1f8fcad7733cdf7c0879d5eca378a9a97dbc384194d602bbadf585eb86` |
| `source-inventory.json` | `2,106` | `e303cb49115eca49ca4bfc04e5cd76897d9c4c34473aca4b6c2de98399f8b3af` |
| `access-ledger.json` | `346` | `daf1e8b9e002512aed1e47ec43caa91cd28c237532c05ef111d5f2c188f7393f` |
| `report.json` | `1,787` | `ae12553dd782ab1f0c5fbab85aceba157dc73be3d9f4c20479485ee89902cfbc` |

G0A decision:

`G0_STRUCTURED_SOURCE_INCREMENT_REPEATABLE_PSEL_BLOCKED`

C1 successor run A and run B are also byte-identical and return the unchanged
decision `C1_DESCRIPTOR_CONTRACT_FROZEN_G0_AUTHORIZED`. Coverage changes from
the C1 zero-source census as follows:

| Field | Observed parents | Observed records |
| --- | ---: | ---: |
| `material_label` | `63` | `129` |
| `extent_micrometres` | `26` | `56` |
| `mass_milligrams` | `26` | `56` |
| each of the other six object/condition axes | `0` | `0` |
| all nine fields complete | `0` | `0` |

C1 report SHA-256 is
`74ecf8bcba404eb2d2b76b64402d1347fd79de25d1d6c61a8f1a4042945df6f5`;
coverage SHA-256 is
`070d9599f06acd9ff8923fe921b3ed8aaa6e9082099e5a9343a963c993cd4c6b`.
All PCM, acoustic-feature, candidate-model, validator, protected and network
access counters are zero.

## Conclusion and decision

The internet-only intake works and adds useful physical scale information to
existing parents without inventing missing facts. This is a real G0 increment,
but not G0 completion:

1. C1 intentionally enriches only parents already present in C0R; it cannot
   create new physical parents from a paper.
2. YCB Table I does not publish wall thickness, shape topology, cavity/opening
   topology or record-level support/impact zone for these recordings.
3. The plate control is one publisher project and has no bound waveform
   correspondence. Treating its twenty rows as independent projects or
   training samples would overstate source power.
4. Descriptor-complete C0R coverage therefore remains `0`, the corrected
   `105`-supported-parent target remains unmet, and signal-blind PSEL cannot
   select the first material pack.

G0A is frozen as a reusable metadata source. G0B is next: define a separate
prospective-parent/corpus-growth boundary, then search for at least two
independent internet projects with physical descriptors plus bound waveform or
transfer-response correspondence for the same objects. PSEL and B1 remain
blocked; authored fallback remains the only production path.

## Verification

- Ruff `0.14.1` format/check: `PASS`.
- `python -m unittest` for G0A and C1 focused suites: `PASS`, `16/16` tests.
- External G0A run A/B recursive diff: `PASS`, no differences.
- External C1 successor run A/B recursive diff: `PASS`, no differences.
- Architecture boundary scan: `ERROR` on the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  G0A adds no source-layout escape hatch and receives no scan credit.
- ProductCheck: `NOT_RUN`; this is an external experimental metadata owner with
  no production consumer or public contract promotion under Proposed SPEC-45.
