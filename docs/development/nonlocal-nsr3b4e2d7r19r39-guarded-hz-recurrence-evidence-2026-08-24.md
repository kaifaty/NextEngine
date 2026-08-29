# NSR3-B4E2D7R19R39 guarded Hager--Zhang recurrence evidence -- 2026-08-24

Status: `PASS / GUARDED_HZ_RECURRENCE_CANDIDATE`.

## Outcome

Starting from exact R37 state 24, the guarded Hager--Zhang lane accepts all
eight frozen memory directions with zero restarts. It strictly dominates the
equal-work steepest lane in objective, violation norm and active projected
mapping at every checkpoint.

| Step | Lane | Objective | Violation | Mapping | Active |
|---:|---|---:|---:|---:|---:|
| `1` | steepest | `4.2118709613243402e-18` | `2.9023683299417185e-9` | `2.4979514364364407e-10` | `840` |
| `1` | Hager--Zhang | `1.3684753550918560e-20` | `1.6543732076480543e-10` | `5.8105799072611780e-11` | `260` |
| `2` | steepest | `3.8778931568868514e-18` | `2.7849212401383460e-9` | `2.4316347892582719e-10` | `846` |
| `2` | Hager--Zhang | `1.8331460251977860e-21` | `6.0549913710884610e-11` | `1.4557215009478321e-11` | `302` |
| `4` | steepest | `3.2853625433399335e-18` | `2.5633425613210316e-9` | `2.2380272381561380e-10` | `840` |
| `4` | Hager--Zhang | `7.8143952143484640e-23` | `1.2501516079538885e-11` | `2.9791300008883827e-12` | `244` |
| `8` | steepest | `2.3588830687047794e-18` | `2.1720419280965916e-9` | `1.8959367948391477e-10` | `838` |
| `8` | Hager--Zhang | `9.1332887060949100e-27` | `1.3515390268945186e-13` | `2.9633395183450478e-14` | `52` |

At step eight the Hager--Zhang/steepest ratios are objective
`3.871870050392043e-9`, violation `6.222435255100725e-5` and mapping
`1.562994888021285e-4`: approximately `2.58e8x`, `1.61e4x` and `6.40e3x`
improvements respectively. Relative to the R37 source, terminal objective,
violation and mapping are `1.99487e-9`, `4.46640e-5` and `1.12151e-4`.

The active count changes `846 -> 260 -> 302 -> 244 -> 52`, yet every HZ
direction remains raw/projected descent-safe. The observed benefit therefore
survives this fixture's changing squared-hinge active set. This is empirical
for the frozen linearized fixture, not a general convergence guarantee.

The terminal mapping is nonzero at `2.9633395183450478e-14`. R39 neither
declares stationarity nor fits a tolerance to that value.

## Controls and work

- R39 identity:
  `acff94da15e985dad6695c64883893cdf3de6d571e9332e48f90b37b812b5fc2`;
- exact R38 parent stdout:
  `7f574289496e70fb76886381c2b1460227f15a3f6d07214b0772dabf7247abb1`;
- exact source checkpoint:
  `463fc296b6adf7eb690c7ad4bbf05e07238424abe83ab0243cb6f7c28ac84be0`;
- exact first HZ record:
  `185635ed433b785f5df56533c467a0ff2a13ab61c89ce9e8cddca9632777e18b`;
- dense root:
  `4a9b0a0382bd883abfce8bcb55ea8875ab08ba9aae0a3ff5fb1109789526f543`;
- checkpoint root:
  `01b2c06848dfb9886b32302e9abf35d6baf294e09ce020880d496333285f669e`;
- HZ memory steps/streak/restarts: `8/8/0`;
- each lane spends exactly `18` pair passes (`10` JVP and `8` VJP); total
  new work is `36` pair passes, zero HVP/model/trial/outer work;
- one workspace lifecycle, all 13 routes and exact rollback pass.

The first diagnostic passed without contract or implementation repair.

## Clean reproducibility

Contract/research commit: `e30f44ea`. Implementation commit: `415fbaff`.

Clean Release build directories:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r39-a.1Ossbu`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r39-b.UMn84P`.

Both binaries have SHA-256
`eaf755396f510b7b43265806d78469393c1b675cc8fd6888b81060dd0578cbdc`,
size `7,491,608` bytes and ELF build-id
`ae04e062eb7588290dd62a9565b6d8d28b71d282`.

Fresh run directories:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r39-a.6FHPYg`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r39-b.oDr5oy`.

The two `4,517`-byte stdout files are byte-exact. Their SHA-256 is
`06812cb75177cd4666b103101f6bc9238fa20760179bc71563f194fe476db252`;
semantic result SHA-256 is
`ce86c4266d224165fec4a3de32786e986f29bae95afe96dc41937f895157203b`.
Both stderr files are empty. No timing was taken or interpreted.

## Decision

Retain the exact R36 curvature prefix plus guarded Hager--Zhang polishing as
the strongest current linearized normal-step candidate. More unchanged inner
continuation would primarily probe binary64 termination rather than the next
architectural risk.

Research and freeze a rollback-only nonlinear moved-state acceptance
discriminator next. It must apply the candidate through the existing
dimensional transaction mapping, rebuild the nonlinear objective/operator,
compare predicted and actual constraint/merit reduction, enforce topology and
trust ownership, and reject without mutation on any failed gate. Coefficients,
penalty policy and tolerance remain unchanged.

R39 applies no correction, evaluates no nonlinear moved state, selects no
tolerance and provides no runtime or production authority.
