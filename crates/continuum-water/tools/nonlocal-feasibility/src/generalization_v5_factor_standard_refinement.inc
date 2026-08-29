namespace {

constexpr const char* AL_R20_R63K_PARENT_SEMANTIC_SHA256 =
    "4a4d1798a88ba0adcc4fd869fefb0ddc536491f5bad1bd49cc6505d57aa78f96";
constexpr const char* AL_R20_R63K_DEFECT_SHA256 =
    "85af6bc432b5c3bf64df2b63621bb1a07d987afe66453ec4ec769bbcfddf839b";
constexpr const char* AL_R20_R63K_WIDE_PARENT_LANE_SHA256 =
    "f069a3186265b4d8c17d04971bdcfe999438410d38a2660976ac5039503ad641";
constexpr const char* AL_R20_R63K_EXPORT_PARENT_LANE_SHA256 =
    "303a150579e25683c2f163e7f93f1a6cceb03f170009ada14c22c8f381df8fb5";
constexpr const char* AL_R20_R63K_STRICT_PARENT_LANE_SHA256 =
    "01072b92e73af43baa6ee93e92e5118e277bcd591d367aa67aa211a1daeb6555";
constexpr std::array<std::size_t, 5U> AL_R20_R63K_DEPTHS{
    1U, 2U, 4U, 8U, 16U};

bool al_r20_r63k_checkpoint_depth(std::size_t depth) {
    return std::find(AL_R20_R63K_DEPTHS.begin(),
        AL_R20_R63K_DEPTHS.end(), depth) != AL_R20_R63K_DEPTHS.end();
}

struct ALR20R63KResidual {
    bool exact = false;
    bool no_underflow = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    Binary128 infinity_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128> value;
    std::vector<Binary128> bound;
    std::string root;
};

ALR20R63KResidual al_r20_r63k_residual(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& candidate) {
    ALR20R63KResidual result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || matrix.size() != dimension * dimension
        || candidate.size() != dimension) return result;
    result.value.resize(dimension);
    result.bound.resize(dimension);
    bool exact = true;
    bool no_underflow = true;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension + 1U);
        std::vector<Binary128> right(dimension + 1U);
        left[0U] = rhs[row];
        right[0U] = static_cast<Binary128>(1.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            left[column + 1U] = matrix[row * dimension + column];
            right[column + 1U] = -candidate[column];
        }
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
        result.value[row] = dot.value;
        result.bound[row] = dot.bound;
        exact = exact && dot.exact && finiteq(dot.value) != 0
            && finiteq(dot.bound) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        result.infinity_bound = fmaxq(result.infinity_bound,
            al_r20_r38_up_add(fabsq(dot.value), dot.bound));
        ++result.dots;
    }
    result.no_underflow = no_underflow;
    result.exact = exact && no_underflow && result.dots == dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.dimension << ':' << result.dots << ':'
        << al_binary128_hex(result.infinity_bound) << ':'
        << al_r20_q_scalar_root(result.value) << ':'
        << al_r20_q_scalar_root(result.bound);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63KCheckpoint {
    bool exact = false;
    std::size_t iteration = 0U;
    ALR20R63ISignCertificate certificate;
    std::string root;
};

struct ALR20R63KLane {
    bool exact = false;
    bool finite = false;
    bool passed = false;
    bool monotone_error = false;
    std::size_t dimension = 0U;
    std::size_t completed_iterations = 0U;
    std::size_t failure_iteration = 0U;
    std::size_t residual_dots = 0U;
    std::size_t update_dots = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::size_t certificate_residual_dots = 0U;
    std::size_t certificate_image_dots = 0U;
    std::size_t first_passing_iteration = 0U;
    Binary128 minimum_error = HUGE_VALQ;
    std::vector<Binary128> solution;
    std::vector<double> solution_d;
    std::vector<std::string> iteration_roots;
    std::vector<ALR20R63KCheckpoint> checkpoints;
    std::string failure_stage;
    std::string root;
};

