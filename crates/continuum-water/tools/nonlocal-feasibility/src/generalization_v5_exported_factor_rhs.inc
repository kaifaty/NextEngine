namespace {

constexpr const char* AL_R20_R63I_PARENT_SEMANTIC_SHA256 =
    "34452faf4640780badf441f0f3d22e12d0e5797194e568cda22c1c0a79156708";
constexpr const char* AL_R20_R63I_INVERSE_SHA256 =
    "467e815a4813e7774523699147db38fbcda06dea9b93bfb1696b410e0df0cfd7";
constexpr const char* AL_R20_R63I_RHS_SHA256 =
    "64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1";
constexpr const char* AL_R20_R63I_SOLUTION_SHA256 =
    "2c3a68b39bc82c4de5c09d9f4d4a091cbfd41024b0cdc3a8e509da7effcfe6e3";
constexpr const char* AL_R20_R63I_WIDE_FACTOR_SHA256 =
    "bf0a9e1b9e2fd4525d267ec63d63640675479ab55c5b6cd3bb3b0c4ea4779b14";
constexpr const char* AL_R20_R63I_EXPORT_R_SHA256 =
    "a7a85789364f5af91aaf71e1531759713c95ec085e48b7ca614daa524c3a4f85";

struct ALR20R63IQSolve {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::vector<Binary128> scaled_rhs;
    std::vector<Binary128> intermediate;
    std::vector<Binary128> permuted_solution;
    std::vector<Binary128> solution;
    std::string root;
};

ALR20R63IQSolve al_r20_r63i_q_solve(
    const std::vector<Binary128>& upper,
    const std::vector<std::size_t>& permutation,
    const std::vector<Binary128>& rhs, Binary128 inverse_scale) {
    ALR20R63IQSolve result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || !(inverse_scale > static_cast<Binary128>(0.0))) return result;
    result.scaled_rhs.resize(dimension);
    result.intermediate.resize(dimension);
    result.permuted_solution.resize(dimension);
    result.solution.resize(dimension);
    bool finite = finiteq(inverse_scale) != 0;
    for (std::size_t row = 0U; row < dimension; ++row) {
        result.scaled_rhs[row] = rhs[permutation[row]] * inverse_scale;
        finite = finite && finiteq(result.scaled_rhs[row]) != 0;
        Binary128Accumulator dot;
        for (std::size_t column = 0U; column < row; ++column) {
            dot.add(upper[column * dimension + row]
                * result.intermediate[column]);
            ++result.forward_terms;
        }
        const Binary128 diagonal = upper[row * dimension + row];
        finite = finite && finiteq(diagonal) != 0
            && diagonal != static_cast<Binary128>(0.0);
        if (!finite) break;
        result.intermediate[row] =
            (result.scaled_rhs[row] - dot.value()) / diagonal;
        finite = finite && finiteq(result.intermediate[row]) != 0;
        ++result.divisions;
    }
    if (finite) {
        for (std::size_t reverse = dimension; reverse-- > 0U;) {
            Binary128Accumulator dot;
            for (std::size_t column = reverse + 1U;
                 column < dimension; ++column) {
                dot.add(upper[reverse * dimension + column]
                    * result.permuted_solution[column]);
                ++result.backward_terms;
            }
            const Binary128 diagonal = upper[reverse * dimension + reverse];
            finite = finite && finiteq(diagonal) != 0
                && diagonal != static_cast<Binary128>(0.0);
            if (!finite) break;
            result.permuted_solution[reverse] =
                (result.intermediate[reverse] - dot.value()) / diagonal;
            finite = finite
                && finiteq(result.permuted_solution[reverse]) != 0;
            ++result.divisions;
        }
    }
    for (std::size_t index = 0U; index < dimension; ++index)
        result.solution[permutation[index]] =
            result.permuted_solution[index];
    result.exact = finite
        && result.forward_terms == dimension * (dimension - 1U) / 2U
        && result.backward_terms == dimension * (dimension - 1U) / 2U
        && result.divisions == 2U * dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.dimension << ':'
        << result.forward_terms << ':' << result.backward_terms << ':'
        << result.divisions << ':'
        << al_r20_q_scalar_root(result.scaled_rhs) << ':'
        << al_r20_q_scalar_root(result.intermediate) << ':'
        << al_r20_q_scalar_root(result.permuted_solution) << ':'
        << al_r20_q_scalar_root(result.solution);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63IDSolve {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::vector<double> scaled_rhs;
    std::vector<double> intermediate;
    std::vector<double> permuted_solution;
    std::vector<double> solution;
    std::string root;
};

ALR20R63IDSolve al_r20_r63i_d_solve(
    const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation,
    const std::vector<Binary128>& rhs, Binary128 inverse_scale) {
    ALR20R63IDSolve result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension))
        return result;
    const double scale = static_cast<double>(inverse_scale);
    result.scaled_rhs.resize(dimension);
    result.intermediate.resize(dimension);
    result.permuted_solution.resize(dimension);
    result.solution.resize(dimension);
    bool finite = std::isfinite(scale) && scale > 0.0;
    for (std::size_t row = 0U; row < dimension; ++row) {
        result.scaled_rhs[row] =
            static_cast<double>(rhs[permutation[row]]) * scale;
        finite = finite && std::isfinite(result.scaled_rhs[row]);
        double dot = 0.0;
        for (std::size_t column = 0U; column < row; ++column) {
            dot += upper[column * dimension + row]
                * result.intermediate[column];
            ++result.forward_terms;
        }
        const double diagonal = upper[row * dimension + row];
        finite = finite && std::isfinite(diagonal) && diagonal != 0.0;
        if (!finite) break;
        result.intermediate[row] =
            (result.scaled_rhs[row] - dot) / diagonal;
        finite = finite && std::isfinite(result.intermediate[row]);
        ++result.divisions;
    }
    if (finite) {
        for (std::size_t reverse = dimension; reverse-- > 0U;) {
            double dot = 0.0;
            for (std::size_t column = reverse + 1U;
                 column < dimension; ++column) {
                dot += upper[reverse * dimension + column]
                    * result.permuted_solution[column];
                ++result.backward_terms;
            }
            const double diagonal = upper[reverse * dimension + reverse];
            finite = finite
                && std::isfinite(diagonal) && diagonal != 0.0;
            if (!finite) break;
            result.permuted_solution[reverse] =
                (result.intermediate[reverse] - dot) / diagonal;
            finite = finite
                && std::isfinite(result.permuted_solution[reverse]);
            ++result.divisions;
        }
    }
    for (std::size_t index = 0U; index < dimension; ++index)
        result.solution[permutation[index]] =
            result.permuted_solution[index];
    result.exact = finite
        && result.forward_terms == dimension * (dimension - 1U) / 2U
        && result.backward_terms == dimension * (dimension - 1U) / 2U
        && result.divisions == 2U * dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.dimension << ':'
        << result.forward_terms << ':' << result.backward_terms << ':'
        << result.divisions << ':'
        << al_r20_double_root(result.scaled_rhs) << ':'
        << al_r20_double_root(result.intermediate) << ':'
        << al_r20_double_root(result.permuted_solution) << ':'
        << al_r20_double_root(result.solution);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ISignCertificate {
    bool exact = false;
    bool passed = false;
    bool no_underflow = false;
    std::size_t dimension = 0U;
    std::size_t residual_dots = 0U;
    std::size_t image_dots = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    Binary128 rho_bound = static_cast<Binary128>(0.0);
    Binary128 residual_inf_bound = static_cast<Binary128>(0.0);
    Binary128 image_inf_bound = static_cast<Binary128>(0.0);
    Binary128 error = static_cast<Binary128>(0.0);
    Binary128 minimum_separation = HUGE_VALQ;
    Binary128 maximum_reference_difference = static_cast<Binary128>(0.0);
    std::vector<Binary128> residual;
    std::vector<Binary128> residual_bound;
    std::vector<Binary128> image;
    std::vector<Binary128> image_bound;
    std::vector<int> signs;
    std::string root;
};

