namespace {

constexpr const char* B4E2D7R19R22_IDENTITY_SHA256 =
    "cbf62bb2861f3fdc7f9cccd3d34348b3d735d356ea093d00407754ac027d6081";
constexpr const char* B4E2D7R19R22_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r22-shadow-outer8-execution|v1|parent=0346640b:88253c46b9c1763ee4855be5909d1f9b58cf02828efa582b86a1eab61e1cb1a0:99d0a756044173aebb640e6d2bc301ac09c993b4ecb92d95bad886b6b4437719|source=r20-state=1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f;r20-receipt=ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd;r21-state=64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f;grant=6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43;grant-receipt=0675f2ab94286aee2b85afe6cf496bcef8fd615efc514143ecf978c51e2bc0f3;active-owner=1987820aa99c22ac2d49abbb894994370a2f85231e9b4792bfc518e3d0eb4c54;history=7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d;position=bda0ac4fa2be5c29484a17c520a968046849a49aa892a7c60b04f808a6199afd;dual=dc5bcc6bb918c3336ebe4bd99e03d7b8e1c92a61ca0db73418303ed0a59484eb;epoch1;outer8;slice102;cumulative625|owner-consume=flags1-to3;consumed-root=08001aecf61cc43ee130187f626264aa093420830244e06293ee7824486c9721|candidate=slice-budget512;total102;available410;oracle=unsliced-budget8704;total625;available8079;both-one-outer8;same-predicted-theta-static-formula-solver-completion|limits=16,16,34,512,288,64,8704|baseline-used=8,2,27,1,102,625,41,25,602,23,2,25,0|bounds=outer1;inner<=16;new-hvp<=410;new-workspaces<=247;new-precision<=39;cumulative-new-hvp<=8079|policy=precancelled-divided;dimensionless-stationarity;binary64-owned-membership;tiered-grace-residual-model|result=roots-derived-after-independent-execution;receipt-NEALOER1-schema1-body404-total420;state-NEALOES1-schema1-body196-total212|controls=parent;source;grant;active-owner;duplicate;stale;resource;candidate;oracle;oracle-mismatch;abort-before-commit;duplicate-replay|routes=shadow-outer8-parent-rejected;shadow-outer8-source-rejected;shadow-outer8-grant-rejected;shadow-outer8-owner-rejected;shadow-outer8-duplicate-rejected;shadow-outer8-stale-epoch-rejected;shadow-outer8-resource-rejected;shadow-outer8-candidate-rejected;shadow-outer8-oracle-rejected;shadow-outer8-oracle-mismatch;shadow-outer8-aborted;shadow-outer8-execution-candidate|precedence=parent,source,grant,owner,duplicate,stale,resource,candidate,oracle,mismatch,abort,candidate|atomic=copy-on-write;active-owner-consume-and-receipt-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-outers1;oracle-outers1;second-substep0;macro0;trajectory0;timing0|public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-shadow-outer8-execution-candidate-only)IDENTITY";

std::string al_shadow_outer8_route(
    bool parent, bool source, bool grant, bool owner, bool duplicate,
    bool stale, bool resource, bool candidate, bool oracle, bool match,
    bool abort) {
    if (!parent) return "SHADOW_OUTER8_PARENT_REJECTED";
    if (!source) return "SHADOW_OUTER8_SOURCE_REJECTED";
    if (!grant) return "SHADOW_OUTER8_GRANT_REJECTED";
    if (!owner) return "SHADOW_OUTER8_OWNER_REJECTED";
    if (duplicate) return "SHADOW_OUTER8_DUPLICATE_REJECTED";
    if (stale) return "SHADOW_OUTER8_STALE_EPOCH_REJECTED";
    if (!resource) return "SHADOW_OUTER8_RESOURCE_REJECTED";
    if (!candidate) return "SHADOW_OUTER8_CANDIDATE_REJECTED";
    if (!oracle) return "SHADOW_OUTER8_ORACLE_REJECTED";
    if (!match) return "SHADOW_OUTER8_ORACLE_MISMATCH";
    if (abort) return "SHADOW_OUTER8_ABORTED";
    return "SHADOW_OUTER8_EXECUTION_CANDIDATE";
}

