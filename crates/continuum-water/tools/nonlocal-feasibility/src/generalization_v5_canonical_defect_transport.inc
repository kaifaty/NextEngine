namespace {

constexpr const char* AL_R20_R63O_PARENT_SEMANTIC_SHA256 =
    "b8d0c7081646d94aaeb441982ac22afee4477f5df426e6311d4ce03079f02772";
constexpr const char* AL_R20_R63O_CANONICAL_PARENT_SHA256 =
    "3f13e864147e022d47a12c88a8e0911c78b5f8f11fd5ba15123e07470802c28e";
constexpr const char* AL_R20_R63O_WIDE_FAILURE_SHA256 =
    "623feaec490e9f3c424eb5665fbe77f0b3244815db5bdd15adc1f4d70f27e6bb";
constexpr const char* AL_R20_R63O_EXPORT_FAILURE_SHA256 =
    "1acc1fcee6e90faba50ca8192fd30f2985ee469fc5e7845ba6c941f82e86c4f1";

struct ALR20R63ODefect {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t canonical_products = 0U;
    std::size_t entries = 0U;
    std::size_t center_differences = 0U;
    std::size_t bound_constructions = 0U;
    std::size_t row_sum_terms = 0U;
    Binary128 maximum_entry = static_cast<Binary128>(0.0);
    Binary128 maximum_row_sum = static_cast<Binary128>(0.0);
    std::vector<Binary128> value;
    std::string canonical_root;
    std::string root;
};

ALR20R63ODefect al_r20_r63o_defect(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& tangent, std::size_t rows,
    std::size_t columns, Binary128 sigma) {
    ALR20R63ODefect result;
    result.dimension = rows;
    if (rows == 0U || matrix.size() != rows * rows
        || tangent.size() != rows * columns) return result;
    result.value.resize(rows * rows);
    bool exact = true;
    for (std::size_t column = 0U; column < rows; ++column) {
        std::vector<Binary128> basis(rows, static_cast<Binary128>(0.0));
        basis[column] = static_cast<Binary128>(1.0);
        const ALR20R63MProduct dense = al_r20_r63m_product(matrix, basis);
        const ALR20R63NProduct direct = al_r20_r63n_product(
            tangent, rows, columns, sigma, basis);
        exact = exact && dense.exact && direct.exact;
        for (std::size_t row = 0U; row < rows; ++row) {
            const ALR20R38Pair difference = al_r20_r38_two_sum(
                dense.value[row], -direct.value[row]);
            const Binary128 residual = al_r20_r38_up_add(
                fabsq(difference.high), fabsq(difference.low));
            const Binary128 arithmetic = al_r20_r38_up_add(
                dense.bound[row], direct.bound[row]);
            const Binary128 enclosure = al_r20_r38_up_add(
                residual, arithmetic);
            result.value[row * rows + column] = enclosure;
            result.maximum_entry = fmaxq(result.maximum_entry, enclosure);
            exact = exact && difference.exact && difference.no_underflow
                && finiteq(residual) != 0 && finiteq(arithmetic) != 0
                && finiteq(enclosure) != 0
                && enclosure >= static_cast<Binary128>(0.0);
            ++result.center_differences;
            ++result.bound_constructions;
            ++result.entries;
        }
        ++result.canonical_products;
    }
    for (std::size_t row = 0U; row < rows; ++row) {
        Binary128 row_sum = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < rows; ++column) {
            row_sum = al_r20_r38_up_add(
                row_sum, result.value[row * rows + column]);
            ++result.row_sum_terms;
        }
        result.maximum_row_sum = fmaxq(result.maximum_row_sum, row_sum);
        exact = exact && finiteq(row_sum) != 0;
    }
    const ALR20R63NCanonical canonical = al_r20_r63n_canonical(
        matrix, tangent, rows, columns, sigma);
    result.canonical_root = canonical.root;
    result.exact = exact && canonical.exact
        && result.canonical_products == rows
        && result.entries == rows * rows
        && result.center_differences == rows * rows
        && result.bound_constructions == rows * rows
        && result.row_sum_terms == rows * rows;
    std::ostringstream material;
    material << result.exact << ':' << result.dimension << ':'
        << result.canonical_products << ':' << result.entries << ':'
        << result.center_differences << ':' << result.bound_constructions
        << ':' << result.row_sum_terms << ':'
        << al_binary128_hex(result.maximum_entry) << ':'
        << al_binary128_hex(result.maximum_row_sum) << ':'
        << al_r20_q_scalar_root(result.value) << ':'
        << result.canonical_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63OComparison {
    bool exact = false;
    bool contained = false;
    std::size_t dimension = 0U;
    std::size_t transport_terms = 0U;
    std::size_t contained_components = 0U;
    Binary128 raw_minimum_slack = HUGE_VALQ;
    Binary128 transported_minimum_slack = HUGE_VALQ;
    Binary128 maximum_model_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_center_residual = static_cast<Binary128>(0.0);
    std::vector<Binary128> model_bound;
    std::string raw_root;
    std::string root;
};

ALR20R63OComparison al_r20_r63o_compare(
    const ALR20R63ODefect& defect,
    const std::vector<Binary128>& vector,
    const ALR20R63MProduct& dense,
    const ALR20R63NProduct& direct) {
    ALR20R63OComparison result;
    result.dimension = vector.size();
    const std::size_t dimension = result.dimension;
    if (!defect.exact || defect.dimension != dimension
        || defect.value.size() != dimension * dimension
        || !dense.exact || !direct.exact
        || dense.value.size() != dimension
        || direct.value.size() != dimension) return result;
    const ALR20R63NComparison raw = al_r20_r63n_compare(dense, direct);
    result.raw_root = raw.root;
    result.raw_minimum_slack = raw.minimum_slack;
    result.model_bound.resize(dimension);
    bool exact = raw.exact;
    std::ostringstream component_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        Binary128 transported = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            transported = al_r20_r38_up_add(transported,
                al_r20_r38_up_multiply(
                    defect.value[row * dimension + column],
                    fabsq(vector[column])));
            ++result.transport_terms;
        }
        result.model_bound[row] = transported;
        result.maximum_model_bound = fmaxq(
            result.maximum_model_bound, transported);
        const ALR20R38Pair difference = al_r20_r38_two_sum(
            dense.value[row], -direct.value[row]);
        const Binary128 residual = al_r20_r38_up_add(
            fabsq(difference.high), fabsq(difference.low));
        const Binary128 combined = al_r20_r38_up_add(dense.bound[row],
            al_r20_r38_up_add(direct.bound[row], transported));
        const bool component_contained = residual <= combined;
        exact = exact && difference.exact && difference.no_underflow
            && finiteq(transported) != 0 && finiteq(residual) != 0
            && finiteq(combined) != 0;
        result.maximum_center_residual = fmaxq(
            result.maximum_center_residual, residual);
        result.transported_minimum_slack = fminq(
            result.transported_minimum_slack, combined - residual);
        if (component_contained) ++result.contained_components;
        component_material << difference.exact << ':'
            << component_contained << ':' << al_binary128_hex(transported)
            << ':' << al_binary128_hex(residual) << ':'
            << al_binary128_hex(combined) << ';';
    }
    result.contained = result.contained_components == dimension;
    result.exact = exact && result.transport_terms == dimension * dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.dimension << ':' << result.transport_terms << ':'
        << result.contained_components << ':'
        << al_binary128_hex(result.raw_minimum_slack) << ':'
        << al_binary128_hex(result.transported_minimum_slack) << ':'
        << al_binary128_hex(result.maximum_model_bound) << ':'
        << al_binary128_hex(result.maximum_center_residual) << ':'
        << al_r20_q_scalar_root(result.model_bound) << ':'
        << result.raw_root << ':' << sha256_hex(component_material.str());
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63OStep {
    bool exact = false;
    std::size_t iteration = 0U;
    Binary128 rho = static_cast<Binary128>(0.0);
    Binary128 denominator = static_cast<Binary128>(0.0);
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 beta = static_cast<Binary128>(0.0);
    Binary128 residual_drift = static_cast<Binary128>(0.0);
    ALR20R63OComparison product_comparison;
    ALR20R63ISignCertificate certificate;
    std::string root;
};

