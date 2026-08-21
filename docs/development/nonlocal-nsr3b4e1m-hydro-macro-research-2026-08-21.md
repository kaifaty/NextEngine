# NSR3-B4E1M Hydro one-macro research -- 2026-08-21

Status: `COMPLETE / ONE_MACRO_TRANSACTION_SELECTED / NO_REFERENCE_COMPARISON`

## Question

What is the smallest execution after B4E1S that can expose nominal nonlinear
solver cost and correctness without mixing it with a reference curve or a
multi-step trajectory?

## Existing transaction path

The complete B4C4C1 lane already owns the required operations in
`run_macro_adaptive_transaction_case`:

1. build the exact frame-start joint fluid/static-support workspace;
2. run the deterministic 48-HVP spectrum when pressure is active;
3. attempt at most four temporal levels from the same committed state;
4. admit only two adjacent passing levels through the existing smoke gate;
5. publish only the selected fine state through balanced canonical
   micrometre publication and its momentum/energy ledger;
6. release every transient/retained workspace and leave zero live ownership.

The nominal Hydro fixture can reuse that path without a new solver: 6,000
fluid centres, 5,824 exact two-layer box support centres, a closed hard-contact
box from `0.025` to `0.975 m`, gravity `-9.81 m/s^2`, the immutable B4E0 static
index and the B4C4C1 flat-CSR retained-workspace candidate.

## Temporal and transaction boundary

B4E1S fixes the initial count at 14. The existing four-level transaction can
therefore attempt only `14, 28, 56, 112` substeps. It stops at the first
adjacent passing pair. Level zero can never be committed by itself; selected
level must be 1--3 and accepted work can be at most 112 substeps.

Earlier attempts are private. Success must expose their work as discarded,
publish exactly one step-1 frame and one ledger entry, and prove that only the
fine member of the selected pair becomes committed state. This is the useful
nominal rollback property without paying for a second forced-failure macro.

No selected level, nonlinear HVP count, outer-trial count, frame root or
runtime is frozen before execution. Freezing any of them would turn the probe
into postulated evidence rather than a discriminator.

## Physical and work gates

The parent transaction's stricter KKT/contact/publication gates remain in
force: finite solve, maximum penetration at most `1e-12 m`, momentum-ledger
residual at most `1e-9`, support-reaction closure at most `1e-10`, canonical
topology and balanced publication correspondence.

B4E1M additionally retains the already selected nominal observables:

- maximum positive density strain at most `1e-3`;
- mechanical-energy creation at most 1% of the initial mechanical scale;
- canonical sample count/COM/q99/momentum aggregates finite and reproducible;
- zero candidate/audit all-pairs evaluations or HVPs;
- one immutable static-index build, flat adjacency only, balanced retained
  workspace transfers/reads/releases and zero final live ownership.

The command runs one transaction, not two. Repeatability comes from two fresh
processes and two independent Release builds whose deterministic reports must
be byte-identical. Clock, RSS and host paths stay outside the report.

## Cost stop

Each fresh process has an external 900-second watchdog. This is not a physics
tolerance and cannot produce PASS. If it fires, preserve the stopped run as
`EXECUTION_BUDGET_EXCEEDED`, authorize only B4EP performance research and do
not tune solver parameters under the same identity. A single first macro
taking 15 minutes already implies an obviously inadmissible full-corpus cost.

## Decision

Freeze a Hydro-only B4E1M command over the existing complete flat transaction.
A deterministic physical/work PASS may authorize only B4E2 first-output
contract research. External reference files remain unopened, step 2 and later
remain forbidden, and no runtime, GPU or production authority is created.
