namespace {

constexpr const char* AL_R20_R63S_PARENT_SEMANTIC_SHA256 =
    "b8783c5390382c3169ff49c9d7df7c8facce50a15c7f46804ed10eb0bb65f382";
constexpr const char* AL_R20_R63S_INVERSE_SHA256 =
    "32508a73b43eab2afd6737761d00b5d166a2b415151fb6ff4d92fd3df0ed4a07";
constexpr const char* AL_R20_R63S_DEFECT_SHA256 =
    "fea187b0a8583cdc08f71521cfcb64f996c4c09c93db71fa94878d24b018b9d5";
constexpr const char* AL_R20_R63S_INVERSE_AUDIT_SHA256 =
    "a86c14bad30ffb45cc65491420b885f41f6bcaf5930014efe71b427a4f092aee";

struct ALR20R63SCertificate {
    bool exact = false;
    bool passed = false;
    std::size_t dimension = 0U;
    std::size_t solution_conversions = 0U;
    std::size_t residual_products = 0U;
    std::size_t residual_entries = 0U;
    std::size_t sign_comparisons = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    ALR20R37Dyadic residual_infinity_numerator;
    ALR20R37Dyadic residual_denominator;
    ALR20R37Dyadic error_numerator;
    ALR20R37Dyadic error_denominator;
    ALR20R37Dyadic minimum_resolved_separation;
    std::vector<int> signs;
    std::string solution_root;
    std::string residual_root;
    std::string sign_root;
    std::string root;
};

ALR20R63SCertificate al_r20_r63s_certificate(
    const std::vector<ALR20R37Dyadic>& rhs,
    const ALR20R63QExactMatrix& matrix,
    const ALR20R63RDefect& inverse_defect,
    const std::vector<Binary128>& solution) {
    ALR20R63SCertificate result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || solution.size() != dimension
        || !matrix.exact || matrix.rows != dimension
        || matrix.oracle_numerator.size() != dimension * dimension
        || !inverse_defect.exact || !inverse_defect.left_contractive
        || !al_r20_r63p_equal(matrix.d, inverse_defect.d)
        || al_r20_r63p_sign(matrix.d) <= 0
        || al_r20_r63p_sign(
            inverse_defect.amplification_denominator) <= 0) return result;
    result.residual_denominator = matrix.d;
    bool conversions_exact = true;
    std::vector<ALR20R37Dyadic> exact_solution(dimension);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(solution[index]);
        conversions_exact = conversions_exact && conversion.exact;
        exact_solution[index] = conversion.value;
        ++result.solution_conversions;
    }
    std::vector<ALR20R37Dyadic> residual(dimension);
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic image;
        for (std::size_t column = 0U; column < dimension; ++column) {
            image = al_r20_r37_add(image, al_r20_r37_multiply(
                matrix.oracle_numerator[row * dimension + column],
                exact_solution[column]));
            ++result.residual_products;
        }
        residual[row] = al_r20_r63p_subtract(
            al_r20_r37_multiply(rhs[row], matrix.d), image);
        const ALR20R37Dyadic absolute = al_r20_r37_absolute(residual[row]);
        if (row == 0U || al_r20_r37_compare_positive(
                absolute, result.residual_infinity_numerator) > 0)
            result.residual_infinity_numerator = absolute;
        ++result.residual_entries;
    }
    result.error_numerator = al_r20_r37_multiply(
        inverse_defect.inverse_infinity_norm,
        result.residual_infinity_numerator);
    result.error_denominator = inverse_defect.amplification_denominator;
    result.signs.resize(dimension);
    bool have_resolved = false;
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R37Dyadic scaled = al_r20_r37_multiply(
            al_r20_r37_absolute(exact_solution[index]),
            result.error_denominator);
        const int comparison = al_r20_r37_compare_positive(
            scaled, result.error_numerator);
        if (al_r20_r63p_sign(exact_solution[index]) > 0
            && comparison > 0) {
            result.signs[index] = 1;
            ++result.positive;
        } else if (al_r20_r63p_sign(exact_solution[index]) < 0
            && comparison > 0) {
            result.signs[index] = -1;
            ++result.negative;
        } else {
            ++result.unresolved;
        }
        if (comparison > 0) {
            const ALR20R37Dyadic separation = al_r20_r63p_subtract(
                scaled, result.error_numerator);
            if (!have_resolved || al_r20_r37_compare_positive(
                    separation, result.minimum_resolved_separation) < 0) {
                result.minimum_resolved_separation = separation;
                have_resolved = true;
            }
        }
        ++result.sign_comparisons;
    }
    result.passed = result.unresolved == 0U
        && result.positive + result.negative == dimension;
    result.solution_root = al_r20_r63q_vector_root(exact_solution);
    result.residual_root = al_r20_r63q_vector_root(residual);
    std::ostringstream sign_material;
    for (int sign : result.signs) sign_material << sign << ',';
    result.sign_root = sha256_hex(sign_material.str());
    result.exact = conversions_exact
        && result.solution_conversions == dimension
        && result.residual_products == dimension * dimension
        && result.residual_entries == dimension
        && result.sign_comparisons == dimension
        && result.positive + result.negative + result.unresolved == dimension
        && (!result.passed || have_resolved);
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':'
        << result.dimension << ':' << result.solution_conversions << ':'
        << result.residual_products << ':' << result.residual_entries << ':'
        << result.sign_comparisons << ':' << result.positive << ':'
        << result.negative << ':' << result.unresolved << ':'
        << al_r20_r37_dyadic_text(result.residual_infinity_numerator) << ':'
        << al_r20_r37_dyadic_text(result.residual_denominator) << ':'
        << al_r20_r37_dyadic_text(result.error_numerator) << ':'
        << al_r20_r37_dyadic_text(result.error_denominator) << ':'
        << al_r20_r37_dyadic_text(result.minimum_resolved_separation) << ':'
        << result.solution_root << ':' << result.residual_root << ':'
        << result.sign_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63SStep {
    bool exact = false;
    std::size_t iteration = 0U;
    Binary128 rho = static_cast<Binary128>(0.0);
    Binary128 denominator = static_cast<Binary128>(0.0);
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 beta = static_cast<Binary128>(0.0);
    // Observational snapshot for later independently certified replay gates.
    // It is deliberately excluded from the frozen R63S root; the existing
    // certificate solution_root already binds these exact values.
    std::vector<Binary128> solution;
    ALR20R63SCertificate certificate;
    std::string root;
};

