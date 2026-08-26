namespace {

constexpr const char* AL_R20_R54_SEMANTIC_SHA256 =
    "cce59025c9d4b606662d436f82463faef6682c8d07e7cb795c8113cdc00c389e";

struct ALR20R55SlopeTrace {
    std::size_t calls = 0U;
    std::size_t inverse_audits = 0U;
    std::size_t cap_hits = 0U;
    std::size_t inverse_columns = 0U;
    std::vector<std::string> matrix_roots;
    std::vector<std::string> rhs_roots;
    std::vector<std::string> solution_roots;
    std::vector<std::string> audit_roots;
    std::vector<std::string> result_roots;
};

thread_local ALR20R55SlopeTrace* al_r20_r55_slope_trace = nullptr;

ALR20SlopeErrorRefinement al_r20_r55_slope_refiner(
    const ALR20DirectSolutionCapture& capture) {
    ALR20SlopeErrorRefinement result;
    ALR20R55SlopeTrace* trace = al_r20_r55_slope_trace;
    if (trace == nullptr) return result;
    ++trace->calls;
    trace->matrix_roots.push_back(capture.factor.matrix_root);
    trace->rhs_roots.push_back(al_r20_q_scalar_root(capture.rhs));
    trace->solution_roots.push_back(al_r20_q_scalar_root(capture.solution));
    if (trace->inverse_audits >= 1U) {
        ++trace->cap_hits;
        result.exact = true;
        result.root = sha256_hex("r55-slope-refinement-cap:"
            + std::to_string(trace->calls));
        trace->result_roots.push_back(result.root);
        return result;
    }
    ALR20ADMMWork work;
    const ALR20VerifiedInverse audit = al_r20_verified_inverse(
        capture.factor, capture.solve_residual, capture.solve_bound,
        capture.solution, capture.cheap_error, &work, &capture.rhs);
    ++trace->inverse_audits;
    trace->inverse_columns += audit.column_solves;
    trace->audit_roots.push_back(audit.root);
    result.inverse_columns = audit.column_solves;
    result.error = audit.refined_error;
    result.audit_root = audit.root;
    result.exact = capture.factor.passed
        && capture.factor.dimension > 0U
        && capture.rhs.size() == capture.factor.dimension
        && capture.solution.size() == capture.factor.dimension
        && audit.dimension == capture.factor.dimension
        && audit.column_solves == capture.factor.dimension
        && work.factor_terms == 0U && work.matrix_outer_terms == 0U
        && finiteq(audit.refined_error) != 0;
    result.passed = result.exact && audit.passed
        && !audit.refinement_attempted;
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':'
        << result.inverse_columns << ':' << al_binary128_hex(result.error)
        << ':' << capture.factor.matrix_root << ':'
        << al_r20_q_scalar_root(capture.rhs) << ':'
        << al_r20_q_scalar_root(capture.solution) << ':' << audit.root;
    result.root = sha256_hex(material.str());
    trace->result_roots.push_back(result.root);
    return result;
}

std::string al_r20_r55_trace_root(const ALR20R55SlopeTrace& trace) {
    std::ostringstream material;
    material << trace.calls << ':' << trace.inverse_audits << ':'
        << trace.cap_hits << ':' << trace.inverse_columns << '|';
    for (std::size_t index = 0U; index < trace.result_roots.size(); ++index)
        material << trace.matrix_roots[index] << ':' << trace.rhs_roots[index]
            << ':' << trace.solution_roots[index] << ':'
            << trace.result_roots[index] << ';';
    material << '|';
    for (const std::string& root : trace.audit_roots) material << root << ';';
    return sha256_hex(material.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_counterflow_slope_trajectory_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool controls_exact = profile_exact
        && al_r20_verified_inverse_controls()
        && al_r20_semismooth_controls()
        && al_r20_r21_event_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    bool parent_exact = controls_exact && cases.size() == 5U
        && al_r20_v4_manifest_root(cases) == AL_R20_V4_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALR20ProblemData problem;
    bool problem_materialized = false;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[2U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[2U];
    }

    ALR20R55SlopeTrace trace;
    ALR20R35CaseAudit counterflow;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r55_slope_trace = &trace;
        al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
        counterflow = al_r20_r35_case(problem);
        al_r20_slope_error_refiner = nullptr;
        al_r20_r55_slope_trace = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    const bool reproduction_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(counterflow)
        && counterflow.diagnostic.steps.size() >= 10U;
    std::size_t attempted_steps = 0U;
    std::size_t consumed_steps = 0U;
    std::size_t first_attempt = counterflow.diagnostic.steps.size();
    for (std::size_t index = 0U;
         index < counterflow.diagnostic.steps.size(); ++index) {
        const ALR20SemismoothStep& step = counterflow.diagnostic.steps[index];
        if (step.slope_refinement_attempted) {
            if (first_attempt == counterflow.diagnostic.steps.size())
                first_attempt = index;
            ++attempted_steps;
        }
        if (step.slope_refinement_consumed) ++consumed_steps;
    }
    const ALR20SemismoothStep* refined_step = first_attempt
            < counterflow.diagnostic.steps.size()
        ? &counterflow.diagnostic.steps[first_attempt] : nullptr;
    std::string refined_direction_root;
    if (refined_step != nullptr && first_attempt > 0U) {
        const std::vector<Binary128>& prefix_lambda =
            counterflow.diagnostic.steps[first_attempt - 1U].next_lambda;
        if (prefix_lambda.size() == refined_step->line_representative.size()) {
            std::vector<Binary128> direction(prefix_lambda.size());
            for (std::size_t row = 0U; row < direction.size(); ++row)
                direction[row] = refined_step->line_representative[row]
                    - prefix_lambda[row];
            refined_direction_root = al_r20_q_scalar_root(direction);
        }
    }
    const bool prefix_exact = reproduction_exact && first_attempt == 9U
        && refined_step != nullptr && refined_step->iteration == 10U
        && refined_step->slope
            == strtoflt128(
                "1.849316406201200507755879126297304928e-19", nullptr)
        && refined_step->initial_slope_bound
            == strtoflt128(
                "2.012390908441786489857500257807782162e-14", nullptr)
        && refined_step->initial_direction_error_bound
            == strtoflt128(
                "8.852949539615971042121435165643669945e-03", nullptr)
        && refined_direction_root == AL_R20_R36_DIRECTION_SHA256;
    const bool trace_exact = prefix_exact && trace.calls == attempted_steps
        && trace.inverse_audits == 1U && trace.inverse_columns == 66U
        && consumed_steps == 1U && !trace.audit_roots.empty()
        && trace.audit_roots.front() == AL_R20_R53_LEGACY_AUDIT_SHA256
        && refined_step->slope_refinement_consumed
        && refined_step->slope_refinement.passed
        && refined_step->slope_refinement.error == strtoflt128(
            "1.093151728063539466044957165073324068e-24", nullptr)
        && refined_step->direction_error_bound
            == refined_step->slope_refinement.error
        && refined_step->direction_certified;
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool apparatus_exact = trace_exact && lifecycle_exact;
    const bool exact = reproduction_exact && apparatus_exact;
    const std::string route = !parent_exact || !reproduction_exact
        ? "COUNTERFLOW_SLOPE_TRAJECTORY_PARENT_REJECTED"
        : (!prefix_exact
            ? "COUNTERFLOW_SLOPE_TRAJECTORY_APPARATUS_REJECTED"
            : (trace.calls == 0U
                ? "COUNTERFLOW_SLOPE_REFINER_UNUSED"
                : (!trace_exact
                    ? "COUNTERFLOW_SLOPE_REFINER_REJECTED"
                    : (!counterflow.diagnostic.certified
                        ? "COUNTERFLOW_SLOPE_LATER_BOUNDARY"
                        : "COUNTERFLOW_SLOPE_TRAJECTORY_CANDIDATE"))));
    const std::string trace_root = al_r20_r55_trace_root(trace);
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r55-slope-trajectory.v1|"
        << AL_R20_R54_SEMANTIC_SHA256 << ':' << route << ':'
        << trace_root << ':' << counterflow.root;
    const std::string result_sha = sha256_hex(semantic.str());

    const ALR20SemismoothStep* final_step =
        counterflow.diagnostic.steps.empty()
        ? nullptr : &counterflow.diagnostic.steps.back();
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r55_slope_trajectory.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r54_sha256\":\"" << AL_R20_R54_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":" << (reproduction_exact ? "true" : "false")
        << ",\"prefix_exact\":" << (prefix_exact ? "true" : "false")
        << ",\"trace_exact\":" << (trace_exact ? "true" : "false")
        << ",\"apparatus_exact\":" << (apparatus_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"refinement\":{\"calls\":" << trace.calls
        << ",\"attempted_steps\":" << attempted_steps
        << ",\"consumed_steps\":" << consumed_steps
        << ",\"inverse_audits\":" << trace.inverse_audits
        << ",\"cap_hits\":" << trace.cap_hits
        << ",\"inverse_columns\":" << trace.inverse_columns
        << ",\"first_attempt_index\":" << first_attempt
        << ",\"trace_root\":\"" << trace_root << "\"}"
        << ",\"boundary_step\":{\"iteration\":"
        << (refined_step == nullptr ? 0U : refined_step->iteration)
        << ",\"initial_error\":\""
        << al_binary128_text(refined_step == nullptr
            ? static_cast<Binary128>(0.0)
            : refined_step->initial_direction_error_bound)
        << "\",\"refined_error\":\""
        << al_binary128_text(refined_step == nullptr
            ? static_cast<Binary128>(0.0)
            : refined_step->direction_error_bound)
        << "\",\"initial_slope_bound\":\""
        << al_binary128_text(refined_step == nullptr
            ? static_cast<Binary128>(0.0) : refined_step->initial_slope_bound)
        << "\",\"refined_slope_bound\":\""
        << al_binary128_text(refined_step == nullptr
            ? static_cast<Binary128>(0.0) : refined_step->slope_bound)
        << "\",\"direction_root\":\"" << refined_direction_root << "\"}"
        << ",\"trajectory\":{\"case_root\":\"" << counterflow.root
        << "\",\"route\":\"" << counterflow.route
        << "\",\"certified\":"
        << (counterflow.diagnostic.certified ? "true" : "false")
        << ",\"accepted_iterations\":"
        << counterflow.diagnostic.accepted_iterations
        << ",\"principal_solves\":"
        << counterflow.diagnostic.total_principal_solves
        << ",\"transitions\":" << counterflow.diagnostic.total_transitions
        << ",\"steps\":" << counterflow.diagnostic.steps.size()
        << ",\"final_solve_failure\":\""
        << (final_step == nullptr ? "ABSENT" : final_step->solve.failure)
        << "\",\"final_step_root\":\""
        << (final_step == nullptr ? "" : final_step->root) << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_inverse_columns\":" << trace.inverse_columns
        << ",\"additional_factorizations\":0,\"compensated_dots\":0"
        << ",\"solution_corrections\":0,\"new_trial_formulae\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
