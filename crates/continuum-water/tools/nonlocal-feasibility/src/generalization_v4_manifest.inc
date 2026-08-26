namespace {

constexpr const char* AL_R20_R32_SEMANTIC_SHA256 =
    "e7b9acafd3d7aee53b23bff4c3a286f7cf4578e5e566cd69ab04d11c866f0f73";

SmokeFixture al_r20_v4_upper_x_lower_z_torsion() {
    constexpr std::array<int, 3U> dimensions{{8, 3, 3}};
    SmokeFixture result;
    result.name = "v4-filled-upper-x-lower-z-torsion-8x3x3";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 8U);
        const double y = static_cast<double>((index / 8U) % 3U);
        const double z = static_cast<double>(index / 24U);
        result.velocity.push_back({
            0.63 + 0.008 * y - 0.006 * z,
            0.075 * (z - 1.0) - 0.014 * (x - 3.5),
            -0.57 - 0.005 * x + 0.009 * y,
        });
    }
    result.boundary = make_box_owned_shell(dimensions, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = al_r20_v3_box_high(dimensions);
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

SmokeFixture al_r20_v4_upper_x_lower_yz_corner() {
    constexpr std::array<int, 3U> dimensions{{4, 6, 4}};
    SmokeFixture result;
    result.name = "v4-filled-upper-x-lower-y-lower-z-corner-4x6x4";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 4U);
        const double y = static_cast<double>((index / 4U) % 6U);
        const double z = static_cast<double>(index / 24U);
        result.velocity.push_back({
            0.52 + 0.010 * y + 0.004 * z,
            -0.49 - 0.007 * x - 0.003 * z,
            -0.61 - 0.006 * y + 0.005 * x,
        });
    }
    result.boundary = make_box_owned_shell(dimensions, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = al_r20_v3_box_high(dimensions);
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

SmokeFixture al_r20_v4_biaxial_counterflow() {
    constexpr std::array<int, 3U> dimensions{{6, 4, 3}};
    SmokeFixture result;
    result.name = "v4-filled-biaxial-counterflow-6x4x3";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 6U);
        const double y = static_cast<double>((index / 6U) % 4U);
        const double z = static_cast<double>(index / 24U);
        result.velocity.push_back({
            0.21 * (y - 1.5) + 0.018 * (z - 1.0),
            -0.19 * (x - 2.5) + 0.012 * z,
            0.43 + 0.011 * x - 0.009 * y,
        });
    }
    result.boundary = make_box_owned_shell(dimensions, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = al_r20_v3_box_high(dimensions);
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

SmokeFixture al_r20_v4_helical_compression() {
    constexpr std::array<int, 3U> dimensions{{4, 4, 5}};
    SmokeFixture result;
    result.name = "v4-filled-helical-compression-4x4x5";
    result.position = make_lattice_fluid(dimensions);
    const Vec3 center = average_values(result.position);
    result.velocity.reserve(result.position.size());
    for (Vec3 position : result.position) {
        const Vec3 radial = position - center;
        result.velocity.push_back({
            0.08 - 2.0 * radial.x - 0.75 * radial.z,
            -0.05 - 2.6 * radial.y + 0.55 * radial.x,
            0.04 - 2.3 * radial.z + 0.65 * radial.y,
        });
    }
    result.boundary = make_box_owned_shell(dimensions, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = al_r20_v3_box_high(dimensions);
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

SmokeFixture al_r20_v4_alternating_y_layer() {
    constexpr std::array<int, 3U> dimensions{{3, 7, 4}};
    SmokeFixture result;
    result.name = "v4-filled-alternating-y-layer-3x7x4";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 3U);
        const std::size_t y_index = (index / 3U) % 7U;
        const double y = static_cast<double>(y_index);
        const double z = static_cast<double>(index / 21U);
        const double sign = y_index % 2U == 0U ? -1.0 : 1.0;
        result.velocity.push_back({
            0.16 * (y - 3.0) + 0.015 * z,
            0.58 + 0.010 * x - 0.006 * z,
            0.22 * sign + 0.020 * (x - 1.0),
        });
    }
    result.boundary = make_box_owned_shell(dimensions, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = al_r20_v3_box_high(dimensions);
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

std::vector<ALR20ManifestCase> al_r20_v4_manifest_cases() {
    std::vector<ALR20ManifestCase> cases;
    cases.push_back({
        "BLIND_HOLDOUT_V4", "r20r33-source-only-before-operator-observation",
        "filled-upper-x-lower-z-torsion", 0U,
        al_r20_v4_upper_x_lower_z_torsion(), {}});
    cases.push_back({
        "BLIND_HOLDOUT_V4", "r20r33-source-only-before-operator-observation",
        "filled-upper-x-lower-y-lower-z-corner", 0U,
        al_r20_v4_upper_x_lower_yz_corner(), {}});
    cases.push_back({
        "BLIND_HOLDOUT_V4", "r20r33-source-only-before-operator-observation",
        "filled-biaxial-counterflow", 0U,
        al_r20_v4_biaxial_counterflow(), {}});
    cases.push_back({
        "BLIND_HOLDOUT_V4", "r20r33-source-only-before-operator-observation",
        "filled-helical-compression", 0U,
        al_r20_v4_helical_compression(), {}});
    cases.push_back({
        "BLIND_HOLDOUT_V4", "r20r33-source-only-before-operator-observation",
        "filled-alternating-y-layer", 0U,
        al_r20_v4_alternating_y_layer(), {}});
    for (ALR20ManifestCase& value : cases)
        value.source_root = al_r20_source_root(value);
    return cases;
}

std::string al_r20_v4_manifest_root(
    const std::vector<ALR20ManifestCase>& cases) {
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r33-v4-manifest.v1|"
        << AL_R20_R32_SEMANTIC_SHA256 << ':' << cases.size() << '|';
    for (const ALR20ManifestCase& value : cases)
        semantic << value.source_root << ';';
    semantic << "operator-executed=0|projection-executed=0|"
        "preflight-executed=0|solver-executed=0|timing-admitted=0";
    return sha256_hex(semantic.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_manifest_controls() {
    const std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    constexpr std::array<std::size_t, 5U> expected_counts{{
        72U, 96U, 72U, 80U, 84U}};
    bool exact = cases.size() == expected_counts.size();
    std::size_t total_particles = 0U;
    std::vector<std::string> source_roots;
    std::vector<std::string> geometry_roots;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        const ALR20ManifestCase& value = cases[index];
        exact = exact && value.role == "BLIND_HOLDOUT_V4"
            && value.provenance
                == "r20r33-source-only-before-operator-observation"
            && al_r20_case_finite_and_owned(value)
            && index < expected_counts.size()
            && value.fixture.position.size() == expected_counts[index];
        total_particles += value.fixture.position.size();
        source_roots.push_back(value.source_root);
        geometry_roots.push_back(value.fixture.geometry_sha256);
    }
    std::sort(source_roots.begin(), source_roots.end());
    std::sort(geometry_roots.begin(), geometry_roots.end());
    exact = exact && total_particles == 404U
        && std::adjacent_find(source_roots.begin(), source_roots.end())
            == source_roots.end()
        && std::adjacent_find(geometry_roots.begin(), geometry_roots.end())
            == geometry_roots.end();
    const std::string result_sha = al_r20_v4_manifest_root(cases);
    const std::string route = exact
        ? "V4_HOLDOUT_MANIFEST_FROZEN"
        : "V4_HOLDOUT_MANIFEST_REJECTED";
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r33_v4_manifest.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_terminal_trajectory_sha256\":\""
        << AL_R20_R32_SEMANTIC_SHA256 << "\""
        << ",\"case_count\":" << cases.size()
        << ",\"total_particles\":" << total_particles
        << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20ManifestCase& value = cases[index];
        report << "{\"name\":\"" << value.fixture.name
            << "\",\"role\":\"" << value.role
            << "\",\"contact_regime\":\"" << value.contact_regime
            << "\",\"fluid_samples\":" << value.fixture.position.size()
            << ",\"boundary_samples\":" << value.fixture.boundary.size()
            << ",\"geometry_sha256\":\""
            << value.fixture.geometry_sha256
            << "\",\"source_sha256\":\"" << value.source_root << "\"}";
    }
    report << "]"
        << ",\"operator_executed\":false,\"projection_executed\":false"
        << ",\"preflight_executed\":false,\"solver_executed\":false"
        << ",\"timing_admitted\":false,\"generalization_authority\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
