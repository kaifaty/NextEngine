namespace {

constexpr const char* B4E2D7R19R18_IDENTITY_SHA256 =
    "6f26279f6422ca0d9b4f76a9eb15ae10d8b961a2d807ba98ca6881055c80bcc0";
constexpr const char* B4E2D7R19R18_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r18-shadow-outer6-resume|v1|parent=a4fe1a74:d930b6d57b1bd11936949d7ef7ab2e7f2c607c048fd9b53db6fcfb94f41b2447:c0c6d0285a724dd3615f5cfab077e4b8bc86a73abede691b60c847889a15a173|source=069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9;grant=266f8ac864d9a83e7bf71fa1df0b4dc33fe964d6064ced058287f04e61f4d475;active-owner=cc6b9e854ca39d940f826899f02827765f293b96bbf9429f7b5b7caceddae9b2;owner-state=d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54;epoch1;outer6;slice0;cumulative523|owner-consume=flags1-to3;consumed-root-abbef5a38a947f21945d9a96e50df2597a4ac4647a2d733c911ff55cb74eaa74|candidate=slice-budget512;total0;oracle=unsliced-budget8704;total523;both-one-outer6;same-position-dual-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=6,2,25,1,0,523,33,21,504,19,2,21,0|bounds=outer1;inner<=16;hvp<=512;workspaces<=255;precision<=43;cumulative<=8704|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model|result=roots-derived-after-independent-execution;receipt-NEALRSM1-schema1-body404-total420;state-NEALRST1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-resume-parent-rejected;shadow-resume-source-rejected;shadow-resume-grant-rejected;shadow-resume-owner-rejected;shadow-resume-duplicate-rejected;shadow-resume-stale-epoch-rejected;shadow-resume-resource-rejected;shadow-resume-candidate-rejected;shadow-resume-oracle-rejected;shadow-resume-oracle-mismatch;shadow-resume-aborted;shadow-outer6-continuation-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer6-continuation-candidate-only)IDENTITY";

constexpr std::size_t AL_RESUME_RECEIPT_ROOT_COUNT = 7U;
constexpr std::size_t AL_RESUME_RECEIPT_BODY_BYTES = 404U;
constexpr std::size_t AL_RESUME_RECEIPT_BYTES = 420U;
constexpr std::size_t AL_RESUME_STATE_BODY_BYTES = 196U;
constexpr std::size_t AL_RESUME_STATE_BYTES = 212U;
constexpr std::uint32_t AL_ACTIVE_OWNER_CONSUMED = 3U;
constexpr std::uint32_t AL_RESUME_RECEIPT_FLAGS = 3U;
constexpr std::uint32_t AL_RESUME_STATE_FLAGS = 31U;

struct ALShadowResumeReceiptV1 {
    std::uint32_t flags = 0U;
    std::uint64_t epoch = 0U;
    std::uint64_t outer = 0U;
    std::array<std::string, AL_RESUME_RECEIPT_ROOT_COUNT> roots;
    std::array<std::uint64_t, AL_V2_LIMIT_COUNT> limits{};
    std::array<std::uint64_t, AL_V2_USED_COUNT> used{};
};

struct ALShadowResumeStateV1 {
    std::uint32_t flags = 0U;
    std::array<std::string, 6U> roots;
};

struct ALOuter6Execution {
    ALNormalizedOuterUpdate update;
    ALSparseWorkTrace work;
    ALSparsePrecisionWorkTrace precision_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALSparseStructuralBudget budget;
    ALNormalizedKrylovForcingTrace forcing_trace;
    ALNormalizedTrustCompletionTrace completion_trace;
    std::string static_identity;
    std::string update_root;
    std::string work_root;
    std::size_t base_total_hvp = 0U;
    std::size_t delta_hvp = 0U;
    std::size_t delta_workspaces = 0U;
    std::size_t delta_precision = 0U;
};

struct ALShadowResumeCase {
    std::string name;
    std::string expected;
    std::string observed;
    bool state_exact = false;
    bool passed = false;
};

bool al_encode_shadow_resume_receipt_v1(
    const ALShadowResumeReceiptV1& receipt, std::string& bytes) {
    std::string body;
    body.reserve(AL_RESUME_RECEIPT_BODY_BYTES);
    al_append_u32_le(body, receipt.flags);
    al_append_u64_le(body, receipt.epoch);
    al_append_u64_le(body, receipt.outer);
    for (const std::string& root : receipt.roots) {
        if (!al_append_digest_bytes(body, root)) return false;
    }
    for (std::uint64_t limit : receipt.limits) {
        al_append_u64_le(body, limit);
    }
    for (std::uint64_t used : receipt.used) {
        al_append_u64_le(body, used);
    }
    return al_encode_canonical_v1(
        "NEALRSM1", AL_RESUME_RECEIPT_BODY_BYTES,
        AL_RESUME_RECEIPT_BYTES, body, bytes);
}

bool al_decode_shadow_resume_receipt_v1(
    const std::string& bytes, ALShadowResumeReceiptV1& receipt) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALRSM1", AL_RESUME_RECEIPT_BODY_BYTES,
            AL_RESUME_RECEIPT_BYTES, offset)
        || !al_read_u32_le(bytes, offset, receipt.flags)
        || receipt.flags != AL_RESUME_RECEIPT_FLAGS
        || !al_read_u64_le(bytes, offset, receipt.epoch)
        || !al_read_u64_le(bytes, offset, receipt.outer)) {
        return false;
    }
    for (std::string& root : receipt.roots) {
        if (!al_read_digest_hex(bytes, offset, root)) return false;
    }
    for (std::uint64_t& limit : receipt.limits) {
        if (!al_read_u64_le(bytes, offset, limit)) return false;
    }
    for (std::uint64_t& used : receipt.used) {
        if (!al_read_u64_le(bytes, offset, used)) return false;
    }
    if (offset != bytes.size()) return false;
    std::string reencoded;
    return al_encode_shadow_resume_receipt_v1(receipt, reencoded)
        && reencoded == bytes;
}

