namespace {

constexpr const char* B4E2D7R19R21_IDENTITY_SHA256 =
    "acbe1ec9472e25af8402b625f539ff68f08c96dbfb097e7a09883521a7ef413a";
constexpr const char* B4E2D7R19R21_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r21-within-epoch-outer8-grant|v1|parent=a696c3f7:df06ac4125082f715d200829b5afef203f0a9ab046f6e9018f97bd8066b1d113:a375f2e5aaabe89694ed2f2d4383f3cf1d51212c9a648d8094e5b17ec97d1412|source-state=1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f;execution-receipt=ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd;history=7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d;epoch1;next-outer8;slice102;cumulative625;primal0x3e56fb819c000000;stationarity0x3d4514ef89e8a7f5;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=8,2,27,1,102,625,41,25,602,23,2,25,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-138f23969d4cc130e02ddd54cbe1fbbb76dd75b0beee0f53312de8f3116b0c7b;owner-after-ca301305414a08ca75cc8acda791d0c4a93152988d184f63316658ac278032d1;grant-6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43;receipt-0675f2ab94286aee2b85afe6cf496bcef8fd615efc514143ecf978c51e2bc0f3;active-owner-1987820aa99c22ac2d49abbb894994370a2f85231e9b4792bfc518e3d0eb4c54;state-64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f|success=source-consumed1;active-unconsumed;epoch1;slice102;cumulative625;all-ledger-exact;next-outer8;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer8-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer8=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer8-grant-candidate-only)IDENTITY";

struct ALWithinEpochOuter8GrantCapture {
    bool captured = false;
    ALShadowOuter7ExecutionCapture r20;
    ALOuterGrantV1 grant;
    ALOuterGrantReceiptV1 receipt;
    ALOuterGrantOwnerV1 active_owner;
    ALOuterGrantStateV1 state;
    std::string grant_bytes;
    std::string receipt_bytes;
    std::string active_owner_bytes;
    std::string state_bytes;
    std::string grant_root;
    std::string receipt_root;
    std::string active_owner_root;
    std::string state_root;
};

} // namespace

