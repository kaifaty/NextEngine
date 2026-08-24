namespace {

constexpr const char* B4E2D7R19R17_IDENTITY_SHA256 =
    "22d564118bbf04dd49865629bb4ad700df2f1bfadc91b9510de2acd7edf51e73";
constexpr const char* B4E2D7R19R17_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r17-owner-epoch-transition|v1|parent=c8a30bf7:d493c68da969911afe7228e26dbfcb32434ebdf3c028e4aaef91a9ca30af689f:e0ed3a9bf7f804eb2ee77ee65d9ebdfe10c71745e24a82bc341eec55047cbdcd|source=069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9;history=9754a0bb714011fe4eb637060c6c1eee930e7eb4a1c7db991f664ae40011af7b;epoch0;next-outer6;slice523;cumulative523|policy=12f7a0956a2a7b911530f348834363db43d2ad0119fc5d6a19f9aa5145a06a6c|encoding=source-owner-NEALSOW1-1-44-60;grant-NEALEPG1-1-252-268;receipt-NEALORC1-1-116-132;active-owner-NEALAOW1-1-108-124;state-NEALOST1-1-164-180;fixed-order;little-endian;raw-digests;no-padding;exact-flags|targets=source-before-174c0609ed0dc2411fbf99fcda920c7c0c37d71ad51799e5facc397ebd2ca9db;source-after-51dc71281dc2c7b77e793e2fa040015764a71a941559195c68594bc61721bb58;grant-266f8ac864d9a83e7bf71fa1df0b4dc33fe964d6064ced058287f04e61f4d475;receipt-485b0cb29256f8ba2f0e6f8701925fa292cd409dc802b7189650009933431875;active-owner-cc6b9e854ca39d940f826899f02827765f293b96bbf9429f7b5b7caceddae9b2;state-before-c28e4b7d7ce948a2358d50626af5ac93c354584af350b4c5d3bf37290c8225d7;state-after-d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54|success=source-consumed1;active-unconsumed;epoch1;slice0;cumulative523;all-other-ledger-exact;envelope-immutable;payload-immutable|controls=valid;source-corrupt;owner-root;duplicate;stale;epoch-overflow;resource-exhausted;abort-before-prepare;abort-after-grant;abort-after-receipt;abort-before-commit;grant-corrupt;receipt-corrupt;active-owner-corrupt;duplicate-replay|routes=owner-transition-source-rejected;owner-transition-owner-rejected;owner-transition-duplicate-rejected;owner-transition-stale-epoch-rejected;owner-transition-epoch-overflow-rejected;owner-transition-resource-rejected;owner-transition-grant-rejected;owner-transition-receipt-rejected;owner-transition-active-owner-rejected;owner-transition-aborted;owner-epoch-transition-candidate|precedence=source,owner,duplicate,stale,overflow,resource,grant,receipt,active-owner,abort,candidate|atomic=copy-on-write-single-commit;all-failures-state-byte-exact;duplicate-idempotent|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|resume=none;outer6=none;solve=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;physics-mutation=none;live-budget-code-change=none;production-policy-change=none;durable-cas=none;concurrent-cas=none|credit=one-private-atomic-owner-epoch-transition-candidate-only)IDENTITY";

constexpr const char* AL_OWNER_EPOCH_POLICY =
    "nextengine.nonlocal.al-owner-epoch-transition-policy|v1|"
    "source-consume-once|epoch-increment-one|slice-reset-zero|"
    "cumulative-preserve|copy-on-write|validate-before-commit";
constexpr const char* AL_OWNER_EPOCH_POLICY_ROOT =
    "12f7a0956a2a7b911530f348834363db43d2ad0119fc5d6a19f9aa5145a06a6c";

constexpr std::size_t AL_SOURCE_OWNER_BODY_BYTES = 44U;
constexpr std::size_t AL_SOURCE_OWNER_BYTES = 60U;
constexpr std::size_t AL_EPOCH_GRANT_BODY_BYTES = 252U;
constexpr std::size_t AL_EPOCH_GRANT_BYTES = 268U;
constexpr std::size_t AL_TRANSITION_RECEIPT_BODY_BYTES = 116U;
constexpr std::size_t AL_TRANSITION_RECEIPT_BYTES = 132U;
constexpr std::size_t AL_ACTIVE_OWNER_BODY_BYTES = 108U;
constexpr std::size_t AL_ACTIVE_OWNER_BYTES = 124U;
constexpr std::size_t AL_OWNER_STATE_BODY_BYTES = 164U;
constexpr std::size_t AL_OWNER_STATE_BYTES = 180U;

constexpr std::uint32_t AL_SOURCE_OWNER_CONSUMED = 1U;
constexpr std::uint32_t AL_EPOCH_GRANT_ACTIVE = 1U;
constexpr std::uint32_t AL_TRANSITION_RECEIPT_FLAGS = 3U;
constexpr std::uint32_t AL_ACTIVE_OWNER_ACTIVE = 1U;
constexpr std::uint32_t AL_OWNER_STATE_TRANSITIONED = 7U;

struct ALSourceOwnerV1 {
    std::uint32_t flags = 0U;
    std::string expected_source_root;
    std::uint64_t expected_epoch = 0U;
};

struct ALEpochGrantV1 {
    std::uint32_t flags = 0U;
    std::uint64_t source_epoch = 0U;
    std::uint64_t active_epoch = 0U;
    std::uint64_t next_outer = 0U;
    std::string source_envelope_root;
    std::string history_root;
    std::array<std::uint64_t, AL_V2_LIMIT_COUNT> limits{};
    std::array<std::uint64_t, AL_V2_USED_COUNT> used{};
};

struct ALOwnerTransitionReceiptV1 {
    std::uint32_t flags = 0U;
    std::uint64_t source_epoch = 0U;
    std::uint64_t active_epoch = 0U;
    std::string source_envelope_root;
    std::string grant_root;
    std::string policy_root;
};

struct ALActiveEpochOwnerV1 {
    std::uint32_t flags = 0U;
    std::string expected_grant_root;
    std::string source_envelope_root;
    std::uint64_t expected_epoch = 0U;
    std::string transition_receipt_root;
};

struct ALOwnerEpochStateV1 {
    std::string source_envelope_root;
    ALSourceOwnerV1 source_owner;
    bool transitioned = false;
    ALEpochGrantV1 grant;
    ALOwnerTransitionReceiptV1 receipt;
    ALActiveEpochOwnerV1 active_owner;
};

struct ALOwnerEpochStateProjectionV1 {
    std::uint32_t flags = 0U;
    std::array<std::string, 5U> roots;
};

enum class ALOwnerEpochInjection {
    None,
    AbortBeforePrepare,
    AbortAfterGrant,
    AbortAfterReceipt,
    AbortBeforeCommit,
    CorruptGrant,
    CorruptReceipt,
    CorruptActiveOwner,
};

struct ALOwnerEpochTransitionResult {
    std::string route;
    std::string grant_root;
    std::string receipt_root;
    std::string active_owner_root;
    std::string state_root;
};