bool al_shadow_resume_receipt_exact(
    const ALShadowResumeReceiptV1& lhs,
    const ALShadowResumeReceiptV1& rhs) {
    return lhs.flags == rhs.flags && lhs.epoch == rhs.epoch
        && lhs.outer == rhs.outer && lhs.roots == rhs.roots
        && lhs.limits == rhs.limits && lhs.used == rhs.used;
}

bool al_encode_shadow_resume_state_v1(
    const ALShadowResumeStateV1& state, std::string& bytes) {
    std::string body;
    body.reserve(AL_RESUME_STATE_BODY_BYTES);
    al_append_u32_le(body, state.flags);
    for (const std::string& root : state.roots) {
        if (!al_append_digest_bytes(body, root)) return false;
    }
    return al_encode_canonical_v1(
        "NEALRST1", AL_RESUME_STATE_BODY_BYTES, AL_RESUME_STATE_BYTES,
        body, bytes);
}

bool al_decode_shadow_resume_state_v1(
    const std::string& bytes, ALShadowResumeStateV1& state) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALRST1", AL_RESUME_STATE_BODY_BYTES,
            AL_RESUME_STATE_BYTES, offset)
        || !al_read_u32_le(bytes, offset, state.flags)
        || state.flags != AL_RESUME_STATE_FLAGS) {
        return false;
    }
    for (std::string& root : state.roots) {
        if (!al_read_digest_hex(bytes, offset, root)) return false;
    }
    if (offset != bytes.size()) return false;
    std::string reencoded;
    return al_encode_shadow_resume_state_v1(state, reencoded)
        && reencoded == bytes;
}

bool al_shadow_resume_state_exact(
    const ALShadowResumeStateV1& lhs,
    const ALShadowResumeStateV1& rhs) {
    return lhs.flags == rhs.flags && lhs.roots == rhs.roots;
}

std::string al_outer6_update_root(const ALNormalizedOuterUpdate& update) {
    std::ostringstream projection;
    al_normalized_outer_update_projection(projection, update);
    projection << '|'
               << update.support.finite_values << ':'
               << binary64_bits(update.support.energy) << ':'
               << update.support.active_centers << ':'
               << update.support.fluid_pairs << ':'
               << update.support.boundary_pairs << '|';
    for (double value : update.support.density) {
        projection << binary64_bits(value) << ';';
    }
    projection << '|';
    for (double value : update.support.constraint) {
        projection << binary64_bits(value) << ';';
    }
    projection << '|';
    for (double value : update.support.active_u) {
        projection << binary64_bits(value) << ';';
    }
    projection << '|' << al_binary64_vec3_root(update.support.gradient);
    for (const ALNormalizedPrivateTrial& trial : update.trials) {
        projection << '|' << trial.recurrence_hvp_calls << ':'
                   << trial.model_hvp_calls << ':'
                   << trial.guarded_completion_attempted << ':'
                   << trial.guarded_completion_eligible << ':'
                   << trial.guarded_completion_used << ':'
                   << trial.grace_tier << ':'
                   << trial.tier2_continuation_attempted << ':'
                   << trial.tier2_continuation_eligible << ':'
                   << trial.direct_model_used << ':'
                   << trial.residual_model_used;
    }
    return sha256_hex(projection.str());
}

std::string al_outer6_work_root(const ALOuter6Execution& value) {
    std::ostringstream projection;
    projection << value.work.workspace_builds << ':'
               << value.work.workspace_releases << ':'
               << value.work.live_workspaces << ':'
               << value.work.maximum_live_workspaces << ':'
               << value.work.pair_visits << ':'
               << value.work.divided_union_visits << ':'
               << value.work.current_members << ':'
               << value.work.trial_members << ':'
               << value.work.all_pair_candidate_calls << ':'
               << value.work.lifecycle_underflow << '|'
               << value.precision_work.superset_builds << ':'
               << value.precision_work.union_candidate_pairs << ':'
               << value.precision_work.long_double_audits << ':'
               << value.precision_work.long_double_union_candidate_pairs
               << ':'
               << value.precision_work.long_double_evaluation_pair_visits
               << ':' << value.precision_work.binary128_audits << ':'
               << value.precision_work.binary128_union_candidate_pairs
               << ':'
               << value.precision_work.binary128_evaluation_pair_visits
               << ':' << value.precision_work.all_pair_candidate_calls
               << '|' << value.static_work.workspace_builds << ':'
               << value.static_work.static_index_builds << ':'
               << value.static_work.support_canonicalizations << ':'
               << value.static_work.index_identity_sha256 << '|'
               << value.adjacency_work.workspace_builds << ':'
               << value.adjacency_work.flat_offset_records << ':'
               << value.adjacency_work.flat_pair_index_records << '|'
               << value.delta_hvp << ':' << value.delta_workspaces << ':'
               << value.delta_precision << ':'
               << value.budget.inner_trials_in_update << ':'
               << value.budget.hvp_in_trust_step << ':'
               << value.forcing_trace.inherited_trust_steps << ':'
               << value.forcing_trace.dimensionless_trust_steps << '|'
               << value.completion_trace.recurrence_hvp << ':'
               << value.completion_trace.direct_model_hvp << ':'
               << value.completion_trace.residual_model_uses << ':'
               << value.completion_trace.guard_attempts << ':'
               << value.completion_trace.tier1_admitted << ':'
               << value.completion_trace.tier1_converged << ':'
               << value.completion_trace.tier2_admitted << ':'
               << value.completion_trace.tier2_converged << ':'
               << value.completion_trace.guard_denied;
    return sha256_hex(projection.str());
}

