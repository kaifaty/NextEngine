namespace {

constexpr const char* B4E2D7R19R29_IDENTITY_SHA256 =
    "e1d54356d0f0c6e65434b5670f63bf92780612eeb19791b388702831e85224f2";
constexpr const char* B4E2D7R19R29_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r29-linearized-feasibility-operator|v1|parent=ca652ba9:9c953a118d6babb192be6be23e94374576f8a48619754567d341576c33a37237:006620643aae830c4bfe5d07e3d0a0610d34ac0fb2ada57d452597ddaa4f12d0|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;receipt-b145ce1af6d3ffca167f2264b55618b0bfccc55c340ff1eee17206569a239af6;history-fd838a046b2629f5d5e4d5aa0ab87160078b2e4b748b98101ac2da00bfa3a37d;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;dual-cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d;epoch1;next-outer12;slice333;cumulative856;used-12,2,26,1,333,856,58,34,824,32,2,34,0;primal0x3e53c2bf74000000;stationarity0x3d2e9d0a6841b669;particles6000;theta0x3fc5cccccccccccd;static-a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8|operator=A=SPACING*Jc;fluid-dof3N;support-fixed;binary64-owned-pairs;pair-once-jvp-vjp;directed-row-reference|rows=all;violated-c>0;phr-active-u+c>0;boundary-coupled;interior;exact-membership-roots|probes=deterministic-fluid-rms1;deterministic-row-rms1;frozen-topology-centered-difference-epsilon2^-16;constant-translation|bounds=pair-directed-componentwise-gamma(16*max-degree+64);finite-difference-relative-l2<=1e-6;adjoint-relative<=1e-12;translation-interior-exact-zero;pair-passes<=8|result=operator-and-partition-roots-derived-after-execution;no-range-solve;no-floor-classification|controls=parent;source;workspace;topology;row-partitions;jvp-reference;finite-difference;adjoint;translation;work;rollback|routes=linearized-operator-parent-rejected;linearized-operator-source-rejected;linearized-operator-workspace-rejected;linearized-operator-topology-rejected;linearized-operator-partition-rejected;linearized-operator-jvp-rejected;linearized-operator-finite-difference-rejected;linearized-operator-adjoint-rejected;linearized-operator-translation-rejected;linearized-operator-work-rejected;linearized-feasibility-operator-candidate|precedence=parent,source,workspace,topology,partition,jvp,finite-difference,adjoint,translation,work,candidate|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;diagnostic-workspaces1;pair-passes<=8;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|lsqr=none;correction=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-linearized-feasibility-operator-candidate-only)IDENTITY";

struct ALLinearizedJvp {
    bool passed = false;
    std::vector<double> value;
    std::vector<double> absolute_sum;
};

double al_r29_scalar_norm(const std::vector<double>& values) {
    long double sum = 0.0L;
    for (double value : values) {
        sum += static_cast<long double>(value)
            * static_cast<long double>(value);
    }
    return std::sqrt(static_cast<double>(sum));
}

std::string al_r29_mask_root(const std::vector<std::uint8_t>& values) {
    std::ostringstream projection;
    projection << values.size() << '|';
    for (std::uint8_t value : values) projection << static_cast<int>(value);
    return sha256_hex(projection.str());
}

std::string al_r29_topology_root(const ALNormalizedWorkspace& workspace) {
    std::ostringstream projection;
    const JointNeighborhood& n = workspace.neighborhood;
    projection << n.fluid.size() << ':' << n.support.size() << ':'
               << n.pairs.size() << ':' << n.fluid_pairs << ':'
               << n.support_pairs << ':' << n.maximum_degree << '|';
    for (std::size_t index = 0U; index < n.pairs.size(); ++index) {
        projection << n.pairs[index].fluid << ':'
                   << n.pairs[index].participant << ':'
                   << binary64_bits(workspace.radius[index]) << ':'
                   << binary64_bits(workspace.weight_first[index]) << ';';
    }
    projection << '|';
    for (std::uint32_t value : n.flat_offsets) projection << value << ':';
    projection << '|';
    for (std::uint32_t value : n.flat_directed_pair_indices) {
        projection << value << ':';
    }
    return sha256_hex(projection.str());
}