struct ALR20R63SLane {
    bool exact = false;
    bool finite = false;
    bool passed = false;
    bool has_pass = false;
    bool stable_after_pass = true;
    std::size_t dimension = 0U;
    std::size_t completed_iterations = 0U;
    std::size_t failure_iteration = 0U;
    std::size_t first_passing_iteration = 0U;
    std::size_t rectangular_products = 0U;
    std::size_t direct_inner_dots = 0U;
    std::size_t direct_outer_dots = 0U;
    std::size_t direct_inner_terms = 0U;
    std::size_t direct_outer_terms = 0U;
    std::size_t direct_scale_products = 0U;
    std::size_t initial_residual_dots = 0U;
    std::size_t preconditioner_solves = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::size_t rho_dots = 0U;
    std::size_t denominator_dots = 0U;
    std::size_t solution_update_dots = 0U;
    std::size_t residual_update_dots = 0U;
    std::size_t direction_update_dots = 0U;
    std::size_t certificates = 0U;
    std::size_t certificate_solution_conversions = 0U;
    std::size_t certificate_residual_products = 0U;
    std::size_t certificate_residual_entries = 0U;
    std::size_t certificate_sign_comparisons = 0U;
    Binary128 minimum_error = HUGE_VALQ;
    std::vector<Binary128> solution;
    std::vector<Binary128> residual;
    std::vector<Binary128> direction;
    ALR20R63SCertificate initial_certificate;
    std::vector<ALR20R63SStep> steps;
    std::string failure_stage;
    std::string root;
};

