namespace {

constexpr const char* B4E2D7R19R54_IDENTITY_SHA256 =
    "14a4a9436b9e02babcadc9869c80749d77b1bf02fa5f02f1aefe52c091da91ca";
constexpr const char* B4E2D7R19R54_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r54-model-projection-decomposition|v1|parent=af354acc:490669f7533c6b160a8426b029ee458cf80dc97439872f9b16bbf077a94b531a:4a778c2e546636c28f38d71a72f86e434c089fbc0a441b2babab1e519d86e80c:HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED|source=r52-record64-2e0afdb4dbdac10cef5ba591b5764f7cce40ca414458e7f5b9c61cdbc83425f0;witness-ccc1084988fb56a829649f60c292210f24723a4932874923640d5bdbf99e8729;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5;pair-directed-f19cc1913ac2cf0061c9f6240f0f6378fc513108c77975e02cbd0f1f45f632ca|pipeline=recursive-master-predicted;direct-gram-binary64;compensated-gram-binary128;fresh-correction-pair-jvp;anchor-raw-plus-correction;fresh-unprojected-target-pair-jvp;captured-projected-directed;stable-topology|geometry=target-root;projected-root;target-norm;projected-norm;displacement-norm;maximum-displacement;changed-components;box-violations;ball-outside;analytic-clamp-check|rows=positive-count;maximum-and-row;worst-projected-row-value-per-stage;stage-difference-maximum-and-row;roots|selection=largest-exact-stage-gap;recursive-direct-first;direct-correction-second;anchor-target-third;target-projected-fourth;else-model-projection-consistent|controls=r53-parent-bytes;source;workspace;topology;master;lambda;gram;correction;target;geometry;row-comparison;work;rollback;route-precedence|routes=model-parent-rejected;model-source-rejected;model-workspace-rejected;model-topology-rejected;model-master-rejected;model-lambda-rejected;model-gram-rejected;model-correction-rejected;model-target-rejected;model-geometry-rejected;model-row-comparison-rejected;model-work-rejected;hildreth-explicit-residual-replacement-required;gram-correction-assembly-required;anchor-addition-reevaluation-required;ball-box-projection-model-required;model-projection-consistent-candidate|precedence=parent,source,workspace,topology,master,lambda,gram,correction,target,geometry,row-comparison,work,largest-gap-stable-order|work=parent-r53-replays1;moved-workspace1;workspace-release1;fresh-correction-pair-jvp1;fresh-target-pair-jvp1;pairpasses2;binary128-gram-fold1;geometry-fold1;new-row-vjp0;new-gram-jvp0;new-audit-jvp0;new-sweeps0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53=unchanged;certificate=current-directed-unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;residual-replacement=none;correction-change=none;projection-change=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-model-projection-classification-only)IDENTITY";

struct ALR54Summary {
    std::size_t positive = 0U;
    double maximum = -std::numeric_limits<double>::infinity();
    std::size_t row = 0U;
    std::string root;
};

struct ALR54QuadSummary {
    std::size_t positive = 0U;
    Binary128 maximum = -HUGE_VALQ;
    std::size_t row = 0U;
    std::string root;
};

struct ALR54Gap {
    Binary128 maximum = static_cast<Binary128>(0.0);
    std::size_t row = 0U;
};

