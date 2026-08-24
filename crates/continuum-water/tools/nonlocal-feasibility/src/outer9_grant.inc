namespace {

constexpr const char* B4E2D7R19R23_IDENTITY_SHA256 =
    "84c510befbdb80777c6aa00cb3650ecf3212cc9d68a0541d0b603768208e0996";
constexpr const char* B4E2D7R19R23_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r23-within-epoch-outer9-grant|v1|parent=32d633fb:32d98ee5549b5321201d9280ddb700967a02de6a83bd8abc283bd2a7d4b18790:9a5380ef56951213b544b14573e7983d84f1b768e9b16b5aa41a33084fb16296|source-state=dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9;execution-receipt=96975d4dda71d7490e91ccccfa95c51de480c1bcd3191367f527d5b001c2fa8e;history=fa9e2db82287c5f3faac6d8c8031c55bab78230fd708479a569329152c74be7c;epoch1;next-outer9;slice182;cumulative705;primal0x3e55dc122c000000;stationarity0x3d15a9f60fbefbe2;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=9,3,32,1,182,705,46,28,679,26,2,28,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-1d7364cd688d8be1cfc0cc3f12fa58975cd8fe318c702da1201c81cbfbb9d7c0;owner-after-f4bb27171627bffbbfdd47551ec84eb2b842d17b29e0f2ff1464c34c01febd3b;grant-4dd8744547f991b40e5871a4c1823c3e073952dc7b51d6c28acabd92cebf5d18;receipt-16b216ea8c7748ee80a5551f3baf5ee8f3afedea13fe5b88c6283cf07f7de285;active-owner-bd3b2356eb10bf422ae07c16ba9433a63a63da1ccd7e567dcdd3dd3bc4fcee81;state-b9b8a2d3f0827d9a66a54aeb24f70c674015b8d968d8aaae16189e4adc72ef82|success=source-consumed1;active-unconsumed;epoch1;slice182;cumulative705;all-ledger-exact;next-outer9;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer9-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer9=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer9-grant-candidate-only)IDENTITY";

