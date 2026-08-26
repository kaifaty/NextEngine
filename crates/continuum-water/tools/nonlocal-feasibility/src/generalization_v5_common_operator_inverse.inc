namespace {

constexpr const char* AL_R20_R63R_PARENT_SEMANTIC_SHA256 =
    "48ec75fe0a173dabbfae012372dd43c39d8d8c34ed5f2dddc39fea39ebd0e345";
constexpr const char* AL_R20_R63R_AUDIT_SHA256 =
    "b24da3fed653f7583b2b49a47a698c3421aa09ba68ae2342f60a1d061c7eae09";
constexpr const char* AL_R20_R63R_MATRIX_SHA256 =
    "1e935dbe8a29210c3601038812c7b6b18bcb73b7d82c3f64bbfaf628c982998a";
constexpr const char* AL_R20_R63R_ORACLE_SHA256 =
    "221b475869ab34ee919d1d3add4aa993ab361e2dd914be308722f03726c41266";
constexpr const char* AL_R20_R63R_FACTOR_SHA256 =
    "fde9aef5ee0233c920ab76c855c612cc722422c67f44d18598211ae901082ec4";
constexpr const char* AL_R20_R63R_PERMUTATION_SHA256 =
    "335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f";

struct ALR20R63RInverse {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t column_solves = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::size_t writes = 0U;
    std::vector<Binary128> value;
    std::vector<std::string> column_roots;
    std::string root;
};

ALR20R63RInverse al_r20_r63r_inverse(
    const std::vector<Binary128>& upper,
    const std::vector<std::size_t>& permutation,
    Binary128 inverse_scale) {
    ALR20R63RInverse result;
    result.dimension = permutation.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || !(inverse_scale > static_cast<Binary128>(0.0))) return result;
    result.value.resize(dimension * dimension);
    bool exact = true;
    for (std::size_t column = 0U; column < dimension; ++column) {
        std::vector<Binary128> basis(
            dimension, static_cast<Binary128>(0.0));
        basis[column] = static_cast<Binary128>(1.0);
        const ALR20R63IQSolve solve = al_r20_r63i_q_solve(
            upper, permutation, basis, inverse_scale);
        exact = exact && solve.exact && solve.solution.size() == dimension;
        if (!solve.exact) break;
        for (std::size_t row = 0U; row < dimension; ++row) {
            result.value[row * dimension + column] = solve.solution[row];
            ++result.writes;
        }
        result.forward_terms += solve.forward_terms;
        result.backward_terms += solve.backward_terms;
        result.divisions += solve.divisions;
        result.column_roots.push_back(solve.root);
        ++result.column_solves;
    }
    result.exact = exact && result.column_solves == dimension
        && result.forward_terms
            == dimension * dimension * (dimension - 1U) / 2U
        && result.backward_terms
            == dimension * dimension * (dimension - 1U) / 2U
        && result.divisions == 2U * dimension * dimension
        && result.writes == dimension * dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.dimension << ':'
        << result.column_solves << ':' << result.forward_terms << ':'
        << result.backward_terms << ':' << result.divisions << ':'
        << result.writes << ':' << al_r20_q_scalar_root(result.value) << '|';
    for (const std::string& root : result.column_roots)
        material << root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63RDefect {
    bool exact = false;
    bool left_contractive = false;
    bool right_contractive = false;
    std::size_t dimension = 0U;
    std::size_t left_products = 0U;
    std::size_t right_products = 0U;
    std::size_t residual_entries = 0U;
    std::size_t row_sum_terms = 0U;
    std::size_t inverse_row_terms = 0U;
    std::size_t asymmetry_terms = 0U;
    ALR20R37Dyadic d;
    ALR20R37Dyadic maximum_left_entry;
    ALR20R37Dyadic maximum_right_entry;
    ALR20R37Dyadic left_infinity_numerator;
    ALR20R37Dyadic right_infinity_numerator;
    ALR20R37Dyadic inverse_infinity_norm;
    ALR20R37Dyadic maximum_asymmetry;
    ALR20R37Dyadic amplification_numerator;
    ALR20R37Dyadic amplification_denominator;
    std::vector<ALR20R37Dyadic> left;
    std::vector<ALR20R37Dyadic> right;
    std::string inverse_root;
    std::string left_root;
    std::string right_root;
    std::string root;
};

ALR20R63RDefect al_r20_r63r_defect(
    const std::vector<ALR20R37Dyadic>& inverse,
    const std::vector<ALR20R37Dyadic>& oracle_numerator,
    const ALR20R37Dyadic& d, std::size_t dimension) {
    ALR20R63RDefect result;
    result.dimension = dimension;
    result.d = d;
    if (dimension == 0U || inverse.size() != dimension * dimension
        || oracle_numerator.size() != dimension * dimension
        || al_r20_r63p_sign(d) <= 0) return result;
    const std::size_t entries = dimension * dimension;
    result.left.resize(entries);
    result.right.resize(entries);
    std::vector<ALR20R37Dyadic> left_rows(dimension);
    std::vector<ALR20R37Dyadic> right_rows(dimension);
    std::vector<ALR20R37Dyadic> inverse_rows(dimension);
    for (std::size_t row = 0U; row < dimension; ++row) {
        for (std::size_t column = 0U; column < dimension; ++column) {
            ALR20R37Dyadic left_product;
            ALR20R37Dyadic right_product;
            for (std::size_t inner = 0U; inner < dimension; ++inner) {
                left_product = al_r20_r37_add(left_product,
                    al_r20_r37_multiply(
                        inverse[row * dimension + inner],
                        oracle_numerator[inner * dimension + column]));
                right_product = al_r20_r37_add(right_product,
                    al_r20_r37_multiply(
                        oracle_numerator[row * dimension + inner],
                        inverse[inner * dimension + column]));
                ++result.left_products;
                ++result.right_products;
            }
            const ALR20R37Dyadic expected = row == column
                ? d : ALR20R37Dyadic{};
            const std::size_t index = row * dimension + column;
            result.left[index] = al_r20_r63p_subtract(
                expected, left_product);
            result.right[index] = al_r20_r63p_subtract(
                expected, right_product);
            const ALR20R37Dyadic left_absolute = al_r20_r37_absolute(
                result.left[index]);
            const ALR20R37Dyadic right_absolute = al_r20_r37_absolute(
                result.right[index]);
            if (index == 0U || al_r20_r37_compare_positive(
                    left_absolute, result.maximum_left_entry) > 0)
                result.maximum_left_entry = left_absolute;
            if (index == 0U || al_r20_r37_compare_positive(
                    right_absolute, result.maximum_right_entry) > 0)
                result.maximum_right_entry = right_absolute;
            left_rows[row] = al_r20_r37_add(
                left_rows[row], left_absolute);
            right_rows[row] = al_r20_r37_add(
                right_rows[row], right_absolute);
            ++result.residual_entries;
            result.row_sum_terms += 2U;
        }
        for (std::size_t column = 0U; column < dimension; ++column) {
            inverse_rows[row] = al_r20_r37_add(inverse_rows[row],
                al_r20_r37_absolute(
                    inverse[row * dimension + column]));
            const ALR20R37Dyadic asymmetry = al_r20_r37_absolute(
                al_r20_r63p_subtract(
                    inverse[row * dimension + column],
                    inverse[column * dimension + row]));
            if ((row == 0U && column == 0U)
                || al_r20_r37_compare_positive(
                    asymmetry, result.maximum_asymmetry) > 0)
                result.maximum_asymmetry = asymmetry;
            ++result.inverse_row_terms;
            ++result.asymmetry_terms;
        }
    }
    for (std::size_t row = 0U; row < dimension; ++row) {
        if (row == 0U || al_r20_r37_compare_positive(
                left_rows[row], result.left_infinity_numerator) > 0)
            result.left_infinity_numerator = left_rows[row];
        if (row == 0U || al_r20_r37_compare_positive(
                right_rows[row], result.right_infinity_numerator) > 0)
            result.right_infinity_numerator = right_rows[row];
        if (row == 0U || al_r20_r37_compare_positive(
                inverse_rows[row], result.inverse_infinity_norm) > 0)
            result.inverse_infinity_norm = inverse_rows[row];
    }
    result.left_contractive = al_r20_r37_compare_positive(
        result.left_infinity_numerator, d) < 0;
    result.right_contractive = al_r20_r37_compare_positive(
        result.right_infinity_numerator, d) < 0;
    result.amplification_numerator = al_r20_r37_multiply(
        result.inverse_infinity_norm, d);
    result.amplification_denominator = al_r20_r63p_subtract(
        d, result.left_infinity_numerator);
    result.inverse_root = al_r20_r63q_vector_root(inverse);
    result.left_root = al_r20_r63q_vector_root(result.left);
    result.right_root = al_r20_r63q_vector_root(result.right);
    result.exact = result.left_products
            == dimension * dimension * dimension
        && result.right_products == dimension * dimension * dimension
        && result.residual_entries == entries
        && result.row_sum_terms == 2U * entries
        && result.inverse_row_terms == entries
        && result.asymmetry_terms == entries
        && (!result.left_contractive
            || al_r20_r63p_sign(result.amplification_denominator) > 0);
    std::ostringstream material;
    material << result.exact << ':' << result.left_contractive << ':'
        << result.right_contractive << ':' << result.dimension << ':'
        << result.left_products << ':' << result.right_products << ':'
        << result.residual_entries << ':' << result.row_sum_terms << ':'
        << result.inverse_row_terms << ':' << result.asymmetry_terms << ':'
        << al_r20_r37_dyadic_text(result.d) << ':'
        << al_r20_r37_dyadic_text(result.maximum_left_entry) << ':'
        << al_r20_r37_dyadic_text(result.maximum_right_entry) << ':'
        << al_r20_r37_dyadic_text(result.left_infinity_numerator) << ':'
        << al_r20_r37_dyadic_text(result.right_infinity_numerator) << ':'
        << al_r20_r37_dyadic_text(result.inverse_infinity_norm) << ':'
        << al_r20_r37_dyadic_text(result.maximum_asymmetry) << ':'
        << al_r20_r37_dyadic_text(result.amplification_numerator) << ':'
        << al_r20_r37_dyadic_text(result.amplification_denominator) << ':'
        << result.inverse_root << ':' << result.left_root << ':'
        << result.right_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63RExactAudit {
    bool exact = false;
    std::size_t conversions = 0U;
    ALR20R63RDefect defect;
    std::string conversion_root;
    std::string root;
};

ALR20R63RExactAudit al_r20_r63r_exact_audit(
    const ALR20R63RInverse& inverse,
    const ALR20R63QExactMatrix& matrix) {
    ALR20R63RExactAudit result;
    if (!inverse.exact || !matrix.exact
        || inverse.dimension != matrix.rows) return result;
    bool conversions_exact = true;
    std::vector<ALR20R37Dyadic> exact_inverse(inverse.value.size());
    for (std::size_t index = 0U; index < inverse.value.size(); ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(inverse.value[index]);
        conversions_exact = conversions_exact && conversion.exact;
        exact_inverse[index] = conversion.value;
        ++result.conversions;
    }
    result.conversion_root = al_r20_r63q_vector_root(exact_inverse);
    result.defect = conversions_exact
        ? al_r20_r63r_defect(exact_inverse, matrix.oracle_numerator,
            matrix.d, matrix.rows)
        : ALR20R63RDefect{};
    result.exact = conversions_exact && result.defect.exact
        && result.conversions == matrix.rows * matrix.rows;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + std::to_string(result.conversions) + ':' + result.conversion_root
        + ':' + result.defect.root);
    return result;
}

std::string al_r20_r63r_classify(
    bool apparatus, bool construction, bool left, bool right) {
    if (!apparatus) return "COMMON_OPERATOR_INVERSE_APPARATUS_REJECTED";
    if (!construction) return "COMMON_OPERATOR_INVERSE_CONSTRUCTION_REJECTED";
    if (!left) return "COMMON_OPERATOR_LEFT_NONCONTRACTIVE";
    if (!right) return "COMMON_OPERATOR_RIGHT_NONCONTRACTIVE";
    return "COMMON_OPERATOR_TWO_SIDED_CANDIDATE";
}

struct ALR20R63RControls {
    bool exact = false;
    bool diagonal = false;
    bool orientation = false;
    bool noncontractive = false;
    bool right_only = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63RControls al_r20_r63r_controls() {
    ALR20R63RControls result;
    const ALR20R37Dyadic zero = al_r20_r63p_control_dyadic(0);
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const ALR20R37Dyadic two = al_r20_r63p_control_dyadic(2);
    const ALR20R37Dyadic four = al_r20_r63p_control_dyadic(4);
    const ALR20R37Dyadic half = al_r20_r63p_control_dyadic(1, -1);
    const ALR20R37Dyadic quarter = al_r20_r63p_control_dyadic(1, -2);
    const std::vector<ALR20R37Dyadic> oracle{two, zero, zero, four};
    const std::vector<ALR20R37Dyadic> exact_inverse{
        half, zero, zero, quarter};
    const ALR20R63RDefect diagonal = al_r20_r63r_defect(
        exact_inverse, oracle, one, 2U);
    result.diagonal = diagonal.exact && diagonal.left_contractive
        && diagonal.right_contractive
        && diagonal.left_infinity_numerator.numerator == 0
        && diagonal.right_infinity_numerator.numerator == 0;
    const std::vector<ALR20R37Dyadic> asymmetric{
        half, quarter, zero, quarter};
    const ALR20R63RDefect oriented = al_r20_r63r_defect(
        asymmetric, oracle, one, 2U);
    result.orientation = oriented.exact
        && oriented.left_root != oriented.right_root
        && !oriented.left_contractive && oriented.right_contractive;
    const std::vector<ALR20R37Dyadic> bad_diagonal{
        one, zero, zero, quarter};
    const ALR20R63RDefect noncontractive = al_r20_r63r_defect(
        bad_diagonal, oracle, one, 2U);
    result.noncontractive = noncontractive.exact
        && !noncontractive.left_contractive;
    const std::vector<ALR20R37Dyadic> right_only_inverse{
        half, zero, quarter, quarter};
    const ALR20R63RDefect right_only = al_r20_r63r_defect(
        right_only_inverse, oracle, one, 2U);
    result.right_only = right_only.exact && right_only.left_contractive
        && !right_only.right_contractive;
    std::vector<ALR20R37Dyadic> mutated = diagonal.left;
    mutated[0U] = al_r20_r37_add(mutated[0U], quarter);
    result.mutation = al_r20_r63q_vector_root(mutated)
        != diagonal.left_root;
    result.classifier =
        al_r20_r63r_classify(false, true, true, true)
            == "COMMON_OPERATOR_INVERSE_APPARATUS_REJECTED"
        && al_r20_r63r_classify(true, false, true, true)
            == "COMMON_OPERATOR_INVERSE_CONSTRUCTION_REJECTED"
        && al_r20_r63r_classify(true, true, false, true)
            == "COMMON_OPERATOR_LEFT_NONCONTRACTIVE"
        && al_r20_r63r_classify(true, true, true, false)
            == "COMMON_OPERATOR_RIGHT_NONCONTRACTIVE"
        && al_r20_r63r_classify(true, true, true, true)
            == "COMMON_OPERATOR_TWO_SIDED_CANDIDATE";
    result.exact = result.diagonal && result.orientation
        && result.noncontractive && result.right_only && result.mutation
        && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + diagonal.root + ':' + oriented.root + ':' + noncontractive.root
        + ':' + right_only.root + ':' + al_r20_r63q_vector_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_common_operator_inverse_controls() {
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
    const ALR20R63RControls controls = al_r20_r63r_controls();
    const ALR20R63QControls parent_controls = al_r20_r63q_controls();
    const bool controls_exact = profile_exact && controls.exact
        && parent_controls.exact;
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
        && factor_capture.source_rows.size() == 102U
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
    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const Binary128 sigma = static_cast<Binary128>(1.0) / inverse_scale;
    const ALR20R63QAudit parent_audit = tangent.exact
        ? al_r20_r63q_audit(grams, tangent, factor_capture.projector,
            factor_capture.matrix, sigma)
        : ALR20R63QAudit{};
    const bool parent_r63q_exact = capture_exact && grams.exact
        && full_profile.exact && tangent.exact
        && parent_audit.exact
        && parent_audit.root == AL_R20_R63R_AUDIT_SHA256
        && parent_audit.matrix.root == AL_R20_R63R_MATRIX_SHA256
        && parent_audit.matrix.oracle_root == AL_R20_R63R_ORACLE_SHA256
        && parent_audit.matrix.tangent_inf_better
        && parent_audit.matrix.tangent_frobenius_better;
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows)
        && al_r20_r63g_index_root(permutation)
            == AL_R20_R63R_PERMUTATION_SHA256;
    std::vector<Binary128> permuted_transpose;
    if (parent_r63q_exact && permutation_exact) {
        permuted_transpose.resize(tangent.columns * tangent.rows);
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                permuted_transpose[scalar * tangent.rows + column] =
                    tangent.values[permutation[column] * tangent.columns
                        + scalar];
    }
    const ALR20R63GQFactor factor = !permuted_transpose.empty()
        ? al_r20_r63g_q_factor(
            permuted_transpose, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    const bool factor_exact = factor.exact
        && factor.root == AL_R20_R63R_FACTOR_SHA256;
    const ALR20R63RInverse inverse = factor_exact
        ? al_r20_r63r_inverse(factor.r, permutation, inverse_scale)
        : ALR20R63RInverse{};
    const ALR20R63RExactAudit inverse_audit = inverse.exact
        ? al_r20_r63r_exact_audit(inverse, parent_audit.matrix)
        : ALR20R63RExactAudit{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const ALR20R63RDefect& defect = inverse_audit.defect;
    const bool work_exact = inverse.column_solves == 102U
        && inverse.forward_terms == 525402U
        && inverse.backward_terms == 525402U
        && inverse.divisions == 20808U && inverse.writes == 10404U
        && inverse_audit.conversions == 10404U
        && defect.left_products == 1061208U
        && defect.right_products == 1061208U
        && defect.residual_entries == 10404U
        && defect.row_sum_terms == 20808U
        && defect.inverse_row_terms == 10404U
        && defect.asymmetry_terms == 10404U;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63q_exact && permutation_exact && factor_exact
        && lifecycle_exact;
    const bool construction_exact = inverse.exact && inverse_audit.exact
        && defect.exact && work_exact;
    const std::string route = al_r20_r63r_classify(apparatus_exact,
        construction_exact, defect.left_contractive,
        defect.right_contractive);
    const bool exact = apparatus_exact && construction_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63r-common-operator-inverse.v1|"
        << AL_R20_R63R_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << parent_audit.root << ':' << factor.root
        << ':' << inverse.root << ':' << inverse_audit.root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63r_common_operator_inverse.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63q_sha256\":\""
        << AL_R20_R63R_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63q_exact\":"
        << (parent_r63q_exact ? "true" : "false")
        << ",\"factor\":{\"exact\":" << (factor_exact ? "true" : "false")
        << ",\"root\":\"" << factor.root << "\"}"
        << ",\"inverse\":{\"exact\":" << (inverse.exact ? "true" : "false")
        << ",\"column_solves\":" << inverse.column_solves
        << ",\"root\":\"" << inverse.root << "\"}"
        << ",\"defect\":{\"exact\":" << (defect.exact ? "true" : "false")
        << ",\"left_contractive\":"
        << (defect.left_contractive ? "true" : "false")
        << ",\"right_contractive\":"
        << (defect.right_contractive ? "true" : "false")
        << ",\"maximum_left_entry\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            defect.maximum_left_entry, defect.d))
        << "\",\"maximum_right_entry\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            defect.maximum_right_entry, defect.d))
        << "\",\"left_infinity\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            defect.left_infinity_numerator, defect.d))
        << "\",\"right_infinity\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            defect.right_infinity_numerator, defect.d))
        << "\",\"inverse_infinity\":\""
        << al_binary128_text(al_r20_r63p_dyadic_projection(
            defect.inverse_infinity_norm))
        << "\",\"maximum_asymmetry\":\""
        << al_binary128_text(al_r20_r63p_dyadic_projection(
            defect.maximum_asymmetry))
        << "\",\"amplification_bound\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            defect.amplification_numerator,
            defect.amplification_denominator))
        << "\",\"inverse_root\":\"" << defect.inverse_root << "\""
        << ",\"left_root\":\"" << defect.left_root << "\""
        << ",\"right_root\":\"" << defect.right_root << "\""
        << ",\"root\":\"" << defect.root << "\"}"
        << ",\"inverse_audit_root\":\"" << inverse_audit.root << "\""
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"construction_exact\":"
        << (construction_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"qr_factorizations\":1,\"canonical_inverse_solves\":"
        << inverse.column_solves
        << ",\"exact_left_products\":" << defect.left_products
        << ",\"exact_right_products\":" << defect.right_products
        << ",\"physical_rhs_solves\":0,\"pcg_updates\":0"
        << ",\"iterative_refinements\":0,\"sparse_constructions\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"regularizations\":0"
        << ",\"replacement_applies\":0,\"following_transitions\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