struct ALR54Result {
    bool lambda_exact = false;
    bool gram_exact = false;
    bool correction_exact = false;
    bool target_exact = false;
    bool geometry_exact = false;
    bool comparison_exact = false;
    std::size_t pair_passes = 0U;
    std::size_t binary128_gram_folds = 0U;
    std::size_t geometry_folds = 0U;
    ALR54Summary recursive;
    ALR54Summary direct64;
    ALR54QuadSummary direct128;
    ALR54Summary correction_upper;
    ALR54Summary anchor_linearity;
    ALR54Summary target_raw;
    ALR54Summary projected_raw;
    ALR54Gap recursive_direct;
    ALR54Gap direct64_direct128;
    ALR54Gap direct_correction;
    ALR54Gap anchor_target;
    ALR54Gap target_projected;
    std::size_t winning_stage = 4U;
    std::size_t final_worst_row = 0U;
    double worst_recursive = 0.0;
    double worst_direct64 = 0.0;
    Binary128 worst_direct128 = static_cast<Binary128>(0.0);
    double worst_correction_upper = 0.0;
    double worst_anchor_linearity = 0.0;
    double worst_target_raw = 0.0;
    double worst_projected_raw = 0.0;
    double worst_anchor_bound = 0.0;
    double target_norm = 0.0;
    double clamped_norm = 0.0;
    double projected_norm = 0.0;
    double displacement_norm = 0.0;
    double maximum_displacement = 0.0;
    std::size_t maximum_displacement_component = 0U;
    std::size_t changed_components = 0U;
    std::size_t box_violations = 0U;
    double maximum_box_violation = 0.0;
    bool ball_outside = false;
    bool analytic_clamp_exact = false;
    std::string target_root;
    std::string projected_root;
    std::string displacement_root;
    std::string comparison_root;
};

ALR54Summary al_r54_summary(
    const std::vector<double>& value,
    const std::vector<std::size_t>* rows = nullptr) {
    ALR54Summary result;
    for (std::size_t i = 0U; i < value.size(); ++i) {
        const std::size_t row = rows == nullptr ? i : (*rows)[i];
        if (value[i] > 0.0) ++result.positive;
        if (value[i] > result.maximum) {
            result.maximum = value[i];
            result.row = row;
        }
    }
    result.root = al_r10_dual_root(value);
    return result;
}

ALR54QuadSummary al_r54_quad_summary(
    const std::vector<Binary128>& value,
    const std::vector<std::size_t>& rows) {
    ALR54QuadSummary result;
    std::ostringstream material;
    for (std::size_t i = 0U; i < value.size(); ++i) {
        if (value[i] > static_cast<Binary128>(0.0)) ++result.positive;
        if (value[i] > result.maximum) {
            result.maximum = value[i];
            result.row = rows[i];
        }
        material << al_binary128_hex(value[i]) << ':';
    }
    result.root = sha256_hex(material.str());
    return result;
}

void al_r54_update_gap(ALR54Gap* gap, Binary128 value, std::size_t row) {
    if (gap != nullptr && value > gap->maximum) {
        gap->maximum = value;
        gap->row = row;
    }
}