ALR20R63KLane al_r20_r63k_q_lane(
    const std::vector<Binary128>& upper,
    const std::vector<std::size_t>& permutation,
    Binary128 inverse_scale,
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& initial,
    const ALR20R63ISignCertificate& baseline,
    const std::vector<int>& reference_signs,
    const std::vector<Binary128>& reference_solution) {
    ALR20R63KLane result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension
        || initial.size() != dimension || !baseline.exact
        || baseline.passed || reference_signs.size() != dimension
        || reference_solution.size() != dimension) return result;
    result.solution = initial;
    bool exact = true;
    bool finite = true;
    bool monotone = true;
    Binary128 previous_error = baseline.error;
    for (std::size_t iteration = 1U; iteration <= 16U; ++iteration) {
        const ALR20R63KResidual residual = al_r20_r63k_residual(
            matrix, rhs, result.solution);
        result.residual_dots += residual.dots;
        if (!residual.exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "residual";
            break;
        }
        const ALR20R63IQSolve correction = al_r20_r63i_q_solve(
            upper, permutation, residual.value, inverse_scale);
        result.forward_terms += correction.forward_terms;
        result.backward_terms += correction.backward_terms;
        result.divisions += correction.divisions;
        if (!correction.exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "factor_solve";
            break;
        }
        std::vector<Binary128> updated(dimension);
        std::ostringstream update_material;
        bool update_exact = true;
        for (std::size_t row = 0U; row < dimension; ++row) {
            const std::vector<Binary128> left{
                result.solution[row], correction.solution[row]};
            const std::vector<Binary128> right{
                static_cast<Binary128>(1.0),
                static_cast<Binary128>(1.0)};
            const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
            updated[row] = dot.value;
            update_exact = update_exact && dot.exact
                && dot.no_underflow && finiteq(dot.value) != 0
                && finiteq(dot.bound) != 0;
            update_material << dot.root << ';';
            ++result.update_dots;
        }
        if (!update_exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "update";
            break;
        }
        result.solution = std::move(updated);
        ++result.completed_iterations;
        result.iteration_roots.push_back(sha256_hex(
            residual.root + ':' + correction.root + ':'
            + sha256_hex(update_material.str()) + ':'
            + al_r20_q_scalar_root(result.solution)));
        if (!al_r20_r63k_checkpoint_depth(iteration)) continue;
        ALR20R63KCheckpoint checkpoint;
        checkpoint.iteration = iteration;
        checkpoint.certificate = al_r20_r63i_sign_certificate(
            matrix, inverse, rhs, result.solution, reference_signs,
            baseline.rho_bound, reference_solution);
        checkpoint.exact = checkpoint.certificate.exact;
        result.certificate_residual_dots +=
            checkpoint.certificate.residual_dots;
        result.certificate_image_dots +=
            checkpoint.certificate.image_dots;
        monotone = monotone
            && checkpoint.certificate.error < previous_error;
        previous_error = checkpoint.certificate.error;
        result.minimum_error = fminq(
            result.minimum_error, checkpoint.certificate.error);
        if (checkpoint.certificate.passed
            && result.first_passing_iteration == 0U)
            result.first_passing_iteration = iteration;
        checkpoint.root = sha256_hex(std::to_string(checkpoint.exact) + ':'
            + std::to_string(checkpoint.iteration) + ':'
            + checkpoint.certificate.root);
        exact = exact && checkpoint.exact;
        result.checkpoints.push_back(std::move(checkpoint));
    }
    result.finite = finite;
    result.monotone_error = finite && monotone;
    const bool complete = result.completed_iterations == 16U;
    result.passed = finite && complete
        && result.first_passing_iteration != 0U;
    result.exact = exact && ((finite && complete
            && result.checkpoints.size() == AL_R20_R63K_DEPTHS.size())
        || (!finite && result.failure_iteration != 0U
            && !result.failure_stage.empty()));
    std::ostringstream material;
    material << result.exact << ':' << result.finite << ':'
        << result.passed << ':' << result.monotone_error << ':'
        << result.dimension << ':' << result.completed_iterations << ':'
        << result.failure_iteration << ':' << result.failure_stage << ':'
        << result.residual_dots << ':' << result.update_dots << ':'
        << result.forward_terms << ':' << result.backward_terms << ':'
        << result.divisions << ':' << result.certificate_residual_dots << ':'
        << result.certificate_image_dots << ':'
        << result.first_passing_iteration << ':'
        << al_binary128_hex(result.minimum_error) << ':'
        << al_r20_q_scalar_root(result.solution) << '|';
    for (const std::string& root : result.iteration_roots)
        material << root << ';';
    material << '|';
    for (const ALR20R63KCheckpoint& checkpoint : result.checkpoints)
        material << checkpoint.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

