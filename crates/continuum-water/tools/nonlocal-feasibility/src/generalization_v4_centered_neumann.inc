namespace {

constexpr const char* AL_R20_R40_SEMANTIC_SHA256 =
    "a999b65a7f3d4817d0aa58b1195942e63df23a2197f564d785701c6fb3dd6f7e";
constexpr const char* AL_R20_R40_LEFT_EXACT_SHA256 =
    "5f3c0e0604477626c8d5e338d2d29ad826c6bd5d064c522a124170680ce373f6";
constexpr const char* AL_R20_R40_LEFT_DOT2_SHA256 =
    "61bf229b48d8ad90c4cbfa7f98ce9c9750dbf933a70463de5bc6a08c3543aa50";
constexpr const char* AL_R20_R40_RESIDUAL_SAMPLES_SHA256 =
    "0774694c13ceb0774e8cf1952b3198a69c6a2fae3f219e7eb23994aa8a5e69df";
constexpr const char* AL_R20_R40_CORRECTION_SHA256 =
    "5318c6a347309242306fa73a69cbd49ecdfc3979dd997ca6f528c4f20894f2d9";
constexpr const char* AL_R20_R40_DIRECTIONAL_SHA256 =
    "9d02237becaff7d4c8b95bb670b20d234baa09e4a1b2ceec695a8dcd19869e99";

constexpr std::array<std::size_t, 6U> AL_R20_R41_DEPTHS{
    1U, 2U, 4U, 8U, 16U, 32U};

struct ALR20R41IntervalMatrix {
    bool exact = false;
    bool no_underflow = false;
    bool contained = false;
    bool contractive = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t contained_entries = 0U;
    std::size_t worst_row = 0U;
    Binary128 rho_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_exact_error = static_cast<Binary128>(0.0);
    std::vector<Binary128> value;
    std::vector<Binary128> bound;
    std::string dot_root;
    std::string root;
};

ALR20R41IntervalMatrix al_r20_r41_left_matrix(
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& matrix,
    const ALR20R37ExactResidual& oracle) {
    ALR20R41IntervalMatrix result;
    result.dimension = oracle.dimension;
    const std::size_t dimension = result.dimension;
    if (!oracle.exact || oracle.residual.size() != dimension * dimension
        || inverse.size() != dimension * dimension
        || matrix.size() != dimension * dimension)
        return result;
    result.value.resize(dimension * dimension);
    result.bound.resize(dimension * dimension);
    bool apparatus_exact = true;
    bool no_underflow = true;
    bool contained = true;
    std::ostringstream dot_material;
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
            const ALR20R37Conversion nominal =
                al_r20_r37_from_binary128(dot.value);
            const ALR20R37Conversion bound =
                al_r20_r37_from_binary128(dot.bound);
            const bool entry_exact = dot.exact
                && nominal.exact && bound.exact;
            ALR20R37Dyadic difference;
            if (entry_exact)
                difference = al_r20_r37_absolute(al_r20_r37_add(
                    oracle.residual[row * dimension + column],
                    {-nominal.value.numerator, nominal.value.exponent}));
            const Binary128 exact_error = entry_exact
                ? al_r20_r37_binary128_up(difference)
                : static_cast<Binary128>(0.0);
            const bool entry_contained = entry_exact
                && al_r20_r37_compare_positive(
                    difference, bound.value) <= 0;
            const std::size_t index = row * dimension + column;
            result.value[index] = dot.value;
            result.bound[index] = dot.bound;
            ++result.dots;
            result.contained_entries += entry_contained ? 1U : 0U;
            apparatus_exact = apparatus_exact && entry_exact
                && finiteq(exact_error) != 0;
            no_underflow = no_underflow && dot.no_underflow;
            contained = contained && entry_contained;
            result.maximum_bound = fmaxq(result.maximum_bound, dot.bound);
            result.maximum_exact_error = fmaxq(
                result.maximum_exact_error, exact_error);
            row_bound = al_r20_r38_up_add(row_bound,
                al_r20_r38_up_add(fabsq(dot.value), dot.bound));
            dot_material << dot.root << ':' << entry_contained << ';';
        }
        dot_material << '|';
        if (row == 0U || row_bound > result.rho_bound) {
            result.rho_bound = row_bound;
            result.worst_row = row;
        }
    }
    result.no_underflow = no_underflow;
    result.contained = contained
        && result.contained_entries == dimension * dimension;
    result.contractive = result.rho_bound
        < static_cast<Binary128>(1.0);
    result.dot_root = sha256_hex(dot_material.str());
    result.exact = apparatus_exact
        && result.dots == dimension * dimension
        && al_r20_admm_finite(result.value)
        && al_r20_admm_finite(result.bound)
        && finiteq(result.rho_bound) != 0
        && finiteq(result.maximum_bound) != 0
        && finiteq(result.maximum_exact_error) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.contained << ':' << result.contractive << ':'
        << dimension << ':' << result.dots << ':'
        << result.contained_entries << ':' << result.worst_row << ':'
        << al_binary128_hex(result.rho_bound) << ':'
        << al_binary128_hex(result.maximum_bound) << ':'
        << al_binary128_hex(result.maximum_exact_error) << ':'
        << result.dot_root << ':' << oracle.root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R41IntervalVector {
    bool exact = false;
    bool no_underflow = false;
    bool contained = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t contained_entries = 0U;
    Binary128 maximum_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_exact_error = static_cast<Binary128>(0.0);
    std::vector<Binary128> value;
    std::vector<Binary128> bound;
    std::string dot_root;
    std::string root;
};

