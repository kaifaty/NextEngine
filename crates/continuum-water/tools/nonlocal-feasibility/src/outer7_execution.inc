namespace {

constexpr const char* B4E2D7R19R20_IDENTITY_SHA256 =
    "a9a172f787195595c948259a73d658bb23894b99a7b1ac4ceefae83afb39e43e";
constexpr const char* B4E2D7R19R20_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r20-shadow-outer7-execution|v1|parent=3a4601fb:0b002ee449562c08b3be84bc94eff219c72d3c2afdf41490d7b65a1fddaffb24:e49c1fa1a3f8621c289bb75c7bc470a72338d3433ab94f49750f87d15d421b97|source=r18-state=5ad2f99d356c9f5f7553d84189e1e279dcd360bdb9a588370a874b2c5fe707bc;r18-receipt=5a029f24a82b60117492f1929a2d63489d2321b63286a9260282101b74ef8a64;r19-state=37d1556efa98ceb68db566a6f553d82295fc6b233fcbfb0aa6d393a96a4ef6b5;grant=cb1b13d53754e7b1408d66eeb0a00f7dc02a697ccd338d8d77834a6f8b1a6776;grant-receipt=28b5b27673dc9ed247957c159f7c4ac071a074902c918a273327a0d0a40b513a;active-owner=f88ea265a2e2544b76ae335770bb548da6aef76c7a3883ff7ea87e62e6f92ac4;history=a36fa9c02821f1eac2785aa0ec35632f088810b3be625303271d2b9750b8a5b1;position=494dc8d29030f06b79b7047a82ce83eb994e863541becc37f60d3343f08ff29f;dual=9ec049a088270e211435a5cc16b4fea23826c261145209cb28748b8dec92c778;epoch1;outer7;slice50;cumulative573|owner-consume=flags1-to3;consumed-root=94df43022a9b8fd28e00f926db066e065bd8fa17ff65598b01dfe48e11e47f8a|candidate=slice-budget512;total50;available462;oracle=unsliced-budget8704;total573;available8131;both-one-outer7;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=7,2,25,1,50,573,37,23,552,21,2,23,0|bounds=outer1;inner<=16;new-hvp<=462;new-workspaces<=251;new-precision<=41;cumulative-new-hvp<=8131|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer7-parent-rejected;shadow-outer7-source-rejected;shadow-outer7-grant-rejected;shadow-outer7-owner-rejected;shadow-outer7-duplicate-rejected;shadow-outer7-stale-epoch-rejected;shadow-outer7-resource-rejected;shadow-outer7-candidate-rejected;shadow-outer7-oracle-rejected;shadow-outer7-oracle-mismatch;shadow-outer7-aborted;shadow-outer7-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer7-execution-candidate-only)IDENTITY";

constexpr std::uint32_t AL_OUTER_EXECUTION_FLAGS = 3U;
constexpr std::uint32_t AL_OUTER_EXECUTION_STATE_FLAGS = 31U;

struct ALOuterExecutionReceiptV1 {
    std::uint32_t flags = AL_OUTER_EXECUTION_FLAGS;
    std::uint64_t epoch = 1U;
    std::uint64_t outer = 7U;
    std::array<std::string, 7U> roots;
    std::array<std::uint64_t, AL_V2_LIMIT_COUNT> limits{};
    std::array<std::uint64_t, AL_V2_USED_COUNT> used{};
};

struct ALOuterExecutionStateV1 {
    std::uint32_t flags = AL_OUTER_EXECUTION_STATE_FLAGS;
    std::array<std::string, 6U> roots;
};

bool al_encode_outer_execution_receipt(
    const ALOuterExecutionReceiptV1& value, std::string& bytes) {
    std::string body;
    al_append_u32_le(body, value.flags);
    al_append_u64_le(body, value.epoch);
    al_append_u64_le(body, value.outer);
    for (const auto& root : value.roots) {
        if (!al_append_digest_bytes(body, root)) return false;
    }
    for (auto limit : value.limits) al_append_u64_le(body, limit);
    for (auto used : value.used) al_append_u64_le(body, used);
    return al_encode_canonical_v1("NEALOER1", 404U, 420U, body, bytes);
}

