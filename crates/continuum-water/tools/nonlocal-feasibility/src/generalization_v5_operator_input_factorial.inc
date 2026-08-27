namespace {

constexpr const char* AL_R20_R63ZC_PARENT_RESULT_SHA256 =
    "acfe6bafe9569eba101ed1e352cc01e149023e295bd8c082bdac58e49a709ebf";
constexpr const char* AL_R20_R63ZC_PARENT_STDOUT_SHA256 =
    "a10567198c7abc54e6702724de65ec80c5fc06b14d6164e4fc3374f1308c1176";
constexpr const char* AL_R20_R63ZC_R63Y_STDOUT_SHA256 =
    "7427ecdb3c4c7c95900bc6046545d298a674be42865936bad543c7e0191f4372";
constexpr const char* AL_R20_R63ZC_R63Z_STDOUT_SHA256 =
    "de22f5f638fa434ca61857f09b5b9f6988d40f33a5e7fcec4f43a4b9b21bda54";
constexpr const char* AL_R20_R63ZC_R63ZA_STDOUT_SHA256 =
    "ed2511615eb0b488b4337133860971f86a0d7ad48c757db49c5fcda32b325f72";
constexpr const char* AL_R20_R63ZC_R63X_SEMANTIC_SHA256 =
    "904614d78c1948f4af836167936bdc528a0c69a0ded7e85cfe9ae264be395a65";
constexpr const char* AL_R20_R63ZC_COMBINED_COMPARATOR_SHA256 =
    "7db8a84ee9cee6506533b4d634d051ee9dfb8e56b23f8cbd06222ddba3b1c8e3";
constexpr const char* AL_R20_R63ZC_FINITE_TRANSACTION_SHA256 =
    "7aea732146e20ee40a77fc06c0311911429687503ddaadf07ce5ab666249bf96";
constexpr const char* AL_R20_R63ZC_FACTOR_SHA256 =
    "a7a85789364f5af91aaf71e1531759713c95ec085e48b7ca614daa524c3a4f85";
constexpr const char* AL_R20_R63ZC_PERMUTATION_SHA256 =
    "335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f";
constexpr const char* AL_R20_R63ZC_TANGENT_SHA256 =
    "114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1";
constexpr const char* AL_R20_R63ZC_R63X_PROFILE_SHA256 =
    "0505cdb98db31a7f087cbc23290b5e0adc964dce05f4e73a4429083d860e97d0";
constexpr const char* AL_R20_R63ZC_R63X_EXPORTED_SHA256 =
    "907c3780ed8e41ae64a74fc60df856a1368348ee96dccb6fe12bb97f73c3a1d1";
constexpr const char* AL_R20_R63ZC_R63X_SOLUTION_SET_SHA256 =
    "d0b42562f5ed45efdf2d454f1c1f6ec93f587fe8045943517f9af076d279c98e";
constexpr std::size_t AL_R20_R63ZC_DIMENSION = 102U;
constexpr std::size_t AL_R20_R63ZC_TANGENT_COLUMNS = 315U;

enum class ALR20R63ZCOperator {
    tangent,
    common,
};

const char* al_r20_r63zc_operator_name(ALR20R63ZCOperator value) {
    return value == ALR20R63ZCOperator::tangent ? "tangent" : "common";
}

struct ALR20R63ZCWork {
    std::size_t certificates = 0U;
    std::size_t operator_products = 0U;
    std::size_t tangent_inner_dots = 0U;
    std::size_t tangent_outer_dots = 0U;
    std::size_t tangent_inner_terms = 0U;
    std::size_t tangent_outer_terms = 0U;
    std::size_t tangent_scale_products = 0U;
    std::size_t common_dots = 0U;
    std::size_t common_terms = 0U;
    std::size_t factor_solves = 0U;
    std::size_t factor_terms = 0U;
    std::size_t factor_divisions = 0U;
    std::size_t scalar_dots = 0U;
    std::size_t scalar_divisions = 0U;
    std::size_t solution_updates = 0U;
    std::size_t residual_updates = 0U;
    std::size_t direction_updates = 0U;
    std::size_t adaptive_stops = 0U;
};

std::string al_r20_r63zc_work_root(const ALR20R63ZCWork& work);

std::string al_r20_r63zc_operator_root(ALR20R63ZCOperator kind,
    const std::string& tangent_root, std::size_t tangent_columns,
    const std::string& sigma, const std::string& common_root) {
    if (kind == ALR20R63ZCOperator::tangent)
        return sha256_hex(std::string("tangent:") + tangent_root + ':'
            + std::to_string(tangent_columns) + ':' + sigma);
    return sha256_hex(std::string("common:") + common_root);
}

struct ALR20R63ZCProduct {
    bool exact = false;
    std::vector<Binary128> value;
    ALR20R63ZCWork work;
    std::string root;
};

ALR20R63ZCProduct al_r20_r63zc_product(ALR20R63ZCOperator kind,
    const std::vector<Binary128>& tangent, std::size_t tangent_columns,
    Binary128 sigma, const ALR20R63ZArtifact& common,
    const std::vector<Binary128>& input) {
    ALR20R63ZCProduct result;
    if (kind == ALR20R63ZCOperator::tangent) {
        const ALR20R63NProduct product = al_r20_r63n_product(
            tangent, input.size(), tangent_columns, sigma, input);
        result.exact = product.exact;
        result.value = product.value;
        result.work.operator_products = 1U;
        result.work.tangent_inner_dots = product.inner_dots;
        result.work.tangent_outer_dots = product.outer_dots;
        result.work.tangent_inner_terms = product.inner_terms;
        result.work.tangent_outer_terms = product.outer_terms;
        result.work.tangent_scale_products = product.scale_products;
        result.root = sha256_hex(std::string("tangent:") + product.root);
    } else {
        std::vector<Binary128> matrix(common.identity.dimension
            * common.identity.dimension);
        bool matrix_exact = al_r20_r63z_artifact_valid(common)
            && common.identity.dimension == input.size();
        for (std::size_t index = 0U;
             index < matrix.size() && matrix_exact; ++index) {
            matrix[index] = static_cast<Binary128>(
                common.components[2U * index])
                + static_cast<Binary128>(common.components[2U * index + 1U]);
            matrix_exact = finiteq(matrix[index]) != 0;
        }
        const ALR20R63MProduct product = matrix_exact
            ? al_r20_r63m_product(matrix, input) : ALR20R63MProduct{};
        result.exact = matrix_exact && product.exact;
        result.value = product.value;
        result.work.operator_products = 1U;
        result.work.common_dots = product.dots;
        result.work.common_terms = product.dots * input.size();
        result.root = sha256_hex(std::string("common:") + common.root
            + ':' + product.root);
    }
    return result;
}

void al_r20_r63zc_add_work(
    ALR20R63ZCWork& target, const ALR20R63ZCWork& source) {
    target.certificates += source.certificates;
    target.operator_products += source.operator_products;
    target.tangent_inner_dots += source.tangent_inner_dots;
    target.tangent_outer_dots += source.tangent_outer_dots;
    target.tangent_inner_terms += source.tangent_inner_terms;
    target.tangent_outer_terms += source.tangent_outer_terms;
    target.tangent_scale_products += source.tangent_scale_products;
    target.common_dots += source.common_dots;
    target.common_terms += source.common_terms;
    target.factor_solves += source.factor_solves;
    target.factor_terms += source.factor_terms;
    target.factor_divisions += source.factor_divisions;
    target.scalar_dots += source.scalar_dots;
    target.scalar_divisions += source.scalar_divisions;
    target.solution_updates += source.solution_updates;
    target.residual_updates += source.residual_updates;
    target.direction_updates += source.direction_updates;
    target.adaptive_stops += source.adaptive_stops;
}

struct ALR20R63ZCState {
    bool exact = false;
    std::size_t index = 0U;
    std::vector<Binary128> solution;
    std::vector<Binary128> residual;
    std::vector<Binary128> direction;
    std::vector<Binary128> preconditioned;
    Binary128 rho = static_cast<Binary128>(0.0);
    Binary128 denominator = static_cast<Binary128>(0.0);
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 beta = static_cast<Binary128>(0.0);
    std::string product_root;
    std::string solve_root;
    std::string root;
};

bool al_r20_r63zc_vector_valid(
    const std::vector<Binary128>& values, std::size_t dimension) {
    if (dimension == 0U || values.size() != dimension) return false;
    return std::all_of(values.begin(), values.end(), [](Binary128 value) {
        return al_r20_r38_normal_or_zero(value);
    });
}

ALR20R63ZCState al_r20_r63zc_state(std::size_t index,
    const std::vector<Binary128>& solution,
    const std::vector<Binary128>& residual,
    const std::vector<Binary128>& direction,
    const std::vector<Binary128>& preconditioned, Binary128 rho,
    Binary128 denominator, Binary128 alpha, Binary128 beta,
    const std::string& product_root, const std::string& solve_root) {
    ALR20R63ZCState result;
    result.index = index;
    result.solution = solution;
    result.residual = residual;
    result.direction = direction;
    result.preconditioned = preconditioned;
    result.rho = rho;
    result.denominator = denominator;
    result.alpha = alpha;
    result.beta = beta;
    result.product_root = product_root;
    result.solve_root = solve_root;
    const std::size_t dimension = solution.size();
    result.exact = al_r20_r63zc_vector_valid(solution, dimension)
        && al_r20_r63zc_vector_valid(residual, dimension)
        && al_r20_r63zc_vector_valid(direction, dimension)
        && al_r20_r63zc_vector_valid(preconditioned, dimension)
        && al_r20_r38_normal_or_zero(rho)
        && al_r20_r38_normal_or_zero(denominator)
        && al_r20_r38_normal_or_zero(alpha)
        && al_r20_r38_normal_or_zero(beta)
        && !product_root.empty() && !solve_root.empty();
    std::ostringstream material;
    material << result.exact << ':' << result.index << ':'
        << al_r20_q_scalar_root(solution) << ':'
        << al_r20_q_scalar_root(residual) << ':'
        << al_r20_q_scalar_root(direction) << ':'
        << al_r20_q_scalar_root(preconditioned) << ':'
        << al_binary128_hex(rho) << ':' << al_binary128_hex(denominator)
        << ':' << al_binary128_hex(alpha) << ':' << al_binary128_hex(beta)
        << ':' << product_root << ':' << solve_root;
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_r63zc_state_valid(const ALR20R63ZCState& state) {
    const ALR20R63ZCState rebuilt = al_r20_r63zc_state(state.index,
        state.solution, state.residual, state.direction,
        state.preconditioned, state.rho, state.denominator,
        state.alpha, state.beta, state.product_root, state.solve_root);
    return state.exact && rebuilt.exact && state.root == rebuilt.root;
}

struct ALR20R63ZCTransaction {
    bool exact = false;
    bool complete = false;
    bool positivity = false;
    std::string failure_stage;
    ALR20R63ZCWork work;
    std::vector<ALR20R63ZCState> states;
    std::vector<std::string> solve_roots;
    std::vector<std::string> scalar_roots;
    std::string operator_root;
    std::string factor_root;
    std::string permutation_root;
    std::string rhs_root;
    std::string inverse_scale;
    std::string root;
};

std::string al_r20_r63zc_transaction_root(
    const ALR20R63ZCTransaction& transaction) {
    std::ostringstream material;
    material << transaction.exact << ':' << transaction.complete << ':'
        << transaction.positivity << ':' << transaction.failure_stage << ':'
        << transaction.operator_root << ':' << transaction.rhs_root << ':'
        << transaction.inverse_scale << ':' << transaction.factor_root << ':'
        << transaction.permutation_root << ':'
        << al_r20_r63zc_work_root(transaction.work) << '|';
    for (const std::string& root : transaction.solve_roots)
        material << "solve:" << root << ';';
    for (const std::string& root : transaction.scalar_roots)
        material << "scalar:" << root << ';';
    for (const ALR20R63ZCState& state : transaction.states)
        material << "state:" << state.root << ';';
    return sha256_hex(material.str());
}

ALR20R63ZCTransaction al_r20_r63zc_transaction(
    ALR20R63ZCOperator kind, const std::vector<Binary128>& tangent,
    std::size_t tangent_columns, Binary128 sigma,
    const ALR20R63ZArtifact& common, const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation, Binary128 inverse_scale,
    const std::vector<Binary128>& rhs) {
    ALR20R63ZCTransaction result;
    const std::size_t dimension = rhs.size();
    const std::vector<Binary128> upper_q = al_r20_r63g_q_from_double(upper);
    result.operator_root = al_r20_r63zc_operator_root(kind,
        al_r20_q_scalar_root(tangent), tangent_columns,
        al_binary128_hex(sigma), common.root);
    result.factor_root = al_r20_double_root(upper);
    result.permutation_root = al_r20_r63g_index_root(permutation);
    result.rhs_root = al_r20_q_scalar_root(rhs);
    result.inverse_scale = al_binary128_hex(inverse_scale);
    bool exact = dimension > 0U
        && upper.size() == dimension * dimension
        && al_r20_r63g_permutation_exact(permutation, dimension)
        && al_r20_r63zc_vector_valid(rhs, dimension)
        && inverse_scale > static_cast<Binary128>(0.0)
        && al_r20_r38_normal_or_zero(inverse_scale)
        && ((kind == ALR20R63ZCOperator::tangent
                && tangent.size() == dimension * tangent_columns
                && tangent_columns > 0U && sigma > static_cast<Binary128>(0.0))
            || (kind == ALR20R63ZCOperator::common
                && al_r20_r63z_artifact_valid(common)
                && common.identity.dimension == dimension));
    result.positivity = exact;
    std::vector<Binary128> x0;
    std::vector<Binary128> r0;
    std::vector<Binary128> z0;
    std::vector<Binary128> p0;
    std::vector<Binary128> x1;
    std::vector<Binary128> r1;
    std::vector<Binary128> z1;
    std::vector<Binary128> p1;
    std::vector<Binary128> x2;
    std::vector<Binary128> r2;
    ALR20R63IQSolve start;
    ALR20R63IQSolve z0_solve;
    ALR20R63IQSolve z1_solve;
    ALR20R63ZCProduct hx0;
    ALR20R63ZCProduct hp0;
    ALR20R63ZCProduct hp1;
    ALR20R63MScalar rho0;
    ALR20R63MScalar denominator0;
    ALR20R63MScalar rho1;
    ALR20R63MScalar denominator1;
    Binary128 alpha0 = static_cast<Binary128>(0.0);
    Binary128 beta1 = static_cast<Binary128>(0.0);
    Binary128 alpha1 = static_cast<Binary128>(0.0);
    if (!exact) {
        result.failure_stage = "input";
    } else {
        start = al_r20_r63i_q_solve(
            upper_q, permutation, rhs, inverse_scale);
        result.solve_roots.push_back(start.root);
        ++result.work.factor_solves;
        result.work.factor_terms += start.forward_terms + start.backward_terms;
        result.work.factor_divisions += start.divisions;
        x0 = start.solution;
        exact = start.exact;
        if (!exact) result.failure_stage = "initial_solve";
    }
    if (exact) {
        hx0 = al_r20_r63zc_product(
            kind, tangent, tangent_columns, sigma, common, x0);
        al_r20_r63zc_add_work(result.work, hx0.work);
        exact = hx0.exact;
        if (!exact) result.failure_stage = "initial_product";
    }
    if (exact) {
        r0.resize(dimension);
        for (std::size_t index = 0U; index < dimension && exact; ++index) {
            const ALR20R38Dot2 dot = al_r20_r38_dot2(
                {rhs[index], -hx0.value[index]},
                {static_cast<Binary128>(1.0),
                    static_cast<Binary128>(1.0)});
            r0[index] = dot.value;
            exact = dot.exact && dot.no_underflow
                && al_r20_r38_normal_or_zero(dot.value);
        }
        if (!exact) result.failure_stage = "initial_residual";
    }
    if (exact) {
        z0_solve = al_r20_r63i_q_solve(
            upper_q, permutation, r0, inverse_scale);
        result.solve_roots.push_back(z0_solve.root);
        ++result.work.factor_solves;
        result.work.factor_terms +=
            z0_solve.forward_terms + z0_solve.backward_terms;
        result.work.factor_divisions += z0_solve.divisions;
        z0 = z0_solve.solution;
        p0 = z0;
        rho0 = al_r20_r63m_scalar(r0, z0);
        result.scalar_roots.push_back(rho0.root);
        ++result.work.scalar_dots;
        exact = z0_solve.exact && rho0.exact;
        result.positivity = exact && rho0.positive;
        if (!exact) result.failure_stage = "initial_preconditioner";
        else if (!result.positivity) result.failure_stage = "initial_rho";
    }
    if (exact && result.positivity) {
        hp0 = al_r20_r63zc_product(
            kind, tangent, tangent_columns, sigma, common, p0);
        al_r20_r63zc_add_work(result.work, hp0.work);
        denominator0 = al_r20_r63m_scalar(p0, hp0.value);
        result.scalar_roots.push_back(denominator0.root);
        ++result.work.scalar_dots;
        exact = hp0.exact && denominator0.exact;
        result.positivity = exact && denominator0.positive;
        if (!exact) result.failure_stage = "iteration1_product";
        else if (!result.positivity)
            result.failure_stage = "iteration1_denominator";
    }
    if (exact && result.positivity) {
        alpha0 = rho0.value / denominator0.value;
        ++result.work.scalar_divisions;
        x1.resize(dimension);
        r1.resize(dimension);
        for (std::size_t index = 0U; index < dimension && exact; ++index) {
            const ALR20R38Dot2 x_dot = al_r20_r38_dot2(
                {x0[index], alpha0},
                {static_cast<Binary128>(1.0), p0[index]});
            const ALR20R38Dot2 r_dot = al_r20_r38_dot2(
                {r0[index], -alpha0},
                {static_cast<Binary128>(1.0), hp0.value[index]});
            x1[index] = x_dot.value;
            r1[index] = r_dot.value;
            exact = x_dot.exact && x_dot.no_underflow
                && r_dot.exact && r_dot.no_underflow
                && al_r20_r38_normal_or_zero(x_dot.value)
                && al_r20_r38_normal_or_zero(r_dot.value);
        }
        result.work.solution_updates += dimension;
        result.work.residual_updates += dimension;
        if (!exact || !al_r20_r38_normal_or_zero(alpha0))
            result.failure_stage = "iteration1_update";
    }
    if (exact && result.positivity) {
        z1_solve = al_r20_r63i_q_solve(
            upper_q, permutation, r1, inverse_scale);
        result.solve_roots.push_back(z1_solve.root);
        ++result.work.factor_solves;
        result.work.factor_terms +=
            z1_solve.forward_terms + z1_solve.backward_terms;
        result.work.factor_divisions += z1_solve.divisions;
        z1 = z1_solve.solution;
        rho1 = al_r20_r63m_scalar(r1, z1);
        result.scalar_roots.push_back(rho1.root);
        ++result.work.scalar_dots;
        exact = z1_solve.exact && rho1.exact;
        result.positivity = exact && rho1.positive;
        if (!exact) result.failure_stage = "iteration1_preconditioner";
        else if (!result.positivity) result.failure_stage = "iteration1_rho";
    }
    if (exact && result.positivity) {
        beta1 = rho1.value / rho0.value;
        ++result.work.scalar_divisions;
        p1.resize(dimension);
        for (std::size_t index = 0U; index < dimension && exact; ++index) {
            const ALR20R38Dot2 dot = al_r20_r38_dot2(
                {z1[index], beta1},
                {static_cast<Binary128>(1.0), p0[index]});
            p1[index] = dot.value;
            exact = dot.exact && dot.no_underflow
                && al_r20_r38_normal_or_zero(dot.value);
        }
        result.work.direction_updates += dimension;
        if (!exact || !al_r20_r38_normal_or_zero(beta1))
            result.failure_stage = "direction_update";
    }
    if (exact && result.positivity) {
        hp1 = al_r20_r63zc_product(
            kind, tangent, tangent_columns, sigma, common, p1);
        al_r20_r63zc_add_work(result.work, hp1.work);
        denominator1 = al_r20_r63m_scalar(p1, hp1.value);
        result.scalar_roots.push_back(denominator1.root);
        ++result.work.scalar_dots;
        exact = hp1.exact && denominator1.exact;
        result.positivity = exact && denominator1.positive;
        if (!exact) result.failure_stage = "iteration2_product";
        else if (!result.positivity)
            result.failure_stage = "iteration2_denominator";
    }
    if (exact && result.positivity) {
        alpha1 = rho1.value / denominator1.value;
        ++result.work.scalar_divisions;
        x2.resize(dimension);
        r2.resize(dimension);
        for (std::size_t index = 0U; index < dimension && exact; ++index) {
            const ALR20R38Dot2 x_dot = al_r20_r38_dot2(
                {x1[index], alpha1},
                {static_cast<Binary128>(1.0), p1[index]});
            const ALR20R38Dot2 r_dot = al_r20_r38_dot2(
                {r1[index], -alpha1},
                {static_cast<Binary128>(1.0), hp1.value[index]});
            x2[index] = x_dot.value;
            r2[index] = r_dot.value;
            exact = x_dot.exact && x_dot.no_underflow
                && r_dot.exact && r_dot.no_underflow
                && al_r20_r38_normal_or_zero(x_dot.value)
                && al_r20_r38_normal_or_zero(r_dot.value);
        }
        result.work.solution_updates += dimension;
        result.work.residual_updates += dimension;
        if (!exact || !al_r20_r38_normal_or_zero(alpha1))
            result.failure_stage = "iteration2_update";
    }
    if (exact && result.positivity) {
        const Binary128 zero = static_cast<Binary128>(0.0);
        result.states.push_back(al_r20_r63zc_state(0U, x0, r0, p0, z0,
            rho0.value, zero, zero, zero, hx0.root, z0_solve.root));
        result.states.push_back(al_r20_r63zc_state(1U, x1, r1, p1, z1,
            rho1.value, denominator0.value, alpha0, beta1,
            hp0.root, z1_solve.root));
        result.states.push_back(al_r20_r63zc_state(2U, x2, r2, p1, z1,
            rho1.value, denominator1.value, alpha1, beta1,
            hp1.root, z1_solve.root));
        result.complete = result.states.size() == 3U
            && std::all_of(result.states.begin(), result.states.end(),
                [](const ALR20R63ZCState& state) {
                    return al_r20_r63zc_state_valid(state);
                });
        if (!result.complete) result.failure_stage = "state_seal";
    }
    result.exact = result.complete
        || (!result.positivity && exact && !result.failure_stage.empty()
            && result.states.empty());
    result.root = al_r20_r63zc_transaction_root(result);
    return result;
}

struct ALR20R63ZCLane {
    bool exact = false;
    bool passed = false;
    std::string name;
    ALR20R63ZCOperator kind = ALR20R63ZCOperator::tangent;
    bool projected_inputs = false;
    std::string profile_root;
    ALR20R63ZCTransaction transaction;
    std::vector<ALR20R63YCertificate> certificates;
    std::string root;
};

bool al_r20_r63zc_profile_payload_valid(const ALR20R63YProfile& profile) {
    if (!profile.exact || !profile.contained || !profile.contractive
        || profile.width != 2U
        || profile.dimension != AL_R20_R63ZC_DIMENSION
        || profile.vector_containments != profile.dimension
        || profile.matrix_containments
            != profile.dimension * profile.dimension
        || profile.vector_components.size()
            != profile.dimension * profile.width
        || profile.vector_radius.size() != profile.dimension
        || profile.matrix_components.size()
            != profile.dimension * profile.dimension * profile.width
        || profile.matrix_radius.size()
            != profile.dimension * profile.dimension)
        return false;
    std::size_t nonzero_vector_lows = 0U;
    std::size_t nonzero_matrix_lows = 0U;
    for (std::size_t index = 1U;
         index < profile.vector_components.size(); index += profile.width)
        nonzero_vector_lows += profile.vector_components[index] != 0.0;
    for (std::size_t index = 1U;
         index < profile.matrix_components.size(); index += profile.width)
        nonzero_matrix_lows += profile.matrix_components[index] != 0.0;
    const double maximum_vector_radius = *std::max_element(
        profile.vector_radius.begin(), profile.vector_radius.end());
    const double maximum_matrix_radius = *std::max_element(
        profile.matrix_radius.begin(), profile.matrix_radius.end());
    if (profile.nonzero_vector_lows != nonzero_vector_lows
        || profile.nonzero_matrix_lows != nonzero_matrix_lows
        || profile.maximum_vector_radius != maximum_vector_radius
        || profile.maximum_matrix_radius != maximum_matrix_radius
        || profile.component_root != sha256_hex(
            al_r20_double_root(profile.vector_components) + ":"
            + al_r20_double_root(profile.matrix_components))
        || profile.radius_root != sha256_hex(
            al_r20_double_root(profile.vector_radius) + ":"
            + al_r20_double_root(profile.matrix_radius))
        || profile.root != al_r20_r63zb_profile_root(profile))
        return false;
    for (double value : profile.vector_components)
        if (!al_r20_r63y_normal_or_zero(value)) return false;
    for (double value : profile.matrix_components)
        if (!al_r20_r63y_normal_or_zero(value)) return false;
    for (double value : profile.vector_radius)
        if (value < 0.0 || !al_r20_r63y_normal_or_zero(value)) return false;
    for (double value : profile.matrix_radius)
        if (value < 0.0 || !al_r20_r63y_normal_or_zero(value)) return false;
    return profile.rho_upper >= 0.0 && profile.rho_upper < 1.0
        && al_r20_r63y_normal_or_zero(profile.rho_upper)
        && al_r20_r63y_normal_or_zero(profile.maximum_vector_radius)
        && al_r20_r63y_normal_or_zero(profile.maximum_matrix_radius);
}

std::string al_r20_r63zc_lane_root(const ALR20R63ZCLane& lane) {
    std::ostringstream material;
    material << lane.exact << ':' << lane.passed << ':' << lane.name
        << ':' << al_r20_r63zc_operator_name(lane.kind) << ':'
        << lane.projected_inputs << ':' << lane.profile_root << ':'
        << lane.transaction.root << '|';
    for (const ALR20R63YCertificate& certificate : lane.certificates)
        material << certificate.root << ';';
    return sha256_hex(material.str());
}

bool al_r20_r63zc_lane_valid(const ALR20R63ZCLane& lane) {
    return lane.exact && !lane.profile_root.empty()
        && lane.transaction.root
            == al_r20_r63zc_transaction_root(lane.transaction)
        && lane.root == al_r20_r63zc_lane_root(lane);
}

ALR20R63ZCLane al_r20_r63zc_lane(const std::string& name,
    ALR20R63ZCOperator kind, bool projected_inputs,
    const std::vector<Binary128>& tangent, std::size_t tangent_columns,
    Binary128 sigma, const ALR20R63ZArtifact& common,
    const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation, Binary128 inverse_scale,
    const std::vector<Binary128>& rhs, const ALR20R63YProfile& profile) {
    ALR20R63ZCLane result;
    result.name = name;
    result.kind = kind;
    result.projected_inputs = projected_inputs;
    result.profile_root = profile.root;
    result.transaction = al_r20_r63zc_transaction(kind, tangent,
        tangent_columns, sigma, common, upper, permutation,
        inverse_scale, rhs);
    if (result.transaction.complete) {
        for (const ALR20R63ZCState& state : result.transaction.states) {
            const ALR20R63YSolution solution = al_r20_r63y_solution(
                state.solution, 2U);
            result.certificates.push_back(
                al_r20_r63y_certificate(profile, solution));
            ++result.transaction.work.certificates;
        }
    }
    result.transaction.root =
        al_r20_r63zc_transaction_root(result.transaction);
    result.passed = al_r20_r63zb_ladder(result.certificates);
    result.exact = result.transaction.exact
        && result.transaction.complete && result.certificates.size() == 3U
        && al_r20_r63zc_profile_payload_valid(profile)
        && result.profile_root == profile.root
        && result.transaction.solve_roots.size() == 3U
        && result.transaction.scalar_roots.size() == 4U
        && std::all_of(result.transaction.solve_roots.begin(),
            result.transaction.solve_roots.end(),
            [](const std::string& root) { return !root.empty(); })
        && std::all_of(result.transaction.scalar_roots.begin(),
            result.transaction.scalar_roots.end(),
            [](const std::string& root) { return !root.empty(); })
        && std::all_of(result.transaction.states.begin(),
            result.transaction.states.end(),
            [](const ALR20R63ZCState& state) {
                return al_r20_r63zc_state_valid(state);
            })
        && std::all_of(result.certificates.begin(), result.certificates.end(),
            [](const ALR20R63YCertificate& certificate) {
                return certificate.exact;
            });
    result.root = al_r20_r63zc_lane_root(result);
    return result;
}

bool al_r20_r63zc_cell_assignment_exact(
    const std::array<const ALR20R63ZCLane*, 4U>& lanes,
    const std::string& tangent_operator_root,
    const std::string& common_operator_root,
    const std::string& factor_root,
    const std::string& permutation_root,
    const std::string& original_rhs_root,
    const std::string& projected_rhs_root,
    const std::string& original_scale,
    const std::string& projected_scale,
    const std::string& profile_root) {
    const auto shared = [&](const ALR20R63ZCLane* lane,
                            const std::string& operator_root,
                            const std::string& rhs_root,
                            const std::string& scale) {
        return lane != nullptr && al_r20_r63zc_lane_valid(*lane)
            && lane->transaction.operator_root == operator_root
            && lane->transaction.factor_root == factor_root
            && lane->transaction.permutation_root == permutation_root
            && lane->transaction.rhs_root == rhs_root
            && lane->transaction.inverse_scale == scale
            && lane->profile_root == profile_root;
    };
    return lanes[0U] != nullptr && lanes[1U] != nullptr
        && lanes[2U] != nullptr && lanes[3U] != nullptr
        && shared(lanes[0U], tangent_operator_root,
            original_rhs_root, original_scale)
        && shared(lanes[1U], tangent_operator_root,
            projected_rhs_root, projected_scale)
        && shared(lanes[2U], common_operator_root,
            original_rhs_root, original_scale)
        && shared(lanes[3U], common_operator_root,
            projected_rhs_root, projected_scale)
        && lanes[0U]->name == "tangent_original"
        && lanes[0U]->kind == ALR20R63ZCOperator::tangent
        && !lanes[0U]->projected_inputs
        && lanes[1U]->name == "tangent_projected"
        && lanes[1U]->kind == ALR20R63ZCOperator::tangent
        && lanes[1U]->projected_inputs
        && lanes[2U]->name == "common_original"
        && lanes[2U]->kind == ALR20R63ZCOperator::common
        && !lanes[2U]->projected_inputs
        && lanes[3U]->name == "common_projected"
        && lanes[3U]->kind == ALR20R63ZCOperator::common
        && lanes[3U]->projected_inputs;
}

std::string al_r20_r63zc_solution_set_root(
    const std::vector<std::vector<Binary128>>& solutions);

bool al_r20_r63zc_baseline_correspondence(
    const ALR20R63ZCLane& lane, const ALR20R63XFixture& fixture) {
    bool exact = lane.exact && lane.passed
        && fixture.exported_solutions.size() == 3U
        && al_r20_r63zc_solution_set_root(fixture.exported_solutions)
            == AL_R20_R63ZC_R63X_SOLUTION_SET_SHA256;
    for (std::size_t state = 0U; state < 3U && exact; ++state)
        exact = al_r20_q_scalar_root(
            lane.transaction.states[state].solution)
            == al_r20_q_scalar_root(fixture.exported_solutions[state]);
    return exact;
}

struct ALR20R63ZCIdentity {
    bool exact = false;
    std::string parent_stdout;
    std::string r63x_semantic;
    std::string r63x_profile_root;
    std::string r63x_endpoint_root;
    std::string r63x_solution_set_root;
    std::string factor_fixture_root;
    std::string factor_root;
    std::string permutation_root;
    std::string tangent_root;
    std::string sigma;
    std::string common_semantic;
    std::string common_root;
    std::string verifier_semantic;
    std::string profile_root;
    std::string original_rhs_root;
    std::string projected_rhs_root;
    std::string original_scale;
    std::string projected_scale;
    std::string cell_order;
    std::string root;
};

std::string al_r20_r63zc_identity_root(const ALR20R63ZCIdentity& value) {
    std::ostringstream material;
    material << value.parent_stdout << ':' << value.r63x_semantic << ':'
        << value.r63x_profile_root << ':' << value.r63x_endpoint_root << ':'
        << value.r63x_solution_set_root << ':' << value.factor_fixture_root
        << ':' << value.factor_root << ':' << value.permutation_root << ':'
        << value.tangent_root << ':' << value.sigma << ':'
        << value.common_semantic << ':'
        << value.common_root << ':' << value.verifier_semantic << ':'
        << value.profile_root << ':' << value.original_rhs_root << ':'
        << value.projected_rhs_root << ':' << value.original_scale << ':'
        << value.projected_scale << ':' << value.cell_order;
    return sha256_hex(material.str());
}

std::string al_r20_r63zc_solution_set_root(
    const std::vector<std::vector<Binary128>>& solutions) {
    std::ostringstream material;
    material << solutions.size() << '|';
    for (const std::vector<Binary128>& solution : solutions)
        material << solution.size() << ':'
            << al_r20_q_scalar_root(solution) << ';';
    return sha256_hex(material.str());
}

bool al_r20_r63zc_projected_inputs_exact(
    const ALR20R63ZALastResult& factor,
    const std::vector<Binary128>& original_rhs,
    const std::vector<Binary128>& projected_rhs, Binary128 original_scale,
    Binary128 projected_scale) {
    if (!al_r20_r63zb_fixture_valid(factor.fixture)
        || factor.fixture.sources.empty()
        || al_r20_q_scalar_root(original_rhs)
            != al_r20_q_scalar_root(factor.fixture.sources[0U])
        || original_scale != factor.fixture.inverse_scale
        || projected_rhs.size() != original_rhs.size()) return false;
    std::vector<Binary128> expected_rhs;
    expected_rhs.reserve(original_rhs.size());
    for (Binary128 value : original_rhs) {
        const ALR20R63ZAScalar projected = al_r20_r63zb_project(value);
        if (!al_r20_r63zb_operand(projected)) return false;
        expected_rhs.push_back(static_cast<Binary128>(projected.high)
            + static_cast<Binary128>(projected.low));
    }
    const ALR20R63ZAScalar expected_scale =
        al_r20_r63zb_project(original_scale);
    return al_r20_r63zb_operand(expected_scale)
        && al_r20_q_scalar_root(projected_rhs)
            == al_r20_q_scalar_root(expected_rhs)
        && projected_scale == static_cast<Binary128>(expected_scale.high)
            + static_cast<Binary128>(expected_scale.low);
}

ALR20R63ZCIdentity al_r20_r63zc_identity(const std::string& parent_stdout,
    const ALR20R63XFixture& fixture, const ALR20R63ZALastResult& factor,
    const ALR20R63ZLastResult& common, const ALR20R63YLastResult& verifier,
    const std::vector<Binary128>& original_rhs,
    const std::vector<Binary128>& projected_rhs, Binary128 original_scale,
    Binary128 projected_scale) {
    ALR20R63ZCIdentity result;
    result.parent_stdout = parent_stdout;
    result.r63x_semantic = fixture.semantic;
    result.r63x_profile_root = fixture.profile_root;
    result.r63x_endpoint_root = fixture.exported_root;
    result.r63x_solution_set_root =
        al_r20_r63zc_solution_set_root(fixture.exported_solutions);
    result.factor_fixture_root = factor.fixture.root;
    result.factor_root = al_r20_double_root(factor.fixture.exported_upper);
    result.permutation_root =
        al_r20_r63g_index_root(factor.fixture.permutation);
    result.tangent_root = fixture.tangent_root;
    result.sigma = al_binary128_hex(fixture.sigma);
    result.common_semantic = common.semantic;
    result.common_root = common.artifact.root;
    result.verifier_semantic = verifier.semantic;
    result.profile_root = verifier.selected_profile.root;
    result.original_rhs_root = al_r20_q_scalar_root(original_rhs);
    result.projected_rhs_root = al_r20_q_scalar_root(projected_rhs);
    result.original_scale = al_binary128_hex(original_scale);
    result.projected_scale = al_binary128_hex(projected_scale);
    result.cell_order =
        "tangent/original;tangent/projected;common/original;common/projected";
    result.root = al_r20_r63zc_identity_root(result);
    result.exact = true;
    return result;
}

bool al_r20_r63zc_identity_valid(const ALR20R63ZCIdentity& identity,
    const ALR20R63XFixture& fixture, const ALR20R63ZALastResult& factor,
    const ALR20R63ZLastResult& common, const ALR20R63YLastResult& verifier,
    const std::vector<Binary128>& original_rhs,
    const std::vector<Binary128>& projected_rhs, Binary128 original_scale,
    Binary128 projected_scale) {
    return identity.exact
        && identity.parent_stdout == AL_R20_R63ZC_PARENT_STDOUT_SHA256
        && fixture.exact
        && fixture.semantic == AL_R20_R63ZC_R63X_SEMANTIC_SHA256
        && fixture.profile_root == AL_R20_R63ZC_R63X_PROFILE_SHA256
        && fixture.exported_root == AL_R20_R63ZC_R63X_EXPORTED_SHA256
        && fixture.exported_solutions.size() == 3U
        && al_r20_r63zc_solution_set_root(fixture.exported_solutions)
            == AL_R20_R63ZC_R63X_SOLUTION_SET_SHA256
        && fixture.tangent.size()
            == AL_R20_R63ZC_DIMENSION * AL_R20_R63ZC_TANGENT_COLUMNS
        && fixture.tangent_columns == AL_R20_R63ZC_TANGENT_COLUMNS
        && fixture.tangent_root == AL_R20_R63ZC_TANGENT_SHA256
        && fixture.tangent_root == al_r20_q_scalar_root(fixture.tangent)
        && fixture.sigma > static_cast<Binary128>(0.0)
        && factor.exact
        && factor.semantic == AL_R20_R63ZB_PARENT_SEMANTIC_SHA256
        && al_r20_r63za_identity_valid(factor.identity)
        && factor.identity.root == AL_R20_R63ZB_PARENT_IDENTITY_SHA256
        && al_r20_r63zb_fixture_valid(factor.fixture)
        && fixture.sigma == static_cast<Binary128>(1.0)
            / factor.fixture.inverse_scale
        && common.exact
        && common.semantic == AL_R20_R63ZB_OPERATOR_SEMANTIC_SHA256
        && al_r20_r63z_artifact_valid(common.artifact)
        && common.artifact.root == AL_R20_R63ZB_OPERATOR_ARTIFACT_SHA256
        && verifier.exact && verifier.selected_width == 2U
        && verifier.semantic == AL_R20_R63ZB_VERIFIER_SEMANTIC_SHA256
        && verifier.twofold_profile_root
            == AL_R20_R63ZB_VERIFIER_PROFILE_SHA256
        && al_r20_r63zb_profile_valid(verifier.selected_profile)
        && al_r20_r63zc_projected_inputs_exact(factor, original_rhs,
            projected_rhs, original_scale, projected_scale)
        && identity.r63x_semantic == fixture.semantic
        && identity.r63x_profile_root == fixture.profile_root
        && identity.r63x_endpoint_root == fixture.exported_root
        && identity.r63x_solution_set_root
            == al_r20_r63zc_solution_set_root(fixture.exported_solutions)
        && identity.r63x_solution_set_root
            == AL_R20_R63ZC_R63X_SOLUTION_SET_SHA256
        && identity.factor_fixture_root == AL_R20_R63ZB_FIXTURE_SHA256
        && identity.factor_fixture_root == factor.fixture.root
        && identity.factor_root == AL_R20_R63ZC_FACTOR_SHA256
        && identity.factor_root
            == al_r20_double_root(factor.fixture.exported_upper)
        && identity.permutation_root == AL_R20_R63ZC_PERMUTATION_SHA256
        && identity.permutation_root
            == al_r20_r63g_index_root(factor.fixture.permutation)
        && identity.tangent_root == fixture.tangent_root
        && identity.sigma == al_binary128_hex(fixture.sigma)
        && identity.common_semantic == AL_R20_R63ZB_OPERATOR_SEMANTIC_SHA256
        && identity.common_semantic == common.semantic
        && identity.common_root == AL_R20_R63ZB_OPERATOR_ARTIFACT_SHA256
        && identity.common_root == common.artifact.root
        && identity.verifier_semantic
            == AL_R20_R63ZB_VERIFIER_SEMANTIC_SHA256
        && identity.verifier_semantic == verifier.semantic
        && identity.profile_root == AL_R20_R63ZB_VERIFIER_PROFILE_SHA256
        && identity.profile_root == verifier.selected_profile.root
        && identity.original_rhs_root == al_r20_q_scalar_root(original_rhs)
        && identity.projected_rhs_root == al_r20_q_scalar_root(projected_rhs)
        && identity.original_scale == al_binary128_hex(original_scale)
        && identity.projected_scale == al_binary128_hex(projected_scale)
        && identity.cell_order
            == "tangent/original;tangent/projected;common/original;common/projected"
        && identity.root == al_r20_r63zc_identity_root(identity);
}

struct ALR20R63ZCIdentityControls {
    bool exact = false;
    bool stale = false;
    bool relabel = false;
    bool tangent = false;
    bool sigma = false;
    bool factor = false;
    bool permutation = false;
    bool common = false;
    bool verifier = false;
    bool original_rhs = false;
    bool projected_rhs = false;
    bool original_scale = false;
    bool projected_scale = false;
    bool endpoint = false;
    bool endpoint_solutions = false;
    std::string root;
};

ALR20R63ZCIdentityControls al_r20_r63zc_identity_controls(
    const ALR20R63ZCIdentity& identity, const ALR20R63XFixture& fixture,
    const ALR20R63ZALastResult& factor, const ALR20R63ZLastResult& common,
    const ALR20R63YLastResult& verifier,
    const std::vector<Binary128>& original_rhs,
    const std::vector<Binary128>& projected_rhs, Binary128 original_scale,
    Binary128 projected_scale) {
    ALR20R63ZCIdentityControls result;
    const auto valid = [&](const ALR20R63ZCIdentity& candidate,
                           const ALR20R63XFixture& candidate_fixture,
                           const ALR20R63ZALastResult& candidate_factor,
                           const ALR20R63ZLastResult& candidate_common,
                           const ALR20R63YLastResult& candidate_verifier,
                           const std::vector<Binary128>& candidate_original,
                           const std::vector<Binary128>& candidate_projected,
                           Binary128 candidate_original_scale,
                           Binary128 candidate_projected_scale) {
        return al_r20_r63zc_identity_valid(candidate, candidate_fixture,
            candidate_factor, candidate_common, candidate_verifier,
            candidate_original, candidate_projected,
            candidate_original_scale, candidate_projected_scale);
    };
    const bool baseline = valid(identity, fixture, factor, common, verifier,
        original_rhs, projected_rhs, original_scale, projected_scale);

    ALR20R63ZCIdentity stale = identity;
    stale.cell_order = "stale-order";
    result.stale = baseline && !valid(stale, fixture, factor, common,
        verifier, original_rhs, projected_rhs,
        original_scale, projected_scale);
    ALR20R63ZCIdentity relabel = identity;
    relabel.cell_order =
        "tangent/original;common/original;tangent/projected;common/projected";
    relabel.root = al_r20_r63zc_identity_root(relabel);
    result.relabel = !valid(relabel, fixture, factor, common, verifier,
        original_rhs, projected_rhs, original_scale, projected_scale);

    ALR20R63XFixture tangent_mutation = fixture;
    if (!tangent_mutation.tangent.empty())
        tangent_mutation.tangent[0U] = nextafterq(
            tangent_mutation.tangent[0U], HUGE_VALQ);
    tangent_mutation.tangent_root =
        al_r20_q_scalar_root(tangent_mutation.tangent);
    const ALR20R63ZCIdentity tangent_identity = al_r20_r63zc_identity(
        identity.parent_stdout, tangent_mutation, factor, common, verifier,
        original_rhs, projected_rhs, original_scale, projected_scale);
    result.tangent = !tangent_mutation.tangent.empty()
        && !valid(tangent_identity, tangent_mutation, factor, common,
            verifier, original_rhs, projected_rhs,
            original_scale, projected_scale);

    ALR20R63XFixture sigma_mutation = fixture;
    sigma_mutation.sigma = nextafterq(sigma_mutation.sigma, HUGE_VALQ);
    const ALR20R63ZCIdentity sigma_identity = al_r20_r63zc_identity(
        identity.parent_stdout, sigma_mutation, factor, common, verifier,
        original_rhs, projected_rhs, original_scale, projected_scale);
    result.sigma = !valid(sigma_identity, sigma_mutation, factor, common,
        verifier, original_rhs, projected_rhs,
        original_scale, projected_scale);

    ALR20R63ZALastResult factor_mutation = factor;
    if (!factor_mutation.fixture.exported_upper.empty())
        factor_mutation.fixture.exported_upper[0U] = std::nextafter(
            factor_mutation.fixture.exported_upper[0U],
            std::numeric_limits<double>::infinity());
    factor_mutation.fixture.root =
        al_r20_r63s_consumption_fixture_root(factor_mutation.fixture);
    const ALR20R63ZCIdentity factor_identity = al_r20_r63zc_identity(
        identity.parent_stdout, fixture, factor_mutation, common, verifier,
        original_rhs, projected_rhs, original_scale, projected_scale);
    result.factor = !factor_mutation.fixture.exported_upper.empty()
        && !valid(factor_identity, fixture, factor_mutation, common,
            verifier, original_rhs, projected_rhs,
            original_scale, projected_scale);

    ALR20R63ZALastResult permutation_mutation = factor;
    if (permutation_mutation.fixture.permutation.size() >= 2U)
        std::swap(permutation_mutation.fixture.permutation[0U],
            permutation_mutation.fixture.permutation[1U]);
    permutation_mutation.fixture.root =
        al_r20_r63s_consumption_fixture_root(permutation_mutation.fixture);
    const ALR20R63ZCIdentity permutation_identity = al_r20_r63zc_identity(
        identity.parent_stdout, fixture, permutation_mutation, common,
        verifier, original_rhs, projected_rhs,
        original_scale, projected_scale);
    result.permutation = permutation_mutation.fixture.permutation.size() >= 2U
        && !valid(permutation_identity, fixture, permutation_mutation,
            common, verifier, original_rhs, projected_rhs,
            original_scale, projected_scale);

    ALR20R63ZLastResult common_mutation = common;
    if (!common_mutation.artifact.components.empty())
        common_mutation.artifact.components[0U] = std::nextafter(
            common_mutation.artifact.components[0U],
            std::numeric_limits<double>::infinity());
    common_mutation.artifact.component_root =
        al_r20_double_root(common_mutation.artifact.components);
    common_mutation.artifact.root =
        al_r20_r63z_artifact_root(common_mutation.artifact);
    const ALR20R63ZCIdentity common_identity = al_r20_r63zc_identity(
        identity.parent_stdout, fixture, factor, common_mutation, verifier,
        original_rhs, projected_rhs, original_scale, projected_scale);
    result.common = !common_mutation.artifact.components.empty()
        && !valid(common_identity, fixture, factor, common_mutation,
            verifier, original_rhs, projected_rhs,
            original_scale, projected_scale);

    ALR20R63YLastResult verifier_mutation = verifier;
    if (!verifier_mutation.selected_profile.vector_components.empty())
        verifier_mutation.selected_profile.vector_components[0U] =
            std::nextafter(
                verifier_mutation.selected_profile.vector_components[0U],
                std::numeric_limits<double>::infinity());
    verifier_mutation.selected_profile.component_root = sha256_hex(
        al_r20_double_root(
            verifier_mutation.selected_profile.vector_components) + ":"
        + al_r20_double_root(
            verifier_mutation.selected_profile.matrix_components));
    verifier_mutation.selected_profile.root =
        al_r20_r63zb_profile_root(verifier_mutation.selected_profile);
    verifier_mutation.twofold_profile_root =
        verifier_mutation.selected_profile.root;
    const ALR20R63ZCIdentity verifier_identity = al_r20_r63zc_identity(
        identity.parent_stdout, fixture, factor, common, verifier_mutation,
        original_rhs, projected_rhs, original_scale, projected_scale);
    result.verifier =
        !verifier_mutation.selected_profile.vector_components.empty()
        && !valid(verifier_identity, fixture, factor, common,
            verifier_mutation, original_rhs, projected_rhs,
            original_scale, projected_scale);

    std::vector<Binary128> original_mutation = original_rhs;
    if (!original_mutation.empty()) original_mutation[0U] = nextafterq(
        original_mutation[0U], HUGE_VALQ);
    const ALR20R63ZCIdentity original_rhs_identity = al_r20_r63zc_identity(
        identity.parent_stdout, fixture, factor, common, verifier,
        original_mutation, projected_rhs, original_scale, projected_scale);
    result.original_rhs = !original_mutation.empty()
        && !valid(original_rhs_identity, fixture, factor, common, verifier,
            original_mutation, projected_rhs,
            original_scale, projected_scale);

    std::vector<Binary128> projected_mutation = projected_rhs;
    if (!projected_mutation.empty()) projected_mutation[0U] = nextafterq(
        projected_mutation[0U], HUGE_VALQ);
    const ALR20R63ZCIdentity projected_rhs_identity = al_r20_r63zc_identity(
        identity.parent_stdout, fixture, factor, common, verifier,
        original_rhs, projected_mutation, original_scale, projected_scale);
    result.projected_rhs = !projected_mutation.empty()
        && !valid(projected_rhs_identity, fixture, factor, common, verifier,
            original_rhs, projected_mutation,
            original_scale, projected_scale);

    const Binary128 original_scale_mutation =
        nextafterq(original_scale, HUGE_VALQ);
    const ALR20R63ZCIdentity original_scale_identity =
        al_r20_r63zc_identity(identity.parent_stdout, fixture, factor,
            common, verifier, original_rhs, projected_rhs,
            original_scale_mutation, projected_scale);
    result.original_scale = !valid(original_scale_identity, fixture, factor,
        common, verifier, original_rhs, projected_rhs,
        original_scale_mutation, projected_scale);
    const Binary128 projected_scale_mutation =
        nextafterq(projected_scale, HUGE_VALQ);
    const ALR20R63ZCIdentity projected_scale_identity =
        al_r20_r63zc_identity(identity.parent_stdout, fixture, factor,
            common, verifier, original_rhs, projected_rhs,
            original_scale, projected_scale_mutation);
    result.projected_scale = !valid(projected_scale_identity, fixture, factor,
        common, verifier, original_rhs, projected_rhs,
        original_scale, projected_scale_mutation);

    ALR20R63XFixture endpoint_mutation = fixture;
    endpoint_mutation.exported_root = "mutated-endpoint";
    const ALR20R63ZCIdentity endpoint_identity = al_r20_r63zc_identity(
        identity.parent_stdout, endpoint_mutation, factor, common, verifier,
        original_rhs, projected_rhs, original_scale, projected_scale);
    result.endpoint = !valid(endpoint_identity, endpoint_mutation, factor,
        common, verifier, original_rhs, projected_rhs,
        original_scale, projected_scale);

    ALR20R63XFixture endpoint_solution_mutation = fixture;
    if (!endpoint_solution_mutation.exported_solutions.empty()
        && !endpoint_solution_mutation.exported_solutions[0U].empty())
        endpoint_solution_mutation.exported_solutions[0U][0U] = nextafterq(
            endpoint_solution_mutation.exported_solutions[0U][0U], HUGE_VALQ);
    const ALR20R63ZCIdentity endpoint_solution_identity =
        al_r20_r63zc_identity(identity.parent_stdout,
            endpoint_solution_mutation, factor, common, verifier,
            original_rhs, projected_rhs, original_scale, projected_scale);
    result.endpoint_solutions =
        !endpoint_solution_mutation.exported_solutions.empty()
        && !endpoint_solution_mutation.exported_solutions[0U].empty()
        && !valid(endpoint_solution_identity, endpoint_solution_mutation,
            factor, common, verifier, original_rhs, projected_rhs,
            original_scale, projected_scale);

    result.exact = baseline && result.stale && result.relabel
        && result.tangent && result.sigma && result.factor
        && result.permutation
        && result.common && result.verifier && result.original_rhs
        && result.projected_rhs && result.original_scale
        && result.projected_scale && result.endpoint
        && result.endpoint_solutions;
    std::ostringstream material;
    material << result.exact << ':' << result.stale << ':' << result.relabel
        << ':' << result.tangent << ':' << result.sigma << ':'
        << result.factor << ':'
        << result.permutation << ':' << result.common << ':'
        << result.verifier << ':' << result.original_rhs << ':'
        << result.projected_rhs << ':' << result.original_scale << ':'
        << result.projected_scale << ':' << result.endpoint << ':'
        << result.endpoint_solutions << ':'
        << identity.root;
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63zc_classify(bool apparatus, bool identity, bool work,
    bool baseline, bool combined, bool input_only, bool operator_only) {
    if (!apparatus) return "OPERATOR_INPUT_FACTORIAL_APPARATUS_REJECTED";
    if (!identity) return "OPERATOR_INPUT_FACTORIAL_IDENTITY_REJECTED";
    if (!work) return "OPERATOR_INPUT_FACTORIAL_WORK_REJECTED";
    if (!baseline) return "TANGENT_ORIGINAL_ENDPOINT_REJECTED";
    if (!combined) return "COMMON_PROJECTED_ENDPOINT_REJECTED";
    if (input_only && !operator_only)
        return "COMMON_OPERATOR_PERTURBATION_SUFFICIENT";
    if (!input_only && operator_only)
        return "INPUT_PROJECTION_PERTURBATION_SUFFICIENT";
    if (!input_only && !operator_only)
        return "BOTH_PERTURBATIONS_INDEPENDENTLY_SUFFICIENT";
    return "JOINT_OPERATOR_INPUT_INTERACTION_REQUIRED";
}

struct ALR20R63ZCControls {
    bool exact = false;
    bool factorial = false;
    bool invalid_input = false;
    bool arithmetic_overflow = false;
    bool nonpositive_rho = false;
    bool nonpositive_denominator = false;
    bool later_nonpositive_rho = false;
    bool later_nonpositive_denominator = false;
    bool root_mutation = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63ZArtifact al_r20_r63zc_control_common() {
    ALR20R63ZArtifact result;
    result.identity = al_r20_r63z_identity(
        2U, "r63zc-control", "r63zc-common", "r63zc-tangent");
    result.entries = 4U;
    result.containments = 4U;
    result.components = {5.0, 0.0, 2.0, 0.0,
                         2.0, 0.0, 2.0, 0.0};
    result.radius.assign(4U, 0.0);
    result.maximum_radius = 0.0;
    result.component_root = al_r20_double_root(result.components);
    result.radius_root = al_r20_double_root(result.radius);
    result.contained = true;
    result.exact = true;
    result.root = al_r20_r63z_artifact_root(result);
    result.exact = al_r20_r63z_artifact_valid(result);
    return result;
}

ALR20R63ZArtifact al_r20_r63zc_diagonal_control(
    double first, double second, const std::string& role) {
    ALR20R63ZArtifact result;
    result.identity = al_r20_r63z_identity(
        2U, "r63zc-" + role + "-parent",
        "r63zc-" + role + "-common",
        "r63zc-" + role + "-tangent");
    result.entries = 4U;
    result.containments = 4U;
    result.components = {first, 0.0, 0.0, 0.0,
                         0.0, 0.0, second, 0.0};
    result.radius.assign(4U, 0.0);
    result.maximum_radius = 0.0;
    result.component_root = al_r20_double_root(result.components);
    result.radius_root = al_r20_double_root(result.radius);
    result.contained = true;
    result.exact = true;
    result.root = al_r20_r63z_artifact_root(result);
    result.exact = al_r20_r63z_artifact_valid(result);
    return result;
}

ALR20R63ZCControls al_r20_r63zc_controls() {
    ALR20R63ZCControls result;
    const std::vector<Binary128> tangent{
        static_cast<Binary128>(2.0), static_cast<Binary128>(0.0),
        static_cast<Binary128>(1.0), static_cast<Binary128>(1.0)};
    const ALR20R63ZArtifact common = al_r20_r63zc_control_common();
    const std::vector<double> upper{1.0, 0.0, 0.0, 1.0};
    const std::vector<std::size_t> permutation{0U, 1U};
    const Binary128 one = static_cast<Binary128>(1.0);
    const std::vector<Binary128> original{
        static_cast<Binary128>(1.0), static_cast<Binary128>(2.0)};
    std::vector<Binary128> projected = original;
    projected[1U] = nextafterq(projected[1U], HUGE_VALQ);
    const ALR20R63ZCTransaction tangent_original = al_r20_r63zc_transaction(
        ALR20R63ZCOperator::tangent, tangent, 2U, one, common,
        upper, permutation, one, original);
    const ALR20R63ZCTransaction common_original = al_r20_r63zc_transaction(
        ALR20R63ZCOperator::common, tangent, 2U, one, common,
        upper, permutation, one, original);
    const ALR20R63ZCTransaction tangent_projected = al_r20_r63zc_transaction(
        ALR20R63ZCOperator::tangent, tangent, 2U, one, common,
        upper, permutation, one, projected);
    result.factorial = tangent_original.complete && common_original.complete
        && tangent_projected.complete
        && al_r20_q_scalar_root(tangent_original.states[0U].solution)
            == al_r20_q_scalar_root(common_original.states[0U].solution)
        && al_r20_q_scalar_root(tangent_original.states[0U].solution)
            != al_r20_q_scalar_root(tangent_projected.states[0U].solution)
        && al_r20_q_scalar_root(tangent_original.states[1U].solution)
            != al_r20_q_scalar_root(common_original.states[1U].solution);
    const ALR20R63ZCTransaction invalid_input = al_r20_r63zc_transaction(
        ALR20R63ZCOperator::common, tangent, 2U, one,
        al_r20_r63zb_diagonal_control_artifact(1.0, "r63zc-invalid"),
        upper, permutation, one,
        {static_cast<Binary128>(1.0)});
    const ALR20R63ZCTransaction zero_rho = al_r20_r63zc_transaction(
        ALR20R63ZCOperator::common, tangent, 2U, one,
        al_r20_r63zb_diagonal_control_artifact(1.0, "r63zc-zero-rho"),
        upper, permutation, one, original);
    const ALR20R63ZCTransaction negative_denominator =
        al_r20_r63zc_transaction(ALR20R63ZCOperator::common, tangent, 2U,
            one, al_r20_r63zb_diagonal_control_artifact(
                -1.0, "r63zc-negative-denominator"),
            upper, permutation, one, original);
    const Binary128 huge = ldexpq(static_cast<Binary128>(1.0), 16000);
    const ALR20R63ZCTransaction arithmetic_overflow =
        al_r20_r63zc_transaction(ALR20R63ZCOperator::common, tangent, 2U,
            one, al_r20_r63zc_diagonal_control(
                std::numeric_limits<double>::max(),
                std::numeric_limits<double>::max(), "arithmetic-overflow"),
            upper, permutation, one, {huge, huge});
    const ALR20R63ZCTransaction later_zero_rho = al_r20_r63zc_transaction(
        ALR20R63ZCOperator::common, tangent, 2U, one,
        al_r20_r63zc_diagonal_control(2.0, 2.0, "later-zero-rho"),
        upper, permutation, one, original);
    const ALR20R63ZCTransaction later_negative_denominator =
        al_r20_r63zc_transaction(ALR20R63ZCOperator::common, tangent, 2U,
            one, al_r20_r63zc_diagonal_control(
                2.0, -1.0, "later-negative-denominator"),
            upper, permutation, one,
            {static_cast<Binary128>(2.0), static_cast<Binary128>(1.0)});
    ALR20R63ZCWork overflow_work;
    overflow_work.operator_products = 1U;
    overflow_work.common_dots = 2U;
    overflow_work.common_terms = 4U;
    overflow_work.factor_solves = 1U;
    overflow_work.factor_terms = 2U;
    overflow_work.factor_divisions = 4U;
    ALR20R63ZCWork zero_rho_work = overflow_work;
    zero_rho_work.factor_solves = 2U;
    zero_rho_work.factor_terms = 4U;
    zero_rho_work.factor_divisions = 8U;
    zero_rho_work.scalar_dots = 1U;
    ALR20R63ZCWork first_denominator_work = zero_rho_work;
    first_denominator_work.operator_products = 2U;
    first_denominator_work.common_dots = 4U;
    first_denominator_work.common_terms = 8U;
    first_denominator_work.scalar_dots = 2U;
    ALR20R63ZCWork later_rho_work = first_denominator_work;
    later_rho_work.factor_solves = 3U;
    later_rho_work.factor_terms = 6U;
    later_rho_work.factor_divisions = 12U;
    later_rho_work.scalar_dots = 3U;
    later_rho_work.scalar_divisions = 1U;
    later_rho_work.solution_updates = 2U;
    later_rho_work.residual_updates = 2U;
    ALR20R63ZCWork later_denominator_work = later_rho_work;
    later_denominator_work.operator_products = 3U;
    later_denominator_work.common_dots = 6U;
    later_denominator_work.common_terms = 12U;
    later_denominator_work.scalar_dots = 4U;
    later_denominator_work.scalar_divisions = 2U;
    later_denominator_work.direction_updates = 2U;
    const ALR20R63ZCWork no_work;
    const auto work_matches = [](const ALR20R63ZCWork& actual,
                                 const ALR20R63ZCWork& expected) {
        return al_r20_r63zc_work_root(actual)
            == al_r20_r63zc_work_root(expected);
    };
    result.invalid_input = !invalid_input.exact && !invalid_input.complete
        && invalid_input.failure_stage == "input"
        && work_matches(invalid_input.work, no_work)
        && invalid_input.states.empty();
    result.arithmetic_overflow = !arithmetic_overflow.exact
        && !arithmetic_overflow.complete
        && arithmetic_overflow.failure_stage == "initial_product"
        && work_matches(arithmetic_overflow.work, overflow_work)
        && arithmetic_overflow.states.empty();
    result.nonpositive_rho = zero_rho.exact && !zero_rho.complete
        && !zero_rho.positivity && zero_rho.failure_stage == "initial_rho"
        && work_matches(zero_rho.work, zero_rho_work)
        && zero_rho.states.empty();
    result.nonpositive_denominator = negative_denominator.exact
        && !negative_denominator.complete
        && !negative_denominator.positivity
        && negative_denominator.failure_stage == "iteration1_denominator"
        && work_matches(negative_denominator.work, first_denominator_work)
        && negative_denominator.states.empty();
    result.later_nonpositive_rho = later_zero_rho.exact
        && !later_zero_rho.complete && !later_zero_rho.positivity
        && later_zero_rho.failure_stage == "iteration1_rho"
        && work_matches(later_zero_rho.work, later_rho_work)
        && later_zero_rho.states.empty();
    result.later_nonpositive_denominator =
        later_negative_denominator.exact
        && !later_negative_denominator.complete
        && !later_negative_denominator.positivity
        && later_negative_denominator.failure_stage
            == "iteration2_denominator"
        && work_matches(
            later_negative_denominator.work, later_denominator_work)
        && later_negative_denominator.states.empty();
    ALR20R63ZCIdentity identity;
    identity.exact = true;
    identity.cell_order = "control-order";
    identity.root = al_r20_r63zc_identity_root(identity);
    ALR20R63ZCIdentity stale = identity;
    stale.cell_order = "stale-order";
    result.root_mutation = identity.root == al_r20_r63zc_identity_root(identity)
        && stale.root != al_r20_r63zc_identity_root(stale);
    ALR20R63ZCTransaction mutated_transaction = tangent_original;
    bool state_mutation_input = mutated_transaction.states.size() == 3U
        && !mutated_transaction.states[0U].residual.empty();
    if (state_mutation_input) {
        ALR20R63ZCState& mutated_state = mutated_transaction.states[0U];
        mutated_state.residual[0U] = nextafterq(
            mutated_state.residual[0U], HUGE_VALQ);
        mutated_state = al_r20_r63zc_state(mutated_state.index,
            mutated_state.solution, mutated_state.residual,
            mutated_state.direction, mutated_state.preconditioned,
            mutated_state.rho, mutated_state.denominator,
            mutated_state.alpha, mutated_state.beta,
            mutated_state.product_root, mutated_state.solve_root);
    }
    const std::string resealed_transaction_root =
        al_r20_r63zc_transaction_root(mutated_transaction);
    result.mutation = state_mutation_input
        && al_r20_r63zc_state_valid(tangent_original.states[0U])
        && al_r20_r63zc_state_valid(mutated_transaction.states[0U])
        && tangent_original.root != resealed_transaction_root
        && mutated_transaction.root != resealed_transaction_root;
    result.classifier =
        al_r20_r63zc_classify(
            false, false, false, false, false, false, false)
            == "OPERATOR_INPUT_FACTORIAL_APPARATUS_REJECTED"
        && al_r20_r63zc_classify(
            true, false, false, false, false, false, false)
            == "OPERATOR_INPUT_FACTORIAL_IDENTITY_REJECTED"
        && al_r20_r63zc_classify(
            true, true, false, false, false, false, false)
            == "OPERATOR_INPUT_FACTORIAL_WORK_REJECTED"
        && al_r20_r63zc_classify(
            true, true, true, false, false, false, false)
            == "TANGENT_ORIGINAL_ENDPOINT_REJECTED"
        && al_r20_r63zc_classify(
            true, true, true, true, false, false, false)
            == "COMMON_PROJECTED_ENDPOINT_REJECTED"
        && al_r20_r63zc_classify(
            true, true, true, true, true, true, false)
            == "COMMON_OPERATOR_PERTURBATION_SUFFICIENT"
        && al_r20_r63zc_classify(
            true, true, true, true, true, false, true)
            == "INPUT_PROJECTION_PERTURBATION_SUFFICIENT"
        && al_r20_r63zc_classify(
            true, true, true, true, true, false, false)
            == "BOTH_PERTURBATIONS_INDEPENDENTLY_SUFFICIENT"
        && al_r20_r63zc_classify(
            true, true, true, true, true, true, true)
            == "JOINT_OPERATOR_INPUT_INTERACTION_REQUIRED";
    result.exact = result.factorial && result.invalid_input
        && result.arithmetic_overflow && result.nonpositive_rho
        && result.nonpositive_denominator && result.later_nonpositive_rho
        && result.later_nonpositive_denominator
        && result.root_mutation && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + std::to_string(result.factorial) + ':'
        + std::to_string(result.invalid_input) + ':'
        + std::to_string(result.arithmetic_overflow) + ':'
        + std::to_string(result.nonpositive_rho) + ':'
        + std::to_string(result.nonpositive_denominator) + ':'
        + std::to_string(result.later_nonpositive_rho) + ':'
        + std::to_string(result.later_nonpositive_denominator) + ':'
        + std::to_string(result.root_mutation) + ':'
        + std::to_string(result.mutation) + ':'
        + std::to_string(result.classifier) + ':'
        + tangent_original.root + ':' + common_original.root + ':'
        + tangent_projected.root + ':' + invalid_input.root + ':'
        + arithmetic_overflow.root + ':' + zero_rho.root + ':'
        + negative_denominator.root + ':' + later_zero_rho.root + ':'
        + later_negative_denominator.root);
    return result;
}

bool al_r20_r63zc_work_exact(const ALR20R63ZCWork& work) {
    return work.certificates == 12U && work.operator_products == 12U
        && work.tangent_inner_dots == 1890U
        && work.tangent_outer_dots == 612U
        && work.tangent_inner_terms == 192780U
        && work.tangent_outer_terms == 192780U
        && work.tangent_scale_products == 612U
        && work.common_dots == 612U && work.common_terms == 62424U
        && work.factor_solves == 12U && work.factor_terms == 123624U
        && work.factor_divisions == 2448U && work.scalar_dots == 16U
        && work.scalar_divisions == 12U
        && work.solution_updates == 816U
        && work.residual_updates == 816U
        && work.direction_updates == 408U && work.adaptive_stops == 0U;
}

std::string al_r20_r63zc_work_root(const ALR20R63ZCWork& work) {
    std::ostringstream material;
    material << work.certificates << ':' << work.operator_products << ':'
        << work.tangent_inner_dots << ':' << work.tangent_outer_dots << ':'
        << work.tangent_inner_terms << ':' << work.tangent_outer_terms << ':'
        << work.tangent_scale_products << ':' << work.common_dots << ':'
        << work.common_terms << ':' << work.factor_solves << ':'
        << work.factor_terms << ':' << work.factor_divisions << ':'
        << work.scalar_dots << ':' << work.scalar_divisions << ':'
        << work.solution_updates << ':' << work.residual_updates << ':'
        << work.direction_updates << ':' << work.adaptive_stops;
    return sha256_hex(material.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_operator_input_factorial_controls() {
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
    const ALR20R63ZCControls controls = al_r20_r63zc_controls();
    const SplitBoundaryReport parent =
        run_al_generalization_v5_twofold_recurrence_controls();
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const std::string r63y_stdout = sha256_hex(al_r20_r63y_last_json + "\n");
    const std::string r63z_stdout = sha256_hex(al_r20_r63z_last_json + "\n");
    const std::string r63za_stdout = sha256_hex(al_r20_r63za_last_json + "\n");
    const bool ancestor_regression_exact =
        r63y_stdout == AL_R20_R63ZC_R63Y_STDOUT_SHA256
        && r63z_stdout == AL_R20_R63ZC_R63Z_STDOUT_SHA256
        && r63za_stdout == AL_R20_R63ZC_R63ZA_STDOUT_SHA256
        && parent_stdout == AL_R20_R63ZC_PARENT_STDOUT_SHA256;
    const std::string ancestor_regression_root = sha256_hex(
        r63y_stdout + ':' + r63z_stdout + ':' + r63za_stdout + ':'
        + parent_stdout);
    const ALR20R63XFixture fixture = al_r20_r63x_last_fixture;
    const ALR20R63ZALastResult factor = al_r20_r63za_last_result;
    const ALR20R63ZLastResult common = al_r20_r63z_last_result;
    const ALR20R63YLastResult verifier = al_r20_r63y_last_result;
    const bool parent_exact = !parent.passed
        && parent_stdout == AL_R20_R63ZC_PARENT_STDOUT_SHA256
        && parent.json.find(AL_R20_R63ZC_PARENT_RESULT_SHA256)
            != std::string::npos
        && parent.json.find("TWOFOLD_RECURRENCE_LADDER_REJECTED")
            != std::string::npos
        && parent.json.find("\"comparator_exclusion_exact\":true")
            != std::string::npos
        && fixture.exact
        && fixture.semantic == AL_R20_R63ZC_R63X_SEMANTIC_SHA256
        && factor.exact && common.exact && verifier.exact;
    std::vector<ALR20R63ZAScalar> projected_rhs_k2;
    std::vector<Binary128> original_rhs;
    std::vector<Binary128> projected_rhs;
    ALR20R63ZAScalar projected_scale_k2;
    Binary128 original_scale = static_cast<Binary128>(0.0);
    Binary128 projected_scale = static_cast<Binary128>(0.0);
    if (parent_exact && !factor.fixture.sources.empty()) {
        original_rhs = factor.fixture.sources[0U];
        projected_rhs.reserve(original_rhs.size());
        projected_rhs_k2.reserve(original_rhs.size());
        for (Binary128 value : original_rhs) {
            const ALR20R63ZAScalar projected = al_r20_r63zb_project(value);
            projected_rhs_k2.push_back(projected);
            projected_rhs.push_back(static_cast<Binary128>(projected.high)
                + static_cast<Binary128>(projected.low));
        }
        original_scale = factor.fixture.inverse_scale;
        projected_scale_k2 = al_r20_r63zb_project(original_scale);
        projected_scale = static_cast<Binary128>(projected_scale_k2.high)
            + static_cast<Binary128>(projected_scale_k2.low);
    }
    const ALR20R63ZCIdentity identity = al_r20_r63zc_identity(
        parent_stdout, fixture, factor, common, verifier,
        original_rhs, projected_rhs, original_scale, projected_scale);
    const bool tangent_fixture_exact = fixture.exact
        && fixture.semantic == AL_R20_R63ZC_R63X_SEMANTIC_SHA256
        && fixture.tangent.size()
            == AL_R20_R63ZC_DIMENSION * AL_R20_R63ZC_TANGENT_COLUMNS
        && fixture.tangent_columns == AL_R20_R63ZC_TANGENT_COLUMNS
        && fixture.tangent_root == AL_R20_R63ZC_TANGENT_SHA256
        && fixture.tangent_root == al_r20_q_scalar_root(fixture.tangent)
        && fixture.sigma > static_cast<Binary128>(0.0);
    const bool factor_fixture_exact = factor.exact
        && al_r20_r63zb_fixture_valid(factor.fixture)
        && identity.factor_root == AL_R20_R63ZC_FACTOR_SHA256
        && identity.permutation_root == AL_R20_R63ZC_PERMUTATION_SHA256;
    const bool common_fixture_exact = common.exact
        && al_r20_r63z_artifact_valid(common.artifact)
        && identity.common_root == common.artifact.root;
    const bool verifier_fixture_exact = verifier.exact
        && al_r20_r63zb_profile_valid(verifier.selected_profile)
        && identity.profile_root == verifier.selected_profile.root;
    const bool input_fixture_exact = original_rhs.size()
            == AL_R20_R63ZC_DIMENSION
        && projected_rhs.size() == AL_R20_R63ZC_DIMENSION
        && original_scale > static_cast<Binary128>(0.0)
        && projected_scale > static_cast<Binary128>(0.0)
        && identity.original_rhs_root == al_r20_q_scalar_root(original_rhs)
        && identity.projected_rhs_root == al_r20_q_scalar_root(projected_rhs)
        && identity.original_scale == al_binary128_hex(original_scale)
        && identity.projected_scale == al_binary128_hex(projected_scale);
    const bool identity_payload_exact = parent_exact
        && al_r20_r63zc_identity_valid(identity, fixture, factor,
            common, verifier, original_rhs, projected_rhs,
            original_scale, projected_scale);
    const ALR20R63ZCIdentityControls identity_controls =
        al_r20_r63zc_identity_controls(identity, fixture, factor, common,
            verifier, original_rhs, projected_rhs,
            original_scale, projected_scale);
    const bool identity_preflight_exact = identity_payload_exact
        && identity_controls.exact;
    const ALR20R63ZCLane tangent_original = identity_preflight_exact
        ? al_r20_r63zc_lane("tangent_original", ALR20R63ZCOperator::tangent,
            false, fixture.tangent, fixture.tangent_columns, fixture.sigma,
            common.artifact, factor.fixture.exported_upper,
            factor.fixture.permutation, original_scale, original_rhs,
            verifier.selected_profile)
        : ALR20R63ZCLane{};
    const ALR20R63ZCLane tangent_projected = identity_preflight_exact
        ? al_r20_r63zc_lane("tangent_projected",
            ALR20R63ZCOperator::tangent, true, fixture.tangent,
            fixture.tangent_columns, fixture.sigma, common.artifact,
            factor.fixture.exported_upper, factor.fixture.permutation,
            projected_scale, projected_rhs, verifier.selected_profile)
        : ALR20R63ZCLane{};
    const ALR20R63ZCLane common_original = identity_preflight_exact
        ? al_r20_r63zc_lane("common_original", ALR20R63ZCOperator::common,
            false, fixture.tangent, fixture.tangent_columns, fixture.sigma,
            common.artifact, factor.fixture.exported_upper,
            factor.fixture.permutation, original_scale, original_rhs,
            verifier.selected_profile)
        : ALR20R63ZCLane{};
    const ALR20R63ZCLane common_projected = identity_preflight_exact
        ? al_r20_r63zc_lane("common_projected", ALR20R63ZCOperator::common,
            true, fixture.tangent, fixture.tangent_columns, fixture.sigma,
            common.artifact, factor.fixture.exported_upper,
            factor.fixture.permutation, projected_scale, projected_rhs,
            verifier.selected_profile)
        : ALR20R63ZCLane{};
    const std::array<const ALR20R63ZCLane*, 4U> lanes{
        &tangent_original, &tangent_projected,
        &common_original, &common_projected};
    const std::string tangent_operator_root = al_r20_r63zc_operator_root(
        ALR20R63ZCOperator::tangent, identity.tangent_root,
        fixture.tangent_columns, identity.sigma, identity.common_root);
    const std::string common_operator_root = al_r20_r63zc_operator_root(
        ALR20R63ZCOperator::common, identity.tangent_root,
        fixture.tangent_columns, identity.sigma, identity.common_root);
    const bool cell_assignment_exact =
        al_r20_r63zc_cell_assignment_exact(lanes,
            tangent_operator_root, common_operator_root,
            identity.factor_root, identity.permutation_root,
            identity.original_rhs_root, identity.projected_rhs_root,
            identity.original_scale, identity.projected_scale,
            identity.profile_root);
    const std::array<const ALR20R63ZCLane*, 4U> relabeled_lanes{
        &tangent_original, &common_original,
        &tangent_projected, &common_projected};
    const bool relabel_assignment_control =
        !al_r20_r63zc_cell_assignment_exact(relabeled_lanes,
            tangent_operator_root, common_operator_root,
            identity.factor_root, identity.permutation_root,
            identity.original_rhs_root, identity.projected_rhs_root,
            identity.original_scale, identity.projected_scale,
            identity.profile_root);
    ALR20R63ZCLane misassigned_projected = tangent_original;
    misassigned_projected.name = "tangent_projected";
    misassigned_projected.projected_inputs = true;
    misassigned_projected.root =
        al_r20_r63zc_lane_root(misassigned_projected);
    const std::array<const ALR20R63ZCLane*, 4U> misassigned_lanes{
        &tangent_original, &misassigned_projected,
        &common_original, &common_projected};
    const bool payload_assignment_control =
        !al_r20_r63zc_cell_assignment_exact(misassigned_lanes,
            tangent_operator_root, common_operator_root,
            identity.factor_root, identity.permutation_root,
            identity.original_rhs_root, identity.projected_rhs_root,
            identity.original_scale, identity.projected_scale,
            identity.profile_root);
    const bool cell_assignment_control = relabel_assignment_control
        && payload_assignment_control;
    const bool identity_exact = identity_preflight_exact
        && cell_assignment_exact && cell_assignment_control;
    ALR20R63YProfile mutated_certificate_profile =
        verifier.selected_profile;
    if (!mutated_certificate_profile.vector_radius.empty())
        mutated_certificate_profile.vector_radius[0U] = std::nextafter(
            mutated_certificate_profile.vector_radius[0U],
            std::numeric_limits<double>::infinity());
    if (!mutated_certificate_profile.vector_radius.empty())
        mutated_certificate_profile.maximum_vector_radius =
            *std::max_element(mutated_certificate_profile.vector_radius.begin(),
                mutated_certificate_profile.vector_radius.end());
    mutated_certificate_profile.component_root = sha256_hex(
        al_r20_double_root(
            mutated_certificate_profile.vector_components) + ":"
        + al_r20_double_root(
            mutated_certificate_profile.matrix_components));
    mutated_certificate_profile.radius_root = sha256_hex(
        al_r20_double_root(mutated_certificate_profile.vector_radius) + ":"
        + al_r20_double_root(mutated_certificate_profile.matrix_radius));
    mutated_certificate_profile.root =
        al_r20_r63zb_profile_root(mutated_certificate_profile);
    const ALR20R63ZCLane certificate_replay = identity_exact
        ? al_r20_r63zc_lane("tangent_original",
            ALR20R63ZCOperator::tangent, false, fixture.tangent,
            fixture.tangent_columns, fixture.sigma, common.artifact,
            factor.fixture.exported_upper, factor.fixture.permutation,
            original_scale, original_rhs, mutated_certificate_profile)
        : ALR20R63ZCLane{};
    bool certificate_isolation = tangent_original.exact
        && certificate_replay.exact
        && !mutated_certificate_profile.vector_radius.empty()
        && certificate_replay.transaction.root
            == tangent_original.transaction.root
        && certificate_replay.root != tangent_original.root;
    bool certificate_changed = false;
    for (std::size_t state = 0U;
         state < tangent_original.transaction.states.size()
             && certificate_isolation; ++state) {
        certificate_isolation =
            certificate_replay.transaction.states[state].root
                == tangent_original.transaction.states[state].root
            && certificate_replay.certificates[state].solution_root
                == tangent_original.certificates[state].solution_root;
        certificate_changed = certificate_changed
            || certificate_replay.certificates[state].root
                != tangent_original.certificates[state].root;
    }
    certificate_isolation = certificate_isolation && certificate_changed;

    const bool baseline_correspondence =
        al_r20_r63zc_baseline_correspondence(tangent_original, fixture);
    ALR20R63XFixture exact_oracle_mutation = fixture;
    bool exact_oracle_mutation_input =
        !exact_oracle_mutation.exported_solutions.empty()
        && !exact_oracle_mutation.exported_solutions[0U].empty();
    if (exact_oracle_mutation_input)
        exact_oracle_mutation.exported_solutions[0U][0U] = nextafterq(
            exact_oracle_mutation.exported_solutions[0U][0U], HUGE_VALQ);
    const ALR20R63ZCLane exact_oracle_replay = identity_exact
        ? al_r20_r63zc_lane("tangent_original",
            ALR20R63ZCOperator::tangent, false, fixture.tangent,
            fixture.tangent_columns, fixture.sigma, common.artifact,
            factor.fixture.exported_upper, factor.fixture.permutation,
            original_scale, original_rhs, verifier.selected_profile)
        : ALR20R63ZCLane{};
    bool exact_oracle_isolation = exact_oracle_mutation_input
        && exact_oracle_replay.exact
        && exact_oracle_replay.transaction.root
            == tangent_original.transaction.root
        && exact_oracle_replay.root == tangent_original.root
        && baseline_correspondence
        && !al_r20_r63zc_baseline_correspondence(
            exact_oracle_replay, exact_oracle_mutation);
    for (std::size_t state = 0U;
         state < tangent_original.transaction.states.size()
             && exact_oracle_isolation; ++state)
        exact_oracle_isolation =
            exact_oracle_replay.transaction.states[state].root
                == tangent_original.transaction.states[state].root
            && exact_oracle_replay.certificates[state].root
                == tangent_original.certificates[state].root;
    ALR20R63ZCWork work;
    bool lanes_exact = true;
    for (const ALR20R63ZCLane* lane : lanes) {
        lanes_exact = lanes_exact && lane->exact;
        al_r20_r63zc_add_work(work, lane->transaction.work);
    }
    const ALR20R63ZBTransaction finite_transaction = identity_exact
        ? al_r20_r63zb_transaction(common.artifact,
            factor.fixture.exported_upper, factor.fixture.permutation,
            projected_scale_k2, projected_rhs_k2)
        : ALR20R63ZBTransaction{};
    const ALR20R63ZBComparator endpoint_comparator =
        finite_transaction.complete
        ? al_r20_r63zb_binary128_comparator(common.artifact,
            projected_rhs_k2, projected_scale_k2,
            factor.fixture.exported_upper, factor.fixture.permutation,
            verifier.selected_profile, finite_transaction.states,
            fixture.exported_solutions)
        : ALR20R63ZBComparator{};
    bool combined_correspondence = common_projected.exact
        && !common_projected.passed && finite_transaction.complete
        && finite_transaction.root == AL_R20_R63ZC_FINITE_TRANSACTION_SHA256
        && endpoint_comparator.exact && !endpoint_comparator.ladder
        && endpoint_comparator.root
            == AL_R20_R63ZC_COMBINED_COMPARATOR_SHA256
        && endpoint_comparator.states.size() == 3U
        && endpoint_comparator.certificates.size() == 3U;
    for (std::size_t state = 0U;
         state < 3U && combined_correspondence; ++state) {
        combined_correspondence = al_r20_q_scalar_root(
            common_projected.transaction.states[state].solution)
                == al_r20_q_scalar_root(endpoint_comparator.states[state])
            && common_projected.certificates[state].root
                == endpoint_comparator.certificates[state].root
            && common_projected.certificates[state].positive == 12U
            && common_projected.certificates[state].negative == 24U
            && common_projected.certificates[state].unresolved == 66U;
    }
    std::array<std::array<Binary128, 3U>, 4U> maximum_baseline_difference{};
    for (std::size_t lane = 0U; lane < lanes.size(); ++lane)
        for (std::size_t state = 0U;
             state < 3U && lanes[lane]->exact; ++state)
            for (std::size_t index = 0U;
                 index < AL_R20_R63ZC_DIMENSION; ++index)
                maximum_baseline_difference[lane][state] = fmaxq(
                    maximum_baseline_difference[lane][state],
                    fabsq(lanes[lane]->transaction.states[state].solution[index]
                        - tangent_original.transaction.states[state]
                            .solution[index]));
    const bool work_exact = lanes_exact && al_r20_r63zc_work_exact(work);
    const std::string work_root = al_r20_r63zc_work_root(work);
    const bool apparatus_exact = profile_exact && controls.exact
        && ancestor_regression_exact
        && certificate_isolation && exact_oracle_isolation;
    const std::string control_bundle_root = sha256_hex(controls.root + ':'
        + identity_controls.root + ':'
        + ancestor_regression_root + ':'
        + std::to_string(cell_assignment_exact) + ':'
        + std::to_string(relabel_assignment_control) + ':'
        + std::to_string(payload_assignment_control) + ':'
        + std::to_string(cell_assignment_control) + ':'
        + std::to_string(certificate_isolation) + ':'
        + certificate_replay.transaction.root + ':'
        + certificate_replay.root + ':'
        + std::to_string(exact_oracle_isolation) + ':'
        + exact_oracle_replay.transaction.root + ':'
        + exact_oracle_replay.root);
    const std::string route = al_r20_r63zc_classify(apparatus_exact,
        identity_exact, work_exact, baseline_correspondence,
        combined_correspondence, tangent_projected.passed,
        common_original.passed);
    const bool exact = apparatus_exact && identity_exact && work_exact
        && baseline_correspondence && combined_correspondence;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r63zc-operator-input-factorial.v3|"
        << AL_R20_R63ZC_PARENT_RESULT_SHA256 << ':' << route << ':'
        << control_bundle_root << ':' << identity.root << ':' << work_root
        << ':'
        << tangent_original.root << ':' << tangent_projected.root << ':'
        << common_original.root << ':' << common_projected.root << ':'
        << endpoint_comparator.root;
    const std::string result_sha = sha256_hex(semantic.str());
    auto lane_json = [](const ALR20R63ZCLane& lane,
                         const std::array<Binary128, 3U>& differences) {
        std::ostringstream stream;
        stream << "{\"exact\":" << (lane.exact ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"operator\":\"" << al_r20_r63zc_operator_name(lane.kind)
            << "\",\"projected_inputs\":"
            << (lane.projected_inputs ? "true" : "false")
            << ",\"operator_root\":\"" << lane.transaction.operator_root
            << "\",\"factor_root\":\"" << lane.transaction.factor_root
            << "\",\"permutation_root\":\""
            << lane.transaction.permutation_root
            << "\",\"rhs_root\":\"" << lane.transaction.rhs_root
            << "\",\"inverse_scale\":\""
            << lane.transaction.inverse_scale
            << "\",\"profile_root\":\"" << lane.profile_root
            << "\",\"transaction_root\":\"" << lane.transaction.root
            << "\",\"solve_roots\":[";
        for (std::size_t index = 0U;
             index < lane.transaction.solve_roots.size(); ++index) {
            if (index != 0U) stream << ',';
            stream << '\"' << lane.transaction.solve_roots[index] << '\"';
        }
        stream << "],\"scalar_roots\":[";
        for (std::size_t index = 0U;
             index < lane.transaction.scalar_roots.size(); ++index) {
            if (index != 0U) stream << ',';
            stream << '\"' << lane.transaction.scalar_roots[index] << '\"';
        }
        stream << "],\"states\":[";
        for (std::size_t state = 0U; state < lane.certificates.size(); ++state) {
            if (state != 0U) stream << ',';
            stream << "{\"state\":" << state << ",\"passed\":"
                << (lane.certificates[state].passed ? "true" : "false")
                << ",\"positive\":" << lane.certificates[state].positive
                << ",\"negative\":" << lane.certificates[state].negative
                << ",\"unresolved\":" << lane.certificates[state].unresolved
                << ",\"error_upper\":"
                << std::setprecision(17)
                << lane.certificates[state].error_upper
                << ",\"solution_root\":\""
                << al_r20_q_scalar_root(
                    lane.transaction.states[state].solution)
                << "\",\"residual_root\":\""
                << al_r20_q_scalar_root(
                    lane.transaction.states[state].residual)
                << "\",\"product_root\":\""
                << lane.transaction.states[state].product_root
                << "\",\"solve_root\":\""
                << lane.transaction.states[state].solve_root
                << "\",\"state_root\":\""
                << lane.transaction.states[state].root
                << "\",\"baseline_difference\":\""
                << al_binary128_text(differences[state]) << "\"}";
        }
        stream << "],\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zc_operator_input_factorial.v3\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
        << ",\"identity_exact\":" << (identity_exact ? "true" : "false")
        << ",\"identity_root\":\"" << identity.root << "\""
        << ",\"identity_controls_exact\":"
        << (identity_controls.exact ? "true" : "false")
        << ",\"identity_controls_root\":\""
        << identity_controls.root << "\""
        << ",\"cell_assignment_exact\":"
        << (cell_assignment_exact ? "true" : "false")
        << ",\"cell_assignment_control\":"
        << (cell_assignment_control ? "true" : "false")
        << ",\"relabel_assignment_control\":"
        << (relabel_assignment_control ? "true" : "false")
        << ",\"payload_assignment_control\":"
        << (payload_assignment_control ? "true" : "false")
        << ",\"r63x_solution_set_root\":\""
        << identity.r63x_solution_set_root << "\""
        << ",\"identity_controls\":{\"stale\":"
        << (identity_controls.stale ? "true" : "false")
        << ",\"relabel\":"
        << (identity_controls.relabel ? "true" : "false")
        << ",\"tangent\":"
        << (identity_controls.tangent ? "true" : "false")
        << ",\"sigma\":"
        << (identity_controls.sigma ? "true" : "false")
        << ",\"factor\":"
        << (identity_controls.factor ? "true" : "false")
        << ",\"permutation\":"
        << (identity_controls.permutation ? "true" : "false")
        << ",\"common\":"
        << (identity_controls.common ? "true" : "false")
        << ",\"verifier\":"
        << (identity_controls.verifier ? "true" : "false")
        << ",\"original_rhs\":"
        << (identity_controls.original_rhs ? "true" : "false")
        << ",\"projected_rhs\":"
        << (identity_controls.projected_rhs ? "true" : "false")
        << ",\"original_scale\":"
        << (identity_controls.original_scale ? "true" : "false")
        << ",\"projected_scale\":"
        << (identity_controls.projected_scale ? "true" : "false")
        << ",\"endpoint\":"
        << (identity_controls.endpoint ? "true" : "false")
        << ",\"endpoint_solutions\":"
        << (identity_controls.endpoint_solutions ? "true" : "false") << "}"
        << ",\"identity_diagnostics\":{\"parent\":"
        << (parent_exact ? "true" : "false")
        << ",\"tangent\":" << (tangent_fixture_exact ? "true" : "false")
        << ",\"factor\":" << (factor_fixture_exact ? "true" : "false")
        << ",\"common\":" << (common_fixture_exact ? "true" : "false")
        << ",\"verifier\":"
        << (verifier_fixture_exact ? "true" : "false")
        << ",\"inputs\":" << (input_fixture_exact ? "true" : "false")
        << ",\"tangent_root\":\"" << fixture.tangent_root << "\""
        << ",\"tangent_payload_root\":\""
        << al_r20_q_scalar_root(fixture.tangent) << "\""
        << ",\"sigma\":\"" << identity.sigma << "\""
        << ",\"sigma_derivation_exact\":"
        << ((factor_fixture_exact
                && fixture.sigma
                    == static_cast<Binary128>(1.0)
                        / factor.fixture.inverse_scale)
                ? "true" : "false")
        << ",\"factor_root\":\"" << identity.factor_root << "\""
        << ",\"permutation_root\":\"" << identity.permutation_root
        << "\"}"
        << ",\"controls_exact\":" << (controls.exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"control_bundle_root\":\"" << control_bundle_root << "\""
        << ",\"controls\":{\"factorial\":"
        << (controls.factorial ? "true" : "false")
        << ",\"invalid_input\":"
        << (controls.invalid_input ? "true" : "false")
        << ",\"arithmetic_overflow\":"
        << (controls.arithmetic_overflow ? "true" : "false")
        << ",\"nonpositive_rho\":"
        << (controls.nonpositive_rho ? "true" : "false")
        << ",\"nonpositive_denominator\":"
        << (controls.nonpositive_denominator ? "true" : "false")
        << ",\"later_nonpositive_rho\":"
        << (controls.later_nonpositive_rho ? "true" : "false")
        << ",\"later_nonpositive_denominator\":"
        << (controls.later_nonpositive_denominator ? "true" : "false")
        << ",\"root_mutation\":"
        << (controls.root_mutation ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"classifier\":" << (controls.classifier ? "true" : "false")
        << ",\"certificate_isolation\":"
        << (certificate_isolation ? "true" : "false")
        << ",\"exact_oracle_isolation\":"
        << (exact_oracle_isolation ? "true" : "false")
        << "},\"ancestor_regression_exact\":"
        << (ancestor_regression_exact ? "true" : "false")
        << ",\"ancestor_regression_root\":\""
        << ancestor_regression_root << "\""
        << ",\"ancestor_stdout_sha256\":{\"r63y\":\""
        << r63y_stdout << "\",\"r63z\":\"" << r63z_stdout
        << "\",\"r63za\":\"" << r63za_stdout
        << "\",\"r63zb\":\"" << parent_stdout << "\"}"
        << ",\"control_replays\":{\"factorial_work_excludes_replays\":true"
        << ",\"certificate_transaction_root\":\""
        << certificate_replay.transaction.root << "\""
        << ",\"certificate_lane_root\":\"" << certificate_replay.root
        << "\",\"exact_oracle_transaction_root\":\""
        << exact_oracle_replay.transaction.root << "\""
        << ",\"exact_oracle_lane_root\":\"" << exact_oracle_replay.root
        << "\"}"
        << ",\"baseline_correspondence\":"
        << (baseline_correspondence ? "true" : "false")
        << ",\"combined_correspondence\":"
        << (combined_correspondence ? "true" : "false")
        << ",\"lanes\":{\"tangent_original\":"
        << lane_json(tangent_original, maximum_baseline_difference[0U])
        << ",\"tangent_projected\":"
        << lane_json(tangent_projected, maximum_baseline_difference[1U])
        << ",\"common_original\":"
        << lane_json(common_original, maximum_baseline_difference[2U])
        << ",\"common_projected\":"
        << lane_json(common_projected, maximum_baseline_difference[3U])
        << "},\"endpoint_comparator_root\":\""
        << endpoint_comparator.root << "\""
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"work_root\":\"" << work_root << "\""
        << ",\"work\":{\"certificates\":" << work.certificates
        << ",\"operator_products\":" << work.operator_products
        << ",\"tangent_inner_dots\":" << work.tangent_inner_dots
        << ",\"tangent_outer_dots\":" << work.tangent_outer_dots
        << ",\"tangent_inner_terms\":" << work.tangent_inner_terms
        << ",\"tangent_outer_terms\":" << work.tangent_outer_terms
        << ",\"tangent_scale_products\":" << work.tangent_scale_products
        << ",\"common_dots\":" << work.common_dots
        << ",\"common_terms\":" << work.common_terms
        << ",\"factor_solves\":" << work.factor_solves
        << ",\"factor_terms\":" << work.factor_terms
        << ",\"factor_divisions\":" << work.factor_divisions
        << ",\"scalar_dots\":" << work.scalar_dots
        << ",\"scalar_divisions\":" << work.scalar_divisions
        << ",\"solution_updates\":" << work.solution_updates
        << ",\"residual_updates\":" << work.residual_updates
        << ",\"direction_updates\":" << work.direction_updates
        << ",\"adaptive_stops\":" << work.adaptive_stops << "}"
        << ",\"observational_binary128_only\":true"
        << ",\"shared_r63y_verifier\":true"
        << ",\"timing_admitted\":false"
        << ",\"runtime_authority\":false"
        << ",\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
