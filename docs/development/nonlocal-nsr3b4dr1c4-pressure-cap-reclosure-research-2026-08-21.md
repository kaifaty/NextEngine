# NSR3-B4DR1C4 pressure-cap reclosure research -- 2026-08-21

Status: `COMPLETE / CAP_300_RECLOSURE_SELECTED / NO_TRAJECTORY_RERUN_YET`

## Question

What is the narrowest reference-profile correction supported by the measured
step-1 convergence curve?

## Selection

Raise only DFSPH pressure `MAX_ITERATIONS` from 100 to 300. Keep the minimum
at 2 and the threshold at `0.01%`. The measured first-step solver exits at 220,
so the new maximum provides bounded headroom without forcing extra work after
convergence.

The profile retains volume `0.000125 m^3` and mass `0.125 kg`. Replacing them
with upstream's 0.8 startup heuristic would create an approximately 20%
underdense interior and cease to be like-for-like with the Nonlocal corpus.
Warm starts also remain disabled: they cannot fix step 1 and would change the
later trajectory state policy.

Because the solver profile changes, R1C4 receives a new contract/report and
payload-manifest identity. Scenario/fluid/boundary roots, contact, timestep,
serialization and every other profile field remain unchanged. The old R1C
failure stays valid.

## Execution recommendation

Implement a distinct `--r1c4-trajectory` mode. Run two fresh Hydro processes
and compare complete payload/report bytes. Only then advance to the Dam pair,
then Orifice pair, stopping on the first failure or mismatch. This is still a
short external reference gate, not R1D generation or a production claim.
