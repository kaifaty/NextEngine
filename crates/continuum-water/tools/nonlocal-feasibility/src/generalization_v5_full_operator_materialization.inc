namespace {

constexpr const char* AL_R20_R63Q_PARENT_SEMANTIC_SHA256 =
    "b7cae5137a2e9c3cd9acdd678044dab5fcf820172db15ea13981b443c765e3e1";
constexpr const char* AL_R20_R63Q_OBSERVATION_SHA256 =
    "a8e4311989c3c8d1e80d6f7d8b0fb840d2e35775de4f9198fad7e24ab124e5a5";
constexpr const char* AL_R20_R63Q_ARBITRATION_SHA256 =
    "c535acfe3323cdb83b5dfe30d572136138388d3efda2e32b3c108f3775d66196";

std::string al_r20_r63q_vector_root(
    const std::vector<ALR20R37Dyadic>& values) {
    std::ostringstream material;
    for (const ALR20R37Dyadic& value : values)
        material << al_r20_r37_dyadic_text(value) << ';';
    return sha256_hex(material.str());
}

struct ALR20R63QExactMatrix {
    bool exact = false;
    bool symmetric = false;
    bool tangent_inf_better = false;
    bool tangent_frobenius_better = false;
    std::size_t rows = 0U;
    std::size_t columns = 0U;
    std::size_t free_components = 0U;
    std::size_t radial_products = 0U;
    std::size_t upper_entries = 0U;
    std::size_t source_gram_products = 0U;
    std::size_t tangent_gram_products = 0U;
    std::size_t oracle_mirrors = 0U;
    std::size_t tangent_mirrors = 0U;
    std::size_t semantic_mirrors = 0U;
    std::size_t ordered_entries = 0U;
    std::size_t dense_wins = 0U;
    std::size_t tangent_wins = 0U;
    std::size_t ties = 0U;
    std::size_t dense_row_terms = 0U;
    std::size_t tangent_row_terms = 0U;
    std::size_t frobenius_terms = 0U;
    std::size_t oracle_row_terms = 0U;
    std::size_t semantic_row_terms = 0U;
    ALR20R37Dyadic s;
    ALR20R37Dyadic d;
    ALR20R37Dyadic sigma;
    ALR20R37Dyadic maximum_dense_error;
    ALR20R37Dyadic maximum_tangent_error;
    ALR20R37Dyadic dense_infinity_error;
    ALR20R37Dyadic tangent_infinity_error;
    ALR20R37Dyadic dense_frobenius_squared;
    ALR20R37Dyadic tangent_frobenius_squared;
    ALR20R37Dyadic oracle_infinity_norm;
    ALR20R37Dyadic semantic_infinity_defect;
    std::vector<ALR20R37Dyadic> radial;
    std::vector<ALR20R37Dyadic> oracle_numerator;
    std::vector<ALR20R37Dyadic> tangent_matrix;
    std::vector<ALR20R37Dyadic> semantic_numerator;
    std::vector<ALR20R37Dyadic> dense_error_numerator;
    std::vector<ALR20R37Dyadic> tangent_error_numerator;
    std::string oracle_root;
    std::string tangent_root;
    std::string semantic_root;
    std::string dense_error_root;
    std::string tangent_error_root;
    std::string root;
};

ALR20R63QExactMatrix al_r20_r63q_exact_matrix(
    const std::vector<ALR20R37Dyadic>& source,
    const std::vector<ALR20R37Dyadic>& tangent,
    const std::vector<ALR20R37Dyadic>& dense,
    const std::vector<ALR20R37Dyadic>& y,
    const std::vector<int>& mask,
    std::size_t rows, std::size_t columns,
    const ALR20R37Dyadic& d,
    const ALR20R37Dyadic& sigma) {
    ALR20R63QExactMatrix result;
    result.rows = rows;
    result.columns = columns;
    result.d = d;
    result.sigma = sigma;
    if (rows == 0U || columns == 0U
        || source.size() != rows * columns
        || tangent.size() != rows * columns
        || dense.size() != rows * rows || y.size() != columns
        || mask.size() != columns || al_r20_r63p_sign(d) <= 0
        || al_r20_r63p_sign(sigma) <= 0) return result;
    for (std::size_t scalar = 0U; scalar < columns; ++scalar)
        if (mask[scalar] == 0) {
            result.s = al_r20_r37_add(result.s,
                al_r20_r37_multiply(y[scalar], y[scalar]));
            ++result.free_components;
        }
    result.radial.resize(rows);
    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t scalar = 0U; scalar < columns; ++scalar)
            if (mask[scalar] == 0) {
                result.radial[row] = al_r20_r37_add(result.radial[row],
                    al_r20_r37_multiply(
                        source[row * columns + scalar], y[scalar]));
                ++result.radial_products;
            }
    const std::size_t entries = rows * rows;
    result.oracle_numerator.resize(entries);
    result.tangent_matrix.resize(entries);
    result.semantic_numerator.resize(entries);
    const ALR20R37Dyadic s_minus_d = al_r20_r63p_subtract(result.s, d);
    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t column = row; column < rows; ++column) {
            ALR20R37Dyadic source_gram;
            ALR20R37Dyadic tangent_gram;
            for (std::size_t scalar = 0U; scalar < columns; ++scalar) {
                if (mask[scalar] == 0) {
                    source_gram = al_r20_r37_add(source_gram,
                        al_r20_r37_multiply(
                            source[row * columns + scalar],
                            source[column * columns + scalar]));
                    ++result.source_gram_products;
                }
                tangent_gram = al_r20_r37_add(tangent_gram,
                    al_r20_r37_multiply(
                        tangent[row * columns + scalar],
                        tangent[column * columns + scalar]));
                ++result.tangent_gram_products;
            }
            const ALR20R37Dyadic radial_product = al_r20_r37_multiply(
                result.radial[row], result.radial[column]);
            const ALR20R37Dyadic oracle_inner = al_r20_r63p_subtract(
                al_r20_r37_multiply(source_gram, d), radial_product);
            const ALR20R37Dyadic oracle = al_r20_r37_multiply(
                sigma, oracle_inner);
            const ALR20R37Dyadic represented = al_r20_r37_multiply(
                sigma, tangent_gram);
            const ALR20R37Dyadic semantic = al_r20_r37_multiply(sigma,
                al_r20_r37_multiply(s_minus_d, radial_product));
            const std::size_t index = row * rows + column;
            result.oracle_numerator[index] = oracle;
            result.tangent_matrix[index] = represented;
            result.semantic_numerator[index] = semantic;
            if (row != column) {
                const std::size_t mirror = column * rows + row;
                result.oracle_numerator[mirror] = oracle;
                result.tangent_matrix[mirror] = represented;
                result.semantic_numerator[mirror] = semantic;
                ++result.oracle_mirrors;
                ++result.tangent_mirrors;
                ++result.semantic_mirrors;
            }
            ++result.upper_entries;
        }
    result.dense_error_numerator.resize(entries);
    result.tangent_error_numerator.resize(entries);
    std::vector<ALR20R37Dyadic> dense_row(rows);
    std::vector<ALR20R37Dyadic> tangent_row(rows);
    std::vector<ALR20R37Dyadic> oracle_row(rows);
    std::vector<ALR20R37Dyadic> semantic_row(rows);
    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t column = 0U; column < rows; ++column) {
            const std::size_t index = row * rows + column;
            const ALR20R37Dyadic dense_error = al_r20_r37_absolute(
                al_r20_r63p_subtract(
                    al_r20_r37_multiply(dense[index], d),
                    result.oracle_numerator[index]));
            const ALR20R37Dyadic tangent_error = al_r20_r37_absolute(
                al_r20_r63p_subtract(
                    al_r20_r37_multiply(result.tangent_matrix[index], d),
                    result.oracle_numerator[index]));
            result.dense_error_numerator[index] = dense_error;
            result.tangent_error_numerator[index] = tangent_error;
            const int comparison = al_r20_r37_compare_positive(
                dense_error, tangent_error);
            if (comparison < 0) ++result.dense_wins;
            else if (comparison > 0) ++result.tangent_wins;
            else ++result.ties;
            if (index == 0U || al_r20_r37_compare_positive(
                    dense_error, result.maximum_dense_error) > 0)
                result.maximum_dense_error = dense_error;
            if (index == 0U || al_r20_r37_compare_positive(
                    tangent_error, result.maximum_tangent_error) > 0)
                result.maximum_tangent_error = tangent_error;
            dense_row[row] = al_r20_r37_add(dense_row[row], dense_error);
            tangent_row[row] = al_r20_r37_add(
                tangent_row[row], tangent_error);
            oracle_row[row] = al_r20_r37_add(oracle_row[row],
                al_r20_r37_absolute(result.oracle_numerator[index]));
            semantic_row[row] = al_r20_r37_add(semantic_row[row],
                al_r20_r37_absolute(result.semantic_numerator[index]));
            result.dense_frobenius_squared = al_r20_r37_add(
                result.dense_frobenius_squared,
                al_r20_r37_multiply(dense_error, dense_error));
            result.tangent_frobenius_squared = al_r20_r37_add(
                result.tangent_frobenius_squared,
                al_r20_r37_multiply(tangent_error, tangent_error));
            ++result.ordered_entries;
            ++result.dense_row_terms;
            ++result.tangent_row_terms;
            result.frobenius_terms += 2U;
            ++result.oracle_row_terms;
            ++result.semantic_row_terms;
        }
    for (std::size_t row = 0U; row < rows; ++row) {
        if (row == 0U || al_r20_r37_compare_positive(
                dense_row[row], result.dense_infinity_error) > 0)
            result.dense_infinity_error = dense_row[row];
        if (row == 0U || al_r20_r37_compare_positive(
                tangent_row[row], result.tangent_infinity_error) > 0)
            result.tangent_infinity_error = tangent_row[row];
        if (row == 0U || al_r20_r37_compare_positive(
                oracle_row[row], result.oracle_infinity_norm) > 0)
            result.oracle_infinity_norm = oracle_row[row];
        if (row == 0U || al_r20_r37_compare_positive(
                semantic_row[row], result.semantic_infinity_defect) > 0)
            result.semantic_infinity_defect = semantic_row[row];
    }
    result.symmetric = true;
    for (std::size_t row = 0U; row < rows && result.symmetric; ++row)
        for (std::size_t column = 0U; column < rows; ++column) {
            const std::size_t left = row * rows + column;
            const std::size_t right = column * rows + row;
            result.symmetric = result.symmetric
                && al_r20_r63p_equal(
                    result.oracle_numerator[left],
                    result.oracle_numerator[right])
                && al_r20_r63p_equal(
                    result.tangent_matrix[left],
                    result.tangent_matrix[right])
                && al_r20_r63p_equal(
                    result.semantic_numerator[left],
                    result.semantic_numerator[right]);
        }
    result.tangent_inf_better = al_r20_r37_compare_positive(
        result.tangent_infinity_error,
        result.dense_infinity_error) < 0;
    result.tangent_frobenius_better = al_r20_r37_compare_positive(
        result.tangent_frobenius_squared,
        result.dense_frobenius_squared) < 0;
    result.oracle_root = al_r20_r63q_vector_root(result.oracle_numerator);
    result.tangent_root = al_r20_r63q_vector_root(result.tangent_matrix);
    result.semantic_root = al_r20_r63q_vector_root(
        result.semantic_numerator);
    result.dense_error_root = al_r20_r63q_vector_root(
        result.dense_error_numerator);
    result.tangent_error_root = al_r20_r63q_vector_root(
        result.tangent_error_numerator);
    const std::size_t expected_upper = rows * (rows + 1U) / 2U;
    const std::size_t expected_mirrors = rows * (rows - 1U) / 2U;
    result.exact = result.symmetric
        && result.free_components > 0U
        && result.radial_products == rows * result.free_components
        && result.upper_entries == expected_upper
        && result.source_gram_products
            == expected_upper * result.free_components
        && result.tangent_gram_products == expected_upper * columns
        && result.oracle_mirrors == expected_mirrors
        && result.tangent_mirrors == expected_mirrors
        && result.semantic_mirrors == expected_mirrors
        && result.ordered_entries == entries
        && result.dense_wins + result.tangent_wins + result.ties == entries
        && result.dense_row_terms == entries
        && result.tangent_row_terms == entries
        && result.frobenius_terms == 2U * entries
        && result.oracle_row_terms == entries
        && result.semantic_row_terms == entries;
    std::ostringstream material;
    material << result.exact << ':' << result.symmetric << ':'
        << result.tangent_inf_better << ':'
        << result.tangent_frobenius_better << ':' << result.rows << ':'
        << result.columns << ':' << result.free_components << ':'
        << result.radial_products << ':' << result.upper_entries << ':'
        << result.source_gram_products << ':'
        << result.tangent_gram_products << ':' << result.oracle_mirrors << ':'
        << result.tangent_mirrors << ':' << result.semantic_mirrors << ':'
        << result.ordered_entries << ':' << result.dense_wins << ':'
        << result.tangent_wins << ':' << result.ties << ':'
        << result.dense_row_terms << ':' << result.tangent_row_terms << ':'
        << result.frobenius_terms << ':' << result.oracle_row_terms << ':'
        << result.semantic_row_terms << ':'
        << al_r20_r37_dyadic_text(result.s) << ':'
        << al_r20_r37_dyadic_text(result.d) << ':'
        << al_r20_r37_dyadic_text(result.sigma) << ':'
        << al_r20_r37_dyadic_text(result.maximum_dense_error) << ':'
        << al_r20_r37_dyadic_text(result.maximum_tangent_error) << ':'
        << al_r20_r37_dyadic_text(result.dense_infinity_error) << ':'
        << al_r20_r37_dyadic_text(result.tangent_infinity_error) << ':'
        << al_r20_r37_dyadic_text(result.dense_frobenius_squared) << ':'
        << al_r20_r37_dyadic_text(result.tangent_frobenius_squared) << ':'
        << al_r20_r37_dyadic_text(result.oracle_infinity_norm) << ':'
        << al_r20_r37_dyadic_text(result.semantic_infinity_defect) << ':'
        << al_r20_r63q_vector_root(result.radial) << ':'
        << result.oracle_root << ':' << result.tangent_root << ':'
        << result.semantic_root << ':' << result.dense_error_root << ':'
        << result.tangent_error_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63QAudit {
    bool exact = false;
    std::size_t source_conversions = 0U;
    std::size_t tangent_conversions = 0U;
    std::size_t dense_conversions = 0U;
    std::size_t projector_conversions = 0U;
    std::size_t scalar_conversions = 0U;
    ALR20R63QExactMatrix matrix;
    std::string conversion_root;
    std::string root;
};

