namespace {

constexpr const char* AL_R20_R63H_PARENT_SEMANTIC_SHA256 =
    "d1b09efa20c088b60796c83b33d5870b9db49b26fa8e0f44477ec22c3825e1eb";
constexpr const char* AL_R20_R63H_TRANSPOSE_SHA256 =
    "8b5db34c925973b26d0761ea2b3e050ca9e665c42a8879716fe3b3fa17588061";
constexpr const char* AL_R20_R63H_PERMUTATION_SHA256 =
    "335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f";
constexpr const char* AL_R20_R63H_Q_FACTOR_SHA256 =
    "fde9aef5ee0233c920ab76c855c612cc722422c67f44d18598211ae901082ec4";
constexpr const char* AL_R20_R63H_D_FACTOR_SHA256 =
    "7cfe979b17c53cf81f4e69fe4ecb51223bb87e97d0b149dcc86df31723818d5d";
constexpr const char* AL_R20_R63H_Q_AUDIT_SHA256 =
    "b4a89421e503e954c8fffb219e2178f99ca9fb07f5375269114a6cf5201ca9bf";
constexpr const char* AL_R20_R63H_D_AUDIT_SHA256 =
    "e0f5fd36c405f0e563053fa9b43bfe4e561e0b66b48725ae1c58b32128b80b21";

std::string al_r20_r63h_classify(bool apparatus, bool wide,
    bool exported) {
    if (!apparatus) return "BOUNDED_WIDE_FACTOR_APPARATUS_REJECTED";
    if (!wide) return "STORED_OPERATOR_WIDE_FACTOR_REJECTED";
    if (!exported) return "WIDE_FACTOR_STATE_REQUIRED";
    return "WIDE_BUILD_BINARY64_EXPORT_CANDIDATE";
}

struct ALR20R63HControls {
    bool exact = false;
    bool promotion = false;
    bool factor = false;
    bool signal = false;
    bool permutation = false;
    bool mutation = false;
    bool boundary = false;
    bool classifier = false;
    std::string root;
};

ALR20R63HControls al_r20_r63h_controls() {
    ALR20R63HControls result;
    const std::vector<double> stored{
        1.0, 0.5, 0.0,
        0.0, 1.0, 0.25,
        0.5, 0.0, 1.0,
        0.25, -0.5, 0.5};
    const std::vector<Binary128> promoted =
        al_r20_r63g_q_from_double(stored);
    bool roundtrip = promoted.size() == stored.size();
    for (std::size_t index = 0U; index < stored.size(); ++index)
        roundtrip = roundtrip
            && static_cast<double>(promoted[index]) == stored[index];
    result.promotion = roundtrip;

    const std::vector<std::size_t> permutation{2U, 0U, 1U};
    std::vector<Binary128> permuted(promoted.size());
    for (std::size_t row = 0U; row < 4U; ++row)
        for (std::size_t column = 0U; column < 3U; ++column)
            permuted[row * 3U + column] =
                promoted[row * 3U + permutation[column]];
    const std::vector<Binary128> gram =
        al_r20_r63g_gram(promoted, 4U, 3U);
    const ALR20R63GQFactor factor =
        al_r20_r63g_q_factor(permuted, 4U, 3U);
    const ALR20R63GFactorAudit audit = al_r20_r63g_factor_audit(
        permuted, factor.q, factor.r, gram, permutation, 4U, 3U, true);
    result.factor = factor.exact && audit.exact;
    result.permutation = al_r20_r63g_permutation_exact(permutation, 3U)
        && audit.gram_contained == 9U;

    const Binary128 tiny = ldexpq(static_cast<Binary128>(1.0), -40);
    const std::vector<Binary128> reference{
        tiny, static_cast<Binary128>(0.0)};
    const ALR20R63GSignal preserved = al_r20_r63g_signal(
        {tiny + tiny / static_cast<Binary128>(4.0),
            static_cast<Binary128>(0.0)}, reference);
    const ALR20R63GSignal rejected = al_r20_r63g_signal(
        {static_cast<Binary128>(3.0) * tiny,
            static_cast<Binary128>(0.0)}, reference);
    result.signal = preserved.exact && preserved.survives
        && rejected.exact && !rejected.survives;
    std::vector<Binary128> mutated_factor = factor.r;
    if (!mutated_factor.empty())
        mutated_factor.back() = nextafterq(
            mutated_factor.back(), HUGE_VALQ);
    std::vector<double> mutated_stored = stored;
    mutated_stored[0U] = std::nextafter(
        mutated_stored[0U], std::numeric_limits<double>::infinity());
    result.mutation = al_r20_double_root(stored)
            != al_r20_double_root(mutated_stored)
        && al_r20_q_scalar_root(factor.r)
            != al_r20_q_scalar_root(mutated_factor);
    const Binary128 signal_norm = tiny * tiny;
    const Binary128 equal_error = signal_norm;
    result.boundary = !(equal_error < signal_norm);
    result.classifier =
        al_r20_r63h_classify(false, true, true)
            == "BOUNDED_WIDE_FACTOR_APPARATUS_REJECTED"
        && al_r20_r63h_classify(true, false, true)
            == "STORED_OPERATOR_WIDE_FACTOR_REJECTED"
        && al_r20_r63h_classify(true, true, false)
            == "WIDE_FACTOR_STATE_REQUIRED"
        && al_r20_r63h_classify(true, true, true)
            == "WIDE_BUILD_BINARY64_EXPORT_CANDIDATE";
    result.exact = result.promotion && result.factor && result.signal
        && result.permutation && result.mutation && result.boundary
        && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + al_r20_double_root(stored) + ':'
        + al_r20_q_scalar_root(promoted) + ':'
        + al_r20_r63g_index_root(permutation) + ':' + factor.root + ':'
        + audit.root + ':' + preserved.root + ':' + rejected.root + ':'
        + al_r20_double_root(mutated_stored) + ':'
        + al_r20_q_scalar_root(mutated_factor));
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_bounded_wide_factor_controls() {
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
    const ALR20R63HControls controls = al_r20_r63h_controls();
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
    const ALR20R63FWitnessImage inherited_image = tangent.exact
        && inherited.exact
        ? al_r20_r63f_witness_image(tangent, inherited.alpha,
            inherited.decomposition.tangent)
        : ALR20R63FWitnessImage{};
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows)
        && al_r20_r63g_index_root(permutation)
            == AL_R20_R63H_PERMUTATION_SHA256;

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
    const ALR20R63GFactorAudit parent_q_audit = parent_q_factor.exact
        ? al_r20_r63g_factor_audit(permuted_q, parent_q_factor.q,
            parent_q_factor.r, grams.tangent, permutation,
            tangent.columns, tangent.rows, true)
        : ALR20R63GFactorAudit{};
    const ALR20R63GFactorAudit parent_d_audit = parent_d_factor.exact
        ? al_r20_r63g_factor_audit(
            al_r20_r63g_q_from_double(parent_d_factor.input),
            al_r20_r63g_q_from_double(parent_d_factor.q),
            al_r20_r63g_q_from_double(parent_d_factor.r), grams.tangent,
            permutation, tangent.columns, tangent.rows, false)
        : ALR20R63GFactorAudit{};