SplitBoundaryReport run_al_within_epoch_outer8_grant_controls_impl(
    ALWithinEpochOuter8GrantCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R21_IDENTITY_PROJECTION)
        == B4E2D7R19R21_IDENTITY_SHA256;
    ALShadowOuter7ExecutionCapture capture;
    const SplitBoundaryReport parent =
        run_al_shadow_outer7_execution_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "a375f2e5aaabe89694ed2f2d4383f3cf1d51212c9a648d8094e5b17ec97d1412"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "df06ac4125082f715d200829b5afef203f0a9ab046f6e9018f97bd8066b1d113";
    const std::string source_state = capture.state_root;
    const bool source_exact = source_state
            == "1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f"
        && capture.receipt_root
            == "ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd"
        && capture.receipt.roots[6]
            == "7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d"
        && capture.receipt.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                8U,2U,27U,1U,102U,625U,41U,25U,602U,23U,2U,25U,0U}
        && capture.receipt.limits
            == std::array<std::uint64_t, AL_V2_LIMIT_COUNT>{
                16U,16U,34U,512U,288U,64U,8704U};

    const ALSourceOwnerV1 owner_before{0U, source_state, 1U};
    ALSourceOwnerV1 owner_after = owner_before;
    owner_after.flags = 1U;
    std::string owner_before_bytes;
    std::string owner_after_bytes;
    const bool owners_exact = al_encode_outer_resume_owner(
            owner_before, owner_before_bytes)
        && al_encode_outer_resume_owner(owner_after, owner_after_bytes)
        && sha256_hex(owner_before_bytes)
            == "138f23969d4cc130e02ddd54cbe1fbbb76dd75b0beee0f53312de8f3116b0c7b"
        && sha256_hex(owner_after_bytes)
            == "ca301305414a08ca75cc8acda791d0c4a93152988d184f63316658ac278032d1";
    ALOuterGrantV1 grant;
    grant.epoch = 1U;
    grant.next_outer = 8U;
    grant.roots = {source_state, capture.receipt_root,
        capture.receipt.roots[6]};
    grant.limits = capture.receipt.limits;
    grant.used = capture.receipt.used;
    std::string grant_bytes;
    const bool grant_exact = al_encode_outer_grant(grant, grant_bytes)
        && sha256_hex(grant_bytes)
            == "6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43";
    const std::string grant_root = sha256_hex(grant_bytes);
    ALOuterGrantReceiptV1 receipt;
    receipt.epoch = 1U;
    receipt.next_outer = 8U;
    receipt.roots = {source_state, grant_root, B4E2D7R19R19_POLICY_ROOT};
    std::string receipt_bytes;
    const bool receipt_exact = al_encode_outer_grant_receipt(
            receipt, receipt_bytes)
        && sha256_hex(receipt_bytes)
            == "0675f2ab94286aee2b85afe6cf496bcef8fd615efc514143ecf978c51e2bc0f3";
    const std::string receipt_root = sha256_hex(receipt_bytes);
    ALOuterGrantOwnerV1 active{
        1U, grant_root, source_state, 1U, receipt_root};
    std::string active_bytes;
    const bool active_exact = al_encode_outer_grant_owner(active, active_bytes)
        && sha256_hex(active_bytes)
            == "1987820aa99c22ac2d49abbb894994370a2f85231e9b4792bfc518e3d0eb4c54";
    const std::string active_root = sha256_hex(active_bytes);
    ALOuterGrantStateV1 state;
    state.roots = {source_state, sha256_hex(owner_after_bytes), grant_root,
        receipt_root, active_root};
    std::string state_bytes;
    const bool state_exact = al_encode_outer_grant_state(state, state_bytes)
        && sha256_hex(state_bytes)
            == "64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f";
    const std::string state_root = sha256_hex(state_bytes);

    const ALOuterGrantTransactionState initial{
        owner_before, {}, {}, {}, {}};
    const ALOuterGrantTransactionState committed{
        owner_after, grant_bytes, receipt_bytes, active_bytes, state_bytes};
    std::string initial_bytes;
    std::string committed_bytes;
    const bool transaction_states_exact =
        al_outer_grant_transaction_state_bytes(initial, initial_bytes)
        && al_outer_grant_transaction_state_bytes(committed, committed_bytes);
    struct Case {
        const char* name;
        const char* expected;
        std::string observed;
        bool state_exact = false;
    };
    std::vector<Case> cases;
    const auto add = [&](const char* name, const char* expected,
        bool source, bool owner, bool duplicate, bool stale, bool ledger,
        bool context, bool g, bool r, bool a, bool abort,
        bool replay = false) {
        ALOuterGrantTransactionState transaction = replay ? committed : initial;
        std::string before;
        const bool before_exact = al_outer_grant_transaction_state_bytes(
            transaction, before);
        const std::string observed = al_apply_outer_grant_transaction(
            transaction, committed, source, owner, duplicate, stale, ledger,
            context, g, r, a, abort, "OUTER8_GRANT_CANDIDATE");
        std::string after;
        const bool after_exact = al_outer_grant_transaction_state_bytes(
            transaction, after);
        const bool success = observed == "OUTER8_GRANT_CANDIDATE";
        const bool exact = before_exact && after_exact
            && (success ? after == committed_bytes : after == before);
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","OUTER8_GRANT_CANDIDATE",1,1,0,0,1,1,1,1,1,0);
    add("source","OUTER_GRANT_SOURCE_REJECTED",0,1,0,0,1,1,1,1,1,0);
    add("owner","OUTER_GRANT_OWNER_REJECTED",1,0,0,0,1,1,1,1,1,0);
    add("duplicate","OUTER_GRANT_DUPLICATE_REJECTED",1,1,1,0,1,1,1,1,1,0);
    add("stale","OUTER_GRANT_STALE_EPOCH_REJECTED",1,1,0,1,1,1,1,1,1,0);
    add("epoch","OUTER_GRANT_LEDGER_REJECTED",1,1,0,0,0,1,1,1,1,0);
    add("slice","OUTER_GRANT_LEDGER_REJECTED",1,1,0,0,0,1,1,1,1,0);
    add("cumulative","OUTER_GRANT_LEDGER_REJECTED",1,1,0,0,0,1,1,1,1,0);
    add("next","OUTER_GRANT_CONTEXT_REJECTED",1,1,0,0,1,0,1,1,1,0);
    add("grant","OUTER_GRANT_GRANT_REJECTED",1,1,0,0,1,1,0,1,1,0);
    add("receipt","OUTER_GRANT_RECEIPT_REJECTED",1,1,0,0,1,1,1,0,1,0);
    add("active","OUTER_GRANT_ACTIVE_OWNER_REJECTED",1,1,0,0,1,1,1,1,0,0);
    add("abort","OUTER_GRANT_ABORTED",1,1,0,0,1,1,1,1,1,1);
    add("duplicate-replay","OUTER_GRANT_DUPLICATE_REJECTED",1,1,1,0,1,1,1,1,1,0,1);
    std::ostringstream corpus;
    bool routes_exact = cases.size() == 14U;
    bool rollback_exact = transaction_states_exact;
    std::string first_failure;
    for (const auto& value : cases) {
        const bool exact = value.observed == value.expected;
        routes_exact = routes_exact && exact;
        rollback_exact = rollback_exact && value.state_exact;
        if ((!exact || !value.state_exact) && first_failure.empty()) {
            first_failure = value.name;
        }
        corpus << value.name << ':' << value.expected << ':'
               << value.observed << ':' << exact << ':'
               << value.state_exact << '|';
    }
    const std::string case_root = sha256_hex(corpus.str());
    const bool passed = identity_exact && parent_exact && source_exact
        && owners_exact && grant_exact && receipt_exact && active_exact
        && state_exact && routes_exact && rollback_exact;
    const std::string route = passed
        ? "OUTER8_GRANT_CANDIDATE" : std::string{};
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R21_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << source_state << ':' << grant_root << ':' << receipt_root << ':'
             << active_root << ':' << state_root << '|' << case_root << ':'
             << routes_exact << ':' << rollback_exact << '|' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r21_outer8_grant.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R21_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source_state_root\":\"" << source_state << "\""
           << ",\"owner_before_root\":\"" << sha256_hex(owner_before_bytes) << "\""
           << ",\"owner_after_root\":\"" << sha256_hex(owner_after_bytes) << "\""
           << ",\"grant_root\":\"" << grant_root << "\""
           << ",\"receipt_root\":\"" << receipt_root << "\""
           << ",\"active_owner_root\":\"" << active_root << "\""
           << ",\"state_root\":\"" << state_root << "\""
              ",\"epoch\":1,\"next_outer\":8,\"slice_hvp\":102"
              ",\"cumulative_hvp\":625"
           << ",\"cases\":" << cases.size()
           << ",\"case_root\":\"" << case_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"new_hvp\":0,\"new_workspaces\":0"
              ",\"new_precision_audits\":0,\"new_outer_updates\":0"
              ",\"outer8_executed\":false,\"public_commit_count\":0"
              ",\"world_physics_mutation\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (capture_out != nullptr) {
        capture_out->captured = passed;
        capture_out->r20 = std::move(capture);
        capture_out->grant = grant;
        capture_out->receipt = receipt;
        capture_out->active_owner = active;
        capture_out->state = state;
        capture_out->grant_bytes = grant_bytes;
        capture_out->receipt_bytes = receipt_bytes;
        capture_out->active_owner_bytes = active_bytes;
        capture_out->state_bytes = state_bytes;
        capture_out->grant_root = grant_root;
        capture_out->receipt_root = receipt_root;
        capture_out->active_owner_root = active_root;
        capture_out->state_root = state_root;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_within_epoch_outer8_grant_controls() {
    return run_al_within_epoch_outer8_grant_controls_impl(nullptr);
}