ALR20R63QAudit al_r20_r63q_audit(
    const ALR20R63EProjectorGrams& grams,
    const ALR20R63FTangentMatrix& tangent,
    const ALR20ProjectorState& projector,
    const std::vector<Binary128>& dense,
    Binary128 sigma_value) {
    ALR20R63QAudit result;
    const std::size_t rows = grams.rows;
    const std::size_t columns = grams.scalars;
    if (!grams.exact || !tangent.exact || !projector.exact
        || rows == 0U || tangent.rows != rows || tangent.columns != columns
        || dense.size() != rows * rows || projector.mask.size() != columns
        || projector.value.size() * 3U != columns) return result;
    bool conversions_exact = true;
    auto convert = [&conversions_exact](Binary128 value) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(value);
        conversions_exact = conversions_exact && conversion.exact;
        return conversion.value;
    };
    std::vector<ALR20R37Dyadic> source(rows * columns);
    std::vector<ALR20R37Dyadic> represented(rows * columns);
    std::vector<ALR20R37Dyadic> dense_exact(rows * rows);
    std::vector<ALR20R37Dyadic> y(columns);
    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t scalar = 0U; scalar < columns; ++scalar) {
            source[row * columns + scalar] = convert(al_r20_q_component(
                grams.source_rows[row][scalar / 3U],
                static_cast<int>(scalar % 3U)));
            represented[row * columns + scalar] = convert(
                tangent.values[row * columns + scalar]);
            ++result.source_conversions;
            ++result.tangent_conversions;
        }
    for (std::size_t index = 0U; index < dense.size(); ++index) {
        dense_exact[index] = convert(dense[index]);
        ++result.dense_conversions;
    }
    for (std::size_t scalar = 0U; scalar < columns; ++scalar)
        if (projector.mask[scalar] == 0) {
            y[scalar] = convert(al_r20_q_component(
                projector.value[scalar / 3U],
                static_cast<int>(scalar % 3U)));
            ++result.projector_conversions;
        }
    const ALR20R37Dyadic d = convert(projector.free_norm_squared);
    const ALR20R37Dyadic sigma = convert(sigma_value);
    result.scalar_conversions = 2U;
    result.matrix = conversions_exact
        ? al_r20_r63q_exact_matrix(source, represented, dense_exact, y,
            projector.mask, rows, columns, d, sigma)
        : ALR20R63QExactMatrix{};
    result.conversion_root = sha256_hex(
        al_r20_r63q_vector_root(source) + ':'
        + al_r20_r63q_vector_root(represented) + ':'
        + al_r20_r63q_vector_root(dense_exact) + ':'
        + al_r20_r63q_vector_root(y) + ':'
        + al_r20_r37_dyadic_text(d) + ':'
        + al_r20_r37_dyadic_text(sigma));
    result.exact = conversions_exact && result.matrix.exact
        && result.source_conversions == rows * columns
        && result.tangent_conversions == rows * columns
        && result.dense_conversions == rows * rows
        && result.projector_conversions == 174U
        && result.scalar_conversions == 2U;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + std::to_string(result.source_conversions) + ':'
        + std::to_string(result.tangent_conversions) + ':'
        + std::to_string(result.dense_conversions) + ':'
        + std::to_string(result.projector_conversions) + ':'
        + std::to_string(result.scalar_conversions) + ':'
        + result.conversion_root + ':' + result.matrix.root);
    return result;
}