    std::vector<Binary128> reference;
    if (inherited_image.exact) {
        reference.reserve(inherited_image.direct.size() * 3U);
        for (const Binary128Vec3& value : inherited_image.direct)
            for (int axis = 0; axis < 3; ++axis)
                reference.push_back(al_r20_q_component(value, axis));
    }
    std::vector<Binary128> stored = transposed;
    for (Binary128& value : stored)
        value = static_cast<Binary128>(static_cast<double>(value));
    std::vector<Binary128> rounded_alpha = inherited.alpha;
    for (Binary128& value : rounded_alpha)
        value = static_cast<Binary128>(static_cast<double>(value));
    std::size_t storage_products = 0U;
    std::size_t alpha_products = 0U;
    std::size_t parent_q_products = 0U;
    std::size_t parent_d_products = 0U;
    const ALR20R63GSignal storage_signal = !reference.empty()
        ? al_r20_r63g_signal(al_r20_r63g_direct_image(stored,
            tangent.columns, tangent.rows, inherited.alpha,
            &storage_products), reference)
        : ALR20R63GSignal{};
    const ALR20R63GSignal alpha_signal = !reference.empty()
        ? al_r20_r63g_signal(al_r20_r63g_direct_image(transposed,
            tangent.columns, tangent.rows, rounded_alpha,
            &alpha_products), reference)
        : ALR20R63GSignal{};
    const ALR20R63GSignal parent_q_signal = parent_q_factor.exact
        && !reference.empty()
        ? al_r20_r63g_signal(al_r20_r63g_factor_image(parent_q_factor.q,
            parent_q_factor.r, tangent.columns, tangent.rows, permutation,
            inherited.alpha, &parent_q_products), reference)
        : ALR20R63GSignal{};
    const ALR20R63GSignal parent_d_signal = parent_d_factor.exact
        && !reference.empty()
        ? al_r20_r63g_signal(al_r20_r63g_factor_image(
            al_r20_r63g_q_from_double(parent_d_factor.q),
            al_r20_r63g_q_from_double(parent_d_factor.r), tangent.columns,
            tangent.rows, permutation, inherited.alpha,
            &parent_d_products), reference)
        : ALR20R63GSignal{};
    const bool parent_apparatus = capture_exact && grams.exact
        && full_profile.exact && inherited.exact && tangent.exact
        && inherited_image.exact && permutation_exact
        && al_r20_q_scalar_root(transposed) == AL_R20_R63H_TRANSPOSE_SHA256
        && parent_q_factor.exact && parent_d_factor.exact
        && parent_q_factor.root == AL_R20_R63H_Q_FACTOR_SHA256
        && parent_d_factor.root == AL_R20_R63H_D_FACTOR_SHA256
        && parent_q_audit.exact && parent_d_audit.exact
        && parent_q_audit.root == AL_R20_R63H_Q_AUDIT_SHA256
        && parent_d_audit.root == AL_R20_R63H_D_AUDIT_SHA256
        && storage_signal.exact && storage_signal.survives
        && alpha_signal.exact && parent_q_signal.exact
        && parent_q_signal.survives && parent_d_signal.exact
        && !parent_d_signal.survives;
    const std::string parent_route = al_r20_r63g_classify(parent_apparatus,
        parent_q_signal.survives, storage_signal.survives,
        parent_d_signal.survives);
    std::ostringstream parent_semantic;
    parent_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63g-rectangular-factor-precision.v1|"
        << AL_R20_R63G_PARENT_SEMANTIC_SHA256 << ':' << parent_route << ':'
        << oblique.root << ':' << factor_capture.matrix_root << ':'
        << al_r20_r63g_controls().root << ':' << tangent.root << ':'
        << al_r20_r63g_index_root(permutation) << ':'
        << parent_q_factor.root << ':' << parent_d_factor.root << ':'
        << parent_q_audit.root << ':' << parent_d_audit.root << ':'
        << storage_signal.root << ':' << alpha_signal.root << ':'
        << parent_q_signal.root << ':' << parent_d_signal.root;
    const bool parent_r63g_exact = parent_apparatus
        && parent_route == "BINARY64_RECTANGULAR_FACTOR_FLOOR"
        && sha256_hex(parent_semantic.str())
            == AL_R20_R63H_PARENT_SEMANTIC_SHA256;