ALR20R41IntervalVector al_r20_r41_z_vector(
    const std::vector<Binary128>& inverse,
    const ALR20R40ResidualSamples& residual,
    const ALR20R40ExactCorrection& oracle) {
    ALR20R41IntervalVector result;
    result.dimension = residual.dimension;
    const std::size_t dimension = result.dimension;
    if (!residual.exact || !oracle.exact
        || residual.nominal.size() != dimension
        || residual.bound.size() != dimension
        || oracle.value.size() != dimension
        || inverse.size() != dimension * dimension)
        return result;
    result.value.resize(dimension);
    result.bound.resize(dimension);
    bool apparatus_exact = true;
    bool no_underflow = true;
    bool contained = true;
    std::ostringstream dot_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension);
        for (std::size_t column = 0U; column < dimension; ++column)
            left[column] = inverse[row * dimension + column];
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, residual.nominal);
        Binary128 input_bound = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < dimension; ++column)
            input_bound = al_r20_r38_up_add(input_bound,
                al_r20_r38_up_multiply(
                    fabsq(left[column]), residual.bound[column]));
        const Binary128 combined_bound = al_r20_r38_up_add(
            dot.bound, input_bound);
        const ALR20R37Conversion nominal =
            al_r20_r37_from_binary128(dot.value);
        const ALR20R37Conversion bound =
            al_r20_r37_from_binary128(combined_bound);
        const bool entry_exact = dot.exact
            && nominal.exact && bound.exact;
        ALR20R37Dyadic difference;
        if (entry_exact)
            difference = al_r20_r37_absolute(al_r20_r37_add(
                oracle.value[row],
                {-nominal.value.numerator, nominal.value.exponent}));
        const Binary128 exact_error = entry_exact
            ? al_r20_r37_binary128_up(difference)
            : static_cast<Binary128>(0.0);
        const bool entry_contained = entry_exact
            && al_r20_r37_compare_positive(
                difference, bound.value) <= 0;
        result.value[row] = dot.value;
        result.bound[row] = combined_bound;
        ++result.dots;
        result.contained_entries += entry_contained ? 1U : 0U;
        apparatus_exact = apparatus_exact && entry_exact
            && finiteq(exact_error) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        contained = contained && entry_contained;
        result.maximum_bound = fmaxq(
            result.maximum_bound, combined_bound);
        result.maximum_exact_error = fmaxq(
            result.maximum_exact_error, exact_error);
        dot_material << dot.root << ':' << al_binary128_hex(input_bound)
            << ':' << entry_contained << ';';
    }
    result.no_underflow = no_underflow;
    result.contained = contained
        && result.contained_entries == dimension;
    result.dot_root = sha256_hex(dot_material.str());
    result.exact = apparatus_exact && result.dots == dimension
        && al_r20_admm_finite(result.value)
        && al_r20_admm_finite(result.bound)
        && finiteq(result.maximum_bound) != 0
        && finiteq(result.maximum_exact_error) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.contained << ':' << dimension << ':' << result.dots << ':'
        << result.contained_entries << ':'
        << al_binary128_hex(result.maximum_bound) << ':'
        << al_binary128_hex(result.maximum_exact_error) << ':'
        << result.dot_root << ':' << oracle.root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R41Checkpoint {
    bool exact = false;
    bool no_underflow = false;
    bool contained = false;
    bool signs_resolved = false;
    std::size_t depth = 0U;
    std::size_t residual_dots = 0U;
    std::size_t center_dots = 0U;
    std::size_t residual_contained = 0U;
    std::size_t center_contained = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    std::size_t sign_conflicts = 0U;
    std::size_t worst_row = 0U;
    Binary128 g_inf_bound = static_cast<Binary128>(0.0);
    Binary128 radius = static_cast<Binary128>(0.0);
    Binary128 minimum_separation = HUGE_VALQ;
    Binary128 maximum_residual_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_residual_exact_error = static_cast<Binary128>(0.0);
    Binary128 maximum_center_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_center_exact_error = static_cast<Binary128>(0.0);
    std::string residual_root;
    std::string center_root;
    std::string root;
};