struct ALR20R63OLane {
    bool exact = false;
    bool finite = false;
    bool correspondence = false;
    bool passed = false;
    std::size_t dimension = 0U;
    std::size_t completed_iterations = 0U;
    std::size_t failure_iteration = 0U;
    std::size_t rectangular_products = 0U;
    std::size_t direct_inner_dots = 0U;
    std::size_t direct_outer_dots = 0U;
    std::size_t direct_inner_terms = 0U;
    std::size_t direct_outer_terms = 0U;
    std::size_t direct_scale_products = 0U;
    std::size_t dense_comparator_dots = 0U;
    std::size_t transport_terms = 0U;
    std::size_t transported_components = 0U;
    std::size_t contained_components = 0U;
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
    std::size_t certificate_residual_dots = 0U;
    std::size_t certificate_image_dots = 0U;
    std::size_t first_passing_iteration = 0U;
    Binary128 minimum_error = HUGE_VALQ;
    Binary128 maximum_residual_drift = static_cast<Binary128>(0.0);
    Binary128 raw_minimum_slack = HUGE_VALQ;
    Binary128 transported_minimum_slack = HUGE_VALQ;
    Binary128 maximum_model_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128> solution;
    std::vector<Binary128> residual;
    std::vector<Binary128> direction;
    std::vector<ALR20R63OStep> steps;
    std::string initial_comparison_root;
    std::string failure_stage;
    std::string root;
};