ALR20R63ISignCertificate al_r20_r63i_sign_certificate(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& candidate,
    const std::vector<int>& reference_signs, Binary128 rho_bound,
    const std::vector<Binary128>& reference_solution) {
    ALR20R63ISignCertificate result;
    result.dimension = rhs.size();
    result.rho_bound = rho_bound;
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension
        || candidate.size() != dimension
        || reference_signs.size() != dimension
        || reference_solution.size() != dimension
        || !(rho_bound < static_cast<Binary128>(1.0))) return result;
    result.residual.resize(dimension);
    result.residual_bound.resize(dimension);
    result.image.resize(dimension);
    result.image_bound.resize(dimension);
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
        result.residual[row] = dot.value;
        result.residual_bound[row] = dot.bound;
        exact = exact && dot.exact
            && finiteq(dot.value) != 0 && finiteq(dot.bound) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        result.residual_inf_bound = fmaxq(result.residual_inf_bound,
            al_r20_r38_up_add(fabsq(dot.value), dot.bound));
        ++result.residual_dots;
    }
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension);
        std::vector<Binary128> right(dimension);
        for (std::size_t column = 0U; column < dimension; ++column) {
            left[column] = inverse[row * dimension + column];
            right[column] = result.residual[column];
        }
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
        Binary128 propagated = dot.bound;
        for (std::size_t column = 0U; column < dimension; ++column)
            propagated = al_r20_r38_up_add(propagated,
                al_r20_r38_up_multiply(
                    fabsq(inverse[row * dimension + column]),
                    result.residual_bound[column]));
        result.image[row] = dot.value;
        result.image_bound[row] = propagated;
        exact = exact && dot.exact && finiteq(dot.value) != 0
            && finiteq(propagated) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        result.image_inf_bound = fmaxq(result.image_inf_bound,
            al_r20_r38_up_add(fabsq(dot.value), propagated));
        ++result.image_dots;
    }
    const Binary128 denominator = nextafterq(
        static_cast<Binary128>(1.0) - rho_bound,
        static_cast<Binary128>(0.0));
    exact = exact && denominator > static_cast<Binary128>(0.0);
    if (exact)
        result.error = nextafterq(result.image_inf_bound / denominator,
            HUGE_VALQ);
    result.signs.assign(dimension, 0);
    bool sign_match = true;
    for (std::size_t index = 0U; index < dimension; ++index) {
        const Binary128 lower = candidate[index] - result.error;
        const Binary128 upper = candidate[index] + result.error;
        if (lower > static_cast<Binary128>(0.0)) {
            result.signs[index] = 1;
            ++result.positive;
            result.minimum_separation = fminq(
                result.minimum_separation, lower);
        } else if (upper < static_cast<Binary128>(0.0)) {
            result.signs[index] = -1;
            ++result.negative;
            result.minimum_separation = fminq(
                result.minimum_separation, -upper);
        } else {
            ++result.unresolved;
            result.minimum_separation = static_cast<Binary128>(0.0);
        }
        sign_match = sign_match
            && result.signs[index] == reference_signs[index];
        result.maximum_reference_difference = fmaxq(
            result.maximum_reference_difference,
            fabsq(candidate[index] - reference_solution[index]));
    }
    result.no_underflow = no_underflow;
    result.passed = exact && no_underflow && sign_match
        && result.unresolved == 0U;
    result.exact = exact
        && result.residual_dots == dimension
        && result.image_dots == dimension
        && finiteq(result.error) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':'
        << result.no_underflow << ':' << result.dimension << ':'
        << result.residual_dots << ':' << result.image_dots << ':'
        << result.positive << ':' << result.negative << ':'
        << result.unresolved << ':' << al_binary128_hex(result.rho_bound)
        << ':' << al_binary128_hex(result.residual_inf_bound) << ':'
        << al_binary128_hex(result.image_inf_bound) << ':'
        << al_binary128_hex(result.error) << ':'
        << al_binary128_hex(result.minimum_separation) << ':'
        << al_binary128_hex(result.maximum_reference_difference) << ':'
        << al_r20_q_scalar_root(result.residual) << ':'
        << al_r20_q_scalar_root(result.residual_bound) << ':'
        << al_r20_q_scalar_root(result.image) << ':'
        << al_r20_q_scalar_root(result.image_bound) << '|';
    for (int sign : result.signs) material << sign << ',';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63i_classify(bool apparatus, bool wide,
    bool export_wide, bool strict_binary64) {
    if (!apparatus) return "EXPORTED_FACTOR_RHS_APPARATUS_REJECTED";
    if (!wide) return "WIDE_RHS_SIGN_CERTIFICATE_REJECTED";
    if (!export_wide) return "EXPORTED_FACTOR_WIDE_CONSUMPTION_REJECTED";
    if (!strict_binary64)
        return "EXPORTED_FACTOR_BINARY64_CONSUMPTION_REJECTED";
    return "EXPORTED_FACTOR_BINARY64_RHS_CANDIDATE";
}

