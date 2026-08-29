# NSR3-B4E2D7R19R28 shadow outer-11 execution research

Date: `2026-08-24`

Status: `COMPLETE / PASS / SHADOW_OUTER11_EXECUTION_CANDIDATE`

## Question

Does exact continuation remain budget-offset invariant at outer 11, and what
does one more unchanged-policy observation say about the sharp slowdown of
primal progress at outer 10?

R28 cannot by itself prove a discretization floor. Primal-progress and
stationarity trends are report-only observations. All existing validity and
admissibility predicates, penalty parameters and the 34-HVP trust cap remain
unchanged.

## Bound source and lanes

R28 binds exact R26 physical state plus the complete R27 grant chain:

```text
R26 state / receipt   317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd
                     3b5c6dd3b8824eddc3aad566a8b663d344e1702f7958dced701c61710d688776
R27 state / grant     369cab15f75ee90237aa5cbe334aa0da5f857e218bec9812773e43faa9682226
                     3ded4fc34e99c45657fad87a0aeb4853e1d948817efad60b4106f659c4adefaa
grant receipt/owner   53a938176504b9059504c511402b5b6f3e8b4dfb17bc6a2aa0f42204160cc330
                     431466b59941f862c97c75c5614be7d2dd525199bfbe9ea5e175753bc4cd431e
position / dual       174cad44a3edbc7eab514986e6be3d1bdd3c219aa68fc3c9913b164ce4058562
                     63770158bc269b879c4e3b98ac12a176be334902f0c929d05d686a157ce40f87
history               7aaf5e5077f937f03db754e407243e8c6b61b0f813931cc59d83f7f7dd866785
used                  11,2,26,1,282,805,54,32,775,30,2,32,0
```

Execute the same outer-update function from independent clones:

| Lane | Starting total | Maximum | New HVP available |
|---|---:|---:|---:|
| candidate | slice `282` | `512` | `230` |
| oracle | cumulative `805` | `8704` | `7899` |

Both execute outer index `11` exactly once. Candidate cannot borrow oracle
capacity. Per-update bounds are 16 trials, 230 new candidate HVP, 234 new
workspaces and 32 new precision audits; the per-trust cap remains 34.

## Required proof

Require complete bit/work equivalence of update/trials, position/dual/support,
topology, update/work roots, resource deltas, failure and admissibility. Report
new primal and stationarity bits and the descriptive progress ratio against
R26. No threshold may be selected after seeing the result.

After equivalence, consume the R27 active owner at independently pre-derived
root `13feea1f5a87fb577ed5f9ccf89e0f46d6acdb03554ea1c48b5e2c81081c5264`
and canonical-roundtrip one `NEALOER1` receipt and one `NEALOES1` state.
Every failure preserves exact transaction and physical input bytes; duplicate
replay is prework and idempotent.

## Interpretation

- exact and admissible: research a separate convergence/publication boundary;
- exact and non-admissible with capacity: preserve the observation and design
  a causal AL-stagnation-versus-discretization-floor discriminator;
- cap or candidate slice exhaustion: hard failure, never adopt oracle state;
- mismatch or solver failure: preserve R27 and diagnose the first exact
  boundary without coefficient, penalty, cap or policy tuning.

The later discriminator should separate outer-mechanism limits from spatial
representation limits. Candidate signals include multiplier/update response,
constraint-residual spatial structure, active boundary/support localization
and controlled refinement only after their formulas and budgets are frozen.
R28 selects none of those tests yet.

## Scope

One private outer-11 candidate and one comparison oracle only. No following
outer, substep, macro, trajectory, timing, public/world commit, durable or
concurrent CAS, runtime integration or production authority.

The frozen implementation passes with exact candidate/oracle roots. Outer 11
restores `4.4290%` primal improvement while remaining non-admissible; outer
10's weak progress was a local observation, not a proven plateau. See the
[dated evidence](nonlocal-nsr3b4e2d7r19r28-shadow-outer11-execution-evidence-2026-08-24.md).