ALR20R63OLane al_r20_r63o_lane(
    const std::vector<Binary128>& upper,
    const std::vector<std::size_t>& permutation,
    Binary128 inverse_scale,
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& tangent,
    std::size_t tangent_columns,
    Binary128 sigma,
    const ALR20R63ODefect& defect,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& initial,
    const ALR20R63ISignCertificate& baseline,
    const std::vector<int>& reference_signs,
    const std::vector<Binary128>& reference_solution) {
    ALR20R63OLane result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension
        || tangent.size() != dimension * tangent_columns
        || !defect.exact || defect.dimension != dimension
        || initial.size() != dimension || !baseline.exact || baseline.passed
        || reference_signs.size() != dimension
        || reference_solution.size() != dimension
        || !(sigma > static_cast<Binary128>(0.0))) return result;
    auto record_product = [&result](const ALR20R63NProduct& direct,
                              const ALR20R63MProduct& dense,
                              const ALR20R63OComparison& comparison) {
        ++result.rectangular_products;
        result.direct_inner_dots += direct.inner_dots;
        result.direct_outer_dots += direct.outer_dots;
        result.direct_inner_terms += direct.inner_terms;
        result.direct_outer_terms += direct.outer_terms;
        result.direct_scale_products += direct.scale_products;
        result.dense_comparator_dots += dense.dots;
        result.transport_terms += comparison.transport_terms;
        result.transported_components += comparison.dimension;
        result.contained_components += comparison.contained_components;
        result.raw_minimum_slack = fminq(
            result.raw_minimum_slack, comparison.raw_minimum_slack);
        result.transported_minimum_slack = fminq(
            result.transported_minimum_slack,
            comparison.transported_minimum_slack);
        result.maximum_model_bound = fmaxq(
            result.maximum_model_bound, comparison.maximum_model_bound);
    };
    result.solution = initial;
    const ALR20R63NProduct initial_direct = al_r20_r63n_product(
        tangent, dimension, tangent_columns, sigma, result.solution);
    const ALR20R63MProduct initial_dense = al_r20_r63m_product(
        matrix, result.solution);
    const ALR20R63OComparison initial_comparison = al_r20_r63o_compare(
        defect, result.solution, initial_dense, initial_direct);
    record_product(initial_direct, initial_dense, initial_comparison);
    result.initial_comparison_root = initial_comparison.root;
    bool exact = initial_direct.exact && initial_dense.exact
        && initial_comparison.exact;
    bool finite = exact && initial_comparison.contained;
    bool correspondence = initial_comparison.contained;
    if (!exact) {
        result.failure_iteration = 0U;
        result.failure_stage = "initial_product";
    } else if (!correspondence) {
        result.failure_iteration = 0U;
        result.failure_stage = "initial_transport_not_contained";
    }
    result.residual.resize(dimension);
    if (finite) {
        for (std::size_t row = 0U; row < dimension; ++row) {
            const ALR20R38Dot2 dot = al_r20_r38_dot2(
                {rhs[row], -initial_direct.value[row]},
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
        ALR20R63OStep step;
        step.iteration = iteration;
        step.rho = rho.value;
        const ALR20R63NProduct direct = al_r20_r63n_product(
            tangent, dimension, tangent_columns, sigma, result.direction);
        const ALR20R63MProduct dense = al_r20_r63m_product(
            matrix, result.direction);
        step.product_comparison = al_r20_r63o_compare(
            defect, result.direction, dense, direct);
        record_product(direct, dense, step.product_comparison);
        if (!direct.exact || !dense.exact
            || !step.product_comparison.exact) {
            finite = false;
            exact = false;
            result.failure_iteration = iteration;
            result.failure_stage = "matrix_product";
            break;
        }
        if (!step.product_comparison.contained) {
            finite = false;
            correspondence = false;
            result.failure_iteration = iteration;
            result.failure_stage = "transport_not_contained";
            break;
        }
        const ALR20R63MScalar denominator = al_r20_r63m_scalar(
            result.direction, direct.value);
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
                {static_cast<Binary128>(1.0), direct.value[row]});
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
        step.certificate = al_r20_r63i_sign_certificate(
            matrix, inverse, rhs, result.solution, reference_signs,
            baseline.rho_bound, reference_solution);
        result.certificate_residual_dots += step.certificate.residual_dots;
        result.certificate_image_dots += step.certificate.image_dots;
        if (!step.certificate.exact) {
            finite = false;
            exact = false;
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
                exact = false;
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
            + step.product_comparison.root + ':'
            + sha256_hex(update_material.str()) + ':'
            + step.certificate.root + ':' + preconditioner_root + ':'
            + rho_new_root + ':' + direction_root);
        exact = exact && step.exact;
        result.steps.push_back(std::move(step));
    }
    result.finite = finite;
    result.correspondence = correspondence
        && result.contained_components == result.transported_components;
    const bool complete = result.completed_iterations == 8U;
    result.passed = finite && complete && result.correspondence
        && result.first_passing_iteration != 0U;
    result.exact = exact && ((finite && complete
            && result.steps.size() == 8U)
        || (!finite && !result.failure_stage.empty()));
    std::ostringstream material;
    material << result.exact << ':' << result.finite << ':'
        << result.correspondence << ':' << result.passed << ':'
        << result.dimension << ':' << result.completed_iterations << ':'
        << result.failure_iteration << ':' << result.failure_stage << ':'
        << result.rectangular_products << ':' << result.direct_inner_dots
        << ':' << result.direct_outer_dots << ':'
        << result.direct_inner_terms << ':' << result.direct_outer_terms
        << ':' << result.direct_scale_products << ':'
        << result.dense_comparator_dots << ':' << result.transport_terms
        << ':' << result.transported_components << ':'
        << result.contained_components << ':'
        << result.initial_residual_dots << ':'
        << result.preconditioner_solves << ':' << result.forward_terms << ':'
        << result.backward_terms << ':' << result.divisions << ':'
        << result.rho_dots << ':' << result.denominator_dots << ':'
        << result.solution_update_dots << ':'
        << result.residual_update_dots << ':'
        << result.direction_update_dots << ':'
        << result.certificate_residual_dots << ':'
        << result.certificate_image_dots << ':'
        << result.first_passing_iteration << ':'
        << al_binary128_hex(result.minimum_error) << ':'
        << al_binary128_hex(result.maximum_residual_drift) << ':'
        << al_binary128_hex(result.raw_minimum_slack) << ':'
        << al_binary128_hex(result.transported_minimum_slack) << ':'
        << al_binary128_hex(result.maximum_model_bound) << ':'
        << al_r20_q_scalar_root(result.solution) << ':'
        << al_r20_q_scalar_root(result.residual) << ':'
        << al_r20_q_scalar_root(result.direction) << ':'
        << result.initial_comparison_root << '|';
    for (const ALR20R63OStep& step : result.steps)
        material << step.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63o_classify(bool apparatus, bool transport,
    bool wide, bool exported) {
    if (!apparatus) return "CANONICAL_DEFECT_TRANSPORT_APPARATUS_REJECTED";
    if (!transport) return "CANONICAL_DEFECT_TRANSPORT_REJECTED";
    if (!wide) return "RETAINED_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED";
    if (!exported)
        return "EXPORTED_FACTOR_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED";
    return "EXPORTED_FACTOR_WIDE_TRANSPORTED_MATRIX_FREE_PCG_CANDIDATE";
}

struct ALR20R63OControls {
    bool exact = false;
    bool defect = false;
    bool transport = false;
    bool zeroed = false;
    bool orientation = false;
    bool pcg = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63OControls al_r20_r63o_controls() {
    ALR20R63OControls result;
    const std::vector<Binary128> tangent{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5),
        static_cast<Binary128>(0.0), static_cast<Binary128>(0.25),
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5)};
    std::vector<Binary128> matrix{
        static_cast<Binary128>(0.625), static_cast<Binary128>(0.375),
        static_cast<Binary128>(0.375), static_cast<Binary128>(0.65625)};
    matrix[0U] += static_cast<Binary128>(0.0625);
    const Binary128 sigma = static_cast<Binary128>(0.5);
    const ALR20R63ODefect defect = al_r20_r63o_defect(
        matrix, tangent, 2U, 3U, sigma);
    result.defect = defect.exact && defect.entries == 4U
        && defect.value[0U] >= static_cast<Binary128>(0.0625);
    const std::vector<Binary128> vector{
        static_cast<Binary128>(2.0), static_cast<Binary128>(-3.0)};
    const ALR20R63MProduct dense = al_r20_r63m_product(matrix, vector);
    const ALR20R63NProduct direct = al_r20_r63n_product(
        tangent, 2U, 3U, sigma, vector);
    const ALR20R63OComparison transported = al_r20_r63o_compare(
        defect, vector, dense, direct);
    result.transport = transported.exact && transported.contained
        && transported.raw_minimum_slack
            < static_cast<Binary128>(0.0)
        && transported.transported_minimum_slack
            >= static_cast<Binary128>(0.0);
    ALR20R63ODefect zeroed_defect = defect;
    zeroed_defect.value[0U] = static_cast<Binary128>(0.0);
    const ALR20R63OComparison zeroed = al_r20_r63o_compare(
        zeroed_defect, vector, dense, direct);
    result.zeroed = zeroed.exact && !zeroed.contained;
    const std::vector<Binary128> square{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5),
        static_cast<Binary128>(0.25), static_cast<Binary128>(1.0)};
    const std::vector<Binary128> square_transposed{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.25),
        static_cast<Binary128>(0.5), static_cast<Binary128>(1.0)};
    const std::vector<Binary128> orientation_vector{
        static_cast<Binary128>(1.0), static_cast<Binary128>(-2.0)};
    const ALR20R63NProduct oriented = al_r20_r63n_product(
        square, 2U, 2U, static_cast<Binary128>(1.0), orientation_vector);
    const ALR20R63NProduct reversed = al_r20_r63n_product(
        square_transposed, 2U, 2U, static_cast<Binary128>(1.0),
        orientation_vector);
    result.orientation = oriented.exact && reversed.exact
        && oriented.root != reversed.root && oriented.value != reversed.value;

    constexpr std::size_t dimension = 8U;
    std::vector<Binary128> control_matrix(dimension * dimension,
        static_cast<Binary128>(0.0));
    std::vector<Binary128> control_inverse(dimension * dimension,
        static_cast<Binary128>(0.0));
    std::vector<Binary128> control_tangent(dimension * dimension,
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
        control_matrix[index * dimension + index] = diagonal;
        control_inverse[index * dimension + index] =
            static_cast<Binary128>(1.0) / diagonal;
        control_tangent[index * dimension + index] = sqrtq(diagonal);
        reference[index] = static_cast<Binary128>(1.0) / diagonal;
        permutation[index] = index;
        upper[index * dimension + index] = upper_diagonal;
    }
    const ALR20R63ODefect control_defect = al_r20_r63o_defect(
        control_matrix, control_tangent, dimension, dimension,
        static_cast<Binary128>(1.0));
    const ALR20R63JDefect inverse_defect = al_r20_r63j_defect(
        control_matrix, control_inverse);
    const ALR20R63ISignCertificate baseline =
        al_r20_r63i_sign_certificate(control_matrix, control_inverse, rhs,
            initial, signs, inverse_defect.rho_bound, reference);
    const ALR20R63OLane lane = al_r20_r63o_lane(
        upper, permutation, static_cast<Binary128>(1.0), control_matrix,
        control_inverse, control_tangent, dimension,
        static_cast<Binary128>(1.0), control_defect, rhs, initial, baseline,
        signs, reference);
    result.pcg = control_defect.exact && lane.exact && lane.finite
        && lane.correspondence && lane.passed
        && lane.completed_iterations == 8U && lane.steps.size() == 8U;
    std::vector<Binary128> mutated = defect.value;
    mutated[0U] = nextafterq(mutated[0U], HUGE_VALQ);
    result.mutation = al_r20_q_scalar_root(mutated)
        != al_r20_q_scalar_root(defect.value);
    result.classifier =
        al_r20_r63o_classify(false, true, true, true)
            == "CANONICAL_DEFECT_TRANSPORT_APPARATUS_REJECTED"
        && al_r20_r63o_classify(true, false, true, true)
            == "CANONICAL_DEFECT_TRANSPORT_REJECTED"
        && al_r20_r63o_classify(true, true, false, true)
            == "RETAINED_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED"
        && al_r20_r63o_classify(true, true, true, false)
            == "EXPORTED_FACTOR_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED"
        && al_r20_r63o_classify(true, true, true, true)
            == "EXPORTED_FACTOR_WIDE_TRANSPORTED_MATRIX_FREE_PCG_CANDIDATE";
    result.exact = result.defect && result.transport && result.zeroed
        && result.orientation && result.pcg && result.mutation
        && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + defect.root + ':' + transported.root + ':' + zeroed.root + ':'
        + oriented.root + ':' + reversed.root + ':' + control_defect.root
        + ':' + lane.root + ':' + al_r20_q_scalar_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_canonical_defect_transport_controls() {
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
    const ALR20R63OControls controls = al_r20_r63o_controls();
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
    const bool tangent_exact = tangent.exact && tangent.rows == 102U
        && tangent.columns == 315U && tangent.nonzero_values == 17748U
        && tangent.value_root == AL_R20_R63N_TANGENT_VALUE_SHA256
        && tangent.row_root == AL_R20_R63N_TANGENT_ROW_SHA256;
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows);
    std::vector<Binary128> transposed;
    std::vector<double> permuted_d;
    if (tangent_exact && permutation_exact) {
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
        && full_profile.exact && tangent_exact && permutation_exact
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
    const Binary128 sigma = static_cast<Binary128>(1.0) / inverse_scale;
    const ALR20R63IQSolve wide_solve = factor_exact
        ? al_r20_r63i_q_solve(wide_factor.r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IQSolve export_wide_solve = factor_exact
        ? al_r20_r63i_q_solve(exported_r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
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
    const bool r63i_exact = factor_exact && reference_exact
        && wide_solve.root == AL_R20_R63J_WIDE_SOLVE_SHA256
        && export_wide_solve.root == AL_R20_R63J_EXPORT_WIDE_SOLVE_SHA256
        && wide_baseline.root == AL_R20_R63J_WIDE_CERTIFICATE_SHA256
        && export_baseline.root
            == AL_R20_R63J_EXPORT_WIDE_CERTIFICATE_SHA256;
    const ALR20R63MLane parent_wide = r63i_exact
        ? al_r20_r63m_lane(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, wide_solve.solution, wide_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63MLane{};
    const ALR20R63MLane parent_export = r63i_exact
        ? al_r20_r63m_lane(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, export_wide_solve.solution, export_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63MLane{};
    const bool parent_r63m_exact = r63i_exact
        && parent_wide.exact && parent_wide.passed
        && parent_export.exact && parent_export.passed
        && parent_wide.root == AL_R20_R63N_WIDE_PARENT_SHA256
        && parent_export.root == AL_R20_R63N_EXPORT_PARENT_SHA256;
    const ALR20R63NCanonical canonical = parent_r63m_exact
        ? al_r20_r63n_canonical(factor_capture.matrix, tangent.values,
            tangent.rows, tangent.columns, sigma)
        : ALR20R63NCanonical{};
    const ALR20R63NLane raw_wide = parent_r63m_exact && canonical.exact
        && canonical.contained
        ? al_r20_r63n_lane(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse, tangent.values,
            tangent.columns, sigma, factor_capture.rhs, wide_solve.solution,
            wide_baseline, reference_signs, reference_certificate.solution)
        : ALR20R63NLane{};
    const ALR20R63NLane raw_export = parent_r63m_exact && canonical.exact
        && canonical.contained
        ? al_r20_r63n_lane(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse, tangent.values,
            tangent.columns, sigma, factor_capture.rhs,
            export_wide_solve.solution, export_baseline, reference_signs,
            reference_certificate.solution)
        : ALR20R63NLane{};
    const bool parent_r63n_exact = parent_r63m_exact
        && canonical.exact && canonical.contained
        && canonical.root == AL_R20_R63O_CANONICAL_PARENT_SHA256
        && raw_wide.exact && !raw_wide.finite && !raw_wide.correspondence
        && raw_export.exact && !raw_export.finite
        && !raw_export.correspondence
        && raw_wide.root == AL_R20_R63O_WIDE_FAILURE_SHA256
        && raw_export.root == AL_R20_R63O_EXPORT_FAILURE_SHA256;
    const ALR20R63ODefect defect = parent_r63n_exact
        ? al_r20_r63o_defect(factor_capture.matrix, tangent.values,
            tangent.rows, tangent.columns, sigma)
        : ALR20R63ODefect{};
    const ALR20R63OLane wide_transported = defect.exact
        ? al_r20_r63o_lane(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse, tangent.values,
            tangent.columns, sigma, defect, factor_capture.rhs,
            wide_solve.solution, wide_baseline, reference_signs,
            reference_certificate.solution)
        : ALR20R63OLane{};
    const ALR20R63OLane export_transported = defect.exact
        ? al_r20_r63o_lane(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse, tangent.values,
            tangent.columns, sigma, defect, factor_capture.rhs,
            export_wide_solve.solution, export_baseline, reference_signs,
            reference_certificate.solution)
        : ALR20R63OLane{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool defect_work_exact = defect.canonical_products == 102U
        && defect.entries == 10404U
        && defect.center_differences == 10404U
        && defect.bound_constructions == 10404U
        && defect.row_sum_terms == 10404U;
    auto lane_work_exact = [](const ALR20R63OLane& lane) {
        if (!lane.finite)
            return lane.exact && !lane.failure_stage.empty();
        return lane.completed_iterations == 8U
            && lane.rectangular_products == 9U
            && lane.direct_inner_dots == 2835U
            && lane.direct_outer_dots == 918U
            && lane.direct_inner_terms == 289170U
            && lane.direct_outer_terms == 289170U
            && lane.direct_scale_products == 918U
            && lane.dense_comparator_dots == 918U
            && lane.transport_terms == 93636U
            && lane.transported_components == 918U
            && lane.contained_components == 918U
            && lane.initial_residual_dots == 102U
            && lane.preconditioner_solves == 8U
            && lane.forward_terms == 41208U
            && lane.backward_terms == 41208U && lane.divisions == 1632U
            && lane.rho_dots == 8U && lane.denominator_dots == 8U
            && lane.solution_update_dots == 816U
            && lane.residual_update_dots == 816U
            && lane.direction_update_dots == 714U
            && lane.certificate_residual_dots == 816U
            && lane.certificate_image_dots == 816U;
    };
    const bool work_exact = defect_work_exact
        && lane_work_exact(wide_transported)
        && lane_work_exact(export_transported);
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63n_exact && defect.exact && wide_transported.exact
        && export_transported.exact && work_exact && lifecycle_exact;
    const bool transport_exact = wide_transported.correspondence
        && export_transported.correspondence;
    const std::string route = al_r20_r63o_classify(apparatus_exact,
        transport_exact, wide_transported.passed, export_transported.passed);
    const bool exact = apparatus_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63o-canonical-defect-transport.v1|"
        << AL_R20_R63O_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << canonical.root << ':' << raw_wide.root
        << ':' << raw_export.root << ':' << defect.root << ':'
        << wide_transported.root << ':' << export_transported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    auto lane_json = [](const char* name, const ALR20R63OLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"finite\":" << (lane.finite ? "true" : "false")
            << ",\"correspondence\":"
            << (lane.correspondence ? "true" : "false")
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
            << "\",\"raw_minimum_slack\":\""
            << al_binary128_text(lane.raw_minimum_slack)
            << "\",\"transported_minimum_slack\":\""
            << al_binary128_text(lane.transported_minimum_slack)
            << "\",\"maximum_model_bound\":\""
            << al_binary128_text(lane.maximum_model_bound)
            << "\",\"steps\":[";
        for (std::size_t index = 0U; index < lane.steps.size(); ++index) {
            if (index != 0U) stream << ',';
            const ALR20R63OStep& step = lane.steps[index];
            stream << "{\"iteration\":" << step.iteration
                << ",\"passed\":"
                << (step.certificate.passed ? "true" : "false")
                << ",\"error\":\""
                << al_binary128_text(step.certificate.error)
                << "\",\"positive\":" << step.certificate.positive
                << ",\"negative\":" << step.certificate.negative
                << ",\"unresolved\":" << step.certificate.unresolved
                << ",\"transport_contained\":"
                << (step.product_comparison.contained ? "true" : "false")
                << ",\"residual_drift\":\""
                << al_binary128_text(step.residual_drift)
                << "\",\"root\":\"" << step.root << "\"}";
        }
        stream << "],\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63o_canonical_defect_transport.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63n_sha256\":\""
        << AL_R20_R63O_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63n_exact\":"
        << (parent_r63n_exact ? "true" : "false")
        << ",\"raw_parent\":{\"wide_slack\":\""
        << al_binary128_text(raw_wide.minimum_product_slack)
        << "\",\"export_slack\":\""
        << al_binary128_text(raw_export.minimum_product_slack) << "\"}"
        << ",\"defect\":{\"exact\":"
        << (defect.exact ? "true" : "false")
        << ",\"entries\":" << defect.entries
        << ",\"maximum_entry\":\""
        << al_binary128_text(defect.maximum_entry)
        << "\",\"maximum_row_sum\":\""
        << al_binary128_text(defect.maximum_row_sum)
        << "\",\"root\":\"" << defect.root << "\"}"
        << ",\"lanes\":{" << lane_json("wide", wide_transported)
        << ',' << lane_json("export_wide", export_transported) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"transport_exact\":"
        << (transport_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"candidate_factor_solves\":"
        << (wide_transported.preconditioner_solves
            + export_transported.preconditioner_solves)
        << ",\"candidate_rectangular_products\":"
        << (wide_transported.rectangular_products
            + export_transported.rectangular_products)
        << ",\"candidate_rectangular_dot2_reductions\":"
        << (wide_transported.direct_inner_dots
            + wide_transported.direct_outer_dots
            + export_transported.direct_inner_dots
            + export_transported.direct_outer_dots)
        << ",\"candidate_rectangular_terms\":"
        << (wide_transported.direct_inner_terms
            + wide_transported.direct_outer_terms
            + export_transported.direct_inner_terms
            + export_transported.direct_outer_terms)
        << ",\"verification_transport_terms\":"
        << (wide_transported.transport_terms
            + export_transported.transport_terms)
        << ",\"defect_inflation_factor\":1"
        << ",\"dense_comparator_work_excluded\":true"
        << ",\"zero_elision_work\":0,\"csr_constructions\":0"
        << ",\"new_factorizations\":0,\"new_inverse_constructions\":0"
        << ",\"rank_actions\":0,\"row_drops\":0"
        << ",\"regularizations\":0,\"replacement_applies\":0"
        << ",\"following_transitions\":0,\"state_updates\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