ALOuter6Execution al_execute_outer6(
    const std::vector<Vec3>& predicted,
    const std::vector<Vec3>& position,
    const std::vector<double>& dual,
    double theta,
    bool unsliced_oracle) {
    ALOuter6Execution result;
    result.base_total_hvp = unsliced_oracle ? 523U : 0U;
    result.budget.maximum_outer_updates = 16U;
    result.budget.maximum_inner_trials_per_update = 16U;
    result.budget.maximum_hvp_per_trust_step = 34U;
    result.budget.maximum_total_hvp = unsliced_oracle ? 8704U : 512U;
    result.budget.maximum_workspace_builds = 288U;
    result.budget.maximum_precision_audits = 64U;
    result.budget.outer_updates = 6U;
    result.budget.inner_trials_in_update = 2U;
    result.budget.hvp_in_trust_step = 25U;
    result.budget.total_hvp = result.base_total_hvp;
    result.budget.workspace_builds = 33U;
    result.budget.precision_audits = 21U;

    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &result.static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    result.static_identity = index.identity_sha256;
    if (index.passed && binding.passed
        && al_sparse_budget_begin_outer_update(&result.budget)) {
        result.update = al_normalized_private_outer_update(
            predicted, position, dual, theta, 6, binding, &result.work,
            &result.precision_work, &result.static_work,
            &result.adjacency_work, &result.budget, true,
            ALNormalizedKrylovForcingPolicy::DimensionlessStationarity,
            &result.forcing_trace, ALPrecisionMembershipPolicy::Binary64Owned,
            nullptr, ALNormalizedTrustCompletionPolicy::TieredGraceResidualModel,
            &result.completion_trace);
    } else {
        result.update.failure = "OUTER6_PREWORK_INVALID";
    }
    result.delta_hvp = result.budget.total_hvp - result.base_total_hvp;
    result.delta_workspaces = result.budget.workspace_builds - 33U;
    result.delta_precision = result.budget.precision_audits - 21U;
    result.update_root = al_outer6_update_root(result.update);
    result.work_root = al_outer6_work_root(result);
    return result;
}

bool al_outer6_trials_exact(
    const ALNormalizedOuterUpdate& lhs,
    const ALNormalizedOuterUpdate& rhs) {
    if (lhs.trials.size() != rhs.trials.size()) return false;
    for (std::size_t index = 0U; index < lhs.trials.size(); ++index) {
        const ALNormalizedPrivateTrial& a = lhs.trials[index];
        const ALNormalizedPrivateTrial& b = rhs.trials[index];
        if (!al_normalized_private_trial_legacy_exact(a, b)
            || a.recurrence_hvp_calls != b.recurrence_hvp_calls
            || a.model_hvp_calls != b.model_hvp_calls
            || a.guarded_completion_attempted
                != b.guarded_completion_attempted
            || a.guarded_completion_eligible
                != b.guarded_completion_eligible
            || a.guarded_completion_used != b.guarded_completion_used
            || a.grace_tier != b.grace_tier
            || a.tier2_continuation_attempted
                != b.tier2_continuation_attempted
            || a.tier2_continuation_eligible
                != b.tier2_continuation_eligible
            || a.direct_model_used != b.direct_model_used
            || a.residual_model_used != b.residual_model_used) {
            return false;
        }
    }
    return true;
}

bool al_outer6_execution_exact(
    const ALOuter6Execution& candidate,
    const ALOuter6Execution& oracle) {
    return candidate.update_root == oracle.update_root
        && candidate.work_root == oracle.work_root
        && candidate.static_identity == oracle.static_identity
        && candidate.delta_hvp == oracle.delta_hvp
        && candidate.delta_workspaces == oracle.delta_workspaces
        && candidate.delta_precision == oracle.delta_precision
        && candidate.budget.outer_updates == oracle.budget.outer_updates
        && candidate.budget.inner_trials_in_update
            == oracle.budget.inner_trials_in_update
        && candidate.budget.hvp_in_trust_step
            == oracle.budget.hvp_in_trust_step
        && candidate.budget.workspace_builds
            == oracle.budget.workspace_builds
        && candidate.budget.precision_audits
            == oracle.budget.precision_audits
        && candidate.update.passed == oracle.update.passed
        && candidate.update.precision_hard_failure
            == oracle.update.precision_hard_failure
        && candidate.update.sign_contradiction
            == oracle.update.sign_contradiction
        && candidate.update.oracle_bound_required
            == oracle.update.oracle_bound_required
        && candidate.update.failure == oracle.update.failure
        && exact_vec3_values(
            candidate.update.position, oracle.update.position)
        && candidate.update.u == oracle.update.u
        && candidate.update.support.finite_values
            == oracle.update.support.finite_values
        && binary64_bits(candidate.update.support.energy)
            == binary64_bits(oracle.update.support.energy)
        && candidate.update.support.density == oracle.update.support.density
        && candidate.update.support.constraint
            == oracle.update.support.constraint
        && candidate.update.support.active_u
            == oracle.update.support.active_u
        && exact_vec3_values(
            candidate.update.support.gradient,
            oracle.update.support.gradient)
        && candidate.update.support.active_centers
            == oracle.update.support.active_centers
        && candidate.update.support.fluid_pairs
            == oracle.update.support.fluid_pairs
        && candidate.update.support.boundary_pairs
            == oracle.update.support.boundary_pairs
        && al_outer6_trials_exact(candidate.update, oracle.update);
}

