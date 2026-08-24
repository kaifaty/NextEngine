namespace {

constexpr const char* B4E2D7R19R25_IDENTITY_SHA256 =
    "668841e76d3d6f1edfc6ba395dc43acdee37dffc690abf591081b25626a7ec90";
constexpr const char* B4E2D7R19R25_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r25-within-epoch-outer10-grant|v1|parent=b6fc1451:e401056652ba56aad8c9f7836ac963a57721ef4d3c498ae29cb1736e03562eed:5b623755c2fd1dc136e9e9281c32b387955b1daea4e435460e9f764892b88bbc|source-state=749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98;execution-receipt=9c618bd10b6aeaed9ae9e5fddd9f7287e4df6d7adf3e76e467543c70e8674e28;history=d39b98c23d63217b0145229b593154fa3b49e2ea2b701ba708c51f5f8eb0b28d;epoch1;next-outer10;slice231;cumulative754;primal0x3e54e2d2e4000000;stationarity0x3dda59ffa2498bd8;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=10,2,24,1,231,754,50,30,726,28,2,30,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-3519d3b80578ac1dfc2b89b68340f1fe748775ab265a9535e2eb7bb5d75039bb;owner-after-11bf48f76ee1c521e2fa29d43e2b0a3f9cd1a03c56e4ee344497099e01e4991f;grant-f825ede7b3fa52c93b47806f92e40f34eefb93daca54d0b2d3092bf0870eede7;receipt-cc39098c863c2fa614086e3702029b0c04182bdfd501ffbf94753d9bd410a8fa;active-owner-2f6aea4168bdc513f21099f363a9aa28cc9171620f4a7c558b2cc340d6a6f6c6;state-b82cd881e7b50794de029e6829e8575b86630d50b0f4caaf74a7526a3bb66153|success=source-consumed1;active-unconsumed;epoch1;slice231;cumulative754;all-ledger-exact;next-outer10;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer10-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer10=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer10-grant-candidate-only)IDENTITY";

struct ALWithinEpochOuter10GrantCapture {
    bool captured = false;
    ALShadowOuter9ExecutionCapture r24;
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

SplitBoundaryReport run_al_within_epoch_outer10_grant_controls_impl(
    ALWithinEpochOuter10GrantCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R25_IDENTITY_PROJECTION)
        == B4E2D7R19R25_IDENTITY_SHA256;
    ALShadowOuter9ExecutionCapture capture;
    const SplitBoundaryReport parent =
        run_al_shadow_outer9_execution_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "5b623755c2fd1dc136e9e9281c32b387955b1daea4e435460e9f764892b88bbc"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "e401056652ba56aad8c9f7836ac963a57721ef4d3c498ae29cb1736e03562eed";
    const std::string source_state = capture.state_root;
    const bool source_exact = source_state
            == "749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98"
        && capture.receipt_root
            == "9c618bd10b6aeaed9ae9e5fddd9f7287e4df6d7adf3e76e467543c70e8674e28"
        && capture.receipt.roots[6]
            == "d39b98c23d63217b0145229b593154fa3b49e2ea2b701ba708c51f5f8eb0b28d"
        && capture.receipt.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                10U,2U,24U,1U,231U,754U,50U,30U,726U,28U,2U,30U,0U}
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
            == "3519d3b80578ac1dfc2b89b68340f1fe748775ab265a9535e2eb7bb5d75039bb"
        && sha256_hex(owner_after_bytes)
            == "11bf48f76ee1c521e2fa29d43e2b0a3f9cd1a03c56e4ee344497099e01e4991f";
    ALOuterGrantV1 grant;
    grant.epoch = 1U;
    grant.next_outer = 10U;
    grant.roots = {source_state, capture.receipt_root,
        capture.receipt.roots[6]};
    grant.limits = capture.receipt.limits;
    grant.used = capture.receipt.used;
    std::string grant_bytes;
    const bool grant_exact = al_encode_outer_grant(grant, grant_bytes)
        && sha256_hex(grant_bytes)
            == "f825ede7b3fa52c93b47806f92e40f34eefb93daca54d0b2d3092bf0870eede7";
    const std::string grant_root = sha256_hex(grant_bytes);
    ALOuterGrantReceiptV1 receipt;
    receipt.epoch = 1U;
    receipt.next_outer = 10U;
    receipt.roots = {source_state, grant_root, B4E2D7R19R19_POLICY_ROOT};
    std::string receipt_bytes;
    const bool receipt_exact = al_encode_outer_grant_receipt(
            receipt, receipt_bytes)
        && sha256_hex(receipt_bytes)
            == "cc39098c863c2fa614086e3702029b0c04182bdfd501ffbf94753d9bd410a8fa";
    const std::string receipt_root = sha256_hex(receipt_bytes);
    ALOuterGrantOwnerV1 active{
        1U, grant_root, source_state, 1U, receipt_root};
    std::string active_bytes;
    const bool active_exact = al_encode_outer_grant_owner(active, active_bytes)
        && sha256_hex(active_bytes)
            == "2f6aea4168bdc513f21099f363a9aa28cc9171620f4a7c558b2cc340d6a6f6c6";
    const std::string active_root = sha256_hex(active_bytes);
    ALOuterGrantStateV1 state;
    state.roots = {source_state, sha256_hex(owner_after_bytes), grant_root,
        receipt_root, active_root};
    std::string state_bytes;
    const bool state_exact = al_encode_outer_grant_state(state, state_bytes)
        && sha256_hex(state_bytes)
            == "b82cd881e7b50794de029e6829e8575b86630d50b0f4caaf74a7526a3bb66153";
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
            context, g, r, a, abort, "OUTER10_GRANT_CANDIDATE");
        std::string after;
        const bool after_exact = al_outer_grant_transaction_state_bytes(
            transaction, after);
        const bool success = observed == "OUTER10_GRANT_CANDIDATE";
        const bool exact = before_exact && after_exact
            && (success ? after == committed_bytes : after == before);
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","OUTER10_GRANT_CANDIDATE",1,1,0,0,1,1,1,1,1,0);
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
        ? "OUTER10_GRANT_CANDIDATE" : std::string{};
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R25_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << source_state << ':' << grant_root << ':' << receipt_root << ':'
             << active_root << ':' << state_root << '|' << case_root << ':'
             << routes_exact << ':' << rollback_exact << '|' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r25_outer10_grant.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R25_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source_state_root\":\"" << source_state << "\""
           << ",\"owner_before_root\":\"" << sha256_hex(owner_before_bytes) << "\""
           << ",\"owner_after_root\":\"" << sha256_hex(owner_after_bytes) << "\""
           << ",\"grant_root\":\"" << grant_root << "\""
           << ",\"receipt_root\":\"" << receipt_root << "\""
           << ",\"active_owner_root\":\"" << active_root << "\""
           << ",\"state_root\":\"" << state_root << "\""
              ",\"epoch\":1,\"next_outer\":10,\"slice_hvp\":231"
              ",\"cumulative_hvp\":754"
           << ",\"cases\":" << cases.size()
           << ",\"case_root\":\"" << case_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"new_hvp\":0,\"new_workspaces\":0"
              ",\"new_precision_audits\":0,\"new_outer_updates\":0"
              ",\"outer10_executed\":false,\"public_commit_count\":0"
              ",\"world_physics_mutation\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (capture_out != nullptr) {
        capture_out->captured = passed;
        capture_out->r24 = std::move(capture);
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

SplitBoundaryReport run_al_within_epoch_outer10_grant_controls() {
    return run_al_within_epoch_outer10_grant_controls_impl(nullptr);
}
