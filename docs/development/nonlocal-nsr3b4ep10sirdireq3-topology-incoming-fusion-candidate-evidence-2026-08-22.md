# NSR3-B4EP10SIRDIREQ3 topology/incoming fusion candidate evidence -- 2026-08-22

Status: `FAIL / CPU_GATE / CANDIDATE_REVERTED / SIRDI_RETAINED`

## Exact stage

The direct-publication implementation is numerically and structurally exact.
Both final fresh candidate processes exit zero, emit empty stderr and are
byte-identical at stdout SHA-256
`a0df3c88145110728ccf342cc961c3a1c1aac60c28e5835ff44cf07f11978ba9`.
Their result is `dc5089bb...9b59`, positive-control result
`e43bb804...41ce` and positive report SHA-256 `198a720e...375b`.

All five SIRDI physics roots, query chain, physical work, retention and
directed-scratch counts remain exact. The candidate records:

| Exact item | Result |
|---|---:|
| builds / publications / forwardings / consumptions | 226 each |
| current-pair visits | 85,716,150 |
| degree/source/endpoint/incoming/read/target entries | 150,845,996 each |
| parallel regions | 3,411, down from 4,089 |
| logical partitions | 218,304, down from 261,696 |
| maximum plan / scratch / added bytes | 5,409,132 / 3,138,680 / 8,547,420 |

Order, coverage, ownership and fallback failures are zero. Missing-endpoint
and target-cursor injections both fail at `FRAME_START_WORKSPACE`, before any
evaluation, HVP, publication or fallback. A separate SIRDI regression remains
byte-exact at stdout SHA-256 `539f1ec5...e7e7`.

## CPU-work stage

The single frozen `CLOCK_PROCESS_CPUTIME_ID` experiment fails its predeclared
gate:

| Pair order | Baseline CPU ns | Candidate CPU ns | Paired speedup |
|---|---:|---:|---:|
| AB | 30,153,297,332 | 30,648,444,097 | 0.983844x |
| BA | 39,084,945,660 | 30,338,118,951 | 1.288311x |
| AB | 30,927,209,734 | 37,578,710,950 | 0.822998x |

The candidate wins only `1/3`; median paired speedup is `0.983844x` versus the
required `1.03x`, and paired-speedup range ratio is `1.565388` versus the
maximum `1.10`. Every timed baseline/candidate report is exact, clock
resolution is 1 ns and stderr is empty. CPU report result is
`5a18e299...5ba0`, raw JSON SHA-256
`35e0579bc942e1a2419699a049cff6aeb7393e578cd850443b5ef1f3701cecf4`.

The 30--39 s sample spread means this experiment cannot distinguish a small
candidate effect from shared-host frequency, memory-contention and OpenMP
wait-work variation. It also shows why the Q2 `0.128869x` structural ratio
must not be translated into solver speed: entry visits and regions do not
have equal CPU cost. Per contract, no rerun or threshold change is admitted.

An earlier harness invocation stopped before all measured samples because it
compared internal JSON without its final newline against the stdout hash that
includes the newline. That representation bug was corrected; its six sample
durations were all zero, so it is not an observed A/B. The stopped artifact is
under `probe-nonlocal-b4ep10sirdireq3cpuab.tD7zI1`.

## Reproducibility and rollback

The exact candidate implementation is commit
`517c9ad6b21ad4322237e5cb1244fea1154f85b6`; CPU harness checkpoint is
`237fc12737fce54094ff5147f75c68f0c83c4def`. The measured Release executable
has SHA-256
`a89e96be19256cebc81bee71ef4f935c7525016f98e307462c06bc3799fa04ac`,
size 4,467,784 bytes and ELF Build ID
`103762ca569ccd1869387c976bf6243e922c7e4c`. Its
`compile_commands.json` SHA-256 is
`79e135ca1ecbb1d76a862c944b574472ecce6fe4141684590f2da33488f03594`.

Measured source SHA-256 values are
`abf687c25c6fc045e496b6c3d11c39631aebf6715e8b87fb9916b2892758f411`
for `boundary_reference.cpp`,
`8b2349c0fef3f44a3f1c6afee1d503929598b5b2b59f1573627aeda13959213c`
for `boundary_reference.hpp` and
`c556372a27e97c41134616af7e34a0281975ccff6766cb8ca66176ee7ffbfba1`
for `formula_reclosure_main.cpp`. Exact raw reports are
under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdireq3-exact.4fqlwc`;
the admitted CPU report is under
`/home/kaifaty/.cache/nextengine/external/probe-nonlocal-b4ep10sirdireq3cpuab.sgTAyf`.

The CPU harness and candidate are reverted by commits
`9315b8e106fa9eb0d05e9689050100f2701612be` and
`40948c39c77d80b837ac6500916636844486c0c5`. The rebuilt rollback preserves
Q2 stdout SHA-256 `44d3279f...8ba9`
and SIRDI stdout SHA-256 `539f1ec5...e7e7`, both with empty stderr; artifacts
are under
`/home/kaifaty/.cache/nextengine/external/regress-nonlocal-b4ep10sirdireq3-rollback.Wh2OOW`.

## Decision

Close Q3 FAIL, retain SIRDI and keep Q2 only as structural evidence. The
candidate is not selected for nominal research and grants no reprofile, wall
speed, B4E2, broad-corpus, runtime/GPU/schema or production authority.

Before another performance implementation, research a measurement-lane
qualification that can separate useful worker CPU from wait/spin and shared
memory/frequency interference. Do not rerun this candidate under Q3.
