namespace {

constexpr const char* B4E2D7R19R62_IDENTITY_SHA256 =
    "aebba856f1088d41440d18d0948636941fa9e3fa3c21d93fc776a4e518776422";
constexpr const char* B4E2D7R19R62_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r62-restoration-exit-transaction|v1|parent=fa7d48de:cdfe69bfccba8161708a92636a256313a3efbb27a8f6e7bb1bfa915a04047cb4:18794b59910842019f4d99ed468cb55ffa6b925bbb3c7c0b4ccfbe46c0377436:TOPOLOGY_OWNED_ROW_LOCAL_AUDIT_CANDIDATE|source=position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;witness-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;state-a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2;filter-source-entry-6dd5f24766c14efd390df5fe178b0ee32d6a7afa5cfc51b517762c48175ad550;filter-next-19c1311bf84aaa236a8ce4ec82359ffadb6db404a721ea804f66daabef472c19;r61-candidate-upper-78472fcb347565bdcc4be443a3cbf220cc5459fd5959a1bcc1c8bd1d428e55bf;active-82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3|mapping=promoted-position-exact-r43;witness-dimensionless-cached-next-normal-only;spacing-times-witness-not-applied|fresh=passive-r61-capture;static-index1;workspace-at-r43;superset-anchored-r43;filtered-pairs-equal-workspace;coordinate-f-h-psi;source-to-r43-contact36000;normal-endpoint-contact-diagnostic36000|filter=gamma0.5;empty-old-filter;source-entry-inserted;candidate-acceptable-h-branch;canonical-one-entry-root;composite-al-merit-excluded|trust=next-radius0.0625;normal-radius0.03125;witness-global-l2-within-normal;all-r43-contact-intervals-contain-witness;diagnostic-normal-endpoint-contact-safe|certificate=reuse-r61-topology-owned-owner;same-r43-workspace-and-witness;one-directed-jvp;one-row-scan;candidate-positive0;candidate-active-root-exact;candidate-upper-root-exact;binary128-none|payload=copy-on-write;position-root;filter-root;radius-bits;witness-root;superset-root;workspace-topology-root;certificate-root;phase-ordinary-trqp-ready;publish-only-after-all-gates|controls=parent;source;capture;workspace;topology;coordinate;filter;contact;normal;certificate;payload;atomic-publication;work;rollback;route-precedence;valid-publish;corrupt-position-reject;corrupt-witness-reject;corrupt-filter-reject;corrupt-radius-reject;corrupt-topology-reject;corrupt-certificate-reject;gate-failure-no-publish;publish-once|routes=restoration-exit-parent-rejected;restoration-exit-source-rejected;restoration-exit-capture-rejected;restoration-exit-workspace-rejected;restoration-exit-topology-rejected;restoration-exit-coordinate-rejected;restoration-exit-filter-rejected;restoration-exit-contact-rejected;restoration-exit-normal-rejected;restoration-exit-certificate-rejected;restoration-exit-payload-rejected;restoration-exit-atomic-publication-rejected;restoration-exit-work-rejected;restoration-exit-transaction-candidate|precedence=parent,source,capture,workspace,topology,coordinate,filter,contact,normal,certificate,payload,atomic,work,candidate|work=parent-r61-replays1;new-static-index1;new-r43-superset1;new-r43-workspace1;workspace-release1;new-coordinate-evaluations1;new-contact-tests72000;new-directed-jvp1;new-row-scan1;transaction-controls9;new-binary128-0;new-density-sweeps0;new-vjp0;new-hvp0;new-model0;new-nonlinear-normal-workspace0;new-tangential-step0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59-r60-r61=unchanged;r61-public-output=byte-exact;promotion=private-payload-only;r43-runtime-commit=none;witness-apply=none;cached-normal-consume=none;filter-runtime-commit=none;restoration-runtime-exit=none;switching=none;trust-runtime-update=none;multiplier-update=none;hessian-update=none;following-outer=none;tolerance=none;gamma-production-selection=none;capacity-change=none;timing=none;runtime=none;production=none|credit=one-private-restoration-exit-transaction-candidate-only)IDENTITY";

