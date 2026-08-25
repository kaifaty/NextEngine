namespace {

constexpr const char* AL_R20_MANIFEST_SHA256 =
    "420031730236445859426960bc5540ae8b8f9522d4b9263fe5e86cf5f13f85b4";
constexpr double AL_R20_SCALE_LOW = 0.00390625; // 2^-8
constexpr double AL_R20_SCALE_HIGH = 256.0;     // 2^8

struct ALR20ProblemData {
    bool exact = false;
    std::string failure;
    std::vector<Vec3> target;
    std::vector<Vec3> lower;
    std::vector<Vec3> upper;
    std::vector<double> diagonal;
    std::vector<double> row_bound;
    std::vector<double> row_scale;
    std::vector<double> projected_raw;
    ALNormalizedWorkspace workspace;
    ALR64Operator op;
    ALR64StructureAudit structure;
    std::size_t source_positive = 0U;
    std::size_t projected_positive = 0U;
    double source_maximum = 0.0;
    double projected_maximum = 0.0;
    double scale_minimum = std::numeric_limits<double>::infinity();
    double scale_maximum = 0.0;
    std::string target_root;
    std::string constraint_root;
    std::string domain_root;
    std::string operator_root;
    std::string problem_root;
};

std::string al_r20_double_root(const std::vector<double>& values) {
    std::ostringstream material;
    material << values.size();
    for (double value : values) material << ':' << binary64_bits(value);
    return sha256_hex(material.str());
}

std::string al_r20_operator_root(const ALR64Operator& op) {
    std::ostringstream material;
    material << "nextengine.nonlocal.r20-sparse-operator.v1|";
    for (std::uint32_t value : op.slot_offsets) material << value << ':';
    material << '|';
    for (const ALR64Slot& value : op.slots)
        material << value.pair << ':' << value.participant << ':'
                 << binary64_bits(value.jacobian.x) << ':'
                 << binary64_bits(value.jacobian.y) << ':'
                 << binary64_bits(value.jacobian.z) << ';';
    material << '|';
    for (std::uint32_t value : op.entry_offsets) material << value << ':';
    material << '|';
    for (const ALR64Entry& value : op.entries)
        material << value.particle << ':'
                 << binary64_bits(value.coefficient.x) << ':'
                 << binary64_bits(value.coefficient.y) << ':'
                 << binary64_bits(value.coefficient.z) << ';';
    material << '|';
    for (std::uint32_t value : op.particle_offsets) material << value << ':';
    material << '|';
    for (const ALR64Incidence& value : op.incidences)
        material << value.row << ':' << value.entry << ';';
    return sha256_hex(material.str());
}

void al_r20_contact_domain(const SmokeFixture& fixture,
    std::vector<Vec3>* lower, std::vector<Vec3>* upper) {
    lower->assign(fixture.position.size(), {
        -AL_R20_TRUST_RADIUS,
        -AL_R20_TRUST_RADIUS,
        -AL_R20_TRUST_RADIUS,
    });
    upper->assign(fixture.position.size(), {
        AL_R20_TRUST_RADIUS,
        AL_R20_TRUST_RADIUS,
        AL_R20_TRUST_RADIUS,
    });
    for (std::size_t particle = 0U;
         particle < fixture.position.size(); ++particle) {
        for (int axis = 0; axis < 3; ++axis) {
            const double source = al_r41_component(
                fixture.position[particle], axis);
            const bool lower_owned = fixture.closed_box_contact
                || fixture.lower_contact[static_cast<std::size_t>(axis)];
            const bool upper_owned = fixture.closed_box_contact;
            if (lower_owned) {
                const double face = fixture.closed_box_contact
                    ? al_r41_component(fixture.contact_low, axis) : RADIUS;
                al_r43_component((*lower)[particle], axis) = std::max(
                    -AL_R20_TRUST_RADIUS, (face - source) / SPACING);
            }
            if (upper_owned) {
                const double face = al_r41_component(
                    fixture.contact_high, axis);
                al_r43_component((*upper)[particle], axis) = std::min(
                    AL_R20_TRUST_RADIUS, (face - source) / SPACING);
            }
        }
    }
}

bool al_r20_scale_control(double residual, double constraint,
    double diagonal, double bound, double scale_factor,
    double* primal_gap, double* dual_gap, double* scale_gap) {
    const double sigma = std::max({
        std::abs(constraint),
        AL_R20_TRUST_RADIUS * std::sqrt(diagonal),
        bound,
    });
    const double lambda = 0.125;
    const double next = std::max(0.0, lambda + residual / diagonal);
    const double mapping = diagonal * (lambda - next);
    const double scaled_constraint = scale_factor * constraint;
    const double scaled_residual = scale_factor * residual;
    const double scaled_diagonal =
        scale_factor * scale_factor * diagonal;
    const double scaled_bound = scale_factor * bound;
    const double scaled_sigma = std::max({
        std::abs(scaled_constraint),
        AL_R20_TRUST_RADIUS * std::sqrt(scaled_diagonal),
        scaled_bound,
    });
    const double scaled_lambda = lambda / scale_factor;
    const double scaled_next = std::max(0.0,
        scaled_lambda + scaled_residual / scaled_diagonal);
    const double scaled_mapping = scaled_diagonal
        * (scaled_lambda - scaled_next);
    const double primal = std::max(0.0, residual) / sigma;
    const double primal_scaled =
        std::max(0.0, scaled_residual) / scaled_sigma;
    const double dual = std::abs(mapping) / sigma;
    const double dual_scaled = std::abs(scaled_mapping) / scaled_sigma;
    *primal_gap = std::abs(primal - primal_scaled);
    *dual_gap = std::abs(dual - dual_scaled);
    *scale_gap = std::abs(scaled_sigma - scale_factor * sigma);
    const double comparison_bound = gamma_factor(128U)
        * (1.0 + std::abs(primal) + std::abs(primal_scaled)
            + std::abs(dual) + std::abs(dual_scaled)
            + std::abs(scaled_sigma) + std::abs(scale_factor * sigma));
    return std::isfinite(*primal_gap) && std::isfinite(*dual_gap)
        && std::isfinite(*scale_gap) && *primal_gap <= comparison_bound
        && *dual_gap <= comparison_bound && *scale_gap <= comparison_bound;
}

ALR20ProblemData al_r20_materialize_problem(
    const ALR20ManifestCase& source,
    ALSparseWorkTrace* sparse_work,
    StaticSupportWorkTrace* static_work,
    FlatAdjacencyWorkTrace* adjacency_work) {
    ALR20ProblemData result;
    const SmokeFixture& fixture = source.fixture;
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    const std::vector<double> dual(fixture.position.size(), 0.0);
    result.workspace = build_al_normalized_sparse_workspace(
        fixture.position, binding, dual,
        al_normalized_theta(SMOKE_FRAME_TIME, KAPPA),
        sparse_work, static_work, adjacency_work);
    if (!index.passed || !binding.passed || !result.workspace.passed) {
        result.failure = "WORKSPACE";
        return result;
    }
    result.op = al_r64_build(result.workspace);
    result.structure = al_r64_structure(result.workspace, result.op);
    if (!result.structure.slot || !result.structure.entry
        || !result.structure.incidence) {
        result.failure = "OPERATOR";
        return result;
    }
    result.target.resize(fixture.position.size());
    for (std::size_t particle = 0U;
         particle < fixture.position.size(); ++particle) {
        const Vec3 predicted_velocity = fixture.velocity[particle]
            + SMOKE_FRAME_TIME * fixture.gravity;
        result.target[particle] =
            (SMOKE_FRAME_TIME / SPACING) * predicted_velocity;
    }
    al_r20_contact_domain(fixture, &result.lower, &result.upper);
    const ALR48Projection projected = al_r48_project_ball_box(
        result.target, result.lower, result.upper, AL_R20_TRUST_RADIUS);
    if (!projected.passed) {
        result.failure = "PROJECTION";
        return result;
    }
    const ALLinearizedJvp image = al_r64_action(result.op, projected.value);
    if (!image.passed
        || image.value.size() != result.workspace.support.constraint.size()) {
        result.failure = "ACTION";
        return result;
    }
    result.projected_raw.resize(image.value.size());
    result.diagonal.resize(image.value.size());
    result.row_bound.resize(image.value.size());
    result.row_scale.resize(image.value.size());
    bool values_exact = true;
    for (std::size_t row = 0U; row < image.value.size(); ++row) {
        const double constraint = result.workspace.support.constraint[row];
        const double raw = constraint + image.value[row];
        const double diagonal = al_r65_sparse_diagonal(result.op, row);
        const std::size_t degree = result.op.slot_offsets[row + 1U]
            - result.op.slot_offsets[row];
        const double scale_term = AL_R20_TRUST_RADIUS * std::sqrt(diagonal);
        const double bound = gamma_factor(16U * degree + 66U)
            * (std::abs(constraint) + scale_term
                + std::numeric_limits<double>::min());
        const double sigma = std::max({
            std::abs(constraint), scale_term, bound});
        result.projected_raw[row] = raw;
        result.diagonal[row] = diagonal;
        result.row_bound[row] = bound;
        result.row_scale[row] = sigma;
        result.source_positive += constraint > 0.0 ? 1U : 0U;
        result.projected_positive += raw > 0.0 ? 1U : 0U;
        result.source_maximum = std::max(result.source_maximum, constraint);
        result.projected_maximum = std::max(result.projected_maximum, raw);
        result.scale_minimum = std::min(result.scale_minimum, sigma);
        result.scale_maximum = std::max(result.scale_maximum, sigma);
        values_exact = values_exact && std::isfinite(raw)
            && std::isfinite(diagonal) && diagonal > 0.0
            && std::isfinite(bound) && bound > 0.0
            && std::isfinite(sigma) && sigma > 0.0;
    }
    std::ostringstream domain;
    domain << al_binary64_vec3_root(result.lower) << ':'
           << al_binary64_vec3_root(result.upper) << ':'
           << binary64_bits(AL_R20_TRUST_RADIUS);
    result.target_root = al_binary64_vec3_root(result.target);
    result.constraint_root =
        al_r20_double_root(result.workspace.support.constraint);
    result.domain_root = sha256_hex(domain.str());
    result.operator_root = al_r20_operator_root(result.op);
    std::ostringstream problem;
    problem << source.source_root << ':' << result.target_root << ':'
            << result.constraint_root << ':' << result.domain_root << ':'
            << result.operator_root << ':'
            << al_r20_double_root(result.diagonal) << ':'
            << al_r20_double_root(result.row_bound) << ':'
            << al_r20_double_root(result.row_scale);
    result.problem_root = sha256_hex(problem.str());
    result.exact = values_exact;
    if (!result.exact) result.failure = "SCALE";
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_operator_preflight_controls() {
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact =
        al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256;
    bool exact = parent_exact;
    bool scaling_exact = true;
    double maximum_primal_gap = 0.0;
    double maximum_dual_gap = 0.0;
    double maximum_scale_gap = 0.0;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    problems.reserve(cases.size());
    for (const ALR20ManifestCase& source : cases) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            source, &sparse_work, &static_work, &adjacency_work);
        exact = exact && problem.exact;
        if (problem.exact) {
            const std::array<std::size_t, 3U> rows{{
                0U,
                problem.diagonal.size() / 2U,
                problem.diagonal.size() - 1U,
            }};
            for (std::size_t row : rows)
                for (double factor : {AL_R20_SCALE_LOW, AL_R20_SCALE_HIGH}) {
                    double primal_gap = 0.0;
                    double dual_gap = 0.0;
                    double scale_gap = 0.0;
                    scaling_exact = scaling_exact && al_r20_scale_control(
                        problem.projected_raw[row],
                        problem.workspace.support.constraint[row],
                        problem.diagonal[row], problem.row_bound[row], factor,
                        &primal_gap, &dual_gap, &scale_gap);
                    maximum_primal_gap = std::max(
                        maximum_primal_gap, primal_gap);
                    maximum_dual_gap = std::max(
                        maximum_dual_gap, dual_gap);
                    maximum_scale_gap = std::max(
                        maximum_scale_gap, scale_gap);
                }
        }
        problems.push_back(std::move(problem));
    }
    bool excitation_exact = true;
    std::size_t excited_transfer = 0U;
    std::size_t excited_holdout = 0U;
    std::size_t quiet_negative = 0U;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        const bool excited = problems[index].projected_positive > 0U;
        if (cases[index].role == "TRANSFER_REGRESSION")
            excited_transfer += excited ? 1U : 0U;
        else if (cases[index].role == "BLIND_HOLDOUT")
            excited_holdout += excited ? 1U : 0U;
        else if (cases[index].role == "KINEMATIC_NEGATIVE_CONTROL")
            quiet_negative += !excited ? 1U : 0U;
    }
    excitation_exact = excited_transfer >= 1U && excited_holdout == 2U
        && quiet_negative == 2U;
    exact = exact && scaling_exact && excitation_exact
        && sparse_work.workspace_builds == cases.size()
        && sparse_work.live_workspaces == cases.size()
        && static_work.static_index_builds == cases.size()
        && static_work.support_canonicalizations == cases.size();
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    exact = exact && lifecycle_exact;

    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20-operator-preflight.v1|"
             << AL_R20_MANIFEST_SHA256 << ':' << parent_exact << ':'
             << scaling_exact << ':' << excitation_exact << ':'
             << lifecycle_exact << '|';
    for (const ALR20ProblemData& problem : problems)
        semantic << problem.problem_root << ';';
    semantic << binary64_bits(maximum_primal_gap) << ':'
             << binary64_bits(maximum_dual_gap) << ':'
             << binary64_bits(maximum_scale_gap);
    const std::string result_sha = sha256_hex(semantic.str());
    const std::string route = !parent_exact
        ? "GENERALIZATION_PARENT_REJECTED"
        : (!scaling_exact ? "GENERALIZATION_SCALING_REJECTED"
          : (!excitation_exact ? "GENERALIZATION_EXCITATION_REJECTED"
                          : (exact ? "GENERALIZATION_OPERATOR_PREFLIGHT_CANDIDATE"
                                   : "GENERALIZATION_OPERATOR_REJECTED")));
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20_operator_preflight.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_manifest_sha256\":\""
           << AL_R20_MANIFEST_SHA256 << "\",\"parent_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20ProblemData& problem = problems[index];
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"exact\":" << (problem.exact ? "true" : "false")
               << ",\"failure\":\"" << problem.failure
               << "\",\"rows\":" << problem.diagonal.size()
               << ",\"slots\":" << problem.op.slots.size()
               << ",\"entries\":" << problem.op.entries.size()
               << ",\"minimum_degree\":" << problem.structure.minimum_degree
               << ",\"maximum_degree\":" << problem.structure.maximum_degree
               << ",\"source_positive\":" << problem.source_positive
               << ",\"projected_target_positive\":"
               << problem.projected_positive
               << ",\"source_maximum\":" << problem.source_maximum
               << ",\"projected_target_maximum\":"
               << problem.projected_maximum
               << ",\"row_scale_minimum\":" << problem.scale_minimum
               << ",\"row_scale_maximum\":" << problem.scale_maximum
               << ",\"target_sha256\":\"" << problem.target_root
               << "\",\"constraint_sha256\":\"" << problem.constraint_root
               << "\",\"domain_sha256\":\"" << problem.domain_root
               << "\",\"operator_sha256\":\"" << problem.operator_root
               << "\",\"problem_sha256\":\"" << problem.problem_root
               << "\"}";
    }
    report << "]"
           << ",\"excitation\":{\"excited_transfer\":"
           << excited_transfer << ",\"excited_holdout\":"
           << excited_holdout << ",\"quiet_negative\":"
           << quiet_negative << ",\"exact\":"
           << (excitation_exact ? "true" : "false") << "}"
           << ",\"scaling\":{\"factors\":[0.00390625,256]"
           << ",\"rows_per_case\":3,\"exact\":"
           << (scaling_exact ? "true" : "false")
           << ",\"maximum_primal_gap\":" << maximum_primal_gap
           << ",\"maximum_dual_gap\":" << maximum_dual_gap
           << ",\"maximum_scale_gap\":" << maximum_scale_gap << "}"
           << ",\"work\":{\"workspace_builds\":"
           << sparse_work.workspace_builds
           << ",\"workspace_releases\":" << sparse_work.workspace_releases
           << ",\"maximum_live_workspaces\":"
           << sparse_work.maximum_live_workspaces
           << ",\"pair_visits\":" << sparse_work.pair_visits << "}"
           << ",\"solver_iterations\":0,\"oracle_cycles\":0"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