ALR54Result al_r54_decompose(const ALNormalizedWorkspace& workspace,
    const ALR50Generator& r50, const ALR51MasterResult& r51,
    const ALR52Record& record, const std::vector<Vec3>& lower,
    const std::vector<Vec3>& upper) {
    ALR54Result result;
    const std::size_t master_count = r51.rows.size();
    const std::size_t row_count = workspace.support.constraint.size();
    const std::size_t particle_count = workspace.neighborhood.fluid.size();
    result.lambda_exact = workspace.passed && master_count == 494U
        && record.solve.core.lambda.size() == master_count
        && record.solve.core.predicted.size() == master_count
        && record.solve.correction.size() == particle_count
        && r50.witness.size() == particle_count
        && record.witness.size() == particle_count
        && std::all_of(record.solve.core.lambda.begin(),
            record.solve.core.lambda.end(), [](double value) {
                return std::isfinite(value) && value >= 0.0;
            });
    if (!result.lambda_exact) return result;

    std::vector<std::size_t> basis_index(master_count);
    result.gram_exact = r51.cache.size() == master_count;
    for (std::size_t i = 0U; i < master_count; ++i) {
        basis_index[i] = al_r50_find_basis(r51.cache, r51.rows[i]);
        result.gram_exact = result.gram_exact
            && basis_index[i] < r51.cache.size()
            && r51.cache[basis_index[i]].gram_column.size() == row_count;
    }
    if (!result.gram_exact) return result;

    std::vector<double> direct64(master_count);
    std::vector<Binary128> direct128(master_count);
    for (std::size_t j = 0U; j < master_count; ++j) {
        const std::size_t row = r51.rows[j];
        double value64 = r50.terminal.upper[row];
        Binary128Accumulator value128;
        value128.add(static_cast<Binary128>(r50.terminal.upper[row]));
        for (std::size_t i = 0U; i < master_count; ++i) {
            const double gram =
                r51.cache[basis_index[i]].gram_column[row];
            value64 -= record.solve.core.lambda[i] * gram;
            value128.add(-static_cast<Binary128>(record.solve.core.lambda[i])
                * static_cast<Binary128>(gram));
        }
        direct64[j] = value64;
        direct128[j] = value128.value();
    }
    ++result.binary128_gram_folds;
    result.recursive = al_r54_summary(record.solve.core.predicted, &r51.rows);
    result.direct64 = al_r54_summary(direct64, &r51.rows);
    result.direct128 = al_r54_quad_summary(direct128, r51.rows);

    const ALLinearizedJvp correction_image =
        al_r29_pair_once_jvp(workspace, record.solve.correction);
    ++result.pair_passes;
    result.correction_exact = correction_image.passed
        && correction_image.value.size() == row_count;
    if (!result.correction_exact) return result;

    std::vector<Vec3> target(particle_count);
    for (std::size_t i = 0U; i < particle_count; ++i)
        target[i] = r50.witness[i] + record.solve.correction[i];
    const ALLinearizedJvp target_image = al_r29_pair_once_jvp(workspace, target);
    ++result.pair_passes;
    result.target_exact = target_image.passed
        && target_image.value.size() == row_count
        && r50.terminal.image.value.size() == row_count
        && record.audit.image.value.size() == row_count;
    if (!result.target_exact) return result;

    std::vector<double> correction_upper(master_count);
    for (std::size_t j = 0U; j < master_count; ++j) {
        const std::size_t row = r51.rows[j];
        correction_upper[j] = r50.terminal.upper[row]
            + correction_image.value[row];
    }
    result.correction_upper =
        al_r54_summary(correction_upper, &r51.rows);

    std::vector<double> anchor_linearity(row_count);
    std::vector<double> target_raw(row_count);
    std::vector<double> projected_raw(row_count);
    for (std::size_t row = 0U; row < row_count; ++row) {
        const double anchor_raw = workspace.support.constraint[row]
            + r50.terminal.image.value[row];
        anchor_linearity[row] = anchor_raw + correction_image.value[row];
        target_raw[row] = workspace.support.constraint[row]
            + target_image.value[row];
        projected_raw[row] = workspace.support.constraint[row]
            + record.audit.image.value[row];
    }
    result.anchor_linearity = al_r54_summary(anchor_linearity);
    result.target_raw = al_r54_summary(target_raw);
    result.projected_raw = al_r54_summary(projected_raw);

    for (std::size_t j = 0U; j < master_count; ++j) {
        const std::size_t row = r51.rows[j];
        al_r54_update_gap(&result.recursive_direct,
            fabsq(static_cast<Binary128>(record.solve.core.predicted[j])
                - direct128[j]), row);
        al_r54_update_gap(&result.direct64_direct128,
            fabsq(static_cast<Binary128>(direct64[j]) - direct128[j]), row);
        al_r54_update_gap(&result.direct_correction,
            fabsq(direct128[j]
                - static_cast<Binary128>(correction_upper[j])), row);
    }
    for (std::size_t row = 0U; row < row_count; ++row) {
        al_r54_update_gap(&result.anchor_target,
            fabsq(static_cast<Binary128>(anchor_linearity[row])
                - static_cast<Binary128>(target_raw[row])), row);
        al_r54_update_gap(&result.target_projected,
            fabsq(static_cast<Binary128>(target_raw[row])
                - static_cast<Binary128>(projected_raw[row])), row);
    }

    std::vector<Vec3> displacement(particle_count);
    std::vector<Vec3> clamped(particle_count);
    bool geometry_finite = lower.size() == particle_count
        && upper.size() == particle_count;
    for (std::size_t i = 0U; i < particle_count && geometry_finite; ++i) {
        displacement[i] = record.witness[i] - target[i];
        for (int axis = 0; axis < 3; ++axis) {
            const double value = al_r41_component(target[i], axis);
            const double low = al_r41_component(lower[i], axis);
            const double high = al_r41_component(upper[i], axis);
            const double clipped = std::max(low, std::min(value, high));
            al_r43_component(clamped[i], axis) = clipped;
            double violation = 0.0;
            if (value < low) violation = low - value;
            else if (value > high) violation = value - high;
            if (violation > 0.0) {
                ++result.box_violations;
                result.maximum_box_violation =
                    std::max(result.maximum_box_violation, violation);
            }
            const std::size_t component = 3U * i
                + static_cast<std::size_t>(axis);
            if (binary64_bits(value)
                != binary64_bits(al_r41_component(record.witness[i], axis)))
                ++result.changed_components;
            const double absolute =
                std::abs(al_r41_component(displacement[i], axis));
            if (absolute > result.maximum_displacement) {
                result.maximum_displacement = absolute;
                result.maximum_displacement_component = component;
            }
            geometry_finite = geometry_finite && std::isfinite(value)
                && std::isfinite(low) && std::isfinite(high)
                && std::isfinite(clipped) && low <= high;
        }
    }
    ++result.geometry_folds;
    result.target_norm = al_r32_vec3_norm(target);
    result.clamped_norm = al_r32_vec3_norm(clamped);
    result.projected_norm = al_r32_vec3_norm(record.witness);
    result.displacement_norm = al_r32_vec3_norm(displacement);
    result.ball_outside = result.clamped_norm > AL_R48_NORMAL_RADIUS;
    result.target_root = al_binary64_vec3_root(target);
    result.projected_root = al_binary64_vec3_root(record.witness);
    result.displacement_root = al_binary64_vec3_root(displacement);
    result.analytic_clamp_exact = !result.ball_outside
        && al_binary64_vec3_root(clamped) == result.projected_root;
    result.geometry_exact = geometry_finite
        && al_r48_vec_finite(target) && al_r48_vec_finite(displacement)
        && std::isfinite(result.target_norm)
        && std::isfinite(result.clamped_norm)
        && std::isfinite(result.projected_norm)
        && std::isfinite(result.displacement_norm)
        && result.analytic_clamp_exact;

    const std::array<Binary128, 4U> gaps{{
        result.recursive_direct.maximum,
        result.direct_correction.maximum,
        result.anchor_target.maximum,
        result.target_projected.maximum}};
    Binary128 largest = static_cast<Binary128>(0.0);
    result.winning_stage = 4U;
    for (std::size_t stage = 0U; stage < gaps.size(); ++stage)
        if (gaps[stage] > largest) {
            largest = gaps[stage];
            result.winning_stage = stage;
        }

    result.final_worst_row = result.projected_raw.row;
    const auto master_it = std::lower_bound(
        r51.rows.begin(), r51.rows.end(), result.final_worst_row);
    const bool worst_in_master = master_it != r51.rows.end()
        && *master_it == result.final_worst_row;
    if (worst_in_master) {
        const std::size_t j = static_cast<std::size_t>(
            std::distance(r51.rows.begin(), master_it));
        result.worst_recursive = record.solve.core.predicted[j];
        result.worst_direct64 = direct64[j];
        result.worst_direct128 = direct128[j];
        result.worst_correction_upper = correction_upper[j];
    }
    result.worst_anchor_linearity =
        anchor_linearity[result.final_worst_row];
    result.worst_target_raw = target_raw[result.final_worst_row];
    result.worst_projected_raw = projected_raw[result.final_worst_row];
    const double anchor_raw = workspace.support.constraint[result.final_worst_row]
        + r50.terminal.image.value[result.final_worst_row];
    result.worst_anchor_bound =
        r50.terminal.upper[result.final_worst_row] - anchor_raw;

    std::ostringstream comparison;
    comparison << result.recursive.root << ':' << result.direct64.root << ':'
               << result.direct128.root << ':'
               << result.correction_upper.root << ':'
               << result.anchor_linearity.root << ':'
               << result.target_raw.root << ':' << result.projected_raw.root
               << '|' << al_binary128_hex(result.recursive_direct.maximum)
               << ':' << result.recursive_direct.row << ':'
               << al_binary128_hex(result.direct64_direct128.maximum) << ':'
               << result.direct64_direct128.row << ':'
               << al_binary128_hex(result.direct_correction.maximum) << ':'
               << result.direct_correction.row << ':'
               << al_binary128_hex(result.anchor_target.maximum) << ':'
               << result.anchor_target.row << ':'
               << al_binary128_hex(result.target_projected.maximum) << ':'
               << result.target_projected.row << '|'
               << result.target_root << ':' << result.projected_root << ':'
               << result.displacement_root << ':' << result.winning_stage;
    result.comparison_root = sha256_hex(comparison.str());
    result.comparison_exact = result.geometry_exact && worst_in_master
        && result.recursive.positive <= master_count
        && result.direct64.positive <= master_count
        && result.direct128.positive <= master_count
        && result.correction_upper.positive <= master_count
        && result.anchor_linearity.positive <= row_count
        && result.target_raw.positive <= row_count
        && result.projected_raw.positive == 219U
        && binary64_bits(result.projected_raw.maximum)
            == binary64_bits(7.7539982072995717e-14)
        && result.projected_raw.row == 848U;
    return result;
}