bool al_decode_outer_execution_receipt(
    const std::string& bytes, ALOuterExecutionReceiptV1& value) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALOER1", 404U, 420U, offset)
        || !al_read_u32_le(bytes, offset, value.flags)
        || value.flags != AL_OUTER_EXECUTION_FLAGS
        || !al_read_u64_le(bytes, offset, value.epoch)
        || !al_read_u64_le(bytes, offset, value.outer)) return false;
    for (auto& root : value.roots) {
        if (!al_read_digest_hex(bytes, offset, root)) return false;
    }
    for (auto& limit : value.limits) {
        if (!al_read_u64_le(bytes, offset, limit)) return false;
    }
    for (auto& used : value.used) {
        if (!al_read_u64_le(bytes, offset, used)) return false;
    }
    if (offset != bytes.size()) return false;
    std::string reencoded;
    return al_encode_outer_execution_receipt(value, reencoded)
        && reencoded == bytes;
}

bool al_encode_outer_execution_state(
    const ALOuterExecutionStateV1& value, std::string& bytes) {
    std::string body;
    al_append_u32_le(body, value.flags);
    for (const auto& root : value.roots) {
        if (!al_append_digest_bytes(body, root)) return false;
    }
    return al_encode_canonical_v1("NEALOES1", 196U, 212U, body, bytes);
}

bool al_decode_outer_execution_state(
    const std::string& bytes, ALOuterExecutionStateV1& value) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALOES1", 196U, 212U, offset)
        || !al_read_u32_le(bytes, offset, value.flags)
        || value.flags != AL_OUTER_EXECUTION_STATE_FLAGS) return false;
    for (auto& root : value.roots) {
        if (!al_read_digest_hex(bytes, offset, root)) return false;
    }
    if (offset != bytes.size()) return false;
    std::string reencoded;
    return al_encode_outer_execution_state(value, reencoded)
        && reencoded == bytes;
}

bool al_outer7_execution_valid(
    const ALOuter6Execution& value,
    std::size_t base_hvp,
    std::size_t maximum_hvp,
    const std::string& expected_static_identity) {
    std::size_t recorded_hvp = 0U;
    std::size_t accepted = 0U;
    std::size_t rejected = 0U;
    for (const auto& trial : value.update.trials) {
        recorded_hvp += static_cast<std::size_t>(trial.hvp_calls);
        accepted += trial.candidate_accepted ? 1U : 0U;
        rejected += trial.candidate_accepted ? 0U : 1U;
    }
    return value.update.passed && value.update.state.outer == 7U
        && value.update.state.finite_values
        && !value.update.precision_hard_failure
        && !value.update.sign_contradiction
        && !value.update.oracle_bound_required
        && !value.budget.exhausted && value.budget.failure.empty()
        && value.budget.outer_updates == 8U
        && value.budget.inner_trials_in_update <= 16U
        && value.delta_hvp <= maximum_hvp - base_hvp
        && value.delta_workspaces <= 251U
        && value.delta_precision <= 41U
        && value.budget.total_hvp == base_hvp + value.delta_hvp
        && value.budget.workspace_builds == 37U + value.delta_workspaces
        && value.budget.precision_audits == 23U + value.delta_precision
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
        && recorded_hvp == value.completion_trace.recurrence_hvp
            + value.completion_trace.direct_model_hvp
        && accepted + rejected == value.update.trials.size()
        && value.delta_precision
            == value.precision_work.long_double_audits
                + value.precision_work.binary128_audits;
}

