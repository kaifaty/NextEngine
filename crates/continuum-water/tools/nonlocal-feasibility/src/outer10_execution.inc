namespace {

constexpr const char* B4E2D7R19R26_IDENTITY_SHA256 =
    "bef2ffec766fd82708ea62ce4090093bce5b33d2389c76d58dd0473ca2e8a2a5";
constexpr const char* B4E2D7R19R26_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r26-shadow-outer10-execution|v1|parent=610dc8bb:b20fffc6ac40ca1f16feb91e412bf7ed7a5be1430502c8921827662ec206cd69:fc8e0182409785c966c4f1c7f391c910e6a30d3d942335a6eb53b1b3e6fc18a9|source=r24-state=749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98;r24-receipt=9c618bd10b6aeaed9ae9e5fddd9f7287e4df6d7adf3e76e467543c70e8674e28;r25-state=b82cd881e7b50794de029e6829e8575b86630d50b0f4caaf74a7526a3bb66153;grant=f825ede7b3fa52c93b47806f92e40f34eefb93daca54d0b2d3092bf0870eede7;grant-receipt=cc39098c863c2fa614086e3702029b0c04182bdfd501ffbf94753d9bd410a8fa;active-owner=2f6aea4168bdc513f21099f363a9aa28cc9171620f4a7c558b2cc340d6a6f6c6;history=d39b98c23d63217b0145229b593154fa3b49e2ea2b701ba708c51f5f8eb0b28d;position=b59fdcd410ab2612246324deb5417aa4cec908c644e7ba0d970c348ca088861a;dual=69c062bb1ce89250d978dce462b8f733bc88bfd48c1fe880e57290be0e07550a;epoch1;outer10;slice231;cumulative754;source-primal0x3e54e2d2e4000000;source-stationarity0x3dda59ffa2498bd8|owner-consume=flags1-to3;consumed-root=3e956008c7c633d5ab74e249af21f1a4503ae9f5709603cdc763b0047d45e28d|candidate=slice-budget512;total231;available281;oracle=unsliced-budget8704;total754;available7950;both-one-outer10;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=10,2,24,1,231,754,50,30,726,28,2,30,0|bounds=outer1;inner<=16;new-hvp<=281;new-workspaces<=238;new-precision<=34;cumulative-new-hvp<=7950|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model;stationarity-trend-report-only|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer10-parent-rejected;shadow-outer10-source-rejected;shadow-outer10-grant-rejected;shadow-outer10-owner-rejected;shadow-outer10-duplicate-rejected;shadow-outer10-stale-epoch-rejected;shadow-outer10-resource-rejected;shadow-outer10-candidate-rejected;shadow-outer10-oracle-rejected;shadow-outer10-oracle-mismatch;shadow-outer10-aborted;shadow-outer10-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer10-execution-candidate-only)IDENTITY";

std::string al_shadow_outer10_route(
    bool parent, bool source, bool grant, bool owner, bool duplicate,
    bool stale, bool resource, bool candidate, bool oracle, bool match,
    bool abort) {
    if (!parent) return "SHADOW_OUTER10_PARENT_REJECTED";
    if (!source) return "SHADOW_OUTER10_SOURCE_REJECTED";
    if (!grant) return "SHADOW_OUTER10_GRANT_REJECTED";
    if (!owner) return "SHADOW_OUTER10_OWNER_REJECTED";
    if (duplicate) return "SHADOW_OUTER10_DUPLICATE_REJECTED";
    if (stale) return "SHADOW_OUTER10_STALE_EPOCH_REJECTED";
    if (!resource) return "SHADOW_OUTER10_RESOURCE_REJECTED";
    if (!candidate) return "SHADOW_OUTER10_CANDIDATE_REJECTED";
    if (!oracle) return "SHADOW_OUTER10_ORACLE_REJECTED";
    if (!match) return "SHADOW_OUTER10_ORACLE_MISMATCH";
    if (abort) return "SHADOW_OUTER10_ABORTED";
    return "SHADOW_OUTER10_EXECUTION_CANDIDATE";
}

