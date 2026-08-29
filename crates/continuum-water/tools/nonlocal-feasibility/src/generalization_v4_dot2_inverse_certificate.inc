namespace {

constexpr const char* AL_R20_R37_SEMANTIC_SHA256 =
    "e9e61c01d6ae9ebe07d7ef707dfb5121f352b1e2df85f8e2a5160d9015a598ff";
constexpr const char* AL_R20_R37_MATRIX_SHA256 =
    "45e2c92bcba77ffd7a19673172e18de8b21f3d4d7f29f6ea13db326533b59bef";
constexpr const char* AL_R20_R37_FACTOR_SHA256 =
    "1c159edbd5b3349a1e07ebcaa08bd63a7c89786d069cb81a70735cc38cadbbf0";
constexpr const char* AL_R20_R37_INVERSE_SHA256 =
    "8ec902ee7265794ca40a6a46624812a6df53d8f4e702620a7122c306c15b5803";
constexpr const char* AL_R20_R37_RESIDUAL_SHA256 =
    "d6bd19f51ecd75a48eef41cc34b59b052b373958af428d4d5885d5e96201dcfa";
constexpr const char* AL_R20_R37_AUDIT_SHA256 =
    "eb9d2fb361511f613ab70bfd0a6ec7d85d3b310027d8eed567ba7a6d4812fefe";

bool al_r20_r38_normal_or_zero(Binary128 value) {
    return finiteq(value) != 0
        && (value == static_cast<Binary128>(0.0)
            || fabsq(value) >= ldexpq(
                static_cast<Binary128>(1.0), -16382));
}

Binary128 al_r20_r38_up_add(Binary128 left, Binary128 right) {
    if (left == static_cast<Binary128>(0.0)
        && right == static_cast<Binary128>(0.0))
        return static_cast<Binary128>(0.0);
    return nextafterq(left + right, HUGE_VALQ);
}

Binary128 al_r20_r38_up_multiply(Binary128 left, Binary128 right) {
    if (left == static_cast<Binary128>(0.0)
        || right == static_cast<Binary128>(0.0))
        return static_cast<Binary128>(0.0);
    return nextafterq(left * right, HUGE_VALQ);
}

Binary128 al_r20_r38_up_divide(Binary128 numerator, Binary128 denominator) {
    if (numerator == static_cast<Binary128>(0.0))
        return static_cast<Binary128>(0.0);
    return nextafterq(numerator / denominator, HUGE_VALQ);
}

struct ALR20R38Pair {
    bool exact = false;
    bool no_underflow = false;
    Binary128 high = static_cast<Binary128>(0.0);
    Binary128 low = static_cast<Binary128>(0.0);
};

ALR20R38Pair al_r20_r38_two_sum(Binary128 left, Binary128 right) {
    ALR20R38Pair result;
    result.high = left + right;
    const Binary128 virtual_right = result.high - left;
    result.low = (left - (result.high - virtual_right))
        + (right - virtual_right);
    result.no_underflow = al_r20_r38_normal_or_zero(left)
        && al_r20_r38_normal_or_zero(right)
        && al_r20_r38_normal_or_zero(result.high)
        && al_r20_r38_normal_or_zero(virtual_right)
        && al_r20_r38_normal_or_zero(result.low);
    result.exact = finiteq(result.high) != 0
        && finiteq(result.low) != 0;
    return result;
}

ALR20R38Pair al_r20_r38_two_product(Binary128 left, Binary128 right) {
    ALR20R38Pair result;
    result.high = left * right;
    result.low = fmaq(left, right, -result.high);
    const bool product_not_lost = left == static_cast<Binary128>(0.0)
        || right == static_cast<Binary128>(0.0)
        || result.high != static_cast<Binary128>(0.0)
        || result.low != static_cast<Binary128>(0.0);
    result.no_underflow = product_not_lost
        && al_r20_r38_normal_or_zero(left)
        && al_r20_r38_normal_or_zero(right)
        && al_r20_r38_normal_or_zero(result.high)
        && al_r20_r38_normal_or_zero(result.low);
    result.exact = finiteq(result.high) != 0
        && finiteq(result.low) != 0;
    return result;
}

struct ALR20R38Dot2 {
    bool exact = false;
    bool no_underflow = false;
    Binary128 value = static_cast<Binary128>(0.0);
    Binary128 bound = static_cast<Binary128>(0.0);
    Binary128 absolute_products = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R38Dot2 al_r20_r38_dot2(
    const std::vector<Binary128>& left,
    const std::vector<Binary128>& right) {
    ALR20R38Dot2 result;
    if (left.empty() || left.size() != right.size()) return result;
    const ALR20R38Pair first = al_r20_r38_two_product(left[0U], right[0U]);
    Binary128 primary = first.high;
    Binary128 correction = first.low;
    bool no_underflow = first.no_underflow;
    result.absolute_products = al_r20_r38_up_multiply(
        fabsq(left[0U]), fabsq(right[0U]));
    for (std::size_t index = 1U; index < left.size(); ++index) {
        const ALR20R38Pair product =
            al_r20_r38_two_product(left[index], right[index]);
        const ALR20R38Pair sum =
            al_r20_r38_two_sum(primary, product.high);
        const Binary128 local = sum.low + product.low;
        correction = correction + local;
        primary = sum.high;
        no_underflow = no_underflow && product.no_underflow
            && sum.no_underflow && al_r20_r38_normal_or_zero(local)
            && al_r20_r38_normal_or_zero(correction);
        result.absolute_products = al_r20_r38_up_add(
            result.absolute_products, al_r20_r38_up_multiply(
                fabsq(left[index]), fabsq(right[index])));
    }
    result.value = primary + correction;
    no_underflow = no_underflow
        && al_r20_r38_normal_or_zero(result.value);
    const Binary128 unit =
        ldexpq(static_cast<Binary128>(1.0), -112);
    const Binary128 count_unit =
        static_cast<Binary128>(left.size()) * unit;
    const Binary128 gamma = al_r20_r38_up_divide(
        count_unit, static_cast<Binary128>(1.0) - count_unit);
    const Binary128 numerator = al_r20_r38_up_add(
        al_r20_r38_up_multiply(unit, fabsq(result.value)),
        al_r20_r38_up_multiply(
            al_r20_r38_up_multiply(gamma, gamma),
            result.absolute_products));
    result.bound = al_r20_r38_up_divide(
        numerator, static_cast<Binary128>(1.0) - unit);
    result.no_underflow = no_underflow;
    result.exact = first.exact
        && finiteq(result.value) != 0 && finiteq(result.bound) != 0
        && result.bound >= static_cast<Binary128>(0.0);
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << left.size() << ':' << al_binary128_hex(result.value) << ':'
        << al_binary128_hex(result.bound) << ':'
        << al_binary128_hex(result.absolute_products) << ':'
        << al_r20_q_scalar_root(left) << ':' << al_r20_q_scalar_root(right);
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_r38_pair_exact(Binary128 left, Binary128 right, bool product) {
    const ALR20R38Pair pair = product
        ? al_r20_r38_two_product(left, right)
        : al_r20_r38_two_sum(left, right);
    const ALR20R37Conversion left_exact =
        al_r20_r37_from_binary128(left);
    const ALR20R37Conversion right_exact =
        al_r20_r37_from_binary128(right);
    const ALR20R37Conversion high_exact =
        al_r20_r37_from_binary128(pair.high);
    const ALR20R37Conversion low_exact =
        al_r20_r37_from_binary128(pair.low);
    if (!pair.exact || !pair.no_underflow || !left_exact.exact
        || !right_exact.exact || !high_exact.exact || !low_exact.exact)
        return false;
    const ALR20R37Dyadic expected = product
        ? al_r20_r37_multiply(left_exact.value, right_exact.value)
        : al_r20_r37_add(left_exact.value, right_exact.value);
    const ALR20R37Dyadic reconstructed =
        al_r20_r37_add(high_exact.value, low_exact.value);
    return expected.numerator == reconstructed.numerator
        && expected.exponent == reconstructed.exponent;
}

bool al_r20_r38_dot_contains(
    const ALR20R38Dot2& dot, const ALR20R37Dyadic& exact,
    Binary128* exact_error_outward) {
    const ALR20R37Conversion nominal =
        al_r20_r37_from_binary128(dot.value);
    const ALR20R37Conversion bound =
        al_r20_r37_from_binary128(dot.bound);
    if (!dot.exact || !nominal.exact || !bound.exact) return false;
    const ALR20R37Dyadic difference = al_r20_r37_absolute(
        al_r20_r37_add(exact,
            {-nominal.value.numerator, nominal.value.exponent}));
    if (exact_error_outward != nullptr)
        *exact_error_outward = al_r20_r37_binary128_up(difference);
    return al_r20_r37_compare_positive(difference, bound.value) <= 0;
}

bool al_r20_r38_dot2_controls() {
    const Binary128 one = static_cast<Binary128>(1.0);
    const Binary128 large = ldexpq(one, 60);
    const bool pairs_exact =
        al_r20_r38_pair_exact(large + one, -large, false)
        && al_r20_r38_pair_exact(
            strtoflt128("1.125", nullptr),
            strtoflt128("0.875", nullptr), true);
    const std::vector<Binary128> left{one, large + one, -large};
    const std::vector<Binary128> right{one, one, one};
    const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
    ALR20R37Dyadic exact;
    for (std::size_t index = 0U; index < left.size(); ++index) {
        const ALR20R37Conversion lhs =
            al_r20_r37_from_binary128(left[index]);
        const ALR20R37Conversion rhs =
            al_r20_r37_from_binary128(right[index]);
        if (!lhs.exact || !rhs.exact) return false;
        exact = al_r20_r37_add(
            exact, al_r20_r37_multiply(lhs.value, rhs.value));
    }
    Binary128 error = static_cast<Binary128>(0.0);
    return pairs_exact && dot.exact && dot.no_underflow
        && al_r20_r38_dot_contains(dot, exact, &error)
        && dot.value == static_cast<Binary128>(2.0)
        && error == static_cast<Binary128>(0.0);
}

struct ALR20R38MatrixAudit {
    bool exact = false;
    bool no_underflow = false;
    bool contained = false;
    bool contractive = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t contained_entries = 0U;
    std::size_t worst_row = 0U;
    Binary128 rho_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_dot_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_exact_error = static_cast<Binary128>(0.0);
    Binary128 maximum_error_to_bound = static_cast<Binary128>(0.0);
    std::string dot_root;
    std::string root;
};

ALR20R38MatrixAudit al_r20_r38_matrix_audit(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const ALR20R37ExactResidual& oracle) {
    ALR20R38MatrixAudit result;
    result.dimension = oracle.dimension;
    const std::size_t dimension = result.dimension;
    if (!oracle.exact || oracle.residual.size() != dimension * dimension
        || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension)
        return result;
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
                left[inner + 1U] = matrix[row * dimension + inner];
                right[inner + 1U] = -inverse[inner * dimension + column];
            }
            const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
            Binary128 exact_error = static_cast<Binary128>(0.0);
            const bool entry_contained = al_r20_r38_dot_contains(
                dot, oracle.residual[row * dimension + column],
                &exact_error);
            ++result.dots;
            result.contained_entries += entry_contained ? 1U : 0U;
            no_underflow = no_underflow && dot.no_underflow;
            contained = contained && entry_contained;
            result.maximum_dot_bound = fmaxq(
                result.maximum_dot_bound, dot.bound);
            result.maximum_exact_error = fmaxq(
                result.maximum_exact_error, exact_error);
            if (dot.bound > static_cast<Binary128>(0.0))
                result.maximum_error_to_bound = fmaxq(
                    result.maximum_error_to_bound,
                    exact_error / dot.bound);
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
    result.exact = result.dots == dimension * dimension
        && finiteq(result.rho_bound) != 0
        && finiteq(result.maximum_dot_bound) != 0
        && finiteq(result.maximum_exact_error) != 0
        && finiteq(result.maximum_error_to_bound) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.contained << ':' << result.contractive << ':'
        << dimension << ':' << result.dots << ':' << result.contained_entries
        << ':' << result.worst_row << ':'
        << al_binary128_hex(result.rho_bound) << ':'
        << al_binary128_hex(result.maximum_dot_bound) << ':'
        << al_binary128_hex(result.maximum_exact_error) << ':'
        << al_binary128_hex(result.maximum_error_to_bound) << ':'
        << result.dot_root << ':' << oracle.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v4_dot2_inverse_certificate_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool parent_controls_exact = profile_exact
        && al_r20_verified_inverse_controls()
        && al_r20_semismooth_controls()
        && al_r20_r21_event_controls()
        && al_r20_r37_dyadic_controls();
    const bool dot2_controls_exact =
        profile_exact && al_r20_r38_dot2_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    bool parent_exact = parent_controls_exact && cases.size() == 5U
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
        && capture.calls == 2U && capture.audit_roots.size() == 2U
        && target_capture_count == 1U
        && capture.audit.root == AL_R20_R36_TORSION_INVERSE_SHA256
        && capture.matrix_root == AL_R20_R37_MATRIX_SHA256
        && capture.factor_root == AL_R20_R37_FACTOR_SHA256
        && al_r20_q_scalar_root(capture.inverse)
            == AL_R20_R37_INVERSE_SHA256
        && capture.audit.dimension == 65U
        && capture.audit.column_solves == 65U
        && solver_executions == 1U;
    const ALR20R37ExactResidual oracle = reproduction_exact
        ? al_r20_r37_exact_residual(
            capture.matrix, capture.inverse, capture.audit.dimension)
        : ALR20R37ExactResidual{};
    const bool oracle_exact = oracle.exact
        && oracle.residual_root == AL_R20_R37_RESIDUAL_SHA256
        && oracle.root == AL_R20_R37_AUDIT_SHA256;
    const ALR20R38MatrixAudit dot2 =
        reproduction_exact && oracle_exact && dot2_controls_exact
        ? al_r20_r38_matrix_audit(capture.matrix, capture.inverse, oracle)
        : ALR20R38MatrixAudit{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool exact = parent_exact && reproduction_exact && oracle_exact
        && dot2_controls_exact && dot2.exact && lifecycle_exact;
    const std::string route = !parent_exact || !reproduction_exact
        || !oracle_exact
        ? "DOT2_PARENT_REJECTED"
        : (!dot2_controls_exact || !dot2.exact
            ? "DOT2_APPARATUS_REJECTED"
            : (!dot2.no_underflow
                ? "DOT2_UNDERFLOW_REJECTED"
                : (!dot2.contained
                    ? "DOT2_CONTAINMENT_REJECTED"
                    : (dot2.contractive
                        ? "DOT2_INVERSE_CERTIFICATE_CANDIDATE"
                        : "DOT2_RESIDUAL_BOUND_NONCONTRACTIVE"))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r38-dot2-certificate.v1|"
        << AL_R20_R37_SEMANTIC_SHA256 << ':' << route << ':'
        << torsion.root << ':' << capture.audit.root << ':'
        << oracle.root << ':' << dot2.root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r38_dot2_certificate.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r37_sha256\":\""
        << AL_R20_R37_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"parent_controls_exact\":"
        << (parent_controls_exact ? "true" : "false")
        << ",\"dot2_controls_exact\":"
        << (dot2_controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"oracle_exact\":" << (oracle_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"case_sha256\":\"" << torsion.root
        << "\",\"matrix_sha256\":\"" << capture.matrix_root
        << "\",\"factor_sha256\":\"" << capture.factor_root
        << "\",\"inverse_sha256\":\""
        << al_r20_q_scalar_root(capture.inverse)
        << "\",\"exact_residual_sha256\":\"" << oracle.residual_root
        << "\",\"exact_audit_sha256\":\"" << oracle.root
        << "\",\"dot2\":{\"no_underflow\":"
        << (dot2.no_underflow ? "true" : "false")
        << ",\"contained\":" << (dot2.contained ? "true" : "false")
        << ",\"contractive\":"
        << (dot2.contractive ? "true" : "false")
        << ",\"dots\":" << dot2.dots
        << ",\"contained_entries\":" << dot2.contained_entries
        << ",\"worst_row\":" << dot2.worst_row
        << ",\"rho_bound\":\"" << al_binary128_text(dot2.rho_bound)
        << "\",\"exact_rho_outward\":\""
        << al_binary128_text(oracle.outward_norm)
        << "\",\"maximum_dot_bound\":\""
        << al_binary128_text(dot2.maximum_dot_bound)
        << "\",\"maximum_exact_error\":\""
        << al_binary128_text(dot2.maximum_exact_error)
        << "\",\"maximum_error_to_bound\":\""
        << al_binary128_text(dot2.maximum_error_to_bound)
        << "\",\"dot_sha256\":\"" << dot2.dot_root
        << "\",\"root\":\"" << dot2.root << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"captured_inverse_calls\":" << capture.calls
        << ",\"captured_inverse_columns\":"
        << capture.total_column_solves
        << ",\"compensated_residual_dots\":" << dot2.dots
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0"
        << ",\"solution_refinements\":0,\"counterflow_replays\":0"
        << ",\"new_trials\":0,\"state_updates\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
