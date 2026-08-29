namespace {

constexpr const char* B4E2D7R19R47_IDENTITY_SHA256 =
    "09b1ce5efc452cbbd4220f6bb09f720d23fdb777b65de7d796e9c64d0e41799f";
constexpr const char* B4E2D7R19R47_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r47-filter-compatibility|v2|parent=3f42e2d2:ecbdc13b1cf1ca889fed1f0ce98f4f4613c06eb222d2e132271cd23cdddaf4e3:8b3390babbb896cef73a487bbf2512abc8f2031ff6ad7299d506928ac0d59cf0:FILTER_FEASIBILITY_STEP_CANDIDATE|source=r43-position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;endpoint-30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187|linear=direct-captured-linear-psi-and-h-authoritative;source-minus-predicted-diagnostic;absolute-error<=gamma4*(abs-source+abs-predicted+abs-direct);direct-h=sqrt(2*direct-psi)-binary64-exact;finite-nonnegative;exact-zero-required-for-strict-linearized-inequality-compatibility;no-fitted-tolerance|geometry=endpoint-global-l2<=0.25;source-active-contact-tangent;no-new-contact-evaluation|filter=r46-admitted-exact;admission-necessary-not-sufficient|restoration=ordinary-theta-step-requires-compatible-normal;filter-acceptable-incompatible-step-selects-restoration;restoration-exit-requires-filter-acceptable-next-trqp-compatible;next-state-compatibility-not-claimed|controls=two-subtraction-forward-bound;zero-linear-compatible;positive-linear-restoration;filter-pass-compatibility-fail;route-precedence;parent;source;linear;trust;contact;filter;work;rollback|routes=filter-compatibility-parent-rejected;filter-compatibility-source-rejected;filter-linear-metric-rejected;filter-trust-rejected;filter-contact-rejected;filter-admission-rejected;filter-compatibility-work-rejected;restoration-compatibility-required;filter-normal-step-compatible-candidate|precedence=parent,source,linear,trust,contact,filter,work,compatibility|work=parent-r46-replays1;new-workspaces0;new-jvp0;new-vjp0;new-hvp0;new-model0;new-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46=unchanged;correction=none;candidate-commit=none;filter-runtime-commit=none;restoration-execution=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-filter-compatibility-classification-only)IDENTITY";

bool al_r47_strictly_compatible(double psi, double h) {
    return std::isfinite(psi) && std::isfinite(h)
        && psi == 0.0 && h == 0.0;
}

std::string al_r47_route(bool parent, bool source, bool linear,
    bool trust, bool contact, bool filter, bool work, bool compatible) {
    if (!parent) return "FILTER_COMPATIBILITY_PARENT_REJECTED";
    if (!source) return "FILTER_COMPATIBILITY_SOURCE_REJECTED";
    if (!linear) return "FILTER_LINEAR_METRIC_REJECTED";
    if (!trust) return "FILTER_TRUST_REJECTED";
    if (!contact) return "FILTER_CONTACT_REJECTED";
    if (!filter) return "FILTER_ADMISSION_REJECTED";
    if (!work) return "FILTER_COMPATIBILITY_WORK_REJECTED";
    if (!compatible) return "RESTORATION_COMPATIBILITY_REQUIRED";
    return "FILTER_NORMAL_STEP_COMPATIBLE_CANDIDATE";
}

struct ALR47ReplayCapture {
    bool captured = false;
    ALR46ReplayCapture r46;
};

} // namespace