ALR20R63SLane al_r20_r63s_lane(
    const std::vector<Binary128>& upper,
    const std::vector<std::size_t>& permutation,
    Binary128 inverse_scale,
    const std::vector<Binary128>& tangent,
    std::size_t tangent_columns,
    Binary128 sigma,
    const std::vector<Binary128>& rhs,
    const std::vector<ALR20R37Dyadic>& exact_rhs,
    const ALR20R63QExactMatrix& matrix,
    const ALR20R63RDefect& inverse_defect,
    const std::vector<Binary128>& initial) {
    ALR20R63SLane result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || tangent.size() != dimension * tangent_columns
        || exact_rhs.size() != dimension || initial.size() != dimension
        || !matrix.exact || !inverse_defect.exact
        || !(sigma > static_cast<Binary128>(0.0))) return result;
    auto record_product = [&result](const ALR20R63NProduct& product) {
        ++result.rectangular_products;
        result.direct_inner_dots += product.inner_dots;
        result.direct_outer_dots += product.outer_dots;
        result.direct_inner_terms += product.inner_terms;
        result.direct_outer_terms += product.outer_terms;
        result.direct_scale_products += product.scale_products;
    };
    auto record_certificate = [&result](const ALR20R63SCertificate& cert,
                                  std::size_t iteration) {
        ++result.certificates;
        result.certificate_solution_conversions += cert.solution_conversions;
        result.certificate_residual_products += cert.residual_products;
        result.certificate_residual_entries += cert.residual_entries;
        result.certificate_sign_comparisons += cert.sign_comparisons;
        if (cert.passed && !result.has_pass) {
            result.has_pass = true;
            result.first_passing_iteration = iteration;
        } else if (result.has_pass && !cert.passed) {
            result.stable_after_pass = false;
        }
        if (cert.exact) {
            const Binary128 error = al_r20_r63p_fraction_projection(
                cert.error_numerator, cert.error_denominator);
            result.minimum_error = fminq(result.minimum_error, error);
        }
    };
    result.solution = initial;
    result.initial_certificate = al_r20_r63s_certificate(
        exact_rhs, matrix, inverse_defect, result.solution);
    bool exact = result.initial_certificate.exact;
    record_certificate(result.initial_certificate, 0U);
    bool finite = exact;
    if (!finite) result.failure_stage = "initial_certificate";
    const ALR20R63NProduct initial_product = al_r20_r63n_product(
        tangent, dimension, tangent_columns, sigma, result.solution);
    record_product(initial_product);
    exact = exact && initial_product.exact;
    finite = finite && initial_product.exact;
    if (!finite && result.failure_stage.empty())
        result.failure_stage = "initial_product";
    result.residual.resize(dimension);
    if (finite) {
        for (std::size_t row = 0U; row < dimension; ++row) {
            const ALR20R38Dot2 dot = al_r20_r38_dot2(
                {rhs[row], -initial_product.value[row]},
                {static_cast<Binary128>(1.0),
                    static_cast<Binary128>(1.0)});
            result.residual[row] = dot.value;
            exact = exact && dot.exact && dot.no_underflow
                && finiteq(dot.value) != 0;
            ++result.initial_residual_dots;
        }
        finite = finite && exact;
        if (!finite) result.failure_stage = "initial_residual";
    }
    ALR20R63IQSolve preconditioned;
    if (finite) {
        preconditioned = al_r20_r63i_q_solve(
            upper, permutation, result.residual, inverse_scale);
        result.forward_terms += preconditioned.forward_terms;
        result.backward_terms += preconditioned.backward_terms;
        result.divisions += preconditioned.divisions;
        ++result.preconditioner_solves;
        if (!preconditioned.exact) {
            finite = false;
            result.failure_stage = "initial_preconditioner";
        } else {
            result.direction = preconditioned.solution;
        }
    }
    ALR20R63MScalar rho;
    if (finite) {
        rho = al_r20_r63m_scalar(result.residual, preconditioned.solution);
        ++result.rho_dots;
        if (!rho.exact || !rho.positive) {
            finite = false;
            result.failure_stage = "initial_rho";
        }
    }
    for (std::size_t iteration = 1U;
         iteration <= 8U && finite; ++iteration) {
        ALR20R63SStep step;
        step.iteration = iteration;
        step.rho = rho.value;
        const ALR20R63NProduct product = al_r20_r63n_product(
            tangent, dimension, tangent_columns, sigma, result.direction);
        record_product(product);
        if (!product.exact) {
            finite = false;
            exact = false;
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
            exact = false;
            result.failure_iteration = iteration;
            result.failure_stage = "update";
            break;
        }
        result.solution = std::move(updated_solution);
        result.residual = std::move(updated_residual);
        ++result.completed_iterations;
        step.solution = result.solution;
        step.certificate = al_r20_r63s_certificate(
            exact_rhs, matrix, inverse_defect, result.solution);
        record_certificate(step.certificate, iteration);
        if (!step.certificate.exact) {
            finite = false;
            exact = false;
            result.failure_iteration = iteration;
            result.failure_stage = "certificate";
            break;
        }
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
            + al_binary128_hex(step.beta) + ':' + product.root + ':'
            + denominator.root + ':' + sha256_hex(update_material.str())
            + ':' + step.certificate.root + ':' + preconditioner_root + ':'
            + rho_new_root + ':' + direction_root);
        exact = exact && step.exact;
        result.steps.push_back(std::move(step));
    }
    result.finite = finite;
    const bool complete = result.completed_iterations == 8U
        && result.steps.size() == 8U;
    result.passed = finite && complete && result.has_pass
        && result.stable_after_pass;
    result.exact = exact && ((finite && complete)
        || (!finite && !result.failure_stage.empty()));
    std::ostringstream material;
    material << result.exact << ':' << result.finite << ':'
        << result.passed << ':' << result.has_pass << ':'
        << result.stable_after_pass << ':' << result.dimension << ':'
        << result.completed_iterations << ':' << result.failure_iteration
        << ':' << result.first_passing_iteration << ':'
        << result.failure_stage << ':' << result.rectangular_products << ':'
        << result.direct_inner_dots << ':' << result.direct_outer_dots << ':'
        << result.direct_inner_terms << ':' << result.direct_outer_terms << ':'
        << result.direct_scale_products << ':'
        << result.initial_residual_dots << ':'
        << result.preconditioner_solves << ':' << result.forward_terms << ':'
        << result.backward_terms << ':' << result.divisions << ':'
        << result.rho_dots << ':' << result.denominator_dots << ':'
        << result.solution_update_dots << ':'
        << result.residual_update_dots << ':'
        << result.direction_update_dots << ':' << result.certificates << ':'
        << result.certificate_solution_conversions << ':'
        << result.certificate_residual_products << ':'
        << result.certificate_residual_entries << ':'
        << result.certificate_sign_comparisons << ':'
        << al_binary128_hex(result.minimum_error) << ':'
        << al_r20_q_scalar_root(result.solution) << ':'
        << al_r20_q_scalar_root(result.residual) << ':'
        << al_r20_q_scalar_root(result.direction) << ':'
        << result.initial_certificate.root << '|';
    for (const ALR20R63SStep& step : result.steps)
        material << step.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63s_classify(
    bool apparatus, bool certificate, bool wide, bool exported) {
    if (!apparatus) return "COMMON_OPERATOR_PCG_APPARATUS_REJECTED";
    if (!certificate) return "COMMON_OPERATOR_CERTIFICATE_REJECTED";
    if (!wide) return "RETAINED_WIDE_COMMON_OPERATOR_PCG_REJECTED";
    if (!exported) return "EXPORTED_FACTOR_COMMON_OPERATOR_PCG_REJECTED";
    return "EXPORTED_FACTOR_COMMON_OPERATOR_PCG_CANDIDATE";
}