struct ALOwnerEpochCaseRecord {
    std::string name;
    std::string expected;
    std::string observed;
    bool state_exact = false;
    bool passed = false;
};

bool al_encode_canonical_v1(
    const char* magic,
    std::size_t expected_body_bytes,
    std::size_t expected_total_bytes,
    const std::string& body,
    std::string& bytes) {
    if (body.size() != expected_body_bytes) return false;
    bytes.assign(magic, 8U);
    al_append_u32_le(bytes, 1U);
    al_append_u32_le(bytes, static_cast<std::uint32_t>(body.size()));
    bytes += body;
    return bytes.size() == expected_total_bytes;
}

bool al_decode_canonical_v1_header(
    const std::string& bytes,
    const char* magic,
    std::size_t expected_body_bytes,
    std::size_t expected_total_bytes,
    std::size_t& offset) {
    if (bytes.size() != expected_total_bytes
        || bytes.compare(0U, 8U, magic) != 0) {
        return false;
    }
    offset = 8U;
    std::uint32_t schema = 0U;
    std::uint32_t body_bytes = 0U;
    return al_read_u32_le(bytes, offset, schema)
        && al_read_u32_le(bytes, offset, body_bytes)
        && schema == 1U && body_bytes == expected_body_bytes
        && bytes.size() - offset == body_bytes;
}

bool al_encode_source_owner_v1(
    const ALSourceOwnerV1& owner, std::string& bytes) {
    std::string body;
    body.reserve(AL_SOURCE_OWNER_BODY_BYTES);
    al_append_u32_le(body, owner.flags);
    if (!al_append_digest_bytes(body, owner.expected_source_root)) {
        return false;
    }
    al_append_u64_le(body, owner.expected_epoch);
    return al_encode_canonical_v1(
        "NEALSOW1", AL_SOURCE_OWNER_BODY_BYTES, AL_SOURCE_OWNER_BYTES,
        body, bytes);
}

bool al_decode_source_owner_v1(
    const std::string& bytes, ALSourceOwnerV1& owner) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALSOW1", AL_SOURCE_OWNER_BODY_BYTES,
            AL_SOURCE_OWNER_BYTES, offset)
        || !al_read_u32_le(bytes, offset, owner.flags)
        || (owner.flags & ~AL_SOURCE_OWNER_CONSUMED) != 0U
        || !al_read_digest_hex(bytes, offset, owner.expected_source_root)
        || !al_read_u64_le(bytes, offset, owner.expected_epoch)
        || offset != bytes.size()) {
        return false;
    }
    std::string reencoded;
    return al_encode_source_owner_v1(owner, reencoded)
        && reencoded == bytes;
}

bool al_source_owner_exact(
    const ALSourceOwnerV1& lhs, const ALSourceOwnerV1& rhs) {
    return lhs.flags == rhs.flags
        && lhs.expected_source_root == rhs.expected_source_root
        && lhs.expected_epoch == rhs.expected_epoch;
}

bool al_encode_epoch_grant_v1(
    const ALEpochGrantV1& grant, std::string& bytes) {
    std::string body;
    body.reserve(AL_EPOCH_GRANT_BODY_BYTES);
    al_append_u32_le(body, grant.flags);
    al_append_u64_le(body, grant.source_epoch);
    al_append_u64_le(body, grant.active_epoch);
    al_append_u64_le(body, grant.next_outer);
    if (!al_append_digest_bytes(body, grant.source_envelope_root)
        || !al_append_digest_bytes(body, grant.history_root)) {
        return false;
    }
    for (std::uint64_t limit : grant.limits) {
        al_append_u64_le(body, limit);
    }
    for (std::uint64_t used : grant.used) {
        al_append_u64_le(body, used);
    }
    return al_encode_canonical_v1(
        "NEALEPG1", AL_EPOCH_GRANT_BODY_BYTES, AL_EPOCH_GRANT_BYTES,
        body, bytes);
}

bool al_decode_epoch_grant_v1(
    const std::string& bytes, ALEpochGrantV1& grant) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALEPG1", AL_EPOCH_GRANT_BODY_BYTES,
            AL_EPOCH_GRANT_BYTES, offset)
        || !al_read_u32_le(bytes, offset, grant.flags)
        || grant.flags != AL_EPOCH_GRANT_ACTIVE
        || !al_read_u64_le(bytes, offset, grant.source_epoch)
        || !al_read_u64_le(bytes, offset, grant.active_epoch)
        || !al_read_u64_le(bytes, offset, grant.next_outer)
        || !al_read_digest_hex(bytes, offset, grant.source_envelope_root)
        || !al_read_digest_hex(bytes, offset, grant.history_root)) {
        return false;
    }
    for (std::uint64_t& limit : grant.limits) {
        if (!al_read_u64_le(bytes, offset, limit)) return false;
    }
    for (std::uint64_t& used : grant.used) {
        if (!al_read_u64_le(bytes, offset, used)) return false;
    }
    if (offset != bytes.size()) return false;
    std::string reencoded;
    return al_encode_epoch_grant_v1(grant, reencoded)
        && reencoded == bytes;
}

bool al_epoch_grant_exact(
    const ALEpochGrantV1& lhs, const ALEpochGrantV1& rhs) {
    return lhs.flags == rhs.flags && lhs.source_epoch == rhs.source_epoch
        && lhs.active_epoch == rhs.active_epoch
        && lhs.next_outer == rhs.next_outer
        && lhs.source_envelope_root == rhs.source_envelope_root
        && lhs.history_root == rhs.history_root
        && lhs.limits == rhs.limits && lhs.used == rhs.used;
}

bool al_encode_owner_transition_receipt_v1(
    const ALOwnerTransitionReceiptV1& receipt, std::string& bytes) {
    std::string body;
    body.reserve(AL_TRANSITION_RECEIPT_BODY_BYTES);
    al_append_u32_le(body, receipt.flags);
    al_append_u64_le(body, receipt.source_epoch);
    al_append_u64_le(body, receipt.active_epoch);
    if (!al_append_digest_bytes(body, receipt.source_envelope_root)
        || !al_append_digest_bytes(body, receipt.grant_root)
        || !al_append_digest_bytes(body, receipt.policy_root)) {
        return false;
    }
    return al_encode_canonical_v1(
        "NEALORC1", AL_TRANSITION_RECEIPT_BODY_BYTES,
        AL_TRANSITION_RECEIPT_BYTES, body, bytes);
}

bool al_decode_owner_transition_receipt_v1(
    const std::string& bytes, ALOwnerTransitionReceiptV1& receipt) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALORC1", AL_TRANSITION_RECEIPT_BODY_BYTES,
            AL_TRANSITION_RECEIPT_BYTES, offset)
        || !al_read_u32_le(bytes, offset, receipt.flags)
        || receipt.flags != AL_TRANSITION_RECEIPT_FLAGS
        || !al_read_u64_le(bytes, offset, receipt.source_epoch)
        || !al_read_u64_le(bytes, offset, receipt.active_epoch)
        || !al_read_digest_hex(bytes, offset, receipt.source_envelope_root)
        || !al_read_digest_hex(bytes, offset, receipt.grant_root)
        || !al_read_digest_hex(bytes, offset, receipt.policy_root)
        || offset != bytes.size()) {
        return false;
    }
    std::string reencoded;
    return al_encode_owner_transition_receipt_v1(receipt, reencoded)
        && reencoded == bytes;
}