constexpr double AL_R62_FILTER_GAMMA = 0.5;
constexpr double AL_R62_NEXT_RADIUS = 0.0625;
constexpr double AL_R62_NORMAL_RADIUS = 0.03125;

struct ALR62Payload {
    std::string position_root;
    std::string filter_root;
    double radius = 0.0;
    std::string witness_root;
    std::string superset_root;
    std::string workspace_topology_root;
    std::string certificate_root;
    std::string phase;
};

std::string al_r62_payload_root(const ALR62Payload& value) {
    std::ostringstream material;
    material << "nextengine.nonlocal.restoration-exit-payload|v1|position="
             << value.position_root << "|filter=" << value.filter_root
             << "|radius=" << binary64_bits(value.radius) << "|witness="
             << value.witness_root << "|superset=" << value.superset_root
             << "|workspace=" << value.workspace_topology_root
             << "|certificate=" << value.certificate_root << "|phase="
             << value.phase;
    return sha256_hex(material.str());
}

bool al_r62_payload_equal(const ALR62Payload& lhs,
    const ALR62Payload& rhs) {
    return lhs.position_root == rhs.position_root
        && lhs.filter_root == rhs.filter_root
        && binary64_bits(lhs.radius) == binary64_bits(rhs.radius)
        && lhs.witness_root == rhs.witness_root
        && lhs.superset_root == rhs.superset_root
        && lhs.workspace_topology_root == rhs.workspace_topology_root
        && lhs.certificate_root == rhs.certificate_root
        && lhs.phase == rhs.phase;
}

struct ALR62Publication {
    bool published = false;
    std::size_t publish_count = 0U;
    ALR62Payload payload;
    std::vector<Vec3> position;
    std::vector<Vec3> cached_normal;
};

bool al_r62_try_publish(ALR62Publication& target,
    const ALR62Payload& proposed, const ALR62Payload& expected,
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& cached_normal, bool gates) {
    if (target.published || !gates || position.empty()
        || position.size() != cached_normal.size()
        || !al_r48_vec_finite(position) || !al_r48_vec_finite(cached_normal)
        || !al_r62_payload_equal(proposed, expected)
        || al_binary64_vec3_root(position) != proposed.position_root
        || al_binary64_vec3_root(cached_normal) != proposed.witness_root)
        return false;
    target.payload = proposed;
    target.position = position;
    target.cached_normal = cached_normal;
    target.published = true;
    ++target.publish_count;
    return true;
}

struct ALR62AtomicControls {
    bool exact = false;
    std::size_t cases = 0U;
    std::string root;
};

