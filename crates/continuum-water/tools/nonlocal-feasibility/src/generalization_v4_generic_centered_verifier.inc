namespace {

constexpr const char* AL_R20_R49_SEMANTIC_SHA256 =
    "719e0d5067dadc7ab09374e923f3f764ae84e1f509784b0b11339024493df3cc";
constexpr std::size_t AL_R20_R50_CENTER_DEPTH = 16U;
constexpr std::size_t AL_R20_R50_DIMENSION = 65U;
constexpr std::size_t AL_R20_R50_DOTS_PER_CERTIFICATE = 5525U;
constexpr std::size_t AL_R20_R50_PAIRS_PER_CERTIFICATE = 360490U;

struct ALR20R50Entry {
    std::size_t dimension = 0U;
    bool worked = false;
    bool replaced = false;
    ALR20R42PracticalCertificate certificate;
    std::string material_root;
    std::string matrix_root;
    std::string inverse_root;
    std::string rhs_root;
    std::string solution_root;
    std::size_t dots = 0U;
    std::size_t input_pairs = 0U;
};

struct ALR20R50Trace {
    std::vector<ALR20R50Entry> entries;
    std::size_t replacements = 0U;
    std::size_t certificate_rejections = 0U;
    std::size_t dimension_rejections = 0U;
    std::size_t cap_hits = 0U;
};

thread_local ALR20R50Trace* al_r20_r50_trace = nullptr;

ALR20VerifiedSolutionRefinement al_r20_r50_refiner(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20VerifiedSolutionRefinement result;
    ALR20R50Trace* trace = al_r20_r50_trace;
    if (trace == nullptr) return result;
    ALR20R50Entry entry;
    entry.dimension = rhs.size();
    entry.matrix_root = al_r20_q_scalar_root(matrix);
    entry.inverse_root = al_r20_q_scalar_root(inverse);
    entry.rhs_root = al_r20_q_scalar_root(rhs);
    entry.solution_root = al_r20_q_scalar_root(solution);
    entry.material_root = sha256_hex(entry.matrix_root + ':'
        + entry.inverse_root + ':' + entry.rhs_root + ':'
        + entry.solution_root);
    if (entry.dimension != AL_R20_R50_DIMENSION) {
        ++trace->dimension_rejections;
        result.exact = true;
        result.root = sha256_hex(std::string("r50-dimension-rejected:")
            + entry.material_root + ':' + std::to_string(entry.dimension));
        trace->entries.push_back(std::move(entry));
        return result;
    }
    if (trace->replacements >= AL_R20_SEMISMOOTH_ITERATION_CAP) {
        ++trace->cap_hits;
        result.exact = true;
        result.root = sha256_hex(std::string("r50-cap-rejected:")
            + entry.material_root + ':'
            + std::to_string(trace->replacements));
        trace->entries.push_back(std::move(entry));
        return result;
    }
    entry.worked = true;
    entry.certificate = al_r20_r42_practical_certificate(
        matrix, inverse, rhs, solution, AL_R20_R50_CENTER_DEPTH, false);
    entry.dots = entry.certificate.left_dots
        + entry.certificate.residual_dots + entry.certificate.z_dots
        + entry.certificate.center_dots
        + entry.certificate.certificate_dots + entry.certificate.sum_dots;
    entry.input_pairs = entry.dimension * entry.dimension
            * (entry.dimension + 1U)
        + entry.dimension * (entry.dimension + 1U)
        + entry.dimension * entry.dimension
        + AL_R20_R50_CENTER_DEPTH * entry.dimension
            * (entry.dimension + 1U)
        + entry.dimension * (entry.dimension + 2U)
        + 2U * entry.dimension;
    result.exact = entry.certificate.exact;
    result.passed = entry.certificate.passed;
    result.error = entry.certificate.error;
    result.solution = entry.certificate.solution;
    result.root = entry.certificate.root;
    entry.replaced = result.passed;
    if (entry.replaced)
        ++trace->replacements;
    else
        ++trace->certificate_rejections;
    trace->entries.push_back(std::move(entry));
    return result;
}

bool al_r20_r50_tuple_exact(
    const ALR20R50Entry& entry,
    const char* matrix_root,
    const char* inverse_root,
    const char* rhs_root,
    const char* solution_root) {
    return entry.matrix_root == matrix_root
        && entry.inverse_root == inverse_root
        && entry.rhs_root == rhs_root
        && entry.solution_root == solution_root;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_generic_centered_verifier_controls() {
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
    ALR20R50Trace trace;
    ALR20R35CaseAudit torsion;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r50_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r50_trace = &trace;
        al_r20_verified_solution_refiner = &al_r20_r50_refiner;
        torsion = al_r20_r35_case(problem);
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r50_trace = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_r50_trace == nullptr;
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool observation_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(torsion);
    bool regression_exact = trace.entries.size() >= 4U;
    if (regression_exact) {
        regression_exact = al_r20_r50_tuple_exact(trace.entries[0U],
                AL_R20_R37_MATRIX_SHA256, AL_R20_R37_INVERSE_SHA256,
                AL_R20_R39_RHS_SHA256, AL_R20_R39_SOLUTION_SHA256)
            && al_r20_r50_tuple_exact(trace.entries[1U],
                AL_R20_R43_SECOND_MATRIX_SHA256,
                AL_R20_R43_SECOND_INVERSE_SHA256,
                AL_R20_R43_SECOND_RHS_SHA256,
                AL_R20_R43_SECOND_SOLUTION_SHA256)
            && al_r20_r50_tuple_exact(trace.entries[2U],
                AL_R20_R46_THIRD_MATRIX_SHA256,
                AL_R20_R46_THIRD_INVERSE_SHA256,
                AL_R20_R46_THIRD_RHS_SHA256,
                AL_R20_R46_THIRD_SOLUTION_SHA256)
            && al_r20_r50_tuple_exact(trace.entries[3U],
                "e24fa32ed6b787c8afc5544c470d8f32f306e2d4360e45f58e109281d1a64054",
                "b583ab4674ce6c87623d3351691a6b548a07c3ebe387315053cdd52fa07316d6",
                "a9d5748646c3c4dd0eed55db9f1992e4ac031920386afccf55d1e2c2ea50543c",
                "819d8ce96231fa71f19d418b17d1b061f07f6e8573452b0899e48f34e96e42b9")
            && trace.entries[1U].certificate.root
                == AL_R20_R46_SECOND_CERTIFICATE_SHA256
            && trace.entries[2U].certificate.root
                == "dc4449ec2ab13d2f501c520e5f5a5c42663d84718b7e2cad416b75f4a104e3a4";
    }
    bool apparatus_exact = trace.dimension_rejections == 0U;
    bool work_exact = true;
    std::size_t total_dots = 0U;
    std::size_t total_input_pairs = 0U;
    std::size_t worked_certificates = 0U;
    std::size_t first_failed_certificate = trace.entries.size();
    std::ostringstream entry_material;
    std::ostringstream entry_report;
    entry_report << '[';
    for (std::size_t index = 0U; index < trace.entries.size(); ++index) {
        const ALR20R50Entry& entry = trace.entries[index];
        if (index != 0U) entry_report << ',';
        if (entry.worked) {
            ++worked_certificates;
            apparatus_exact = apparatus_exact && entry.certificate.exact
                && entry.certificate.no_underflow;
            work_exact = work_exact
                && entry.dots == AL_R20_R50_DOTS_PER_CERTIFICATE
                && entry.input_pairs == AL_R20_R50_PAIRS_PER_CERTIFICATE;
            total_dots += entry.dots;
            total_input_pairs += entry.input_pairs;
            if (!entry.replaced
                && first_failed_certificate == trace.entries.size())
                first_failed_certificate = index;
        }
        entry_material << entry.material_root << ':'
            << entry.certificate.root << ':' << entry.replaced << ';';
        entry_report << "{\"index\":" << index
            << ",\"dimension\":" << entry.dimension
            << ",\"worked\":" << (entry.worked ? "true" : "false")
            << ",\"replaced\":" << (entry.replaced ? "true" : "false")
            << ",\"material_root\":\"" << entry.material_root
            << "\",\"matrix_root\":\"" << entry.matrix_root
            << "\",\"inverse_root\":\"" << entry.inverse_root
            << "\",\"rhs_root\":\"" << entry.rhs_root
            << "\",\"solution_root\":\"" << entry.solution_root
            << "\",\"certificate_root\":\"" << entry.certificate.root
            << "\",\"rho_bound\":\""
            << al_binary128_text(entry.certificate.rho_bound)
            << "\",\"radius\":\""
            << al_binary128_text(entry.certificate.radius)
            << "\",\"positive\":" << entry.certificate.positive
            << ",\"negative\":" << entry.certificate.negative
            << ",\"unresolved\":" << entry.certificate.unresolved
            << ",\"minimum_separation\":\""
            << al_binary128_text(entry.certificate.minimum_separation)
            << "\",\"dots\":" << entry.dots
            << ",\"input_pairs\":" << entry.input_pairs << '}';
    }
    entry_report << ']';
    const std::string entry_root = sha256_hex(entry_material.str());
    work_exact = work_exact
        && total_dots == worked_certificates
            * AL_R20_R50_DOTS_PER_CERTIFICATE
        && total_input_pairs == worked_certificates
            * AL_R20_R50_PAIRS_PER_CERTIFICATE
        && total_dots <= AL_R20_SEMISMOOTH_ITERATION_CAP
            * AL_R20_R50_DOTS_PER_CERTIFICATE
        && total_input_pairs <= AL_R20_SEMISMOOTH_ITERATION_CAP
            * AL_R20_R50_PAIRS_PER_CERTIFICATE;
    const bool cap_exact = trace.cap_hits == 0U
        && trace.replacements <= AL_R20_SEMISMOOTH_ITERATION_CAP;
    const ALR20SemismoothStep* final_step = torsion.diagnostic.steps.empty()
        ? nullptr : &torsion.diagnostic.steps.back();
    const ALR20NNQPSolve* final_solve = final_step == nullptr
        ? nullptr : &final_step->solve;
    const bool later_inverse_boundary = final_solve != nullptr
        && final_solve->failure == "VERIFIED_INVERSE_AUDIT_REJECTED";
    const bool exact = observation_exact && regression_exact
        && apparatus_exact && work_exact && cap_exact && lifecycle_exact;
    const std::string route = !parent_exact || !observation_exact
        ? "GENERIC_VERIFIER_PARENT_REJECTED"
        : (!regression_exact
            ? "GENERIC_VERIFIER_REGRESSION_REJECTED"
            : (!apparatus_exact
                ? "GENERIC_VERIFIER_APPARATUS_REJECTED"
                : (!work_exact
                    ? "GENERIC_VERIFIER_WORK_REJECTED"
                    : (!cap_exact
                        ? "GENERIC_VERIFIER_CAP_REJECTED"
                        : (later_inverse_boundary
                            ? "GENERIC_VERIFIER_LATER_INVERSE_BOUNDARY"
                            : (!torsion.diagnostic.certified
                                ? "GENERIC_VERIFIER_LATER_BOUNDARY"
                                : "GENERIC_VERIFIER_TORSION_CANDIDATE"))))));
    const std::string final_step_root = final_step == nullptr
        ? "" : final_step->root;
    const std::string final_solve_failure = final_solve == nullptr
        ? "NONE" : final_solve->failure;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r50-generic-verifier.v1|"
        << AL_R20_R49_SEMANTIC_SHA256 << ':' << route << ':'
        << entry_root << ':' << torsion.root << ':' << final_step_root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r50_generic_verifier.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r49_sha256\":\""
        << AL_R20_R49_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"observation_exact\":"
        << (observation_exact ? "true" : "false")
        << ",\"regression_exact\":"
        << (regression_exact ? "true" : "false")
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"cap_exact\":" << (cap_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"policy\":{\"dimension\":65,\"center_depth\":16"
        << ",\"replacement_cap\":32,\"root_selected\":false}"
        << ",\"trace\":{\"calls\":" << trace.entries.size()
        << ",\"worked_certificates\":" << worked_certificates
        << ",\"replacements\":" << trace.replacements
        << ",\"certificate_rejections\":"
        << trace.certificate_rejections
        << ",\"dimension_rejections\":" << trace.dimension_rejections
        << ",\"cap_hits\":" << trace.cap_hits
        << ",\"first_failed_certificate\":"
        << first_failed_certificate
        << ",\"entry_root\":\"" << entry_root
        << "\",\"entries\":" << entry_report.str() << '}'
        << ",\"work\":{\"dots_per_certificate\":5525"
        << ",\"pairs_per_certificate\":360490"
        << ",\"total_dots\":" << total_dots
        << ",\"total_input_pairs\":" << total_input_pairs
        << ",\"maximum_dots\":176800"
        << ",\"maximum_input_pairs\":11535680}"
        << ",\"trajectory\":{\"case_root\":\"" << torsion.root
        << "\",\"case_route\":\"" << torsion.route
        << "\",\"certified\":"
        << (torsion.diagnostic.certified ? "true" : "false")
        << ",\"accepted_iterations\":"
        << torsion.diagnostic.accepted_iterations
        << ",\"principal_solves\":"
        << torsion.diagnostic.total_principal_solves
        << ",\"transitions\":" << torsion.diagnostic.total_transitions
        << ",\"steps\":" << torsion.diagnostic.steps.size()
        << ",\"final_solve_failure\":\"" << final_solve_failure
        << "\",\"final_step_root\":\"" << final_step_root << "\"}"
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