ALR20R41Checkpoint al_r20_r41_checkpoint(
    std::size_t depth,
    const std::vector<Binary128>& correction,
    const std::vector<Binary128>& solution,
    Binary128 parent_solution_error,
    const ALR20R41IntervalMatrix& left,
    const ALR20R37ExactResidual& left_oracle,
    const ALR20R41IntervalVector& z,
    const ALR20R40ExactCorrection& z_oracle) {
    ALR20R41Checkpoint result;
    result.depth = depth;
    const std::size_t dimension = solution.size();
    if (!left.exact || !left_oracle.exact || !z.exact || !z_oracle.exact
        || correction.size() != dimension
        || left.dimension != dimension || z.dimension != dimension)
        return result;
    std::vector<ALR20R37Dyadic> correction_exact(dimension);
    bool apparatus_exact = true;
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R37Conversion converted =
            al_r20_r37_from_binary128(correction[index]);
        apparatus_exact = apparatus_exact && converted.exact;
        correction_exact[index] = converted.value;
    }
    bool no_underflow = true;
    bool contained = true;
    std::vector<Binary128> centers(dimension);
    std::vector<Binary128> center_bounds(dimension);
    std::ostringstream residual_material;
    std::ostringstream center_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic exact_g = z_oracle.value[row];
        for (std::size_t column = 0U; column < dimension; ++column)
            exact_g = al_r20_r37_add(exact_g, al_r20_r37_multiply(
                left_oracle.residual[row * dimension + column],
                correction_exact[column]));
        exact_g = al_r20_r37_add(exact_g,
            {-correction_exact[row].numerator,
                correction_exact[row].exponent});

        std::vector<Binary128> residual_left(dimension + 2U);
        std::vector<Binary128> residual_right(dimension + 2U,
            static_cast<Binary128>(1.0));
        residual_left[0U] = z.value[row];
        for (std::size_t column = 0U; column < dimension; ++column) {
            residual_left[column + 1U] =
                left.value[row * dimension + column];
            residual_right[column + 1U] = correction[column];
        }
        residual_left[dimension + 1U] = -correction[row];
        const ALR20R38Dot2 residual_dot =
            al_r20_r38_dot2(residual_left, residual_right);
        Binary128 input_bound = z.bound[row];
        for (std::size_t column = 0U; column < dimension; ++column)
            input_bound = al_r20_r38_up_add(input_bound,
                al_r20_r38_up_multiply(
                    left.bound[row * dimension + column],
                    fabsq(correction[column])));
        const Binary128 residual_bound = al_r20_r38_up_add(
            residual_dot.bound, input_bound);
        const ALR20R37Conversion residual_nominal =
            al_r20_r37_from_binary128(residual_dot.value);
        const ALR20R37Conversion residual_bound_exact =
            al_r20_r37_from_binary128(residual_bound);
        const bool residual_entry_exact = residual_dot.exact
            && residual_nominal.exact && residual_bound_exact.exact;
        ALR20R37Dyadic residual_difference;
        if (residual_entry_exact)
            residual_difference = al_r20_r37_absolute(al_r20_r37_add(
                exact_g, {-residual_nominal.value.numerator,
                    residual_nominal.value.exponent}));
        const Binary128 residual_exact_error = residual_entry_exact
            ? al_r20_r37_binary128_up(residual_difference)
            : static_cast<Binary128>(0.0);
        const bool residual_contained = residual_entry_exact
            && al_r20_r37_compare_positive(
                residual_difference, residual_bound_exact.value) <= 0;
        ++result.residual_dots;
        result.residual_contained += residual_contained ? 1U : 0U;
        apparatus_exact = apparatus_exact && residual_entry_exact
            && finiteq(residual_exact_error) != 0;
        no_underflow = no_underflow && residual_dot.no_underflow;
        contained = contained && residual_contained;
        result.maximum_residual_bound = fmaxq(
            result.maximum_residual_bound, residual_bound);
        result.maximum_residual_exact_error = fmaxq(
            result.maximum_residual_exact_error, residual_exact_error);
        const Binary128 component_g = al_r20_r38_up_add(
            fabsq(residual_dot.value), residual_bound);
        if (row == 0U || component_g > result.g_inf_bound) {
            result.g_inf_bound = component_g;
            result.worst_row = row;
        }
        residual_material << residual_dot.root << ':'
            << al_binary128_hex(input_bound) << ':'
            << residual_contained << ';';

        std::vector<Binary128> sum_left{solution[row], correction[row]};
        const std::vector<Binary128> sum_right{
            static_cast<Binary128>(1.0), static_cast<Binary128>(1.0)};
        const ALR20R38Dot2 center_dot =
            al_r20_r38_dot2(sum_left, sum_right);
        const ALR20R37Conversion solution_exact =
            al_r20_r37_from_binary128(solution[row]);
        ALR20R37Dyadic exact_center;
        if (solution_exact.exact)
            exact_center = al_r20_r37_add(
                solution_exact.value, correction_exact[row]);
        Binary128 center_exact_error = static_cast<Binary128>(0.0);
        const bool center_contained = solution_exact.exact
            && al_r20_r38_dot_contains(
                center_dot, exact_center, &center_exact_error);
        centers[row] = center_dot.value;
        center_bounds[row] = center_dot.bound;
        ++result.center_dots;
        result.center_contained += center_contained ? 1U : 0U;
        apparatus_exact = apparatus_exact && solution_exact.exact
            && center_dot.exact && finiteq(center_exact_error) != 0;
        no_underflow = no_underflow && center_dot.no_underflow;
        contained = contained && center_contained;
        result.maximum_center_bound = fmaxq(
            result.maximum_center_bound, center_dot.bound);
        result.maximum_center_exact_error = fmaxq(
            result.maximum_center_exact_error, center_exact_error);
        center_material << center_dot.root << ':'
            << center_contained << ';';
    }
    if (left.contractive) {
        const Binary128 denominator = nextafterq(
            static_cast<Binary128>(1.0) - left.rho_bound, -HUGE_VALQ);
        result.radius = al_r20_r38_up_divide(
            result.g_inf_bound, denominator);
        for (std::size_t row = 0U; row < dimension; ++row) {
            const Binary128 total_bound = al_r20_r38_up_add(
                result.radius, center_bounds[row]);
            result.minimum_separation = fminq(result.minimum_separation,
                fabsq(centers[row]) - total_bound);
            int centered_sign = 0;
            if (centers[row] + total_bound
                    < static_cast<Binary128>(0.0)) {
                centered_sign = -1;
                ++result.negative;
            } else if (centers[row] - total_bound
                    > static_cast<Binary128>(0.0)) {
                centered_sign = 1;
                ++result.positive;
            } else {
                ++result.unresolved;
            }
            int parent_sign = 0;
            if (solution[row] + parent_solution_error
                    < static_cast<Binary128>(0.0))
                parent_sign = -1;
            else if (solution[row] - parent_solution_error
                    > static_cast<Binary128>(0.0))
                parent_sign = 1;
            if (parent_sign != 0 && centered_sign != 0
                && parent_sign != centered_sign)
                ++result.sign_conflicts;
        }
    } else {
        result.radius = static_cast<Binary128>(0.0);
        result.minimum_separation = static_cast<Binary128>(0.0);
        result.unresolved = dimension;
    }
    result.no_underflow = no_underflow;
    result.contained = contained
        && result.residual_contained == dimension
        && result.center_contained == dimension;
    result.signs_resolved = result.unresolved == 0U
        && result.positive + result.negative == dimension;
    result.residual_root = sha256_hex(residual_material.str());
    result.center_root = sha256_hex(center_material.str());
    result.exact = apparatus_exact
        && result.residual_dots == dimension
        && result.center_dots == dimension
        && finiteq(result.g_inf_bound) != 0
        && finiteq(result.radius) != 0
        && finiteq(result.minimum_separation) != 0
        && finiteq(result.maximum_residual_bound) != 0
        && finiteq(result.maximum_residual_exact_error) != 0
        && finiteq(result.maximum_center_bound) != 0
        && finiteq(result.maximum_center_exact_error) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.contained << ':' << result.signs_resolved << ':'
        << depth << ':' << result.residual_dots << ':'
        << result.center_dots << ':' << result.residual_contained << ':'
        << result.center_contained << ':' << result.positive << ':'
        << result.negative << ':' << result.unresolved << ':'
        << result.sign_conflicts << ':' << result.worst_row << ':'
        << al_binary128_hex(result.g_inf_bound) << ':'
        << al_binary128_hex(result.radius) << ':'
        << al_binary128_hex(result.minimum_separation) << ':'
        << al_binary128_hex(result.maximum_residual_bound) << ':'
        << al_binary128_hex(result.maximum_residual_exact_error) << ':'
        << al_binary128_hex(result.maximum_center_bound) << ':'
        << al_binary128_hex(result.maximum_center_exact_error) << ':'
        << result.residual_root << ':' << result.center_root;
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_r41_checkpoint_depth(std::size_t depth) {
    return std::find(AL_R20_R41_DEPTHS.begin(),
        AL_R20_R41_DEPTHS.end(), depth) != AL_R20_R41_DEPTHS.end();
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_centered_neumann_controls() {
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
    ALR20VerifiedInverseCapture capture;
    capture.selected_root = AL_R20_R36_TORSION_INVERSE_SHA256;
    ALR20R35CaseAudit torsion;
    std::size_t solver_executions = 0U;
    const bool capture_slot_clear =
        al_r20_verified_inverse_capture == nullptr;
    if (parent_exact && capture_slot_clear) {
        al_r20_verified_inverse_capture = &capture;
        torsion = al_r20_r35_case(problem);
        al_r20_verified_inverse_capture = nullptr;
        solver_executions = 1U;
    }
    const std::size_t target_capture_count = static_cast<std::size_t>(
        std::count(capture.audit_roots.begin(), capture.audit_roots.end(),
            AL_R20_R36_TORSION_INVERSE_SHA256));
    const bool reproduction_exact = parent_exact && capture_slot_clear
        && torsion.root == AL_R20_R35_TORSION_CASE_SHA256
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.steps.back().root
            == AL_R20_R35_TORSION_STEP_SHA256
        && capture.calls == 2U && target_capture_count == 1U
        && capture.audit.root == AL_R20_R36_TORSION_INVERSE_SHA256
        && capture.matrix_root == AL_R20_R37_MATRIX_SHA256
        && capture.factor_root == AL_R20_R37_FACTOR_SHA256
        && al_r20_q_scalar_root(capture.inverse)
            == AL_R20_R37_INVERSE_SHA256
        && al_r20_q_scalar_root(capture.rhs) == AL_R20_R39_RHS_SHA256
        && al_r20_q_scalar_root(capture.solution)
            == AL_R20_R39_SOLUTION_SHA256
        && solver_executions == 1U;
    const ALR20R37ExactResidual right_oracle = reproduction_exact
        ? al_r20_r37_exact_residual(
            capture.matrix, capture.inverse, capture.audit.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit right_certificate = right_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.matrix, capture.inverse, right_oracle)
        : ALR20R38MatrixAudit{};
    const ALR20R39ExactVectorResidual residual_oracle =
        reproduction_exact && right_certificate.root
                == AL_R20_R38_CERTIFICATE_SHA256
        ? al_r20_r39_exact_vector_residual(
            capture.matrix, capture.rhs, capture.solution)
        : ALR20R39ExactVectorResidual{};
    const ALR20R39SolutionAudit parent_solution = residual_oracle.exact
        ? al_r20_r39_solution_audit(capture.matrix, capture.inverse,
            capture.rhs, capture.solution, right_certificate, residual_oracle)
        : ALR20R39SolutionAudit{};
    const bool parent_solution_exact =
        residual_oracle.root == AL_R20_R39_RESIDUAL_SHA256
        && parent_solution.root == AL_R20_R39_SOLUTION_AUDIT_SHA256;
    const ALR20R37ExactResidual left_oracle = parent_solution_exact
        ? al_r20_r37_exact_residual(
            capture.inverse, capture.matrix, capture.audit.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit left_certificate = left_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.inverse, capture.matrix, left_oracle)
        : ALR20R38MatrixAudit{};
    const ALR20R40ResidualSamples residual_samples = parent_solution_exact
        ? al_r20_r40_residual_samples(
            capture.matrix, capture.rhs, capture.solution, residual_oracle)
        : ALR20R40ResidualSamples{};
    const ALR20R40ExactCorrection correction_oracle = residual_samples.exact
        ? al_r20_r40_exact_correction(capture.inverse, residual_oracle)
        : ALR20R40ExactCorrection{};
    const ALR20R40DirectionalAudit directional =
        correction_oracle.exact && left_certificate.exact
        ? al_r20_r40_directional_audit(capture.inverse, capture.solution,
            residual_samples, correction_oracle, left_certificate)
        : ALR20R40DirectionalAudit{};
    const bool r40_exact = reproduction_exact && parent_solution_exact
        && left_oracle.root == AL_R20_R40_LEFT_EXACT_SHA256
        && left_certificate.root == AL_R20_R40_LEFT_DOT2_SHA256
        && residual_samples.root == AL_R20_R40_RESIDUAL_SAMPLES_SHA256
        && correction_oracle.root == AL_R20_R40_CORRECTION_SHA256
        && directional.root == AL_R20_R40_DIRECTIONAL_SHA256;

    const ALR20R41IntervalMatrix left = r40_exact
        ? al_r20_r41_left_matrix(
            capture.inverse, capture.matrix, left_oracle)
        : ALR20R41IntervalMatrix{};
    const ALR20R41IntervalVector z = r40_exact
        ? al_r20_r41_z_vector(
            capture.inverse, residual_samples, correction_oracle)
        : ALR20R41IntervalVector{};
    std::vector<Binary128> center(capture.audit.dimension,
        static_cast<Binary128>(0.0));
    std::vector<ALR20R41Checkpoint> checkpoints;
    std::size_t generation_dots = 0U;
    bool generation_exact = r40_exact && left.exact && z.exact;
    bool generation_no_underflow = true;
    std::ostringstream generation_material;
    for (std::size_t depth = 1U; generation_exact && depth <= 32U; ++depth) {
        std::vector<Binary128> next(center.size());
        for (std::size_t row = 0U; row < center.size(); ++row) {
            std::vector<Binary128> lhs(center.size() + 1U);
            std::vector<Binary128> rhs(center.size() + 1U);
            lhs[0U] = z.value[row];
            rhs[0U] = static_cast<Binary128>(1.0);
            for (std::size_t column = 0U; column < center.size(); ++column) {
                lhs[column + 1U] = left.value[row * center.size() + column];
                rhs[column + 1U] = center[column];
            }
            const ALR20R38Dot2 dot = al_r20_r38_dot2(lhs, rhs);
            next[row] = dot.value;
            ++generation_dots;
            generation_exact = generation_exact && dot.exact
                && finiteq(dot.value) != 0 && finiteq(dot.bound) != 0;
            generation_no_underflow = generation_no_underflow
                && dot.no_underflow;
            generation_material << depth << ':' << row << ':'
                << dot.root << ';';
        }
        center = std::move(next);
        if (al_r20_r41_checkpoint_depth(depth))
            checkpoints.push_back(al_r20_r41_checkpoint(depth, center,
                capture.solution, parent_solution.solution_error_bound,
                left, left_oracle, z, correction_oracle));
    }
    const std::string generation_root =
        sha256_hex(generation_material.str());
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    bool checkpoints_exact = checkpoints.size() == AL_R20_R41_DEPTHS.size();
    bool checkpoints_no_underflow = true;
    bool checkpoints_contained = true;
    std::size_t first_resolved_depth = 0U;
    std::size_t total_residual_dots = 0U;
    std::size_t total_center_dots = 0U;
    for (const ALR20R41Checkpoint& checkpoint : checkpoints) {
        checkpoints_exact = checkpoints_exact && checkpoint.exact;
        checkpoints_no_underflow = checkpoints_no_underflow
            && checkpoint.no_underflow;
        checkpoints_contained = checkpoints_contained
            && checkpoint.contained;
        total_residual_dots += checkpoint.residual_dots;
        total_center_dots += checkpoint.center_dots;
        if (first_resolved_depth == 0U && checkpoint.signs_resolved
            && checkpoint.sign_conflicts == 0U)
            first_resolved_depth = checkpoint.depth;
    }
    const ALR20R41Checkpoint final_checkpoint = checkpoints.empty()
        ? ALR20R41Checkpoint{} : checkpoints.back();
    const bool exact = parent_exact && reproduction_exact
        && parent_solution_exact && r40_exact && left.exact && z.exact
        && generation_exact && checkpoints_exact && lifecycle_exact;
    const std::string route = !parent_exact || !reproduction_exact
            || !parent_solution_exact || !r40_exact
        ? "CENTERED_NEUMANN_PARENT_REJECTED"
        : (!left.exact || !z.exact || !generation_exact || !checkpoints_exact
            ? "CENTERED_NEUMANN_APPARATUS_REJECTED"
            : (!left.no_underflow || !z.no_underflow
                    || !generation_no_underflow || !checkpoints_no_underflow
                ? "CENTERED_NEUMANN_UNDERFLOW_REJECTED"
                : (!left.contained || !z.contained || !checkpoints_contained
                    ? "CENTERED_NEUMANN_CONTAINMENT_REJECTED"
                    : (!left.contractive
                        ? "CENTERED_NEUMANN_LEFT_NONCONTRACTIVE"
                        : (final_checkpoint.sign_conflicts != 0U
                            ? "CENTERED_NEUMANN_SIGN_CONFLICT"
                            : (!final_checkpoint.signs_resolved
                                ? "CENTERED_NEUMANN_SIGN_UNRESOLVED"
                                : "CENTERED_NEUMANN_SOLUTION_CANDIDATE"))))));
    std::ostringstream checkpoint_semantic;
    for (const ALR20R41Checkpoint& checkpoint : checkpoints)
        checkpoint_semantic << checkpoint.root << ';';
    const std::string checkpoint_root =
        sha256_hex(checkpoint_semantic.str());
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r41-centered.v1|"
        << AL_R20_R40_SEMANTIC_SHA256 << ':' << route << ':'
        << torsion.root << ':' << left.root << ':' << z.root << ':'
        << generation_root << ':' << checkpoint_root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream checkpoint_report;
    checkpoint_report << '[';
    for (std::size_t index = 0U; index < checkpoints.size(); ++index) {
        if (index != 0U) checkpoint_report << ',';
        const ALR20R41Checkpoint& checkpoint = checkpoints[index];
        checkpoint_report << "{\"depth\":" << checkpoint.depth
            << ",\"root\":\"" << checkpoint.root
            << "\",\"contained\":"
            << (checkpoint.contained ? "true" : "false")
            << ",\"no_underflow\":"
            << (checkpoint.no_underflow ? "true" : "false")
            << ",\"g_inf_bound\":\""
            << al_binary128_text(checkpoint.g_inf_bound)
            << "\",\"radius\":\""
            << al_binary128_text(checkpoint.radius)
            << "\",\"positive\":" << checkpoint.positive
            << ",\"negative\":" << checkpoint.negative
            << ",\"unresolved\":" << checkpoint.unresolved
            << ",\"sign_conflicts\":" << checkpoint.sign_conflicts
            << ",\"minimum_separation\":\""
            << al_binary128_text(checkpoint.minimum_separation)
            << "\",\"worst_row\":" << checkpoint.worst_row
            << ",\"maximum_residual_bound\":\""
            << al_binary128_text(checkpoint.maximum_residual_bound)
            << "\",\"maximum_residual_exact_error\":\""
            << al_binary128_text(checkpoint.maximum_residual_exact_error)
            << "\",\"maximum_center_bound\":\""
            << al_binary128_text(checkpoint.maximum_center_bound)
            << "\",\"maximum_center_exact_error\":\""
            << al_binary128_text(checkpoint.maximum_center_exact_error)
            << "\"}";
    }
    checkpoint_report << ']';
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r41_centered.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r40_sha256\":\""
        << AL_R20_R40_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"r40_exact\":" << (r40_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"case_sha256\":\"" << torsion.root
        << "\",\"left\":{\"root\":\"" << left.root
        << "\",\"rho_bound\":\"" << al_binary128_text(left.rho_bound)
        << "\",\"worst_row\":" << left.worst_row
        << ",\"contained_entries\":" << left.contained_entries
        << "},\"z\":{\"root\":\"" << z.root
        << "\",\"contained_entries\":" << z.contained_entries
        << ",\"maximum_bound\":\"" << al_binary128_text(z.maximum_bound)
        << "\"},\"generation_root\":\"" << generation_root
        << "\",\"checkpoint_root\":\"" << checkpoint_root
        << "\",\"first_resolved_depth\":" << first_resolved_depth
        << ",\"checkpoints\":" << checkpoint_report.str()
        << ",\"solver_executions\":" << solver_executions
        << ",\"captured_inverse_calls\":" << capture.calls
        << ",\"captured_inverse_columns\":" << capture.total_column_solves
        << ",\"left_defect_dots\":" << left.dots
        << ",\"solution_residual_dots\":" << residual_samples.dimension
        << ",\"z_dots\":" << z.dots
        << ",\"center_generation_dots\":" << generation_dots
        << ",\"checkpoint_residual_dots\":" << total_residual_dots
        << ",\"checkpoint_center_dots\":" << total_center_dots
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0,\"additional_solves\":0"
        << ",\"correction_applies\":0,\"nnqp_decisions\":0"
        << ",\"nnqp_transitions\":0,\"counterflow_replays\":0"
        << ",\"new_trials\":0,\"state_updates\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