ALR62AtomicControls al_r62_atomic_controls(const ALR62Payload& payload,
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& cached_normal) {
    ALR62AtomicControls result;
    ALR62Publication valid;
    const bool valid_publish = al_r62_try_publish(valid, payload, payload,
        position, cached_normal, true);
    ++result.cases;
    const bool repeat_rejected = !al_r62_try_publish(valid, payload, payload,
        position, cached_normal, true) && valid.publish_count == 1U;
    ++result.cases;

    std::vector<Vec3> bad_position = position;
    bad_position[0].x = std::nextafter(
        bad_position[0].x, std::numeric_limits<double>::infinity());
    ALR62Publication position_slot;
    const bool position_rejected = !al_r62_try_publish(position_slot,
        payload, payload, bad_position, cached_normal, true)
        && !position_slot.published;
    ++result.cases;

    std::vector<Vec3> bad_normal = cached_normal;
    bad_normal[0].x = std::nextafter(
        bad_normal[0].x, std::numeric_limits<double>::infinity());
    ALR62Publication normal_slot;
    const bool normal_rejected = !al_r62_try_publish(normal_slot,
        payload, payload, position, bad_normal, true)
        && !normal_slot.published;
    ++result.cases;

    ALR62Payload bad_filter = payload;
    bad_filter.filter_root.push_back('x');
    ALR62Publication filter_slot;
    const bool filter_rejected = !al_r62_try_publish(filter_slot,
        bad_filter, payload, position, cached_normal, true)
        && !filter_slot.published;
    ++result.cases;

    ALR62Payload bad_radius = payload;
    bad_radius.radius = std::nextafter(
        bad_radius.radius, std::numeric_limits<double>::infinity());
    ALR62Publication radius_slot;
    const bool radius_rejected = !al_r62_try_publish(radius_slot,
        bad_radius, payload, position, cached_normal, true)
        && !radius_slot.published;
    ++result.cases;

    ALR62Payload bad_topology = payload;
    bad_topology.workspace_topology_root.push_back('x');
    ALR62Publication topology_slot;
    const bool topology_rejected = !al_r62_try_publish(topology_slot,
        bad_topology, payload, position, cached_normal, true)
        && !topology_slot.published;
    ++result.cases;

    ALR62Payload bad_certificate = payload;
    bad_certificate.certificate_root.push_back('x');
    ALR62Publication certificate_slot;
    const bool certificate_rejected = !al_r62_try_publish(certificate_slot,
        bad_certificate, payload, position, cached_normal, true)
        && !certificate_slot.published;
    ++result.cases;

    ALR62Publication gate_slot;
    const bool gate_rejected = !al_r62_try_publish(gate_slot, payload,
        payload, position, cached_normal, false) && !gate_slot.published;
    ++result.cases;

    result.exact = valid_publish && valid.published
        && al_r62_payload_equal(valid.payload, payload)
        && exact_vec3_values(valid.position, position)
        && exact_vec3_values(valid.cached_normal, cached_normal)
        && repeat_rejected && position_rejected && normal_rejected
        && filter_rejected && radius_rejected && topology_rejected
        && certificate_rejected && gate_rejected && result.cases == 9U;
    std::ostringstream material;
    material << valid_publish << ':' << repeat_rejected << ':'
             << position_rejected << ':' << normal_rejected << ':'
             << filter_rejected << ':' << radius_rejected << ':'
             << topology_rejected << ':' << certificate_rejected << ':'
             << gate_rejected << ':' << result.cases << ':'
             << al_r62_payload_root(payload);
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r62_route(bool parent, bool source, bool capture,
    bool workspace, bool topology, bool coordinate, bool filter,
    bool contact, bool normal, bool certificate, bool payload,
    bool atomic, bool work) {
    if (!parent) return "RESTORATION_EXIT_PARENT_REJECTED";
    if (!source) return "RESTORATION_EXIT_SOURCE_REJECTED";
    if (!capture) return "RESTORATION_EXIT_CAPTURE_REJECTED";
    if (!workspace) return "RESTORATION_EXIT_WORKSPACE_REJECTED";
    if (!topology) return "RESTORATION_EXIT_TOPOLOGY_REJECTED";
    if (!coordinate) return "RESTORATION_EXIT_COORDINATE_REJECTED";
    if (!filter) return "RESTORATION_EXIT_FILTER_REJECTED";
    if (!contact) return "RESTORATION_EXIT_CONTACT_REJECTED";
    if (!normal) return "RESTORATION_EXIT_NORMAL_REJECTED";
    if (!certificate) return "RESTORATION_EXIT_CERTIFICATE_REJECTED";
    if (!payload) return "RESTORATION_EXIT_PAYLOAD_REJECTED";
    if (!atomic) return "RESTORATION_EXIT_ATOMIC_PUBLICATION_REJECTED";
    if (!work) return "RESTORATION_EXIT_WORK_REJECTED";
    return "RESTORATION_EXIT_TRANSACTION_CANDIDATE";
}

} // namespace

