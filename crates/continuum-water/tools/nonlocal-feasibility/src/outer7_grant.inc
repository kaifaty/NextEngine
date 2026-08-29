namespace {

constexpr const char* B4E2D7R19R19_IDENTITY_SHA256 =
    "1b16ea2def4561ff4f2c92879c40b1868c8894c0f53d900eafbc9778c7b50ec0";
constexpr const char* B4E2D7R19R19_POLICY =
    "nextengine.nonlocal.al-within-epoch-outer-grant-policy|v1|"
    "source-consume-once|epoch-preserve|slice-preserve|"
    "cumulative-preserve|next-outer-increment-one|copy-on-write|"
    "validate-before-commit";
constexpr const char* B4E2D7R19R19_POLICY_ROOT =
    "d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e";
constexpr const char* B4E2D7R19R19_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r19-within-epoch-outer7-grant|v1|parent=ad6daf67:3047e85dc42f22afe287b47b0bc3aaa2e9de3bb767b01b6d4c516497c51841e8:baadf44a6ae0a2d83865dc41d5e9de1fef82e64eb52f36fc0b3a78a9c213e14d|source-state=5ad2f99d356c9f5f7553d84189e1e279dcd360bdb9a588370a874b2c5fe707bc;resume-receipt=5a029f24a82b60117492f1929a2d63489d2321b63286a9260282101b74ef8a64;history=a36fa9c02821f1eac2785aa0ec35632f088810b3be625303271d2b9750b8a5b1;epoch1;next-outer7;slice50;cumulative573;primal0x3e5afe281c000000;stationarity0x3d5eabe1f7af94f6;admissible0|policy=d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e|limits=16,16,34,512,288,64,8704|used=7,2,25,1,50,573,37,23,552,21,2,23,0|encoding=source-owner-NEALROW1-1-44-60;grant-NEALOGT1-1-276-292;receipt-NEALOGR1-1-116-132;active-owner-NEALOOW1-1-108-124;state-NEALOGS1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=owner-before-6ead6c878eb810ab6ed7046014e02afd8ea6aeba1d8c1bd57b2f2e33f8bc37a1;owner-after-ea9f947b8778e3147f3ba83f3fbf6a2f04f703cb1b3a45def70739b2c71ca0e4;grant-cb1b13d53754e7b1408d66eeb0a00f7dc02a697ccd338d8d77834a6f8b1a6776;receipt-28b5b27673dc9ed247957c159f7c4ac071a074902c918a273327a0d0a40b513a;active-owner-f88ea265a2e2544b76ae335770bb548da6aef76c7a3883ff7ea87e62e6f92ac4;state-37d1556efa98ceb68db566a6f553d82295fc6b233fcbfb0aa6d393a96a4ef6b5|success=source-consumed1;active-unconsumed;epoch1;slice50;cumulative573;all-ledger-exact;next-outer7;physics-immutable|controls=valid;source;owner-root;duplicate;stale;epoch-change;slice-reset;cumulative-change;next-outer;grant-corrupt;receipt-corrupt;active-owner-corrupt;abort-before-commit;duplicate-replay|routes=outer-grant-source-rejected;outer-grant-owner-rejected;outer-grant-duplicate-rejected;outer-grant-stale-epoch-rejected;outer-grant-ledger-rejected;outer-grant-context-rejected;outer-grant-grant-rejected;outer-grant-receipt-rejected;outer-grant-active-owner-rejected;outer-grant-aborted;outer7-grant-candidate|precedence=source,owner,duplicate,stale,ledger,context,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|outer7=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;world-physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-within-epoch-outer7-grant-candidate-only)IDENTITY";

struct ALOuterGrantV1 {
    std::uint32_t flags = 1U;
    std::uint64_t epoch = 1U;
    std::uint64_t next_outer = 7U;
    std::array<std::string, 3U> roots;
    std::array<std::uint64_t, AL_V2_LIMIT_COUNT> limits{};
    std::array<std::uint64_t, AL_V2_USED_COUNT> used{};
};
struct ALOuterGrantReceiptV1 {
    std::uint32_t flags = 3U;
    std::uint64_t epoch = 1U;
    std::uint64_t next_outer = 7U;
    std::array<std::string, 3U> roots;
};
struct ALOuterGrantOwnerV1 {
    std::uint32_t flags = 1U;
    std::string grant_root;
    std::string source_state_root;
    std::uint64_t epoch = 1U;
    std::string receipt_root;
};
struct ALOuterGrantStateV1 {
    std::uint32_t flags = 7U;
    std::array<std::string, 5U> roots;
};

