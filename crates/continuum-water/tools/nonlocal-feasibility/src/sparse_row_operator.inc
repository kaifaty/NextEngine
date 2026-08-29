namespace {

constexpr const char* B4E2D7R19R64_IDENTITY_SHA256 =
    "a3cd93bb1dc785e7784ff048bc8989b19215e5bc010ffe7a859b3a098f3ffca2";
constexpr const char* B4E2D7R19R64_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r64-sparse-row-operator-equivalence|v1|parent=87a4f2ee:38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62:9f456232956a0e080063123acf8a4a0b2dac7a6fd8f2615b270f0958e66a8440:TANGENTIAL_MASTER_EXPANSION_REQUIRED|source=r63-state-216c12aeeb47686e823a9911f0aa9dd14ddee53db1701874488dc325a292cea5;position-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;topology-cfcc7ebbeac9ce8d54093d3af109367e5b68b4aceadf11ed516dbfddf5c2d73f;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5;degree44,113,53,5992|representation=all6000rows;directed-slot-offsets-equal-workspace-flat-offsets;slot-pair;slot-participant;unscaled-pair-jacobian;aggregated-entry-offsets;unique-sorted-fluid-particle;coefficient-spacing-times-row-gradient;particle-incidence-offsets;incidence-row-and-entry-backreference;stable-order|action=deterministic-r29-fluid-probe;sparse-directed-slot-order;same-spacing-times-dot-expression;value-and-absolute-sum;fresh-directed-reference|transpose=deterministic-r29-row-probe;sparse-aggregated-row-order;fresh-pair-once-vjp;component-bound=gamma(8*particle-incidence-degree+64)*(reference-absolute+sparse-absolute)|captured=all494-r51rows;full-gradient-reconstruct;component-bound=gamma(8*row-degree+64)*(reference-absolute+sparse-absolute+entry-absolute);direct-diagonal;long-double-diagonal;captured-relative<=1e-12|overlap=particle-to-row-incidence;epoch-tags;dense-scratch6000;touched-rows-only;stable-entry-incidence-order;all494-columns-by6000rows;component-bound=gamma(32*(source-row-degree+target-row-degree)+128)*(reference-absolute+sparse-absolute+overlap-absolute)|classification=all-action-transpose-gradient-diagonal-gram-bits-exact->sparse-row-operator-exact-equivalence-candidate;else-all-predeclared-bounds-pass->sparse-row-operator-bounded-equivalence-candidate;else-arithmetic-alignment-required|controls=parent;source;capture;workspace;topology;slot-structure;entry-structure;incidence-structure;action;transpose;captured-gradient;diagonal;overlap;bounds;mutation-offset;mutation-entry-particle;mutation-incidence-backreference;mutation-epoch-reuse;work;rollback;route-precedence|routes=sparse-row-parent-rejected;sparse-row-source-rejected;sparse-row-capture-rejected;sparse-row-workspace-rejected;sparse-row-topology-rejected;sparse-row-slot-rejected;sparse-row-entry-rejected;sparse-row-incidence-rejected;sparse-row-action-rejected;sparse-row-transpose-rejected;sparse-row-gradient-rejected;sparse-row-diagonal-rejected;sparse-row-overlap-rejected;sparse-row-bound-rejected;sparse-row-controls-rejected;sparse-row-work-rejected;sparse-row-operator-exact-equivalence-candidate;sparse-row-operator-bounded-equivalence-candidate;sparse-row-operator-arithmetic-alignment-required|precedence=parent,source,capture,workspace,topology,slot,entry,incidence,action,transpose,gradient,diagonal,overlap,bound,controls,work,exact,bounded,alignment|work=parent-r63-replays1;new-static-index1;new-r43-workspace1;workspace-release1;sparse-row-builds6000;slot-scans1;entry-builds6000;incidence-builds1;fresh-directed-jvp1;fresh-pair-vjp1;captured-gradient-checks494;captured-diagonal-checks494;incidence-overlap-columns494;column-values2964000;dense-gram-storage0;new-row-vjp0;new-gram-jvp0;new-hildreth0;new-projection0;new-nonlinear-trial0;new-hvp0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59-r60-r61-r62-r63=unchanged;r63-public-output=byte-exact;operator-private-only;dynamic-active-set=none;runtime-state=none;normal-apply=none;tangential-apply=none;filter=none;switching=none;trust-update=none;following-outer=none;tolerance-fit=none;binary128=none;timing=none;runtime=none;production=none|credit=one-private-sparse-row-operator-equivalence-classification-only)IDENTITY";

struct ALR64Slot {
    std::size_t pair = 0U;
    std::size_t participant = 0U;
    Vec3 jacobian;
};

struct ALR64Entry {
    std::size_t particle = 0U;
    Vec3 coefficient;
};

struct ALR64Incidence {
    std::size_t row = 0U;
    std::size_t entry = 0U;
};

struct ALR64Operator {
    std::vector<std::uint32_t> slot_offsets;
    std::vector<ALR64Slot> slots;
    std::vector<std::uint32_t> entry_offsets;
    std::vector<ALR64Entry> entries;
    std::vector<std::uint32_t> particle_offsets;
    std::vector<ALR64Incidence> incidences;
};