struct ALR20R63IControls {
    bool exact = false;
    bool solve = false;
    bool permutation = false;
    bool zero_diagonal = false;
    bool certificate = false;
    bool noncontractive = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63IControls al_r20_r63i_controls() {
    ALR20R63IControls result;
    const std::vector<Binary128> upper{
        static_cast<Binary128>(2.0), static_cast<Binary128>(0.5),
        static_cast<Binary128>(0.0), static_cast<Binary128>(1.0)};
    const std::vector<double> upper_d{2.0, 0.5, 0.0, 1.0};
    const std::vector<std::size_t> permutation{1U, 0U};
    const std::vector<Binary128> rhs{
        static_cast<Binary128>(3.0), static_cast<Binary128>(2.0)};
    const ALR20R63IQSolve q = al_r20_r63i_q_solve(
        upper, permutation, rhs, static_cast<Binary128>(1.0));
    const ALR20R63IDSolve d = al_r20_r63i_d_solve(
        upper_d, permutation, rhs, static_cast<Binary128>(1.0));
    result.solve = q.exact && d.exact;
    result.permutation = result.solve
        && q.solution.size() == d.solution.size()
        && std::equal(q.solution.begin(), q.solution.end(),
            d.solution.begin(), [](Binary128 left, double right) {
                return left == static_cast<Binary128>(right);
            });
    std::vector<Binary128> zero = upper;
    zero[3U] = static_cast<Binary128>(0.0);
    result.zero_diagonal = !al_r20_r63i_q_solve(
        zero, permutation, rhs, static_cast<Binary128>(1.0)).exact;

    const std::vector<Binary128> matrix{static_cast<Binary128>(2.0)};
    const std::vector<Binary128> inverse{static_cast<Binary128>(0.5)};
    const std::vector<Binary128> one_rhs{static_cast<Binary128>(2.0)};
    const std::vector<Binary128> one_solution{static_cast<Binary128>(1.0)};
    const std::vector<int> signs{1};
    const ALR20R63ISignCertificate certificate =
        al_r20_r63i_sign_certificate(matrix, inverse, one_rhs,
            one_solution, signs, static_cast<Binary128>(0.0), one_solution);
    result.certificate = certificate.exact && certificate.passed
        && certificate.positive == 1U && certificate.error >= 0.0;
    const ALR20R63ISignCertificate noncontractive =
        al_r20_r63i_sign_certificate(matrix,
            {static_cast<Binary128>(0.0)}, one_rhs, one_solution, signs,
            static_cast<Binary128>(1.0), one_solution);
    result.noncontractive = !noncontractive.exact
        && !noncontractive.passed;
    std::vector<Binary128> mutated = q.solution;
    mutated[0U] = nextafterq(mutated[0U], HUGE_VALQ);
    result.mutation = al_r20_q_scalar_root(q.solution)
        != al_r20_q_scalar_root(mutated);
    result.classifier =
        al_r20_r63i_classify(false, true, true, true)
            == "EXPORTED_FACTOR_RHS_APPARATUS_REJECTED"
        && al_r20_r63i_classify(true, false, true, true)
            == "WIDE_RHS_SIGN_CERTIFICATE_REJECTED"
        && al_r20_r63i_classify(true, true, false, true)
            == "EXPORTED_FACTOR_WIDE_CONSUMPTION_REJECTED"
        && al_r20_r63i_classify(true, true, true, false)
            == "EXPORTED_FACTOR_BINARY64_CONSUMPTION_REJECTED"
        && al_r20_r63i_classify(true, true, true, true)
            == "EXPORTED_FACTOR_BINARY64_RHS_CANDIDATE";
    result.exact = result.solve && result.permutation
        && result.zero_diagonal && result.certificate
        && result.noncontractive && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':' + q.root
        + ':' + d.root + ':' + certificate.root + ':'
        + al_r20_q_scalar_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_exported_factor_rhs_controls() {
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
    const ALR20R63IControls controls = al_r20_r63i_controls();
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
    const ALR20R63EWitness inherited = full_profile.exact
        ? al_r20_r63e_witness(
            grams, factor_capture.projector, full_profile.binary128)
        : ALR20R63EWitness{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const ALR20R63FWitnessImage inherited_image = tangent.exact
        && inherited.exact
        ? al_r20_r63f_witness_image(tangent, inherited.alpha,
            inherited.decomposition.tangent)
        : ALR20R63FWitnessImage{};
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows);

    std::vector<Binary128> transposed;
    std::vector<Binary128> permuted_q;
    std::vector<double> permuted_d;
    if (tangent.exact && permutation_exact) {
        transposed.resize(tangent.columns * tangent.rows);
        permuted_q.resize(transposed.size());
        permuted_d.resize(transposed.size());
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                transposed[scalar * tangent.rows + column] =
                    tangent.values[column * tangent.columns + scalar];
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column) {
                permuted_q[scalar * tangent.rows + column] =
                    transposed[scalar * tangent.rows + permutation[column]];
                permuted_d[scalar * tangent.rows + column] =
                    static_cast<double>(
                        permuted_q[scalar * tangent.rows + column]);
            }
    }
    const ALR20R63GQFactor parent_q_factor = !permuted_q.empty()
        ? al_r20_r63g_q_factor(
            permuted_q, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    const ALR20R63GDFactor parent_d_factor = !permuted_d.empty()
        ? al_r20_r63g_d_factor(
            permuted_d, tangent.columns, tangent.rows)
        : ALR20R63GDFactor{};
    std::vector<Binary128> stored = transposed;
    for (Binary128& value : stored)
        value = static_cast<Binary128>(static_cast<double>(value));
    const std::vector<Binary128> stored_permuted =
        al_r20_r63g_q_from_double(parent_d_factor.input);
    const std::vector<Binary128> stored_gram =
        al_r20_r63g_gram(stored, tangent.columns, tangent.rows);
    const ALR20R63GQFactor wide_factor = !stored_permuted.empty()
        ? al_r20_r63g_q_factor(
            stored_permuted, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    std::vector<double> exported_q_d;
    std::vector<double> exported_r_d;
    if (wide_factor.exact) {
        exported_q_d.reserve(wide_factor.q.size());
        exported_r_d.reserve(wide_factor.r.size());
        for (Binary128 value : wide_factor.q)
            exported_q_d.push_back(static_cast<double>(value));
        for (Binary128 value : wide_factor.r)
            exported_r_d.push_back(static_cast<double>(value));
    }
    const std::vector<Binary128> exported_q =
        al_r20_r63g_q_from_double(exported_q_d);
    const std::vector<Binary128> exported_r =
        al_r20_r63g_q_from_double(exported_r_d);
    const ALR20R63GFactorAudit wide_audit = wide_factor.exact
        ? al_r20_r63g_factor_audit(stored_permuted, wide_factor.q,
            wide_factor.r, stored_gram, permutation, tangent.columns,
            tangent.rows, true)
        : ALR20R63GFactorAudit{};
    const ALR20R63GFactorAudit export_audit = wide_factor.exact
        ? al_r20_r63g_factor_audit(stored_permuted, exported_q,
            exported_r, stored_gram, permutation, tangent.columns,
            tangent.rows, false)
        : ALR20R63GFactorAudit{};
    const bool parent_r63h_exact = capture_exact && grams.exact
        && full_profile.exact && inherited.exact && inherited_image.exact
        && tangent.exact && permutation_exact && parent_q_factor.exact
        && parent_d_factor.exact && wide_factor.exact && wide_audit.exact
        && export_audit.exact
        && wide_factor.root == AL_R20_R63I_WIDE_FACTOR_SHA256
        && al_r20_double_root(exported_r_d) == AL_R20_R63I_EXPORT_R_SHA256;

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
        && reference_certificate.unresolved == 0U;

    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const ALR20R63IQSolve wide_solve = parent_r63h_exact
        ? al_r20_r63i_q_solve(wide_factor.r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IQSolve export_wide_solve = parent_r63h_exact
        ? al_r20_r63i_q_solve(exported_r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IDSolve strict_solve = parent_r63h_exact
        ? al_r20_r63i_d_solve(exported_r_d, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IDSolve{};
    const std::vector<Binary128> strict_solution =
        al_r20_r63g_q_from_double(strict_solve.solution);
    const ALR20R63ISignCertificate wide_certificate =
        wide_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            wide_solve.solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const ALR20R63ISignCertificate export_wide_certificate =
        export_wide_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            export_wide_solve.solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const ALR20R63ISignCertificate strict_certificate =
        strict_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            strict_solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = wide_solve.forward_terms == 5151U
        && wide_solve.backward_terms == 5151U
        && wide_solve.divisions == 204U
        && export_wide_solve.forward_terms == 5151U
        && export_wide_solve.backward_terms == 5151U
        && export_wide_solve.divisions == 204U
        && strict_solve.forward_terms == 5151U
        && strict_solve.backward_terms == 5151U
        && strict_solve.divisions == 204U
        && wide_certificate.residual_dots == 102U
        && wide_certificate.image_dots == 102U
        && export_wide_certificate.residual_dots == 102U
        && export_wide_certificate.image_dots == 102U
        && strict_certificate.residual_dots == 102U
        && strict_certificate.image_dots == 102U;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63h_exact && reference_exact
        && wide_solve.exact && export_wide_solve.exact && strict_solve.exact
        && wide_certificate.exact && export_wide_certificate.exact
        && strict_certificate.exact && lifecycle_exact && work_exact;
    const std::string route = al_r20_r63i_classify(apparatus_exact,
        wide_certificate.passed, export_wide_certificate.passed,
        strict_certificate.passed);
    const bool exact = apparatus_exact;

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63i-exported-factor-rhs.v1|"
        << AL_R20_R63I_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << reference_certificate.root << ':'
        << wide_factor.root << ':' << al_r20_double_root(exported_r_d) << ':'
        << wide_solve.root << ':' << export_wide_solve.root << ':'
        << strict_solve.root << ':' << wide_certificate.root << ':'
        << export_wide_certificate.root << ':' << strict_certificate.root;
    const std::string result_sha = sha256_hex(semantic.str());

    auto certificate_json = [](const char* name,
        const ALR20R63ISignCertificate& certificate) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (certificate.exact ? "true" : "false")
            << ",\"passed\":"
            << (certificate.passed ? "true" : "false")
            << ",\"residual_inf_bound\":\""
            << al_binary128_text(certificate.residual_inf_bound)
            << "\",\"image_inf_bound\":\""
            << al_binary128_text(certificate.image_inf_bound)
            << "\",\"error\":\""
            << al_binary128_text(certificate.error)
            << "\",\"positive\":" << certificate.positive
            << ",\"negative\":" << certificate.negative
            << ",\"unresolved\":" << certificate.unresolved
            << ",\"minimum_separation\":\""
            << al_binary128_text(certificate.minimum_separation)
            << "\",\"maximum_reference_difference\":\""
            << al_binary128_text(certificate.maximum_reference_difference)
            << "\",\"root\":\"" << certificate.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63i_exported_factor_rhs.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63h_sha256\":\""
        << AL_R20_R63I_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"solve\":" << (controls.solve ? "true" : "false")
        << ",\"permutation\":"
        << (controls.permutation ? "true" : "false")
        << ",\"zero_diagonal\":"
        << (controls.zero_diagonal ? "true" : "false")
        << ",\"certificate\":"
        << (controls.certificate ? "true" : "false")
        << ",\"noncontractive\":"
        << (controls.noncontractive ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"parent\":{\"exact\":"
        << (parent_r63h_exact ? "true" : "false")
        << ",\"wide_factor_root\":\"" << wide_factor.root << "\""
        << ",\"export_r_root\":\""
        << al_r20_double_root(exported_r_d) << "\"}"
        << ",\"reference\":{\"exact\":"
        << (reference_exact ? "true" : "false")
        << ",\"rho_bound\":\""
        << al_binary128_text(reference_certificate.rho_bound)
        << "\",\"error\":\""
        << al_binary128_text(reference_certificate.error)
        << "\",\"minimum_separation\":\""
        << al_binary128_text(reference_certificate.minimum_separation)
        << "\",\"positive\":" << reference_certificate.positive
        << ",\"negative\":" << reference_certificate.negative
        << ",\"root\":\"" << reference_certificate.root << "\"}"
        << ",\"solves\":{\"wide_root\":\"" << wide_solve.root
        << "\",\"export_wide_root\":\"" << export_wide_solve.root
        << "\",\"strict_binary64_root\":\"" << strict_solve.root
        << "\"}"
        << ",\"certificates\":{"
        << certificate_json("wide", wide_certificate) << ','
        << certificate_json("export_wide", export_wide_certificate) << ','
        << certificate_json("strict_binary64", strict_certificate) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"qr_factorizations\":3,\"nnqp_rhs_solves\":3"
        << ",\"triangular_solves\":6,\"iterative_refinements\":0"
        << ",\"new_inverse_constructions\":0,\"lsqr_solves\":0"
        << ",\"svd_factorizations\":0,\"rank_actions\":0"
        << ",\"row_drops\":0,\"regularizations\":0"
        << ",\"reference_center_depth\":4,\"center_updates\":0"
        << ",\"replacement_applies\":0,\"following_transitions\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