struct ALR20R63SControls {
    bool exact = false;
    bool zero_residual = false;
    bool bounded = false;
    bool unresolved = false;
    bool denominator = false;
    bool parent_algorithms = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63SControls al_r20_r63s_controls() {
    ALR20R63SControls result;
    const ALR20R37Dyadic zero = al_r20_r63p_control_dyadic(0);
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const std::vector<ALR20R37Dyadic> identity{one, zero, zero, one};
    const ALR20R63RDefect defect = al_r20_r63r_defect(
        identity, identity, one, 2U);
    ALR20R63QExactMatrix matrix;
    matrix.exact = true;
    matrix.rows = 2U;
    matrix.columns = 2U;
    matrix.d = one;
    matrix.oracle_numerator = identity;
    const std::vector<ALR20R37Dyadic> rhs{one,
        al_r20_r63p_control_dyadic(-1)};
    const ALR20R63SCertificate exact = al_r20_r63s_certificate(
        rhs, matrix, defect,
        {static_cast<Binary128>(1.0), static_cast<Binary128>(-1.0)});
    result.zero_residual = exact.exact && exact.passed
        && exact.positive == 1U && exact.negative == 1U
        && exact.residual_infinity_numerator.numerator == 0;
    const ALR20R63SCertificate bounded = al_r20_r63s_certificate(
        rhs, matrix, defect,
        {static_cast<Binary128>(0.5), static_cast<Binary128>(-1.0)});
    result.bounded = bounded.exact && !bounded.passed
        && bounded.residual_infinity_numerator.numerator != 0;
    const ALR20R63SCertificate unresolved = al_r20_r63s_certificate(
        rhs, matrix, defect,
        {static_cast<Binary128>(0.0), static_cast<Binary128>(-1.0)});
    result.unresolved = unresolved.exact && !unresolved.passed
        && unresolved.unresolved > 0U;
    ALR20R63RDefect invalid = defect;
    invalid.amplification_denominator = {};
    const ALR20R63SCertificate invalid_certificate = al_r20_r63s_certificate(
        rhs, matrix, invalid,
        {static_cast<Binary128>(1.0), static_cast<Binary128>(-1.0)});
    result.denominator = !invalid_certificate.exact;
    result.parent_algorithms = al_r20_r63m_controls().exact
        && al_r20_r63n_controls().exact;
    std::vector<int> mutated = exact.signs;
    mutated[0U] = -mutated[0U];
    std::ostringstream mutation_material;
    for (int sign : mutated) mutation_material << sign << ',';
    const std::string first_pass_zero = sha256_hex(
        std::string("1:0:") + exact.root);
    const std::string first_pass_one = sha256_hex(
        std::string("1:1:") + exact.root);
    result.mutation = sha256_hex(mutation_material.str()) != exact.sign_root
        && exact.root != bounded.root
        && exact.solution_root != bounded.solution_root
        && exact.residual_root != bounded.residual_root
        && exact.error_numerator.numerator
            != bounded.error_numerator.numerator
        && exact.sign_root != bounded.sign_root
        && first_pass_zero != first_pass_one;
    result.classifier =
        al_r20_r63s_classify(false, true, true, true)
            == "COMMON_OPERATOR_PCG_APPARATUS_REJECTED"
        && al_r20_r63s_classify(true, false, true, true)
            == "COMMON_OPERATOR_CERTIFICATE_REJECTED"
        && al_r20_r63s_classify(true, true, false, true)
            == "RETAINED_WIDE_COMMON_OPERATOR_PCG_REJECTED"
        && al_r20_r63s_classify(true, true, true, false)
            == "EXPORTED_FACTOR_COMMON_OPERATOR_PCG_REJECTED"
        && al_r20_r63s_classify(true, true, true, true)
            == "EXPORTED_FACTOR_COMMON_OPERATOR_PCG_CANDIDATE";
    result.exact = result.zero_residual && result.bounded
        && result.unresolved && result.denominator
        && result.parent_algorithms && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + exact.root + ':' + bounded.root + ':' + unresolved.root + ':'
        + invalid_certificate.root + ':' + sha256_hex(mutation_material.str())
        + ':' + first_pass_zero + ':' + first_pass_one);
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_common_operator_pcg_controls() {
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
    const ALR20R63SControls controls = al_r20_r63s_controls();
    const ALR20R63RControls parent_controls = al_r20_r63r_controls();
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
        && al_r20_q_scalar_root(factor_capture.rhs) == AL_R20_R63I_RHS_SHA256
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
    const ALR20R63QAudit common_audit = tangent.exact
        ? al_r20_r63q_audit(grams, tangent, factor_capture.projector,
            factor_capture.matrix, sigma)
        : ALR20R63QAudit{};
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows);
    std::vector<Binary128> permuted_original;
    std::vector<double> permuted_d;
    if (common_audit.exact && permutation_exact) {
        permuted_original.resize(tangent.columns * tangent.rows);
        permuted_d.resize(permuted_original.size());
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column) {
                const Binary128 value = tangent.values[
                    permutation[column] * tangent.columns + scalar];
                permuted_original[scalar * tangent.rows + column] = value;
                permuted_d[scalar * tangent.rows + column] =
                    static_cast<double>(value);
            }
    }
    const ALR20R63GQFactor original_factor = !permuted_original.empty()
        ? al_r20_r63g_q_factor(
            permuted_original, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    const ALR20R63RInverse verifier_inverse = original_factor.exact
        ? al_r20_r63r_inverse(
            original_factor.r, permutation, inverse_scale)
        : ALR20R63RInverse{};
    const ALR20R63RExactAudit verifier_audit = verifier_inverse.exact
        ? al_r20_r63r_exact_audit(verifier_inverse, common_audit.matrix)
        : ALR20R63RExactAudit{};
    const bool parent_r63r_exact = capture_exact && common_audit.exact
        && common_audit.root == AL_R20_R63R_AUDIT_SHA256
        && common_audit.matrix.root == AL_R20_R63R_MATRIX_SHA256
        && common_audit.matrix.oracle_root == AL_R20_R63R_ORACLE_SHA256
        && al_r20_r63g_index_root(permutation)
            == AL_R20_R63R_PERMUTATION_SHA256
        && original_factor.root == AL_R20_R63R_FACTOR_SHA256
        && verifier_inverse.root == AL_R20_R63S_INVERSE_SHA256
        && verifier_audit.defect.root == AL_R20_R63S_DEFECT_SHA256
        && verifier_audit.root == AL_R20_R63S_INVERSE_AUDIT_SHA256
        && verifier_audit.defect.left_contractive
        && verifier_audit.defect.right_contractive;
    const ALR20R63GDFactor stored_d_factor = !permuted_d.empty()
        ? al_r20_r63g_d_factor(
            permuted_d, tangent.columns, tangent.rows)
        : ALR20R63GDFactor{};
    const std::vector<Binary128> stored_permuted =
        al_r20_r63g_q_from_double(stored_d_factor.input);
    const ALR20R63GQFactor wide_factor = !stored_permuted.empty()
        ? al_r20_r63g_q_factor(
            stored_permuted, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    std::vector<double> exported_d;
    if (wide_factor.exact)
        for (Binary128 value : wide_factor.r)
            exported_d.push_back(static_cast<double>(value));
    const std::vector<Binary128> exported_r =
        al_r20_r63g_q_from_double(exported_d);
    const bool preconditioners_exact = stored_d_factor.exact
        && wide_factor.exact
        && wide_factor.root == AL_R20_R63I_WIDE_FACTOR_SHA256
        && al_r20_double_root(exported_d) == AL_R20_R63I_EXPORT_R_SHA256;
    const ALR20R63IQSolve wide_start = preconditioners_exact
        ? al_r20_r63i_q_solve(wide_factor.r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IQSolve export_start = preconditioners_exact
        ? al_r20_r63i_q_solve(exported_r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const bool starts_exact = wide_start.exact && export_start.exact
        && wide_start.root == AL_R20_R63J_WIDE_SOLVE_SHA256
        && export_start.root == AL_R20_R63J_EXPORT_WIDE_SOLVE_SHA256;
    bool rhs_conversions_exact = true;
    std::size_t rhs_conversions = 0U;
    std::vector<ALR20R37Dyadic> exact_rhs(factor_capture.rhs.size());
    for (std::size_t index = 0U; index < factor_capture.rhs.size(); ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(factor_capture.rhs[index]);
        rhs_conversions_exact = rhs_conversions_exact && conversion.exact;
        exact_rhs[index] = conversion.value;
        ++rhs_conversions;
    }
    const std::string exact_rhs_root = al_r20_r63q_vector_root(exact_rhs);
    const ALR20R63SLane wide = parent_r63r_exact && starts_exact
        && rhs_conversions_exact
        ? al_r20_r63s_lane(wide_factor.r, permutation, inverse_scale,
            tangent.values, tangent.columns, sigma, factor_capture.rhs,
            exact_rhs, common_audit.matrix, verifier_audit.defect,
            wide_start.solution)
        : ALR20R63SLane{};
    const ALR20R63SLane exported = parent_r63r_exact && starts_exact
        && rhs_conversions_exact
        ? al_r20_r63s_lane(exported_r, permutation, inverse_scale,
            tangent.values, tangent.columns, sigma, factor_capture.rhs,
            exact_rhs, common_audit.matrix, verifier_audit.defect,
            export_start.solution)
        : ALR20R63SLane{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    auto lane_work_exact = [](const ALR20R63SLane& lane) {
        return lane.completed_iterations == 8U
            && lane.rectangular_products == 9U
            && lane.direct_inner_dots == 2835U
            && lane.direct_outer_dots == 918U
            && lane.direct_inner_terms == 289170U
            && lane.direct_outer_terms == 289170U
            && lane.direct_scale_products == 918U
            && lane.initial_residual_dots == 102U
            && lane.preconditioner_solves == 8U
            && lane.forward_terms == 41208U
            && lane.backward_terms == 41208U && lane.divisions == 1632U
            && lane.rho_dots == 8U && lane.denominator_dots == 8U
            && lane.solution_update_dots == 816U
            && lane.residual_update_dots == 816U
            && lane.direction_update_dots == 714U
            && lane.certificates == 9U
            && lane.certificate_solution_conversions == 918U
            && lane.certificate_residual_products == 93636U
            && lane.certificate_residual_entries == 918U
            && lane.certificate_sign_comparisons == 918U;
    };
    const bool work_exact = lane_work_exact(wide)
        && lane_work_exact(exported);
    const bool final_signs_equal = wide.steps.size() == 8U
        && exported.steps.size() == 8U
        && wide.steps.back().certificate.signs
            == exported.steps.back().certificate.signs;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63r_exact && preconditioners_exact && starts_exact
        && rhs_conversions_exact && rhs_conversions == 102U
        && lifecycle_exact;
    const bool certificate_exact = wide.exact && exported.exact
        && work_exact && final_signs_equal;
    const std::string route = al_r20_r63s_classify(apparatus_exact,
        certificate_exact, wide.passed, exported.passed);
    const bool exact = apparatus_exact && certificate_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63s-common-operator-pcg.v1|"
        << AL_R20_R63S_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << common_audit.root << ':'
        << verifier_audit.root << ':' << exact_rhs_root << ':'
        << wide.root << ':' << exported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    auto certificate_json = [](const ALR20R63SCertificate& certificate) {
        std::ostringstream stream;
        stream << "{\"exact\":" << (certificate.exact ? "true" : "false")
            << ",\"passed\":" << (certificate.passed ? "true" : "false")
            << ",\"residual_infinity\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.residual_infinity_numerator,
                certificate.residual_denominator))
            << "\",\"error\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.error_numerator,
                certificate.error_denominator))
            << "\",\"minimum_resolved_margin\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.minimum_resolved_separation,
                certificate.error_denominator))
            << "\",\"positive\":" << certificate.positive
            << ",\"negative\":" << certificate.negative
            << ",\"unresolved\":" << certificate.unresolved
            << ",\"sign_root\":\"" << certificate.sign_root << "\""
            << ",\"root\":\"" << certificate.root << "\"}";
        return stream.str();
    };
    auto lane_json = [&certificate_json](
                         const char* name, const ALR20R63SLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"finite\":" << (lane.finite ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"has_pass\":" << (lane.has_pass ? "true" : "false")
            << ",\"stable_after_pass\":"
            << (lane.stable_after_pass ? "true" : "false")
            << ",\"completed_iterations\":" << lane.completed_iterations
            << ",\"first_passing_iteration\":"
            << lane.first_passing_iteration
            << ",\"failure_iteration\":" << lane.failure_iteration
            << ",\"failure_stage\":\"" << lane.failure_stage << "\""
            << ",\"minimum_error\":\""
            << al_binary128_text(lane.minimum_error)
            << "\",\"initial_certificate\":"
            << certificate_json(lane.initial_certificate) << ",\"steps\":[";
        for (std::size_t index = 0U; index < lane.steps.size(); ++index) {
            if (index != 0U) stream << ',';
            stream << "{\"iteration\":" << lane.steps[index].iteration
                << ",\"certificate\":"
                << certificate_json(lane.steps[index].certificate)
                << ",\"root\":\"" << lane.steps[index].root << "\"}";
        }
        stream << "],\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63s_common_operator_pcg.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63r_sha256\":\""
        << AL_R20_R63S_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63r_exact\":"
        << (parent_r63r_exact ? "true" : "false")
        << ",\"rhs_conversions\":" << rhs_conversions
        << ",\"exact_rhs_root\":\"" << exact_rhs_root << "\""
        << ",\"lanes\":{" << lane_json("wide", wide) << ','
        << lane_json("exported", exported) << '}'
        << ",\"final_signs_equal\":"
        << (final_signs_equal ? "true" : "false")
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"certificate_exact\":"
        << (certificate_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"candidate_factor_solves\":"
        << (wide.preconditioner_solves + exported.preconditioner_solves)
        << ",\"candidate_rectangular_products\":"
        << (wide.rectangular_products + exported.rectangular_products)
        << ",\"verification_exact_residual_products\":"
        << (wide.certificate_residual_products
            + exported.certificate_residual_products)
        << ",\"dense_candidate_products\":0,\"dense_inverse_uses\":0"
        << ",\"legacy_sign_uses\":0,\"defect_transport_terms\":0"
        << ",\"sparse_constructions\":0,\"rank_actions\":0,\"row_drops\":0"
        << ",\"regularizations\":0,\"replacement_applies\":0"
        << ",\"following_transitions\":0,\"state_updates\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
