# Physical Sound V29 Q1a-M — strict-Steel internet source-growth protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / TWO_PHASE / METADATA_ONLY / ZERO_SIGNAL / WHOLE_PROJECT_POWER` |
| Research | [Q1a research](physical-sound-v29-q1a-metal-source-growth-research-2026-09-02.md) |
| Roadmap | [V29 Q1a-M](../plans/physical-sound-synthesis-roadmap-v29.md) |
| Baseline | Q1-M closure commit `1b272d87` |
| Product effect | None; authored clips remain authority and Q2/P2 stay blocked. |

## Question and bounded claim

Q1a-M v1 asks whether eight explicitly preregistered Freesound projects can
close Q1-M's raw source deficits and whether the combined projects can populate
both protected evaluations while reserving five whole projects for the other
roles.

It may capture and normalize publisher HTML metadata, verify current
repository exposure, preserve license/provenance labels and solve project-level
power. It cannot request an API credential, open audio/preview bytes, parse an
audio header, inspect a waveform, assign a role, train a model or relax Q1-M.

## Frozen inputs

The owner reads:

1. `lab/profiles/physical-sound-v29-q1a-source-growth.v1.json`;
2. Q0-M `metal-source-inventory.json`, exactly `172,172` bytes with SHA-256
   `9a04c8a2fb9f0c6a3d351dac2796936d933d223e9400afc79a95f06e01665f43`;
3. Q1-M `metal-role-power-audit.json`, exactly `72,478` bytes with SHA-256
   `f98e80c7ad21fd698718d2e31c858b3b4d53f386c69356b5252397a1d3e20944`;
4. the exact Git tree at baseline commit `1b272d87` for candidate-project URL
   exposure search;
5. one bounded public HTML response for each preregistered pack and sound URL.

All heavy/raw responses and generated outputs remain outside the repository.
Inputs must be regular files below the configured external roots. Symlinks,
duplicate JSON keys, unknown fields, hash drift, redirects away from the exact
HTTPS URL, non-HTML content, oversized pages and output replacement fail
closed.

## Two phases

### Capture

`capture` performs one GET for each exact URL with a declared user agent and a
`1 MiB` response ceiling. It stores raw HTML plus `capture.json` in a fresh
external directory. The manifest binds URL, final URL, byte count, SHA-256,
content type, baseline commit and exact baseline Git matches.

Capture follows no preview, CDN, download, waveform, image or API link. A
network error terminates without manufacturing a partial source revision.
Capture is acquisition evidence and is not expected to be byte-stable across
time because public pages contain dynamic counters and CSRF state.

### Audit

`audit` is network-disabled. It reads one frozen capture, removes all dynamic
page fields and emits canonical normalized identities. Official audit A and B
must be recursively byte-identical.

Each normalized project revision binds:

- exact author and pack ID;
- pack title and canonical URL;
- selected sound ID, uploader and exact pack membership;
- publisher title and description used for material/object/action evidence;
- per-sound license URL and label;
- raw page identities and one normalized project-revision SHA-256.

## Material and object policy

The only positive relation is `exact_steel_candidate`. Publisher text must
bind an unqualified whole-word `steel` to the declared sounding object. The
following do not count:

```text
stainless steel
carbon steel
zinc-plated steel
iron
aluminium
generic metal
tool-only steel
```

`Stainless Steel` remains an exact publisher label in the evidence but maps to
`other_metal_candidate`. Non-Metal groups require an explicit material-bearing
object phrase and impact action. Repeated recordings of one object remain one
group. Dimensions that identify separate plates/rods may define separate
groups within the same indivisible project.

The Bibow `426850` control must be excluded because its title says Steel while
its description says Aluminium. Description conflict outranks a convenient
filename. Any unexpected new conflict fails the project closed.

Allowed future-payload license families are CC0 and attribution-only Creative
Commons. BY-NC, incompatible, absent or unknown terms remain metadata-visible
but role-ineligible. Q1a redistributes no source page or signal.

## Current exposure ledger

For every project and sound, capture searches exact canonical identities in
the baseline Git tree. Audit also checks that no candidate publisher/project
identity exists in Q0-M or Q1-M. A match is reported and makes the project
ineligible; absence certifies only:

```text
current_repository_metadata_audited / signal_unopened_by_q1a
```

It is not a claim that no person on the internet has heard the sound. The
ledger records metadata access separately from protected signal access.

## Ordered gates and decisions

Audit evaluates:

1. input/profile/schema/hash and zero-signal closure;
2. complete capture with canonical URL/author/pack/sound membership;
3. compatible license metadata and exact material/object/action evidence;
4. no candidate overlap with baseline/Q0/Q1 identities;
5. at least seven added role-capable projects;
6. at least nine added strict-Steel groups;
7. at least one added non-Metal group, so aggregate rejects exceed `70`;
8. combined raw totals at least `32` Steel and `70` rejects;
9. two disjoint protected project sets, each with at least two projects and
   `16/35`, leaving at least five projects untouched.

If gates 1-8 pass but gate 9 fails, emit:

```text
Q1A_RAW_GROWTH_VERIFIED_BALANCED_ROLE_POWER_REQUIRED
```

If gate 9 passes, emit:

```text
Q1A_BALANCED_SOURCE_POWER_FEASIBLE_FRESH_Q1_NEXT
```

Any earlier insufficiency emits:

```text
Q1A_SOURCE_GROWTH_INSUFFICIENT
```

Only the second decision may route to a fresh Q1 release. No Q1a decision
assigns roles or authorizes payload access.

## Outputs and access closure

Capture writes:

```text
raw/*.html
capture.json
```

Audit writes:

```text
metal-source-growth-audit.json
report.json
```

Every audit reports zero for source payload bytes, preview bytes, audio
headers, PCM samples, waveform/feature values, force/mesh values, protected
signals and role values. HTML bytes are `publisher_metadata_bytes_read`.

## Required verification

Focused tests cover at least:

1. dynamic HTML fields normalize identically;
2. exact Steel succeeds while Stainless Steel and generic Metal do not;
3. filename/description material conflict excludes the row;
4. repeated recordings remain one physical group;
5. author/pack/sound mismatch, absent pack link and unknown license fail;
6. duplicate project/group/sound identity fails;
7. raw aggregate floors do not bypass whole-project partition failure;
8. baseline exposure match makes a project ineligible;
9. symlink/hash/schema/output/repository-path guards fail closed;
10. capture/audit access ledgers remain zero-signal.

Official capture is performed once. The frozen capture is audited into fresh A
and B directories and compared recursively. No source HTML, audio, dataset,
checkpoint or generated sound enters Git.

## Next boundary

If raw growth passes but balanced power fails, the next increment targets the
exact best-frontier deficits, preferably through one multi-object dual-class
project rather than more repeats of one vessel. Q2, P2 and all protected
payloads remain sealed until a fresh Q1 role freeze passes unchanged.