struct ALR64StructureAudit {
    bool slot = false;
    bool entry = false;
    bool incidence = false;
    std::size_t minimum_degree = 0U;
    std::size_t maximum_degree = 0U;
    std::size_t distinct_degrees = 0U;
    std::size_t below_maximum = 0U;
};

struct ALR64Comparison {
    bool passed = false;
    bool bits_exact = true;
    std::size_t values = 0U;
    std::size_t exact_values = 0U;
    double maximum_error = 0.0;
    double maximum_bound = 0.0;
    double maximum_ratio = 0.0;
};

bool al_r64_finite(Vec3 value) {
    return std::isfinite(value.x) && std::isfinite(value.y)
        && std::isfinite(value.z);
}

double al_r64_component(Vec3 value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

void al_r64_accumulate_absolute(Vec3 value, Vec3* target) {
    target->x += std::abs(value.x);
    target->y += std::abs(value.y);
    target->z += std::abs(value.z);
}

ALR64Operator al_r64_build(const ALNormalizedWorkspace& workspace) {
    ALR64Operator result;
    const JointNeighborhood& neighborhood = workspace.neighborhood;
    const std::size_t rows = neighborhood.fluid.size();
    result.slot_offsets = neighborhood.flat_offsets;
    result.slots.reserve(neighborhood.flat_directed_pair_indices.size());
    result.entry_offsets.reserve(rows + 1U);
    result.entry_offsets.push_back(0U);
    for (std::size_t row = 0U; row < rows; ++row) {
        Vec3 center;
        std::vector<ALR64Entry> row_entries;
        const std::size_t begin = neighborhood.flat_offsets[row];
        const std::size_t end = neighborhood.flat_offsets[row + 1U];
        row_entries.reserve(end - begin + 1U);
        for (std::size_t index = begin; index < end; ++index) {
            const std::size_t pair_index =
                neighborhood.flat_directed_pair_indices[index];
            const std::size_t participant = joint_pair_participant(
                neighborhood, neighborhood.pairs[pair_index], row);
            const double radius = workspace.radius[pair_index];
            Vec3 jacobian;
            if (radius > 1.0e-15 && radius <= HORIZON) {
                const Vec3 displacement = neighborhood.fluid[row].position
                    - joint_position(neighborhood, participant);
                jacobian = MASS / REST_DENSITY
                    * workspace.weight_first[pair_index]
                    * (displacement / radius);
            }
            result.slots.push_back({pair_index, participant, jacobian});
            const Vec3 coefficient = SPACING * jacobian;
            center += coefficient;
            if (participant < rows)
                row_entries.push_back({participant, -coefficient});
        }
        row_entries.push_back({row, center});
        std::sort(row_entries.begin(), row_entries.end(),
            [](const ALR64Entry& lhs, const ALR64Entry& rhs) {
                return lhs.particle < rhs.particle;
            });
        for (const ALR64Entry& entry : row_entries) {
            if (!result.entries.empty()
                && result.entries.back().particle == entry.particle
                && result.entries.size() > result.entry_offsets.back()) {
                result.entries.back().coefficient += entry.coefficient;
            } else {
                result.entries.push_back(entry);
            }
        }
        result.entry_offsets.push_back(
            static_cast<std::uint32_t>(result.entries.size()));
    }

    result.particle_offsets.assign(rows + 1U, 0U);
    for (const ALR64Entry& entry : result.entries)
        ++result.particle_offsets[entry.particle + 1U];
    for (std::size_t i = 1U; i < result.particle_offsets.size(); ++i)
        result.particle_offsets[i] += result.particle_offsets[i - 1U];
    result.incidences.resize(result.entries.size());
    std::vector<std::uint32_t> cursor = result.particle_offsets;
    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t index = result.entry_offsets[row];
             index < result.entry_offsets[row + 1U]; ++index) {
            const std::size_t particle = result.entries[index].particle;
            result.incidences[cursor[particle]++] = {row, index};
        }
    return result;
}