struct ALWithinEpochOuter9GrantCapture {
    bool captured = false;
    ALShadowOuter8ExecutionCapture r22;
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

SplitBoundaryReport run_al_within_epoch_outer9_grant_controls_impl(
    ALWithinEpochOuter9GrantCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R23_IDENTITY_PROJECTION)
        == B4E2D7R19R23_IDENTITY_SHA256;
    ALShadowOuter8ExecutionCapture capture;
    const SplitBoundaryReport parent =
        run_al_shadow_outer8_execution_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "9a5380ef56951213b544b14573e7983d84f1b768e9b16b5aa41a33084fb16296"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "32d98ee5549b5321201d9280ddb700967a02de6a83bd8abc283bd2a7d4b18790";
    const std::string source_state = capture.state_root;
    const bool source_exact = source_state
            == "dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9"
        && capture.receipt_root
            == "96975d4dda71d7490e91ccccfa95c51de480c1bcd3191367f527d5b001c2fa8e"
        && capture.receipt.roots[6]
            == "fa9e2db82287c5f3faac6d8c8031c55bab78230fd708479a569329152c74be7c"
        && capture.receipt.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                9U,3U,32U,1U,182U,705U,46U,28U,679U,26U,2U,28U,0U}
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
            == "1d7364cd688d8be1cfc0cc3f12fa58975cd8fe318c702da1201c81cbfbb9d7c0"
        && sha256_hex(owner_after_bytes)
            == "f4bb27171627bffbbfdd47551ec84eb2b842d17b29e0f2ff1464c34c01febd3b";
    ALOuterGrantV1 grant;
    grant.epoch = 1U;
    grant.next_outer = 9U;
    grant.roots = {source_state, capture.receipt_root,
        capture.receipt.roots[6]};
    grant.limits = capture.receipt.limits;
    grant.used = capture.receipt.used;
    std::string grant_bytes;
    const bool grant_exact = al_encode_outer_grant(grant, grant_bytes)
        && sha256_hex(grant_bytes)
            == "4dd8744547f991b40e5871a4c1823c3e073952dc7b51d6c28acabd92cebf5d18";
    const std::string grant_root = sha256_hex(grant_bytes);
    ALOuterGrantReceiptV1 receipt;
    receipt.epoch = 1U;
    receipt.next_outer = 9U;
    receipt.roots = {source_state, grant_root, B4E2D7R19R19_POLICY_ROOT};
    std::string receipt_bytes;
    const bool receipt_exact = al_encode_outer_grant_receipt(
            receipt, receipt_bytes)
        && sha256_hex(receipt_bytes)
            == "16b216ea8c7748ee80a5551f3baf5ee8f3afedea13fe5b88c6283cf07f7de285";
    const std::string receipt_root = sha256_hex(receipt_bytes);
    ALOuterGrantOwnerV1 active{
        1U, grant_root, source_state, 1U, receipt_root};
    std::string active_bytes;
    const bool active_exact = al_encode_outer_grant_owner(active, active_bytes)
        && sha256_hex(active_bytes)
            == "bd3b2356eb10bf422ae07c16ba9433a63a63da1ccd7e567dcdd3dd3bc4fcee81";
    const std::string active_root = sha256_hex(active_bytes);
    ALOuterGrantStateV1 state;
    state.roots = {source_state, sha256_hex(owner_after_bytes), grant_root,
        receipt_root, active_root};
    std::string state_bytes;
    const bool state_exact = al_encode_outer_grant_state(state, state_bytes)
        && sha256_hex(state_bytes)
            == "b9b8a2d3f0827d9a66a54aeb24f70c674015b8d968d8aaae16189e4adc72ef82";
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
            context, g, r, a, abort, "OUTER9_GRANT_CANDIDATE");
        std::string after;
        const bool after_exact = al_outer_grant_transaction_state_bytes(
            transaction, after);
        const bool success = observed == "OUTER9_GRANT_CANDIDATE";
        const bool exact = before_exact && after_exact
            && (success ? after == committed_bytes : after == before);
        cases.push_back({name, expected, observed, exact});
    };
    add("valid","OUTER9_GRANT_CANDIDATE",1,1,0,0,1,1,1,1,1,0);
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
        ? "OUTER9_GRANT_CANDIDATE" : std::string{};
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R23_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << source_state << ':' << grant_root << ':' << receipt_root << ':'
             << active_root << ':' << state_root << '|' << case_root << ':'
             << routes_exact << ':' << rollback_exact << '|' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r23_outer9_grant.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R23_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source_state_root\":\"" << source_state << "\""
           << ",\"owner_before_root\":\"" << sha256_hex(owner_before_bytes) << "\""
           << ",\"owner_after_root\":\"" << sha256_hex(owner_after_bytes) << "\""
           << ",\"grant_root\":\"" << grant_root << "\""
           << ",\"receipt_root\":\"" << receipt_root << "\""
           << ",\"active_owner_root\":\"" << active_root << "\""
           << ",\"state_root\":\"" << state_root << "\""
              ",\"epoch\":1,\"next_outer\":9,\"slice_hvp\":182"
              ",\"cumulative_hvp\":705"
           << ",\"cases\":" << cases.size()
           << ",\"case_root\":\"" << case_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"new_hvp\":0,\"new_workspaces\":0"
              ",\"new_precision_audits\":0,\"new_outer_updates\":0"
              ",\"outer9_executed\":false,\"public_commit_count\":0"
              ",\"world_physics_mutation\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (capture_out != nullptr) {
        capture_out->captured = passed;
        capture_out->r22 = std::move(capture);
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

SplitBoundaryReport run_al_within_epoch_outer9_grant_controls() {
    return run_al_within_epoch_outer9_grant_controls_impl(nullptr);
}
