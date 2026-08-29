namespace {

constexpr const char* AL_R20_VERIFIED_INVERSE_SHA256 =
    "afca1c777053172169e227bcf3625ad7c828892520340db5eb45c98d53b4374a";

Vec3 al_r20_v3_box_high(std::array<int, 3U> dimensions) {
    return {
        RADIUS + static_cast<double>(dimensions[0] - 1) * SPACING,
        RADIUS + static_cast<double>(dimensions[1] - 1) * SPACING,
        RADIUS + static_cast<double>(dimensions[2] - 1) * SPACING,
    };
}

SmokeFixture al_r20_v3_lower_x_upper_z_edge() {
    constexpr std::array<int, 3U> dimensions{{6, 3, 4}};
    SmokeFixture result;
    result.name = "v3-filled-lower-x-upper-z-edge-6x3x4";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 6U);
        const double y = static_cast<double>((index / 6U) % 3U);
        const double z = static_cast<double>(index / 18U);
        result.velocity.push_back({
            -0.62 - 0.007 * y - 0.004 * z,
            0.055 * (y - 1.0) + 0.006 * x,
            0.55 + 0.009 * x + 0.005 * z,
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

SmokeFixture al_r20_v3_lower_xy_upper_z_corner() {
    constexpr std::array<int, 3U> dimensions{{5, 5, 3}};
    SmokeFixture result;
    result.name = "v3-filled-lower-x-lower-y-upper-z-corner-5x5x3";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 5U);
        const double y = static_cast<double>((index / 5U) % 5U);
        const double z = static_cast<double>(index / 25U);
        result.velocity.push_back({
            -0.48 - 0.006 * z - 0.003 * y,
            -0.44 - 0.008 * x - 0.002 * z,
            0.58 + 0.005 * y + 0.004 * x,
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

SmokeFixture al_r20_v3_upper_y_shear_layer() {
    constexpr std::array<int, 3U> dimensions{{7, 2, 5}};
    SmokeFixture result;
    result.name = "v3-filled-upper-y-shear-layer-7x2x5";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 7U);
        const double y = static_cast<double>((index / 7U) % 2U);
        const double z = static_cast<double>(index / 14U);
        result.velocity.push_back({
            0.18 * (z - 2.0) + 0.012 * y,
            0.66 + 0.006 * x + 0.004 * z,
            -0.045 * (x - 3.0) - 0.010 * y,
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

SmokeFixture al_r20_v3_radial_compression() {
    constexpr std::array<int, 3U> dimensions{{5, 3, 4}};
    SmokeFixture result;
    result.name = "v3-filled-radial-compression-5x3x4";
    result.position = make_lattice_fluid(dimensions);
    const Vec3 center = average_values(result.position);
    result.velocity.reserve(result.position.size());
    for (Vec3 position : result.position) {
        const Vec3 radial = position - center;
        result.velocity.push_back({
            0.11 - 2.4 * radial.x + 0.15 * radial.z,
            -0.07 - 2.1 * radial.y - 0.08 * radial.x,
            0.09 - 2.7 * radial.z + 0.10 * radial.y,
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

std::vector<ALR20ManifestCase> al_r20_v3_manifest_cases() {
    std::vector<ALR20ManifestCase> cases;
    cases.push_back({
        "BLIND_HOLDOUT_V3", "r20r9-source-only-before-operator-observation",
        "filled-lower-x-upper-z-edge", 0U,
        al_r20_v3_lower_x_upper_z_edge(), {}});
    cases.push_back({
        "BLIND_HOLDOUT_V3", "r20r9-source-only-before-operator-observation",
        "filled-lower-x-lower-y-upper-z-corner", 0U,
        al_r20_v3_lower_xy_upper_z_corner(), {}});
    cases.push_back({
        "BLIND_HOLDOUT_V3", "r20r9-source-only-before-operator-observation",
        "filled-upper-y-shear-layer", 0U,
        al_r20_v3_upper_y_shear_layer(), {}});
    cases.push_back({
        "BLIND_HOLDOUT_V3", "r20r9-source-only-before-operator-observation",
        "filled-radial-compression", 0U,
        al_r20_v3_radial_compression(), {}});
    for (ALR20ManifestCase& value : cases)
        value.source_root = al_r20_source_root(value);
    return cases;
}

std::string al_r20_v3_manifest_root(
    const std::vector<ALR20ManifestCase>& cases) {
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r9-v3-manifest.v1|"
             << AL_R20_VERIFIED_INVERSE_SHA256 << ':' << cases.size() << '|';
    for (const ALR20ManifestCase& value : cases)
        semantic << value.source_root << ';';
    semantic << "operator-executed=0|preflight-executed=0|"
                "solver-executed=0|timing-admitted=0";
    return sha256_hex(semantic.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_manifest_controls() {
    const std::vector<ALR20ManifestCase> cases =
        al_r20_v3_manifest_cases();
    constexpr std::array<std::size_t, 4U> expected_counts{{
        72U, 75U, 70U, 60U}};
    bool exact = cases.size() == expected_counts.size();
    std::size_t total_particles = 0U;
    std::vector<std::string> source_roots;
    std::vector<std::string> geometry_roots;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        const ALR20ManifestCase& value = cases[index];
        exact = exact && value.role == "BLIND_HOLDOUT_V3"
            && al_r20_case_finite_and_owned(value)
            && index < expected_counts.size()
            && value.fixture.position.size() == expected_counts[index];
        total_particles += value.fixture.position.size();
        source_roots.push_back(value.source_root);
        geometry_roots.push_back(value.fixture.geometry_sha256);
    }
    std::sort(source_roots.begin(), source_roots.end());
    std::sort(geometry_roots.begin(), geometry_roots.end());
    exact = exact && total_particles == 277U
        && std::adjacent_find(source_roots.begin(), source_roots.end())
            == source_roots.end()
        && std::adjacent_find(geometry_roots.begin(), geometry_roots.end())
            == geometry_roots.end();
    const std::string result_sha = al_r20_v3_manifest_root(cases);
    const std::string route = exact
        ? "V3_HOLDOUT_MANIFEST_FROZEN"
        : "V3_HOLDOUT_MANIFEST_REJECTED";
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r9_v3_manifest.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_verified_inverse_sha256\":\""
           << AL_R20_VERIFIED_INVERSE_SHA256
           << "\",\"case_count\":" << cases.size()
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
              ",\"operator_executed\":false"
              ",\"preflight_executed\":false"
              ",\"solver_executed\":false"
              ",\"timing_admitted\":false"
              ",\"generalization_authority\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