ALR64StructureAudit al_r64_structure(
    const ALNormalizedWorkspace& workspace, const ALR64Operator& op) {
    ALR64StructureAudit result;
    const JointNeighborhood& neighborhood = workspace.neighborhood;
    const std::size_t rows = neighborhood.fluid.size();
    result.slot = op.slot_offsets == neighborhood.flat_offsets
        && op.slot_offsets.size() == rows + 1U
        && !op.slot_offsets.empty() && op.slot_offsets.front() == 0U
        && op.slot_offsets.back() == op.slots.size();
    std::vector<std::size_t> degrees;
    degrees.reserve(rows);
    if (result.slot) {
        for (std::size_t row = 0U; row < rows && result.slot; ++row) {
            const std::size_t begin = op.slot_offsets[row];
            const std::size_t end = op.slot_offsets[row + 1U];
            result.slot = begin <= end && end <= op.slots.size();
            degrees.push_back(end - begin);
            for (std::size_t index = begin; index < end && result.slot; ++index) {
                const ALR64Slot& slot = op.slots[index];
                const std::size_t flat =
                    neighborhood.flat_directed_pair_indices[index];
                result.slot = slot.pair == flat
                    && slot.pair < neighborhood.pairs.size()
                    && slot.participant == joint_pair_participant(neighborhood,
                        neighborhood.pairs[slot.pair], row)
                    && al_r64_finite(slot.jacobian);
            }
        }
    }
    if (!degrees.empty()) {
        result.minimum_degree = *std::min_element(degrees.begin(), degrees.end());
        result.maximum_degree = *std::max_element(degrees.begin(), degrees.end());
        result.below_maximum = static_cast<std::size_t>(std::count_if(
            degrees.begin(), degrees.end(), [&](std::size_t value) {
                return value < result.maximum_degree;
            }));
        std::sort(degrees.begin(), degrees.end());
        result.distinct_degrees = static_cast<std::size_t>(
            std::unique(degrees.begin(), degrees.end()) - degrees.begin());
    }

    result.entry = op.entry_offsets.size() == rows + 1U
        && !op.entry_offsets.empty() && op.entry_offsets.front() == 0U
        && op.entry_offsets.back() == op.entries.size();
    for (std::size_t row = 0U; row < rows && result.entry; ++row) {
        const std::size_t begin = op.entry_offsets[row];
        const std::size_t end = op.entry_offsets[row + 1U];
        result.entry = begin < end && begin <= end && end <= op.entries.size();
        std::size_t previous = rows;
        for (std::size_t index = begin; index < end && result.entry; ++index) {
            const ALR64Entry& entry = op.entries[index];
            result.entry = entry.particle < rows && al_r64_finite(entry.coefficient)
                && (index == begin || previous < entry.particle);
            previous = entry.particle;
        }
    }

    result.incidence = op.particle_offsets.size() == rows + 1U
        && !op.particle_offsets.empty() && op.particle_offsets.front() == 0U
        && op.particle_offsets.back() == op.incidences.size()
        && op.incidences.size() == op.entries.size();
    std::vector<std::uint8_t> seen(op.entries.size(), 0U);
    for (std::size_t particle = 0U;
         particle < rows && result.incidence; ++particle) {
        const std::size_t begin = op.particle_offsets[particle];
        const std::size_t end = op.particle_offsets[particle + 1U];
        result.incidence = begin <= end && end <= op.incidences.size();
        std::size_t previous_row = 0U;
        for (std::size_t index = begin;
             index < end && result.incidence; ++index) {
            const ALR64Incidence incidence = op.incidences[index];
            result.incidence = incidence.row < rows
                && incidence.entry < op.entries.size()
                && op.entries[incidence.entry].particle == particle
                && incidence.entry >= op.entry_offsets[incidence.row]
                && incidence.entry < op.entry_offsets[incidence.row + 1U]
                && seen[incidence.entry]++ == 0U
                && (index == begin || previous_row < incidence.row);
            previous_row = incidence.row;
        }
    }
    result.incidence = result.incidence
        && std::all_of(seen.begin(), seen.end(),
            [](std::uint8_t value) { return value == 1U; });
    return result;
}

ALLinearizedJvp al_r64_action(const ALR64Operator& op,
    const std::vector<Vec3>& direction) {
    ALLinearizedJvp result;
    const std::size_t rows = direction.size();
    if (op.slot_offsets.size() != rows + 1U) return result;
    result.value.assign(rows, 0.0);
    result.absolute_sum.assign(rows, 0.0);
    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t index = op.slot_offsets[row];
             index < op.slot_offsets[row + 1U]; ++index) {
            const ALR64Slot& slot = op.slots[index];
            const Vec3 relative = slot.participant < rows
                ? direction[row] - direction[slot.participant] : direction[row];
            const double term = SPACING * dot(slot.jacobian, relative);
            result.value[row] += term;
            result.absolute_sum[row] += std::abs(term);
        }
    result.passed = al_r30_finite(result.value)
        && al_r30_finite(result.absolute_sum);
    return result;
}

std::vector<Vec3> al_r64_transpose(const ALR64Operator& op,
    const std::vector<double>& weights, std::vector<Vec3>* absolute) {
    const std::size_t rows = weights.size();
    std::vector<Vec3> result(rows);
    if (absolute != nullptr) absolute->assign(rows, Vec3{});
    if (op.entry_offsets.size() != rows + 1U) return {};
    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t index = op.entry_offsets[row];
             index < op.entry_offsets[row + 1U]; ++index) {
            const ALR64Entry& entry = op.entries[index];
            const Vec3 term = weights[row] * entry.coefficient;
            result[entry.particle] += term;
            if (absolute != nullptr)
                al_r64_accumulate_absolute(term, &(*absolute)[entry.particle]);
        }
    return result;
}

bool al_r64_epoch_transition(
    std::uint32_t previous, std::uint32_t candidate) {
    return previous != std::numeric_limits<std::uint32_t>::max()
        && candidate != 0U && candidate == previous + 1U;
}