struct ALShadowOuter8ExecutionCapture {
    bool captured = false;
    ALWithinEpochOuter8GrantCapture r21;
    ALOuter6Execution candidate;
    ALOuterExecutionReceiptV1 receipt;
    ALOuterExecutionStateV1 state;
    std::string receipt_bytes;
    std::string state_bytes;
    std::string receipt_root;
    std::string state_root;
};

} // namespace

SplitBoundaryReport run_al_shadow_outer8_execution_controls_impl(
    ALShadowOuter8ExecutionCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R22_IDENTITY_PROJECTION)
        == B4E2D7R19R22_IDENTITY_SHA256;
    ALWithinEpochOuter8GrantCapture capture;
    const SplitBoundaryReport parent =
        run_al_within_epoch_outer8_grant_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "99d0a756044173aebb640e6d2bc301ac09c993b4ecb92d95bad886b6b4437719"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "88253c46b9c1763ee4855be5909d1f9b58cf02828efa582b86a1eab61e1cb1a0";

    const auto& r20 = capture.r20;
    const auto& r19 = r20.r19;
    const auto& r18 = r19.r18;
    const auto& r17 = r18.r17;
    const ALV2EnvelopeCapture& r16 = r17.r16;
    const ALSoftCapSuspensionCapture& r14 = r16.r15.r14;
    const ALNormalizedTotalHvpBoundaryCapture& boundary =
        r14.r13.r12.r11.r10.total_hvp_boundary;
    const std::vector<Vec3>& predicted = boundary.predicted_position;
    const std::vector<Vec3>& position = r20.candidate.update.position;
    const std::vector<double>& dual = r20.candidate.update.u;
    const bool source_exact = r20.state_root
            == "1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f"
        && r20.receipt_root
            == "ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd"
        && capture.state_root
            == "64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f"
        && capture.grant_root
            == "6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43"
        && capture.receipt_root
            == "0675f2ab94286aee2b85afe6cf496bcef8fd615efc514143ecf978c51e2bc0f3"
        && capture.active_owner_root
            == "1987820aa99c22ac2d49abbb894994370a2f85231e9b4792bfc518e3d0eb4c54"
        && al_binary64_vec3_root(position)
            == "bda0ac4fa2be5c29484a17c520a968046849a49aa892a7c60b04f808a6199afd"
        && al_r10_dual_root(dual)
            == "dc5bcc6bb918c3336ebe4bd99e03d7b8e1c92a61ca0db73418303ed0a59484eb"
        && capture.grant.roots[2]
            == "7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d";
    const bool resource_exact = capture.grant.limits
            == std::array<std::uint64_t, AL_V2_LIMIT_COUNT>{
                16U,16U,34U,512U,288U,64U,8704U}
        && capture.grant.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                8U,2U,27U,1U,102U,625U,41U,25U,602U,23U,2U,25U,0U};

    ALOuter6Execution candidate;
    ALOuter6Execution oracle;
    if (parent_exact && source_exact && resource_exact) {
        candidate = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 8U,
            102U, 512U, 41U, 25U);
        oracle = al_execute_outer_update(
            predicted, position, dual, boundary.theta, 8U,
            625U, 8704U, 41U, 25U);
    }
    const std::string& static_root = r16.envelope.roots[AL_V2_STATIC_ROOT];
    const bool candidate_valid = al_outer7_execution_valid(
        candidate, 102U, 512U, static_root, 8U, 41U, 25U, 247U, 39U);
    const bool oracle_valid = al_outer7_execution_valid(
        oracle, 625U, 8704U, static_root, 8U, 41U, 25U, 247U, 39U);
    const bool oracle_exact = candidate_valid && oracle_valid
        && al_outer6_execution_exact(candidate, oracle);

    const auto successor_used = al_outer7_successor_used(
        capture.grant.used, candidate, 9U);
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
            == "08001aecf61cc43ee130187f626264aa093420830244e06293ee7824486c9721";
    const std::string consumed_owner_root = sha256_hex(consumed_owner_bytes);

    ALOuterExecutionReceiptV1 receipt;
    receipt.outer = 8U;
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
    state.roots = {r20.state_root, capture.state_root, capture.grant_root,
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
        const std::string observed = al_shadow_outer8_route(
            p,s,g,o,duplicate,stale,resource,c,oracle_ok,match,abort);
        if (observed == "SHADOW_OUTER8_EXECUTION_CANDIDATE") {
            transaction = committed;
        }
        const std::string after = al_outer7_transaction_bytes(transaction);
        const bool exact = observed == "SHADOW_OUTER8_EXECUTION_CANDIDATE"
            ? after == committed_bytes && before == initial_bytes
            : after == before;
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","SHADOW_OUTER8_EXECUTION_CANDIDATE",1,1,1,1,0,0,1,1,1,1,0);
    add("parent","SHADOW_OUTER8_PARENT_REJECTED",0,1,1,1,0,0,1,1,1,1,0);
    add("source","SHADOW_OUTER8_SOURCE_REJECTED",1,0,1,1,0,0,1,1,1,1,0);
    add("grant","SHADOW_OUTER8_GRANT_REJECTED",1,1,0,1,0,0,1,1,1,1,0);
    add("owner","SHADOW_OUTER8_OWNER_REJECTED",1,1,1,0,0,0,1,1,1,1,0);
    add("duplicate","SHADOW_OUTER8_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0);
    add("stale","SHADOW_OUTER8_STALE_EPOCH_REJECTED",1,1,1,1,0,1,1,1,1,1,0);
    add("resource","SHADOW_OUTER8_RESOURCE_REJECTED",1,1,1,1,0,0,0,1,1,1,0);
    add("candidate","SHADOW_OUTER8_CANDIDATE_REJECTED",1,1,1,1,0,0,1,0,1,1,0);
    add("oracle","SHADOW_OUTER8_ORACLE_REJECTED",1,1,1,1,0,0,1,1,0,1,0);
    add("mismatch","SHADOW_OUTER8_ORACLE_MISMATCH",1,1,1,1,0,0,1,1,1,0,0);
    add("abort","SHADOW_OUTER8_ABORTED",1,1,1,1,0,0,1,1,1,1,1);
    add("duplicate-replay","SHADOW_OUTER8_DUPLICATE_REJECTED",1,1,1,1,1,0,1,1,1,1,0,1);
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
            == "bda0ac4fa2be5c29484a17c520a968046849a49aa892a7c60b04f808a6199afd"
        && al_r10_dual_root(dual)
            == "dc5bcc6bb918c3336ebe4bd99e03d7b8e1c92a61ca0db73418303ed0a59484eb";
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
        ? "SHADOW_OUTER8_EXECUTION_CANDIDATE" : std::string{};
    std::size_t accepted = 0U;
    std::size_t rejected = 0U;
    for (const auto& trial : candidate.update.trials) {
        accepted += trial.candidate_accepted ? 1U : 0U;
        rejected += trial.candidate_accepted ? 0U : 1U;
    }
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R22_IDENTITY_SHA256 << '|' << parent_stdout << '|'
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
              "nsr3b4e2d7r19r22_shadow_outer8_execution.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R22_IDENTITY_SHA256 << "\""
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
        capture_out->r21 = std::move(capture);
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

SplitBoundaryReport run_al_shadow_outer8_execution_controls() {
    return run_al_shadow_outer8_execution_controls_impl(nullptr);
}
