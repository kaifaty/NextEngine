# V28 R1 result — M0c prefix-bounded owner and resource gate

| Field | Result |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / PREFIX_EQUIVALENCE_PASS / FULL_ENTRY_REPEAT_EXACT_PASS / RESOURCE_A_PASS / RESOURCE_B_PASS / NORMALIZED_REPEAT_PASS / OFFICIAL_VALUES_SEALED / R2_AUTHORIZED` |
| Protocol | [V28 R0](physical-sound-v28-r0-m0c-prefix-resource-protocol-2026-09-02.md) `28cab4283e9ea0178b59100960245d27e04c5e749842dd87fa011ec788ca9000` |
| Protocol commit | `c7faddea` |
| Implementation commit | `d316982dc92b1a2c6e86d33928a913155a42015e` |
| Implementation root | `524411d3c299420cfebebf63b2d66b1c444847f60f42b67218446203825f6d32` |
| Allowed claim | The fresh M0c execution owner preserves the frozen synthetic objective and completes its declared training surface reproducibly inside the resource envelope |
| Not claimed | Model quality, naturalness, Metal/Glass admission, arbitrary-force transfer, runtime authority or any interrupted M0b value |

## Outcome

R1 passes. M0c changes only the differentiable synthetic render horizon from
the stored `144,000` frames to the maximum frozen loss window, `4,096`. Exact
fixtures preserve the rendered prefix, loss components and every parameter
gradient. The complete M0c entry is repeat-exact, and two isolated
official-shape resource runs finish all `10,000` optimizer steps with large
wall/RSS margins.

This closes only the resource/equivalence gate. The official feasibility model
has not been run, constructed or inspected. Roadmap V28 R2 is now authorized
under its fresh manifest and one-use A/B order.

## Frozen implementation

New owners:

- `lab/scripts/physical_sound_v28_m0c_common.py`;
- `lab/scripts/physical_sound_v28_m0c_model.py`;
- `lab/scripts/physical_sound_v28_m0c_resource.py`;
- `lab/scripts/physical_sound_v28_m0c_train.py`;
- `lab/tests/test_physical_sound_v28_m0c_train.py`.

The M0c model module explicitly aliases the frozen M0a representation and owns
only a prefix-bounded `synthetic_loss`. The M0c entry binds that model and new
schemas into the frozen M0b owner for the duration of one call, then restores
every inherited module binding even on failure. No M0a/M0b source changed.

Inherited hashes were rechecked exactly, including:

- M0a common/model/train `592b3813…3e89` / `6075b412…0a47` /
  `86ada581…63b5`;
- M0b common/surface/alignment/preprocess/train `5811796e…9624` /
  `c1279901…1705` / `1452848d…8314` / `12bf5727…5ced` /
  `0c4492b7…d40c`.

## Gate E — exact prefix equivalence

The external CLI ran twice in fresh directories under fixed one-thread CPU
settings. Both canonical reports are byte-identical:

| Field | A | B |
| --- | --- | --- |
| Status | `Pass` | `Pass` |
| Cases | `8` | `8` |
| Full/prefix frames | `144,000 / 4,096` | same |
| Random seeds | `3101..3105` | same |
| Bound rows | negative / zero / positive | same |
| Render/loss/components/gradients exact | all | all |
| Late mutation exact | `true` | `true` |
| Early mutation detected | `true` | `true` |
| Invalid short/non-finite/non-positive cases reject | all | all |
| Report SHA-256 | `04f6927070297b969d29200207b6e3e461ffc01d5a8cb12335a543f38c82e402` | same |

Evidence root:
`/tmp/nextengine-v28-m0c-equivalence-jrCGsW`.

The test-only contract teacher audio was zero-padded from its `0.02 s` fixture
length to `4,096` frames and its local evidence/combined hashes were rebuilt.
This changes no official artifact and supplies no quality claim; it only lets
the inherited complete-entry fixture satisfy R0's minimum target contract.

## Gate C — complete owner conformance

The M0c full entry ran twice from identical fixture inputs into fresh outputs:

- all nine canonical artifacts reproduce recursively byte-for-byte;
- M0c report/experiment/protocol/root identities are bound;
- surface and padded-alignment reports remain inherited;
- candidate A/B bytes are exact internally;
- the contract fixture reaches its declared no-quality `PASS` path;
- M0b rejects the M0c schema;
- bad protocol, implementation root and inherited hash reject before work;
- forced MLflow failure leaves no canonical or abandoned staging output;
- inherited M0a training owner is restored after M0c returns.

The focused combined suite reports `14 tests / PASS` in `45.890 s` across M0a,
M0b and M0c.

## Gate R — two official-shape resource runs

Both runs used fixed random fixture values only: 32 synthetic examples with
stored `144,000`-frame targets, three transfer contexts, two recordings, five
variants, `7,500` synthetic and `2,500` real/replay steps. They ran as fresh
systemd user scopes with `MemoryMax=4G`, `MemorySwapMax=0`,
`IPAddressDeny=any`, one thread and a `600 s` outer timeout.

| Metric | Run A | Run B | Gate |
| --- | ---: | ---: | ---: |
| Completed optimizer steps | `10,000` | `10,000` | exactly `10,000` |
| Internal candidate A/B exact | `true` | `true` | `true` |
| Workload wall | `164.616 s` | `155.428 s` | `<=600 s` |
| Internal peak RSS | `1,010,020,352 B` | `1,007,652,864 B` | `<3,758,096,384 B` |
| `/usr/bin/time` elapsed | `168.07 s` | `158.86 s` | diagnostic |
| `/usr/bin/time` max RSS | `1,068,800 KiB` | `1,082,684 KiB` | diagnostic |
| Swap / socket messages | `0 / 0` | `0 / 0` | diagnostic |
| Official values opened | `false` | `false` | `false` |
| Model/loss values published | `false / false` | `false / false` | `false / false` |

After removing only the preregistered `wall_seconds` and `peak_rss_bytes`
diagnostics, both canonical reports have SHA-256
`078ff4034c2bd1309b970d396ac27d56ed05c65eab131e485a355b9ebdbb6af6`
and compare equal.

Evidence root:
`/tmp/nextengine-v28-m0c-resource-faIgcI`.

## Verification

| Check | Result |
| --- | --- |
| `uvx ruff check` on new modules/tests | `PASS` |
| `uvx ruff format --check` on new modules/tests | `PASS` |
| Python compile on new modules/tests | `PASS` |
| M0a + M0b + M0c focused suite | `PASS`, `14/14` |
| Gate E external A/B canonical comparison | `PASS`, byte-exact |
| Gate R external A/B normalized comparison | `PASS`, `10,000/10,000` twice |
| Inherited implementation hashes | `PASS`, unchanged |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known unrelated `SOURCE_LAYOUT_ESCAPE_HATCH` in `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs` |

The boundary finding predates and is outside the M0c Python/docs surface. It is
reported unchanged and does not weaken any M0c gate.

## Access audit

- No combined/T0/X0/official/protected source was read by Gate E or Gate R.
- No interrupted R1 model, checkpoint, metric or partial training value was
  opened.
- Gate C used generated contract fixtures only.
- Admission shadow and RealImpact row `2407` remain sealed.
- External resource reports contain only identity, workload counts, timing/RSS
  and booleans; no weights, predictions, gradients, losses or audio hashes.
- No dataset, PCM, checkpoint, generated audio or external report enters Git.

## Decision

`R1 = PASS`. Freeze implementation commit `d316982d` and root
`524411d3…6d32`. Do not alter M0c from later results.

The smallest next action is V28 R2:

1. construct a fresh external M0c official manifest binding the same official
   source hashes, protocol `28cab428…9000` and implementation root
   `524411d3…6d32`;
2. run official A once under `900 s`, `4 GiB`, no swap/network and one thread;
3. run B only if A publishes a canonical terminal report;
4. compare canonical artifacts and publish one terminal feasibility result.
