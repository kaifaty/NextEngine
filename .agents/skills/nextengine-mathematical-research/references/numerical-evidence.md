# Numerical evidence playbook

Use this reference for numerical solvers, physical models, geometry, control,
stochastic methods, fixed-point arithmetic and floating-point claims.

## Establish the model before measuring

Record:

- variables, dimensions, units and nondimensional groups;
- coordinate frames, signs, norms and reference points;
- domain, initial/boundary/contact conditions and degenerate cases;
- discretization, topology and update/reduction order;
- finite-precision, rounding, overflow, FMA, subnormal and convergence-branch
  assumptions;
- exact parameter/profile/input identity and which values may vary.

Check zero/small/large limits, symmetries, invariants, positivity, monotonicity,
conservation and well-posedness before tuning a solver.

## Build an independent oracle ladder

Use the lowest sufficient rung, adding independence as the claim grows:

1. hand-derived scalar or one-element case;
2. dimensional, sign and limiting-case checks;
3. exact/arbitrary-precision computation for small cases;
4. manufactured solution or analytically known special case;
5. separately written reference implementation using another formulation;
6. interval/enclosure method or exhaustive finite certificate;
7. refinement, sensitivity and production-profile comparison.

An oracle that calls the candidate kernel, shares its indexing/reduction code
or imports its intermediate results is a consistency check, not an independent
oracle. Record shared lineage explicitly.

## Predeclare controls and discriminators

Every experiment should name:

- a successful control that proves the apparatus can observe success;
- a negative or deliberately broken control that proves it can observe
  failure;
- the first metric or witness that distinguishes each serious hypothesis;
- precision, tolerance and why the tolerance is above numerical ambiguity yet
  below the product distinction;
- parameter domain, seeds, sample count, refinement path and stopping rule;
- failure retention and the rule forbidding selective retry or point removal.

Prefer one small counterfactual with controls over another full expensive run.

When searching for a smallest counterexample, predeclare a well-founded
simplicity order over structure, active terms and discrete inputs. A continuous
parameter domain may have only an infimum or failure boundary, not a minimum;
report a verified stable/unstable bracket plus the first failing representable
point on the frozen grid instead of inventing a smallest real value.

## Match evidence to the claim

| Claim | Required questions and discriminators |
| --- | --- |
| Consistency / convergence | What continuous/discrete problem is approximated? What norm and order? Does refinement enter the asymptotic regime? Are space, time and iteration errors separated? |
| Stability | Which stability notion, norm and domain? Separate spatial/model stability, time-integrator stability, nonlinear convergence and executed finite-precision behavior. For linear updates check power-boundedness or an energy argument rather than spectral radius alone. Is the observed plateau truncation, conditioning or instability? Search the smallest divergent/adversarial case. |
| Conservation / balance | Define the complete control volume and every source, sink, boundary flux, topology change and quantization residual. Test local and global balance separately. |
| Linear/nonlinear solve | Record conditioning, scaling, rank, residual and backward error; distinguish feasibility from numerical validity; compare KKT/constraint residuals, not only an objective. |
| Fixed-point / integer | Prove range and overflow bounds, rounding direction, saturation behavior and accumulated residual; test extremal values and split/merge/topology paths. |
| IEEE floating point | Separate real-arithmetic theorem from executed result; pin reduction/factorization order and math primitives; inspect cancellation, FMA and branch thresholds; use error bounds or interval checks where material. |
| Geometry / collision | Include exact predicates or robust error bounds, degeneracies, coplanar/tangent/equality cases, topology consistency and scale extremes. |
| ODE/PDE / physical solver | Check dimensional consistency, boundary compatibility, stiffness/CFL or energy argument, positivity, mass/momentum/energy behavior, manufactured cases and grid/time-step refinement. |
| Control / optimization | State objective, constraints, feasibility, observability/controllability when applicable, local versus global guarantee, constraint residuals and adversarial initial states. |
| Stochastic / RL-related | Freeze estimator, seeds and selection rule; report uncertainty and multiple comparisons; distinguish mathematical reward/property claims from learned-policy quality and environment validity. |
| Performance algorithm | First preserve exact/tolerant correctness roots; measure the limiting resource and target host/profile; keep asymptotic complexity separate from constant-factor benchmark evidence. |

## Diagnose failures causally

Localize the first failing boundary and cluster failures by mode, scale,
topology, phase or active set. Compare competing explanations, including
non-local ones:

- invalid model or incompatible constraints;
- incorrect discretization or boundary condition;
- rank/conditioning/scaling defect;
- finite-precision or tolerance ambiguity;
- candidate implementation defect;
- reference-oracle defect;
- real physical infeasibility;
- insufficient capacity, budget or measurement power.

Do not tune a tolerance, friction coefficient, iteration ceiling, seed or
reward until the experiment distinguishes these explanations.

## Preserve exact evidence identity

Record the command, commit, tool/library versions, host/profile when relevant,
input and output hashes, precision, seeds, sample counts and first decisive
diagnostic. Keep heavy artifacts outside Git and link by stable external path
or identifier. Report unavailable evidence as `NOT_TESTED`, never as implied
support.