SplitBoundaryReport run_al_filter_compatibility_controls_impl(
    ALR47ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R47_IDENTITY_PROJECTION)
        == B4E2D7R19R47_IDENTITY_SHA256;
    ALR46ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_filter_globalization_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "ecbdc13b1cf1ca889fed1f0ce98f4f4613c06eb222d2e132271cd23cdddaf4e3"
        && parent.json.find(
            "8b3390babbb896cef73a487bbf2512abc8f2031ff6ad7299d506928ac0d59cf0")
            != std::string::npos
        && parent.json.find("FILTER_FEASIBILITY_STEP_CANDIDATE")
            != std::string::npos;

    ALR43ReplayCapture& r43 = capture.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string endpoint_before =
        al_binary64_vec3_root(r43.projected_endpoint);
    const std::string trial_before =
        al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const bool source_exact = parent_exact && r30.captured
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && endpoint_before
            == "30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_before
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187";

    constexpr double dense_source = 3.2918458374056909e-15;
    constexpr double dense_predicted = 2.6064437667433723e-15;
    constexpr double dense_direct = 6.8540207066231842e-16;
    const double dense_reconstructed = dense_source - dense_predicted;
    const double dense_error = std::abs(dense_reconstructed - dense_direct);
    const double dense_bound = gamma_factor(4U)
        * (std::abs(dense_source) + std::abs(dense_predicted)
            + std::abs(dense_direct));
    const bool dense = identity && dense_error > 0.0
        && dense_error <= dense_bound
        && al_r47_strictly_compatible(0.0, 0.0)
        && !al_r47_strictly_compatible(
            std::numeric_limits<double>::denorm_min(), 0.0)
        && al_r47_route(true, true, true, true, true, true, true, false)
            == "RESTORATION_COMPATIBILITY_REQUIRED"
        && al_r47_route(true, true, true, true, true, true, true, true)
            == "FILTER_NORMAL_STEP_COMPATIBLE_CANDIDATE";

    const double recomputed_linear_psi =
        r43.source_psi - r43.predicted_psi_reduction;
    const double recomputed_linear_h =
        std::sqrt(2.0 * recomputed_linear_psi);
    const double reconstruction_error =
        std::abs(recomputed_linear_psi - r43.linear_psi);
    const double reconstruction_bound = gamma_factor(4U)
        * (std::abs(r43.source_psi)
            + std::abs(r43.predicted_psi_reduction)
            + std::abs(r43.linear_psi));
    const bool direct_h_exact = binary64_bits(r43.linear_h)
        == binary64_bits(std::sqrt(2.0 * r43.linear_psi));
    const bool linear_exact = source_exact && dense
        && std::isfinite(r43.source_psi) && r43.source_psi > 0.0
        && std::isfinite(r43.predicted_psi_reduction)
        && r43.predicted_psi_reduction > 0.0
        && std::isfinite(r43.linear_psi) && r43.linear_psi > 0.0
        && std::isfinite(r43.linear_h) && r43.linear_h > 0.0
        && std::isfinite(recomputed_linear_psi)
        && std::isfinite(recomputed_linear_h)
        && std::isfinite(reconstruction_error)
        && std::isfinite(reconstruction_bound)
        && reconstruction_error <= reconstruction_bound
        && direct_h_exact
        && binary64_bits(capture.source_psi)
            == binary64_bits(r43.source_psi)
        && capture.filter_admitted;
    const bool compatible = linear_exact
        && al_r47_strictly_compatible(r43.linear_psi, r43.linear_h);

    const double endpoint_norm = al_r32_vec3_norm(r43.projected_endpoint);
    const bool trust_exact = linear_exact
        && al_r32_vec3_finite(r43.projected_endpoint)
        && std::isfinite(endpoint_norm) && endpoint_norm <= 0.25;
    std::size_t contact_components = 0U;
    std::size_t active_components = 0U;
    std::size_t tangent_violations = 0U;
    for (std::size_t i = 0U; i < r43.projected_endpoint.size(); ++i) {
        for (int axis = 0; axis < 3; ++axis) {
            ++contact_components;
            const std::size_t component =
                3U * i + static_cast<std::size_t>(axis);
            const std::uint8_t owner = r43.owners[component];
            const double value =
                al_r41_component(r43.projected_endpoint[i], axis);
            if (owner == 0U) continue;
            ++active_components;
            tangent_violations += (owner == 1U && value < 0.0)
                || (owner == 2U && value > 0.0) ? 1U : 0U;
        }
    }
    const bool contact_exact = trust_exact
        && contact_components == 18000U && active_components == 1290U
        && tangent_violations == 0U;
    const bool filter_exact = contact_exact && capture.filter_admitted
        && capture.source_h > capture.trial_h
        && parent.json.find("\"admitted\":true") != std::string::npos;
    const bool work_exact = contact_components == 18000U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && endpoint_before
            == al_binary64_vec3_root(r43.projected_endpoint)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners);

    const std::array<const char*, 9U> expected_routes{{
        "FILTER_COMPATIBILITY_PARENT_REJECTED",
        "FILTER_COMPATIBILITY_SOURCE_REJECTED",
        "FILTER_LINEAR_METRIC_REJECTED",
        "FILTER_TRUST_REJECTED",
        "FILTER_CONTACT_REJECTED",
        "FILTER_ADMISSION_REJECTED",
        "FILTER_COMPATIBILITY_WORK_REJECTED",
        "RESTORATION_COMPATIBILITY_REQUIRED",
        "FILTER_NORMAL_STEP_COMPATIBLE_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t index_value = 0U;
         index_value < expected_routes.size(); ++index_value) {
        std::array<bool, 7U> gates{};
        gates.fill(true);
        if (index_value < gates.size()) gates[index_value] = false;
        const bool route_compatible = index_value + 1U
            == expected_routes.size();
        const std::string observed = al_r47_route(gates[0], gates[1],
            gates[2], gates[3], gates[4], gates[5], gates[6],
            route_compatible);
        routes_exact = routes_exact
            && observed == expected_routes[index_value];
        route_material << expected_routes[index_value] << ':' << observed
                       << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r47_route(parent_exact, source_exact,
        linear_exact, trust_exact, contact_exact, filter_exact,
        work_exact && rollback, compatible);
    const bool passed = identity && parent_exact && source_exact && dense
        && linear_exact && trust_exact && contact_exact && filter_exact
        && work_exact && rollback && routes_exact && !compatible
        && route == "RESTORATION_COMPATIBILITY_REQUIRED";
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R47_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << r43.source_psi << ':' << r43.predicted_psi_reduction << ':'
             << r43.linear_psi << ':' << r43.linear_h << ':'
             << reconstruction_error << ':' << reconstruction_bound << ':'
             << r43.linear_active << ':' << compatible << '|'
             << endpoint_norm << ':' << active_components << ':'
             << tangent_violations << '|' << work_exact << ':' << rollback
             << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r47_filter_compatibility.v2\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R47_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"linear\":{\"source_psi\":" << r43.source_psi
           << ",\"predicted_reduction\":"
           << r43.predicted_psi_reduction << ",\"residual_psi\":"
           << r43.linear_psi << ",\"residual_h\":" << r43.linear_h
           << ",\"reconstructed_psi\":" << recomputed_linear_psi
           << ",\"reconstructed_h\":" << recomputed_linear_h
           << ",\"reconstruction_error\":" << reconstruction_error
           << ",\"reconstruction_bound\":" << reconstruction_bound
           << ",\"direct_h_exact\":"
           << (direct_h_exact ? "true" : "false")
           << ",\"active\":" << r43.linear_active
           << ",\"strictly_compatible\":"
           << (compatible ? "true" : "false") << "}"
           << ",\"geometry\":{\"endpoint_norm\":" << endpoint_norm
           << ",\"trust_radius\":0.25,\"contact_components\":"
           << contact_components << ",\"active_components\":"
           << active_components << ",\"tangent_violations\":"
           << tangent_violations << "}"
           << ",\"filter\":{\"admitted\":"
           << (capture.filter_admitted ? "true" : "false")
           << ",\"ordinary_transaction_authorized\":false"
              ",\"restoration_exit_authorized\":false}"
           << ",\"work\":{\"new_workspaces\":0,\"new_jvp\":0"
              ",\"new_vjp\":0,\"new_hvp\":0,\"new_model\":0"
              ",\"new_trial\":0,\"new_outer\":0,\"exact\":"
           << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"correction_applied\":false,\"candidate_committed\":false"
              ",\"filter_runtime_committed\":false,\"restoration_executed\":false"
              ",\"following_outer_executed\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r46 = std::move(capture);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_filter_compatibility_controls() {
    return run_al_filter_compatibility_controls_impl(nullptr);
}