struct ALShadowOuter10ExecutionCapture {
    bool captured = false;
    ALWithinEpochOuter10GrantCapture r25;
    ALOuter6Execution candidate;
    ALOuterExecutionReceiptV1 receipt;
    ALOuterExecutionStateV1 state;
    std::string receipt_bytes;
    std::string state_bytes;
    std::string receipt_root;
    std::string state_root;
};

} // namespace

SplitBoundaryReport run_al_shadow_outer10_execution_controls_impl(
    ALShadowOuter10ExecutionCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R26_IDENTITY_PROJECTION)
        == B4E2D7R19R26_IDENTITY_SHA256;
    ALWithinEpochOuter10GrantCapture capture;
    const SplitBoundaryReport parent =
        run_al_within_epoch_outer10_grant_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "fc8e0182409785c966c4f1c7f391c910e6a30d3d942335a6eb53b1b3e6fc18a9"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "b20fffc6ac40ca1f16feb91e412bf7ed7a5be1430502c8921827662ec206cd69";

    const auto& r24 = capture.r24;
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
    const std::vector<Vec3>& predicted = boundary.predicted_position;
    const std::vector<Vec3>& position = r24.candidate.update.position;
    const std::vector<double>& dual = r24.candidate.update.u;
    const bool source_exact = r24.state_root
            == "749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98"
        && r24.receipt_root
            == "9c618bd10b6aeaed9ae9e5fddd9f7287e4df6d7adf3e76e467543c70e8674e28"
        && capture.state_root
            == "b82cd881e7b50794de029e6829e8575b86630d50b0f4caaf74a7526a3bb66153"
        && capture.grant_root
            == "f825ede7b3fa52c93b47806f92e40f34eefb93daca54d0b2d3092bf0870eede7"
        && capture.receipt_root
            == "cc39098c863c2fa614086e3702029b0c04182bdfd501ffbf94753d9bd410a8fa"
        && capture.active_owner_root
            == "2f6aea4168bdc513f21099f363a9aa28cc9171620f4a7c558b2cc340d6a6f6c6"
        && al_binary64_vec3_root(position)
            == "b59fdcd410ab2612246324deb5417aa4cec908c644e7ba0d970c348ca088861a"
        && al_r10_dual_root(dual)
            == "69c062bb1ce89250d978dce462b8f733bc88bfd48c1fe880e57290be0e07550a"
        && capture.grant.roots[2]
            == "d39b98c23d63217b0145229b593154fa3b49e2ea2b701ba708c51f5f8eb0b28d";
    const bool resource_exact = capture.grant.limits
            == std::array<std::uint64_t, AL_V2_LIMIT_COUNT>{
                16U,16U,34U,512U,288U,64U,8704U}
        && capture.grant.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                10U,2U,24U,1U,231U,754U,50U,30U,726U,28U,2U,30U,0U};

    ALOuter6Execution candidate;
    ALOuter6Execution oracle;
    if (parent_exact && source_exact && resource_exact) {
        candidate = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 10U,
            231U, 512U, 50U, 30U);
        oracle = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 10U,
            754U, 8704U, 50U, 30U);
    }
    const std::string& static_root = r16.envelope.roots[AL_V2_STATIC_ROOT];
    const bool candidate_valid = al_outer7_execution_valid(
        candidate, 231U, 512U, static_root, 10U, 50U, 30U, 238U, 34U);
    const bool oracle_valid = al_outer7_execution_valid(
        oracle, 754U, 8704U, static_root, 10U, 50U, 30U, 238U, 34U);
    const bool oracle_exact = candidate_valid && oracle_valid
        && al_outer6_execution_exact(candidate, oracle);

    const auto successor_used = al_outer7_successor_used(
        capture.grant.used, candidate, 11U);
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
            == "3e956008c7c633d5ab74e249af21f1a4503ae9f5709603cdc763b0047d45e28d";
    const std::string consumed_owner_root = sha256_hex(consumed_owner_bytes);

    ALOuterExecutionReceiptV1 receipt;
    receipt.outer = 10U;
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
    state.roots = {r24.state_root, capture.state_root, capture.grant_root,
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
        const std::string observed = al_shadow_outer10_route(
            p,s,g,o,duplicate,stale,resource,c,oracle_ok,match,abort);
        if (observed == "SHADOW_OUTER10_EXECUTION_CANDIDATE") {
            transaction = committed;
        }
        const std::string after = al_outer7_transaction_bytes(transaction);
        const bool exact = observed == "SHADOW_OUTER10_EXECUTION_CANDIDATE"
            ? after == committed_bytes && before == initial_bytes
            : after == before;
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","SHADOW_OUTER10_EXECUTION_CANDIDATE",1,1,1,1,0,0,1,1,1,1,0);
    add("parent","SHADOW_OUTER10_PARENT_REJECTED",0,1,1,1,0,0,1,1,1,1,0);
    add("source","SHADOW_OUTER10_SOURCE_REJECTED",1,0,1,1,0,0,1,1,1,1,0);
    add("grant","SHADOW_OUTER10_GRANT_REJECTED",1,1,0,1,0,0,1,1,1,1,0);
    add("owner","SHADOW_OUTER10_OWNER_REJECTED",1,1,1,0,0,0,1,1,1,1,0);
    add("duplicate","SHADOW_OUTER10_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0);
    add("stale","SHADOW_OUTER10_STALE_EPOCH_REJECTED",1,1,1,1,0,1,1,1,1,1,0);
    add("resource","SHADOW_OUTER10_RESOURCE_REJECTED",1,1,1,1,0,0,0,1,1,1,0);
    add("candidate","SHADOW_OUTER10_CANDIDATE_REJECTED",1,1,1,1,0,0,1,0,1,1,0);
    add("oracle","SHADOW_OUTER10_ORACLE_REJECTED",1,1,1,1,0,0,1,1,0,1,0);
    add("mismatch","SHADOW_OUTER10_ORACLE_MISMATCH",1,1,1,1,0,0,1,1,1,0,0);
    add("abort","SHADOW_OUTER10_ABORTED",1,1,1,1,0,0,1,1,1,1,1);
    add("duplicate-replay","SHADOW_OUTER10_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0,1);
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
            == "b59fdcd410ab2612246324deb5417aa4cec908c644e7ba0d970c348ca088861a"
        && al_r10_dual_root(dual)
            == "69c062bb1ce89250d978dce462b8f733bc88bfd48c1fe880e57290be0e07550a";
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
        ? "SHADOW_OUTER10_EXECUTION_CANDIDATE" : std::string{};
    std::size_t accepted = 0U;
    std::size_t rejected = 0U;
    for (const auto& trial : candidate.update.trials) {
        accepted += trial.candidate_accepted ? 1U : 0U;
        rejected += trial.candidate_accepted ? 0U : 1U;
    }
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R26_IDENTITY_SHA256 << '|' << parent_stdout << '|'
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
              "nsr3b4e2d7r19r26_shadow_outer10_execution.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R26_IDENTITY_SHA256 << "\""
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
    if (capture_out != nullptr) {
        capture_out->captured = passed;
        capture_out->r25 = std::move(capture);
        capture_out->candidate = candidate;
        capture_out->receipt = receipt;
        capture_out->state = state;
        capture_out->receipt_bytes = receipt_bytes;
        capture_out->state_bytes = state_bytes;
        capture_out->receipt_root = receipt_root;
        capture_out->state_root = state_root;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_shadow_outer10_execution_controls() {
    return run_al_shadow_outer10_execution_controls_impl(nullptr);
}