std::vector<Vec3> al_r29_fluid_probe(std::size_t count) {
    std::vector<Vec3> result = deterministic_direction(count);
    const double scale = std::sqrt(static_cast<double>(count));
    for (Vec3& value : result) value = scale * value;
    return result;
}

std::vector<double> al_r29_row_probe(std::size_t count) {
    std::vector<double> result(count);
    long double squared = 0.0L;
    for (std::size_t index = 0U; index < count; ++index) {
        const double i = static_cast<double>(index + 1U);
        result[index] = std::sin(0.41 * i) + 0.5 * std::cos(1.37 * i);
        squared += static_cast<long double>(result[index])
            * static_cast<long double>(result[index]);
    }
    const double rms = count == 0U ? 0.0
        : std::sqrt(static_cast<double>(squared)
            / static_cast<double>(count));
    if (rms > 0.0) {
        for (double& value : result) value /= rms;
    }
    return result;
}

Vec3 al_r29_pair_jacobian(
    const ALNormalizedWorkspace& workspace,
    std::size_t pair_index) {
    const JointPair pair = workspace.neighborhood.pairs[pair_index];
    const double radius = workspace.radius[pair_index];
    if (radius <= 1.0e-15 || radius > HORIZON) return {};
    const Vec3 displacement =
        workspace.neighborhood.fluid[pair.fluid].position
        - joint_position(workspace.neighborhood, pair.participant);
    return MASS / REST_DENSITY * workspace.weight_first[pair_index]
        * (displacement / radius);
}

ALLinearizedJvp al_r29_pair_once_jvp(
    const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& direction) {
    ALLinearizedJvp result;
    const std::size_t count = workspace.neighborhood.fluid.size();
    if (!workspace.passed || direction.size() != count) return result;
    result.value.assign(count, 0.0);
    result.absolute_sum.assign(count, 0.0);
    for (std::size_t pair_index = 0U;
         pair_index < workspace.neighborhood.pairs.size(); ++pair_index) {
        const JointPair pair = workspace.neighborhood.pairs[pair_index];
        const Vec3 jacobian = al_r29_pair_jacobian(workspace, pair_index);
        const Vec3 relative = pair.participant < count
            ? direction[pair.fluid] - direction[pair.participant]
            : direction[pair.fluid];
        const double term = SPACING * dot(jacobian, relative);
        result.value[pair.fluid] += term;
        result.absolute_sum[pair.fluid] += std::abs(term);
        if (pair.participant < count) {
            result.value[pair.participant] += term;
            result.absolute_sum[pair.participant] += std::abs(term);
        }
    }
    result.passed = std::all_of(result.value.begin(), result.value.end(),
        [](double value) { return std::isfinite(value); });
    return result;
}

ALLinearizedJvp al_r29_directed_jvp(
    const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& direction) {
    ALLinearizedJvp result;
    const std::size_t count = workspace.neighborhood.fluid.size();
    if (!workspace.passed || direction.size() != count) return result;
    result.value.assign(count, 0.0);
    result.absolute_sum.assign(count, 0.0);
    for (std::size_t center = 0U; center < count; ++center) {
        const std::size_t begin = workspace.neighborhood.flat_offsets[center];
        const std::size_t end = workspace.neighborhood.flat_offsets[center + 1U];
        for (std::size_t slot = begin; slot < end; ++slot) {
            const std::size_t pair_index =
                workspace.neighborhood.flat_directed_pair_indices[slot];
            const std::size_t participant = joint_pair_participant(
                workspace.neighborhood,
                workspace.neighborhood.pairs[pair_index], center);
            const double radius = workspace.radius[pair_index];
            if (radius <= 1.0e-15 || radius > HORIZON) continue;
            const Vec3 displacement =
                workspace.neighborhood.fluid[center].position
                - joint_position(workspace.neighborhood, participant);
            const Vec3 jacobian = MASS / REST_DENSITY
                * workspace.weight_first[pair_index]
                * (displacement / radius);
            const Vec3 relative = participant < count
                ? direction[center] - direction[participant]
                : direction[center];
            const double term = SPACING * dot(jacobian, relative);
            result.value[center] += term;
            result.absolute_sum[center] += std::abs(term);
        }
    }
    result.passed = std::all_of(result.value.begin(), result.value.end(),
        [](double value) { return std::isfinite(value); });
    return result;
}