bool al_outer6_execution_valid(
    const ALOuter6Execution& value,
    bool unsliced_oracle,
    const std::string& expected_static_identity) {
    std::size_t recorded_hvp = 0U;
    std::size_t recorded_accepted = 0U;
    std::size_t recorded_rejected = 0U;
    for (const ALNormalizedPrivateTrial& trial : value.update.trials) {
        recorded_hvp += static_cast<std::size_t>(trial.hvp_calls);
        recorded_accepted += trial.candidate_accepted ? 1U : 0U;
        recorded_rejected += trial.candidate_accepted ? 0U : 1U;
    }
    const std::size_t expected_total =
        (unsliced_oracle ? 523U : 0U) + value.delta_hvp;
    return value.update.passed && value.update.state.outer == 6
        && value.update.state.finite_values
        && !value.update.precision_hard_failure
        && !value.update.sign_contradiction
        && !value.update.oracle_bound_required
        && !value.budget.exhausted && value.budget.failure.empty()
        && value.budget.outer_updates == 7U
        && value.budget.inner_trials_in_update <= 16U
        && value.delta_hvp <= 512U
        && value.delta_workspaces <= 255U
        && value.delta_precision <= 43U
        && value.budget.total_hvp == expected_total
        && 523U + value.delta_hvp <= 8704U
        && value.budget.workspace_builds == 33U + value.delta_workspaces
        && value.budget.precision_audits == 21U + value.delta_precision
        && value.work.workspace_builds == value.delta_workspaces
        && value.work.workspace_releases == value.delta_workspaces
        && value.work.live_workspaces == 0U
        && value.work.maximum_live_workspaces <= 2U
        && !value.work.lifecycle_underflow
        && value.work.all_pair_candidate_calls == 0U
        && value.precision_work.all_pair_candidate_calls == 0U
        && value.static_work.static_index_builds == 1U
        && value.static_work.support_canonicalizations == 1U
        && value.static_work.workspace_builds == value.delta_workspaces
        && value.adjacency_work.workspace_builds == value.delta_workspaces
        && value.static_identity == expected_static_identity
        && recorded_hvp == value.delta_hvp
        && recorded_hvp
            == value.completion_trace.recurrence_hvp
                + value.completion_trace.direct_model_hvp
        && recorded_accepted + recorded_rejected
            == value.update.trials.size()
        && value.delta_precision
            == value.precision_work.long_double_audits
                + value.precision_work.binary128_audits;
}

std::array<std::uint64_t, AL_V2_USED_COUNT>
al_outer6_successor_used(
    const ALEpochGrantV1& grant,
    const ALOuter6Execution& execution) {
    auto used = grant.used;
    std::size_t accepted = 0U;
    std::size_t rejected = 0U;
    std::size_t residual_models = 0U;
    for (const ALNormalizedPrivateTrial& trial : execution.update.trials) {
        accepted += trial.candidate_accepted ? 1U : 0U;
        rejected += trial.candidate_accepted ? 0U : 1U;
        residual_models += trial.residual_model_used ? 1U : 0U;
    }
    used[AL_V2_USED_OUTER] = 7U;
    used[AL_V2_USED_INNER] = execution.budget.inner_trials_in_update;
    used[AL_V2_USED_LAST_TRIAL_HVP] =
        execution.update.trials.empty() ? 0U
        : static_cast<std::uint64_t>(
            execution.update.trials.back().hvp_calls);
    used[AL_V2_USED_SLICE_HVP] = execution.delta_hvp;
    used[AL_V2_USED_CUMULATIVE_HVP] = 523U + execution.delta_hvp;
    used[AL_V2_USED_WORKSPACES] = 33U + execution.delta_workspaces;
    used[AL_V2_USED_PRECISION] = 21U + execution.delta_precision;
    used[AL_V2_USED_RECURRENCE_HVP] +=
        execution.completion_trace.recurrence_hvp;
    used[AL_V2_USED_DIRECT_MODEL_HVP] +=
        execution.completion_trace.direct_model_hvp;
    used[AL_V2_USED_RESIDUAL_MODELS] += residual_models;
    used[AL_V2_USED_ACCEPTED] += accepted;
    used[AL_V2_USED_REJECTED] += rejected;
    return used;
}

std::string al_outer6_successor_history_root(
    const std::string& previous_history,
    const std::string& update_root,
    const std::string& position_root,
    const std::string& dual_root,
    const std::array<std::uint64_t, AL_V2_USED_COUNT>& used) {
    std::ostringstream projection;
    projection << "nextengine.nonlocal.al-continuation-history|v2|"
               << previous_history << '|' << update_root << '|'
               << position_root << '|' << dual_root << '|';
    for (std::uint64_t value : used) projection << value << ':';
    return sha256_hex(projection.str());
}

std::string al_shadow_resume_route(
    bool parent,
    bool source,
    bool grant,
    bool owner,
    bool consumed,
    bool stale,
    bool resource,
    bool candidate,
    bool oracle,
    bool match,
    bool abort) {
    if (!parent) return "SHADOW_RESUME_PARENT_REJECTED";
    if (!source) return "SHADOW_RESUME_SOURCE_REJECTED";
    if (!grant) return "SHADOW_RESUME_GRANT_REJECTED";
    if (!owner) return "SHADOW_RESUME_OWNER_REJECTED";
    if (consumed) return "SHADOW_RESUME_DUPLICATE_REJECTED";
    if (stale) return "SHADOW_RESUME_STALE_EPOCH_REJECTED";
    if (!resource) return "SHADOW_RESUME_RESOURCE_REJECTED";
    if (!candidate) return "SHADOW_RESUME_CANDIDATE_REJECTED";
    if (!oracle) return "SHADOW_RESUME_ORACLE_REJECTED";
    if (!match) return "SHADOW_RESUME_ORACLE_MISMATCH";
    if (abort) return "SHADOW_RESUME_ABORTED";
    return "SHADOW_OUTER6_CONTINUATION_CANDIDATE";
}

