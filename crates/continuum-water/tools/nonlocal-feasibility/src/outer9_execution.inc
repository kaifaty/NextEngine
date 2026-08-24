namespace {

constexpr const char* B4E2D7R19R24_IDENTITY_SHA256 =
    "69a60aefa8ea6b080acf1235815f5fc8d38492644031a7bfa6f246f0768f72a1";
constexpr const char* B4E2D7R19R24_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r24-shadow-outer9-execution|v1|parent=1f177bb7:c1aca629743893dc973112f9ced8262728e641084cc172e09bfe8ce5bf467a01:27e37a5a05689e8483956038cbc63ce24ded6d0307c41b7e9e86a44c0c42bfa8|source=r22-state=dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9;r22-receipt=96975d4dda71d7490e91ccccfa95c51de480c1bcd3191367f527d5b001c2fa8e;r23-state=b9b8a2d3f0827d9a66a54aeb24f70c674015b8d968d8aaae16189e4adc72ef82;grant=4dd8744547f991b40e5871a4c1823c3e073952dc7b51d6c28acabd92cebf5d18;grant-receipt=16b216ea8c7748ee80a5551f3baf5ee8f3afedea13fe5b88c6283cf07f7de285;active-owner=bd3b2356eb10bf422ae07c16ba9433a63a63da1ccd7e567dcdd3dd3bc4fcee81;history=fa9e2db82287c5f3faac6d8c8031c55bab78230fd708479a569329152c74be7c;position=68d0b821be6f1bdd71f70bf570035be7713cd578fe08774644cbd7c53c672b0f;dual=fbbe1973290706320cbb5f3f3280b6e62b493e3abad4d280ac2985951b7568f3;epoch1;outer9;slice182;cumulative705|owner-consume=flags1-to3;consumed-root=3f4b2fb5d84b3f27260e53e344d5141530b8105f577483aca7a435ec85372522|candidate=slice-budget512;total182;available330;oracle=unsliced-budget8704;total705;available7999;both-one-outer9;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=9,3,32,1,182,705,46,28,679,26,2,28,0|bounds=outer1;inner<=16;new-hvp<=330;new-workspaces<=242;new-precision<=36;cumulative-new-hvp<=7999|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer9-parent-rejected;shadow-outer9-source-rejected;shadow-outer9-grant-rejected;shadow-outer9-owner-rejected;shadow-outer9-duplicate-rejected;shadow-outer9-stale-epoch-rejected;shadow-outer9-resource-rejected;shadow-outer9-candidate-rejected;shadow-outer9-oracle-rejected;shadow-outer9-oracle-mismatch;shadow-outer9-aborted;shadow-outer9-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer9-execution-candidate-only)IDENTITY";

std::string al_shadow_outer9_route(
    bool parent, bool source, bool grant, bool owner, bool duplicate,
    bool stale, bool resource, bool candidate, bool oracle, bool match,
    bool abort) {
    if (!parent) return "SHADOW_OUTER9_PARENT_REJECTED";
    if (!source) return "SHADOW_OUTER9_SOURCE_REJECTED";
    if (!grant) return "SHADOW_OUTER9_GRANT_REJECTED";
    if (!owner) return "SHADOW_OUTER9_OWNER_REJECTED";
    if (duplicate) return "SHADOW_OUTER9_DUPLICATE_REJECTED";
    if (stale) return "SHADOW_OUTER9_STALE_EPOCH_REJECTED";
    if (!resource) return "SHADOW_OUTER9_RESOURCE_REJECTED";
    if (!candidate) return "SHADOW_OUTER9_CANDIDATE_REJECTED";
    if (!oracle) return "SHADOW_OUTER9_ORACLE_REJECTED";
    if (!match) return "SHADOW_OUTER9_ORACLE_MISMATCH";
    if (abort) return "SHADOW_OUTER9_ABORTED";
    return "SHADOW_OUTER9_EXECUTION_CANDIDATE";
}

struct ALShadowOuter9ExecutionCapture {
    bool captured = false;
    ALWithinEpochOuter9GrantCapture r23;
    ALOuter6Execution candidate;
    ALOuterExecutionReceiptV1 receipt;
    ALOuterExecutionStateV1 state;
    std::string receipt_bytes;
    std::string state_bytes;
    std::string receipt_root;
    std::string state_root;
};

} // namespace