std::vector<Vec3> al_r29_pair_once_vjp(
    const ALNormalizedWorkspace& workspace,
    const std::vector<double>& weights) {
    const std::size_t count = workspace.neighborhood.fluid.size();
    std::vector<Vec3> result(count);
    if (!workspace.passed || weights.size() != count) return {};
    for (std::size_t pair_index = 0U;
         pair_index < workspace.neighborhood.pairs.size(); ++pair_index) {
        const JointPair pair = workspace.neighborhood.pairs[pair_index];
        const Vec3 jacobian = al_r29_pair_jacobian(workspace, pair_index);
        const double row_weight = pair.participant < count
            ? weights[pair.fluid] + weights[pair.participant]
            : weights[pair.fluid];
        const Vec3 term = SPACING * row_weight * jacobian;
        result[pair.fluid] += term;
        if (pair.participant < count) result[pair.participant] += -term;
    }
    return result;
}

std::vector<double> al_r29_frozen_constraints(
    const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& position) {
    const std::size_t count = workspace.neighborhood.fluid.size();
    if (!workspace.passed || position.size() != count) return {};
    std::vector<double> density(count, MASS * weight(0.0));
    for (const JointPair pair : workspace.neighborhood.pairs) {
        const Vec3 participant = pair.participant < count
            ? position[pair.participant]
            : workspace.neighborhood.support[
                pair.participant - count].position;
        const double contribution = MASS * weight(
            norm(position[pair.fluid] - participant));
        density[pair.fluid] += contribution;
        if (pair.participant < count) density[pair.participant] += contribution;
    }
    std::vector<double> result(count);
    for (std::size_t index = 0U; index < count; ++index) {
        result[index] = density[index] / REST_DENSITY - 1.0;
    }
    return result;
}

double al_r29_relative_error(
    const std::vector<double>& lhs,
    const std::vector<double>& rhs) {
    if (lhs.size() != rhs.size() || lhs.empty()) {
        return std::numeric_limits<double>::infinity();
    }
    std::vector<double> difference(lhs.size());
    for (std::size_t index = 0U; index < lhs.size(); ++index) {
        difference[index] = lhs[index] - rhs[index];
    }
    return al_r29_scalar_norm(difference)
        / std::max(al_r29_scalar_norm(rhs), 1.0e-300);
}

std::string al_r29_route(
    bool parent, bool source, bool workspace, bool topology,
    bool partition, bool jvp, bool finite_difference, bool adjoint,
    bool translation, bool work) {
    if (!parent) return "LINEARIZED_OPERATOR_PARENT_REJECTED";
    if (!source) return "LINEARIZED_OPERATOR_SOURCE_REJECTED";
    if (!workspace) return "LINEARIZED_OPERATOR_WORKSPACE_REJECTED";
    if (!topology) return "LINEARIZED_OPERATOR_TOPOLOGY_REJECTED";
    if (!partition) return "LINEARIZED_OPERATOR_PARTITION_REJECTED";
    if (!jvp) return "LINEARIZED_OPERATOR_JVP_REJECTED";
    if (!finite_difference) {
        return "LINEARIZED_OPERATOR_FINITE_DIFFERENCE_REJECTED";
    }
    if (!adjoint) return "LINEARIZED_OPERATOR_ADJOINT_REJECTED";
    if (!translation) return "LINEARIZED_OPERATOR_TRANSLATION_REJECTED";
    if (!work) return "LINEARIZED_OPERATOR_WORK_REJECTED";
    return "LINEARIZED_FEASIBILITY_OPERATOR_CANDIDATE";
}

} // namespace

