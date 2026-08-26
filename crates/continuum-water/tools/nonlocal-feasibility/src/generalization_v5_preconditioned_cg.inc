namespace {

constexpr const char* AL_R20_R63M_PARENT_SEMANTIC_SHA256 =
    "16633e078acd397d817ecfc0ec5639b41135258209fc355fea3eb9c58c557333";
constexpr const char* AL_R20_R63M_WIDE_FRONTIER_SHA256 =
    "ca308376d80d1df349e03b4b58e5f681d0692e56f621426c2ccf4ad7d9a34136";
constexpr const char* AL_R20_R63M_EXPORT_FRONTIER_SHA256 =
    "16ac04bff852fe13a7c974d9bcb9be1a774bccb12f250f4c55e86eb2c2582f63";

struct ALR20R63MProduct {
    bool exact = false;
    bool no_underflow = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::vector<Binary128> value;
    std::vector<Binary128> bound;
    std::string root;
};

ALR20R63MProduct al_r20_r63m_product(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& vector) {
    ALR20R63MProduct result;
    result.dimension = vector.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || matrix.size() != dimension * dimension)
        return result;
    result.value.resize(dimension);
    result.bound.resize(dimension);
    bool exact = true;
    bool no_underflow = true;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension);
        for (std::size_t column = 0U; column < dimension; ++column)
            left[column] = matrix[row * dimension + column];
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, vector);
        result.value[row] = dot.value;
        result.bound[row] = dot.bound;
        exact = exact && dot.exact && finiteq(dot.value) != 0
            && finiteq(dot.bound) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        ++result.dots;
    }
    result.no_underflow = no_underflow;
    result.exact = exact && no_underflow && result.dots == dimension;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + std::to_string(result.dimension) + ':'
        + std::to_string(result.dots) + ':'
        + al_r20_q_scalar_root(result.value) + ':'
        + al_r20_q_scalar_root(result.bound));
    return result;
}

struct ALR20R63MScalar {
    bool exact = false;
    bool no_underflow = false;
    bool positive = false;
    Binary128 value = static_cast<Binary128>(0.0);
    Binary128 bound = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R63MScalar al_r20_r63m_scalar(
    const std::vector<Binary128>& left,
    const std::vector<Binary128>& right) {
    ALR20R63MScalar result;
    const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
    result.value = dot.value;
    result.bound = dot.bound;
    result.no_underflow = dot.no_underflow;
    result.positive = dot.value - dot.bound
        > static_cast<Binary128>(0.0);
    result.exact = dot.exact && dot.no_underflow
        && finiteq(dot.value) != 0 && finiteq(dot.bound) != 0;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + std::to_string(result.positive) + ':' + dot.root);
    return result;
}

struct ALR20R63MStep {
    bool exact = false;
    std::size_t iteration = 0U;
    Binary128 rho = static_cast<Binary128>(0.0);
    Binary128 denominator = static_cast<Binary128>(0.0);
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 beta = static_cast<Binary128>(0.0);
    Binary128 residual_drift = static_cast<Binary128>(0.0);
    ALR20R63ISignCertificate certificate;
    std::string root;
};

struct ALR20R63MLane {
    bool exact = false;
    bool finite = false;
    bool passed = false;
    std::size_t dimension = 0U;
    std::size_t completed_iterations = 0U;
    std::size_t failure_iteration = 0U;
    std::size_t preconditioner_solves = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::size_t initial_residual_dots = 0U;
    std::size_t product_dots = 0U;
    std::size_t rho_dots = 0U;
    std::size_t denominator_dots = 0U;
    std::size_t solution_update_dots = 0U;
    std::size_t residual_update_dots = 0U;
    std::size_t direction_update_dots = 0U;
    std::size_t certificate_residual_dots = 0U;
    std::size_t certificate_image_dots = 0U;
    std::size_t first_passing_iteration = 0U;
    Binary128 minimum_error = HUGE_VALQ;
    Binary128 maximum_residual_drift = static_cast<Binary128>(0.0);
    std::vector<Binary128> solution;
    std::vector<Binary128> residual;
    std::vector<Binary128> direction;
    std::vector<ALR20R63MStep> steps;
    std::string failure_stage;
    std::string root;
};

