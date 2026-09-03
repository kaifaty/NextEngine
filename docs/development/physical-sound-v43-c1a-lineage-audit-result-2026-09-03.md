# Physical sound V43 C1A lineage audit result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / C0_IDENTITY_REPAIR_REQUIRED / D2_C0R_NEXT / V0_BLOCKED_BY_IDENTITY_AND_POWER / NO_ACOUSTIC_OR_MODEL_ACCESS`

Planning authority: [Roadmap V43](../plans/physical-sound-synthesis-roadmap-v43.md)

## Outcome

C1A turns the preliminary source-lineage concern into a hash-bound,
repeat-exact result. The official [AV-MSF paper](https://arxiv.org/abs/2608.05145)
declares evaluation on ObjectFolder Real and RealImpact. The frozen
[AV-MSF project page](https://zisenshao.github.io/AV-MSF/) and
[ObjectFolder Real table](https://objectfolder.stanford.edu/objectfolder-real-download)
identify matching object IDs `6` and `80`, while the current C0 corpus places
their AV-MSF and ObjectFolder representations in different roles.

The audit therefore returns `C0_IDENTITY_REPAIR_REQUIRED`. It does not decode
audio, open acoustic feature objects, inspect generator/validator candidate
outputs, access protected payloads or make network requests. It authorizes only
an immutable D2/C0R role and parent repair.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v43_c1a_lineage_audit_v1.py`,
  `29,385 bytes`,
  `sha256=a8686afde4c442927a34c111ec580fa48cdb8687affcb4d1007a82a5773057fe`;
- profile: `lab/profiles/physical-sound-v43-c1a-lineage-audit.v1.json`,
  `5,239 bytes`,
  `sha256=b9f99be5a540c83f10ab3d690e27c9c4a76687795ee651cd0beea9b3a35d71d9`;
- tests: `lab/tests/test_physical_sound_v43_c1a_lineage_audit_v1.py`,
  `15,702 bytes`,
  `sha256=17d6fde069a36ecd75435ecd3e123ab1085b79a4492da4ac298b4981db8a970f`.

The profile binds SPEC-45, the immutable C0/R0 evidence records, the tracked C0
profile, the owner itself, all five exact C0 metadata inputs and eight external
official-evidence objects. The living roadmap is intentionally not a hash
dependency, so advancing its milestone status cannot break replay. The captured
web/PDF/image/video bytes are identity evidence only; C1A hash-checks them and
parses only the frozen textual dataset/object/material declarations. It does
not infer acoustic quality from images or decode media streams.

## Exact findings

| Object | Current evidence | C1A finding | Required D2 action |
| --- | --- | --- | --- |
| `6 / Blue_Bowl` | Five C0 records occupy `generator_train` and `validator_calibration`; both sources label it `Glass` | `ConfirmedPhysicalAlias` | Merge both current parent IDs into `realimpact-6-bowl--objectfolder-real-object-6`. |
| `80 / Spoon_Holder` | Five C0 records occupy both roles; ObjectFolder says `Wood`, AV-MSF says `Ceramic` | `QuarantineIdentityConflict` | Merge into `objectfolder-real-object-80--av-msf-object-80` and mask/quarantine material supervision. |

AV-MSF is a derived evaluation view of already disclosed source families, not
an independent validator project. The fail-closed repair plan moves all `20`
AV-MSF records from `validator_calibration` to `generator_train`; it does not
cherry-pick only the two confirmed aliases.

## Corrected planning shape

These are C1A-computed D2 inputs, not a claim that C0R already exists:

| Surface | Current C0 | Expected after D2/C0R |
| --- | ---: | ---: |
| generator-train records | `44` | `64` |
| generator-development records | `70` | `70` |
| validator-calibration records | `25` | `5` |
| generator-train physical parents | `26` | `34` |
| generator-development physical parents | `30` | `30` |
| validator-calibration physical parents | `11` | `1` |
| all physical parents | `67` | `65` |
| validator-calibration projects | `2` | `1` |

The corrected parent-role graph is disjoint, but one validator project/parent
has insufficient independent power. V0 is therefore blocked pending V0S/V0P
source growth. Old B0/R0 numbers remain exact for the old train/development
surface, but a future B1 must use a new B0R/R0R floor derived from C0R.

## Access and authority

Both official runs record:

| Counter | Value |
| --- | ---: |
| C0 metadata bytes read | `484,331` |
| frozen public identity-evidence bytes read | `23,291,561` |
| acoustic feature objects read | `0` |
| decoded audio samples | `0` |
| candidate model bytes | `0` |
| validator candidate outputs | `0` |
| protected payload bytes | `0` |
| network requests | `0` |

The terminal artifact grants no product, public-contract, candidate-training,
descriptor-baseline, validator-calibration, admission or runtime authority.
Authored fallback remains mandatory.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v43-c1a-lineage-audit-2026-09-03`

`run-a` consumes C0 `run-a` plus source-evidence `source-a`; `run-b` consumes
their independently captured/reproduced `run-b`/`source-b` counterparts.
Recursive comparison is empty. Each output contains six files and `12,812`
bytes. The SHA-256 of the canonical sorted `path / bytes / sha256` inventory is:

`e11341f3b1ec5ccfe7712df325ded2ba34f0401a98a23785afb025682497510b`

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access-ledger.json` | 1,018 | `5258b6df4873c79ee3104af7bf933bfdb5a606ad0ee8bc7363203c3ee520d9cb` |
| `alias-findings.json` | 1,378 | `ca45f3544a6ff6e6ef455e7877e41225c320d5f058d418b6c1566a031eca0937` |
| `evidence-inventory.json` | 1,842 | `d58cde0a031278c18d77388f84ebd2aa532ce0706218950bba955dc8d2cbe6d5` |
| `profile.json` | 5,239 | `b9f99be5a540c83f10ab3d690e27c9c4a76687795ee651cd0beea9b3a35d71d9` |
| `repair-plan.json` | 1,476 | `0f956d9ca15a100f66c3f8c3174f3b3c4896a275da2da0a58e17857d5cfc815d` |
| `report.json` | 1,859 | `a133e028e2afe33f994bbe7367ac0edb6c2a9488f7c4ae20a05ff9f73d48ed10` |

The report binds C0 manifest root
`e375fa63c0a514da52bf3c3f38d73a464c42fd33f61bd899f6f0e52f3e9280a0`
and public evidence root
`a0133675e8a820fe6ca4122d75b04a935b665273f680933d69f177d852848d51`.

## Verification and next authority

- focused C1A suite: `PASS`, `7/7` tests;
- combined D0/D1/C0/C1A lineage suite under pinned NumPy `2.5.2` and SciPy
  `1.18.0`: `PASS`, `32/32` tests;
- Ruff `0.14.1` format/check, Python compile and canonical profile validation:
  `PASS`;
- official external A/B and recursive byte comparison: `PASS`;
- mutation coverage rejects evidence bytes, missing dataset declarations,
  material drift and projection/manifest mismatch before publication;
- output overwrite, repository path, symlink and injected late-write paths fail
  without a partial result;
- `cargo run -p xtask -- boundary-scan`: `FAIL` on the pre-existing tracked
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  C1A adds no source-layout escape hatch and receives no boundary-scan credit;
- runtime ProductChecks: `NOT_RUN`, because C1A is external research tooling
  under SPEC-45 `Proposed` and changes no production consumer.

D2/C0R is now the only authorized next corpus action. C1, B0R/R0R, V0,
generator training and cooker work remain blocked until the immutable corrected
role projection exists.