bool al_encode_outer_resume_owner(
    const ALSourceOwnerV1& value, std::string& bytes) {
    std::string body;
    al_append_u32_le(body, value.flags);
    if (!al_append_digest_bytes(body, value.expected_source_root)) {
        return false;
    }
    al_append_u64_le(body, value.expected_epoch);
    return al_encode_canonical_v1("NEALROW1", 44U, 60U, body, bytes);
}

bool al_encode_outer_grant(
    const ALOuterGrantV1& value, std::string& bytes) {
    std::string body;
    al_append_u32_le(body, value.flags);
    al_append_u64_le(body, value.epoch);
    al_append_u64_le(body, value.next_outer);
    for (const auto& root : value.roots) {
        if (!al_append_digest_bytes(body, root)) return false;
    }
    for (auto limit : value.limits) al_append_u64_le(body, limit);
    for (auto used : value.used) al_append_u64_le(body, used);
    return al_encode_canonical_v1("NEALOGT1", 276U, 292U, body, bytes);
}
bool al_encode_outer_grant_receipt(
    const ALOuterGrantReceiptV1& value, std::string& bytes) {
    std::string body;
    al_append_u32_le(body, value.flags);
    al_append_u64_le(body, value.epoch);
    al_append_u64_le(body, value.next_outer);
    for (const auto& root : value.roots) {
        if (!al_append_digest_bytes(body, root)) return false;
    }
    return al_encode_canonical_v1("NEALOGR1", 116U, 132U, body, bytes);
}
bool al_encode_outer_grant_owner(
    const ALOuterGrantOwnerV1& value, std::string& bytes) {
    std::string body;
    al_append_u32_le(body, value.flags);
    if (!al_append_digest_bytes(body, value.grant_root)
        || !al_append_digest_bytes(body, value.source_state_root)) {
        return false;
    }
    al_append_u64_le(body, value.epoch);
    if (!al_append_digest_bytes(body, value.receipt_root)) return false;
    return al_encode_canonical_v1("NEALOOW1", 108U, 124U, body, bytes);
}
bool al_encode_outer_grant_state(
    const ALOuterGrantStateV1& value, std::string& bytes) {
    std::string body;
    al_append_u32_le(body, value.flags);
    for (const auto& root : value.roots) {
        if (!al_append_digest_bytes(body, root)) return false;
    }
    return al_encode_canonical_v1("NEALOGS1", 164U, 180U, body, bytes);
}

std::string al_outer_grant_route(
    bool source, bool owner, bool duplicate, bool stale, bool ledger,
    bool context, bool grant, bool receipt, bool active, bool abort,
    const char* candidate_route = "OUTER7_GRANT_CANDIDATE") {
    if (!source) return "OUTER_GRANT_SOURCE_REJECTED";
    if (!owner) return "OUTER_GRANT_OWNER_REJECTED";
    if (duplicate) return "OUTER_GRANT_DUPLICATE_REJECTED";
    if (stale) return "OUTER_GRANT_STALE_EPOCH_REJECTED";
    if (!ledger) return "OUTER_GRANT_LEDGER_REJECTED";
    if (!context) return "OUTER_GRANT_CONTEXT_REJECTED";
    if (!grant) return "OUTER_GRANT_GRANT_REJECTED";
    if (!receipt) return "OUTER_GRANT_RECEIPT_REJECTED";
    if (!active) return "OUTER_GRANT_ACTIVE_OWNER_REJECTED";
    if (abort) return "OUTER_GRANT_ABORTED";
    return candidate_route;
}

struct ALOuterGrantTransactionState {
    ALSourceOwnerV1 source_owner;
    std::string grant_bytes;
    std::string receipt_bytes;
    std::string active_owner_bytes;
    std::string state_bytes;
};