ALR20R63KLane al_r20_r63k_d_lane(
    const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation,
    Binary128 inverse_scale,
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<double>& initial,
    const ALR20R63ISignCertificate& baseline,
    const std::vector<int>& reference_signs,
    const std::vector<Binary128>& reference_solution) {
    ALR20R63KLane result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension
        || initial.size() != dimension || !baseline.exact
        || baseline.passed || reference_signs.size() != dimension
        || reference_solution.size() != dimension) return result;
    result.solution_d = initial;
    bool exact = true;
    bool finite = true;
    bool monotone = true;
    Binary128 previous_error = baseline.error;
    for (std::size_t iteration = 1U; iteration <= 16U; ++iteration) {
        result.solution = al_r20_r63g_q_from_double(result.solution_d);
        const ALR20R63KResidual residual = al_r20_r63k_residual(
            matrix, rhs, result.solution);
        result.residual_dots += residual.dots;
        if (!residual.exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "residual";
            break;
        }
        const ALR20R63IDSolve correction = al_r20_r63i_d_solve(
            upper, permutation, residual.value, inverse_scale);
        result.forward_terms += correction.forward_terms;
        result.backward_terms += correction.backward_terms;
        result.divisions += correction.divisions;
        if (!correction.exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "factor_solve";
            break;
        }
        std::vector<double> updated(dimension);
        bool update_exact = true;
        for (std::size_t row = 0U; row < dimension; ++row) {
            updated[row] = result.solution_d[row] + correction.solution[row];
            update_exact = update_exact && std::isfinite(updated[row]);
            ++result.update_dots;
        }
        if (!update_exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "update";
            break;
        }
        result.solution_d = std::move(updated);
        result.solution = al_r20_r63g_q_from_double(result.solution_d);
        ++result.completed_iterations;
        result.iteration_roots.push_back(sha256_hex(
            residual.root + ':' + correction.root + ':'
            + al_r20_double_root(result.solution_d)));
        if (!al_r20_r63k_checkpoint_depth(iteration)) continue;
        ALR20R63KCheckpoint checkpoint;
        checkpoint.iteration = iteration;
        checkpoint.certificate = al_r20_r63i_sign_certificate(
            matrix, inverse, rhs, result.solution, reference_signs,
            baseline.rho_bound, reference_solution);
        checkpoint.exact = checkpoint.certificate.exact;
        result.certificate_residual_dots +=
            checkpoint.certificate.residual_dots;
        result.certificate_image_dots +=
            checkpoint.certificate.image_dots;
        monotone = monotone
            && checkpoint.certificate.error < previous_error;
        previous_error = checkpoint.certificate.error;
        result.minimum_error = fminq(
            result.minimum_error, checkpoint.certificate.error);
        if (checkpoint.certificate.passed
            && result.first_passing_iteration == 0U)
            result.first_passing_iteration = iteration;
        checkpoint.root = sha256_hex(std::to_string(checkpoint.exact) + ':'
            + std::to_string(checkpoint.iteration) + ':'
            + checkpoint.certificate.root);
        exact = exact && checkpoint.exact;
        result.checkpoints.push_back(std::move(checkpoint));
    }
    result.finite = finite;
    result.monotone_error = finite && monotone;
    const bool complete = result.completed_iterations == 16U;
    result.passed = finite && complete
        && result.first_passing_iteration != 0U;
    result.exact = exact && ((finite && complete
            && result.checkpoints.size() == AL_R20_R63K_DEPTHS.size())
        || (!finite && result.failure_iteration != 0U
            && !result.failure_stage.empty()));
    std::ostringstream material;
    material << result.exact << ':' << result.finite << ':'
        << result.passed << ':' << result.monotone_error << ':'
        << result.dimension << ':' << result.completed_iterations << ':'
        << result.failure_iteration << ':' << result.failure_stage << ':'
        << result.residual_dots << ':' << result.update_dots << ':'
        << result.forward_terms << ':' << result.backward_terms << ':'
        << result.divisions << ':' << result.certificate_residual_dots << ':'
        << result.certificate_image_dots << ':'
        << result.first_passing_iteration << ':'
        << al_binary128_hex(result.minimum_error) << ':'
        << al_r20_double_root(result.solution_d) << '|';
    for (const std::string& root : result.iteration_roots)
        material << root << ';';
    material << '|';
    for (const ALR20R63KCheckpoint& checkpoint : result.checkpoints)
        material << checkpoint.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63k_classify(bool apparatus, bool wide,
    bool export_wide, bool strict_binary64) {
    if (!apparatus) return "FACTOR_STANDARD_REFINEMENT_APPARATUS_REJECTED";
    if (!wide) return "WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED";
    if (!export_wide)
        return "EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_REJECTED";
    if (!strict_binary64)
        return "EXPORTED_FACTOR_BINARY64_STANDARD_REFINEMENT_REJECTED";
    return "EXPORTED_FACTOR_STANDARD_REFINEMENT_CANDIDATE";
}

