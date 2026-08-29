namespace {

constexpr const char* AL_R20_R41_SEMANTIC_SHA256 =
    "0ff6f9cc597166ca962e3fa3c97b29ed6350f62b22c6bb2768fa2a95daedf3cb";
constexpr const char* AL_R20_R41_DEPTH4_SHA256 =
    "3352aeb198fa9c93f57ec47076207599a8199a616553e7ebeafd3dd49e1b843f";

struct ALR20R42PracticalCertificate {
    bool exact = false;
    bool passed = false;
    bool no_underflow = false;
    bool contractive = false;
    std::size_t dimension = 0U;
    std::size_t left_dots = 0U;
    std::size_t residual_dots = 0U;
    std::size_t z_dots = 0U;
    std::size_t center_dots = 0U;
    std::size_t certificate_dots = 0U;
    std::size_t sum_dots = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    std::size_t worst_row = 0U;
    Binary128 rho_bound = static_cast<Binary128>(0.0);
    Binary128 g_inf_bound = static_cast<Binary128>(0.0);
    Binary128 radius = static_cast<Binary128>(0.0);
    Binary128 error = static_cast<Binary128>(0.0);
    Binary128 minimum_separation = HUGE_VALQ;
    Binary128 maximum_left_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_residual_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_z_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_g_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_center_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128> solution;
    std::string left_root;
    std::string residual_root;
    std::string z_root;
    std::string generation_root;
    std::string certificate_root;
    std::string center_root;
    std::string root;
};

ALR20R42PracticalCertificate al_r20_r42_practical_certificate(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution,
    std::size_t center_depth = 4U,
    bool require_first_target_values = true) {
    ALR20R42PracticalCertificate result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if ((center_depth != 4U && center_depth != 8U
            && center_depth != 16U)
        || dimension == 0U || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension
        || solution.size() != dimension)
        return result;
    bool exact = true;
    bool no_underflow = true;
    std::vector<Binary128> c_value(dimension * dimension);
    std::vector<Binary128> c_bound(dimension * dimension);
    std::ostringstream left_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        Binary128 row_bound = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            std::vector<Binary128> left(dimension + 1U);
            std::vector<Binary128> right(dimension + 1U);
            left[0U] = row == column
                ? static_cast<Binary128>(1.0)
                : static_cast<Binary128>(0.0);
            right[0U] = static_cast<Binary128>(1.0);
            for (std::size_t inner = 0U; inner < dimension; ++inner) {
                left[inner + 1U] = inverse[row * dimension + inner];
                right[inner + 1U] = -matrix[inner * dimension + column];
            }
            const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
            const std::size_t index = row * dimension + column;
            c_value[index] = dot.value;
            c_bound[index] = dot.bound;
            ++result.left_dots;
            exact = exact && dot.exact && finiteq(dot.value) != 0
                && finiteq(dot.bound) != 0;
            no_underflow = no_underflow && dot.no_underflow;
            result.maximum_left_bound = fmaxq(
                result.maximum_left_bound, dot.bound);
            row_bound = al_r20_r38_up_add(row_bound,
                al_r20_r38_up_add(fabsq(dot.value), dot.bound));
            left_material << dot.root << ';';
        }
        left_material << '|';
        if (row == 0U || row_bound > result.rho_bound)
            result.rho_bound = row_bound;
    }
    result.left_root = sha256_hex(left_material.str());
    result.contractive = result.rho_bound
        < static_cast<Binary128>(1.0);

    std::vector<Binary128> residual_value(dimension);
    std::vector<Binary128> residual_bound(dimension);
    std::ostringstream residual_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension + 1U);
        std::vector<Binary128> right(dimension + 1U);
        left[0U] = rhs[row];
        right[0U] = static_cast<Binary128>(1.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            left[column + 1U] = matrix[row * dimension + column];
            right[column + 1U] = -solution[column];
        }
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
        residual_value[row] = dot.value;
        residual_bound[row] = dot.bound;
        ++result.residual_dots;
        exact = exact && dot.exact && finiteq(dot.value) != 0
            && finiteq(dot.bound) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        result.maximum_residual_bound = fmaxq(
            result.maximum_residual_bound, dot.bound);
        residual_material << dot.root << ';';
    }
    result.residual_root = sha256_hex(residual_material.str());

    std::vector<Binary128> z_value(dimension);
    std::vector<Binary128> z_bound(dimension);
    std::ostringstream z_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension);
        for (std::size_t column = 0U; column < dimension; ++column)
            left[column] = inverse[row * dimension + column];
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, residual_value);
        Binary128 input_bound = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < dimension; ++column)
            input_bound = al_r20_r38_up_add(input_bound,
                al_r20_r38_up_multiply(
                    fabsq(left[column]), residual_bound[column]));
        z_value[row] = dot.value;
        z_bound[row] = al_r20_r38_up_add(dot.bound, input_bound);
        ++result.z_dots;
        exact = exact && dot.exact && finiteq(dot.value) != 0
            && finiteq(dot.bound) != 0 && finiteq(z_bound[row]) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        result.maximum_z_bound = fmaxq(
            result.maximum_z_bound, z_bound[row]);
        z_material << dot.root << ':' << al_binary128_hex(input_bound) << ';';
    }
    result.z_root = sha256_hex(z_material.str());

    std::vector<Binary128> correction(dimension,
        static_cast<Binary128>(0.0));
    std::ostringstream generation_material;
    for (std::size_t depth = 1U; depth <= center_depth; ++depth) {
        std::vector<Binary128> next(dimension);
        for (std::size_t row = 0U; row < dimension; ++row) {
            std::vector<Binary128> left(dimension + 1U);
            std::vector<Binary128> right(dimension + 1U);
            left[0U] = z_value[row];
            right[0U] = static_cast<Binary128>(1.0);
            for (std::size_t column = 0U; column < dimension; ++column) {
                left[column + 1U] = c_value[row * dimension + column];
                right[column + 1U] = correction[column];
            }
            const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
            next[row] = dot.value;
            ++result.center_dots;
            exact = exact && dot.exact && finiteq(dot.value) != 0
                && finiteq(dot.bound) != 0;
            no_underflow = no_underflow && dot.no_underflow;
            generation_material << depth << ':' << row << ':'
                << dot.root << ';';
        }
        correction = std::move(next);
    }
    result.generation_root = sha256_hex(generation_material.str());

    std::vector<Binary128> center_bound(dimension);
    result.solution.resize(dimension);
    std::ostringstream certificate_material;
    std::ostringstream center_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension + 2U);
        std::vector<Binary128> right(dimension + 2U,
            static_cast<Binary128>(1.0));
        left[0U] = z_value[row];
        for (std::size_t column = 0U; column < dimension; ++column) {
            left[column + 1U] = c_value[row * dimension + column];
            right[column + 1U] = correction[column];
        }
        left[dimension + 1U] = -correction[row];
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
        Binary128 input_bound = z_bound[row];
        for (std::size_t column = 0U; column < dimension; ++column)
            input_bound = al_r20_r38_up_add(input_bound,
                al_r20_r38_up_multiply(
                    c_bound[row * dimension + column],
                    fabsq(correction[column])));
        const Binary128 g_bound = al_r20_r38_up_add(
            dot.bound, input_bound);
        const Binary128 component = al_r20_r38_up_add(
            fabsq(dot.value), g_bound);
        if (row == 0U || component > result.g_inf_bound) {
            result.g_inf_bound = component;
            result.worst_row = row;
        }
        ++result.certificate_dots;
        exact = exact && dot.exact && finiteq(dot.value) != 0
            && finiteq(dot.bound) != 0 && finiteq(g_bound) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        result.maximum_g_bound = fmaxq(result.maximum_g_bound, g_bound);
        certificate_material << dot.root << ':'
            << al_binary128_hex(input_bound) << ';';

        const std::vector<Binary128> sum_left{
            solution[row], correction[row]};
        const std::vector<Binary128> sum_right{
            static_cast<Binary128>(1.0), static_cast<Binary128>(1.0)};
        const ALR20R38Dot2 sum = al_r20_r38_dot2(sum_left, sum_right);
        result.solution[row] = sum.value;
        center_bound[row] = sum.bound;
        ++result.sum_dots;
        exact = exact && sum.exact && finiteq(sum.value) != 0
            && finiteq(sum.bound) != 0;
        no_underflow = no_underflow && sum.no_underflow;
        result.maximum_center_bound = fmaxq(
            result.maximum_center_bound, sum.bound);
        center_material << sum.root << ';';
    }
    result.certificate_root = sha256_hex(certificate_material.str());
    result.center_root = sha256_hex(center_material.str());
    if (result.contractive) {
        const Binary128 denominator = nextafterq(
            static_cast<Binary128>(1.0) - result.rho_bound, -HUGE_VALQ);
        result.radius = al_r20_r38_up_divide(
            result.g_inf_bound, denominator);
        result.error = al_r20_r38_up_add(
            result.radius, result.maximum_center_bound);
        for (Binary128 value : result.solution) {
            result.minimum_separation = fminq(
                result.minimum_separation, fabsq(value) - result.error);
            if (value + result.error < static_cast<Binary128>(0.0))
                ++result.negative;
            else if (value - result.error > static_cast<Binary128>(0.0))
                ++result.positive;
            else
                ++result.unresolved;
        }
    } else {
        result.unresolved = dimension;
        result.minimum_separation = static_cast<Binary128>(0.0);
    }
    result.no_underflow = no_underflow;
    result.exact = exact
        && result.left_dots == dimension * dimension
        && result.residual_dots == dimension && result.z_dots == dimension
        && result.center_dots == center_depth * dimension
        && result.certificate_dots == dimension
        && result.sum_dots == dimension
        && al_r20_admm_finite(result.solution)
        && finiteq(result.rho_bound) != 0
        && finiteq(result.g_inf_bound) != 0
        && finiteq(result.radius) != 0
        && finiteq(result.error) != 0
        && finiteq(result.minimum_separation) != 0;
    const bool r41_values_exact = !require_first_target_values
        || (result.rho_bound == strtoflt128(
                "1.317676975770132537272002207352103601e-01", nullptr)
            && ((center_depth == 4U
                    && result.radius == strtoflt128(
                        "1.421399732012795356965411694525785042e-01",
                        nullptr))
                || (center_depth == 8U
                    && result.radius == strtoflt128(
                        "4.575300864161587341257666716201492586e-14",
                        nullptr)))
            && result.positive == 24U && result.negative == 41U);
    result.passed = result.exact && result.no_underflow
        && result.contractive && r41_values_exact
        && result.unresolved == 0U
        && result.minimum_separation > static_cast<Binary128>(0.0);
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':'
        << result.no_underflow << ':' << result.contractive << ':'
        << dimension << ':' << result.left_dots << ':'
        << result.residual_dots << ':' << result.z_dots << ':'
        << result.center_dots << ':' << result.certificate_dots << ':'
        << result.sum_dots << ':' << result.positive << ':'
        << result.negative << ':' << result.unresolved << ':'
        << result.worst_row << ':' << al_binary128_hex(result.rho_bound)
        << ':' << al_binary128_hex(result.g_inf_bound) << ':'
        << al_binary128_hex(result.radius) << ':'
        << al_binary128_hex(result.error) << ':'
        << al_binary128_hex(result.minimum_separation) << ':'
        << result.left_root << ':' << result.residual_root << ':'
        << result.z_root << ':' << result.generation_root << ':'
        << result.certificate_root << ':' << result.center_root << ':'
        << AL_R20_R41_DEPTH4_SHA256;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R42HookTrace {
    std::size_t attempts = 0U;
    std::size_t target_matches = 0U;
    std::size_t replacements = 0U;
    std::size_t non_target_attempts = 0U;
    ALR20R42PracticalCertificate certificate;
    std::string target_material_root;
};