std::size_t al_shadow_resume_route_index(const std::string& route) {
    if (route == "SHADOW_RESUME_PARENT_REJECTED") return 0U;
    if (route == "SHADOW_RESUME_SOURCE_REJECTED") return 1U;
    if (route == "SHADOW_RESUME_GRANT_REJECTED") return 2U;
    if (route == "SHADOW_RESUME_OWNER_REJECTED") return 3U;
    if (route == "SHADOW_RESUME_DUPLICATE_REJECTED") return 4U;
    if (route == "SHADOW_RESUME_STALE_EPOCH_REJECTED") return 5U;
    if (route == "SHADOW_RESUME_RESOURCE_REJECTED") return 6U;
    if (route == "SHADOW_RESUME_CANDIDATE_REJECTED") return 7U;
    if (route == "SHADOW_RESUME_ORACLE_REJECTED") return 8U;
    if (route == "SHADOW_RESUME_ORACLE_MISMATCH") return 9U;
    if (route == "SHADOW_RESUME_ABORTED") return 10U;
    return 11U;
}

bool al_decode_active_epoch_owner_for_resume_v1(
    const std::string& bytes, ALActiveEpochOwnerV1& owner) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALAOW1", AL_ACTIVE_OWNER_BODY_BYTES,
            AL_ACTIVE_OWNER_BYTES, offset)
        || !al_read_u32_le(bytes, offset, owner.flags)
        || (owner.flags != AL_ACTIVE_OWNER_ACTIVE
            && owner.flags != AL_ACTIVE_OWNER_CONSUMED)
        || !al_read_digest_hex(bytes, offset, owner.expected_grant_root)
        || !al_read_digest_hex(bytes, offset, owner.source_envelope_root)
        || !al_read_u64_le(bytes, offset, owner.expected_epoch)
        || !al_read_digest_hex(
            bytes, offset, owner.transition_receipt_root)
        || offset != bytes.size()) {
        return false;
    }
    std::string reencoded;
    return al_encode_active_epoch_owner_v1(owner, reencoded)
        && reencoded == bytes;
}

struct ALShadowOuter6ResumeCapture {
    bool captured = false;
    ALOwnerEpochTransitionCapture r17;
    ALOuter6Execution candidate;
    ALShadowResumeReceiptV1 receipt;
    ALShadowResumeStateV1 state;
    std::string receipt_root;
    std::string state_root;
};

} // namespace