SplitBoundaryReport run_al_restoration_exit_transaction_controls() {
    const bool identity = sha256_hex(B4E2D7R19R62_IDENTITY_PROJECTION)
        == B4E2D7R19R62_IDENTITY_SHA256;
    ALR61ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_certificate_integration_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "cdfe69bfccba8161708a92636a256313a3efbb27a8f6e7bb1bfa915a04047cb4"
        && parent.json.find(
            "18794b59910842019f4d99ed468cb55ffa6b925bbb3c7c0b4ccfbe46c0377436")
            != std::string::npos
        && parent.json.find("TOPOLOGY_OWNED_ROW_LOCAL_AUDIT_CANDIDATE")
            != std::string::npos;

    ALR60ReplayCapture& r60 = capture.r60;
    ALR59ReplayCapture& r59_capture = r60.r59;
    ALR58ReplayCapture& r58_capture = r59_capture.r58;
    ALR57Result& r58 = r58_capture.result;
    ALR57ReplayCapture& r57_capture = r58_capture.r57;
    ALR56ReplayCapture& r56_capture = r57_capture.r56;
    ALR55ReplayCapture& r55_capture = r56_capture.r55;
    ALR54ReplayCapture& r54_capture = r55_capture.r54;
    ALR53ReplayCapture& r53_capture = r54_capture.r53;
    ALR51ReplayCapture& r51_capture = r53_capture.r52.r51;
    ALR46ReplayCapture& r46 =
        r51_capture.r50.r49.r48.r47.r46;
    ALR43ReplayCapture& r43 = r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;

    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string witness_before = al_binary64_vec3_root(r58.witness);
    const std::string state_before = r58.state_root;
    const std::string old_superset_before =
        al_r42_superset_root(r43.r42.superset);
    const std::string r61_upper_before = capture.audit.candidate_root;
    const std::string r61_active_before = capture.audit.candidate_active_root;
    const bool source_exact = parent_exact && r30.captured
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_before
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187"
        && witness_before
            == "040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd"
        && state_before
            == "a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2"
        && r61_upper_before
            == "78472fcb347565bdcc4be443a3cbf220cc5459fd5959a1bcc1c8bd1d428e55bf"
        && r61_active_before
            == "82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3";
    const bool capture_exact = source_exact && capture.audit.owner_exact
        && capture.audit.candidate_positive == 0U
        && capture.audit.pair_passes == 1U
        && capture.audit.row_scans == 1U && capture.controls.cases == 6U;

    ALSparseWorkTrace workspace_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    JointNeighborhood promoted_superset = b4ep3_build_superset(
        tagged_points(r43.projected_trial), binding);
    JointNeighborhood promoted_filtered = b4ep3_filter_superset(
        tagged_points(r43.projected_trial), promoted_superset);
    ALNormalizedWorkspace promoted = build_al_normalized_sparse_workspace(
        r43.projected_trial, binding, r30.dual, r30.theta,
        &workspace_work, &static_work, &adjacency_work);
    const bool workspace_exact = capture_exact && index.passed
        && binding.passed && promoted_superset.passed
        && promoted_filtered.passed && promoted.passed;
    const ALR42Coverage promoted_coverage =
        al_r42_coverage(promoted_superset, promoted_filtered);
    const std::string promoted_superset_root =
        al_r42_superset_root(promoted_superset);
    const std::string promoted_topology_root =
        al_r42_superset_root(promoted.neighborhood);
    const bool topology_exact = workspace_exact && promoted_coverage.exact
        && promoted_filtered.pairs == promoted.neighborhood.pairs
        && al_r42_superset_root(promoted_filtered)
            == promoted_topology_root;

    const ALR40HingeMetric promoted_metric =
        al_r40_hinge_metric(promoted.support.constraint);
    const ALNormalizedInner promoted_inner = evaluate_al_normalized_inner(
        promoted.support, r43.projected_trial, r30.predicted_position);
    const bool coordinate_exact = topology_exact && promoted_metric.finite
        && promoted_inner.passed
        && binary64_bits(promoted_inner.inertia)
            == binary64_bits(r46.trial_f)
        && binary64_bits(promoted_metric.norm) == binary64_bits(r46.trial_h)
        && binary64_bits(promoted_metric.psi) == binary64_bits(r46.trial_psi)
        && promoted_metric.active == 464U;
    const ALR46FilterDecision filter_decision = al_r46_filter_decision(
        r46.source_h, promoted_metric.norm, r46.objective_reduction,
        AL_R62_FILTER_GAMMA);
    const std::string source_entry_root =
        al_r46_filter_entry_root(r46.source_h, r46.source_f);
    const std::string next_filter_root = sha256_hex(
        std::string("nextengine.nonlocal.filter|v1|entries=1|")
        + source_entry_root);
    const bool filter_exact = coordinate_exact && r46.filter_admitted
        && source_entry_root
            == "6dd5f24766c14efd390df5fe178b0ee32d6a7afa5cfc51b517762c48175ad550"
        && next_filter_root
            == "19c1311bf84aaa236a8ce4ec82359ffadb6db404a721ea804f66daabef472c19"
        && filter_decision.acceptable && filter_decision.h_branch
        && !filter_decision.f_branch && filter_decision.h_slack > 0.0;

    const ALR41ContactAudit promoted_contact = al_r41_contact_audit(
        r30.position, r43.projected_trial, fixture);
    std::vector<Vec3> normal_endpoint(r43.projected_trial.size());
    std::vector<Vec3> normal_lower(r43.projected_trial.size());
    std::vector<Vec3> normal_upper(r43.projected_trial.size());
    bool intervals_exact = filter_exact
        && r58.witness.size() == r43.projected_trial.size();
    std::size_t active_faces = 0U;
    for (std::size_t i = 0U; i < r43.projected_trial.size(); ++i) {
        normal_endpoint[i] = r43.projected_trial[i] + SPACING * r58.witness[i];
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face = al_r41_component(fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            if (y < low_face) {
                low = 0.0;
                ++active_faces;
            } else if (y > high_face) {
                high = 0.0;
                ++active_faces;
            }
            al_r43_component(normal_lower[i], axis) = low;
            al_r43_component(normal_upper[i], axis) = high;
            const double value = al_r41_component(r58.witness[i], axis);
            intervals_exact = intervals_exact && std::isfinite(low)
                && std::isfinite(high) && low <= value && value <= high;
        }
    }
    const double witness_norm = al_r32_vec3_norm(r58.witness);
    const ALR41ContactAudit normal_contact = al_r41_contact_audit(
        r43.projected_trial, normal_endpoint, fixture);
    const bool contact_exact = filter_exact && promoted_contact.finite
        && promoted_contact.tests == 36000U
        && promoted_contact.newly_penetrating == 0U
        && promoted_contact.worsened == 0U
        && promoted_contact.trial_maximum <= promoted_contact.source_maximum
        && normal_contact.finite && normal_contact.tests == 36000U
        && normal_contact.newly_penetrating == 0U
        && normal_contact.worsened == 0U
        && normal_contact.trial_maximum <= normal_contact.source_maximum;
    const bool normal_exact = contact_exact && intervals_exact
        && active_faces == 1290U && al_r48_vec_finite(normal_endpoint)
        && std::isfinite(witness_norm)
        && AL_R62_NORMAL_RADIUS == 0.5 * AL_R62_NEXT_RADIUS
        && witness_norm <= AL_R62_NORMAL_RADIUS;

    ALR61Audit audit;
    if (normal_exact)
        audit = al_r61_topology_owned_audit(promoted, r58.witness);
    const bool certificate_exact = normal_exact && audit.owner_exact
        && audit.candidate_positive == 0U
        && audit.candidate_root == r61_upper_before
        && audit.candidate_active_root == r61_active_before
        && audit.relation_exact && audit.pair_passes == 1U
        && audit.row_scans == 1U;

    ALR62Payload payload;
    payload.position_root = trial_before;
    payload.filter_root = next_filter_root;
    payload.radius = AL_R62_NEXT_RADIUS;
    payload.witness_root = witness_before;
    payload.superset_root = promoted_superset_root;
    payload.workspace_topology_root = promoted_topology_root;
    payload.certificate_root = audit.candidate_root;
    payload.phase = "ordinary-trqp-ready";
    const std::string payload_root = al_r62_payload_root(payload);
    const bool payload_exact = certificate_exact
        && payload.position_root == trial_before
        && payload.filter_root == next_filter_root
        && binary64_bits(payload.radius) == binary64_bits(0.0625)
        && payload.witness_root == witness_before
        && !payload.superset_root.empty()
        && !payload.workspace_topology_root.empty()
        && payload.certificate_root == r61_upper_before
        && payload.phase == "ordinary-trqp-ready";

    if (promoted.passed)
        release_al_normalized_workspace(promoted, &workspace_work);
    const ALR62AtomicControls atomic_controls = al_r62_atomic_controls(
        payload, r43.projected_trial, r58.witness);
    const bool work_exact = workspace_work.workspace_builds == 1U
        && workspace_work.workspace_releases == 1U
        && workspace_work.live_workspaces == 0U
        && workspace_work.maximum_live_workspaces == 1U
        && !workspace_work.lifecycle_underflow
        && workspace_work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 1U
        && adjacency_work.workspace_builds == 1U
        && promoted_contact.tests + normal_contact.tests == 72000U
        && audit.pair_passes == 1U && audit.row_scans == 1U
        && atomic_controls.cases == 9U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && witness_before == al_binary64_vec3_root(r58.witness)
        && state_before == r58.state_root
        && old_superset_before == al_r42_superset_root(r43.r42.superset)
        && r61_upper_before == capture.audit.candidate_root
        && r61_active_before == capture.audit.candidate_active_root;

    const bool all_before_atomic = identity && parent_exact && source_exact
        && capture_exact && workspace_exact && topology_exact
        && coordinate_exact && filter_exact && contact_exact && normal_exact
        && certificate_exact && payload_exact && work_exact && rollback;
    ALR62Publication publication;
    const bool published = al_r62_try_publish(publication, payload, payload,
        r43.projected_trial, r58.witness,
        all_before_atomic && atomic_controls.exact);
    const bool atomic_exact = atomic_controls.exact && published
        && publication.published && publication.publish_count == 1U
        && al_r62_payload_equal(publication.payload, payload)
        && exact_vec3_values(publication.position, r43.projected_trial)
        && exact_vec3_values(publication.cached_normal, r58.witness);

    const std::array<const char*, 14U> expected_routes{{
        "RESTORATION_EXIT_PARENT_REJECTED",
        "RESTORATION_EXIT_SOURCE_REJECTED",
        "RESTORATION_EXIT_CAPTURE_REJECTED",
        "RESTORATION_EXIT_WORKSPACE_REJECTED",
        "RESTORATION_EXIT_TOPOLOGY_REJECTED",
        "RESTORATION_EXIT_COORDINATE_REJECTED",
        "RESTORATION_EXIT_FILTER_REJECTED",
        "RESTORATION_EXIT_CONTACT_REJECTED",
        "RESTORATION_EXIT_NORMAL_REJECTED",
        "RESTORATION_EXIT_CERTIFICATE_REJECTED",
        "RESTORATION_EXIT_PAYLOAD_REJECTED",
        "RESTORATION_EXIT_ATOMIC_PUBLICATION_REJECTED",
        "RESTORATION_EXIT_WORK_REJECTED",
        "RESTORATION_EXIT_TRANSACTION_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 13U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r62_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            gates[12]);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r62_route(parent_exact, source_exact,
        capture_exact, workspace_exact, topology_exact, coordinate_exact,
        filter_exact, contact_exact, normal_exact, certificate_exact,
        payload_exact, atomic_exact, work_exact && rollback);
    const bool passed = identity && parent_exact && source_exact
        && capture_exact && workspace_exact && topology_exact
        && coordinate_exact && filter_exact && contact_exact && normal_exact
        && certificate_exact && payload_exact && atomic_exact && work_exact
        && rollback && routes_exact
        && route == "RESTORATION_EXIT_TRANSACTION_CANDIDATE";

    const std::string normal_endpoint_root =
        al_binary64_vec3_root(normal_endpoint);
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R62_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << promoted_superset_root << ':' << promoted_topology_root << ':'
             << promoted_coverage.root << '|'
             << promoted_inner.inertia << ':' << promoted_metric.norm << ':'
             << promoted_metric.psi << ':' << promoted_metric.active << ':'
             << filter_decision.h_slack << ':' << source_entry_root << ':'
             << next_filter_root << '|'
             << promoted_contact.root << ':' << normal_contact.root << ':'
             << normal_endpoint_root << ':' << active_faces << ':'
             << witness_norm << '|'
             << audit.candidate_root << ':' << audit.candidate_active_root
             << ':' << audit.candidate_positive << '|'
             << payload_root << ':' << atomic_controls.root << ':'
             << publication.publish_count << '|' << work_exact << ':'
             << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r62_restoration_exit_transaction.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R62_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"mapping\":{\"promoted_position_root\":\"" << trial_before
           << "\",\"cached_normal_root\":\"" << witness_before
           << "\",\"normal_applied_to_position\":false}"
           << ",\"topology\":{\"superset_root\":\"" << promoted_superset_root
           << "\",\"workspace_root\":\"" << promoted_topology_root
           << "\",\"coverage_root\":\"" << promoted_coverage.root
           << "\",\"coverage_exact\":"
           << (promoted_coverage.exact ? "true" : "false") << "}"
           << ",\"coordinates\":{\"f\":" << promoted_inner.inertia
           << ",\"h\":" << promoted_metric.norm << ",\"psi\":"
           << promoted_metric.psi << ",\"active\":" << promoted_metric.active
           << ",\"gamma\":" << AL_R62_FILTER_GAMMA
           << ",\"h_slack\":" << filter_decision.h_slack
           << ",\"h_branch\":"
           << (filter_decision.h_branch ? "true" : "false")
           << ",\"filter_root\":\"" << next_filter_root << "\"}"
           << ",\"contact\":{\"restoration_tests\":" << promoted_contact.tests
           << ",\"restoration_new\":" << promoted_contact.newly_penetrating
           << ",\"restoration_worsened\":" << promoted_contact.worsened
           << ",\"normal_tests\":" << normal_contact.tests
           << ",\"normal_new\":" << normal_contact.newly_penetrating
           << ",\"normal_worsened\":" << normal_contact.worsened
           << ",\"normal_endpoint_root\":\"" << normal_endpoint_root << "\"}"
           << ",\"normal\":{\"next_radius\":" << AL_R62_NEXT_RADIUS
           << ",\"normal_radius\":" << AL_R62_NORMAL_RADIUS
           << ",\"witness_norm\":" << witness_norm
           << ",\"active_faces\":" << active_faces
           << ",\"intervals_exact\":"
           << (intervals_exact ? "true" : "false") << "}"
           << ",\"certificate\":{\"positive\":" << audit.candidate_positive
           << ",\"maximum_upper\":" << audit.candidate_maximum
           << ",\"upper_root\":\"" << audit.candidate_root
           << "\",\"active_root\":\"" << audit.candidate_active_root << "\"}"
           << ",\"transaction\":{\"payload_root\":\"" << payload_root
           << "\",\"phase\":\"" << payload.phase
           << "\",\"published\":" << (publication.published ? "true" : "false")
           << ",\"publish_count\":" << publication.publish_count
           << ",\"control_cases\":" << atomic_controls.cases
           << ",\"controls_root\":\"" << atomic_controls.root << "\"}"
           << ",\"work\":{\"workspace_builds\":"
           << workspace_work.workspace_builds << ",\"workspace_releases\":"
           << workspace_work.workspace_releases << ",\"contact_tests\":"
           << promoted_contact.tests + normal_contact.tests
           << ",\"directed_jvp\":" << audit.pair_passes
           << ",\"row_scans\":" << audit.row_scans
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"private_promotion_candidate\":"
           << (publication.published ? "true" : "false")
           << ",\"runtime_state_mutated\":false"
              ",\"cached_normal_consumed\":false"
              ",\"filter_runtime_committed\":false"
              ",\"trust_runtime_updated\":false"
              ",\"tangential_step_executed\":false"
              ",\"following_outer_executed\":false"
              ",\"binary128_runtime\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {passed, report.str()};
}