thread_local ALR20R42HookTrace* al_r20_r42_hook_trace = nullptr;

ALR20VerifiedSolutionRefinement al_r20_r42_refiner(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20VerifiedSolutionRefinement result;
    ALR20R42HookTrace* trace = al_r20_r42_hook_trace;
    if (trace == nullptr) return result;
    ++trace->attempts;
    const std::string matrix_root = al_r20_q_scalar_root(matrix);
    const std::string inverse_root = al_r20_q_scalar_root(inverse);
    const std::string rhs_root = al_r20_q_scalar_root(rhs);
    const std::string solution_root = al_r20_q_scalar_root(solution);
    trace->target_material_root = sha256_hex(matrix_root + ':' + inverse_root
        + ':' + rhs_root + ':' + solution_root);
    const bool target = matrix_root == AL_R20_R37_MATRIX_SHA256
        && inverse_root == AL_R20_R37_INVERSE_SHA256
        && rhs_root == AL_R20_R39_RHS_SHA256
        && solution_root == AL_R20_R39_SOLUTION_SHA256;
    if (!target || trace->replacements != 0U) {
        trace->non_target_attempts += !target ? 1U : 0U;
        result.exact = true;
        result.root = sha256_hex(std::string("r42-non-target:")
            + trace->target_material_root + ':'
            + std::to_string(trace->replacements));
        return result;
    }
    ++trace->target_matches;
    trace->certificate = al_r20_r42_practical_certificate(
        matrix, inverse, rhs, solution);
    result.exact = trace->certificate.exact;
    result.passed = trace->certificate.passed;
    result.error = trace->certificate.error;
    result.solution = trace->certificate.solution;
    result.root = trace->certificate.root;
    if (result.passed) ++trace->replacements;
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v4_centered_refinement_trajectory_controls() {
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
        && al_r20_r21_event_controls()
        && al_r20_r37_dyadic_controls()
        && al_r20_r38_dot2_controls();
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
            cases[0U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[0U];
    }
    ALR20R42HookTrace trace;
    ALR20R35CaseAudit torsion;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r42_hook_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r42_hook_trace = &trace;
        al_r20_verified_solution_refiner = &al_r20_r42_refiner;
        torsion = al_r20_r35_case(problem);
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r42_hook_trace = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_r42_hook_trace == nullptr;
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool observation_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(torsion);
    const bool hook_exact = trace.certificate.exact
        && trace.certificate.passed && trace.certificate.no_underflow;
    const bool cardinality_exact = trace.target_matches == 1U
        && trace.replacements == 1U;
    const bool consumed = cardinality_exact
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.total_transitions > 0U;
    const bool exact = observation_exact && hook_exact
        && cardinality_exact && consumed && lifecycle_exact;
    const std::string route = !parent_exact || !observation_exact
        ? "CENTERED_REFINEMENT_PARENT_REJECTED"
        : (!hook_exact
            ? "CENTERED_REFINEMENT_HOOK_REJECTED"
            : (!cardinality_exact
                ? "CENTERED_REFINEMENT_TARGET_CARDINALITY_REJECTED"
                : (!consumed
                    ? "CENTERED_REFINEMENT_NOT_CONSUMED"
                    : (torsion.diagnostic.certified
                        ? "CENTERED_REFINEMENT_TORSION_CANDIDATE"
                        : "CENTERED_REFINEMENT_LATER_BOUNDARY"))));
    const ALR20SemismoothStep* final_step = torsion.diagnostic.steps.empty()
        ? nullptr : &torsion.diagnostic.steps.back();
    const std::string final_solve_failure = final_step == nullptr
        ? "NONE" : final_step->solve.failure;
    const std::string final_step_root = final_step == nullptr
        ? "" : final_step->root;
    const ALR20NNQPSolve* final_solve = final_step == nullptr
        ? nullptr : &final_step->solve;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r42-trajectory.v1|"
        << AL_R20_R41_SEMANTIC_SHA256 << ':' << route << ':'
        << trace.certificate.root << ':' << torsion.root << ':'
        << final_step_root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r42_trajectory.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r41_sha256\":\""
        << AL_R20_R41_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"observation_exact\":"
        << (observation_exact ? "true" : "false")
        << ",\"hook_slot_clear\":"
        << (hook_slot_clear ? "true" : "false")
        << ",\"hook_slot_released\":"
        << (hook_slot_released ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"hook\":{\"attempts\":" << trace.attempts
        << ",\"target_matches\":" << trace.target_matches
        << ",\"replacements\":" << trace.replacements
        << ",\"non_target_attempts\":" << trace.non_target_attempts
        << ",\"material_root\":\"" << trace.target_material_root << "\"}"
        << ",\"certificate\":{\"root\":\"" << trace.certificate.root
        << "\",\"rho_bound\":\""
        << al_binary128_text(trace.certificate.rho_bound)
        << "\",\"g_inf_bound\":\""
        << al_binary128_text(trace.certificate.g_inf_bound)
        << "\",\"radius\":\""
        << al_binary128_text(trace.certificate.radius)
        << "\",\"uniform_error\":\""
        << al_binary128_text(trace.certificate.error)
        << "\",\"positive\":" << trace.certificate.positive
        << ",\"negative\":" << trace.certificate.negative
        << ",\"unresolved\":" << trace.certificate.unresolved
        << ",\"minimum_separation\":\""
        << al_binary128_text(trace.certificate.minimum_separation)
        << "\",\"left_root\":\"" << trace.certificate.left_root
        << "\",\"residual_root\":\"" << trace.certificate.residual_root
        << "\",\"z_root\":\"" << trace.certificate.z_root
        << "\",\"generation_root\":\""
        << trace.certificate.generation_root
        << "\",\"certificate_root\":\""
        << trace.certificate.certificate_root
        << "\",\"center_root\":\"" << trace.certificate.center_root
        << "\"}"
        << ",\"trajectory\":{\"case_root\":\"" << torsion.root
        << "\",\"case_route\":\"" << torsion.route
        << "\",\"certified\":"
        << (torsion.diagnostic.certified ? "true" : "false")
        << ",\"active_set_rejected\":"
        << (torsion.diagnostic.active_set_rejected ? "true" : "false")
        << ",\"direction_rejected\":"
        << (torsion.diagnostic.direction_rejected ? "true" : "false")
        << ",\"globalization_rejected\":"
        << (torsion.diagnostic.globalization_rejected ? "true" : "false")
        << ",\"accepted_iterations\":"
        << torsion.diagnostic.accepted_iterations
        << ",\"principal_solves\":"
        << torsion.diagnostic.total_principal_solves
        << ",\"transitions\":" << torsion.diagnostic.total_transitions
        << ",\"steps\":" << torsion.diagnostic.steps.size()
        << ",\"final_solve_failure\":\"" << final_solve_failure
        << "\",\"final_step_root\":\"" << final_step_root << "\""
        << ",\"ratio_failure\":\""
        << (final_solve == nullptr ? "NONE" : final_solve->ratio_failure)
        << "\",\"ratio_best_source_row\":"
        << (final_solve == nullptr ? 0U
            : final_solve->ratio_best_source_row)
        << ",\"ratio_competitor_source_row\":"
        << (final_solve == nullptr ? 0U
            : final_solve->ratio_competitor_source_row)
        << ",\"ratio_best_value\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_best_value)
        << "\",\"ratio_competitor_value\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_competitor_value)
        << "\",\"ratio_best_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_best_bound)
        << "\",\"ratio_competitor_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_competitor_bound)
        << "\",\"ratio_current_error\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_current_error)
        << "\",\"ratio_candidate_error\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_candidate_error)
        << "\",\"ratio_best_round_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_best_round_bound)
        << "\",\"ratio_best_input_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_best_input_bound)
        << "\",\"ratio_competitor_round_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_competitor_round_bound)
        << "\",\"ratio_competitor_input_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_competitor_input_bound)
        << "\",\"ratio_inverse_audit_passed\":"
        << (final_solve != nullptr
                && final_solve->ratio_inverse_audit.passed
            ? "true" : "false")
        << ",\"ratio_inverse_audit_root\":\""
        << (final_solve == nullptr
            ? "" : final_solve->ratio_inverse_audit.root)
        << "\",\"ratio_refined_failure\":\""
        << (final_solve == nullptr
            ? "NOT_RUN" : final_solve->ratio_refined_failure)
        << "\",\"ratio_affine_refined_failure\":\""
        << (final_solve == nullptr
            ? "NOT_RUN" : final_solve->ratio_affine_refined_failure)
        << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0"
        << ",\"correction_applies\":" << trace.replacements
        << ",\"counterflow_replays\":0,\"new_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