ALR20R63MLane al_r20_r63m_lane(
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
    ALR20R63MLane result;
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
    const ALR20R63KResidual initial_residual = al_r20_r63k_residual(
        matrix, rhs, result.solution);
    result.initial_residual_dots = initial_residual.dots;
    if (!initial_residual.exact) return result;
    result.residual = initial_residual.value;
    ALR20R63IQSolve preconditioned = al_r20_r63i_q_solve(
        upper, permutation, result.residual, inverse_scale);
    result.forward_terms += preconditioned.forward_terms;
    result.backward_terms += preconditioned.backward_terms;
    result.divisions += preconditioned.divisions;
    ++result.preconditioner_solves;
    if (!preconditioned.exact) {
        result.exact = true;
        result.finite = false;
        result.failure_iteration = 0U;
        result.failure_stage = "initial_preconditioner";
    } else {
        result.direction = preconditioned.solution;
    }
    ALR20R63MScalar rho;
    if (result.failure_stage.empty()) {
        rho = al_r20_r63m_scalar(result.residual, preconditioned.solution);
        ++result.rho_dots;
        if (!rho.exact || !rho.positive) {
            result.exact = true;
            result.finite = false;
            result.failure_iteration = 0U;
            result.failure_stage = "initial_rho";
        }
    }
    bool exact = result.failure_stage.empty();
    bool finite = result.failure_stage.empty();
    for (std::size_t iteration = 1U;
         iteration <= 8U && finite; ++iteration) {
        ALR20R63MStep step;
        step.iteration = iteration;
        step.rho = rho.value;
        const ALR20R63MProduct product = al_r20_r63m_product(
            matrix, result.direction);
        result.product_dots += product.dots;
        if (!product.exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "matrix_product";
            break;
        }
        const ALR20R63MScalar denominator = al_r20_r63m_scalar(
            result.direction, product.value);
        ++result.denominator_dots;
        step.denominator = denominator.value;
        if (!denominator.exact || !denominator.positive) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "nonpositive_denominator";
            break;
        }
        step.alpha = rho.value / denominator.value;
        if (finiteq(step.alpha) == 0) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "alpha";
            break;
        }
        std::vector<Binary128> updated_solution(dimension);
        std::vector<Binary128> updated_residual(dimension);
        std::ostringstream update_material;
        bool update_exact = true;
        for (std::size_t row = 0U; row < dimension; ++row) {
            const ALR20R38Dot2 x_dot = al_r20_r38_dot2(
                {result.solution[row], step.alpha},
                {static_cast<Binary128>(1.0), result.direction[row]});
            const ALR20R38Dot2 r_dot = al_r20_r38_dot2(
                {result.residual[row], -step.alpha},
                {static_cast<Binary128>(1.0), product.value[row]});
            updated_solution[row] = x_dot.value;
            updated_residual[row] = r_dot.value;
            update_exact = update_exact && x_dot.exact && r_dot.exact
                && x_dot.no_underflow && r_dot.no_underflow
                && finiteq(x_dot.value) != 0 && finiteq(r_dot.value) != 0;
            update_material << x_dot.root << ':' << r_dot.root << ';';
            ++result.solution_update_dots;
            ++result.residual_update_dots;
        }
        if (!update_exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "update";
            break;
        }
        result.solution = std::move(updated_solution);
        result.residual = std::move(updated_residual);
        ++result.completed_iterations;
        step.certificate = al_r20_r63i_sign_certificate(
            matrix, inverse, rhs, result.solution, reference_signs,
            baseline.rho_bound, reference_solution);
        result.certificate_residual_dots += step.certificate.residual_dots;
        result.certificate_image_dots += step.certificate.image_dots;
        if (!step.certificate.exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "certificate";
            break;
        }
        for (std::size_t row = 0U; row < dimension; ++row)
            step.residual_drift = fmaxq(step.residual_drift,
                al_r20_r38_up_add(
                    fabsq(result.residual[row]
                        - step.certificate.residual[row]),
                    step.certificate.residual_bound[row]));
        result.maximum_residual_drift = fmaxq(
            result.maximum_residual_drift, step.residual_drift);
        result.minimum_error = fminq(
            result.minimum_error, step.certificate.error);
        if (step.certificate.passed
            && result.first_passing_iteration == 0U)
            result.first_passing_iteration = iteration;
        std::string preconditioner_root;
        std::string rho_new_root;
        std::string direction_root;
        if (iteration < 8U) {
            preconditioned = al_r20_r63i_q_solve(
                upper, permutation, result.residual, inverse_scale);
            result.forward_terms += preconditioned.forward_terms;
            result.backward_terms += preconditioned.backward_terms;
            result.divisions += preconditioned.divisions;
            ++result.preconditioner_solves;
            if (!preconditioned.exact) {
                finite = false;
                result.failure_iteration = iteration;
                result.failure_stage = "preconditioner";
                break;
            }
            const ALR20R63MScalar rho_new = al_r20_r63m_scalar(
                result.residual, preconditioned.solution);
            ++result.rho_dots;
            if (!rho_new.exact || !rho_new.positive) {
                finite = false;
                result.failure_iteration = iteration;
                result.failure_stage = "rho";
                break;
            }
            step.beta = rho_new.value / rho.value;
            if (finiteq(step.beta) == 0) {
                finite = false;
                result.failure_iteration = iteration;
                result.failure_stage = "beta";
                break;
            }
            std::vector<Binary128> updated_direction(dimension);
            bool direction_exact = true;
            std::ostringstream direction_material;
            for (std::size_t row = 0U; row < dimension; ++row) {
                const ALR20R38Dot2 dot = al_r20_r38_dot2(
                    {preconditioned.solution[row], step.beta},
                    {static_cast<Binary128>(1.0), result.direction[row]});
                updated_direction[row] = dot.value;
                direction_exact = direction_exact && dot.exact
                    && dot.no_underflow && finiteq(dot.value) != 0;
                direction_material << dot.root << ';';
                ++result.direction_update_dots;
            }
            if (!direction_exact) {
                finite = false;
                result.failure_iteration = iteration;
                result.failure_stage = "direction";
                break;
            }
            result.direction = std::move(updated_direction);
            preconditioner_root = preconditioned.root;
            rho_new_root = rho_new.root;
            direction_root = sha256_hex(direction_material.str());
            rho = rho_new;
        }
        step.exact = finite;
        step.root = sha256_hex(std::to_string(step.exact) + ':'
            + std::to_string(step.iteration) + ':'
            + al_binary128_hex(step.rho) + ':'
            + al_binary128_hex(step.denominator) + ':'
            + al_binary128_hex(step.alpha) + ':'
            + al_binary128_hex(step.beta) + ':'
            + al_binary128_hex(step.residual_drift) + ':'
            + product.root + ':' + denominator.root + ':'
            + sha256_hex(update_material.str()) + ':'
            + step.certificate.root + ':' + preconditioner_root + ':'
            + rho_new_root + ':' + direction_root);
        exact = exact && step.exact;
        result.steps.push_back(std::move(step));
    }
    result.finite = finite;
    const bool complete = result.completed_iterations == 8U;
    result.passed = finite && complete
        && result.first_passing_iteration != 0U;
    result.exact = result.exact || (exact && ((finite && complete
            && result.steps.size() == 8U)
        || (!finite && !result.failure_stage.empty())));
    std::ostringstream material;
    material << result.exact << ':' << result.finite << ':'
        << result.passed << ':' << result.dimension << ':'
        << result.completed_iterations << ':' << result.failure_iteration
        << ':' << result.failure_stage << ':' << result.preconditioner_solves
        << ':' << result.forward_terms << ':' << result.backward_terms << ':'
        << result.divisions << ':' << result.initial_residual_dots << ':'
        << result.product_dots << ':' << result.rho_dots << ':'
        << result.denominator_dots << ':' << result.solution_update_dots << ':'
        << result.residual_update_dots << ':' << result.direction_update_dots
        << ':' << result.certificate_residual_dots << ':'
        << result.certificate_image_dots << ':'
        << result.first_passing_iteration << ':'
        << al_binary128_hex(result.minimum_error) << ':'
        << al_binary128_hex(result.maximum_residual_drift) << ':'
        << al_r20_q_scalar_root(result.solution) << ':'
        << al_r20_q_scalar_root(result.residual) << ':'
        << al_r20_q_scalar_root(result.direction) << '|';
    for (const ALR20R63MStep& step : result.steps)
        material << step.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63m_classify(
    bool apparatus, bool wide, bool exported) {
    if (!apparatus) return "PRECONDITIONED_CG_APPARATUS_REJECTED";
    if (!wide) return "RETAINED_WIDE_PCG_REJECTED";
    if (!exported) return "EXPORTED_FACTOR_WIDE_PCG_REJECTED";
    return "EXPORTED_FACTOR_WIDE_PCG_CANDIDATE";
}