bool al_owner_transition_receipt_exact(
    const ALOwnerTransitionReceiptV1& lhs,
    const ALOwnerTransitionReceiptV1& rhs) {
    return lhs.flags == rhs.flags && lhs.source_epoch == rhs.source_epoch
        && lhs.active_epoch == rhs.active_epoch
        && lhs.source_envelope_root == rhs.source_envelope_root
        && lhs.grant_root == rhs.grant_root
        && lhs.policy_root == rhs.policy_root;
}

bool al_encode_active_epoch_owner_v1(
    const ALActiveEpochOwnerV1& owner, std::string& bytes) {
    std::string body;
    body.reserve(AL_ACTIVE_OWNER_BODY_BYTES);
    al_append_u32_le(body, owner.flags);
    if (!al_append_digest_bytes(body, owner.expected_grant_root)
        || !al_append_digest_bytes(body, owner.source_envelope_root)) {
        return false;
    }
    al_append_u64_le(body, owner.expected_epoch);
    if (!al_append_digest_bytes(body, owner.transition_receipt_root)) {
        return false;
    }
    return al_encode_canonical_v1(
        "NEALAOW1", AL_ACTIVE_OWNER_BODY_BYTES, AL_ACTIVE_OWNER_BYTES,
        body, bytes);
}

bool al_decode_active_epoch_owner_v1(
    const std::string& bytes, ALActiveEpochOwnerV1& owner) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALAOW1", AL_ACTIVE_OWNER_BODY_BYTES,
            AL_ACTIVE_OWNER_BYTES, offset)
        || !al_read_u32_le(bytes, offset, owner.flags)
        || owner.flags != AL_ACTIVE_OWNER_ACTIVE
        || !al_read_digest_hex(bytes, offset, owner.expected_grant_root)
        || !al_read_digest_hex(bytes, offset, owner.source_envelope_root)
        || !al_read_u64_le(bytes, offset, owner.expected_epoch)
        || !al_read_digest_hex(
            bytes, offset, owner.transition_receipt_root)
        || offset != bytes.size()) {
        return false;
    }
    std::string reencoded;
    return al_encode_active_epoch_owner_v1(owner, reencoded)
        && reencoded == bytes;
}

bool al_active_epoch_owner_exact(
    const ALActiveEpochOwnerV1& lhs,
    const ALActiveEpochOwnerV1& rhs) {
    return lhs.flags == rhs.flags
        && lhs.expected_grant_root == rhs.expected_grant_root
        && lhs.source_envelope_root == rhs.source_envelope_root
        && lhs.expected_epoch == rhs.expected_epoch
        && lhs.transition_receipt_root == rhs.transition_receipt_root;
}

std::string al_zero_digest() {
    return std::string(64U, '0');
}

bool al_encode_owner_epoch_state_projection_v1(
    const ALOwnerEpochStateProjectionV1& state, std::string& bytes) {
    std::string body;
    body.reserve(AL_OWNER_STATE_BODY_BYTES);
    al_append_u32_le(body, state.flags);
    for (const std::string& root : state.roots) {
        if (!al_append_digest_bytes(body, root)) return false;
    }
    return al_encode_canonical_v1(
        "NEALOST1", AL_OWNER_STATE_BODY_BYTES, AL_OWNER_STATE_BYTES,
        body, bytes);
}

bool al_decode_owner_epoch_state_projection_v1(
    const std::string& bytes, ALOwnerEpochStateProjectionV1& state) {
    std::size_t offset = 0U;
    if (!al_decode_canonical_v1_header(
            bytes, "NEALOST1", AL_OWNER_STATE_BODY_BYTES,
            AL_OWNER_STATE_BYTES, offset)
        || !al_read_u32_le(bytes, offset, state.flags)
        || (state.flags != 0U
            && state.flags != AL_OWNER_STATE_TRANSITIONED)) {
        return false;
    }
    for (std::string& root : state.roots) {
        if (!al_read_digest_hex(bytes, offset, root)) return false;
    }
    if (offset != bytes.size()) return false;
    if (state.flags == 0U
        && (state.roots[2] != al_zero_digest()
            || state.roots[3] != al_zero_digest()
            || state.roots[4] != al_zero_digest())) {
        return false;
    }
    std::string reencoded;
    return al_encode_owner_epoch_state_projection_v1(state, reencoded)
        && reencoded == bytes;
}

bool al_owner_epoch_state_projection_exact(
    const ALOwnerEpochStateProjectionV1& lhs,
    const ALOwnerEpochStateProjectionV1& rhs) {
    return lhs.flags == rhs.flags && lhs.roots == rhs.roots;
}

bool al_project_owner_epoch_state_v1(
    const ALOwnerEpochStateV1& state,
    ALOwnerEpochStateProjectionV1& projection,
    std::string& bytes) {
    std::string source_owner_bytes;
    if (!al_encode_source_owner_v1(state.source_owner, source_owner_bytes)) {
        return false;
    }
    projection.flags = state.transitioned
        ? AL_OWNER_STATE_TRANSITIONED : 0U;
    projection.roots[0] = state.source_envelope_root;
    projection.roots[1] = sha256_hex(source_owner_bytes);
    projection.roots[2] = al_zero_digest();
    projection.roots[3] = al_zero_digest();
    projection.roots[4] = al_zero_digest();
    if (state.transitioned) {
        std::string grant_bytes;
        std::string receipt_bytes;
        std::string active_owner_bytes;
        if (!al_encode_epoch_grant_v1(state.grant, grant_bytes)
            || !al_encode_owner_transition_receipt_v1(
                state.receipt, receipt_bytes)
            || !al_encode_active_epoch_owner_v1(
                state.active_owner, active_owner_bytes)) {
            return false;
        }
        projection.roots[2] = sha256_hex(grant_bytes);
        projection.roots[3] = sha256_hex(receipt_bytes);
        projection.roots[4] = sha256_hex(active_owner_bytes);
    }
    return al_encode_owner_epoch_state_projection_v1(projection, bytes);
}

bool al_owner_epoch_state_roundtrip_v1(
    const ALOwnerEpochStateV1& state,
    std::string& bytes,
    std::string& root) {
    ALOwnerEpochStateProjectionV1 projected;
    if (!al_project_owner_epoch_state_v1(state, projected, bytes)) {
        return false;
    }
    ALOwnerEpochStateProjectionV1 decoded;
    std::string reencoded;
    const bool exact = al_decode_owner_epoch_state_projection_v1(
            bytes, decoded)
        && al_owner_epoch_state_projection_exact(decoded, projected)
        && al_encode_owner_epoch_state_projection_v1(decoded, reencoded)
        && reencoded == bytes;
    root = exact ? sha256_hex(bytes) : std::string{};
    return exact;
}

