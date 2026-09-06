# Physical sound V37 T0 exact truth protocol result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Decision | `T0_EXACT_TRUTH_PROTOCOL_PASS` |
| Scope | Value-free closure of the V37 synthetic truth generator and composite pre-access seal |
| Official target access | `0` train, `0` development, `0` method-holdout rows |
| Prior execution seal | `608f901938c4031e63353a9231e005bb1040ac21c50ed3c7746e01525e211d7f` |
| Composite input seal | `7bae65cc0290c3ec54eedef5d9ef2dd3d8e4ab1881945db71c8c80dcd1361573` |
| External A/B evidence | `/tmp/nextengine-v37-t0-official.KmVWTM` |

## Result

T0 closes a pre-access protocol defect without spending any official V37 role.
F0 had frozen fresh truth family names, component seeds, mixture identities and
scientific gates, but did not specify the coefficient derivation, exact
normalizations or reduction order needed to produce one reproducible target
array. It also did not publish the exact alignment from canonical `row_id` to
the hidden operator-mixture weights.

The omission made official D0 unsafe: two otherwise conforming implementations
could have produced different targets, and choosing the missing details during
D0 implementation would have introduced researcher discretion after the
scientific family had been selected. No target formula was evaluated and no
official capability was issued before this finding.

The repair is a supplement rather than a rewrite of F0 or E0:

- F0 remains the immutable fresh role/science identity freeze;
- A0–E0 structural, terminal, determinism and cost conclusions remain valid;
- T0 adds one exact executable truth protocol and binds it to the prior E0
  execution seal in a new composite input seal;
- official D0 must verify the composite seal before it can issue its sole
  capability or evaluate any train/development target.

## Frozen truth protocol

[The T0 profile](../../lab/profiles/physical-sound-v37-t0-exact-truth-protocol.v1.json)
now fixes all previously ambiguous numerical choices:

- a domain-separated SHA-256 PRF maps each pre-existing F0 seed, coefficient
  group and index to IEEE-754 binary64 through an exact `u53` construction;
- 72 derived coefficients have commitment root
  `07932a8069157b0b62eb49fe166f94f857d5b7d717a1fea86636d18091070ace`;
- field span, area normalization, modal RMS, node/query coordinates and
  canonical reduction order are explicit;
- local anisotropic integral, two-hop symmetric graph diffusion and rank-four
  field/query interaction are executable formulas;
- decay, global-gain and contact axes have exact order and bounds;
- mixture weights remain hidden truth metadata, not model inputs, but every
  canonical row has an immutable alignment commitment.

The full target-free row metadata closure is:

| Role | Rows | Commitment root |
| --- | ---: | --- |
| Train | `6,480` | `9906ea4490669558691503b9aca5d5291117229eb3143cfcb051bd97a9c7bda0` |
| Development | `4,320` | `c75e9bbedf97fd45f8b905928e3a516f62b0ca3a3629f87c38852889f9f6bd00` |
| Method holdout | `4,320` | `95d5a300151ea1a2dee7f129c9770ed9729ff3a2ae298dfc782b819de47af1df` |
| All roles | `15,120` | `48508bf0226e4d3894d577bc0f77affb7f71d4bc6b0c4864cd5b6a4c52bdf9b0` |

Enumerating these identities does not construct or evaluate a target. The
method-holdout target counter therefore remains exact zero.

## Value-free conformance

[The T0 owner](../../lab/scripts/physical_sound_v37_t0_exact_truth_protocol_v1.py)
executes the exact target path only on a four-row artificial field that is not
part of any official role. Its immutable target root is
`6dbfe6ada1cd8b995e7960df3d18281acf5d8b6f58c0117f4c5ac23ceb9d758c`.

The conformance suite proves:

- canonical source/query permutation preserves targets byte-for-byte;
- changing a modal probe value changes the result;
- changing a symmetric graph edge changes the result;
- changing the hidden mixture changes the result;
- all three target axes are finite and inside their frozen bounds;
- labelled input, row/metadata misalignment, ambiguous PRF semantics,
  dependency drift and occupied publication targets reject before output.

Component roots on the artificial fixture are:

| Component | Root |
| --- | --- |
| Local | `72f717856393cd189eb52b9ca8ff515d2ba0cc17ce8e4c59c17e717b9fa551df` |
| Diffusion | `5ad31ca7ac7c4277904d28b5433fbc25fc44565fd5d1eb825d7997673b761dc7` |
| Global | `b9401dfa1e283c48f1ae4a1e9b5c774c514cf9ccee5aa646c2bb57b7ca2b2a1c` |

## Composite pre-access seal

[The checked seal](../../lab/profiles/physical-sound-v37-t0-official-input-seal.v1.json)
binds:

- the prior repeat-exact E0 execution seal;
- the exact T0 profile and current T0 owner;
- all direct profile/code dependencies;
- the same Python/NumPy/Torch CPU environment already frozen by E0;
- the 72-coefficient commitment;
- all three official row-metadata commitments;
- the artificial conformance root and target-axis order;
- zero forbidden access.

Its self-hash is
`7bae65cc0290c3ec54eedef5d9ef2dd3d8e4ab1881945db71c8c80dcd1361573`.
Any owner, profile, dependency, formula, coefficient, metadata or previous E0
seal drift invalidates it before official target construction.

## Independent A/B evidence

Two fresh processes produced identical stdout, empty stderr and identical six
file trees:

| Run | Wall | Peak RSS |
| --- | ---: | ---: |
| A | `1.72 s` | `618,144 KiB` |
| B | `1.75 s` | `617,608 KiB` |

Important file hashes are:

| Artifact | SHA-256 |
| --- | --- |
| Artificial conformance | `a14b82f124a350d38f86df8f8984a07dbd1ff18109d4b53b7e14a6fb1c9ac911` |
| Coefficient commitment | `be6dd94a5d0b319aaad0d98f7e5c031d03b7464f273915fcdedb3050686b1725` |
| Row metadata commitments | `1283216a8e26dda592957a4e82efceba415ffc0dbfc5ae118873785dcf6d2f71` |
| Checked seal file | `9a4ed1c9193e793288c8db93a8b8cf46280b75104821bc04a3b0cf0317696f52` |
| Report/stdout | `7af19c476b9b253e5517a492e46bc2bfb6cc220fbdda3eed5f6e271d7245fd45` |

Peak RSS includes import of the already sealed PyTorch/C0 dependency closure;
T0 itself performs no model initialization or training.

## Verification

- Ruff `0.16.3` format/check and Python compile: `PASS`;
- strict mypy `2.0.0` with skipped dependency imports: `PASS`;
- focused T0 tests: `7/7 PASS`;
- combined V37 Python tests in the frozen NumPy `2.3.5` / Torch `2.8.0`
  environment: `52/52 PASS`;
- focused xtask physical-sound registry tests: `161/161 PASS`;
- `cargo run -p xtask -- boundary-scan`: expected repository-baseline `FAIL`
  only on unchanged
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no Cargo ProductCheck applies before a production cooker/demo consumer.

## Authority and next action

T0 proves reproducible target semantics, not scientific quality. It grants no
real-material, validator, admission, cooker, demo or runtime claim. SPEC-45
remains `Proposed`, and authored clips remain mandatory fallback.

The only newly authorized action is implementation of a provider that verifies
both the E0 and T0 composite seals before issuing one D0 capability. That owner
may then materialize exactly the frozen train and development roles in the
predetermined A/B execution. H0 stays unavailable unless D0 naturally returns
repeat-exact `Pass`; any other terminal result closes QSO-v0 without repair or
retry.