struct ALR20R63MControls {
    bool exact = false;
    bool scalar = false;
    bool breakdown = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63MControls al_r20_r63m_controls() {
    ALR20R63MControls result;
    constexpr std::size_t dimension = 8U;
    std::vector<Binary128> matrix(dimension * dimension,
        static_cast<Binary128>(0.0));
    std::vector<Binary128> inverse(dimension * dimension,
        static_cast<Binary128>(0.0));
    std::vector<Binary128> rhs(dimension, static_cast<Binary128>(1.0));
    std::vector<Binary128> initial(dimension, static_cast<Binary128>(-3.0));
    std::vector<Binary128> reference(dimension);
    std::vector<int> signs(dimension, 1);
    std::vector<std::size_t> permutation(dimension);
    std::vector<Binary128> upper(dimension * dimension,
        static_cast<Binary128>(0.0));
    const Binary128 upper_diagonal = sqrtq(
        static_cast<Binary128>(4.0) / static_cast<Binary128>(3.0));
    for (std::size_t index = 0U; index < dimension; ++index) {
        const Binary128 diagonal = static_cast<Binary128>(index + 1U);
        matrix[index * dimension + index] = diagonal;
        inverse[index * dimension + index] =
            static_cast<Binary128>(1.0) / diagonal;
        reference[index] = static_cast<Binary128>(1.0) / diagonal;
        permutation[index] = index;
        upper[index * dimension + index] = upper_diagonal;
    }
    const ALR20R63JDefect defect = al_r20_r63j_defect(matrix, inverse);
    const ALR20R63ISignCertificate baseline =
        al_r20_r63i_sign_certificate(matrix, inverse, rhs, initial, signs,
            defect.rho_bound, reference);
    const ALR20R63MLane lane = al_r20_r63m_lane(
        upper, permutation, static_cast<Binary128>(1.0), matrix, inverse,
        rhs, initial, baseline, signs, reference);
    result.scalar = lane.exact && lane.finite && lane.passed
        && lane.completed_iterations == 8U && lane.steps.size() == 8U;
    const std::vector<Binary128> scalar_matrix{
        static_cast<Binary128>(-1.0)};
    const std::vector<Binary128> scalar_inverse{
        static_cast<Binary128>(0.75)};
    const std::vector<Binary128> scalar_rhs{
        static_cast<Binary128>(1.0)};
    const std::vector<Binary128> scalar_initial{
        static_cast<Binary128>(-3.0)};
    const std::vector<Binary128> scalar_reference{
        static_cast<Binary128>(1.0)};
    const std::vector<int> scalar_signs{1};
    const std::vector<std::size_t> scalar_permutation{0U};
    const std::vector<Binary128> scalar_upper{upper_diagonal};
    const ALR20R63ISignCertificate scalar_baseline =
        al_r20_r63i_sign_certificate({static_cast<Binary128>(1.0)},
            scalar_inverse, scalar_rhs, scalar_initial, scalar_signs,
            static_cast<Binary128>(0.25), scalar_reference);
    const ALR20R63MLane breakdown = al_r20_r63m_lane(
        scalar_upper, scalar_permutation, static_cast<Binary128>(1.0),
        scalar_matrix, scalar_inverse, scalar_rhs, scalar_initial,
        scalar_baseline, scalar_signs, scalar_reference);
    result.breakdown = breakdown.exact && !breakdown.finite
        && !breakdown.passed && breakdown.failure_iteration == 1U
        && breakdown.failure_stage == "nonpositive_denominator";
    std::vector<Binary128> mutated = lane.solution;
    mutated[0U] = nextafterq(mutated[0U], HUGE_VALQ);
    result.mutation = al_r20_q_scalar_root(mutated)
        != al_r20_q_scalar_root(lane.solution);
    result.classifier =
        al_r20_r63m_classify(false, true, true)
            == "PRECONDITIONED_CG_APPARATUS_REJECTED"
        && al_r20_r63m_classify(true, false, true)
            == "RETAINED_WIDE_PCG_REJECTED"
        && al_r20_r63m_classify(true, true, false)
            == "EXPORTED_FACTOR_WIDE_PCG_REJECTED"
        && al_r20_r63m_classify(true, true, true)
            == "EXPORTED_FACTOR_WIDE_PCG_CANDIDATE";
    result.exact = result.scalar && result.breakdown
        && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + lane.root + ':' + breakdown.root + ':'
        + al_r20_q_scalar_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_preconditioned_cg_controls() {
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
    const ALR20R63MControls controls = al_r20_r63m_controls();
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
    const ALR20R63ISignCertificate export_baseline =
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
        && export_baseline.root == AL_R20_R63J_EXPORT_WIDE_CERTIFICATE_SHA256
        && strict_baseline.root == AL_R20_R63J_STRICT_CERTIFICATE_SHA256;
    const ALR20R63KLane parent_wide = r63i_exact
        ? al_r20_r63k_q_lane(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, wide_solve.solution, wide_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63KLane{};
    const ALR20R63KLane parent_export = r63i_exact
        ? al_r20_r63k_q_lane(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, export_wide_solve.solution, export_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63KLane{};
    const ALR20R63KLane parent_strict = r63i_exact
        ? al_r20_r63k_d_lane(exported_r_d, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, strict_solve.solution, strict_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63KLane{};
    const bool r63k_exact = r63i_exact
        && parent_wide.root == AL_R20_R63L_WIDE_PARENT_SHA256
        && parent_export.root == AL_R20_R63L_EXPORT_PARENT_SHA256
        && parent_strict.root == AL_R20_R63L_STRICT_PARENT_SHA256;
    const ALR20R63LContinuation parent_wide_frontier = r63k_exact
        ? al_r20_r63l_continue(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, parent_wide, reference_signs,
            reference_certificate.solution)
        : ALR20R63LContinuation{};
    const ALR20R63LContinuation parent_export_frontier = r63k_exact
        ? al_r20_r63l_continue(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, parent_export, reference_signs,
            reference_certificate.solution)
        : ALR20R63LContinuation{};
    const bool parent_r63l_exact = r63k_exact
        && parent_wide_frontier.exact && parent_wide_frontier.passed
        && parent_export_frontier.exact && parent_export_frontier.passed
        && parent_wide_frontier.root == AL_R20_R63M_WIDE_FRONTIER_SHA256
        && parent_export_frontier.root == AL_R20_R63M_EXPORT_FRONTIER_SHA256;

    const ALR20R63MLane wide_pcg = parent_r63l_exact
        ? al_r20_r63m_lane(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, wide_solve.solution, wide_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63MLane{};
    const ALR20R63MLane export_pcg = parent_r63l_exact
        ? al_r20_r63m_lane(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, export_wide_solve.solution, export_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63MLane{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    auto pcg_work_exact = [](const ALR20R63MLane& lane) {
        if (!lane.finite) return lane.exact && !lane.failure_stage.empty();
        return lane.completed_iterations == 8U
            && lane.preconditioner_solves == 8U
            && lane.forward_terms == 41208U
            && lane.backward_terms == 41208U && lane.divisions == 1632U
            && lane.initial_residual_dots == 102U
            && lane.product_dots == 816U && lane.rho_dots == 8U
            && lane.denominator_dots == 8U
            && lane.solution_update_dots == 816U
            && lane.residual_update_dots == 816U
            && lane.direction_update_dots == 714U
            && lane.certificate_residual_dots == 816U
            && lane.certificate_image_dots == 816U;
    };
    const bool work_exact = pcg_work_exact(wide_pcg)
        && pcg_work_exact(export_pcg);
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63l_exact && wide_pcg.exact && export_pcg.exact
        && work_exact && lifecycle_exact;
    const std::string route = al_r20_r63m_classify(
        apparatus_exact, wide_pcg.passed, export_pcg.passed);
    const bool exact = apparatus_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r63m-preconditioned-cg.v1|"
        << AL_R20_R63M_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << parent_wide_frontier.root << ':'
        << parent_export_frontier.root << ':' << wide_pcg.root << ':'
        << export_pcg.root;
    const std::string result_sha = sha256_hex(semantic.str());
    auto lane_json = [](const char* name, const ALR20R63MLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"finite\":" << (lane.finite ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"completed_iterations\":" << lane.completed_iterations
            << ",\"failure_iteration\":" << lane.failure_iteration
            << ",\"failure_stage\":\"" << lane.failure_stage << "\""
            << ",\"first_passing_iteration\":"
            << lane.first_passing_iteration
            << ",\"minimum_error\":\""
            << al_binary128_text(lane.minimum_error)
            << "\",\"maximum_residual_drift\":\""
            << al_binary128_text(lane.maximum_residual_drift)
            << "\",\"steps\":[";
        for (std::size_t index = 0U; index < lane.steps.size(); ++index) {
            if (index != 0U) stream << ',';
            const ALR20R63MStep& step = lane.steps[index];
            stream << "{\"iteration\":" << step.iteration
                << ",\"passed\":"
                << (step.certificate.passed ? "true" : "false")
                << ",\"error\":\""
                << al_binary128_text(step.certificate.error)
                << "\",\"positive\":" << step.certificate.positive
                << ",\"negative\":" << step.certificate.negative
                << ",\"unresolved\":" << step.certificate.unresolved
                << ",\"residual_drift\":\""
                << al_binary128_text(step.residual_drift)
                << "\",\"root\":\"" << step.root << "\"}";
        }
        stream << "],\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63m_preconditioned_cg.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63l_sha256\":\""
        << AL_R20_R63M_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63l_exact\":"
        << (parent_r63l_exact ? "true" : "false")
        << ",\"lanes\":{" << lane_json("wide", wide_pcg) << ','
        << lane_json("export_wide", export_pcg) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"pcg_factor_solves\":"
        << (wide_pcg.preconditioner_solves + export_pcg.preconditioner_solves)
        << ",\"pcg_algorithm_h_applications\":"
        << (2U + wide_pcg.completed_iterations
            + export_pcg.completed_iterations)
        << ",\"stationary_baseline_factor_solves\":38"
        << ",\"strict_binary64_work\":0,\"dense_inverse_corrections\":0"
        << ",\"gmres_solves\":0,\"minres_solves\":0"
        << ",\"new_inverse_constructions\":0,\"new_factorizations\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"regularizations\":0"
        << ",\"replacement_applies\":0,\"following_transitions\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