    std::vector<Binary128> stored_permuted =
        al_r20_r63g_q_from_double(parent_d_factor.input);
    bool stored_permutation_exact = stored_permuted.size() == stored.size();
    for (std::size_t scalar = 0U; scalar < tangent.columns
         && stored_permutation_exact; ++scalar)
        for (std::size_t column = 0U; column < tangent.rows; ++column)
            stored_permutation_exact = stored_permutation_exact
                && stored_permuted[scalar * tangent.rows + column]
                    == stored[scalar * tangent.rows + permutation[column]];
    const std::vector<Binary128> stored_gram = stored_permutation_exact
        ? al_r20_r63g_gram(stored, tangent.columns, tangent.rows)
        : std::vector<Binary128>{};
    const ALR20R63GQFactor wide_factor = stored_permutation_exact
        ? al_r20_r63g_q_factor(
            stored_permuted, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    const ALR20R63GFactorAudit wide_audit = wide_factor.exact
        ? al_r20_r63g_factor_audit(stored_permuted, wide_factor.q,
            wide_factor.r, stored_gram, permutation, tangent.columns,
            tangent.rows, true)
        : ALR20R63GFactorAudit{};
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
    const ALR20R63GFactorAudit export_audit = wide_factor.exact
        ? al_r20_r63g_factor_audit(stored_permuted, exported_q,
            exported_r, stored_gram, permutation, tangent.columns,
            tangent.rows, false)
        : ALR20R63GFactorAudit{};

    std::size_t wide_products = 0U;
    std::size_t export_products = 0U;
    const std::vector<Binary128> wide_image = wide_factor.exact
        ? al_r20_r63g_factor_image(wide_factor.q, wide_factor.r,
            tangent.columns, tangent.rows, permutation, inherited.alpha,
            &wide_products)
        : std::vector<Binary128>{};
    const std::vector<Binary128> export_image = wide_factor.exact
        ? al_r20_r63g_factor_image(exported_q, exported_r,
            tangent.columns, tangent.rows, permutation, inherited.alpha,
            &export_products)
        : std::vector<Binary128>{};
    const ALR20R63GSignal wide_original =
        al_r20_r63g_signal(wide_image, reference);
    const ALR20R63GSignal wide_stored =
        al_r20_r63g_signal(wide_image, storage_signal.values);
    const ALR20R63GSignal export_original =
        al_r20_r63g_signal(export_image, reference);
    const ALR20R63GSignal export_stored =
        al_r20_r63g_signal(export_image, storage_signal.values);

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = wide_factor.norm_terms == 26979U
        && wide_factor.reflector_products
            == parent_q_factor.reflector_products
        && wide_factor.q_products == parent_q_factor.q_products
        && wide_audit.orthogonality_products == 3277260U
        && wide_audit.reconstruction_products == 3277260U
        && wide_audit.gram_products == 1061208U
        && export_audit.orthogonality_products == 3277260U
        && export_audit.reconstruction_products == 3277260U
        && export_audit.gram_products == 1061208U
        && wide_products == 42534U && export_products == 42534U;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63g_exact && stored_permutation_exact
        && stored_gram.size() == 10404U && wide_factor.exact
        && wide_audit.exact && export_audit.exact
        && wide_original.exact && wide_stored.exact
        && export_original.exact && export_stored.exact
        && lifecycle_exact && work_exact;
    const bool wide_survives = wide_original.survives
        && wide_stored.survives;
    const bool export_survives = export_original.survives
        && export_stored.survives;
    const std::string route = al_r20_r63h_classify(
        apparatus_exact, wide_survives, export_survives);
    const bool exact = apparatus_exact && wide_survives;

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63h-bounded-wide-factor.v1|"
        << AL_R20_R63H_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << al_r20_q_scalar_root(stored) << ':'
        << al_r20_q_scalar_root(stored_permuted) << ':'
        << al_r20_q_scalar_root(stored_gram) << ':' << wide_factor.root << ':'
        << wide_audit.root << ':' << al_r20_double_root(exported_q_d) << ':'
        << al_r20_double_root(exported_r_d) << ':' << export_audit.root << ':'
        << wide_original.root << ':' << wide_stored.root << ':'
        << export_original.root << ':' << export_stored.root;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63h_bounded_wide_factor.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63g_sha256\":\""
        << AL_R20_R63H_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"promotion\":" << (controls.promotion ? "true" : "false")
        << ",\"factor\":" << (controls.factor ? "true" : "false")
        << ",\"signal\":" << (controls.signal ? "true" : "false")
        << ",\"permutation\":"
        << (controls.permutation ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"boundary\":" << (controls.boundary ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"parent\":{\"exact\":"
        << (parent_r63g_exact ? "true" : "false")
        << ",\"route\":\"" << parent_route << "\"}"
        << ",\"storage\":{\"permutation_exact\":"
        << (stored_permutation_exact ? "true" : "false")
        << ",\"input_root\":\"" << al_r20_q_scalar_root(stored) << "\""
        << ",\"permuted_root\":\""
        << al_r20_q_scalar_root(stored_permuted) << "\""
        << ",\"gram_root\":\"" << al_r20_q_scalar_root(stored_gram)
        << "\"}"
        << ",\"wide_factor\":{\"exact\":"
        << (wide_factor.exact ? "true" : "false")
        << ",\"nonzero_diagonal\":" << wide_factor.nonzero_diagonal
        << ",\"minimum_abs_diagonal\":\""
        << al_binary128_text(wide_factor.minimum_abs_diagonal) << "\""
        << ",\"maximum_abs_diagonal\":\""
        << al_binary128_text(wide_factor.maximum_abs_diagonal) << "\""
        << ",\"root\":\"" << wide_factor.root << "\"}"
        << ",\"wide_audit\":{\"exact\":"
        << (wide_audit.exact ? "true" : "false")
        << ",\"orthogonality_residual\":\""
        << al_binary128_text(wide_audit.orthogonality_residual)
        << "\",\"reconstruction_residual\":\""
        << al_binary128_text(wide_audit.reconstruction_residual)
        << "\",\"gram_residual\":\""
        << al_binary128_text(wide_audit.gram_residual)
        << "\",\"root\":\"" << wide_audit.root << "\"}"
        << ",\"export\":{\"finite\":"
        << (export_audit.finite ? "true" : "false")
        << ",\"q_root\":\"" << al_r20_double_root(exported_q_d) << "\""
        << ",\"r_root\":\"" << al_r20_double_root(exported_r_d) << "\""
        << ",\"orthogonality_residual\":\""
        << al_binary128_text(export_audit.orthogonality_residual)
        << "\",\"reconstruction_residual\":\""
        << al_binary128_text(export_audit.reconstruction_residual)
        << "\",\"gram_residual\":\""
        << al_binary128_text(export_audit.gram_residual)
        << "\",\"root\":\"" << export_audit.root << "\"}"
        << ",\"signals\":{\"original_norm_squared\":\""
        << al_binary128_text(wide_original.reference_norm_squared) << "\""
        << ",\"stored_norm_squared\":\""
        << al_binary128_text(wide_stored.reference_norm_squared) << "\""
        << ",\"wide_original_error_squared\":\""
        << al_binary128_text(wide_original.error_squared) << "\""
        << ",\"wide_stored_error_squared\":\""
        << al_binary128_text(wide_stored.error_squared) << "\""
        << ",\"wide_survives\":"
        << (wide_survives ? "true" : "false")
        << ",\"export_original_error_squared\":\""
        << al_binary128_text(export_original.error_squared) << "\""
        << ",\"export_stored_error_squared\":\""
        << al_binary128_text(export_stored.error_squared) << "\""
        << ",\"export_survives\":"
        << (export_survives ? "true" : "false") << "}"
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"qr_factorizations\":3,\"nnqp_rhs_solves\":0"
        << ",\"triangular_solves\":0,\"iterative_refinements\":0"
        << ",\"lsqr_solves\":0,\"svd_factorizations\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"regularizations\":0"
        << ",\"center_iterations\":0,\"replacement_applies\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