void al_r64_observe(ALR64Comparison* result, double reference,
    double candidate, double bound) {
    const double error = std::abs(reference - candidate);
    const bool exact = binary64_bits(reference) == binary64_bits(candidate);
    ++result->values;
    if (exact) ++result->exact_values;
    result->bits_exact = result->bits_exact && exact;
    result->maximum_error = std::max(result->maximum_error, error);
    result->maximum_bound = std::max(result->maximum_bound, bound);
    if (bound > 0.0)
        result->maximum_ratio = std::max(result->maximum_ratio, error / bound);
    result->passed = result->passed && std::isfinite(reference)
        && std::isfinite(candidate) && std::isfinite(error)
        && std::isfinite(bound) && error <= bound;
}

std::string al_r64_route(bool parent, bool source, bool capture,
    bool workspace, bool topology, bool slot, bool entry, bool incidence,
    bool action, bool transpose, bool gradient, bool diagonal, bool overlap,
    bool bound, bool controls, bool work, bool exact, bool bounded) {
    if (!parent) return "SPARSE_ROW_PARENT_REJECTED";
    if (!source) return "SPARSE_ROW_SOURCE_REJECTED";
    if (!capture) return "SPARSE_ROW_CAPTURE_REJECTED";
    if (!workspace) return "SPARSE_ROW_WORKSPACE_REJECTED";
    if (!topology) return "SPARSE_ROW_TOPOLOGY_REJECTED";
    if (!slot) return "SPARSE_ROW_SLOT_REJECTED";
    if (!entry) return "SPARSE_ROW_ENTRY_REJECTED";
    if (!incidence) return "SPARSE_ROW_INCIDENCE_REJECTED";
    if (!action) return "SPARSE_ROW_ACTION_REJECTED";
    if (!transpose) return "SPARSE_ROW_TRANSPOSE_REJECTED";
    if (!gradient) return "SPARSE_ROW_GRADIENT_REJECTED";
    if (!diagonal) return "SPARSE_ROW_DIAGONAL_REJECTED";
    if (!overlap) return "SPARSE_ROW_OVERLAP_REJECTED";
    if (!bound) return "SPARSE_ROW_BOUND_REJECTED";
    if (!controls) return "SPARSE_ROW_CONTROLS_REJECTED";
    if (!work) return "SPARSE_ROW_WORK_REJECTED";
    if (exact) return "SPARSE_ROW_OPERATOR_EXACT_EQUIVALENCE_CANDIDATE";
    if (bounded) return "SPARSE_ROW_OPERATOR_BOUNDED_EQUIVALENCE_CANDIDATE";
    return "SPARSE_ROW_OPERATOR_ARITHMETIC_ALIGNMENT_REQUIRED";
}

} // namespace