bool al_epoch_grant_semantically_valid(
    const ALEpochGrantV1& grant,
    const ALContinuationEnvelopeV2& source,
    const std::string& source_root) {
    if (source.used[AL_V2_USED_EPOCH]
        == std::numeric_limits<std::uint64_t>::max()) {
        return false;
    }
    auto expected_used = source.used;
    expected_used[AL_V2_USED_EPOCH] =
        source.used[AL_V2_USED_EPOCH] + 1U;
    expected_used[AL_V2_USED_SLICE_HVP] = 0U;
    return grant.flags == AL_EPOCH_GRANT_ACTIVE
        && grant.source_epoch == source.used[AL_V2_USED_EPOCH]
        && grant.active_epoch == expected_used[AL_V2_USED_EPOCH]
        && grant.next_outer == source.next_outer
        && grant.source_envelope_root == source_root
        && grant.history_root == source.roots[AL_V2_HISTORY_ROOT]
        && grant.limits == source.limits && grant.used == expected_used;
}

bool al_transition_receipt_semantically_valid(
    const ALOwnerTransitionReceiptV1& receipt,
    const ALContinuationEnvelopeV2& source,
    const std::string& source_root,
    const std::string& grant_root) {
    return receipt.flags == AL_TRANSITION_RECEIPT_FLAGS
        && receipt.source_epoch == source.used[AL_V2_USED_EPOCH]
        && receipt.active_epoch
            == source.used[AL_V2_USED_EPOCH] + 1U
        && receipt.source_envelope_root == source_root
        && receipt.grant_root == grant_root
        && receipt.policy_root == AL_OWNER_EPOCH_POLICY_ROOT;
}

bool al_active_owner_semantically_valid(
    const ALActiveEpochOwnerV1& owner,
    const ALContinuationEnvelopeV2& source,
    const std::string& source_root,
    const std::string& grant_root,
    const std::string& receipt_root) {
    return owner.flags == AL_ACTIVE_OWNER_ACTIVE
        && owner.expected_grant_root == grant_root
        && owner.source_envelope_root == source_root
        && owner.expected_epoch == source.used[AL_V2_USED_EPOCH] + 1U
        && owner.transition_receipt_root == receipt_root;
}

ALOwnerEpochTransitionResult al_apply_owner_epoch_transition(
    const std::string& source_bytes,
    const std::string& claimed_source_root,
    const ALContinuationEnvelopeV2& expected_source,
    const std::vector<Vec3>& position_payload,
    const std::vector<double>& dual_payload,
    const std::string& parent_transaction_root,
    ALOwnerEpochStateV1& state,
    ALOwnerEpochInjection injection) {
    std::string state_before_bytes;
    std::string state_before_root;
    std::string source_owner_bytes;
    ALSourceOwnerV1 decoded_source_owner;
    if (!al_owner_epoch_state_roundtrip_v1(
            state, state_before_bytes, state_before_root)
        || !al_encode_source_owner_v1(
            state.source_owner, source_owner_bytes)
        || !al_decode_source_owner_v1(
            source_owner_bytes, decoded_source_owner)
        || !al_source_owner_exact(
            decoded_source_owner, state.source_owner)
        || state.source_envelope_root
            != state.source_owner.expected_source_root) {
        return {"OWNER_TRANSITION_OWNER_REJECTED", {}, {}, {}, {}};
    }

    const ALContinuationOwnerV2 source_owner{
        state.source_owner.expected_source_root,
        state.source_owner.expected_epoch,
        (state.source_owner.flags & AL_SOURCE_OWNER_CONSUMED) != 0U};
    const ALV2ValidationResult source_validation =
        al_validate_continuation_v2(
            source_bytes, claimed_source_root, source_owner,
            expected_source, position_payload, dual_payload,
            parent_transaction_root);
    if (source_validation.route == "TOKEN_V2_OWNER_REJECTED") {
        return {"OWNER_TRANSITION_OWNER_REJECTED", {}, {}, {}, {}};
    }
    if (source_validation.route == "TOKEN_V2_DUPLICATE_REJECTED") {
        return {"OWNER_TRANSITION_DUPLICATE_REJECTED", {}, {}, {}, {}};
    }
    if (source_validation.route == "TOKEN_V2_STALE_EPOCH_REJECTED") {
        return {"OWNER_TRANSITION_STALE_EPOCH_REJECTED", {}, {}, {}, {}};
    }
    if (source_validation.route != "TOKEN_V2_VALIDATION_CANDIDATE") {
        return {"OWNER_TRANSITION_SOURCE_REJECTED", {}, {}, {}, {}};
    }
    if (expected_source.used[AL_V2_USED_EPOCH]
        == std::numeric_limits<std::uint64_t>::max()) {
        return {
            "OWNER_TRANSITION_EPOCH_OVERFLOW_REJECTED", {}, {}, {}, {}};
    }
    if (expected_source.used[AL_V2_USED_CUMULATIVE_HVP]
        >= expected_source.limits[AL_V2_LIMIT_CUMULATIVE_HVP]) {
        return {"OWNER_TRANSITION_RESOURCE_REJECTED", {}, {}, {}, {}};
    }
    if (injection == ALOwnerEpochInjection::AbortBeforePrepare) {
        return {"OWNER_TRANSITION_ABORTED", {}, {}, {}, {}};
    }

    ALOwnerEpochStateV1 candidate = state;
    candidate.source_owner.flags |= AL_SOURCE_OWNER_CONSUMED;
    candidate.grant.flags = AL_EPOCH_GRANT_ACTIVE;
    candidate.grant.source_epoch =
        expected_source.used[AL_V2_USED_EPOCH];
    candidate.grant.active_epoch = candidate.grant.source_epoch + 1U;
    candidate.grant.next_outer = expected_source.next_outer;
    candidate.grant.source_envelope_root = claimed_source_root;
    candidate.grant.history_root =
        expected_source.roots[AL_V2_HISTORY_ROOT];
    candidate.grant.limits = expected_source.limits;
    candidate.grant.used = expected_source.used;
    candidate.grant.used[AL_V2_USED_EPOCH] =
        candidate.grant.active_epoch;
    candidate.grant.used[AL_V2_USED_SLICE_HVP] = 0U;
    if (injection == ALOwnerEpochInjection::AbortAfterGrant) {
        return {"OWNER_TRANSITION_ABORTED", {}, {}, {}, {}};
    }
    if (injection == ALOwnerEpochInjection::CorruptGrant) {
        candidate.grant.flags ^= 2U;
    }
    std::string grant_bytes;
    ALEpochGrantV1 decoded_grant;
    if (!al_encode_epoch_grant_v1(candidate.grant, grant_bytes)
        || !al_decode_epoch_grant_v1(grant_bytes, decoded_grant)
        || !al_epoch_grant_exact(decoded_grant, candidate.grant)
        || !al_epoch_grant_semantically_valid(
            decoded_grant, expected_source, claimed_source_root)) {
        return {"OWNER_TRANSITION_GRANT_REJECTED", {}, {}, {}, {}};
    }
    const std::string grant_root = sha256_hex(grant_bytes);

    candidate.receipt.flags = AL_TRANSITION_RECEIPT_FLAGS;
    candidate.receipt.source_epoch = candidate.grant.source_epoch;
    candidate.receipt.active_epoch = candidate.grant.active_epoch;
    candidate.receipt.source_envelope_root = claimed_source_root;
    candidate.receipt.grant_root = grant_root;
    candidate.receipt.policy_root = AL_OWNER_EPOCH_POLICY_ROOT;
    if (injection == ALOwnerEpochInjection::AbortAfterReceipt) {
        return {"OWNER_TRANSITION_ABORTED", {}, {}, {}, {}};
    }
    if (injection == ALOwnerEpochInjection::CorruptReceipt) {
        candidate.receipt.flags ^= 4U;
    }
    std::string receipt_bytes;
    ALOwnerTransitionReceiptV1 decoded_receipt;
    if (!al_encode_owner_transition_receipt_v1(
            candidate.receipt, receipt_bytes)
        || !al_decode_owner_transition_receipt_v1(
            receipt_bytes, decoded_receipt)
        || !al_owner_transition_receipt_exact(
            decoded_receipt, candidate.receipt)
        || !al_transition_receipt_semantically_valid(
            decoded_receipt, expected_source, claimed_source_root,
            grant_root)) {
        return {"OWNER_TRANSITION_RECEIPT_REJECTED", {}, {}, {}, {}};
    }
    const std::string receipt_root = sha256_hex(receipt_bytes);

    candidate.active_owner.flags = AL_ACTIVE_OWNER_ACTIVE;
    candidate.active_owner.expected_grant_root = grant_root;
    candidate.active_owner.source_envelope_root = claimed_source_root;
    candidate.active_owner.expected_epoch = candidate.grant.active_epoch;
    candidate.active_owner.transition_receipt_root = receipt_root;
    if (injection == ALOwnerEpochInjection::CorruptActiveOwner) {
        candidate.active_owner.flags ^= 2U;
    }
    std::string active_owner_bytes;
    ALActiveEpochOwnerV1 decoded_active_owner;
    if (!al_encode_active_epoch_owner_v1(
            candidate.active_owner, active_owner_bytes)
        || !al_decode_active_epoch_owner_v1(
            active_owner_bytes, decoded_active_owner)
        || !al_active_epoch_owner_exact(
            decoded_active_owner, candidate.active_owner)
        || !al_active_owner_semantically_valid(
            decoded_active_owner, expected_source, claimed_source_root,
            grant_root, receipt_root)) {
        return {
            "OWNER_TRANSITION_ACTIVE_OWNER_REJECTED", {}, {}, {}, {}};
    }
    const std::string active_owner_root = sha256_hex(active_owner_bytes);

    std::string consumed_owner_bytes;
    ALSourceOwnerV1 decoded_consumed_owner;
    if (!al_encode_source_owner_v1(
            candidate.source_owner, consumed_owner_bytes)
        || !al_decode_source_owner_v1(
            consumed_owner_bytes, decoded_consumed_owner)
        || !al_source_owner_exact(
            decoded_consumed_owner, candidate.source_owner)) {
        return {"OWNER_TRANSITION_OWNER_REJECTED", {}, {}, {}, {}};
    }
    candidate.transitioned = true;
    std::string state_after_bytes;
    std::string state_after_root;
    if (!al_owner_epoch_state_roundtrip_v1(
            candidate, state_after_bytes, state_after_root)) {
        return {
            "OWNER_TRANSITION_ACTIVE_OWNER_REJECTED", {}, {}, {}, {}};
    }
    if (injection == ALOwnerEpochInjection::AbortBeforeCommit) {
        return {"OWNER_TRANSITION_ABORTED", {}, {}, {}, {}};
    }

    state = std::move(candidate);
    return {"OWNER_EPOCH_TRANSITION_CANDIDATE", grant_root, receipt_root,
        active_owner_root, state_after_root};
}