std::array<std::uint64_t, AL_V2_USED_COUNT> al_outer7_successor_used(
    const std::array<std::uint64_t, AL_V2_USED_COUNT>& baseline,
    const ALOuter6Execution& execution) {
    auto used = baseline;
    std::uint64_t accepted = 0U;
    std::uint64_t rejected = 0U;
    std::uint64_t residual_models = 0U;
    for (const auto& trial : execution.update.trials) {
        accepted += trial.candidate_accepted ? 1U : 0U;
        rejected += trial.candidate_accepted ? 0U : 1U;
        residual_models += trial.residual_model_used ? 1U : 0U;
    }
    used[AL_V2_USED_OUTER] = 8U;
    used[AL_V2_USED_INNER] = execution.budget.inner_trials_in_update;
    used[AL_V2_USED_LAST_TRIAL_HVP] = execution.update.trials.empty() ? 0U
        : static_cast<std::uint64_t>(execution.update.trials.back().hvp_calls);
    used[AL_V2_USED_SLICE_HVP] += execution.delta_hvp;
    used[AL_V2_USED_CUMULATIVE_HVP] += execution.delta_hvp;
    used[AL_V2_USED_WORKSPACES] += execution.delta_workspaces;
    used[AL_V2_USED_PRECISION] += execution.delta_precision;
    used[AL_V2_USED_RECURRENCE_HVP] +=
        execution.completion_trace.recurrence_hvp;
    used[AL_V2_USED_DIRECT_MODEL_HVP] +=
        execution.completion_trace.direct_model_hvp;
    used[AL_V2_USED_RESIDUAL_MODELS] += residual_models;
    used[AL_V2_USED_ACCEPTED] += accepted;
    used[AL_V2_USED_REJECTED] += rejected;
    return used;
}

std::string al_shadow_outer7_route(
    bool parent, bool source, bool grant, bool owner, bool duplicate,
    bool stale, bool resource, bool candidate, bool oracle, bool match,
    bool abort) {
    if (!parent) return "SHADOW_OUTER7_PARENT_REJECTED";
    if (!source) return "SHADOW_OUTER7_SOURCE_REJECTED";
    if (!grant) return "SHADOW_OUTER7_GRANT_REJECTED";
    if (!owner) return "SHADOW_OUTER7_OWNER_REJECTED";
    if (duplicate) return "SHADOW_OUTER7_DUPLICATE_REJECTED";
    if (stale) return "SHADOW_OUTER7_STALE_EPOCH_REJECTED";
    if (!resource) return "SHADOW_OUTER7_RESOURCE_REJECTED";
    if (!candidate) return "SHADOW_OUTER7_CANDIDATE_REJECTED";
    if (!oracle) return "SHADOW_OUTER7_ORACLE_REJECTED";
    if (!match) return "SHADOW_OUTER7_ORACLE_MISMATCH";
    if (abort) return "SHADOW_OUTER7_ABORTED";
    return "SHADOW_OUTER7_EXECUTION_CANDIDATE";
}

struct ALOuter7TransactionState {
    std::string active_owner_bytes;
    std::string receipt_bytes;
    std::string state_bytes;
};

std::string al_outer7_transaction_bytes(
    const ALOuter7TransactionState& value) {
    std::string bytes;
    for (const auto* field : {&value.active_owner_bytes,
            &value.receipt_bytes, &value.state_bytes}) {
        al_append_u64_le(bytes, static_cast<std::uint64_t>(field->size()));
        bytes.append(*field);
    }
    return bytes;
}

} // namespace