std::string al_r20_r63q_classify(
    bool apparatus, bool oracle, bool candidate) {
    if (!apparatus) return "FULL_OPERATOR_MATERIALIZATION_APPARATUS_REJECTED";
    if (!oracle) return "EXACT_COMMON_OPERATOR_CONSTRUCTION_REJECTED";
    if (!candidate) return "TANGENT_GRAM_FULL_OPERATOR_REJECTED";
    return "TANGENT_GRAM_FULL_OPERATOR_CANDIDATE";
}

struct ALR20R63QControls {
    bool exact = false;
    bool oracle = false;
    bool tangent_candidate = false;
    bool tangent_rejected = false;
    bool symmetry = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63QControls al_r20_r63q_controls() {
    ALR20R63QControls result;
    const ALR20R37Dyadic zero = al_r20_r63p_control_dyadic(0);
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const ALR20R37Dyadic half = al_r20_r63p_control_dyadic(1, -1);
    const ALR20R37Dyadic three_quarters =
        al_r20_r63p_control_dyadic(3, -2);
    const ALR20R37Dyadic source_values[4] = {one, zero, zero, one};
    const ALR20R37Dyadic tangent_values[4] = {
        three_quarters, zero, zero, one};
    std::vector<ALR20R37Dyadic> source(
        std::begin(source_values), std::end(source_values));
    std::vector<ALR20R37Dyadic> tangent(
        std::begin(tangent_values), std::end(tangent_values));
    std::vector<ALR20R37Dyadic> dense_exact{
        three_quarters, zero, zero, one};
    const std::vector<ALR20R37Dyadic> y{half, zero};
    const std::vector<int> mask{0, 0};
    const ALR20R63QExactMatrix oracle = al_r20_r63q_exact_matrix(
        source, tangent, dense_exact, y, mask, 2U, 2U, one, one);
    result.oracle = oracle.exact && oracle.symmetric
        && al_r20_r63p_equal(
            oracle.oracle_numerator[0U], three_quarters)
        && oracle.oracle_numerator[1U].numerator == 0
        && oracle.oracle_numerator[2U].numerator == 0
        && al_r20_r63p_equal(oracle.oracle_numerator[3U], one)
        && al_r20_r63p_equal(oracle.semantic_numerator[0U],
            al_r20_r63p_control_dyadic(-3, -4));
    std::vector<ALR20R37Dyadic> dense_bad = dense_exact;
    dense_bad[0U] = zero;
    const ALR20R63QExactMatrix candidate = al_r20_r63q_exact_matrix(
        source, tangent, dense_bad, y, mask, 2U, 2U, one, one);
    result.tangent_candidate = candidate.exact
        && candidate.tangent_inf_better
        && candidate.tangent_frobenius_better;
    std::vector<ALR20R37Dyadic> tangent_bad = tangent;
    tangent_bad[0U] = al_r20_r63p_control_dyadic(2);
    const ALR20R63QExactMatrix rejected = al_r20_r63q_exact_matrix(
        source, tangent_bad, dense_exact, y, mask, 2U, 2U, one, one);
    result.tangent_rejected = rejected.exact
        && !rejected.tangent_inf_better
        && !rejected.tangent_frobenius_better;
    result.symmetry = oracle.oracle_mirrors == 1U
        && oracle.tangent_mirrors == 1U && oracle.semantic_mirrors == 1U
        && oracle.symmetric;
    std::vector<ALR20R37Dyadic> mutated = oracle.oracle_numerator;
    mutated[0U] = al_r20_r37_add(mutated[0U], half);
    result.mutation = al_r20_r63q_vector_root(mutated)
        != oracle.oracle_root;
    result.classifier =
        al_r20_r63q_classify(false, true, true)
            == "FULL_OPERATOR_MATERIALIZATION_APPARATUS_REJECTED"
        && al_r20_r63q_classify(true, false, true)
            == "EXACT_COMMON_OPERATOR_CONSTRUCTION_REJECTED"
        && al_r20_r63q_classify(true, true, false)
            == "TANGENT_GRAM_FULL_OPERATOR_REJECTED"
        && al_r20_r63q_classify(true, true, true)
            == "TANGENT_GRAM_FULL_OPERATOR_CANDIDATE";
    result.exact = result.oracle && result.tangent_candidate
        && result.tangent_rejected && result.symmetry && result.mutation
        && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + oracle.root + ':' + candidate.root + ':' + rejected.root + ':'
        + al_r20_r63q_vector_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_full_operator_materialization_controls() {
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
    const ALR20R63QControls controls = al_r20_r63q_controls();
    const ALR20R63PControls parent_controls = al_r20_r63p_controls();
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
    const ALR20R63EWitness witness = full_profile.exact
        ? al_r20_r63e_witness(
            grams, factor_capture.projector, full_profile.binary128)
        : ALR20R63EWitness{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const Binary128 sigma = static_cast<Binary128>(1.0) / inverse_scale;
    const ALR20R63PObservation parent_observation = tangent.exact
        && witness.exact
        ? al_r20_r63p_observe(grams, tangent, factor_capture.projector,
            witness, factor_capture.matrix, sigma)
        : ALR20R63PObservation{};
    const bool parent_r63p_exact = capture_exact && grams.exact
        && grams.full_root == AL_R20_R63P_FULL_MATRIX_SHA256
        && full_profile.exact && witness.exact
        && witness.root == AL_R20_R63P_WITNESS_SHA256
        && tangent.exact && tangent.value_root
            == AL_R20_R63N_TANGENT_VALUE_SHA256
        && parent_controls.root
            == "806d258c6882829c89eb023aa0e85288540efd05b65f80347e7f7dd5756def05"
        && parent_observation.exact
        && parent_observation.root == AL_R20_R63Q_OBSERVATION_SHA256
        && parent_observation.arbitration.root
            == AL_R20_R63Q_ARBITRATION_SHA256
        && parent_observation.arbitration.nonidempotent
        && !parent_observation.arbitration.structural
        && parent_observation.arbitration.common_oracle_selects_tangent
        && al_r20_r63p_classify(true,
            parent_observation.arbitration.nonidempotent,
            parent_observation.arbitration.structural)
            == "PROJECTOR_RANK_ONE_EXPLANATION_REJECTED";
    const ALR20R63QAudit audit = parent_r63p_exact
        ? al_r20_r63q_audit(grams, tangent, factor_capture.projector,
            factor_capture.matrix, sigma)
        : ALR20R63QAudit{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const ALR20R63QExactMatrix& matrix = audit.matrix;
    const bool work_exact = audit.source_conversions == 32130U
        && audit.tangent_conversions == 32130U
        && audit.dense_conversions == 10404U
        && audit.projector_conversions == 174U
        && audit.scalar_conversions == 2U
        && matrix.radial_products == 17748U
        && matrix.upper_entries == 5253U
        && matrix.source_gram_products == 914022U
        && matrix.tangent_gram_products == 1654695U
        && matrix.oracle_mirrors == 5151U
        && matrix.tangent_mirrors == 5151U
        && matrix.semantic_mirrors == 5151U
        && matrix.ordered_entries == 10404U
        && matrix.dense_row_terms == 10404U
        && matrix.tangent_row_terms == 10404U
        && matrix.frobenius_terms == 20808U
        && matrix.oracle_row_terms == 10404U
        && matrix.semantic_row_terms == 10404U;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63p_exact && lifecycle_exact;
    const bool oracle_exact = audit.exact && matrix.exact && work_exact;
    const bool candidate = oracle_exact && matrix.tangent_inf_better
        && matrix.tangent_frobenius_better
        && parent_observation.arbitration.common_oracle_selects_tangent;
    const std::string route = al_r20_r63q_classify(
        apparatus_exact, oracle_exact, candidate);
    const bool exact = apparatus_exact && oracle_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63q-full-operator-materialization.v1|"
        << AL_R20_R63Q_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << parent_observation.root << ':'
        << audit.root;
    const std::string result_sha = sha256_hex(semantic.str());
    const ALR20R37Dyadic d_squared = al_r20_r37_multiply(matrix.d, matrix.d);
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63q_full_operator_materialization.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63p_sha256\":\""
        << AL_R20_R63Q_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63p_exact\":"
        << (parent_r63p_exact ? "true" : "false")
        << ",\"matrix\":{\"exact\":" << (matrix.exact ? "true" : "false")
        << ",\"symmetric\":" << (matrix.symmetric ? "true" : "false")
        << ",\"dimension\":" << matrix.rows
        << ",\"entries\":" << matrix.ordered_entries
        << ",\"dense_wins\":" << matrix.dense_wins
        << ",\"tangent_wins\":" << matrix.tangent_wins
        << ",\"ties\":" << matrix.ties
        << ",\"maximum_dense_error\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            matrix.maximum_dense_error, matrix.d))
        << "\",\"maximum_tangent_error\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            matrix.maximum_tangent_error, matrix.d))
        << "\",\"dense_infinity_error\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            matrix.dense_infinity_error, matrix.d))
        << "\",\"tangent_infinity_error\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            matrix.tangent_infinity_error, matrix.d))
        << "\",\"dense_frobenius_squared\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            matrix.dense_frobenius_squared, d_squared))
        << "\",\"tangent_frobenius_squared\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            matrix.tangent_frobenius_squared, d_squared))
        << "\",\"oracle_infinity_norm\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            matrix.oracle_infinity_norm, matrix.d))
        << "\",\"semantic_infinity_defect\":\""
        << al_binary128_text(al_r20_r63p_fraction_projection(
            matrix.semantic_infinity_defect, d_squared))
        << "\",\"tangent_inf_better\":"
        << (matrix.tangent_inf_better ? "true" : "false")
        << ",\"tangent_frobenius_better\":"
        << (matrix.tangent_frobenius_better ? "true" : "false")
        << ",\"oracle_root\":\"" << matrix.oracle_root << "\""
        << ",\"tangent_root\":\"" << matrix.tangent_root << "\""
        << ",\"semantic_root\":\"" << matrix.semantic_root << "\""
        << ",\"dense_error_root\":\"" << matrix.dense_error_root << "\""
        << ",\"tangent_error_root\":\""
        << matrix.tangent_error_root << "\",\"root\":\""
        << matrix.root << "\"}"
        << ",\"audit_root\":\"" << audit.root << "\""
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"oracle_exact\":" << (oracle_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"exact_source_gram_products\":"
        << matrix.source_gram_products
        << ",\"exact_tangent_gram_products\":"
        << matrix.tangent_gram_products
        << ",\"rhs_solves\":0,\"inverse_constructions\":0"
        << ",\"factor_solves\":0,\"triangular_solves\":0,\"pcg_updates\":0"
        << ",\"sparse_constructions\":0,\"rank_actions\":0,\"row_drops\":0"
        << ",\"regularizations\":0,\"replacement_applies\":0"
        << ",\"following_transitions\":0,\"state_updates\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}

