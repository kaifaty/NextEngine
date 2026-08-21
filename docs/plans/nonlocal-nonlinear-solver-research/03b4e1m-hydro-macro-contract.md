# NSR3-B4E1M -- nominal Hydro one-macro contract

Status: `FROZEN / IMPLEMENTATION_AND_EXECUTION_AUTHORIZED / STEP1_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e1m-hydro-macro|v1|parent=76453ea9d74f996c52a710a126c444b57662485aaad74c71e041e9e952fe17ac:f57c9ed22f8988a20f88b2f877b9bdf6b328f17b677cc68ceaf02251287e37c9|alignment=c53a112830cb94c4139da75c2044d28f61673cb1aa05c116098967aee41f7bbe:79a8932136a64c1cfa953caafe323cb9e860cb134c1d4dc0907be7f40891587a|candidate=66e318cb69e0b0c0a3a40a2beafa2099ebf151287581b191a242e82dac6d6f3c|publication=e713a61649fc230b189fca9eda3628b69f9369c706df35f0a2b080a4bd189a70|scenario=c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0|initial-frame=999cc0c925e52dc873be53f911d3effc0a2bf48fe8c5538e9fe3286b14fc76c7|static=daafa32e95eea258c51704d30d7654a702778d560d59fab749a96180b0a6b297|pairs=26ab8b79194d53686510d59a11e013900c79ccad57585549acc2007aa1e66414:354630:615072:118|spectrum=31179.700485618621:499.43728723929792:14:48|macro=step1;levels=14,28,56,112;selected=adjacent-pass;accepted<=112|transaction=retained-flat-csr;fine-only;one-frame;one-ledger|physics=strain<=1e-3;energy<=1%;penetration<=2.5mm;ledger<=1e-9|work=static-index1;no-all-pairs;live0|runs=2-process-byte-exact;timing=external;watchdog=900s|reference=closed|credit=b4e2-design-only
```

Identity SHA-256:
`0cdc26e1d0b406fecc39c64cebfee080be32804b993e6d433fa0a74658efb4cc`.

## Command and exact parent

Add only:

```text
nonlocal-formula-reclosure --nominal-hydro-macro-probe
```

Require B4E0/B4E1S identities/results and reconstruct exact Hydro scenario,
initial frame, support/static-index and flat pair roots before the transaction.
Create one immutable static index and reuse its checked binding throughout.
Do not call a parent command, decode an external reference or run any second
macro step.

## One-macro transaction

Call the existing complete `run_macro_adaptive_transaction_case` once with:

- 6,000 initial fluid positions and zero velocities;
- exact 5,824 support positions and canonical closed-box topology;
- scenario root `c430b679...909a0`, frame index zero and macro step one;
- query recording, accepted-workspace retention and flat adjacency enabled;
- capacities 11,824 participants, 960,000 pairs and 160 neighbors.

Require `START_ACTIVE`, exactly 48 spectral HVPs, initial count 14 and planned
levels drawn in order from `14,28,56,112`. Selection must be the first
adjacent passing pair at level 1--3; accepted substeps are at most 112.
Unsuccessful earlier levels may be skipped only through the existing exact
recoverable-reject classifier. Any other failure is fatal and retained as
negative evidence.

## Physics, publication and work gates

PASS requires the unchanged parent transaction gates plus:

- maximum positive density strain `<=1e-3`;
- energy creation `max(0,max_mechanical-initial_mechanical)` no greater than
  `0.01*max(abs(initial_mechanical),N*M*9.81*0.05,1e-12)`;
- maximum decoded penetration `<=0.0025 m` while retaining the stricter
  private KKT penetration gate;
- one committed step-1 frame, one ledger entry, fine-only commit and exact
  recomputed frame/trajectory/ledger roots;
- finite canonical COM, q99 x/y, momentum, kinetic, pressure, gravitational
  and mechanical aggregates;
- exact attempted/accepted/discarded accounting and balanced retained
  transfer/read/release ownership;
- zero nested adjacency rows, zero candidate/audit all-pairs work and zero
  live workspaces after the command.

The deterministic report includes solver/refinement counters, roots,
aggregates and work receipts, but no time, RSS or machine path. Do not freeze
an expected selected level, nonlinear HVP count or output root before running.

## Repeatability, watchdog and exit

Run one transaction in each of two fresh processes from two independent
Release builds and require byte-identical JSON. Wrap each process in an
external 900-second watchdog plus `/usr/bin/time -v`.

A watchdog exit is `EXECUTION_BUDGET_EXCEEDED`, grants no PASS and routes only
to B4EP performance research. A deterministic transaction FAIL is preserved
without tuning under this identity. PASS selects only
`NOMINAL_HYDRO_ONE_MACRO_CANDIDATE` and authorizes B4E2 first-output research/
contract design. Keep `reference_curve_decoded=false`,
`b4e_comparison_execution_authorized=false`, `runtime_authority=false` and
`production_authority=false`.