SplitBoundaryReport run_al_shadow_outer9_execution_controls_impl(
    ALShadowOuter9ExecutionCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R24_IDENTITY_PROJECTION)
        == B4E2D7R19R24_IDENTITY_SHA256;
    ALWithinEpochOuter9GrantCapture capture;
    const SplitBoundaryReport parent =
        run_al_within_epoch_outer9_grant_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "27e37a5a05689e8483956038cbc63ce24ded6d0307c41b7e9e86a44c0c42bfa8"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "c1aca629743893dc973112f9ced8262728e641084cc172e09bfe8ce5bf467a01";

    const auto& r22 = capture.r22;
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
    const std::vector<Vec3>& position = r22.candidate.update.position;
    const std::vector<double>& dual = r22.candidate.update.u;
    const bool source_exact = r22.state_root
            == "dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9"
        && r22.receipt_root
            == "96975d4dda71d7490e91ccccfa95c51de480c1bcd3191367f527d5b001c2fa8e"
        && capture.state_root
            == "b9b8a2d3f0827d9a66a54aeb24f70c674015b8d968d8aaae16189e4adc72ef82"
        && capture.grant_root
            == "4dd8744547f991b40e5871a4c1823c3e073952dc7b51d6c28acabd92cebf5d18"
        && capture.receipt_root
            == "16b216ea8c7748ee80a5551f3baf5ee8f3afedea13fe5b88c6283cf07f7de285"
        && capture.active_owner_root
            == "bd3b2356eb10bf422ae07c16ba9433a63a63da1ccd7e567dcdd3dd3bc4fcee81"
        && al_binary64_vec3_root(position)
            == "68d0b821be6f1bdd71f70bf570035be7713cd578fe08774644cbd7c53c672b0f"
        && al_r10_dual_root(dual)
            == "fbbe1973290706320cbb5f3f3280b6e62b493e3abad4d280ac2985951b7568f3"
        && capture.grant.roots[2]
            == "fa9e2db82287c5f3faac6d8c8031c55bab78230fd708479a569329152c74be7c";
    const bool resource_exact = capture.grant.limits
            == std::array<std::uint64_t, AL_V2_LIMIT_COUNT>{
                16U,16U,34U,512U,288U,64U,8704U}
        && capture.grant.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                9U,3U,32U,1U,182U,705U,46U,28U,679U,26U,2U,28U,0U};

    ALOuter6Execution candidate;
    ALOuter6Execution oracle;
    if (parent_exact && source_exact && resource_exact) {
        candidate = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 9U,
            182U, 512U, 46U, 28U);
        oracle = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 9U,
            705U, 8704U, 46U, 28U);
    }
    const std::string& static_root = r16.envelope.roots[AL_V2_STATIC_ROOT];
    const bool candidate_valid = al_outer7_execution_valid(
        candidate, 182U, 512U, static_root, 9U, 46U, 28U, 242U, 36U);
    const bool oracle_valid = al_outer7_execution_valid(
        oracle, 705U, 8704U, static_root, 9U, 46U, 28U, 242U, 36U);
    const bool oracle_exact = candidate_valid && oracle_valid
        && al_outer6_execution_exact(candidate, oracle);

    const auto successor_used = al_outer7_successor_used(
        capture.grant.used, candidate, 10U);
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
            == "3f4b2fb5d84b3f27260e53e344d5141530b8105f577483aca7a435ec85372522";
    const std::string consumed_owner_root = sha256_hex(consumed_owner_bytes);

    ALOuterExecutionReceiptV1 receipt;
    receipt.outer = 9U;
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
    state.roots = {r22.state_root, capture.state_root, capture.grant_root,
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
        const std::string observed = al_shadow_outer9_route(
            p,s,g,o,duplicate,stale,resource,c,oracle_ok,match,abort);
        if (observed == "SHADOW_OUTER9_EXECUTION_CANDIDATE") {
            transaction = committed;
        }
        const std::string after = al_outer7_transaction_bytes(transaction);
        const bool exact = observed == "SHADOW_OUTER9_EXECUTION_CANDIDATE"
            ? after == committed_bytes && before == initial_bytes
            : after == before;
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","SHADOW_OUTER9_EXECUTION_CANDIDATE",1,1,1,1,0,0,1,1,1,1,0);
    add("parent","SHADOW_OUTER9_PARENT_REJECTED",0,1,1,1,0,0,1,1,1,1,0);
    add("source","SHADOW_OUTER9_SOURCE_REJECTED",1,0,1,1,0,0,1,1,1,1,0);
    add("grant","SHADOW_OUTER9_GRANT_REJECTED",1,1,0,1,0,0,1,1,1,1,0);
    add("owner","SHADOW_OUTER9_OWNER_REJECTED",1,1,1,0,0,0,1,1,1,1,0);
    add("duplicate","SHADOW_OUTER9_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0);
    add("stale","SHADOW_OUTER9_STALE_EPOCH_REJECTED",1,1,1,1,0,1,1,1,1,1,0);
    add("resource","SHADOW_OUTER9_RESOURCE_REJECTED",1,1,1,1,0,0,0,1,1,1,0);
    add("candidate","SHADOW_OUTER9_CANDIDATE_REJECTED",1,1,1,1,0,0,1,0,1,1,0);
    add("oracle","SHADOW_OUTER9_ORACLE_REJECTED",1,1,1,1,0,0,1,1,0,1,0);
    add("mismatch","SHADOW_OUTER9_ORACLE_MISMATCH",1,1,1,1,0,0,1,1,1,0,0);
    add("abort","SHADOW_OUTER9_ABORTED",1,1,1,1,0,0,1,1,1,1,1);
    add("duplicate-replay","SHADOW_OUTER9_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0,1);
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
            == "68d0b821be6f1bdd71f70bf570035be7713cd578fe08774644cbd7c53c672b0f"
        && al_r10_dual_root(dual)
            == "fbbe1973290706320cbb5f3f3280b6e62b493e3abad4d280ac2985951b7568f3";
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
        ? "SHADOW_OUTER9_EXECUTION_CANDIDATE" : std::string{};
    std::size_t accepted = 0U;
    std::size_t rejected = 0U;
    for (const auto& trial : candidate.update.trials) {
        accepted += trial.candidate_accepted ? 1U : 0U;
        rejected += trial.candidate_accepted ? 0U : 1U;
    }
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R24_IDENTITY_SHA256 << '|' << parent_stdout << '|'
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
              "nsr3b4e2d7r19r24_shadow_outer9_execution.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R24_IDENTITY_SHA256 << "\""
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
        capture_out->r23 = std::move(capture);
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

SplitBoundaryReport run_al_shadow_outer9_execution_controls() {
    return run_al_shadow_outer9_execution_controls_impl(nullptr);
}
