namespace {

constexpr const char* B4E2D7R19R27_IDENTITY_SHA256 =
    "98a135b8600c66d628e41827803f4ea2573330f2ba198f509f740d53087209e5";
constexpr const char* B4E2D7R19R27_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r27-within-epoch-outer11-grant|v1|parent=a38bbe72:67a4734dc549f19e57812813ec89e21ce23a90d5e1bb8415a41b7fcd1de78295:6f5d971e6c0dfe3e4f21c48130e3e82f861355a42691b836f5adcc4b78c97f22|source-state=317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd;execution-receipt=3b5c6dd3b8824eddc3aad566a8b663d344e1702f7958dced701c61710d688776;history=7aaf5e5077f937f03db754e407243e8c6b61b0f813931cc59d83f7f7dd866785;epoch1;next-outer11;slice282;cumulative805;primal0x3e54ad2eec000000;stationarity0x3d1770a9a48ed9c8;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=11,2,26,1,282,805,54,32,775,30,2,32,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-526ac852b7e52a48e2692b611f2e4ea8e8356fd52019fbeeb9c4935db404f281;owner-after-94e0619fd8babb7398035bdfb06e17a569b18cd15957f09b1a32680502089eba;grant-3ded4fc34e99c45657fad87a0aeb4853e1d948817efad60b4106f659c4adefaa;receipt-53a938176504b9059504c511402b5b6f3e8b4dfb17bc6a2aa0f42204160cc330;active-owner-431466b59941f862c97c75c5614be7d2dd525199bfbe9ea5e175753bc4cd431e;state-369cab15f75ee90237aa5cbe334aa0da5f857e218bec9812773e43faa9682226|success=source-consumed1;active-unconsumed;epoch1;slice282;cumulative805;all-ledger-exact;next-outer11;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer11-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer11=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer11-grant-candidate-only)IDENTITY";

struct ALWithinEpochOuter11GrantCapture {
    bool captured = false;
    ALShadowOuter10ExecutionCapture r26;
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

SplitBoundaryReport run_al_within_epoch_outer11_grant_controls_impl(
    ALWithinEpochOuter11GrantCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R27_IDENTITY_PROJECTION)
        == B4E2D7R19R27_IDENTITY_SHA256;
    ALShadowOuter10ExecutionCapture capture;
    const SplitBoundaryReport parent =
        run_al_shadow_outer10_execution_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "6f5d971e6c0dfe3e4f21c48130e3e82f861355a42691b836f5adcc4b78c97f22"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "67a4734dc549f19e57812813ec89e21ce23a90d5e1bb8415a41b7fcd1de78295";
    const std::string source_state = capture.state_root;
    const bool source_exact = source_state
            == "317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd"
        && capture.receipt_root
            == "3b5c6dd3b8824eddc3aad566a8b663d344e1702f7958dced701c61710d688776"
        && capture.receipt.roots[6]
            == "7aaf5e5077f937f03db754e407243e8c6b61b0f813931cc59d83f7f7dd866785"
        && capture.receipt.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                11U,2U,26U,1U,282U,805U,54U,32U,775U,30U,2U,32U,0U}
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
            == "526ac852b7e52a48e2692b611f2e4ea8e8356fd52019fbeeb9c4935db404f281"
        && sha256_hex(owner_after_bytes)
            == "94e0619fd8babb7398035bdfb06e17a569b18cd15957f09b1a32680502089eba";
    ALOuterGrantV1 grant;
    grant.epoch = 1U;
    grant.next_outer = 11U;
    grant.roots = {source_state, capture.receipt_root,
        capture.receipt.roots[6]};
    grant.limits = capture.receipt.limits;
    grant.used = capture.receipt.used;
    std::string grant_bytes;
    const bool grant_exact = al_encode_outer_grant(grant, grant_bytes)
        && sha256_hex(grant_bytes)
            == "3ded4fc34e99c45657fad87a0aeb4853e1d948817efad60b4106f659c4adefaa";
    const std::string grant_root = sha256_hex(grant_bytes);
    ALOuterGrantReceiptV1 receipt;
    receipt.epoch = 1U;
    receipt.next_outer = 11U;
    receipt.roots = {source_state, grant_root, B4E2D7R19R19_POLICY_ROOT};
    std::string receipt_bytes;
    const bool receipt_exact = al_encode_outer_grant_receipt(
            receipt, receipt_bytes)
        && sha256_hex(receipt_bytes)
            == "53a938176504b9059504c511402b5b6f3e8b4dfb17bc6a2aa0f42204160cc330";
    const std::string receipt_root = sha256_hex(receipt_bytes);
    ALOuterGrantOwnerV1 active{
        1U, grant_root, source_state, 1U, receipt_root};
    std::string active_bytes;
    const bool active_exact = al_encode_outer_grant_owner(active, active_bytes)
        && sha256_hex(active_bytes)
            == "431466b59941f862c97c75c5614be7d2dd525199bfbe9ea5e175753bc4cd431e";
    const std::string active_root = sha256_hex(active_bytes);
    ALOuterGrantStateV1 state;
    state.roots = {source_state, sha256_hex(owner_after_bytes), grant_root,
        receipt_root, active_root};
    std::string state_bytes;
    const bool state_exact = al_encode_outer_grant_state(state, state_bytes)
        && sha256_hex(state_bytes)
            == "369cab15f75ee90237aa5cbe334aa0da5f857e218bec9812773e43faa9682226";
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
            context, g, r, a, abort, "OUTER11_GRANT_CANDIDATE");
        std::string after;
        const bool after_exact = al_outer_grant_transaction_state_bytes(
            transaction, after);
        const bool success = observed == "OUTER11_GRANT_CANDIDATE";
        const bool exact = before_exact && after_exact
            && (success ? after == committed_bytes : after == before);
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","OUTER11_GRANT_CANDIDATE",1,1,0,0,1,1,1,1,1,0);
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
        ? "OUTER11_GRANT_CANDIDATE" : std::string{};
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R27_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << source_state << ':' << grant_root << ':' << receipt_root << ':'
             << active_root << ':' << state_root << '|' << case_root << ':'
             << routes_exact << ':' << rollback_exact << '|' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r27_outer11_grant.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R27_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source_state_root\":\"" << source_state << "\""
           << ",\"owner_before_root\":\"" << sha256_hex(owner_before_bytes) << "\""
           << ",\"owner_after_root\":\"" << sha256_hex(owner_after_bytes) << "\""
           << ",\"grant_root\":\"" << grant_root << "\""
           << ",\"receipt_root\":\"" << receipt_root << "\""
           << ",\"active_owner_root\":\"" << active_root << "\""
           << ",\"state_root\":\"" << state_root << "\""
              ",\"epoch\":1,\"next_outer\":11,\"slice_hvp\":282"
              ",\"cumulative_hvp\":805"
           << ",\"cases\":" << cases.size()
           << ",\"case_root\":\"" << case_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"new_hvp\":0,\"new_workspaces\":0"
              ",\"new_precision_audits\":0,\"new_outer_updates\":0"
              ",\"outer11_executed\":false,\"public_commit_count\":0"
              ",\"world_physics_mutation\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (capture_out != nullptr) {
        capture_out->captured = passed;
        capture_out->r26 = std::move(capture);
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

SplitBoundaryReport run_al_within_epoch_outer11_grant_controls() {
    return run_al_within_epoch_outer11_grant_controls_impl(nullptr);
}