bool al_outer_grant_transaction_state_bytes(
    const ALOuterGrantTransactionState& value, std::string& bytes) {
    std::string owner_bytes;
    if (!al_encode_outer_resume_owner(value.source_owner, owner_bytes)) {
        return false;
    }
    bytes.clear();
    const std::array<std::string, 5U> fields{
        owner_bytes, value.grant_bytes, value.receipt_bytes,
        value.active_owner_bytes, value.state_bytes};
    for (const auto& field : fields) {
        al_append_u64_le(bytes, static_cast<std::uint64_t>(field.size()));
        bytes.append(field);
    }
    return true;
}

std::string al_apply_outer_grant_transaction(
    ALOuterGrantTransactionState& state,
    const ALOuterGrantTransactionState& committed,
    bool source, bool owner, bool duplicate, bool stale, bool ledger,
    bool context, bool grant, bool receipt, bool active, bool abort,
    const char* candidate_route = "OUTER7_GRANT_CANDIDATE") {
    const std::string route = al_outer_grant_route(
        source, owner, duplicate, stale, ledger, context, grant, receipt,
        active, abort, candidate_route);
    if (route == candidate_route) {
        ALOuterGrantTransactionState candidate = committed;
        state = std::move(candidate);
    }
    return route;
}

struct ALWithinEpochOuter7GrantCapture {
    bool captured = false;
    ALShadowOuter6ResumeCapture r18;
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

SplitBoundaryReport run_al_within_epoch_outer7_grant_controls_impl(
    ALWithinEpochOuter7GrantCapture* capture_out) {
    const bool identity_exact = sha256_hex(B4E2D7R19R19_IDENTITY_PROJECTION)
        == B4E2D7R19R19_IDENTITY_SHA256;
    const bool policy_exact = sha256_hex(B4E2D7R19R19_POLICY)
        == B4E2D7R19R19_POLICY_ROOT;
    ALShadowOuter6ResumeCapture capture;
    const SplitBoundaryReport parent =
        run_al_shadow_outer6_resume_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "baadf44a6ae0a2d83865dc41d5e9de1fef82e64eb52f36fc0b3a78a9c213e14d"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "3047e85dc42f22afe287b47b0bc3aaa2e9de3bb767b01b6d4c516497c51841e8";
    const std::string source_state = capture.state_root;
    const bool source_exact = source_state
            == "5ad2f99d356c9f5f7553d84189e1e279dcd360bdb9a588370a874b2c5fe707bc"
        && capture.receipt_root
            == "5a029f24a82b60117492f1929a2d63489d2321b63286a9260282101b74ef8a64"
        && capture.receipt.used
            == std::array<std::uint64_t, AL_V2_USED_COUNT>{
                7U,2U,25U,1U,50U,573U,37U,23U,552U,21U,2U,23U,0U};

    const ALSourceOwnerV1 owner_before{0U, source_state, 1U};
    ALSourceOwnerV1 owner_after = owner_before;
    owner_after.flags = 1U;
    std::string owner_before_bytes, owner_after_bytes;
    const bool owners_exact = al_encode_outer_resume_owner(
            owner_before, owner_before_bytes)
        && al_encode_outer_resume_owner(owner_after, owner_after_bytes)
        && sha256_hex(owner_before_bytes)
            == "6ead6c878eb810ab6ed7046014e02afd8ea6aeba1d8c1bd57b2f2e33f8bc37a1"
        && sha256_hex(owner_after_bytes)
            == "ea9f947b8778e3147f3ba83f3fbf6a2f04f703cb1b3a45def70739b2c71ca0e4";

    ALOuterGrantV1 grant;
    grant.roots = {source_state, capture.receipt_root,
        capture.receipt.roots[6]};
    grant.limits = capture.receipt.limits;
    grant.used = capture.receipt.used;
    std::string grant_bytes;
    const bool grant_exact = al_encode_outer_grant(grant, grant_bytes)
        && sha256_hex(grant_bytes)
            == "cb1b13d53754e7b1408d66eeb0a00f7dc02a697ccd338d8d77834a6f8b1a6776";
    const std::string grant_root = sha256_hex(grant_bytes);
    ALOuterGrantReceiptV1 receipt;
    receipt.roots = {source_state, grant_root, B4E2D7R19R19_POLICY_ROOT};
    std::string receipt_bytes;
    const bool receipt_exact = al_encode_outer_grant_receipt(
            receipt, receipt_bytes)
        && sha256_hex(receipt_bytes)
            == "28b5b27673dc9ed247957c159f7c4ac071a074902c918a273327a0d0a40b513a";
    const std::string receipt_root = sha256_hex(receipt_bytes);
    ALOuterGrantOwnerV1 active{1U, grant_root, source_state, 1U, receipt_root};
    std::string active_bytes;
    const bool active_exact = al_encode_outer_grant_owner(active, active_bytes)
        && sha256_hex(active_bytes)
            == "f88ea265a2e2544b76ae335770bb548da6aef76c7a3883ff7ea87e62e6f92ac4";
    ALOuterGrantStateV1 state;
    state.roots = {source_state, sha256_hex(owner_after_bytes), grant_root,
        receipt_root, sha256_hex(active_bytes)};
    std::string state_bytes;
    const bool state_exact = al_encode_outer_grant_state(state, state_bytes)
        && sha256_hex(state_bytes)
            == "37d1556efa98ceb68db566a6f553d82295fc6b233fcbfb0aa6d393a96a4ef6b5";

    ALOuterGrantTransactionState initial{owner_before, {}, {}, {}, {}};
    ALOuterGrantTransactionState committed{
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
        ALOuterGrantTransactionState state = replay ? committed : initial;
        std::string before;
        const bool before_exact =
            al_outer_grant_transaction_state_bytes(state, before);
        const std::string observed = al_apply_outer_grant_transaction(
            state, committed, source, owner, duplicate, stale, ledger,
            context, g, r, a, abort);
        std::string after;
        const bool after_exact =
            al_outer_grant_transaction_state_bytes(state, after);
        const bool success = observed == "OUTER7_GRANT_CANDIDATE";
        const bool state_exact = before_exact && after_exact
            && (success ? after == committed_bytes : after == before);
        cases.push_back({name, expected, observed, state_exact});
    };
    add("valid","OUTER7_GRANT_CANDIDATE",1,1,0,0,1,1,1,1,1,0);
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
    const bool cases_exact = routes_exact && rollback_exact;
    const bool passed = identity_exact && policy_exact && parent_exact
        && source_exact && owners_exact && grant_exact && receipt_exact
        && active_exact && state_exact && cases_exact;
    const std::string route = passed ? "OUTER7_GRANT_CANDIDATE" : "";
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|") << B4E2D7R19R19_IDENTITY_SHA256
             << '|' << parent_stdout << ':' << parent_semantic << '|'
             << source_state << ':' << grant_root << ':' << receipt_root << ':'
             << sha256_hex(active_bytes) << ':' << sha256_hex(state_bytes)
             << '|' << cases.size() << ':' << case_root << ':' << routes_exact
             << ':' << rollback_exact
             << '|' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r19_outer7_grant.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R19_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source_state_root\":\"" << source_state << "\""
           << ",\"owner_before_root\":\"" << sha256_hex(owner_before_bytes) << "\""
           << ",\"owner_after_root\":\"" << sha256_hex(owner_after_bytes) << "\""
           << ",\"grant_root\":\"" << grant_root << "\""
           << ",\"receipt_root\":\"" << receipt_root << "\""
           << ",\"active_owner_root\":\"" << sha256_hex(active_bytes) << "\""
           << ",\"state_root\":\"" << sha256_hex(state_bytes) << "\""
           << ",\"epoch\":1,\"next_outer\":7,\"slice_hvp\":50,\"cumulative_hvp\":573"
           << ",\"cases\":" << cases.size() << ",\"case_root\":\"" << case_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\",\"new_hvp\":0,\"new_workspaces\":0"
              ",\"new_precision_audits\":0,\"new_outer_updates\":0"
              ",\"outer7_executed\":false,\"public_commit_count\":0"
              ",\"world_physics_mutation\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (capture_out != nullptr) {
        capture_out->captured = passed;
        capture_out->r18 = std::move(capture);
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
        capture_out->active_owner_root = sha256_hex(active_bytes);
        capture_out->state_root = sha256_hex(state_bytes);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_within_epoch_outer7_grant_controls() {
    return run_al_within_epoch_outer7_grant_controls_impl(nullptr);
}