bool al_r54_dense_controls(std::string* output_root) {
    Binary128Accumulator direct;
    direct.add(static_cast<Binary128>(1.0));
    direct.add(-static_cast<Binary128>(0.25));
    direct.add(-static_cast<Binary128>(0.5));
    std::vector<Vec3> target{{2.0, -2.0, 0.5}};
    std::vector<Vec3> low{{-1.0, -1.0, -1.0}};
    std::vector<Vec3> high{{1.0, 1.0, 1.0}};
    Vec3 clamped;
    for (int axis = 0; axis < 3; ++axis)
        al_r43_component(clamped, axis) = std::max(
            al_r41_component(low[0], axis),
            std::min(al_r41_component(target[0], axis),
                al_r41_component(high[0], axis)));
    const bool passed = direct.value() == static_cast<Binary128>(0.25)
        && clamped.x == 1.0 && clamped.y == -1.0 && clamped.z == 0.5;
    std::ostringstream material;
    material << passed << ':' << al_binary128_hex(direct.value()) << ':'
             << al_binary64_vec3_root({clamped});
    if (output_root != nullptr) *output_root = sha256_hex(material.str());
    return passed;
}

std::string al_r54_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool lambda, bool gram, bool correction,
    bool target, bool geometry, bool comparison, bool work,
    std::size_t winning_stage) {
    if (!parent) return "MODEL_PARENT_REJECTED";
    if (!source) return "MODEL_SOURCE_REJECTED";
    if (!workspace) return "MODEL_WORKSPACE_REJECTED";
    if (!topology) return "MODEL_TOPOLOGY_REJECTED";
    if (!master) return "MODEL_MASTER_REJECTED";
    if (!lambda) return "MODEL_LAMBDA_REJECTED";
    if (!gram) return "MODEL_GRAM_REJECTED";
    if (!correction) return "MODEL_CORRECTION_REJECTED";
    if (!target) return "MODEL_TARGET_REJECTED";
    if (!geometry) return "MODEL_GEOMETRY_REJECTED";
    if (!comparison) return "MODEL_ROW_COMPARISON_REJECTED";
    if (!work) return "MODEL_WORK_REJECTED";
    if (winning_stage == 0U)
        return "HILDRETH_EXPLICIT_RESIDUAL_REPLACEMENT_REQUIRED";
    if (winning_stage == 1U) return "GRAM_CORRECTION_ASSEMBLY_REQUIRED";
    if (winning_stage == 2U) return "ANCHOR_ADDITION_REEVALUATION_REQUIRED";
    if (winning_stage == 3U) return "BALL_BOX_PROJECTION_MODEL_REQUIRED";
    return "MODEL_PROJECTION_CONSISTENT_CANDIDATE";
}

