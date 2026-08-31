# Physical sound V16 — decoupled learning and admission rebaseline

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `DECISION_COMPLETE / LEARNING_UNBLOCKED / ADMISSION_STILL_BLOCKED` |
| Replaces | V15 execution ordering; V15 data minima and product boundaries remain unchanged |
| Signal access | `0` new PCM samples, `0` force samples, `0` protected samples |
| Product effect | None; SPEC-45 remains `Proposed` and authored clips remain authoritative |

## Question

Must the project find a complete protected real dataset before it can test the
neural modal-field hypothesis, or can model feasibility, disclosed-data quality
and independent admission be investigated as separate claims?

## Observation

[V15 S0c](physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md)
proves that the currently inventoried sources cannot fill the unchanged
`4 train + 1 generator development + 1 validator calibration + 1 method
holdout + 1 admission shadow` Metal shape. It does not prove that a structured
neural modal field is incapable of recovering known modes or useful contact
variation.

V15 nevertheless blocks its known-truth neural oracle on the real-source role
freeze. Those dependencies answer different questions:

1. engine-owned truth can test whether the representation and optimizer recover
   a known physical field;
2. disclosed or historically exposed real groups can test whether the candidate
   beats classical controls on real signals, but cannot provide independent
   admission evidence;
3. fresh protected real groups are required to calibrate the automatic
   validator, test the method and open a shadow exactly once.

Waiting for the third kind of evidence before testing the first spends source
research effort without knowing whether the model family is worth admitting.
Opening protected evidence before a candidate survives the cheap tests creates
the opposite problem: scarce holdout value is consumed by an unproven method.

## Decision

V16 separates the program into three evidence lanes with one-way promotion:

| Certificate | Permitted evidence | What it may claim | What it cannot claim |
| --- | --- | --- | --- |
| `Capability` | engine-owned known truth and mutations | the implementation can recover and reject a bounded modal field | realism or usefulness on published objects |
| `CandidateQuality` | hash-frozen generator train/development data, including explicitly disclosed prior exposure | the candidate beats named controls on those disclosed real groups | independent generalization, validator accuracy or runtime readiness |
| `Admission` | fresh, revision-disjoint `evaluation_complete` T2/T3 groups | an independent frozen validator accepts, rejects or returns OOD at the declared research gate | production support for arbitrary objects or true material constants |

The `Capability` lane starts immediately. The source lane continues in parallel
and may classify a previously exposed group as generator-only only when a known
manifest schema proves its role; unknown historical schemas remain protected or
unknown. The `CandidateQuality` lane opens real signal only after generator
roles, metrics, controls, compute ceiling and stop rules are frozen.

The `Admission` lane retains all V15 minima. It cannot borrow an exposed object,
synthetic teacher, T4 semantic corpus or source-relative label to fill protected
credit. Validator structure and mutations may be implemented early, but its
thresholds and release stay draft until protected calibration and method
holdout exist.

## Consequences

- A missing protected source no longer prevents useful algorithm research.
- A passing synthetic oracle does not make the sound realistic; it only opens
  disclosed real development.
- A good disclosed-data demo is report-only until protected admission exists.
- A failed truth or real-data tournament stops the model family before scarce
  protected evidence is consumed.
- A source-growth failure may terminate as
  `CANDIDATE_RESEARCH_COMPLETE / ADMISSION_BLOCKED`; no cooked production asset
  or runtime path follows.
- Existing authored clips remain the complete fallback for every material.
- Neural inference remains offline. The engine receives only deterministic
  cooked clips after admission and a later product promotion decision.

## Reconsideration conditions

- Known-truth evidence shows that source-role uncertainty materially changes
  the representation experiment itself.
- A published source supplies the complete Metal role shape before the
  capability lane finishes; it may be frozen but its protected signals remain
  sealed until the candidate reaches admission.
- A future Accepted ADR authorizes a different runtime or evidence boundary.

