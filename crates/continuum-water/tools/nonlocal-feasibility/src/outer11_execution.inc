namespace {

constexpr const char* B4E2D7R19R28_IDENTITY_SHA256 =
    "e7133e07d6104bb41da8ef95446459eccee331c44fea096867b3ae9230f940cd";
constexpr const char* B4E2D7R19R28_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r28-shadow-outer11-execution|v1|parent=1ee7936f:3760daf964f16b64d6c7685b8cdde255f31812d5aacee55cc7c18d2a079d65c2:7f08a85aae890fcf48d207cbdd7be8424675b98eef25506aad68b604a16e47dd|source=r26-state=317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd;r26-receipt=3b5c6dd3b8824eddc3aad566a8b663d344e1702f7958dced701c61710d688776;r27-state=369cab15f75ee90237aa5cbe334aa0da5f857e218bec9812773e43faa9682226;grant=3ded4fc34e99c45657fad87a0aeb4853e1d948817efad60b4106f659c4adefaa;grant-receipt=53a938176504b9059504c511402b5b6f3e8b4dfb17bc6a2aa0f42204160cc330;active-owner=431466b59941f862c97c75c5614be7d2dd525199bfbe9ea5e175753bc4cd431e;history=7aaf5e5077f937f03db754e407243e8c6b61b0f813931cc59d83f7f7dd866785;position=174cad44a3edbc7eab514986e6be3d1bdd3c219aa68fc3c9913b164ce4058562;dual=63770158bc269b879c4e3b98ac12a176be334902f0c929d05d686a157ce40f87;epoch1;outer11;slice282;cumulative805;source-primal0x3e54ad2eec000000;source-stationarity0x3d1770a9a48ed9c8|owner-consume=flags1-to3;consumed-root=13feea1f5a87fb577ed5f9ccf89e0f46d6acdb03554ea1c48b5e2c81081c5264|candidate=slice-budget512;total282;available230;oracle=unsliced-budget8704;total805;available7899;both-one-outer11;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=11,2,26,1,282,805,54,32,775,30,2,32,0|bounds=outer1;inner<=16;new-hvp<=230;new-workspaces<=234;new-precision<=32;cumulative-new-hvp<=7899|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model;primal-progress-trend-report-only;stationarity-trend-report-only|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer11-parent-rejected;shadow-outer11-source-rejected;shadow-outer11-grant-rejected;shadow-outer11-owner-rejected;shadow-outer11-duplicate-rejected;shadow-outer11-stale-epoch-rejected;shadow-outer11-resource-rejected;shadow-outer11-candidate-rejected;shadow-outer11-oracle-rejected;shadow-outer11-oracle-mismatch;shadow-outer11-aborted;shadow-outer11-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer11-execution-candidate-only)IDENTITY";

std::string al_shadow_outer11_route(
    bool parent, bool source, bool grant, bool owner, bool duplicate,
    bool stale, bool resource, bool candidate, bool oracle, bool match,
    bool abort) {
    if (!parent) return "SHADOW_OUTER11_PARENT_REJECTED";
    if (!source) return "SHADOW_OUTER11_SOURCE_REJECTED";
    if (!grant) return "SHADOW_OUTER11_GRANT_REJECTED";
    if (!owner) return "SHADOW_OUTER11_OWNER_REJECTED";
    if (duplicate) return "SHADOW_OUTER11_DUPLICATE_REJECTED";
    if (stale) return "SHADOW_OUTER11_STALE_EPOCH_REJECTED";
    if (!resource) return "SHADOW_OUTER11_RESOURCE_REJECTED";
    if (!candidate) return "SHADOW_OUTER11_CANDIDATE_REJECTED";
    if (!oracle) return "SHADOW_OUTER11_ORACLE_REJECTED";
    if (!match) return "SHADOW_OUTER11_ORACLE_MISMATCH";
    if (abort) return "SHADOW_OUTER11_ABORTED";
    return "SHADOW_OUTER11_EXECUTION_CANDIDATE";
}

struct ALShadowOuter11ExecutionCapture {
    bool captured = false;
    ALWithinEpochOuter11GrantCapture r27;
    ALOuter6Execution candidate;
    ALOuterExecutionReceiptV1 receipt;
    ALOuterExecutionStateV1 state;
    std::string receipt_bytes;
    std::string state_bytes;
    std::string receipt_root;
    std::string state_root;
};

} // namespace

