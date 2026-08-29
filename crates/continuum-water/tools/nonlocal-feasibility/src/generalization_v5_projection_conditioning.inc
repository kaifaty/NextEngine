namespace {

constexpr const char* AL_R20_R63A_PARENT_SEMANTIC_SHA256 =
    "70f298d4e5931bda4a2e3d6d6ff6123f2739f992a084cf204653a01265411496";
constexpr const char* AL_R20_R63A_QQ_RIGHT_SHA256 =
    "23164539cab7325e17a1673e6a45abbbe3739d370cdba64c09de90d84b0a3539";
constexpr const char* AL_R20_R63A_QQ_LEFT_SHA256 =
    "0b611bbbfb6459b867a4536ceedadfd152994eab718b26e0a3051c1ceb072515";
constexpr const char* AL_R20_R63A_DD_RIGHT_SHA256 =
    "ec53b9a43ac0221abd9275ba6bf04e701b49c1f373b93f8a76f88fc8b349e303";
constexpr const char* AL_R20_R63A_DD_LEFT_SHA256 =
    "3e2328d7d9278c1a031678e5d307103f16e51d46067e7a370af16016542d3cdd";

struct ALR20R63AExactMatrix {
    bool exact = false;
    std::size_t dimension = 0U;
    std::vector<ALR20R37Dyadic> values;
    std::string root;
};

ALR20R63AExactMatrix al_r20_r63a_exact_matrix(
    const std::vector<Binary128>& values, std::size_t dimension) {
    ALR20R63AExactMatrix result;
    result.dimension = dimension;
    if (dimension == 0U || values.size() != dimension * dimension)
        return result;
    bool exact = true;
    std::ostringstream material;
    for (Binary128 value : values) {
        const ALR20R37Conversion converted =
            al_r20_r37_from_binary128(value);
        exact = exact && converted.exact;
        result.values.push_back(converted.value);
        material << al_r20_r37_dyadic_text(converted.value) << ';';
    }
    result.exact = exact;
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<ALR20R37Dyadic> al_r20_r63a_difference(
    const std::vector<ALR20R37Dyadic>& left,
    const std::vector<ALR20R37Dyadic>& right) {
    std::vector<ALR20R37Dyadic> difference;
    if (left.size() != right.size()) return difference;
    difference.reserve(left.size());
    for (std::size_t index = 0U; index < left.size(); ++index)
        difference.push_back(al_r20_r37_add(
            left[index], al_r20_r63_negate(right[index])));
    return difference;
}

std::vector<ALR20R37Dyadic> al_r20_r63a_sum(
    const std::vector<ALR20R37Dyadic>& left,
    const std::vector<ALR20R37Dyadic>& right) {
    std::vector<ALR20R37Dyadic> sum;
    if (left.size() != right.size()) return sum;
    sum.reserve(left.size());
    for (std::size_t index = 0U; index < left.size(); ++index)
        sum.push_back(al_r20_r37_add(left[index], right[index]));
    return sum;
}

bool al_r20_r63a_equal(
    const std::vector<ALR20R37Dyadic>& left,
    const std::vector<ALR20R37Dyadic>& right) {
    if (left.size() != right.size()) return false;
    for (std::size_t index = 0U; index < left.size(); ++index)
        if (left[index].numerator != right[index].numerator
            || left[index].exponent != right[index].exponent)
            return false;
    return true;
}

Binary128 al_r20_r63a_binary128_down(
    const ALR20R37Dyadic& positive) {
    if (positive.numerator == 0) return static_cast<Binary128>(0.0);
    const Binary128 upper = al_r20_r37_binary128_up(positive);
    const ALR20R37Conversion represented =
        al_r20_r37_from_binary128(upper);
    if (!represented.exact) return static_cast<Binary128>(0.0);
    return al_r20_r37_compare_positive(represented.value, positive) == 0
        ? upper : nextafterq(upper, static_cast<Binary128>(0.0));
}

Binary128 al_r20_r63a_ratio_up(
    const ALR20R37Dyadic& numerator,
    const ALR20R37Dyadic& denominator) {
    if (denominator.numerator == 0) return HUGE_VALQ;
    const Binary128 upper = al_r20_r37_binary128_up(numerator);
    const Binary128 lower = al_r20_r63a_binary128_down(denominator);
    if (!(lower > static_cast<Binary128>(0.0))) return HUGE_VALQ;
    return nextafterq(upper / lower, HUGE_VALQ);
}

struct ALR20R63ANorm {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t worst_row = 0U;
    ALR20R37Dyadic value;
    Binary128 outward = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R63ANorm al_r20_r63a_norm(
    const std::vector<ALR20R37Dyadic>& values,
    std::size_t dimension) {
    ALR20R63ANorm result;
    result.dimension = dimension;
    if (dimension == 0U || values.size() != dimension * dimension)
        return result;
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic row_sum;
        for (std::size_t column = 0U; column < dimension; ++column)
            row_sum = al_r20_r37_add(row_sum, al_r20_r37_absolute(
                values[row * dimension + column]));
        if (row == 0U
            || al_r20_r37_compare_positive(row_sum, result.value) > 0) {
            result.value = row_sum;
            result.worst_row = row;
        }
    }
    result.outward = al_r20_r37_binary128_up(result.value);
    const ALR20R37Conversion outward =
        al_r20_r37_from_binary128(result.outward);
    result.exact = outward.exact
        && al_r20_r37_compare_positive(outward.value, result.value) >= 0;
    std::ostringstream material;
    material << result.exact << ':' << dimension << ':' << result.worst_row
        << ':' << al_r20_r37_dyadic_text(result.value) << ':'
        << al_binary128_hex(result.outward);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63AFacts {
    bool exact = false;
    bool symmetric = false;
    std::size_t dimension = 0U;
    std::size_t nonzero_entries = 0U;
    std::size_t positive_diagonal = 0U;
    ALR20R63ANorm norm;
    ALR20R63ANorm asymmetry;
    ALR20R37Dyadic maximum_entry;
    ALR20R37Dyadic minimum_entry;
    ALR20R37Dyadic maximum_row;
    ALR20R37Dyadic minimum_row;
    ALR20R37Dyadic maximum_diagonal;
    ALR20R37Dyadic minimum_diagonal;
    Binary128 entry_ratio = static_cast<Binary128>(0.0);
    Binary128 row_ratio = static_cast<Binary128>(0.0);
    Binary128 diagonal_ratio = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R63AFacts al_r20_r63a_facts(
    const std::vector<ALR20R37Dyadic>& values,
    std::size_t dimension) {
    ALR20R63AFacts result;
    result.dimension = dimension;
    if (dimension == 0U || values.size() != dimension * dimension)
        return result;
    result.norm = al_r20_r63a_norm(values, dimension);
    bool have_entry = false;
    bool have_row = false;
    bool have_diagonal = false;
    std::vector<ALR20R37Dyadic> asymmetry;
    asymmetry.reserve(values.size());
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic row_norm;
        for (std::size_t column = 0U; column < dimension; ++column) {
            const ALR20R37Dyadic absolute = al_r20_r37_absolute(
                values[row * dimension + column]);
            row_norm = al_r20_r37_add(row_norm, absolute);
            if (absolute.numerator != 0) {
                ++result.nonzero_entries;
                if (!have_entry
                    || al_r20_r37_compare_positive(
                        absolute, result.minimum_entry) < 0)
                    result.minimum_entry = absolute;
                if (!have_entry
                    || al_r20_r37_compare_positive(
                        absolute, result.maximum_entry) > 0)
                    result.maximum_entry = absolute;
                have_entry = true;
            }
            asymmetry.push_back(al_r20_r37_add(
                values[row * dimension + column],
                al_r20_r63_negate(values[column * dimension + row])));
        }
        if (row_norm.numerator != 0) {
            if (!have_row || al_r20_r37_compare_positive(
                    row_norm, result.minimum_row) < 0)
                result.minimum_row = row_norm;
            if (!have_row || al_r20_r37_compare_positive(
                    row_norm, result.maximum_row) > 0)
                result.maximum_row = row_norm;
            have_row = true;
        }
        const ALR20R37Dyadic diagonal =
            values[row * dimension + row];
        const ALR20R37Dyadic diagonal_absolute =
            al_r20_r37_absolute(diagonal);
        if (diagonal.numerator != 0 && !(diagonal.numerator < 0))
            ++result.positive_diagonal;
        if (diagonal_absolute.numerator != 0) {
            if (!have_diagonal || al_r20_r37_compare_positive(
                    diagonal_absolute, result.minimum_diagonal) < 0)
                result.minimum_diagonal = diagonal_absolute;
            if (!have_diagonal || al_r20_r37_compare_positive(
                    diagonal_absolute, result.maximum_diagonal) > 0)
                result.maximum_diagonal = diagonal_absolute;
            have_diagonal = true;
        }
    }
    result.asymmetry = al_r20_r63a_norm(asymmetry, dimension);
    result.symmetric = result.asymmetry.exact
        && result.asymmetry.value.numerator == 0;
    result.entry_ratio = have_entry
        ? al_r20_r63a_ratio_up(result.maximum_entry, result.minimum_entry)
        : static_cast<Binary128>(0.0);
    result.row_ratio = have_row
        ? al_r20_r63a_ratio_up(result.maximum_row, result.minimum_row)
        : static_cast<Binary128>(0.0);
    result.diagonal_ratio = have_diagonal
        ? al_r20_r63a_ratio_up(
            result.maximum_diagonal, result.minimum_diagonal)
        : static_cast<Binary128>(0.0);
    result.exact = result.norm.exact && result.asymmetry.exact
        && have_entry && have_row
        && finiteq(result.entry_ratio) != 0
        && finiteq(result.row_ratio) != 0
        && finiteq(result.diagonal_ratio) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.symmetric << ':'
        << dimension << ':' << result.nonzero_entries << ':'
        << result.positive_diagonal << ':' << result.norm.root << ':'
        << result.asymmetry.root << ':'
        << al_r20_r37_dyadic_text(result.maximum_entry) << ':'
        << al_r20_r37_dyadic_text(result.minimum_entry) << ':'
        << al_r20_r37_dyadic_text(result.maximum_row) << ':'
        << al_r20_r37_dyadic_text(result.minimum_row) << ':'
        << al_r20_r37_dyadic_text(result.maximum_diagonal) << ':'
        << al_r20_r37_dyadic_text(result.minimum_diagonal) << ':'
        << al_binary128_hex(result.entry_ratio) << ':'
        << al_binary128_hex(result.row_ratio) << ':'
        << al_binary128_hex(result.diagonal_ratio);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ADecomposition {
    bool exact = false;
    bool closure_first = false;
    bool closure_second = false;
    ALR20R37ExactResidual qq;
    ALR20R37ExactResidual matrix_only;
    ALR20R37ExactResidual inverse_only;
    ALR20R37ExactResidual combined;
    ALR20R63ANorm matrix_first;
    ALR20R63ANorm inverse_after;
    ALR20R63ANorm inverse_first;
    ALR20R63ANorm matrix_after;
    ALR20R63ANorm matrix_maximum;
    ALR20R63ANorm inverse_maximum;
    std::string root;
};

ALR20R63ANorm al_r20_r63a_max_norm(
    const ALR20R63ANorm& left, const ALR20R63ANorm& right) {
    return al_r20_r37_compare_positive(left.value, right.value) >= 0
        ? left : right;
}

ALR20R63ADecomposition al_r20_r63a_decomposition(
    const std::vector<Binary128>& matrix_q,
    const std::vector<Binary128>& matrix_d,
    const std::vector<Binary128>& inverse_q,
    const std::vector<Binary128>& inverse_d,
    std::size_t dimension,
    bool right_order) {
    ALR20R63ADecomposition result;
    if (right_order) {
        result.qq = al_r20_r37_exact_residual(
            matrix_q, inverse_q, dimension);
        result.matrix_only = al_r20_r37_exact_residual(
            matrix_d, inverse_q, dimension);
        result.inverse_only = al_r20_r37_exact_residual(
            matrix_q, inverse_d, dimension);
        result.combined = al_r20_r37_exact_residual(
            matrix_d, inverse_d, dimension);
    } else {
        result.qq = al_r20_r37_exact_residual(
            inverse_q, matrix_q, dimension);
        result.matrix_only = al_r20_r37_exact_residual(
            inverse_q, matrix_d, dimension);
        result.inverse_only = al_r20_r37_exact_residual(
            inverse_d, matrix_q, dimension);
        result.combined = al_r20_r37_exact_residual(
            inverse_d, matrix_d, dimension);
    }
    const std::vector<ALR20R37Dyadic> matrix_first =
        al_r20_r63a_difference(
            result.matrix_only.residual, result.qq.residual);
    const std::vector<ALR20R37Dyadic> inverse_after =
        al_r20_r63a_difference(
            result.combined.residual, result.matrix_only.residual);
    const std::vector<ALR20R37Dyadic> inverse_first =
        al_r20_r63a_difference(
            result.inverse_only.residual, result.qq.residual);
    const std::vector<ALR20R37Dyadic> matrix_after =
        al_r20_r63a_difference(
            result.combined.residual, result.inverse_only.residual);
    result.matrix_first = al_r20_r63a_norm(matrix_first, dimension);
    result.inverse_after = al_r20_r63a_norm(inverse_after, dimension);
    result.inverse_first = al_r20_r63a_norm(inverse_first, dimension);
    result.matrix_after = al_r20_r63a_norm(matrix_after, dimension);
    result.matrix_maximum = al_r20_r63a_max_norm(
        result.matrix_first, result.matrix_after);
    result.inverse_maximum = al_r20_r63a_max_norm(
        result.inverse_first, result.inverse_after);
    result.closure_first = al_r20_r63a_equal(result.combined.residual,
        al_r20_r63a_sum(result.qq.residual,
            al_r20_r63a_sum(matrix_first, inverse_after)));
    result.closure_second = al_r20_r63a_equal(result.combined.residual,
        al_r20_r63a_sum(result.qq.residual,
            al_r20_r63a_sum(inverse_first, matrix_after)));
    result.exact = result.qq.exact && result.matrix_only.exact
        && result.inverse_only.exact && result.combined.exact
        && result.matrix_first.exact && result.inverse_after.exact
        && result.inverse_first.exact && result.matrix_after.exact
        && result.closure_first && result.closure_second;
    std::ostringstream material;
    material << result.exact << ':' << result.closure_first << ':'
        << result.closure_second << ':' << result.qq.root << ':'
        << result.matrix_only.root << ':' << result.inverse_only.root << ':'
        << result.combined.root << ':' << result.matrix_first.root << ':'
        << result.inverse_after.root << ':' << result.inverse_first.root << ':'
        << result.matrix_after.root;
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_r63a_at_least_factor(
    const ALR20R37Dyadic& left,
    const ALR20R37Dyadic& right,
    std::int64_t factor) {
    const ALR20R37Dyadic scaled = al_r20_r37_multiply(
        right, {ALR20R37Integer(factor), 0});
    return al_r20_r37_compare_positive(left, scaled) >= 0;
}

std::string al_r20_r63a_classification(
    const ALR20R63ADecomposition& right,
    const ALR20R63ADecomposition& left) {
    const bool any_inverse = right.inverse_maximum.value.numerator != 0
        || left.inverse_maximum.value.numerator != 0;
    const bool any_matrix = right.matrix_maximum.value.numerator != 0
        || left.matrix_maximum.value.numerator != 0;
    const bool inverse_dominant = any_inverse
        && al_r20_r63a_at_least_factor(right.inverse_maximum.value,
            right.matrix_maximum.value, 16)
        && al_r20_r63a_at_least_factor(left.inverse_maximum.value,
            left.matrix_maximum.value, 16);
    const bool matrix_dominant = any_matrix
        && al_r20_r63a_at_least_factor(right.matrix_maximum.value,
            right.inverse_maximum.value, 16)
        && al_r20_r63a_at_least_factor(left.matrix_maximum.value,
            left.inverse_maximum.value, 16);
    if (inverse_dominant && !matrix_dominant)
        return "INVERSE_PROJECTION_DOMINATES";
    if (matrix_dominant && !inverse_dominant)
        return "MATRIX_PROJECTION_DOMINATES";
    return "MIXED_PROJECTION_CONTRIBUTION";
}

struct ALR20R63AControls {
    bool exact = false;
    bool matrix_class = false;
    bool inverse_class = false;
    bool mixed_class = false;
    bool mutation_rejected = false;
    std::string root;
};

ALR20R63AControls al_r20_r63a_controls() {
    ALR20R63AControls result;
    const Binary128 one = static_cast<Binary128>(1.0);
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 epsilon = ldexpq(one, -10);
    const std::vector<Binary128> identity{one, zero, zero, one};
    const std::vector<Binary128> matrix_changed{
        one + epsilon, zero, zero, one};
    const std::vector<Binary128> inverse_changed{
        one - epsilon, zero, zero, one};
    const ALR20R63ADecomposition matrix_right = al_r20_r63a_decomposition(
        identity, matrix_changed, identity, identity, 2U, true);
    const ALR20R63ADecomposition matrix_left = al_r20_r63a_decomposition(
        identity, matrix_changed, identity, identity, 2U, false);
    const ALR20R63ADecomposition inverse_right = al_r20_r63a_decomposition(
        identity, identity, identity, inverse_changed, 2U, true);
    const ALR20R63ADecomposition inverse_left = al_r20_r63a_decomposition(
        identity, identity, identity, inverse_changed, 2U, false);
    const ALR20R63ADecomposition mixed_right = al_r20_r63a_decomposition(
        identity, matrix_changed, identity, inverse_changed, 2U, true);
    const ALR20R63ADecomposition mixed_left = al_r20_r63a_decomposition(
        identity, matrix_changed, identity, inverse_changed, 2U, false);
    result.matrix_class = matrix_right.exact && matrix_left.exact
        && al_r20_r63a_classification(matrix_right, matrix_left)
            == "MATRIX_PROJECTION_DOMINATES";
    result.inverse_class = inverse_right.exact && inverse_left.exact
        && al_r20_r63a_classification(inverse_right, inverse_left)
            == "INVERSE_PROJECTION_DOMINATES";
    result.mixed_class = mixed_right.exact && mixed_left.exact
        && al_r20_r63a_classification(mixed_right, mixed_left)
            == "MIXED_PROJECTION_CONTRIBUTION";
    std::vector<ALR20R37Dyadic> mutated = mixed_right.qq.residual;
    if (!mutated.empty())
        mutated[0U] = al_r20_r37_add(
            mutated[0U], {ALR20R37Integer(1), 0});
    const std::vector<ALR20R37Dyadic> first = al_r20_r63a_difference(
        mixed_right.matrix_only.residual, mixed_right.qq.residual);
    const std::vector<ALR20R37Dyadic> second = al_r20_r63a_difference(
        mixed_right.combined.residual,
        mixed_right.matrix_only.residual);
    result.mutation_rejected = !al_r20_r63a_equal(
        mixed_right.combined.residual,
        al_r20_r63a_sum(mutated, al_r20_r63a_sum(first, second)));
    result.exact = result.matrix_class && result.inverse_class
        && result.mixed_class && result.mutation_rejected;
    std::ostringstream material;
    material << result.exact << ':' << result.matrix_class << ':'
        << result.inverse_class << ':' << result.mixed_class << ':'
        << result.mutation_rejected << ':' << matrix_right.root << ':'
        << matrix_left.root << ':' << inverse_right.root << ':'
        << inverse_left.root << ':' << mixed_right.root << ':'
        << mixed_left.root;
    result.root = sha256_hex(material.str());
    return result;
}

ALR20R37Dyadic al_r20_r63a_proxy(
    const ALR20R63AFacts& matrix,
    const ALR20R63AFacts& inverse) {
    return al_r20_r37_multiply(matrix.norm.value, inverse.norm.value);
}

void al_r20_r63a_write_facts(
    std::ostringstream& report,
    const char* name,
    const ALR20R63AFacts& facts) {
    report << "\"" << name << "\":{\"exact\":"
        << (facts.exact ? "true" : "false")
        << ",\"symmetric\":" << (facts.symmetric ? "true" : "false")
        << ",\"positive_diagonal\":" << facts.positive_diagonal
        << ",\"nonzero_entries\":" << facts.nonzero_entries
        << ",\"norm_outward\":\""
        << al_binary128_text(facts.norm.outward)
        << "\",\"asymmetry_outward\":\""
        << al_binary128_text(facts.asymmetry.outward)
        << "\",\"entry_ratio\":\""
        << al_binary128_text(facts.entry_ratio)
        << "\",\"row_ratio\":\""
        << al_binary128_text(facts.row_ratio)
        << "\",\"diagonal_ratio\":\""
        << al_binary128_text(facts.diagonal_ratio)
        << "\",\"root\":\"" << facts.root << "\"}";
}

void al_r20_r63a_write_decomposition(
    std::ostringstream& report,
    const char* name,
    const ALR20R63ADecomposition& value) {
    report << "\"" << name << "\":{\"exact\":"
        << (value.exact ? "true" : "false")
        << ",\"closure_first\":"
        << (value.closure_first ? "true" : "false")
        << ",\"closure_second\":"
        << (value.closure_second ? "true" : "false")
        << ",\"qq_rho\":\"" << al_binary128_text(value.qq.outward_norm)
        << "\",\"matrix_only_rho\":\""
        << al_binary128_text(value.matrix_only.outward_norm)
        << "\",\"inverse_only_rho\":\""
        << al_binary128_text(value.inverse_only.outward_norm)
        << "\",\"combined_rho\":\""
        << al_binary128_text(value.combined.outward_norm)
        << "\",\"matrix_first\":\""
        << al_binary128_text(value.matrix_first.outward)
        << "\",\"inverse_after\":\""
        << al_binary128_text(value.inverse_after.outward)
        << "\",\"inverse_first\":\""
        << al_binary128_text(value.inverse_first.outward)
        << "\",\"matrix_after\":\""
        << al_binary128_text(value.matrix_after.outward)
        << "\",\"matrix_maximum\":\""
        << al_binary128_text(value.matrix_maximum.outward)
        << "\",\"inverse_maximum\":\""
        << al_binary128_text(value.inverse_maximum.outward)
        << "\",\"qq_root\":\"" << value.qq.root
        << "\",\"matrix_only_root\":\"" << value.matrix_only.root
        << "\",\"inverse_only_root\":\"" << value.inverse_only.root
        << "\",\"combined_root\":\"" << value.combined.root
        << "\",\"root\":\"" << value.root << "\"}";
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_projection_conditioning_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113 && sizeof(double) == 8U
        && DBL_MANT_DIG == 53 && std::fegetround() == FE_TONEAREST;
    const ALR20R63AControls controls = al_r20_r63a_controls();
    const bool controls_exact = profile_exact && controls.exact
        && al_r20_r37_dyadic_controls();
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
            && problem.problem_root == AL_R20_R60_PROBLEM_SHA256;
    }
    ALR20R60Capture capture;
    ALR20R55SlopeTrace slope;
    ALR20R35CaseAudit oblique;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r60_capture = &capture;
        al_r20_verified_solution_refiner = &al_r20_r60_capture_refiner;
        al_r20_r55_slope_trace = &slope;
        al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
        oblique = al_r20_r35_case(problem);
        al_r20_slope_error_refiner = nullptr;
        al_r20_r55_slope_trace = nullptr;
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r60_capture = nullptr;
        solver_executions = 1U;
    }
    const bool capture_exact = parent_exact && hook_slot_clear
        && al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && capture.calls == 1U && capture.dimension == AL_R20_R63_DIMENSION
        && capture.material_root == AL_R20_R63_MATERIAL_SHA256
        && capture.matrix_root == AL_R20_R63_MATRIX_SHA256
        && capture.inverse_root == AL_R20_R63_INVERSE_SHA256
        && capture.rhs_root == AL_R20_R63_RHS_SHA256
        && capture.solution_root == AL_R20_R63_SOLUTION_SHA256
        && slope.calls == 0U && slope.inverse_audits == 0U;

    std::vector<double> matrix_d_values;
    std::vector<double> inverse_d_values;
    if (capture_exact) {
        matrix_d_values.reserve(capture.matrix.size());
        inverse_d_values.reserve(capture.inverse.size());
        for (Binary128 value : capture.matrix)
            matrix_d_values.push_back(static_cast<double>(value));
        for (Binary128 value : capture.inverse)
            inverse_d_values.push_back(static_cast<double>(value));
    }
    const std::vector<Binary128> matrix_d =
        al_r20_r63_promote(matrix_d_values);
    const std::vector<Binary128> inverse_d =
        al_r20_r63_promote(inverse_d_values);
    const bool projection_exact = capture_exact
        && matrix_d.size() == capture.matrix.size()
        && inverse_d.size() == capture.inverse.size()
        && al_r20_double_root(matrix_d_values)
            == "83c5bf929d36fc0c578e14c18f71b30305f7455348b98cc09939c78e41e63838"
        && al_r20_double_root(inverse_d_values)
            == "9b5f37e184b15295b4cedf04dc145eae8c022aeb83f795724e1da0c442a233f5";

    const ALR20R63ADecomposition right = projection_exact
        ? al_r20_r63a_decomposition(capture.matrix, matrix_d,
            capture.inverse, inverse_d, capture.dimension, true)
        : ALR20R63ADecomposition{};
    const ALR20R63ADecomposition left = projection_exact
        ? al_r20_r63a_decomposition(capture.matrix, matrix_d,
            capture.inverse, inverse_d, capture.dimension, false)
        : ALR20R63ADecomposition{};
    const bool parent_roots_exact = right.qq.root
            == AL_R20_R63A_QQ_RIGHT_SHA256
        && left.qq.root == AL_R20_R63A_QQ_LEFT_SHA256
        && right.combined.root == AL_R20_R63A_DD_RIGHT_SHA256
        && left.combined.root == AL_R20_R63A_DD_LEFT_SHA256;

    const ALR20R63AExactMatrix aq =
        al_r20_r63a_exact_matrix(capture.matrix, capture.dimension);
    const ALR20R63AExactMatrix ad =
        al_r20_r63a_exact_matrix(matrix_d, capture.dimension);
    const ALR20R63AExactMatrix xq =
        al_r20_r63a_exact_matrix(capture.inverse, capture.dimension);
    const ALR20R63AExactMatrix xd =
        al_r20_r63a_exact_matrix(inverse_d, capture.dimension);
    const std::vector<ALR20R37Dyadic> delta_a =
        al_r20_r63a_difference(aq.values, ad.values);
    const std::vector<ALR20R37Dyadic> delta_x =
        al_r20_r63a_difference(xq.values, xd.values);
    const ALR20R63AFacts aq_facts =
        al_r20_r63a_facts(aq.values, capture.dimension);
    const ALR20R63AFacts ad_facts =
        al_r20_r63a_facts(ad.values, capture.dimension);
    const ALR20R63AFacts xq_facts =
        al_r20_r63a_facts(xq.values, capture.dimension);
    const ALR20R63AFacts xd_facts =
        al_r20_r63a_facts(xd.values, capture.dimension);
    const ALR20R63AFacts delta_a_facts =
        al_r20_r63a_facts(delta_a, capture.dimension);
    const ALR20R63AFacts delta_x_facts =
        al_r20_r63a_facts(delta_x, capture.dimension);
    const bool facts_exact = aq.exact && ad.exact && xq.exact && xd.exact
        && aq_facts.exact && ad_facts.exact && xq_facts.exact
        && xd_facts.exact && delta_a_facts.exact && delta_x_facts.exact;
    const ALR20R37Dyadic qq_proxy = al_r20_r63a_proxy(aq_facts, xq_facts);
    const ALR20R37Dyadic dd_proxy = al_r20_r63a_proxy(ad_facts, xd_facts);
    const ALR20R37Dyadic unit{ALR20R37Integer(1), -53};
    const Binary128 qq_proxy_outward = al_r20_r37_binary128_up(qq_proxy);
    const Binary128 dd_proxy_outward = al_r20_r37_binary128_up(dd_proxy);
    const Binary128 qq_unit_proxy_outward = al_r20_r37_binary128_up(
        al_r20_r37_multiply(qq_proxy, unit));
    const Binary128 dd_unit_proxy_outward = al_r20_r37_binary128_up(
        al_r20_r37_multiply(dd_proxy, unit));

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = right.exact && left.exact;
    const bool original_contractive = right.qq.contractive
        && left.qq.contractive;
    const bool combined_noncontractive = !right.combined.contractive
        && !left.combined.contractive;
    const std::string classification =
        al_r20_r63a_classification(right, left);
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_exact && capture_exact && projection_exact
        && parent_roots_exact && work_exact && facts_exact && lifecycle_exact;
    const bool exact = apparatus_exact && original_contractive
        && combined_noncontractive;
    std::string route = classification;
    if (!profile_exact || !controls_exact)
        route = "PROJECTION_DECOMPOSITION_CONTROL_REJECTED";
    else if (!parent_exact || !capture_exact || !projection_exact)
        route = "PROJECTION_DECOMPOSITION_PARENT_REJECTED";
    else if (!parent_roots_exact || !work_exact || !facts_exact
        || !lifecycle_exact)
        route = "PROJECTION_DECOMPOSITION_APPARATUS_REJECTED";
    else if (!original_contractive)
        route = "PROJECTION_DECOMPOSITION_ORIGINAL_REJECTED";
    else if (!combined_noncontractive)
        route = "PROJECTION_DECOMPOSITION_COMBINED_REJECTED";

    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r63a-projection-conditioning.v1|"
        << AL_R20_R63A_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << capture.material_root << ':'
        << controls.root << ':' << right.root << ':' << left.root << ':'
        << aq_facts.root << ':' << ad_facts.root << ':' << xq_facts.root
        << ':' << xd_facts.root << ':' << delta_a_facts.root << ':'
        << delta_x_facts.root << ':' << al_binary128_hex(qq_proxy_outward)
        << ':' << al_binary128_hex(dd_proxy_outward);
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63a_projection_conditioning.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63_sha256\":\""
        << AL_R20_R63A_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"matrix_class\":"
        << (controls.matrix_class ? "true" : "false")
        << ",\"inverse_class\":"
        << (controls.inverse_class ? "true" : "false")
        << ",\"mixed_class\":"
        << (controls.mixed_class ? "true" : "false")
        << ",\"mutation_rejected\":"
        << (controls.mutation_rejected ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"capture_exact\":" << (capture_exact ? "true" : "false")
        << ",\"dimension\":" << capture.dimension
        << ",\"case_root\":\"" << oblique.root
        << "\",\"material_root\":\"" << capture.material_root << "\""
        << ",\"parent_roots_exact\":"
        << (parent_roots_exact ? "true" : "false") << ',';
    al_r20_r63a_write_decomposition(report, "right", right);
    report << ',';
    al_r20_r63a_write_decomposition(report, "left", left);
    report << ",\"facts\":{";
    al_r20_r63a_write_facts(report, "matrix_q", aq_facts);
    report << ',';
    al_r20_r63a_write_facts(report, "matrix_d", ad_facts);
    report << ',';
    al_r20_r63a_write_facts(report, "inverse_q", xq_facts);
    report << ',';
    al_r20_r63a_write_facts(report, "inverse_d", xd_facts);
    report << ',';
    al_r20_r63a_write_facts(report, "matrix_projection_delta", delta_a_facts);
    report << ',';
    al_r20_r63a_write_facts(report, "inverse_projection_delta", delta_x_facts);
    report << "}"
        << ",\"conditioning_proxy\":{\"qq\":\""
        << al_binary128_text(qq_proxy_outward)
        << "\",\"dd\":\"" << al_binary128_text(dd_proxy_outward)
        << "\",\"unit_qq\":\""
        << al_binary128_text(qq_unit_proxy_outward)
        << "\",\"unit_dd\":\""
        << al_binary128_text(dd_unit_proxy_outward) << "\"}"
        << ",\"classification\":\"" << classification << "\""
        << ",\"original_contractive\":"
        << (original_contractive ? "true" : "false")
        << ",\"combined_noncontractive\":"
        << (combined_noncontractive ? "true" : "false")
        << ",\"work\":{\"exact_residual_products\":8"
        << ",\"dot2_candidate_dots\":0,\"factorizations\":0"
        << ",\"inverse_columns\":0,\"center_iterations\":0}"
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"replacement_applies\":0,\"state_updates\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