SplitBoundaryReport run_al_sparse_row_operator_controls() {
    const bool identity = sha256_hex(B4E2D7R19R64_IDENTITY_PROJECTION)
        == B4E2D7R19R64_IDENTITY_SHA256;
    ALR63ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_cached_normal_projection_trqp_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62"
        && parent.json.find(
            "9f456232956a0e080063123acf8a4a0b2dac7a6fd8f2615b270f0958e66a8440")
            != std::string::npos
        && parent.json.find("TANGENTIAL_MASTER_EXPANSION_REQUIRED")
            != std::string::npos;

    ALR62ReplayCapture& r62 = capture.r62;
    ALR61ReplayCapture& r61 = r62.r61;
    ALR58ReplayCapture& r58_capture = r61.r60.r59.r58;
    ALR57ReplayCapture& r57_capture = r58_capture.r57;
    ALR56ReplayCapture& r56_capture = r57_capture.r56;
    ALR55ReplayCapture& r55_capture = r56_capture.r55;
    ALR54ReplayCapture& r54_capture = r55_capture.r54;
    ALR53ReplayCapture& r53_capture = r54_capture.r53;
    ALR51ReplayCapture& r51_capture = r53_capture.r52.r51;
    ALR51MasterResult& master = r51_capture.master;
    ALR43ReplayCapture& r43 =
        r51_capture.r50.r49.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;

    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string target_before = al_binary64_vec3_root(capture.target);
    const std::string state_before = capture.result.state_root;
    const std::string master_before = master.union_root;
    const std::string cache_before = al_r51_cache_root(master.cache);
    const bool source_exact = parent_exact
        && state_before
            == "216c12aeeb47686e823a9911f0aa9dd14ddee53db1701874488dc325a292cea5"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && master_before
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28"
        && cache_before
            == "f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5";
    const bool capture_exact = source_exact && master.rows.size() == 494U
        && master.cache.size() == 494U && capture.target.size() == 6000U
        && capture.result.records.size() == 4U;

    ALSparseWorkTrace workspace_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    ALNormalizedWorkspace workspace = build_al_normalized_sparse_workspace(
        r43.projected_trial, binding, r30.dual, r30.theta,
        &workspace_work, &static_work, &adjacency_work);
    const bool workspace_exact = capture_exact && index.passed
        && binding.passed && workspace.passed
        && workspace.neighborhood.fluid.size() == 6000U;
    const std::string topology_root =
        al_r42_superset_root(workspace.neighborhood);
    const std::string operator_topology_root = al_r29_topology_root(workspace);
    const bool topology_exact = workspace_exact
        && topology_root
            == "cfcc7ebbeac9ce8d54093d3af109367e5b68b4aceadf11ed516dbfddf5c2d73f"
        && topology_root == r62.payload.workspace_topology_root
        && !operator_topology_root.empty();

    ALR64Operator op;
    if (workspace.passed) op = al_r64_build(workspace);
    const ALR64StructureAudit structure = al_r64_structure(workspace, op);
    const bool degree_exact = structure.minimum_degree == 44U
        && structure.maximum_degree == 113U
        && structure.distinct_degrees == 53U
        && structure.below_maximum == 5992U
        && structure.maximum_degree == workspace.neighborhood.maximum_degree;
    const bool slot_exact = structure.slot && degree_exact;
    const bool entry_exact = structure.entry;
    const bool incidence_exact = structure.incidence;

    const std::vector<Vec3> fluid_probe = al_r29_fluid_probe(6000U);
    const ALLinearizedJvp sparse_action = al_r64_action(op, fluid_probe);
    const ALLinearizedJvp reference_action =
        al_r29_directed_jvp(workspace, fluid_probe);
    bool action_exact = sparse_action.passed && reference_action.passed
        && sparse_action.value.size() == reference_action.value.size();
    std::size_t action_exact_values = 0U;
    for (std::size_t row = 0U;
         row < reference_action.value.size() && action_exact; ++row) {
        const bool value_exact = binary64_bits(sparse_action.value[row])
            == binary64_bits(reference_action.value[row]);
        const bool absolute_exact =
            binary64_bits(sparse_action.absolute_sum[row])
            == binary64_bits(reference_action.absolute_sum[row]);
        action_exact = value_exact && absolute_exact;
        action_exact_values += static_cast<std::size_t>(value_exact)
            + static_cast<std::size_t>(absolute_exact);
    }

    const std::vector<double> row_probe = al_r29_row_probe(6000U);
    std::vector<Vec3> sparse_transpose_absolute;
    const std::vector<Vec3> sparse_transpose =
        al_r64_transpose(op, row_probe, &sparse_transpose_absolute);
    const std::vector<Vec3> reference_transpose =
        al_r29_pair_once_vjp(workspace, row_probe);
    std::vector<Vec3> reference_transpose_absolute(6000U);
    for (std::size_t pair_index = 0U;
         pair_index < workspace.neighborhood.pairs.size(); ++pair_index) {
        const JointPair pair = workspace.neighborhood.pairs[pair_index];
        const Vec3 jacobian = al_r29_pair_jacobian(workspace, pair_index);
        const double weight = pair.participant < 6000U
            ? row_probe[pair.fluid] + row_probe[pair.participant]
            : row_probe[pair.fluid];
        const Vec3 term = SPACING * weight * jacobian;
        al_r64_accumulate_absolute(term,
            &reference_transpose_absolute[pair.fluid]);
        if (pair.participant < 6000U)
            al_r64_accumulate_absolute(-term,
                &reference_transpose_absolute[pair.participant]);
    }
    ALR64Comparison transpose_comparison;
    transpose_comparison.passed = sparse_transpose.size() == 6000U
        && reference_transpose.size() == 6000U;
    for (std::size_t particle = 0U; particle < 6000U; ++particle) {
        const std::size_t degree = op.particle_offsets[particle + 1U]
            - op.particle_offsets[particle];
        const double gamma = gamma_factor(8U * degree + 64U);
        for (int axis = 0; axis < 3; ++axis) {
            const double reference =
                al_r64_component(reference_transpose[particle], axis);
            const double sparse =
                al_r64_component(sparse_transpose[particle], axis);
            const double bound = gamma * (
                al_r64_component(reference_transpose_absolute[particle], axis)
                + al_r64_component(sparse_transpose_absolute[particle], axis));
            al_r64_observe(&transpose_comparison, reference, sparse, bound);
        }
    }

    ALR64Comparison gradient_comparison;
    gradient_comparison.passed = true;
    ALR64Comparison diagonal_comparison;
    diagonal_comparison.passed = true;
    for (const ALR50RowBasis& basis : master.cache) {
        const std::size_t row = basis.row;
        std::vector<Vec3> reconstructed(6000U);
        for (std::size_t index_entry = op.entry_offsets[row];
             index_entry < op.entry_offsets[row + 1U]; ++index_entry) {
            const ALR64Entry& entry_value = op.entries[index_entry];
            reconstructed[entry_value.particle] = entry_value.coefficient;
        }
        const std::size_t degree = op.slot_offsets[row + 1U]
            - op.slot_offsets[row];
        const double gamma = gamma_factor(8U * degree + 64U);
        for (std::size_t particle = 0U; particle < 6000U; ++particle)
            for (int axis = 0; axis < 3; ++axis) {
                const double reference =
                    al_r64_component(basis.gradient[particle], axis);
                const double sparse = al_r64_component(
                    reconstructed[particle], axis);
                const double entry_absolute = std::abs(sparse);
                const double bound = gamma * (std::abs(reference)
                    + std::abs(sparse) + entry_absolute);
                al_r64_observe(
                    &gradient_comparison, reference, sparse, bound);
            }
        double direct = 0.0;
        long double long_direct = 0.0L;
        for (std::size_t index_entry = op.entry_offsets[row];
             index_entry < op.entry_offsets[row + 1U]; ++index_entry) {
            const Vec3 coefficient = op.entries[index_entry].coefficient;
            direct += norm_squared(coefficient);
            long_direct += static_cast<long double>(coefficient.x)
                    * static_cast<long double>(coefficient.x)
                + static_cast<long double>(coefficient.y)
                    * static_cast<long double>(coefficient.y)
                + static_cast<long double>(coefficient.z)
                    * static_cast<long double>(coefficient.z);
        }
        const double scale = std::max({std::abs(basis.diagonal),
            std::abs(direct), std::abs(static_cast<double>(long_direct)),
            1.0e-300});
        const double error = std::max(std::abs(basis.diagonal - direct),
            std::abs(basis.diagonal - static_cast<double>(long_direct)));
        al_r64_observe(&diagonal_comparison, basis.diagonal, direct,
            1.0e-12 * scale);
        diagonal_comparison.passed = diagonal_comparison.passed
            && std::isfinite(error) && error <= 1.0e-12 * scale;
        diagonal_comparison.bits_exact = diagonal_comparison.bits_exact
            && binary64_bits(basis.diagonal)
                == binary64_bits(static_cast<double>(long_direct));
    }

    ALR64Comparison overlap_comparison;
    overlap_comparison.passed = true;
    std::vector<double> scratch(6000U, 0.0);
    std::vector<double> absolute_scratch(6000U, 0.0);
    std::vector<std::uint32_t> tags(6000U, 0U);
    std::vector<std::size_t> touched;
    std::uint32_t epoch = 0U;
    bool epoch_exact = true;
    for (const ALR50RowBasis& basis : master.cache) {
        const std::uint32_t previous_epoch = epoch;
        ++epoch;
        epoch_exact = epoch_exact
            && al_r64_epoch_transition(previous_epoch, epoch);
        touched.clear();
        const std::size_t source = basis.row;
        for (std::size_t source_entry = op.entry_offsets[source];
             source_entry < op.entry_offsets[source + 1U]; ++source_entry) {
            const ALR64Entry& lhs = op.entries[source_entry];
            for (std::size_t incidence_index = op.particle_offsets[lhs.particle];
                 incidence_index < op.particle_offsets[lhs.particle + 1U];
                 ++incidence_index) {
                const ALR64Incidence incidence =
                    op.incidences[incidence_index];
                const ALR64Entry& rhs = op.entries[incidence.entry];
                if (tags[incidence.row] != epoch) {
                    tags[incidence.row] = epoch;
                    scratch[incidence.row] = 0.0;
                    absolute_scratch[incidence.row] = 0.0;
                    touched.push_back(incidence.row);
                }
                const double term = dot(lhs.coefficient, rhs.coefficient);
                scratch[incidence.row] += term;
                absolute_scratch[incidence.row] += std::abs(term);
            }
        }
        const std::size_t source_degree = op.slot_offsets[source + 1U]
            - op.slot_offsets[source];
        for (std::size_t target = 0U; target < 6000U; ++target) {
            const double sparse = tags[target] == epoch ? scratch[target] : 0.0;
            const double absolute = tags[target] == epoch
                ? absolute_scratch[target] : 0.0;
            const std::size_t target_degree = op.slot_offsets[target + 1U]
                - op.slot_offsets[target];
            const double gamma = gamma_factor(
                32U * (source_degree + target_degree) + 128U);
            const double reference = basis.gram_column[target];
            const double bound = gamma
                * (std::abs(reference) + std::abs(sparse) + absolute);
            al_r64_observe(
                &overlap_comparison, reference, sparse, bound);
        }
    }

    bool mutation_offset_rejected = false;
    if (op.slot_offsets.size() > 1U) {
        ALR64Operator corrupted = op;
        ++corrupted.slot_offsets[1U];
        mutation_offset_rejected =
            !al_r64_structure(workspace, corrupted).slot;
    }
    bool mutation_entry_rejected = false;
    if (!op.entries.empty()) {
        ALR64Operator corrupted = op;
        corrupted.entries.front().particle = 6000U;
        mutation_entry_rejected =
            !al_r64_structure(workspace, corrupted).entry;
    }
    bool mutation_incidence_rejected = false;
    if (!op.incidences.empty()) {
        ALR64Operator corrupted = op;
        corrupted.incidences.front().entry = corrupted.entries.size();
        mutation_incidence_rejected =
            !al_r64_structure(workspace, corrupted).incidence;
    }
    const bool mutation_epoch_rejected =
        !al_r64_epoch_transition(epoch, epoch);
    const bool controls_exact = mutation_offset_rejected
        && mutation_entry_rejected && mutation_incidence_rejected
        && mutation_epoch_rejected && epoch_exact;

    const bool gradient_exact = gradient_comparison.passed;
    const bool diagonal_exact = diagonal_comparison.passed;
    const bool overlap_exact = overlap_comparison.passed && epoch_exact;
    const bool transpose_exact = transpose_comparison.passed;
    const bool bounds_exact = std::isfinite(transpose_comparison.maximum_bound)
        && std::isfinite(gradient_comparison.maximum_bound)
        && std::isfinite(diagonal_comparison.maximum_bound)
        && std::isfinite(overlap_comparison.maximum_bound);
    const bool bits_exact = action_exact && transpose_comparison.bits_exact
        && gradient_comparison.bits_exact && diagonal_comparison.bits_exact
        && overlap_comparison.bits_exact;
    const bool bounded_exact = action_exact && transpose_exact
        && gradient_exact && diagonal_exact && overlap_exact && bounds_exact;

    if (workspace.passed)
        release_al_normalized_workspace(workspace, &workspace_work);
    const bool work_exact = workspace_work.workspace_builds == 1U
        && workspace_work.workspace_releases == 1U
        && workspace_work.live_workspaces == 0U
        && workspace_work.maximum_live_workspaces == 1U
        && !workspace_work.lifecycle_underflow
        && workspace_work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 1U
        && adjacency_work.workspace_builds == 1U
        && master.cache.size() == 494U
        && overlap_comparison.values == 2964000U
        && gradient_comparison.values == 8892000U
        && diagonal_comparison.values == 494U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && target_before == al_binary64_vec3_root(capture.target)
        && state_before == capture.result.state_root
        && master_before == master.union_root
        && cache_before == al_r51_cache_root(master.cache);

    const std::array<const char*, 19U> expected_routes{{
        "SPARSE_ROW_PARENT_REJECTED", "SPARSE_ROW_SOURCE_REJECTED",
        "SPARSE_ROW_CAPTURE_REJECTED", "SPARSE_ROW_WORKSPACE_REJECTED",
        "SPARSE_ROW_TOPOLOGY_REJECTED", "SPARSE_ROW_SLOT_REJECTED",
        "SPARSE_ROW_ENTRY_REJECTED", "SPARSE_ROW_INCIDENCE_REJECTED",
        "SPARSE_ROW_ACTION_REJECTED", "SPARSE_ROW_TRANSPOSE_REJECTED",
        "SPARSE_ROW_GRADIENT_REJECTED", "SPARSE_ROW_DIAGONAL_REJECTED",
        "SPARSE_ROW_OVERLAP_REJECTED", "SPARSE_ROW_BOUND_REJECTED",
        "SPARSE_ROW_CONTROLS_REJECTED", "SPARSE_ROW_WORK_REJECTED",
        "SPARSE_ROW_OPERATOR_EXACT_EQUIVALENCE_CANDIDATE",
        "SPARSE_ROW_OPERATOR_BOUNDED_EQUIVALENCE_CANDIDATE",
        "SPARSE_ROW_OPERATOR_ARITHMETIC_ALIGNMENT_REQUIRED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 16U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const bool exact_case = route_index == 16U;
        const bool bounded_case = route_index == 16U || route_index == 17U;
        const std::string observed = al_r64_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            gates[12], gates[13], gates[14], gates[15], exact_case,
            bounded_case);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r64_route(parent_exact, source_exact,
        capture_exact, workspace_exact, topology_exact, slot_exact,
        entry_exact, incidence_exact, action_exact, transpose_exact,
        gradient_exact, diagonal_exact, overlap_exact, bounds_exact,
        controls_exact, work_exact && rollback, bits_exact, bounded_exact);
    const bool passed = identity && parent_exact && source_exact
        && capture_exact && workspace_exact && topology_exact && slot_exact
        && entry_exact && incidence_exact && action_exact && transpose_exact
        && gradient_exact && diagonal_exact && overlap_exact && bounds_exact
        && controls_exact && work_exact && rollback && routes_exact
        && bounded_exact;

    std::ostringstream operator_projection;
    operator_projection << op.slot_offsets.size() << ':' << op.slots.size()
        << ':' << op.entry_offsets.size() << ':' << op.entries.size() << ':'
        << op.particle_offsets.size() << ':' << op.incidences.size() << '|';
    for (const ALR64Entry& entry_value : op.entries)
        operator_projection << entry_value.particle << ':'
            << binary64_bits(entry_value.coefficient.x) << ':'
            << binary64_bits(entry_value.coefficient.y) << ':'
            << binary64_bits(entry_value.coefficient.z) << ';';
    const std::string operator_root = sha256_hex(operator_projection.str());
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
        << B4E2D7R19R64_IDENTITY_SHA256 << '|' << parent_stdout << '|'
        << topology_root << ':' << operator_topology_root << ':'
        << operator_root << '|'
        << structure.minimum_degree << ':' << structure.maximum_degree << ':'
        << structure.distinct_degrees << ':' << structure.below_maximum << '|'
        << action_exact_values << ':' << transpose_comparison.exact_values << ':'
        << gradient_comparison.exact_values << ':'
        << diagonal_comparison.exact_values << ':'
        << overlap_comparison.exact_values << '|'
        << transpose_comparison.maximum_ratio << ':'
        << gradient_comparison.maximum_ratio << ':'
        << overlap_comparison.maximum_ratio << '|'
        << controls_exact << ':' << work_exact << ':' << rollback << '|'
        << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r64_sparse_row_operator.v1\",\"status\":\""
        << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
        << B4E2D7R19R64_IDENTITY_SHA256
        << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
        << ",\"source\":{\"state_root\":\"" << state_before
        << "\",\"topology_root\":\"" << topology_root
        << "\",\"operator_topology_root\":\"" << operator_topology_root
        << "\",\"operator_root\":\"" << operator_root << "\"}"
        << ",\"structure\":{\"rows\":6000,\"slots\":" << op.slots.size()
        << ",\"entries\":" << op.entries.size()
        << ",\"incidences\":" << op.incidences.size()
        << ",\"minimum_degree\":" << structure.minimum_degree
        << ",\"maximum_degree\":" << structure.maximum_degree
        << ",\"distinct_degrees\":" << structure.distinct_degrees
        << ",\"below_maximum\":" << structure.below_maximum << "}"
        << ",\"action\":{\"exact_values\":" << action_exact_values
        << ",\"values\":12000,\"bits_exact\":"
        << (action_exact ? "true" : "false") << "}"
        << ",\"transpose\":{\"exact_values\":"
        << transpose_comparison.exact_values << ",\"values\":"
        << transpose_comparison.values << ",\"maximum_error\":"
        << transpose_comparison.maximum_error << ",\"maximum_bound\":"
        << transpose_comparison.maximum_bound << ",\"maximum_ratio\":"
        << transpose_comparison.maximum_ratio << "}"
        << ",\"gradient\":{\"checks\":494,\"exact_values\":"
        << gradient_comparison.exact_values << ",\"values\":"
        << gradient_comparison.values << ",\"maximum_ratio\":"
        << gradient_comparison.maximum_ratio << "}"
        << ",\"diagonal\":{\"checks\":494,\"exact_values\":"
        << diagonal_comparison.exact_values << ",\"maximum_ratio\":"
        << diagonal_comparison.maximum_ratio << "}"
        << ",\"overlap\":{\"columns\":494,\"values\":"
        << overlap_comparison.values << ",\"exact_values\":"
        << overlap_comparison.exact_values << ",\"maximum_error\":"
        << overlap_comparison.maximum_error << ",\"maximum_bound\":"
        << overlap_comparison.maximum_bound << ",\"maximum_ratio\":"
        << overlap_comparison.maximum_ratio << "}"
        << ",\"controls\":{\"offset_rejected\":"
        << (mutation_offset_rejected ? "true" : "false")
        << ",\"entry_rejected\":"
        << (mutation_entry_rejected ? "true" : "false")
        << ",\"incidence_rejected\":"
        << (mutation_incidence_rejected ? "true" : "false")
        << ",\"epoch_reuse_rejected\":"
        << (mutation_epoch_rejected ? "true" : "false") << "}"
        << ",\"work\":{\"parent_replays\":1,\"workspace_builds\":1"
           ",\"workspace_releases\":1,\"sparse_row_builds\":6000"
           ",\"slot_scans\":1,\"entry_builds\":6000"
           ",\"incidence_builds\":1,\"fresh_directed_jvp\":1"
           ",\"fresh_pair_vjp\":1,\"captured_gradient_checks\":494"
           ",\"captured_diagonal_checks\":494,\"overlap_columns\":494"
           ",\"column_values\":2964000,\"dense_gram_storage\":0"
           ",\"new_row_vjp\":0,\"new_gram_jvp\":0,\"new_hildreth\":0"
           ",\"new_projection\":0,\"new_nonlinear_trial\":0"
           ",\"new_hvp\":0,\"new_outer\":0,\"exact\":"
        << (work_exact ? "true" : "false") << "}"
        << ",\"classification\":{\"bits_exact\":"
        << (bits_exact ? "true" : "false") << ",\"bounded_exact\":"
        << (bounded_exact ? "true" : "false") << "}"
        << ",\"route_cases\":" << expected_routes.size()
        << ",\"route_root\":\"" << route_root
        << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
        << ",\"rollback_exact\":" << (rollback ? "true" : "false")
        << ",\"route\":\"" << route
        << "\",\"dynamic_active_set_executed\":false"
           ",\"runtime_state_mutated\":false,\"normal_applied\":false"
           ",\"tangential_applied\":false,\"filter_executed\":false"
           ",\"switching_executed\":false,\"following_outer_executed\":false"
           ",\"timing_admitted\":false,\"runtime_authority\":false"
           ",\"production_authority\":false,\"result_sha256\":\""
        << result_sha << "\"}";
    return {passed, report.str()};
}