std::size_t al_owner_epoch_route_index(const std::string& route) {
    if (route == "OWNER_TRANSITION_SOURCE_REJECTED") return 0U;
    if (route == "OWNER_TRANSITION_OWNER_REJECTED") return 1U;
    if (route == "OWNER_TRANSITION_DUPLICATE_REJECTED") return 2U;
    if (route == "OWNER_TRANSITION_STALE_EPOCH_REJECTED") return 3U;
    if (route == "OWNER_TRANSITION_EPOCH_OVERFLOW_REJECTED") return 4U;
    if (route == "OWNER_TRANSITION_RESOURCE_REJECTED") return 5U;
    if (route == "OWNER_TRANSITION_GRANT_REJECTED") return 6U;
    if (route == "OWNER_TRANSITION_RECEIPT_REJECTED") return 7U;
    if (route == "OWNER_TRANSITION_ACTIVE_OWNER_REJECTED") return 8U;
    if (route == "OWNER_TRANSITION_ABORTED") return 9U;
    return 10U;
}

} // namespace

SplitBoundaryReport run_al_owner_epoch_transition_controls() {
    const bool identity_exact = sha256_hex(B4E2D7R19R17_IDENTITY_PROJECTION)
        == B4E2D7R19R17_IDENTITY_SHA256;
    const bool policy_exact = sha256_hex(AL_OWNER_EPOCH_POLICY)
        == AL_OWNER_EPOCH_POLICY_ROOT;

    ALV2EnvelopeCapture capture;
    const SplitBoundaryReport r16_parent =
        run_al_v2_envelope_validation_controls_impl(&capture);
    const std::string r16_stdout_sha256 = sha256_hex(r16_parent.json + "\n");
    const bool r16_semantic_exact = r16_parent.json.find(
        "\"result_sha256\":\""
        "e0ed3a9bf7f804eb2ee77ee65d9ebdfe10c71745e24a82bc341eec55047cbdcd"
        "\"") != std::string::npos;
    const bool parents_retained = r16_parent.json.find(
        "\"transitive_parents_retained\":true") != std::string::npos
        && r16_parent.json.find(
            "257244d44f3ec4460889968221cf95f0f4ca72090ec6406c7b7349a0710b81b4")
            != std::string::npos;
    const bool parent_exact = r16_parent.passed && capture.captured
        && r16_semantic_exact && parents_retained
        && r16_stdout_sha256
            == "d493c68da969911afe7228e26dbfcb32434ebdf3c028e4aaef91a9ca30af689f";

    const ALSoftCapSuspensionCapture& r14 = capture.r15.r14;
    const std::vector<Vec3>& position = r14.r13.r12.trial_position;
    const std::vector<double>& dual = r14.r13.outer_u;
    const ALContinuationEnvelopeV2 baseline = capture.envelope;
    const std::string baseline_bytes = capture.envelope_bytes;
    const std::string baseline_root = capture.envelope_root;
    const std::string parent_transaction_root =
        capture.parent_transaction_root;

    ALOwnerEpochStateV1 baseline_state;
    baseline_state.source_envelope_root = baseline_root;
    baseline_state.source_owner = {
        0U, baseline_root, baseline.used[AL_V2_USED_EPOCH]};
    std::string source_owner_before_bytes;
    ALSourceOwnerV1 decoded_source_owner_before;
    const bool source_owner_before_exact =
        al_encode_source_owner_v1(
            baseline_state.source_owner, source_owner_before_bytes)
        && source_owner_before_bytes.size() == AL_SOURCE_OWNER_BYTES
        && al_decode_source_owner_v1(
            source_owner_before_bytes, decoded_source_owner_before)
        && al_source_owner_exact(
            decoded_source_owner_before, baseline_state.source_owner)
        && sha256_hex(source_owner_before_bytes)
            == "174c0609ed0dc2411fbf99fcda920c7c0c37d71ad51799e5facc397ebd2ca9db";
    std::string state_before_bytes;
    std::string state_before_root;
    const bool state_before_exact = al_owner_epoch_state_roundtrip_v1(
            baseline_state, state_before_bytes, state_before_root)
        && state_before_bytes.size() == AL_OWNER_STATE_BYTES
        && state_before_root
            == "c28e4b7d7ce948a2358d50626af5ac93c354584af350b4c5d3bf37290c8225d7";

    std::vector<ALOwnerEpochCaseRecord> cases;
    std::ostringstream case_projection;
    ALOwnerEpochStateV1 successful_state;
    ALOwnerEpochTransitionResult successful_result;
    const auto observe_case = [&] (
        const std::string& name,
        const std::string& expected_route,
        const std::string& source_bytes,
        const std::string& claimed_root,
        const ALContinuationEnvelopeV2& expected_source,
        ALOwnerEpochStateV1 state,
        ALOwnerEpochInjection injection,
        bool should_commit) {
        std::string before_bytes;
        std::string before_root;
        const bool before_exact = al_owner_epoch_state_roundtrip_v1(
            state, before_bytes, before_root);
        const ALOwnerEpochTransitionResult result =
            al_apply_owner_epoch_transition(
                source_bytes, claimed_root, expected_source, position, dual,
                parent_transaction_root, state, injection);
        std::string after_bytes;
        std::string after_root;
        const bool after_exact = al_owner_epoch_state_roundtrip_v1(
            state, after_bytes, after_root);
        const bool state_exact = before_exact && after_exact
            && (should_commit
                    ? before_bytes != after_bytes
                    : before_bytes == after_bytes);
        ALOwnerEpochCaseRecord record;
        record.name = name;
        record.expected = expected_route;
        record.observed = result.route;
        record.state_exact = state_exact;
        record.passed = record.expected == record.observed
            && record.state_exact;
        case_projection << record.name << ':' << record.expected << ':'
                        << record.observed << ':' << before_root << ':'
                        << after_root << ':' << record.state_exact << '|';
        cases.push_back(std::move(record));
        if (name == "valid") {
            successful_state = std::move(state);
            successful_result = result;
        }
    };

    observe_case("valid", "OWNER_EPOCH_TRANSITION_CANDIDATE",
        baseline_bytes, baseline_root, baseline, baseline_state,
        ALOwnerEpochInjection::None, true);

    std::string corrupt_source = baseline_bytes;
    corrupt_source[0] ^= 1;
    observe_case("source-corrupt", "OWNER_TRANSITION_SOURCE_REJECTED",
        corrupt_source, sha256_hex(corrupt_source), baseline, baseline_state,
        ALOwnerEpochInjection::None, false);
    ALOwnerEpochStateV1 wrong_owner_state = baseline_state;
    al_mutate_digest(wrong_owner_state.source_owner.expected_source_root);
    wrong_owner_state.source_envelope_root =
        wrong_owner_state.source_owner.expected_source_root;
    observe_case("owner-root", "OWNER_TRANSITION_OWNER_REJECTED",
        baseline_bytes, baseline_root, baseline, wrong_owner_state,
        ALOwnerEpochInjection::None, false);
    ALOwnerEpochStateV1 duplicate_state = baseline_state;
    duplicate_state.source_owner.flags = AL_SOURCE_OWNER_CONSUMED;
    observe_case("duplicate", "OWNER_TRANSITION_DUPLICATE_REJECTED",
        baseline_bytes, baseline_root, baseline, duplicate_state,
        ALOwnerEpochInjection::None, false);
    ALOwnerEpochStateV1 stale_state = baseline_state;
    stale_state.source_owner.expected_epoch = 1U;
    observe_case("stale", "OWNER_TRANSITION_STALE_EPOCH_REJECTED",
        baseline_bytes, baseline_root, baseline, stale_state,
        ALOwnerEpochInjection::None, false);

    ALContinuationEnvelopeV2 overflow_source = baseline;
    overflow_source.used[AL_V2_USED_EPOCH] =
        std::numeric_limits<std::uint64_t>::max();
    std::string overflow_bytes;
    const bool overflow_encoded = al_encode_continuation_v2(
        overflow_source, overflow_bytes);
    const std::string overflow_root = overflow_encoded
        ? sha256_hex(overflow_bytes) : std::string{};
    ALOwnerEpochStateV1 overflow_state;
    overflow_state.source_envelope_root = overflow_root;
    overflow_state.source_owner = {
        0U, overflow_root, overflow_source.used[AL_V2_USED_EPOCH]};
    observe_case("epoch-overflow",
        "OWNER_TRANSITION_EPOCH_OVERFLOW_REJECTED", overflow_bytes,
        overflow_root, overflow_source, overflow_state,
        ALOwnerEpochInjection::None, false);

    ALContinuationEnvelopeV2 exhausted_source = baseline;
    exhausted_source.used[AL_V2_USED_CUMULATIVE_HVP] =
        exhausted_source.limits[AL_V2_LIMIT_CUMULATIVE_HVP];
    exhausted_source.used[AL_V2_USED_RECURRENCE_HVP] =
        exhausted_source.used[AL_V2_USED_CUMULATIVE_HVP]
        - exhausted_source.used[AL_V2_USED_DIRECT_MODEL_HVP];
    exhausted_source.roots[AL_V2_HISTORY_ROOT] =
        al_continuation_v2_history_root(
            parent_transaction_root, exhausted_source);
    std::string exhausted_bytes;
    const bool exhausted_encoded = al_encode_continuation_v2(
        exhausted_source, exhausted_bytes);
    const std::string exhausted_root = exhausted_encoded
        ? sha256_hex(exhausted_bytes) : std::string{};
    ALOwnerEpochStateV1 exhausted_state;
    exhausted_state.source_envelope_root = exhausted_root;
    exhausted_state.source_owner = {0U, exhausted_root, 0U};
    observe_case("resource-exhausted",
        "OWNER_TRANSITION_RESOURCE_REJECTED", exhausted_bytes,
        exhausted_root, exhausted_source, exhausted_state,
        ALOwnerEpochInjection::None, false);

    observe_case("abort-before-prepare", "OWNER_TRANSITION_ABORTED",
        baseline_bytes, baseline_root, baseline, baseline_state,
        ALOwnerEpochInjection::AbortBeforePrepare, false);
    observe_case("abort-after-grant", "OWNER_TRANSITION_ABORTED",
        baseline_bytes, baseline_root, baseline, baseline_state,
        ALOwnerEpochInjection::AbortAfterGrant, false);
    observe_case("abort-after-receipt", "OWNER_TRANSITION_ABORTED",
        baseline_bytes, baseline_root, baseline, baseline_state,
        ALOwnerEpochInjection::AbortAfterReceipt, false);
    observe_case("abort-before-commit", "OWNER_TRANSITION_ABORTED",
        baseline_bytes, baseline_root, baseline, baseline_state,
        ALOwnerEpochInjection::AbortBeforeCommit, false);
    observe_case("grant-corrupt", "OWNER_TRANSITION_GRANT_REJECTED",
        baseline_bytes, baseline_root, baseline, baseline_state,
        ALOwnerEpochInjection::CorruptGrant, false);
    observe_case("receipt-corrupt", "OWNER_TRANSITION_RECEIPT_REJECTED",
        baseline_bytes, baseline_root, baseline, baseline_state,
        ALOwnerEpochInjection::CorruptReceipt, false);
    observe_case("active-owner-corrupt",
        "OWNER_TRANSITION_ACTIVE_OWNER_REJECTED", baseline_bytes,
        baseline_root, baseline, baseline_state,
        ALOwnerEpochInjection::CorruptActiveOwner, false);
    observe_case("duplicate-replay", "OWNER_TRANSITION_DUPLICATE_REJECTED",
        baseline_bytes, baseline_root, baseline, successful_state,
        ALOwnerEpochInjection::None, false);

    std::array<std::size_t, 11U> route_counts{};
    for (const ALOwnerEpochCaseRecord& record : cases) {
        ++route_counts[al_owner_epoch_route_index(record.observed)];
    }
    const bool route_counts_exact = route_counts
        == std::array<std::size_t, 11U>{
            1U, 1U, 2U, 1U, 1U, 1U, 1U, 1U, 1U, 4U, 1U};
    const bool cases_exact = cases.size() == 15U
        && std::all_of(cases.begin(), cases.end(),
            [](const ALOwnerEpochCaseRecord& record) {
                return record.passed;
            });
    const std::string case_root = sha256_hex(case_projection.str());

    std::string source_owner_after_bytes;
    ALSourceOwnerV1 decoded_source_owner_after;
    std::string grant_bytes;
    ALEpochGrantV1 decoded_grant;
    std::string receipt_bytes;
    ALOwnerTransitionReceiptV1 decoded_receipt;
    std::string active_owner_bytes;
    ALActiveEpochOwnerV1 decoded_active_owner;
    std::string state_after_bytes;
    std::string state_after_root;
    const bool canonical_targets_exact =
        successful_state.transitioned
        && al_encode_source_owner_v1(
            successful_state.source_owner, source_owner_after_bytes)
        && source_owner_after_bytes.size() == AL_SOURCE_OWNER_BYTES
        && al_decode_source_owner_v1(
            source_owner_after_bytes, decoded_source_owner_after)
        && al_source_owner_exact(
            decoded_source_owner_after, successful_state.source_owner)
        && sha256_hex(source_owner_after_bytes)
            == "51dc71281dc2c7b77e793e2fa040015764a71a941559195c68594bc61721bb58"
        && al_encode_epoch_grant_v1(successful_state.grant, grant_bytes)
        && grant_bytes.size() == AL_EPOCH_GRANT_BYTES
        && al_decode_epoch_grant_v1(grant_bytes, decoded_grant)
        && al_epoch_grant_exact(decoded_grant, successful_state.grant)
        && sha256_hex(grant_bytes)
            == "266f8ac864d9a83e7bf71fa1df0b4dc33fe964d6064ced058287f04e61f4d475"
        && al_encode_owner_transition_receipt_v1(
            successful_state.receipt, receipt_bytes)
        && receipt_bytes.size() == AL_TRANSITION_RECEIPT_BYTES
        && al_decode_owner_transition_receipt_v1(
            receipt_bytes, decoded_receipt)
        && al_owner_transition_receipt_exact(
            decoded_receipt, successful_state.receipt)
        && sha256_hex(receipt_bytes)
            == "485b0cb29256f8ba2f0e6f8701925fa292cd409dc802b7189650009933431875"
        && al_encode_active_epoch_owner_v1(
            successful_state.active_owner, active_owner_bytes)
        && active_owner_bytes.size() == AL_ACTIVE_OWNER_BYTES
        && al_decode_active_epoch_owner_v1(
            active_owner_bytes, decoded_active_owner)
        && al_active_epoch_owner_exact(
            decoded_active_owner, successful_state.active_owner)
        && sha256_hex(active_owner_bytes)
            == "cc6b9e854ca39d940f826899f02827765f293b96bbf9429f7b5b7caceddae9b2"
        && al_owner_epoch_state_roundtrip_v1(
            successful_state, state_after_bytes, state_after_root)
        && state_after_bytes.size() == AL_OWNER_STATE_BYTES
        && state_after_root
            == "d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54"
        && successful_result.grant_root == sha256_hex(grant_bytes)
        && successful_result.receipt_root == sha256_hex(receipt_bytes)
        && successful_result.active_owner_root
            == sha256_hex(active_owner_bytes)
        && successful_result.state_root == state_after_root;

    auto expected_success_used = baseline.used;
    expected_success_used[AL_V2_USED_EPOCH] = 1U;
    expected_success_used[AL_V2_USED_SLICE_HVP] = 0U;
    const bool transition_exact = canonical_targets_exact
        && successful_state.source_envelope_root == baseline_root
        && successful_state.source_owner.flags == AL_SOURCE_OWNER_CONSUMED
        && successful_state.source_owner.expected_source_root == baseline_root
        && successful_state.source_owner.expected_epoch == 0U
        && successful_state.grant.source_epoch == 0U
        && successful_state.grant.active_epoch == 1U
        && successful_state.grant.next_outer == 6U
        && successful_state.grant.limits == baseline.limits
        && successful_state.grant.used == expected_success_used
        && successful_state.grant.used[AL_V2_USED_CUMULATIVE_HVP] == 523U
        && successful_state.active_owner.expected_epoch == 1U
        && successful_state.active_owner.flags == AL_ACTIVE_OWNER_ACTIVE;
    const bool source_immutable = baseline_bytes == capture.envelope_bytes
        && sha256_hex(baseline_bytes) == baseline_root
        && baseline_root
            == "069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9";
    const bool payload_immutable = al_binary64_vec3_root(position)
            == baseline.roots[AL_V2_POSITION_ROOT]
        && al_r10_dual_root(dual) == baseline.roots[AL_V2_DUAL_ROOT];
    const bool negative_rollback_exact = std::all_of(
        cases.begin() + 1, cases.end(),
        [](const ALOwnerEpochCaseRecord& record) {
            return record.state_exact;
        });
    const bool encoded_controls = overflow_encoded && exhausted_encoded;
    const bool passed = identity_exact && policy_exact && parent_exact
        && source_owner_before_exact && state_before_exact
        && encoded_controls && cases_exact && route_counts_exact
        && canonical_targets_exact && transition_exact && source_immutable
        && payload_immutable && negative_rollback_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!policy_exact) first_failure = "POLICY_IDENTITY";
    else if (!parent_exact) first_failure = "D7R19R16_PARENT_BYTES";
    else if (!source_owner_before_exact) first_failure = "SOURCE_OWNER_BEFORE";
    else if (!state_before_exact) first_failure = "STATE_BEFORE";
    else if (!encoded_controls) first_failure = "NEGATIVE_ENCODE";
    else if (!cases_exact) first_failure = "NEGATIVE_ROUTE_OR_ROLLBACK";
    else if (!route_counts_exact) first_failure = "ROUTE_COUNTS";
    else if (!canonical_targets_exact) first_failure = "CANONICAL_TARGETS";
    else if (!transition_exact) first_failure = "TRANSITION";
    else if (!source_immutable) first_failure = "SOURCE_MUTATION";
    else if (!payload_immutable) first_failure = "PAYLOAD_MUTATION";
    else if (!negative_rollback_exact) first_failure = "ROLLBACK";
    const std::string route = passed
        ? "OWNER_EPOCH_TRANSITION_CANDIDATE" : std::string{};

    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << B4E2D7R19R17_IDENTITY_SHA256 << '|'
             << r16_stdout_sha256 << ':' << r16_semantic_exact << ':'
             << parents_retained << '|' << AL_OWNER_EPOCH_POLICY_ROOT << '|'
             << baseline_root << ':' << source_immutable << ':'
             << payload_immutable << '|'
             << sha256_hex(source_owner_before_bytes) << ':'
             << sha256_hex(source_owner_after_bytes) << '|'
             << successful_result.grant_root << ':'
             << successful_result.receipt_root << ':'
             << successful_result.active_owner_root << '|'
             << state_before_root << ':' << state_after_root << '|'
             << cases.size() << ':' << case_root << ':' << cases_exact << ':'
             << route_counts_exact << ':' << negative_rollback_exact << '|'
             << transition_exact << '|' << route;
    const std::string result_sha256 = sha256_hex(semantic.str());

    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r17_owner_epoch_transition.v1\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R17_IDENTITY_SHA256
           << "\",\"status\":\"" << (passed ? "PASS" : "FAIL")
           << "\",\"first_failure\":\"" << first_failure << '"'
           << ",\"parent\":{\"r16_stdout_sha256\":\""
           << r16_stdout_sha256 << "\",\"r16_semantic_exact\":"
           << (r16_semantic_exact ? "true" : "false")
           << ",\"transitive_parents_retained\":"
           << (parents_retained ? "true" : "false")
           << ",\"exact\":" << (parent_exact ? "true" : "false")
           << "},\"source\":{\"envelope_root\":\"" << baseline_root
           << "\",\"envelope_bytes\":" << baseline_bytes.size()
           << ",\"immutable\":"
           << (source_immutable ? "true" : "false")
           << ",\"payload_immutable\":"
           << (payload_immutable ? "true" : "false")
           << "},\"policy_root\":\"" << AL_OWNER_EPOCH_POLICY_ROOT
           << "\",\"canonical\":{\"source_owner_bytes\":"
           << source_owner_after_bytes.size()
           << ",\"source_owner_before_root\":\""
           << sha256_hex(source_owner_before_bytes)
           << "\",\"source_owner_after_root\":\""
           << sha256_hex(source_owner_after_bytes)
           << "\",\"grant_bytes\":" << grant_bytes.size()
           << ",\"grant_root\":\"" << successful_result.grant_root
           << "\",\"receipt_bytes\":" << receipt_bytes.size()
           << ",\"receipt_root\":\"" << successful_result.receipt_root
           << "\",\"active_owner_bytes\":" << active_owner_bytes.size()
           << ",\"active_owner_root\":\""
           << successful_result.active_owner_root
           << "\",\"state_bytes\":" << state_after_bytes.size()
           << ",\"state_before_root\":\"" << state_before_root
           << "\",\"state_after_root\":\"" << state_after_root
           << "\",\"targets_exact\":"
           << (canonical_targets_exact ? "true" : "false")
           << "},\"transition\":{\"source_consumed\":true,"
              "\"active_owner_consumed\":false,\"source_epoch\":0,"
              "\"active_epoch\":1,\"slice_hvp_before\":523,"
              "\"slice_hvp_after\":0,\"cumulative_hvp\":523,"
              "\"all_other_ledger_exact\":"
           << (transition_exact ? "true" : "false")
           << "},\"corpus\":{\"cases\":" << cases.size()
           << ",\"case_root\":\"" << case_root
           << "\",\"all_exact\":" << (cases_exact ? "true" : "false")
           << ",\"route_counts\":[";
    for (std::size_t index = 0U; index < route_counts.size(); ++index) {
        if (index != 0U) report << ',';
        report << route_counts[index];
    }
    report << "],\"route_counts_exact\":"
           << (route_counts_exact ? "true" : "false")
           << ",\"all_failures_state_exact\":"
           << (negative_rollback_exact ? "true" : "false")
           << ",\"records\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        report << "{\"name\":\"" << cases[index].name
               << "\",\"expected\":\"" << cases[index].expected
               << "\",\"observed\":\"" << cases[index].observed
               << "\",\"state_exact\":"
               << (cases[index].state_exact ? "true" : "false")
               << ",\"passed\":"
               << (cases[index].passed ? "true" : "false") << '}';
    }
    report << "]},\"route\":\"" << route << '"'
           << ",\"control_parent_substeps\":1"
              ",\"encoding_and_hash_only\":true"
              ",\"copy_on_write_single_commit\":true"
              ",\"durable_cas\":false,\"concurrent_cas\":false"
              ",\"new_workspaces\":0,\"new_hvp\":0"
              ",\"new_model_hvp\":0,\"new_trials\":0"
              ",\"new_precision_audits\":0,\"new_outer_updates\":0"
              ",\"resume_executed\":false,\"outer6_executed\":false"
              ",\"second_solve_executed\":false"
              ",\"second_substep_executed\":false"
              ",\"macro_frames\":0,\"trajectory_steps\":0"
              ",\"public_schema_created\":false"
              ",\"public_commit_count\":0,\"physics_mutation\":false"
              ",\"live_budget_code_changed\":false"
              ",\"production_policy_changed\":false"
              ",\"timing_admitted\":false,\"speedup_claim\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha256 << "\"}";
    return {passed, report.str()};
}