SplitBoundaryReport run_al_shadow_outer7_execution_controls() {
    const bool identity_exact = sha256_hex(B4E2D7R19R20_IDENTITY_PROJECTION)
        == B4E2D7R19R20_IDENTITY_SHA256;
    ALWithinEpochOuter7GrantCapture capture;
    const SplitBoundaryReport parent =
        run_al_within_epoch_outer7_grant_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "e49c1fa1a3f8621c289bb75c7bc470a72338d3433ab94f49750f87d15d421b97"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "0b002ee449562c08b3be84bc94eff219c72d3c2afdf41490d7b65a1fddaffb24";

    const auto& r18 = capture.r18;
    const auto& r17 = r18.r17;
    const ALV2EnvelopeCapture& r16 = r17.r16;
    const ALSoftCapSuspensionCapture& r14 = r16.r15.r14;
    const ALNormalizedTotalHvpBoundaryCapture& boundary =
        r14.r13.r12.r11.r10.total_hvp_boundary;
    const std::vector<Vec3>& predicted = boundary.predicted_position;
    const std::vector<Vec3>& position = r18.candidate.update.position;
    const std::vector<double>& dual = r18.candidate.update.u;
    const bool source_exact = r18.state_root
            == "5ad2f99d356c9f5f7553d84189e1e279dcd360bdb9a588370a874b2c5fe707bc"
        && r18.receipt_root
            == "5a029f24a82b60117492f1929a2d63489d2321b63286a9260282101b74ef8a64"
        && capture.state_root
            == "37d1556efa98ceb68db566a6f553d82295fc6b233fcbfb0aa6d393a96a4ef6b5"
        && capture.grant_root
            == "cb1b13d53754e7b1408d66eeb0a00f7dc02a697ccd338d8d77834a6f8b1a6776"
        && capture.receipt_root
            == "28b5b27673dc9ed247957c159f7c4ac071a074902c918a273327a0d0a40b513a"
        && capture.active_owner_root
            == "f88ea265a2e2544b76ae335770bb548da6aef76c7a3883ff7ea87e62e6f92ac4"
        && al_binary64_vec3_root(position)
            == "494dc8d29030f06b79b7047a82ce83eb994e863541becc37f60d3343f08ff29f"
        && al_r10_dual_root(dual)
            == "9ec049a088270e211435a5cc16b4fea23826c261145209cb28748b8dec92c778"
        && capture.grant.roots[2]
            == "a36fa9c02821f1eac2785aa0ec35632f088810b3be625303271d2b9750b8a5b1";
    const bool resource_exact = capture.grant.limits
            == std::array<std::uint64_t, AL_V2_LIMIT_COUNT>{
                16U,16U,34U,512U,288U,64U,8704U}
        && capture.grant.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                7U,2U,25U,1U,50U,573U,37U,23U,552U,21U,2U,23U,0U};

    ALOuter6Execution candidate;
    ALOuter6Execution oracle;
    if (parent_exact && source_exact && resource_exact) {
        candidate = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 7U,
            50U, 512U, 37U, 23U);
        oracle = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 7U,
            573U, 8704U, 37U, 23U);
    }
    const std::string& static_root = r16.envelope.roots[AL_V2_STATIC_ROOT];
    const bool candidate_valid = al_outer7_execution_valid(
        candidate, 50U, 512U, static_root);
    const bool oracle_valid = al_outer7_execution_valid(
        oracle, 573U, 8704U, static_root);
    const bool oracle_exact = candidate_valid && oracle_valid
        && al_outer6_execution_exact(candidate, oracle);

    const auto successor_used = al_outer7_successor_used(
        capture.grant.used, candidate);
    const std::string position_root =
        al_binary64_vec3_root(candidate.update.position);
    const std::string dual_root = al_r10_dual_root(candidate.update.u);
    const std::string history_root = al_outer6_successor_history_root(
        capture.grant.roots[2], candidate.update_root,
        position_root, dual_root, successor_used);

    ALOuterGrantOwnerV1 consumed_owner = capture.active_owner;
    consumed_owner.flags = 3U;
    std::string consumed_owner_bytes;
    const bool consumed_owner_exact = al_encode_outer_grant_owner(
            consumed_owner, consumed_owner_bytes)
        && sha256_hex(consumed_owner_bytes)
            == "94df43022a9b8fd28e00f926db066e065bd8fa17ff65598b01dfe48e11e47f8a";
    const std::string consumed_owner_root = sha256_hex(consumed_owner_bytes);

    ALOuterExecutionReceiptV1 receipt;
    receipt.roots = {capture.state_root, capture.grant_root,
        consumed_owner_root, candidate.update_root, position_root,
        dual_root, history_root};
    receipt.limits = capture.grant.limits;
    receipt.used = successor_used;
    std::string receipt_bytes;
    ALOuterExecutionReceiptV1 decoded_receipt;
    const bool receipt_exact = oracle_exact && consumed_owner_exact
        && al_encode_outer_execution_receipt(receipt, receipt_bytes)
        && al_decode_outer_execution_receipt(receipt_bytes, decoded_receipt)
        && decoded_receipt.flags == receipt.flags
        && decoded_receipt.epoch == receipt.epoch
        && decoded_receipt.outer == receipt.outer
        && decoded_receipt.roots == receipt.roots
        && decoded_receipt.limits == receipt.limits
        && decoded_receipt.used == receipt.used;
    const std::string receipt_root = receipt_exact
        ? sha256_hex(receipt_bytes) : std::string{};

    ALOuterExecutionStateV1 state;
    state.roots = {r18.state_root, capture.state_root, capture.grant_root,
        capture.receipt_root, consumed_owner_root, receipt_root};
    std::string state_bytes;
    ALOuterExecutionStateV1 decoded_state;
    const bool state_exact = receipt_exact
        && al_encode_outer_execution_state(state, state_bytes)
        && al_decode_outer_execution_state(state_bytes, decoded_state)
        && decoded_state.flags == state.flags
        && decoded_state.roots == state.roots;
    const std::string state_root = state_exact
        ? sha256_hex(state_bytes) : std::string{};

    const ALOuter7TransactionState initial{
        capture.active_owner_bytes, {}, capture.state_bytes};
    const ALOuter7TransactionState committed{
        consumed_owner_bytes, receipt_bytes, state_bytes};
    const std::string initial_bytes = al_outer7_transaction_bytes(initial);
    const std::string committed_bytes = al_outer7_transaction_bytes(committed);
    struct Case {
        const char* name;
        const char* expected;
        std::string observed;
        bool state_exact = false;
    };
    std::vector<Case> cases;
    const auto add = [&](const char* name, const char* expected,
        bool p, bool s, bool g, bool o, bool duplicate, bool stale,
        bool resource, bool c, bool oracle_ok, bool match, bool abort,
        bool replay = false) {
        ALOuter7TransactionState transaction = replay ? committed : initial;
        const std::string before = al_outer7_transaction_bytes(transaction);
        const std::string observed = al_shadow_outer7_route(
            p,s,g,o,duplicate,stale,resource,c,oracle_ok,match,abort);
        if (observed == "SHADOW_OUTER7_EXECUTION_CANDIDATE") {
            transaction = committed;
        }
        const std::string after = al_outer7_transaction_bytes(transaction);
        const bool exact = observed == "SHADOW_OUTER7_EXECUTION_CANDIDATE"
            ? after == committed_bytes && before == initial_bytes
            : after == before;
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","SHADOW_OUTER7_EXECUTION_CANDIDATE",1,1,1,1,0,0,1,1,1,1,0);
    add("parent","SHADOW_OUTER7_PARENT_REJECTED",0,1,1,1,0,0,1,1,1,1,0);
    add("source","SHADOW_OUTER7_SOURCE_REJECTED",1,0,1,1,0,0,1,1,1,1,0);
    add("grant","SHADOW_OUTER7_GRANT_REJECTED",1,1,0,1,0,0,1,1,1,1,0);
    add("owner","SHADOW_OUTER7_OWNER_REJECTED",1,1,1,0,0,0,1,1,1,1,0);
    add("duplicate","SHADOW_OUTER7_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0);
    add("stale","SHADOW_OUTER7_STALE_EPOCH_REJECTED",1,1,1,1,0,1,1,1,1,1,0);
    add("resource","SHADOW_OUTER7_RESOURCE_REJECTED",1,1,1,1,0,0,0,1,1,1,0);
    add("candidate","SHADOW_OUTER7_CANDIDATE_REJECTED",1,1,1,1,0,0,1,0,1,1,0);
    add("oracle","SHADOW_OUTER7_ORACLE_REJECTED",1,1,1,1,0,0,1,1,0,1,0);
    add("mismatch","SHADOW_OUTER7_ORACLE_MISMATCH",1,1,1,1,0,0,1,1,1,0,0);
    add("abort","SHADOW_OUTER7_ABORTED",1,1,1,1,0,0,1,1,1,1,1);
    add("duplicate-replay","SHADOW_OUTER7_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0,1);
    std::ostringstream corpus;
    bool cases_exact = cases.size() == 13U;
    std::string first_case_failure;
    for (const auto& value : cases) {
        const bool exact = value.observed == value.expected
            && value.state_exact;
        cases_exact = cases_exact && exact;
        if (!exact && first_case_failure.empty()) first_case_failure = value.name;
        corpus << value.name << ':' << value.expected << ':'
               << value.observed << ':' << value.state_exact << '|';
    }
    const std::string case_root = sha256_hex(corpus.str());
    const bool input_rollback_exact =
        al_binary64_vec3_root(position)
            == "494dc8d29030f06b79b7047a82ce83eb994e863541becc37f60d3343f08ff29f"
        && al_r10_dual_root(dual)
            == "9ec049a088270e211435a5cc16b4fea23826c261145209cb28748b8dec92c778";
    const bool passed = identity_exact && parent_exact && source_exact
        && resource_exact && candidate_valid && oracle_valid && oracle_exact
        && consumed_owner_exact && receipt_exact && state_exact
        && cases_exact && input_rollback_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!resource_exact) first_failure = "RESOURCE_PREWORK";
    else if (!candidate_valid) first_failure = "CANDIDATE";
    else if (!oracle_valid) first_failure = "ORACLE";
    else if (!oracle_exact) first_failure = "ORACLE_MISMATCH";
    else if (!consumed_owner_exact) first_failure = "OWNER_CONSUME";
    else if (!receipt_exact) first_failure = "RECEIPT";
    else if (!state_exact) first_failure = "STATE";
    else if (!cases_exact) first_failure = first_case_failure;
    else if (!input_rollback_exact) first_failure = "INPUT_ROLLBACK";
    const std::string route = passed
        ? "SHADOW_OUTER7_EXECUTION_CANDIDATE" : std::string{};
    std::size_t accepted = 0U;
    std::size_t rejected = 0U;
    for (const auto& trial : candidate.update.trials) {
        accepted += trial.candidate_accepted ? 1U : 0U;
        rejected += trial.candidate_accepted ? 0U : 1U;
    }
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R20_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << candidate.update_root << ':' << candidate.work_root << ':'
             << oracle.update_root << ':' << oracle.work_root << ':'
             << oracle_exact << '|' << candidate.delta_hvp << ':'
             << candidate.delta_workspaces << ':' << candidate.delta_precision
             << '|' << position_root << ':' << dual_root << ':' << history_root
             << '|' << consumed_owner_root << ':' << receipt_root << ':'
             << state_root << '|' << case_root << ':' << cases_exact << '|'
             << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r20_shadow_outer7_execution.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R20_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"candidate\":{\"update_root\":\"" << candidate.update_root
           << "\",\"work_root\":\"" << candidate.work_root
           << "\",\"valid\":" << (candidate_valid ? "true" : "false")
           << ",\"trials\":" << candidate.update.trials.size()
           << ",\"accepted\":" << accepted << ",\"rejected\":" << rejected
           << ",\"hvp_delta\":" << candidate.delta_hvp
           << ",\"workspace_delta\":" << candidate.delta_workspaces
           << ",\"precision_delta\":" << candidate.delta_precision
           << ",\"primal\":" << std::setprecision(17)
           << candidate.update.state.primal << ",\"stationarity\":"
           << candidate.update.state.stationarity << ",\"admissible\":"
           << (candidate.update.state.admissible ? "true" : "false") << "}"
           << ",\"oracle\":{\"update_root\":\"" << oracle.update_root
           << "\",\"work_root\":\"" << oracle.work_root
           << "\",\"valid\":" << (oracle_valid ? "true" : "false")
           << ",\"exact\":" << (oracle_exact ? "true" : "false") << "}"
           << ",\"position_root\":\"" << position_root << "\""
           << ",\"dual_root\":\"" << dual_root << "\""
           << ",\"history_root\":\"" << history_root << "\""
           << ",\"consumed_owner_root\":\"" << consumed_owner_root << "\""
           << ",\"receipt_root\":\"" << receipt_root << "\""
           << ",\"state_root\":\"" << state_root << "\""
           << ",\"successor_used\":[";
    for (std::size_t i = 0U; i < successor_used.size(); ++i) {
        if (i != 0U) report << ',';
        report << successor_used[i];
    }
    report << "],\"cases\":" << cases.size()
           << ",\"case_root\":\"" << case_root << "\""
           << ",\"cases_exact\":" << (cases_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"candidate_outer_updates\":1,\"oracle_outer_updates\":1"
              ",\"second_substeps\":0,\"macro_frames\":0"
              ",\"trajectory_steps\":0,\"public_commit_count\":0"
              ",\"world_physics_mutation\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {passed, report.str()};
}