struct ALR20R63KControls {
    bool exact = false;
    bool ladder = false;
    bool scalar = false;
    bool failure = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63KControls al_r20_r63k_controls() {
    ALR20R63KControls result;
    result.ladder = AL_R20_R63K_DEPTHS
        == std::array<std::size_t, 5U>{1U, 2U, 4U, 8U, 16U};
    const std::vector<Binary128> matrix{static_cast<Binary128>(1.0)};
    const std::vector<Binary128> inverse{static_cast<Binary128>(0.75)};
    const std::vector<Binary128> rhs{static_cast<Binary128>(1.0)};
    const std::vector<Binary128> initial{static_cast<Binary128>(-3.0)};
    const std::vector<Binary128> reference{static_cast<Binary128>(1.0)};
    const std::vector<int> signs{1};
    const Binary128 upper_value = sqrtq(
        static_cast<Binary128>(4.0) / static_cast<Binary128>(3.0));
    const std::vector<Binary128> upper{upper_value};
    const std::vector<std::size_t> permutation{0U};
    const ALR20R63ISignCertificate baseline =
        al_r20_r63i_sign_certificate(matrix, inverse, rhs, initial, signs,
            static_cast<Binary128>(0.25), reference);
    const ALR20R63KLane lane = al_r20_r63k_q_lane(upper, permutation,
        static_cast<Binary128>(1.0), matrix, inverse, rhs, initial, baseline,
        signs, reference);
    result.scalar = lane.exact && lane.finite && lane.passed
        && lane.first_passing_iteration == 2U
        && lane.checkpoints.size() == 5U;
    const ALR20R63KLane failed = al_r20_r63k_q_lane(
        {static_cast<Binary128>(0.0)}, permutation,
        static_cast<Binary128>(1.0), matrix, inverse, rhs, initial, baseline,
        signs, reference);
    result.failure = failed.exact && !failed.finite && !failed.passed
        && failed.failure_iteration == 1U
        && failed.failure_stage == "factor_solve";
    std::vector<Binary128> mutated = lane.solution;
    mutated[0U] = nextafterq(mutated[0U], HUGE_VALQ);
    result.mutation = al_r20_q_scalar_root(mutated)
        != al_r20_q_scalar_root(lane.solution);
    result.classifier =
        al_r20_r63k_classify(false, true, true, true)
            == "FACTOR_STANDARD_REFINEMENT_APPARATUS_REJECTED"
        && al_r20_r63k_classify(true, false, true, true)
            == "WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED"
        && al_r20_r63k_classify(true, true, false, true)
            == "EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_REJECTED"
        && al_r20_r63k_classify(true, true, true, false)
            == "EXPORTED_FACTOR_BINARY64_STANDARD_REFINEMENT_REJECTED"
        && al_r20_r63k_classify(true, true, true, true)
            == "EXPORTED_FACTOR_STANDARD_REFINEMENT_CANDIDATE";
    result.exact = result.ladder && result.scalar && result.failure
        && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + lane.root + ':' + failed.root + ':'
        + al_r20_q_scalar_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_factor_standard_refinement_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__) && !defined(__FAST_MATH__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT128_MANT_DIG == 113
        && sizeof(double) == 8U && FLT_RADIX == 2 && DBL_MANT_DIG == 53
        && FLT_EVAL_METHOD == 0 && std::numeric_limits<double>::is_iec559
        && std::numeric_limits<double>::has_denorm == std::denorm_present
        && std::fegetround() == FE_TONEAREST;
    const ALR20R63KControls controls = al_r20_r63k_controls();
    const bool controls_exact = profile_exact && controls.exact;

    std::vector<ALR20ManifestCase> cases = al_r20_v5_manifest_cases();
    bool parent_exact = controls_exact && cases.size() == 6U
        && al_r20_v5_manifest_root(cases) == AL_R20_V5_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALR20ProblemData problem;
    bool problem_materialized = false;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[3U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.target.size() == 105U
            && problem.problem_root == AL_R20_R60_PROBLEM_SHA256;
    }

    ALR20R60Capture solution_capture;
    ALR20VerifiedInverseCapture factor_capture;
    factor_capture.selected_matrix_root = AL_R20_R63_MATRIX_SHA256;
    ALR20R55SlopeTrace slope;
    ALR20R35CaseAudit oblique;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_verified_inverse_projector_context == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r60_capture = &solution_capture;
        al_r20_verified_solution_refiner = &al_r20_r60_capture_refiner;
        al_r20_verified_inverse_capture = &factor_capture;
        al_r20_r55_slope_trace = &slope;
        al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
        oblique = al_r20_r35_case(problem);
        al_r20_slope_error_refiner = nullptr;
        al_r20_r55_slope_trace = nullptr;
        al_r20_verified_inverse_capture = nullptr;
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r60_capture = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released =
        al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_verified_inverse_projector_context == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && solution_capture.calls == 1U
        && factor_capture.calls == 125U
        && factor_capture.selected_calls == 1U
        && factor_capture.selected_column_solves == 102U
        && factor_capture.matrix_root == AL_R20_R63_MATRIX_SHA256
        && al_r20_q_scalar_root(factor_capture.inverse)
            == AL_R20_R63I_INVERSE_SHA256
        && al_r20_q_scalar_root(factor_capture.rhs)
            == AL_R20_R63I_RHS_SHA256
        && al_r20_q_scalar_root(factor_capture.solution)
            == AL_R20_R63I_SOLUTION_SHA256
        && factor_capture.rhs.size() == 102U
        && factor_capture.solution.size() == 102U
        && factor_capture.projector.exact
        && slope.calls == 0U && slope.inverse_audits == 0U;

    const ALR20R63EProjectorGrams grams = capture_exact
        ? al_r20_r63e_projector_grams(problem.op,
            factor_capture.source_rows, problem.target.size(),
            factor_capture.projector, factor_capture.matrix)
        : ALR20R63EProjectorGrams{};
    const ALR20R63ERankProfile full_profile = grams.exact
        ? al_r20_r63e_rank_profile(grams.full, grams.rows)
        : ALR20R63ERankProfile{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows);
    std::vector<Binary128> transposed;
    std::vector<double> permuted_d;
    if (tangent.exact && permutation_exact) {
        transposed.resize(tangent.columns * tangent.rows);
        permuted_d.resize(transposed.size());
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                transposed[scalar * tangent.rows + column] =
                    tangent.values[column * tangent.columns + scalar];
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                permuted_d[scalar * tangent.rows + column] =
                    static_cast<double>(transposed[
                        scalar * tangent.rows + permutation[column]]);
    }
    const ALR20R63GDFactor parent_d_factor = !permuted_d.empty()
        ? al_r20_r63g_d_factor(
            permuted_d, tangent.columns, tangent.rows)
        : ALR20R63GDFactor{};
    const std::vector<Binary128> stored_permuted =
        al_r20_r63g_q_from_double(parent_d_factor.input);
    const ALR20R63GQFactor wide_factor = !stored_permuted.empty()
        ? al_r20_r63g_q_factor(
            stored_permuted, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    std::vector<double> exported_r_d;
    if (wide_factor.exact) {
        exported_r_d.reserve(wide_factor.r.size());
        for (Binary128 value : wide_factor.r)
            exported_r_d.push_back(static_cast<double>(value));
    }
    const std::vector<Binary128> exported_r =
        al_r20_r63g_q_from_double(exported_r_d);
    const bool factor_exact = capture_exact && grams.exact
        && full_profile.exact && tangent.exact && permutation_exact
        && parent_d_factor.exact && wide_factor.exact
        && wide_factor.root == AL_R20_R63I_WIDE_FACTOR_SHA256
        && al_r20_double_root(exported_r_d)
            == AL_R20_R63I_EXPORT_R_SHA256;

    const ALR20R42PracticalCertificate reference_certificate = capture_exact
        ? al_r20_r42_practical_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            factor_capture.solution, 4U, false)
        : ALR20R42PracticalCertificate{};
    std::vector<int> reference_signs(102U, 0);
    bool reference_signs_exact = reference_certificate.passed
        && reference_certificate.solution.size() == 102U;
    for (std::size_t index = 0U;
         index < reference_signs.size() && reference_signs_exact; ++index) {
        if (reference_certificate.solution[index]
                - reference_certificate.error
            > static_cast<Binary128>(0.0))
            reference_signs[index] = 1;
        else if (reference_certificate.solution[index]
                + reference_certificate.error
            < static_cast<Binary128>(0.0))
            reference_signs[index] = -1;
        else
            reference_signs_exact = false;
    }
    const bool reference_exact = reference_signs_exact
        && reference_certificate.exact && reference_certificate.passed
        && reference_certificate.contractive
        && reference_certificate.positive == 24U
        && reference_certificate.negative == 78U
        && reference_certificate.unresolved == 0U
        && reference_certificate.root == AL_R20_R63J_REFERENCE_SHA256;

    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const ALR20R63IQSolve wide_solve = factor_exact
        ? al_r20_r63i_q_solve(wide_factor.r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IQSolve export_wide_solve = factor_exact
        ? al_r20_r63i_q_solve(exported_r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IDSolve strict_solve = factor_exact
        ? al_r20_r63i_d_solve(exported_r_d, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IDSolve{};
    const std::vector<Binary128> strict_solution =
        al_r20_r63g_q_from_double(strict_solve.solution);
    const ALR20R63ISignCertificate wide_baseline =
        wide_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            wide_solve.solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const ALR20R63ISignCertificate export_wide_baseline =
        export_wide_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            export_wide_solve.solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const ALR20R63ISignCertificate strict_baseline =
        strict_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            strict_solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const bool r63i_exact = factor_exact && reference_exact
        && wide_solve.root == AL_R20_R63J_WIDE_SOLVE_SHA256
        && export_wide_solve.root == AL_R20_R63J_EXPORT_WIDE_SOLVE_SHA256
        && strict_solve.root == AL_R20_R63J_STRICT_SOLVE_SHA256
        && wide_baseline.root == AL_R20_R63J_WIDE_CERTIFICATE_SHA256
        && export_wide_baseline.root
            == AL_R20_R63J_EXPORT_WIDE_CERTIFICATE_SHA256
        && strict_baseline.root == AL_R20_R63J_STRICT_CERTIFICATE_SHA256;

    const ALR20R63JDefect defect = r63i_exact
        ? al_r20_r63j_defect(
            factor_capture.matrix, factor_capture.inverse)
        : ALR20R63JDefect{};
    const ALR20R63JLane parent_wide_lane = defect.exact
        ? al_r20_r63j_lane(factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, wide_solve.solution, wide_baseline, defect,
            reference_signs, reference_certificate.solution)
        : ALR20R63JLane{};
    const ALR20R63JLane parent_export_lane = defect.exact
        ? al_r20_r63j_lane(factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, export_wide_solve.solution,
            export_wide_baseline, defect, reference_signs,
            reference_certificate.solution)
        : ALR20R63JLane{};
    const ALR20R63JLane parent_strict_lane = defect.exact
        ? al_r20_r63j_lane(factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, strict_solution, strict_baseline, defect,
            reference_signs, reference_certificate.solution)
        : ALR20R63JLane{};
    const bool parent_r63j_exact = r63i_exact && defect.exact
        && defect.root == AL_R20_R63K_DEFECT_SHA256
        && parent_wide_lane.exact && parent_wide_lane.passed
        && parent_export_lane.exact && parent_export_lane.passed
        && parent_strict_lane.exact && parent_strict_lane.passed
        && parent_wide_lane.root == AL_R20_R63K_WIDE_PARENT_LANE_SHA256
        && parent_export_lane.root == AL_R20_R63K_EXPORT_PARENT_LANE_SHA256
        && parent_strict_lane.root == AL_R20_R63K_STRICT_PARENT_LANE_SHA256;

    const ALR20R63KLane wide_lane = parent_r63j_exact
        ? al_r20_r63k_q_lane(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, wide_solve.solution, wide_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63KLane{};
    const ALR20R63KLane export_wide_lane = parent_r63j_exact
        ? al_r20_r63k_q_lane(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, export_wide_solve.solution,
            export_wide_baseline, reference_signs,
            reference_certificate.solution)
        : ALR20R63KLane{};
    const ALR20R63KLane strict_lane = parent_r63j_exact
        ? al_r20_r63k_d_lane(exported_r_d, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, strict_solve.solution, strict_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63KLane{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    auto completed_work_exact = [](const ALR20R63KLane& lane) {
        if (!lane.finite) return lane.exact
            && lane.failure_iteration != 0U && !lane.failure_stage.empty();
        return lane.completed_iterations == 16U
            && lane.residual_dots == 1632U
            && lane.update_dots == 1632U
            && lane.forward_terms == 82416U
            && lane.backward_terms == 82416U
            && lane.divisions == 3264U
            && lane.certificate_residual_dots == 510U
            && lane.certificate_image_dots == 510U;
    };
    const bool work_exact = completed_work_exact(wide_lane)
        && completed_work_exact(export_wide_lane)
        && completed_work_exact(strict_lane);
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63j_exact && wide_lane.exact && export_wide_lane.exact
        && strict_lane.exact && work_exact && lifecycle_exact;
    const std::string route = al_r20_r63k_classify(apparatus_exact,
        wide_lane.passed, export_wide_lane.passed, strict_lane.passed);
    const bool exact = apparatus_exact;

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63k-factor-standard-refinement.v1|"
        << AL_R20_R63K_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << defect.root << ':'
        << parent_wide_lane.root << ':' << parent_export_lane.root << ':'
        << parent_strict_lane.root << ':' << wide_lane.root << ':'
        << export_wide_lane.root << ':' << strict_lane.root;
    const std::string result_sha = sha256_hex(semantic.str());

    auto lane_json = [](const char* name, const ALR20R63KLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"finite\":" << (lane.finite ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"monotone_error\":"
            << (lane.monotone_error ? "true" : "false")
            << ",\"completed_iterations\":" << lane.completed_iterations
            << ",\"failure_iteration\":" << lane.failure_iteration
            << ",\"failure_stage\":\"" << lane.failure_stage << "\""
            << ",\"first_passing_iteration\":"
            << lane.first_passing_iteration
            << ",\"minimum_error\":\""
            << al_binary128_text(lane.minimum_error)
            << "\",\"checkpoints\":[";
        for (std::size_t index = 0U; index < lane.checkpoints.size(); ++index) {
            if (index != 0U) stream << ',';
            const ALR20R63KCheckpoint& checkpoint = lane.checkpoints[index];
            const ALR20R63ISignCertificate& certificate =
                checkpoint.certificate;
            stream << "{\"iteration\":" << checkpoint.iteration
                << ",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"error\":\""
                << al_binary128_text(certificate.error)
                << "\",\"residual_inf_bound\":\""
                << al_binary128_text(certificate.residual_inf_bound)
                << "\",\"positive\":" << certificate.positive
                << ",\"negative\":" << certificate.negative
                << ",\"unresolved\":" << certificate.unresolved
                << ",\"minimum_separation\":\""
                << al_binary128_text(certificate.minimum_separation)
                << "\",\"maximum_reference_difference\":\""
                << al_binary128_text(
                    certificate.maximum_reference_difference)
                << "\",\"root\":\"" << checkpoint.root << "\"}";
        }
        stream << "],\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63k_factor_standard_refinement.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63j_sha256\":\""
        << AL_R20_R63K_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63j_exact\":"
        << (parent_r63j_exact ? "true" : "false")
        << ",\"lanes\":{" << lane_json("wide", wide_lane) << ','
        << lane_json("export_wide", export_wide_lane) << ','
        << lane_json("strict_binary64", strict_lane) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"factor_refinement_iterations\":"
        << (wide_lane.completed_iterations
            + export_wide_lane.completed_iterations
            + strict_lane.completed_iterations)
        << ",\"dense_inverse_corrections\":0"
        << ",\"checkpoint_certificates\":"
        << (wide_lane.checkpoints.size() + export_wide_lane.checkpoints.size()
            + strict_lane.checkpoints.size())
        << ",\"gmres_solves\":0,\"pcg_solves\":0,\"minres_solves\":0"
        << ",\"new_inverse_constructions\":0,\"new_factorizations\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"regularizations\":0"
        << ",\"replacement_applies\":0,\"following_transitions\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