SplitBoundaryReport run_al_shadow_outer6_resume_controls_impl(
    ALShadowOuter6ResumeCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R18_IDENTITY_PROJECTION)
        == B4E2D7R19R18_IDENTITY_SHA256;

    ALOwnerEpochTransitionCapture capture;
    const SplitBoundaryReport r17_parent =
        run_al_owner_epoch_transition_controls_impl(&capture);
    const std::string r17_stdout_sha256 = sha256_hex(r17_parent.json + "\n");
    const bool r17_semantic_exact = r17_parent.json.find(
        "\"result_sha256\":\""
        "c0c6d0285a724dd3615f5cfab077e4b8bc86a73abede691b60c847889a15a173"
        "\"") != std::string::npos;
    const bool parents_retained = r17_parent.json.find(
        "\"transitive_parents_retained\":true") != std::string::npos
        && r17_parent.json.find(
            "d493c68da969911afe7228e26dbfcb32434ebdf3c028e4aaef91a9ca30af689f")
            != std::string::npos;
    const bool parent_exact = r17_parent.passed && capture.captured
        && r17_semantic_exact && parents_retained
        && r17_stdout_sha256
            == "d930b6d57b1bd11936949d7ef7ab2e7f2c607c048fd9b53db6fcfb94f41b2447";

    const ALV2EnvelopeCapture& r16 = capture.r16;
    const ALSoftCapSuspensionCapture& r14 = r16.r15.r14;
    const ALNormalizedTotalHvpBoundaryCapture& boundary =
        r14.r13.r12.r11.r10.total_hvp_boundary;
    const std::vector<Vec3>& position = r14.r13.r12.trial_position;
    const std::vector<double>& dual = r14.r13.outer_u;
    const std::vector<Vec3>& predicted = boundary.predicted_position;
    const ALContinuationEnvelopeV2& source = r16.envelope;

    const ALContinuationOwnerV2 validation_owner{
        r16.envelope_root, source.used[AL_V2_USED_EPOCH], false};
    const ALV2ValidationResult source_validation =
        al_validate_continuation_v2(
            r16.envelope_bytes, r16.envelope_root, validation_owner,
            source, position, dual, r16.parent_transaction_root);
    const bool source_exact = source_validation.route
            == "TOKEN_V2_VALIDATION_CANDIDATE"
        && r16.envelope_root
            == "069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9";

    std::string grant_bytes;
    ALEpochGrantV1 decoded_grant;
    const bool grant_exact = al_encode_epoch_grant_v1(
            capture.state.grant, grant_bytes)
        && al_decode_epoch_grant_v1(grant_bytes, decoded_grant)
        && al_epoch_grant_exact(decoded_grant, capture.state.grant)
        && al_epoch_grant_semantically_valid(
            decoded_grant, source, r16.envelope_root)
        && sha256_hex(grant_bytes)
            == "266f8ac864d9a83e7bf71fa1df0b4dc33fe964d6064ced058287f04e61f4d475";
    const std::string grant_root = sha256_hex(grant_bytes);

    std::string transition_receipt_bytes;
    ALOwnerTransitionReceiptV1 decoded_transition_receipt;
    const bool transition_receipt_exact =
        al_encode_owner_transition_receipt_v1(
            capture.state.receipt, transition_receipt_bytes)
        && al_decode_owner_transition_receipt_v1(
            transition_receipt_bytes, decoded_transition_receipt)
        && al_owner_transition_receipt_exact(
            decoded_transition_receipt, capture.state.receipt)
        && al_transition_receipt_semantically_valid(
            decoded_transition_receipt, source, r16.envelope_root,
            grant_root);
    const std::string transition_receipt_root =
        sha256_hex(transition_receipt_bytes);

    std::string active_owner_bytes;
    ALActiveEpochOwnerV1 decoded_active_owner;
    const bool active_owner_exact = al_encode_active_epoch_owner_v1(
            capture.state.active_owner, active_owner_bytes)
        && al_decode_active_epoch_owner_v1(
            active_owner_bytes, decoded_active_owner)
        && al_active_epoch_owner_exact(
            decoded_active_owner, capture.state.active_owner)
        && al_active_owner_semantically_valid(
            decoded_active_owner, source, r16.envelope_root, grant_root,
            transition_receipt_root)
        && sha256_hex(active_owner_bytes)
            == "cc6b9e854ca39d940f826899f02827765f293b96bbf9429f7b5b7caceddae9b2";
    std::string r17_state_bytes;
    std::string r17_state_root;
    const bool r17_state_exact = al_owner_epoch_state_roundtrip_v1(
            capture.state, r17_state_bytes, r17_state_root)
        && r17_state_root
            == "d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54";
    const bool chain_exact = source_exact && grant_exact
        && transition_receipt_exact && active_owner_exact
        && r17_state_exact;
    const bool resource_prework_exact = decoded_grant.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                6U, 2U, 25U, 1U, 0U, 523U, 33U, 21U,
                504U, 19U, 2U, 21U, 0U}
        && decoded_grant.limits
            == std::array<std::uint64_t, AL_V2_LIMIT_COUNT>{
                16U, 16U, 34U, 512U, 288U, 64U, 8704U}
        && decoded_grant.used[AL_V2_USED_CUMULATIVE_HVP]
            < decoded_grant.limits[AL_V2_LIMIT_CUMULATIVE_HVP];

    ALOuter6Execution candidate;
    ALOuter6Execution oracle;
    if (parent_exact && chain_exact && resource_prework_exact) {
        candidate = al_execute_outer6(
            predicted, position, dual, boundary.theta, false);
        oracle = al_execute_outer6(
            predicted, position, dual, boundary.theta, true);
    }
    const bool candidate_valid = al_outer6_execution_valid(
        candidate, false, source.roots[AL_V2_STATIC_ROOT]);
    const bool oracle_valid = al_outer6_execution_valid(
        oracle, true, source.roots[AL_V2_STATIC_ROOT]);
    const bool oracle_exact = candidate_valid && oracle_valid
        && al_outer6_execution_exact(candidate, oracle);

    const auto successor_used = al_outer6_successor_used(
        decoded_grant, candidate);
    const std::string position_root =
        al_binary64_vec3_root(candidate.update.position);
    const std::string dual_root = al_r10_dual_root(candidate.update.u);
    const std::string successor_history = al_outer6_successor_history_root(
        decoded_grant.history_root, candidate.update_root,
        position_root, dual_root, successor_used);

    ALActiveEpochOwnerV1 consumed_active_owner = capture.state.active_owner;
    consumed_active_owner.flags = AL_ACTIVE_OWNER_CONSUMED;
    std::string consumed_active_owner_bytes;
    ALActiveEpochOwnerV1 decoded_consumed_active_owner;
    const bool consumed_owner_exact = al_encode_active_epoch_owner_v1(
            consumed_active_owner, consumed_active_owner_bytes)
        && al_decode_active_epoch_owner_for_resume_v1(
            consumed_active_owner_bytes, decoded_consumed_active_owner)
        && al_active_epoch_owner_exact(
            decoded_consumed_active_owner, consumed_active_owner)
        && sha256_hex(consumed_active_owner_bytes)
            == "abbef5a38a947f21945d9a96e50df2597a4ac4647a2d733c911ff55cb74eaa74";
    const std::string consumed_active_owner_root =
        sha256_hex(consumed_active_owner_bytes);

    ALShadowResumeReceiptV1 receipt;
    receipt.flags = AL_RESUME_RECEIPT_FLAGS;
    receipt.epoch = 1U;
    receipt.outer = 6U;
    receipt.roots = {r16.envelope_root, grant_root, r17_state_root,
        candidate.update_root, position_root, dual_root, successor_history};
    receipt.limits = decoded_grant.limits;
    receipt.used = successor_used;
    std::string receipt_bytes;
    ALShadowResumeReceiptV1 decoded_receipt;
    const bool receipt_exact = oracle_exact && consumed_owner_exact
        && al_encode_shadow_resume_receipt_v1(receipt, receipt_bytes)
        && receipt_bytes.size() == AL_RESUME_RECEIPT_BYTES
        && al_decode_shadow_resume_receipt_v1(receipt_bytes, decoded_receipt)
        && al_shadow_resume_receipt_exact(decoded_receipt, receipt);
    const std::string receipt_root = receipt_exact
        ? sha256_hex(receipt_bytes) : std::string{};

    std::string source_owner_bytes;
    const bool source_owner_encoded = al_encode_source_owner_v1(
        capture.state.source_owner, source_owner_bytes);
    ALShadowResumeStateV1 resume_state;
    resume_state.flags = AL_RESUME_STATE_FLAGS;
    resume_state.roots = {r16.envelope_root,
        source_owner_encoded ? sha256_hex(source_owner_bytes) : std::string{},
        grant_root, transition_receipt_root, consumed_active_owner_root,
        receipt_root};
    std::string resume_state_bytes;
    ALShadowResumeStateV1 decoded_resume_state;
    const bool resume_state_exact = receipt_exact && source_owner_encoded
        && al_encode_shadow_resume_state_v1(resume_state, resume_state_bytes)
        && resume_state_bytes.size() == AL_RESUME_STATE_BYTES
        && al_decode_shadow_resume_state_v1(
            resume_state_bytes, decoded_resume_state)
        && al_shadow_resume_state_exact(decoded_resume_state, resume_state);
    const std::string resume_state_root = resume_state_exact
        ? sha256_hex(resume_state_bytes) : std::string{};

    std::vector<ALShadowResumeCase> cases;
    std::ostringstream case_projection;
    const auto observe_case = [&] (
        const std::string& name,
        const std::string& expected,
        bool parent,
        bool source_ok,
        bool grant_ok,
        bool owner_ok,
        bool consumed,
        bool stale,
        bool resource,
        bool candidate_ok,
        bool oracle_ok,
        bool match,
        bool abort,
        const std::string& before_root,
        const std::string& after_root,
        bool should_commit) {
        ALShadowResumeCase record;
        record.name = name;
        record.expected = expected;
        record.observed = al_shadow_resume_route(
            parent, source_ok, grant_ok, owner_ok, consumed, stale,
            resource, candidate_ok, oracle_ok, match, abort);
        record.state_exact = should_commit
            ? before_root != after_root : before_root == after_root;
        record.passed = record.expected == record.observed
            && record.state_exact;
        case_projection << record.name << ':' << record.expected << ':'
                        << record.observed << ':' << before_root << ':'
                        << after_root << ':' << record.state_exact << '|';
        cases.push_back(std::move(record));
    };
    const bool all_positive = parent_exact && source_exact && grant_exact
        && active_owner_exact && resource_prework_exact
        && candidate_valid && oracle_valid && oracle_exact
        && receipt_exact && resume_state_exact;
    observe_case("valid", "SHADOW_OUTER6_CONTINUATION_CANDIDATE",
        parent_exact, source_exact, grant_exact, active_owner_exact,
        false, false, resource_prework_exact, candidate_valid, oracle_valid,
        oracle_exact && receipt_exact && resume_state_exact, false,
        r17_state_root, resume_state_root, true);
    observe_case("parent", "SHADOW_RESUME_PARENT_REJECTED",
        false, true, true, true, false, false, true, true, true, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("source", "SHADOW_RESUME_SOURCE_REJECTED",
        true, false, true, true, false, false, true, true, true, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("grant", "SHADOW_RESUME_GRANT_REJECTED",
        true, true, false, true, false, false, true, true, true, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("active-owner", "SHADOW_RESUME_OWNER_REJECTED",
        true, true, true, false, false, false, true, true, true, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("duplicate", "SHADOW_RESUME_DUPLICATE_REJECTED",
        true, true, true, true, true, false, true, true, true, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("stale", "SHADOW_RESUME_STALE_EPOCH_REJECTED",
        true, true, true, true, false, true, true, true, true, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("resource", "SHADOW_RESUME_RESOURCE_REJECTED",
        true, true, true, true, false, false, false, true, true, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("candidate", "SHADOW_RESUME_CANDIDATE_REJECTED",
        true, true, true, true, false, false, true, false, true, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("oracle", "SHADOW_RESUME_ORACLE_REJECTED",
        true, true, true, true, false, false, true, true, false, true, false,
        r17_state_root, r17_state_root, false);
    observe_case("oracle-mismatch", "SHADOW_RESUME_ORACLE_MISMATCH",
        true, true, true, true, false, false, true, true, true, false, false,
        r17_state_root, r17_state_root, false);
    observe_case("abort-before-commit", "SHADOW_RESUME_ABORTED",
        true, true, true, true, false, false, true, true, true, true, true,
        r17_state_root, r17_state_root, false);
    observe_case("duplicate-replay", "SHADOW_RESUME_DUPLICATE_REJECTED",
        true, true, true, true, true, false, true, true, true, true, false,
        resume_state_root, resume_state_root, false);

    std::array<std::size_t, 12U> route_counts{};
    for (const ALShadowResumeCase& record : cases) {
        ++route_counts[al_shadow_resume_route_index(record.observed)];
    }
    const bool route_counts_exact = route_counts
        == std::array<std::size_t, 12U>{
            1U, 1U, 1U, 1U, 2U, 1U, 1U, 1U, 1U, 1U, 1U, 1U};
    const bool cases_exact = cases.size() == 13U
        && std::all_of(cases.begin(), cases.end(),
            [](const ALShadowResumeCase& value) { return value.passed; });
    const std::string case_root = sha256_hex(case_projection.str());
    const bool input_rollback_exact =
        al_binary64_vec3_root(position)
            == source.roots[AL_V2_POSITION_ROOT]
        && al_r10_dual_root(dual) == source.roots[AL_V2_DUAL_ROOT]
        && al_binary64_vec3_root(predicted)
            == source.roots[AL_V2_PREDICTED_ROOT]
        && r17_state_root
            == "d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54";
    const bool passed = identity_exact && all_positive && cases_exact
        && route_counts_exact && input_rollback_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "D7R19R17_PARENT_BYTES";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!grant_exact || !transition_receipt_exact) {
        first_failure = "GRANT_CHAIN";
    } else if (!active_owner_exact || !r17_state_exact) {
        first_failure = "ACTIVE_OWNER";
    } else if (!resource_prework_exact) first_failure = "RESOURCE_PREWORK";
    else if (!candidate_valid) first_failure = "CANDIDATE";
    else if (!oracle_valid) first_failure = "ORACLE";
    else if (!oracle_exact) first_failure = "ORACLE_MISMATCH";
    else if (!consumed_owner_exact) first_failure = "OWNER_CONSUME";
    else if (!receipt_exact) first_failure = "RECEIPT";
    else if (!resume_state_exact) first_failure = "RESUME_STATE";
    else if (!cases_exact) first_failure = "NEGATIVE_ROUTE_OR_ROLLBACK";
    else if (!route_counts_exact) first_failure = "ROUTE_COUNTS";
    else if (!input_rollback_exact) first_failure = "INPUT_ROLLBACK";
    const std::string route = passed
        ? "SHADOW_OUTER6_CONTINUATION_CANDIDATE" : std::string{};

    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << B4E2D7R19R18_IDENTITY_SHA256 << '|'
             << r17_stdout_sha256 << ':' << r17_semantic_exact << ':'
             << parents_retained << '|' << r17_state_root << ':'
             << grant_root << ':' << sha256_hex(active_owner_bytes) << '|'
             << candidate.update_root << ':' << candidate.work_root << ':'
             << oracle.update_root << ':' << oracle.work_root << ':'
             << oracle_exact << '|'
             << candidate.delta_hvp << ':' << candidate.delta_workspaces
             << ':' << candidate.delta_precision << ':'
             << candidate.budget.inner_trials_in_update << '|'
             << position_root << ':' << dual_root << ':'
             << successor_history << '|' << consumed_active_owner_root << ':'
             << receipt_root << ':' << resume_state_root << '|'
             << cases.size() << ':' << case_root << ':' << cases_exact << ':'
             << route_counts_exact << ':' << input_rollback_exact << '|'
             << route;
    const std::string result_sha256 = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(std::numeric_limits<double>::max_digits10)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r18_shadow_outer6_resume.v1\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R18_IDENTITY_SHA256
           << "\",\"status\":\"" << (passed ? "PASS" : "FAIL")
           << "\",\"first_failure\":\"" << first_failure << '"'
           << ",\"parent\":{\"r17_stdout_sha256\":\""
           << r17_stdout_sha256 << "\",\"r17_semantic_exact\":"
           << (r17_semantic_exact ? "true" : "false")
           << ",\"transitive_parents_retained\":"
           << (parents_retained ? "true" : "false")
           << ",\"exact\":" << (parent_exact ? "true" : "false")
           << "},\"chain\":{\"source_exact\":"
           << (source_exact ? "true" : "false")
           << ",\"grant_exact\":" << (grant_exact ? "true" : "false")
           << ",\"transition_receipt_exact\":"
           << (transition_receipt_exact ? "true" : "false")
           << ",\"active_owner_exact\":"
           << (active_owner_exact ? "true" : "false")
           << ",\"r17_state_root\":\"" << r17_state_root
           << "\"},\"candidate\":{\"update_root\":\""
           << candidate.update_root << "\",\"work_root\":\""
           << candidate.work_root << "\",\"passed\":"
           << (candidate_valid ? "true" : "false")
           << ",\"trials\":" << candidate.update.trials.size()
           << ",\"inner_accepted\":"
           << candidate.update.state.inner_accepted
           << ",\"inner_rejected\":"
           << candidate.update.state.inner_rejected
           << ",\"hvp_delta\":" << candidate.delta_hvp
           << ",\"workspace_delta\":" << candidate.delta_workspaces
           << ",\"precision_delta\":" << candidate.delta_precision
           << ",\"primal\":" << candidate.update.state.primal
           << ",\"stationarity\":"
           << candidate.update.state.stationarity
           << ",\"admissible\":"
           << (candidate.update.state.admissible ? "true" : "false")
           << "},\"oracle\":{\"update_root\":\""
           << oracle.update_root << "\",\"work_root\":\""
           << oracle.work_root << "\",\"passed\":"
           << (oracle_valid ? "true" : "false")
           << ",\"exact\":" << (oracle_exact ? "true" : "false")
           << "},\"successor\":{\"position_root\":\""
           << position_root << "\",\"dual_root\":\"" << dual_root
           << "\",\"history_root\":\"" << successor_history
           << "\",\"consumed_active_owner_root\":\""
           << consumed_active_owner_root
           << "\",\"receipt_bytes\":" << receipt_bytes.size()
           << ",\"receipt_root\":\"" << receipt_root
           << "\",\"state_bytes\":" << resume_state_bytes.size()
           << ",\"state_root\":\"" << resume_state_root
           << "\",\"canonical_exact\":"
           << (receipt_exact && resume_state_exact ? "true" : "false")
           << ",\"used\":[";
    for (std::size_t index = 0U; index < successor_used.size(); ++index) {
        if (index != 0U) report << ',';
        report << successor_used[index];
    }
    report << "]},\"corpus\":{\"cases\":" << cases.size()
           << ",\"case_root\":\"" << case_root
           << "\",\"all_exact\":" << (cases_exact ? "true" : "false")
           << ",\"route_counts\":[";
    for (std::size_t index = 0U; index < route_counts.size(); ++index) {
        if (index != 0U) report << ',';
        report << route_counts[index];
    }
    report << "],\"route_counts_exact\":"
           << (route_counts_exact ? "true" : "false")
           << "},\"input_rollback_exact\":"
           << (input_rollback_exact ? "true" : "false")
           << ",\"route\":\"" << route << '"'
           << ",\"control_parent_substeps\":1"
              ",\"candidate_outer_updates\":1"
              ",\"oracle_outer_updates\":1"
           << ",\"shadow_commit_count\":" << (passed ? 1 : 0)
           << ",\"outer7_executed\":false"
              ",\"second_substep_executed\":false"
              ",\"macro_frames\":0,\"trajectory_steps\":0"
              ",\"public_schema_created\":false"
              ",\"public_commit_count\":0"
              ",\"world_physics_mutation\":false"
              ",\"live_budget_code_changed\":false"
              ",\"production_policy_changed\":false"
              ",\"durable_cas\":false,\"concurrent_cas\":false"
              ",\"timing_admitted\":false,\"speedup_claim\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha256 << "\"}";
    if (capture_out != nullptr) {
        capture_out->captured = passed;
        capture_out->r17 = std::move(capture);
        capture_out->candidate = candidate;
        capture_out->receipt = receipt;
        capture_out->state = resume_state;
        capture_out->receipt_root = receipt_root;
        capture_out->state_root = resume_state_root;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_shadow_outer6_resume_controls() {
    return run_al_shadow_outer6_resume_controls_impl(nullptr);
}