SplitBoundaryReport run_al_shadow_outer11_execution_controls_impl(
    ALShadowOuter11ExecutionCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R28_IDENTITY_PROJECTION)
        == B4E2D7R19R28_IDENTITY_SHA256;
    ALWithinEpochOuter11GrantCapture capture;
    const SplitBoundaryReport parent =
        run_al_within_epoch_outer11_grant_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "7f08a85aae890fcf48d207cbdd7be8424675b98eef25506aad68b604a16e47dd"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "3760daf964f16b64d6c7685b8cdde255f31812d5aacee55cc7c18d2a079d65c2";

    const auto& r26 = capture.r26;
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
    const std::vector<Vec3>& predicted = boundary.predicted_position;
    const std::vector<Vec3>& position = r26.candidate.update.position;
    const std::vector<double>& dual = r26.candidate.update.u;
    const bool source_exact = r26.state_root
            == "317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd"
        && r26.receipt_root
            == "3b5c6dd3b8824eddc3aad566a8b663d344e1702f7958dced701c61710d688776"
        && capture.state_root
            == "369cab15f75ee90237aa5cbe334aa0da5f857e218bec9812773e43faa9682226"
        && capture.grant_root
            == "3ded4fc34e99c45657fad87a0aeb4853e1d948817efad60b4106f659c4adefaa"
        && capture.receipt_root
            == "53a938176504b9059504c511402b5b6f3e8b4dfb17bc6a2aa0f42204160cc330"
        && capture.active_owner_root
            == "431466b59941f862c97c75c5614be7d2dd525199bfbe9ea5e175753bc4cd431e"
        && al_binary64_vec3_root(position)
            == "174cad44a3edbc7eab514986e6be3d1bdd3c219aa68fc3c9913b164ce4058562"
        && al_r10_dual_root(dual)
            == "63770158bc269b879c4e3b98ac12a176be334902f0c929d05d686a157ce40f87"
        && capture.grant.roots[2]
            == "7aaf5e5077f937f03db754e407243e8c6b61b0f813931cc59d83f7f7dd866785";
    const bool resource_exact = capture.grant.limits
            == std::array<std::uint64_t, AL_V2_LIMIT_COUNT>{
                16U,16U,34U,512U,288U,64U,8704U}
        && capture.grant.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                11U,2U,26U,1U,282U,805U,54U,32U,775U,30U,2U,32U,0U};

    ALOuter6Execution candidate;
    ALOuter6Execution oracle;
    if (parent_exact && source_exact && resource_exact) {
        candidate = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 11U,
            282U, 512U, 54U, 32U);
        oracle = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 11U,
            805U, 8704U, 54U, 32U);
    }
    const std::string& static_root = r16.envelope.roots[AL_V2_STATIC_ROOT];
    const bool candidate_valid = al_outer7_execution_valid(
        candidate, 282U, 512U, static_root, 11U, 54U, 32U, 234U, 32U);
    const bool oracle_valid = al_outer7_execution_valid(
        oracle, 805U, 8704U, static_root, 11U, 54U, 32U, 234U, 32U);
    const bool oracle_exact = candidate_valid && oracle_valid
        && al_outer6_execution_exact(candidate, oracle);

    const auto successor_used = al_outer7_successor_used(
        capture.grant.used, candidate, 12U);
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
            == "13feea1f5a87fb577ed5f9ccf89e0f46d6acdb03554ea1c48b5e2c81081c5264";
    const std::string consumed_owner_root = sha256_hex(consumed_owner_bytes);

    ALOuterExecutionReceiptV1 receipt;
    receipt.outer = 11U;
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
    state.roots = {r26.state_root, capture.state_root, capture.grant_root,
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
        const std::string observed = al_shadow_outer11_route(
            p,s,g,o,duplicate,stale,resource,c,oracle_ok,match,abort);
        if (observed == "SHADOW_OUTER11_EXECUTION_CANDIDATE") {
            transaction = committed;
        }
        const std::string after = al_outer7_transaction_bytes(transaction);
        const bool exact = observed == "SHADOW_OUTER11_EXECUTION_CANDIDATE"
            ? after == committed_bytes && before == initial_bytes
            : after == before;
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","SHADOW_OUTER11_EXECUTION_CANDIDATE",1,1,1,1,0,0,1,1,1,1,0);
    add("parent","SHADOW_OUTER11_PARENT_REJECTED",0,1,1,1,0,0,1,1,1,1,0);
    add("source","SHADOW_OUTER11_SOURCE_REJECTED",1,0,1,1,0,0,1,1,1,1,0);
    add("grant","SHADOW_OUTER11_GRANT_REJECTED",1,1,0,1,0,0,1,1,1,1,0);
    add("owner","SHADOW_OUTER11_OWNER_REJECTED",1,1,1,0,0,0,1,1,1,1,0);
    add("duplicate","SHADOW_OUTER11_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0);
    add("stale","SHADOW_OUTER11_STALE_EPOCH_REJECTED",1,1,1,1,0,1,1,1,1,1,0);
    add("resource","SHADOW_OUTER11_RESOURCE_REJECTED",1,1,1,1,0,0,0,1,1,1,0);
    add("candidate","SHADOW_OUTER11_CANDIDATE_REJECTED",1,1,1,1,0,0,1,0,1,1,0);
    add("oracle","SHADOW_OUTER11_ORACLE_REJECTED",1,1,1,1,0,0,1,1,0,1,0);
    add("mismatch","SHADOW_OUTER11_ORACLE_MISMATCH",1,1,1,1,0,0,1,1,1,0,0);
    add("abort","SHADOW_OUTER11_ABORTED",1,1,1,1,0,0,1,1,1,1,1);
    add("duplicate-replay","SHADOW_OUTER11_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0,1);
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
            == "174cad44a3edbc7eab514986e6be3d1bdd3c219aa68fc3c9913b164ce4058562"
        && al_r10_dual_root(dual)
            == "63770158bc269b879c4e3b98ac12a176be334902f0c929d05d686a157ce40f87";
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
        ? "SHADOW_OUTER11_EXECUTION_CANDIDATE" : std::string{};
    std::size_t accepted = 0U;
    std::size_t rejected = 0U;
    for (const auto& trial : candidate.update.trials) {
        accepted += trial.candidate_accepted ? 1U : 0U;
        rejected += trial.candidate_accepted ? 0U : 1U;
    }
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R28_IDENTITY_SHA256 << '|' << parent_stdout << '|'
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
              "nsr3b4e2d7r19r28_shadow_outer11_execution.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R28_IDENTITY_SHA256 << "\""
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
        capture_out->r27 = std::move(capture);
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

SplitBoundaryReport run_al_shadow_outer11_execution_controls() {
    return run_al_shadow_outer11_execution_controls_impl(nullptr);
}
