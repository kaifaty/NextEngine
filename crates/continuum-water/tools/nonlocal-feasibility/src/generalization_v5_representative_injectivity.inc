namespace {

constexpr const char* AL_R20_R63F_PARENT_SEMANTIC_SHA256 =
    "2c9e75c42c140cb828b5421cb8727e2371746e5a0eb65e75698cc4e634b16002";

struct ALR20R63FTangentMatrix {
    bool exact = false;
    std::size_t rows = 0U;
    std::size_t columns = 0U;
    std::size_t row_constructions = 0U;
    std::size_t scalar_writes = 0U;
    std::size_t nonzero_values = 0U;
    std::vector<Binary128> values;
    std::vector<std::vector<Binary128Vec3>> rows_vector;
    std::string row_root;
    std::string value_root;
    std::string root;
};

ALR20R63FTangentMatrix al_r20_r63f_tangent_matrix(
    const ALR20R63EProjectorGrams& grams,
    const ALR20ProjectorState& projector,
    const std::vector<std::size_t>& source_rows) {
    ALR20R63FTangentMatrix result;
    result.rows = grams.rows;
    result.columns = grams.scalars;
    if (!grams.exact || source_rows.size() != grams.rows) return result;
    std::ostringstream row_material;
    result.values.reserve(result.rows * result.columns);
    result.rows_vector.reserve(result.rows);
    bool exact = true;
    for (std::size_t row = 0U; row < grams.rows; ++row) {
        row_material << source_rows[row] << ',';
        const ALR20R63EVectorDecomposition decomposition =
            al_r20_r63e_decompose(projector, grams.source_rows[row]);
        exact = exact && decomposition.exact;
        result.rows_vector.push_back(decomposition.tangent);
        for (const Binary128Vec3& value : decomposition.tangent)
            for (int axis = 0; axis < 3; ++axis) {
                const Binary128 component = al_r20_q_component(value, axis);
                exact = exact && finiteq(component) != 0;
                if (component != static_cast<Binary128>(0.0))
                    ++result.nonzero_values;
                result.values.push_back(component);
                ++result.scalar_writes;
            }
        ++result.row_constructions;
    }
    result.row_root = sha256_hex(row_material.str());
    result.value_root = al_r20_q_scalar_root(result.values);
    result.exact = exact && result.row_constructions == result.rows
        && result.scalar_writes == result.rows * result.columns
        && result.values.size() == result.rows * result.columns;
    std::ostringstream material;
    material << result.exact << ':' << result.rows << ':' << result.columns
        << ':' << result.row_constructions << ':' << result.scalar_writes
        << ':' << result.nonzero_values << ':' << result.row_root << ':'
        << result.value_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63FGramIdentity {
    bool exact = false;
    std::size_t entries = 0U;
    std::size_t contained = 0U;
    std::size_t products = 0U;
    Binary128 maximum_residual = static_cast<Binary128>(0.0);
    Binary128 maximum_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128> image_gram;
    std::string root;
};

ALR20R63FGramIdentity al_r20_r63f_gram_identity(
    const ALR20R63FTangentMatrix& tangent,
    const std::vector<Binary128>& expected) {
    ALR20R63FGramIdentity result;
    if (!tangent.exact
        || expected.size() != tangent.rows * tangent.rows) return result;
    result.image_gram.resize(expected.size());
    bool finite = true;
    for (std::size_t row = 0U; row < tangent.rows; ++row)
        for (std::size_t column = 0U; column < tangent.rows; ++column) {
            Binary128Accumulator dot;
            Binary128 absolute = static_cast<Binary128>(0.0);
            for (std::size_t scalar = 0U;
                 scalar < tangent.columns; ++scalar) {
                const Binary128 product =
                    tangent.values[row * tangent.columns + scalar]
                    * tangent.values[column * tangent.columns + scalar];
                dot.add(product);
                absolute += fabsq(product);
                ++result.products;
            }
            const std::size_t index = row * tangent.rows + column;
            result.image_gram[index] = dot.value();
            const Binary128 residual = fabsq(dot.value() - expected[index]);
            const Binary128 bound = al_r20_q_gamma(
                32U * tangent.columns + 8192U)
                * fmaxq(static_cast<Binary128>(1.0), absolute);
            finite = finite && finiteq(dot.value()) != 0
                && finiteq(residual) != 0 && finiteq(bound) != 0;
            result.maximum_residual = fmaxq(
                result.maximum_residual, residual);
            result.maximum_bound = fmaxq(result.maximum_bound, bound);
            if (residual <= bound) ++result.contained;
            ++result.entries;
        }
    result.exact = finite && result.entries == tangent.rows * tangent.rows
        && result.contained == result.entries
        && result.products
            == tangent.rows * tangent.rows * tangent.columns;
    std::ostringstream material;
    material << result.exact << ':' << result.entries << ':'
        << result.contained << ':' << result.products << ':'
        << al_binary128_hex(result.maximum_residual) << ':'
        << al_binary128_hex(result.maximum_bound) << ':'
        << al_r20_q_scalar_root(result.image_gram) << ':'
        << al_r20_q_scalar_root(expected);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63FWitnessImage {
    bool exact = false;
    bool nonzero = false;
    std::size_t scalars = 0U;
    std::size_t combination_terms = 0U;
    std::size_t contained = 0U;
    Binary128 direct_norm_squared = static_cast<Binary128>(0.0);
    Binary128 expected_norm_squared = static_cast<Binary128>(0.0);
    Binary128 maximum_residual = static_cast<Binary128>(0.0);
    Binary128 maximum_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128Vec3> direct;
    std::string root;
};

ALR20R63FWitnessImage al_r20_r63f_witness_image(
    const ALR20R63FTangentMatrix& tangent,
    const std::vector<Binary128>& alpha,
    const std::vector<Binary128Vec3>& expected) {
    ALR20R63FWitnessImage result;
    result.scalars = tangent.columns;
    if (!tangent.exact || alpha.size() != tangent.rows
        || expected.size() * 3U != tangent.columns) return result;
    std::vector<Binary128Accumulator> sum(tangent.columns);
    std::vector<Binary128> absolute(tangent.columns,
        static_cast<Binary128>(0.0));
    for (std::size_t row = 0U; row < tangent.rows; ++row)
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar) {
            const Binary128 term = alpha[row]
                * tangent.values[row * tangent.columns + scalar];
            sum[scalar].add(term);
            absolute[scalar] += fabsq(term);
            ++result.combination_terms;
        }
    result.direct.resize(tangent.columns / 3U);
    bool finite = true;
    for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar) {
        const Binary128 value = sum[scalar].value();
        const Binary128 expected_value = al_r20_q_component(
            expected[scalar / 3U], static_cast<int>(scalar % 3U));
        const Binary128 residual = fabsq(value - expected_value);
        const Binary128 bound = al_r20_q_gamma(
            64U * tangent.rows + 8192U)
            * fmaxq(static_cast<Binary128>(1.0), absolute[scalar]);
        finite = finite && finiteq(value) != 0
            && finiteq(residual) != 0 && finiteq(bound) != 0;
        al_r20_q_set_component(&result.direct[scalar / 3U],
            static_cast<int>(scalar % 3U), value);
        result.maximum_residual = fmaxq(result.maximum_residual, residual);
        result.maximum_bound = fmaxq(result.maximum_bound, bound);
        if (residual <= bound) ++result.contained;
    }
    result.direct_norm_squared =
        al_r20_projector_vector_dot(result.direct, result.direct);
    result.expected_norm_squared =
        al_r20_projector_vector_dot(expected, expected);
    const Binary128 norm_residual = fabsq(
        result.direct_norm_squared - result.expected_norm_squared);
    const Binary128 norm_bound = al_r20_q_gamma(
        128U * tangent.columns + 16384U)
        * fmaxq(static_cast<Binary128>(1.0),
            result.expected_norm_squared);
    result.nonzero = result.direct_norm_squared
            > static_cast<Binary128>(0.0)
        && result.expected_norm_squared > static_cast<Binary128>(0.0);
    result.exact = finite && result.nonzero
        && result.combination_terms == tangent.rows * tangent.columns
        && result.contained == tangent.columns
        && norm_residual <= norm_bound;
    std::ostringstream material;
    material << result.exact << ':' << result.nonzero << ':'
        << result.scalars << ':' << result.combination_terms << ':'
        << result.contained << ':'
        << al_binary128_hex(result.direct_norm_squared) << ':'
        << al_binary128_hex(result.expected_norm_squared) << ':'
        << al_binary128_hex(result.maximum_residual) << ':'
        << al_binary128_hex(result.maximum_bound) << ':'
        << al_r20_q_vec_root(result.direct) << ':'
        << al_r20_q_vec_root(expected);
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63f_classify(bool apparatus,
    bool modular_full, bool exact_kernel, bool witness_nonzero) {
    if (!apparatus) return "REPRESENTATIVE_INJECTIVITY_APPARATUS_REJECTED";
    if (modular_full && witness_nonzero)
        return "LOCAL_REPRESENTATIVE_NULLSPACE_REFUTED";
    if (exact_kernel)
        return "LOCAL_REPRESENTATIVE_NULLSPACE_CANDIDATE";
    return "RECTANGULAR_MODULAR_RANK_INCONCLUSIVE";
}

struct ALR20R63FControls {
    bool exact = false;
    bool modular = false;
    bool gram = false;
    bool nonzero = false;
    bool shrunk = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63FControls al_r20_r63f_controls() {
    ALR20R63FControls result;
    const std::vector<Binary128> full{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.5), static_cast<Binary128>(0.0),
        static_cast<Binary128>(1.0), static_cast<Binary128>(-0.25)};
    const std::vector<Binary128> duplicate{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5),
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5)};
    const ALR20R63DModularRank full61 = al_r20_r63d_modular_rank(
        full, 2U, 3U, AL_R20_R63D_PRIME_61);
    const ALR20R63DModularRank full31 = al_r20_r63d_modular_rank(
        full, 2U, 3U, AL_R20_R63D_PRIME_31);
    const ALR20R63DModularRank duplicate61 = al_r20_r63d_modular_rank(
        duplicate, 2U, 2U, AL_R20_R63D_PRIME_61);
    result.modular = full61.exact && full31.exact && duplicate61.exact
        && full61.rank == 2U && full31.rank == 2U
        && duplicate61.rank == 1U;

    const std::vector<Binary128> rows{
        static_cast<Binary128>(1.0), static_cast<Binary128>(2.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(3.0)};
    ALR20R63FTangentMatrix tangent;
    tangent.exact = true;
    tangent.rows = 2U;
    tangent.columns = 2U;
    tangent.values = rows;
    const std::vector<Binary128> gram{
        static_cast<Binary128>(5.0), static_cast<Binary128>(6.0),
        static_cast<Binary128>(6.0), static_cast<Binary128>(9.0)};
    const ALR20R63FGramIdentity identity =
        al_r20_r63f_gram_identity(tangent, gram);
    result.gram = identity.exact
        && identity.maximum_residual == static_cast<Binary128>(0.0);

    const Binary128 tiny = ldexpq(static_cast<Binary128>(1.0), -100);
    result.nonzero = tiny > static_cast<Binary128>(0.0)
        && tiny * tiny > static_cast<Binary128>(0.0);
    const Binary128 shrunk_bound = nextafterq(tiny,
        static_cast<Binary128>(0.0));
    result.shrunk = shrunk_bound < tiny && !(tiny <= shrunk_bound);
    std::vector<Binary128> mutated = rows;
    mutated[3U] = nextafterq(mutated[3U], HUGE_VALQ);
    result.mutation = al_r20_q_scalar_root(rows)
        != al_r20_q_scalar_root(mutated);
    result.classifier =
        al_r20_r63f_classify(true, true, false, true)
            == "LOCAL_REPRESENTATIVE_NULLSPACE_REFUTED"
        && al_r20_r63f_classify(true, false, true, false)
            == "LOCAL_REPRESENTATIVE_NULLSPACE_CANDIDATE"
        && al_r20_r63f_classify(true, false, false, false)
            == "RECTANGULAR_MODULAR_RANK_INCONCLUSIVE";
    result.exact = result.modular && result.gram && result.nonzero
        && result.shrunk && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + full61.root + ':' + full31.root + ':' + duplicate61.root + ':'
        + identity.root + ':' + al_r20_q_scalar_root(rows) + ':'
        + al_r20_q_scalar_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_representative_injectivity_controls() {
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
    const ALR20R63FControls controls = al_r20_r63f_controls();
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
    const ALR20R63EWitness inherited = full_profile.exact
        ? al_r20_r63e_witness(
            grams, factor_capture.projector, full_profile.binary128)
        : ALR20R63EWitness{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const ALR20R63DModularRank rank61 = tangent.exact
        ? al_r20_r63d_modular_rank(tangent.values, tangent.rows,
            tangent.columns, AL_R20_R63D_PRIME_61)
        : ALR20R63DModularRank{};
    const ALR20R63DModularRank rank31 = tangent.exact
        ? al_r20_r63d_modular_rank(tangent.values, tangent.rows,
            tangent.columns, AL_R20_R63D_PRIME_31)
        : ALR20R63DModularRank{};
    const bool modular_full = rank61.rank == 102U || rank31.rank == 102U;
    const ALR20R63FGramIdentity gram_identity = tangent.exact
        ? al_r20_r63f_gram_identity(tangent, grams.tangent)
        : ALR20R63FGramIdentity{};
    const ALR20R63FWitnessImage witness_image =
        tangent.exact && inherited.exact
        ? al_r20_r63f_witness_image(tangent, inherited.alpha,
            inherited.decomposition.tangent)
        : ALR20R63FWitnessImage{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = tangent.rows == 102U && tangent.columns == 315U
        && tangent.row_constructions == 102U
        && tangent.scalar_writes == 32130U
        && rank61.mapped_values == 32130U
        && rank31.mapped_values == 32130U
        && gram_identity.entries == 10404U
        && gram_identity.products == 3277260U
        && witness_image.combination_terms == 32130U;
    const bool apparatus_exact = profile_exact && controls_exact
        && capture_exact && grams.exact && full_profile.exact
        && full_profile.binary128.rank == 101U && inherited.exact
        && tangent.exact && rank61.exact && rank31.exact
        && gram_identity.exact && witness_image.exact
        && lifecycle_exact && work_exact;
    const std::string route = al_r20_r63f_classify(apparatus_exact,
        modular_full, false, witness_image.nonzero);
    const bool exact = apparatus_exact;

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63f-representative-injectivity.v1|"
        << AL_R20_R63F_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << factor_capture.matrix_root << ':'
        << controls.root << ':' << tangent.root << ':' << rank61.root << ':'
        << rank31.root << ':' << gram_identity.root << ':'
        << witness_image.root;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63f_representative_injectivity.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63e_sha256\":\""
        << AL_R20_R63F_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"modular\":" << (controls.modular ? "true" : "false")
        << ",\"gram\":" << (controls.gram ? "true" : "false")
        << ",\"nonzero\":" << (controls.nonzero ? "true" : "false")
        << ",\"shrunk\":" << (controls.shrunk ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"capture\":{\"exact\":"
        << (capture_exact ? "true" : "false")
        << ",\"observed_inverse_calls\":" << factor_capture.calls
        << ",\"selected_inverse_calls\":" << factor_capture.selected_calls
        << ",\"matrix_root\":\"" << factor_capture.matrix_root << "\"}"
        << ",\"tangent\":{\"exact\":"
        << (tangent.exact ? "true" : "false")
        << ",\"rows\":" << tangent.rows
        << ",\"columns\":" << tangent.columns
        << ",\"nonzero_values\":" << tangent.nonzero_values
        << ",\"row_root\":\"" << tangent.row_root
        << "\",\"value_root\":\"" << tangent.value_root << "\"}"
        << ",\"modular\":{\"full\":"
        << (modular_full ? "true" : "false")
        << ",\"rank_prime61\":" << rank61.rank
        << ",\"rank_prime31\":" << rank31.rank
        << ",\"mapped_prime61\":" << rank61.mapped_values
        << ",\"mapped_prime31\":" << rank31.mapped_values
        << ",\"root_prime61\":\"" << rank61.root
        << "\",\"root_prime31\":\"" << rank31.root << "\"}"
        << ",\"gram_identity\":{\"exact\":"
        << (gram_identity.exact ? "true" : "false")
        << ",\"contained\":" << gram_identity.contained
        << ",\"entries\":" << gram_identity.entries
        << ",\"products\":" << gram_identity.products
        << ",\"maximum_residual\":\""
        << al_binary128_text(gram_identity.maximum_residual)
        << "\",\"maximum_bound\":\""
        << al_binary128_text(gram_identity.maximum_bound)
        << "\",\"root\":\"" << gram_identity.root << "\"}"
        << ",\"witness_image\":{\"exact\":"
        << (witness_image.exact ? "true" : "false")
        << ",\"nonzero\":"
        << (witness_image.nonzero ? "true" : "false")
        << ",\"contained\":" << witness_image.contained
        << ",\"direct_norm_squared\":\""
        << al_binary128_text(witness_image.direct_norm_squared)
        << "\",\"expected_norm_squared\":\""
        << al_binary128_text(witness_image.expected_norm_squared)
        << "\",\"maximum_residual\":\""
        << al_binary128_text(witness_image.maximum_residual)
        << "\",\"maximum_bound\":\""
        << al_binary128_text(witness_image.maximum_bound)
        << "\",\"root\":\"" << witness_image.root << "\"}"
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"qr_factorizations\":0,\"nnqp_rhs_solves\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"center_iterations\":0"
        << ",\"replacement_applies\":0,\"state_updates\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
