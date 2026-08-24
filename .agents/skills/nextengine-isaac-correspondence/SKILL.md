---
name: nextengine-isaac-correspondence
description: "Audit NextEngine canonical CPU PhysX versus Isaac Lab GPU mirror identity, trajectories, rewards, contacts, terminations and applied targets. Use when preparing or reviewing MODEL-MIRROR-P1/P2, translating BodySchema to USD, checking Isaac/PhysX versions, comparing CPU and GPU NPZ trajectories, investigating mirror drift, or deciding whether mirror-trained artifacts are admissible. Russian triggers include: correspondence, CPU против Isaac, PhysX parity, MODEL-MIRROR, Isaac drift, USD descriptor, совпадение наград и контактов."
---

# NextEngine Isaac correspondence

Keep CPU PhysX/headless canonical. Isaac is a replaceable accelerated mirror;
passing correspondence permits bounded mirror use but never makes GPU replay
authority.

## Load the governing profile

1. Resolve the repository and read `AGENTS.md` plus the physics, motor and
   deterministic-training rows in `docs/architecture/agent-routing.md`.
2. Read SPEC-35 and the profile-specific ADRs in full. For the current
   biomechanics tracker, include ADR-069 and ADR-070.
3. Read [references/correspondence-contract.md](references/correspondence-contract.md)
   before choosing exact versus tolerant comparisons.
4. Read the tracked Isaac profile instead of relying on upstream examples.
   NextEngine currently pins Isaac Lab/Isaac Sim independently from engine
   PhysX; code or advice for a different major release is not presumed valid.

## Audit identity before simulation

Run:

```text
python .agents/skills/nextengine-isaac-correspondence/scripts/audit_correspondence.py \
  --repository . \
  --generation-manifest <external generation-manifest.json> \
  --environment-profile lab/profiles/humanoid-reference-tracker.v1.json \
  --descriptor <external descriptor.json> \
  --usd <external humanoid.usda> \
  --isaac-profile lab/profiles/isaac-lab-physx-stage0.v1.json \
  [--corpus-root <external corpus directory>] \
  [--gate-report <external TRAIN-4 gate report>]
```

Stop before Isaac startup on any schema, version, generation admission, body,
descriptor, USD, corpus, profile or gate hash mismatch.

## Generate correspondence evidence

For standing/flat-command/curriculum profiles, collect matching declared CPU
and GPU trajectory corpora and use the repository command:

```text
python -m next_lab correspondence \
  --cpu <external cpu.npz> \
  --gpu <external gpu.npz> \
  --store <external training store>
```

Use identical logical inputs, seed descriptors, episode/slot assignment and
sample floors. Never align or discard inconvenient frames after collection.

The biomechanics reference tracker additionally requires
`MODEL-MIRROR-P2`: reference selection/phase/features, reward components,
terminal facts and post-safety applied targets must correspond. A P1 report or
reference sanity smoke is useful evidence but cannot be relabelled P2.

Audit an emitted report with:

```text
python .agents/skills/nextengine-isaac-correspondence/scripts/audit_correspondence.py \
  <same identity arguments> \
  --report <external model-mirror report.json> \
  [--cpu <external cpu.npz> --gpu <external gpu.npz>] \
  --require-passed-report
```

## Interpret failures

- Exact ID/hash/order mismatch: fix descriptor, translator, schedule, feature
  order or stale artifact closure; do not add tolerance.
- Joint/root/velocity drift: inspect axes, frames, inertias, solver/drive
  settings, actuation timing and applied targets.
- Contact drift: inspect collider roles, filters, materials, sole definitions
  and contact sampling cadence.
- Reward/termination drift: recompute from the same committed facts and ordered
  Q16 component profile; trainer-side fixes are invalid.
- Non-finite or sample-floor failure: reject the corpus.

Do not invent a tolerance, relax a frozen threshold or change the canonical
physical model to make correspondence pass. Use
`$nextengine-mathematical-research` for a disputed numerical/model claim and
`$nextengine-architecture` for any resulting semantic change before collecting
new admission evidence.

For direct-torque profiles, imported joint drives must not fight the agent.
The current reference tracker is residual position target plus engine safety
and fixed PD; do not disable those drives by copying advice for another action
profile.

## Report boundaries

Report identity readiness, P1/P2 evidence status and every missing run
separately. `NotRun` is not `Pass`. Even a passing mirror report does not prove
statistical policy quality, portable runtime parity, TRAIN advancement or R5
completion.