SplitBoundaryReport run_al_linearized_feasibility_operator_controls_impl(
    ALShadowOuter11ExecutionCapture* output_capture) {
    const bool identity_exact = sha256_hex(B4E2D7R19R29_IDENTITY_PROJECTION)
        == B4E2D7R19R29_IDENTITY_SHA256;
    ALShadowOuter11ExecutionCapture capture;
    const SplitBoundaryReport parent =
        run_al_shadow_outer11_execution_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "006620643aae830c4bfe5d07e3d0a0610d34ac0fb2ada57d452597ddaa4f12d0"
        "\"") != std::string::npos;
    const bool parent_exact = identity_exact && parent.passed
        && capture.captured && parent_semantic && parent_stdout
            == "9c953a118d6babb192be6be23e94374576f8a48619754567d341576c33a37237";

    const std::vector<Vec3>& position = capture.candidate.update.position;
    const std::vector<double>& dual = capture.candidate.update.u;
    const auto& r26 = capture.r27.r26;
    const auto& r25 = r26.r25;
    const auto& r24 = r25.r24;
    const auto& r23 = r24.r23;
    const auto& r22 = r23.r22;
    const auto& r21 = r22.r21;
    const auto& r20 = r21.r20;
    const auto& r19 = r20.r19;
    const auto& r18 = r19.r18;
    const auto& r17 = r18.r17;
    const ALV2EnvelopeCapture& r16 = r17.r16;
    const ALSoftCapSuspensionCapture& r14 = r16.r15.r14;
    const ALNormalizedTotalHvpBoundaryCapture& boundary =
        r14.r13.r12.r11.r10.total_hvp_boundary;
    const bool source_exact = capture.state_root
            == "851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7"
        && capture.receipt_root
            == "b145ce1af6d3ffca167f2264b55618b0bfccc55c340ff1eee17206569a239af6"
        && capture.receipt.roots[6]
            == "fd838a046b2629f5d5e4d5aa0ab87160078b2e4b748b98101ac2da00bfa3a37d"
        && al_binary64_vec3_root(position)
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && al_r10_dual_root(dual)
            == "cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d"
        && position.size() == 6000U && dual.size() == position.size()
        && binary64_bits(boundary.theta) == 0x3fc5cccccccccccdULL
        && capture.receipt.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                12U,2U,26U,1U,333U,856U,58U,34U,824U,32U,2U,34U,0U};
    const std::string position_before = al_binary64_vec3_root(position);
    const std::string dual_before = al_r10_dual_root(dual);
    const std::string constraint_before = al_r10_dual_root(
        capture.candidate.update.support.constraint);

    ALSparseWorkTrace work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    ALNormalizedWorkspace workspace;
    if (parent_exact && source_exact && index.passed && binding.passed) {
        workspace = build_al_normalized_sparse_workspace(
            position, binding, dual, boundary.theta, &work,
            &static_work, &adjacency_work);
    }
    const bool workspace_exact = workspace.passed
        && index.identity_sha256
            == "a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8"
        && workspace.support.density
            == capture.candidate.update.support.density
        && workspace.support.constraint
            == capture.candidate.update.support.constraint;
    const std::string topology_root = al_r29_topology_root(workspace);
    const bool topology_exact = workspace_exact
        && !topology_root.empty()
        && workspace.neighborhood.flat_adjacency
        && workspace.neighborhood.flat_offsets.size() == position.size() + 1U
        && workspace.neighborhood.maximum_degree <= B4C0_MAX_NEIGHBORS;
    const std::size_t pair_count = workspace.neighborhood.pairs.size();
    const std::size_t maximum_degree = workspace.neighborhood.maximum_degree;

    const std::size_t count = position.size();
    std::vector<std::uint8_t> violated(count, 0U);
    std::vector<std::uint8_t> active(count, 0U);
    std::vector<std::uint8_t> boundary_rows(count, 0U);
    std::vector<std::uint8_t> interior(count, 1U);
    std::size_t violated_count = 0U;
    std::size_t active_count = 0U;
    for (std::size_t index_value = 0U; index_value < count; ++index_value) {
        violated[index_value] =
            workspace_exact && workspace.support.constraint[index_value] > 0.0;
        active[index_value] = workspace_exact
            && dual[index_value] + workspace.support.constraint[index_value] > 0.0;
        violated_count += violated[index_value];
        active_count += active[index_value];
    }
    if (workspace_exact) {
        for (const JointPair pair : workspace.neighborhood.pairs) {
            if (pair.participant >= count) boundary_rows[pair.fluid] = 1U;
        }
    }
    std::size_t boundary_count = 0U;
    for (std::size_t index_value = 0U; index_value < count; ++index_value) {
        interior[index_value] = boundary_rows[index_value] ? 0U : 1U;
        boundary_count += boundary_rows[index_value];
    }
    const std::size_t interior_count = count - boundary_count;
    const bool partitions_exact = topology_exact && violated_count > 0U
        && active_count >= violated_count && active_count <= count
        && boundary_count > 0U && interior_count > 0U;
    const std::string violated_root = al_r29_mask_root(violated);
    const std::string active_root = al_r29_mask_root(active);
    const std::string boundary_root = al_r29_mask_root(boundary_rows);
    const std::string interior_root = al_r29_mask_root(interior);

    std::size_t pair_passes = 0U;
    const std::vector<Vec3> direction = al_r29_fluid_probe(count);
    const ALLinearizedJvp candidate =
        al_r29_pair_once_jvp(workspace, direction);
    ++pair_passes;
    const ALLinearizedJvp reference =
        al_r29_directed_jvp(workspace, direction);
    ++pair_passes;
    const double operations = 16.0
        * static_cast<double>(workspace.neighborhood.maximum_degree) + 64.0;
    const double unit = std::numeric_limits<double>::epsilon();
    const double gamma = operations * unit / (1.0 - operations * unit);
    std::size_t jvp_bound_failures = 0U;
    double maximum_jvp_bound_ratio = 0.0;
    if (candidate.value.size() == reference.value.size()) {
        for (std::size_t index_value = 0U;
             index_value < candidate.value.size(); ++index_value) {
            const double error = std::abs(
                candidate.value[index_value] - reference.value[index_value]);
            const double scale = std::max({
                candidate.absolute_sum[index_value],
                reference.absolute_sum[index_value], 1.0e-300});
            const double bound = gamma * scale;
            if (error > bound) ++jvp_bound_failures;
            maximum_jvp_bound_ratio = std::max(maximum_jvp_bound_ratio,
                bound > 0.0 ? error / bound : (error == 0.0 ? 0.0
                    : std::numeric_limits<double>::infinity()));
        }
    } else {
        jvp_bound_failures = 1U;
    }
    const bool jvp_exact = candidate.passed && reference.passed
        && jvp_bound_failures == 0U
        && std::isfinite(maximum_jvp_bound_ratio);

    constexpr double epsilon = 0x1p-16;
    std::vector<Vec3> plus = position;
    std::vector<Vec3> minus = position;
    for (std::size_t index_value = 0U; index_value < count; ++index_value) {
        plus[index_value] += SPACING * epsilon * direction[index_value];
        minus[index_value] += -SPACING * epsilon * direction[index_value];
    }
    const std::vector<double> plus_constraint =
        al_r29_frozen_constraints(workspace, plus);
    ++pair_passes;
    const std::vector<double> minus_constraint =
        al_r29_frozen_constraints(workspace, minus);
    ++pair_passes;
    std::vector<double> finite_difference(count);
    if (plus_constraint.size() == count && minus_constraint.size() == count) {
        for (std::size_t index_value = 0U; index_value < count; ++index_value) {
            finite_difference[index_value] =
                (plus_constraint[index_value] - minus_constraint[index_value])
                / (2.0 * epsilon);
        }
    }
    const double finite_difference_error = al_r29_relative_error(
        finite_difference, candidate.value);
    const bool finite_difference_exact = jvp_exact
        && std::isfinite(finite_difference_error)
        && finite_difference_error <= 1.0e-6;

    const std::vector<double> row_weights = al_r29_row_probe(count);
    const std::vector<Vec3> adjoint =
        al_r29_pair_once_vjp(workspace, row_weights);
    ++pair_passes;
    long double lhs = 0.0L;
    long double rhs = 0.0L;
    if (adjoint.size() == count && row_weights.size() == count) {
        for (std::size_t index_value = 0U; index_value < count; ++index_value) {
            lhs += static_cast<long double>(candidate.value[index_value])
                * static_cast<long double>(row_weights[index_value]);
            rhs += static_cast<long double>(direction[index_value].x)
                    * static_cast<long double>(adjoint[index_value].x)
                + static_cast<long double>(direction[index_value].y)
                    * static_cast<long double>(adjoint[index_value].y)
                + static_cast<long double>(direction[index_value].z)
                    * static_cast<long double>(adjoint[index_value].z);
        }
    }
    const long double adjoint_scale = std::max({
        std::abs(lhs), std::abs(rhs), 1.0e-300L});
    const double adjoint_error = static_cast<double>(
        std::abs(lhs - rhs) / adjoint_scale);
    const bool adjoint_exact = adjoint.size() == count
        && std::all_of(adjoint.begin(), adjoint.end(),
            [](Vec3 value) { return finite(value); })
        && std::isfinite(adjoint_error) && adjoint_error <= 1.0e-12;

    const std::vector<Vec3> translation(count, Vec3{1.0, -0.5, 0.25});
    const ALLinearizedJvp translation_image =
        al_r29_pair_once_jvp(workspace, translation);
    ++pair_passes;
    bool translation_interior_zero = translation_image.passed;
    bool translation_boundary_nonzero = false;
    for (std::size_t index_value = 0U; index_value < count; ++index_value) {
        if (interior[index_value] && translation_image.value[index_value] != 0.0) {
            translation_interior_zero = false;
        }
        if (boundary_rows[index_value]
            && translation_image.value[index_value] != 0.0) {
            translation_boundary_nonzero = true;
        }
    }
    const bool translation_exact = translation_interior_zero
        && translation_boundary_nonzero;

    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes <= 8U
        && work.workspace_builds == 1U && work.workspace_releases == 1U
        && work.live_workspaces == 0U && work.maximum_live_workspaces == 1U
        && !work.lifecycle_underflow && work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 1U
        && adjacency_work.workspace_builds == 1U;
    const bool rollback_exact = position_before == al_binary64_vec3_root(position)
        && dual_before == al_r10_dual_root(dual)
        && constraint_before == al_r10_dual_root(
            capture.candidate.update.support.constraint);

    struct RouteCase {
        const char* expected;
        std::string observed;
    };
    const std::array<RouteCase, 11U> routes{{
        {"LINEARIZED_FEASIBILITY_OPERATOR_CANDIDATE", al_r29_route(1,1,1,1,1,1,1,1,1,1)},
        {"LINEARIZED_OPERATOR_PARENT_REJECTED", al_r29_route(0,1,1,1,1,1,1,1,1,1)},
        {"LINEARIZED_OPERATOR_SOURCE_REJECTED", al_r29_route(1,0,1,1,1,1,1,1,1,1)},
        {"LINEARIZED_OPERATOR_WORKSPACE_REJECTED", al_r29_route(1,1,0,1,1,1,1,1,1,1)},
        {"LINEARIZED_OPERATOR_TOPOLOGY_REJECTED", al_r29_route(1,1,1,0,1,1,1,1,1,1)},
        {"LINEARIZED_OPERATOR_PARTITION_REJECTED", al_r29_route(1,1,1,1,0,1,1,1,1,1)},
        {"LINEARIZED_OPERATOR_JVP_REJECTED", al_r29_route(1,1,1,1,1,0,1,1,1,1)},
        {"LINEARIZED_OPERATOR_FINITE_DIFFERENCE_REJECTED", al_r29_route(1,1,1,1,1,1,0,1,1,1)},
        {"LINEARIZED_OPERATOR_ADJOINT_REJECTED", al_r29_route(1,1,1,1,1,1,1,0,1,1)},
        {"LINEARIZED_OPERATOR_TRANSLATION_REJECTED", al_r29_route(1,1,1,1,1,1,1,1,0,1)},
        {"LINEARIZED_OPERATOR_WORK_REJECTED", al_r29_route(1,1,1,1,1,1,1,1,1,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const RouteCase& value : routes) {
        routes_exact = routes_exact && value.expected == value.observed;
        route_projection << value.expected << ':' << value.observed << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const bool passed = parent_exact && source_exact && workspace_exact
        && topology_exact && partitions_exact && jvp_exact
        && finite_difference_exact && adjoint_exact && translation_exact
        && work_exact && rollback_exact && routes_exact;
    const std::string route = passed
        ? "LINEARIZED_FEASIBILITY_OPERATOR_CANDIDATE" : std::string{};
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!topology_exact) first_failure = "TOPOLOGY";
    else if (!partitions_exact) first_failure = "PARTITION";
    else if (!jvp_exact) first_failure = "JVP";
    else if (!finite_difference_exact) first_failure = "FINITE_DIFFERENCE";
    else if (!adjoint_exact) first_failure = "ADJOINT";
    else if (!translation_exact) first_failure = "TRANSLATION";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    const std::string jvp_root = al_r10_dual_root(candidate.value);
    const std::string reference_root = al_r10_dual_root(reference.value);
    const std::string finite_difference_root =
        al_r10_dual_root(finite_difference);
    const std::string adjoint_root = al_binary64_vec3_root(adjoint);
    const std::string translation_root =
        al_r10_dual_root(translation_image.value);
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R29_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << topology_root << '|' << violated_root << ':' << active_root
             << ':' << boundary_root << ':' << interior_root << '|'
             << jvp_root << ':' << reference_root << ':'
             << finite_difference_root << ':' << adjoint_root << ':'
             << translation_root << '|' << binary64_bits(finite_difference_error)
             << ':' << binary64_bits(adjoint_error) << ':'
             << jvp_bound_failures << ':' << pair_passes << '|'
             << route_root << ':' << routes_exact << '|' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r29_linearized_feasibility_operator.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R29_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"topology_root\":\"" << topology_root << "\""
           << ",\"particles\":" << count
           << ",\"pairs\":" << pair_count
           << ",\"maximum_degree\":" << maximum_degree
           << ",\"rows\":{\"violated\":" << violated_count
           << ",\"active\":" << active_count
           << ",\"boundary\":" << boundary_count
           << ",\"interior\":" << interior_count
           << ",\"violated_root\":\"" << violated_root << "\""
           << ",\"active_root\":\"" << active_root << "\""
           << ",\"boundary_root\":\"" << boundary_root << "\""
           << ",\"interior_root\":\"" << interior_root << "\"}"
           << ",\"operator\":{\"jvp_root\":\"" << jvp_root << "\""
           << ",\"reference_root\":\"" << reference_root << "\""
           << ",\"jvp_bound_failures\":" << jvp_bound_failures
           << ",\"maximum_jvp_bound_ratio\":"
           << std::setprecision(17) << maximum_jvp_bound_ratio
           << ",\"finite_difference_root\":\""
           << finite_difference_root << "\""
           << ",\"finite_difference_relative_l2\":"
           << finite_difference_error
           << ",\"adjoint_root\":\"" << adjoint_root << "\""
           << ",\"adjoint_relative\":" << adjoint_error
           << ",\"translation_root\":\"" << translation_root << "\"}"
           << ",\"work\":{\"diagnostic_workspaces\":1,\"pair_passes\":"
           << pair_passes
           << ",\"new_hvp\":0,\"new_trials\":0,\"new_precision\":0"
              ",\"new_outer\":0,\"lsqr_iterations\":0}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"state_mutation\":false,\"floor_classification\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    SplitBoundaryReport result{passed, report.str()};
    if (output_capture != nullptr) {
        *output_capture = std::move(capture);
    }
    return result;
}

SplitBoundaryReport run_al_linearized_feasibility_operator_controls() {
    return run_al_linearized_feasibility_operator_controls_impl(nullptr);
}