struct ALR54ReplayCapture {
    bool captured = false;
    ALR53ReplayCapture r53;
    ALR54Result decomposition;
};

} // namespace

SplitBoundaryReport run_al_model_projection_decomposition_controls_impl(
    ALR54ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R54_IDENTITY_PROJECTION)
        == B4E2D7R19R54_IDENTITY_SHA256;
    ALR53ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_operator_consistency_decomposition_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "490669f7533c6b160a8426b029ee458cf80dc97439872f9b16bbf077a94b531a"
        && parent.json.find(
            "4a778c2e546636c28f38d71a72f86e434c089fbc0a441b2babab1e519d86e80c")
            != std::string::npos
        && parent.json.find("HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED")
            != std::string::npos;
    ALR52ReplayCapture& r52_capture = capture.r52;
    ALR51ReplayCapture& r51_capture = r52_capture.r51;
    ALR52Result& r52 = r52_capture.depth;
    ALR51MasterResult& r51 = r51_capture.master;
    ALR50Generator& r50 = r51_capture.r50.generator;
    ALR43ReplayCapture& r43 =
        r51_capture.r50.r49.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before = al_r42_superset_root(r43.r42.superset);
    const std::string anchor_before = al_binary64_vec3_root(r50.witness);
    const std::string witness_before = parent_exact
        ? al_binary64_vec3_root(r52.records[2].witness) : std::string{};
    const bool source_exact = parent_exact && r30.captured
        && r52.records.size() == 3U && r52.records[2].sweeps == 64U
        && r52.records[2].root
            == "2e0afdb4dbdac10cef5ba591b5764f7cce40ca414458e7f5b9c61cdbc83425f0"
        && witness_before
            == "ccc1084988fb56a829649f60c292210f24723a4932874923640d5bdbf99e8729"
        && r51.union_root
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28"
        && r51.extended_cache_root
            == "f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5"
        && capture.decomposition.pair_root
            == "f19cc1913ac2cf0061c9f6240f0f6378fc513108c77975e02cbd0f1f45f632ca"
        && capture.decomposition.directed_root
            == capture.decomposition.pair_root;

    std::string dense_root;
    const bool dense = identity && al_r54_dense_controls(&dense_root);
    ALSparseWorkTrace workspace_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    JointNeighborhood superset = b4ep3_build_superset(
        tagged_points(r30.position), binding);
    JointNeighborhood moved_filtered = b4ep3_filter_superset(
        tagged_points(r43.projected_trial), superset);
    ALNormalizedWorkspace moved = build_al_normalized_sparse_workspace(
        r43.projected_trial, binding, r30.dual, r30.theta,
        &workspace_work, &static_work, &adjacency_work);
    const bool workspace_exact = source_exact && dense && index.passed
        && binding.passed && superset.passed && moved.passed;
    const bool topology_exact = workspace_exact
        && al_r42_superset_root(superset) == superset_before
        && moved_filtered.pairs == moved.neighborhood.pairs;
    const bool master_exact = topology_exact && r51.rows.size() == 494U
        && r51.cache.size() == 494U
        && al_r51_cache_root(r51.cache) == r51.extended_cache_root;

    std::vector<Vec3> lower(r43.projected_trial.size());
    std::vector<Vec3> upper(r43.projected_trial.size());
    bool intervals_exact = master_exact;
    for (std::size_t i = 0U; i < r43.projected_trial.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face =
                al_r41_component(fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            if (y < low_face) low = 0.0;
            else if (y > high_face) high = 0.0;
            al_r43_component(lower[i], axis) = low;
            al_r43_component(upper[i], axis) = high;
            intervals_exact = intervals_exact && std::isfinite(low)
                && std::isfinite(high) && low <= 0.0 && high >= 0.0;
        }

    ALR54Result decomposition;
    if (intervals_exact) decomposition = al_r54_decompose(
        moved, r50, r51, r52.records[2], lower, upper);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);
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
        && decomposition.pair_passes == 2U
        && decomposition.binary128_gram_folds == 1U
        && decomposition.geometry_folds == 1U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && anchor_before == al_binary64_vec3_root(r50.witness)
        && witness_before == al_binary64_vec3_root(r52.records[2].witness)
        && capture.decomposition.pair_full_sign_disagreements == 0U;

    const std::array<const char*, 17U> expected_routes{{
        "MODEL_PARENT_REJECTED", "MODEL_SOURCE_REJECTED",
        "MODEL_WORKSPACE_REJECTED", "MODEL_TOPOLOGY_REJECTED",
        "MODEL_MASTER_REJECTED", "MODEL_LAMBDA_REJECTED",
        "MODEL_GRAM_REJECTED", "MODEL_CORRECTION_REJECTED",
        "MODEL_TARGET_REJECTED", "MODEL_GEOMETRY_REJECTED",
        "MODEL_ROW_COMPARISON_REJECTED", "MODEL_WORK_REJECTED",
        "HILDRETH_EXPLICIT_RESIDUAL_REPLACEMENT_REQUIRED",
        "GRAM_CORRECTION_ASSEMBLY_REQUIRED",
        "ANCHOR_ADDITION_REEVALUATION_REQUIRED",
        "BALL_BOX_PROJECTION_MODEL_REQUIRED",
        "MODEL_PROJECTION_CONSISTENT_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 12U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::size_t stage = route_index < 12U ? 4U : route_index - 12U;
        const std::string observed = al_r54_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            stage);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r54_route(parent_exact, source_exact,
        workspace_exact, topology_exact, master_exact,
        decomposition.lambda_exact, decomposition.gram_exact,
        decomposition.correction_exact, decomposition.target_exact,
        decomposition.geometry_exact, decomposition.comparison_exact,
        work_exact && rollback, decomposition.winning_stage);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && topology_exact && master_exact
        && decomposition.lambda_exact && decomposition.gram_exact
        && decomposition.correction_exact && decomposition.target_exact
        && decomposition.geometry_exact && decomposition.comparison_exact
        && work_exact && rollback && routes_exact;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R54_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << r52.records[2].root << ':' << witness_before << ':'
             << decomposition.comparison_root << '|'
             << decomposition.recursive.positive << ':'
             << decomposition.direct64.positive << ':'
             << decomposition.direct128.positive << ':'
             << decomposition.correction_upper.positive << ':'
             << decomposition.anchor_linearity.positive << ':'
             << decomposition.target_raw.positive << ':'
             << decomposition.projected_raw.positive << '|'
             << decomposition.winning_stage << ':'
             << decomposition.target_root << ':'
             << decomposition.projected_root << ':'
             << decomposition.displacement_root << '|'
             << decomposition.pair_passes << ':'
             << decomposition.binary128_gram_folds << ':'
             << decomposition.geometry_folds << ':' << work_exact << ':'
             << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r54_model_projection_decomposition.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R54_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"record_root\":\"" << r52.records[2].root
           << "\",\"witness_root\":\"" << witness_before << "\"}"
           << ",\"positive\":{\"recursive\":"
           << decomposition.recursive.positive << ",\"direct64\":"
           << decomposition.direct64.positive << ",\"direct128\":"
           << decomposition.direct128.positive
           << ",\"correction_upper\":"
           << decomposition.correction_upper.positive
           << ",\"anchor_linearity\":"
           << decomposition.anchor_linearity.positive
           << ",\"target_raw\":" << decomposition.target_raw.positive
           << ",\"projected_raw\":"
           << decomposition.projected_raw.positive << "}"
           << ",\"maxima\":{\"recursive\":"
           << decomposition.recursive.maximum << ",\"recursive_row\":"
           << decomposition.recursive.row << ",\"direct64\":"
           << decomposition.direct64.maximum << ",\"direct64_row\":"
           << decomposition.direct64.row << ",\"direct128\":"
           << static_cast<double>(decomposition.direct128.maximum)
           << ",\"direct128_row\":" << decomposition.direct128.row
           << ",\"correction_upper\":"
           << decomposition.correction_upper.maximum
           << ",\"correction_upper_row\":"
           << decomposition.correction_upper.row
           << ",\"anchor_linearity\":"
           << decomposition.anchor_linearity.maximum
           << ",\"anchor_linearity_row\":"
           << decomposition.anchor_linearity.row << ",\"target_raw\":"
           << decomposition.target_raw.maximum << ",\"target_raw_row\":"
           << decomposition.target_raw.row << ",\"projected_raw\":"
           << decomposition.projected_raw.maximum
           << ",\"projected_raw_row\":"
           << decomposition.projected_raw.row << "}"
           << ",\"gaps\":{\"recursive_direct\":"
           << static_cast<double>(decomposition.recursive_direct.maximum)
           << ",\"recursive_direct_row\":"
           << decomposition.recursive_direct.row
           << ",\"direct64_direct128\":"
           << static_cast<double>(decomposition.direct64_direct128.maximum)
           << ",\"direct64_direct128_row\":"
           << decomposition.direct64_direct128.row
           << ",\"direct_correction\":"
           << static_cast<double>(decomposition.direct_correction.maximum)
           << ",\"direct_correction_row\":"
           << decomposition.direct_correction.row
           << ",\"anchor_target\":"
           << static_cast<double>(decomposition.anchor_target.maximum)
           << ",\"anchor_target_row\":"
           << decomposition.anchor_target.row
           << ",\"target_projected\":"
           << static_cast<double>(decomposition.target_projected.maximum)
           << ",\"target_projected_row\":"
           << decomposition.target_projected.row
           << ",\"winning_stage\":" << decomposition.winning_stage << "}"
           << ",\"worst_projected_row\":{\"row\":"
           << decomposition.final_worst_row << ",\"recursive\":"
           << decomposition.worst_recursive << ",\"direct64\":"
           << decomposition.worst_direct64 << ",\"direct128\":"
           << static_cast<double>(decomposition.worst_direct128)
           << ",\"correction_upper\":"
           << decomposition.worst_correction_upper
           << ",\"anchor_linearity\":"
           << decomposition.worst_anchor_linearity << ",\"target_raw\":"
           << decomposition.worst_target_raw << ",\"projected_raw\":"
           << decomposition.worst_projected_raw << ",\"anchor_bound\":"
           << decomposition.worst_anchor_bound << "}"
           << ",\"geometry\":{\"target_root\":\""
           << decomposition.target_root << "\",\"projected_root\":\""
           << decomposition.projected_root
           << "\",\"displacement_root\":\""
           << decomposition.displacement_root << "\",\"target_norm\":"
           << decomposition.target_norm << ",\"clamped_norm\":"
           << decomposition.clamped_norm << ",\"projected_norm\":"
           << decomposition.projected_norm << ",\"displacement_norm\":"
           << decomposition.displacement_norm
           << ",\"maximum_displacement\":"
           << decomposition.maximum_displacement
           << ",\"maximum_displacement_component\":"
           << decomposition.maximum_displacement_component
           << ",\"changed_components\":"
           << decomposition.changed_components << ",\"box_violations\":"
           << decomposition.box_violations
           << ",\"maximum_box_violation\":"
           << decomposition.maximum_box_violation << ",\"ball_outside\":"
           << (decomposition.ball_outside ? "true" : "false")
           << ",\"analytic_clamp_exact\":"
           << (decomposition.analytic_clamp_exact ? "true" : "false") << "}"
           << ",\"roots\":{\"recursive\":\""
           << decomposition.recursive.root << "\",\"direct64\":\""
           << decomposition.direct64.root << "\",\"direct128\":\""
           << decomposition.direct128.root
           << "\",\"correction_upper\":\""
           << decomposition.correction_upper.root
           << "\",\"anchor_linearity\":\""
           << decomposition.anchor_linearity.root << "\",\"target_raw\":\""
           << decomposition.target_raw.root << "\",\"projected_raw\":\""
           << decomposition.projected_raw.root << "\",\"comparison\":\""
           << decomposition.comparison_root << "\"}"
           << ",\"work\":{\"workspace_builds\":"
           << workspace_work.workspace_builds << ",\"workspace_releases\":"
           << workspace_work.workspace_releases << ",\"pair_passes\":"
           << decomposition.pair_passes << ",\"binary128_gram_folds\":"
           << decomposition.binary128_gram_folds
           << ",\"geometry_folds\":" << decomposition.geometry_folds
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"dense_root\":\"" << dense_root << "\""
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"current_directed_certificate_unchanged\":true"
              ",\"residual_replacement_applied\":false"
              ",\"correction_changed\":false,\"projection_changed\":false"
              ",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r53 = std::move(capture);
        replay_capture->decomposition = std::move(decomposition);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_model_projection_decomposition_controls() {
    return run_al_model_projection_decomposition_controls_impl(nullptr);
}
