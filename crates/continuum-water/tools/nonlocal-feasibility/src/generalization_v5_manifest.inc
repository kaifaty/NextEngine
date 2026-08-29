namespace {

constexpr const char* AL_R20_R56_SEMANTIC_SHA256 =
    "daf7c2ceb1c1e803feb992abc5975a3dff980487a583fb4d203d165b605d20b7";

void al_r20_v5_finish_fixture(
    SmokeFixture& fixture,
    const std::array<int, 3U>& dimensions) {
    fixture.boundary = make_box_owned_shell(dimensions, 2);
    fixture.closed_box_contact = true;
    fixture.contact_low = {RADIUS, RADIUS, RADIUS};
    fixture.contact_high = al_r20_v3_box_high(dimensions);
    fixture.maximum_participants =
        fixture.position.size() + fixture.boundary.size();
    fixture.maximum_pairs = 160U * fixture.position.size();
    fixture.geometry_sha256 = geometry_hash(fixture);
}

SmokeFixture al_r20_v5_triaxial_saddle() {
    constexpr std::array<int, 3U> dimensions{{5, 5, 4}};
    SmokeFixture result;
    result.name = "v5-filled-triaxial-saddle-5x5x4";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 5U);
        const double y = static_cast<double>((index / 5U) % 5U);
        const double z = static_cast<double>(index / 25U);
        result.velocity.push_back({
            0.34 + 0.09 * (x - 2.0) - 0.07 * (y - 2.0) + 0.013 * z,
            -0.28 + 0.08 * (y - 2.0) + 0.05 * (z - 1.5) - 0.011 * x,
            0.22 - 0.06 * (z - 1.5) + 0.04 * (x - 2.0) - 0.009 * y,
        });
    }
    al_r20_v5_finish_fixture(result, dimensions);
    return result;
}

SmokeFixture al_r20_v5_layered_xz_shear() {
    constexpr std::array<int, 3U> dimensions{{9, 3, 4}};
    SmokeFixture result;
    result.name = "v5-filled-layered-xz-shear-9x3x4";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const std::size_t x_index = index % 9U;
        const std::size_t z_index = index / 27U;
        const double x = static_cast<double>(x_index);
        const double y = static_cast<double>((index / 9U) % 3U);
        const double z = static_cast<double>(z_index);
        const double s = static_cast<double>(
            static_cast<int>((x_index + 2U * z_index) % 3U) - 1);
        result.velocity.push_back({
            0.47 + 0.12 * s + 0.007 * y,
            0.18 * (z - 1.5) - 0.025 * (x - 4.0),
            -0.36 + 0.015 * x - 0.009 * y,
        });
    }
    al_r20_v5_finish_fixture(result, dimensions);
    return result;
}

SmokeFixture al_r20_v5_checkerboard_compression() {
    constexpr std::array<int, 3U> dimensions{{4, 8, 3}};
    SmokeFixture result;
    result.name = "v5-filled-checkerboard-compression-4x8x3";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const std::size_t x_index = index % 4U;
        const std::size_t y_index = (index / 4U) % 8U;
        const std::size_t z_index = index / 32U;
        const double x = static_cast<double>(x_index);
        const double y = static_cast<double>(y_index);
        const double z = static_cast<double>(z_index);
        const double q = (x_index + y_index + z_index) % 2U == 0U
            ? -1.0 : 1.0;
        result.velocity.push_back({
            -0.11 * (x - 1.5) + 0.17 * q,
            0.39 - 0.08 * (y - 3.5),
            -0.13 * (z - 1.0) + 0.10 * q + 0.006 * y,
        });
    }
    al_r20_v5_finish_fixture(result, dimensions);
    return result;
}

SmokeFixture al_r20_v5_oblique_jet_twist() {
    constexpr std::array<int, 3U> dimensions{{3, 5, 7}};
    SmokeFixture result;
    result.name = "v5-filled-oblique-jet-twist-3x5x7";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 3U);
        const double y = static_cast<double>((index / 3U) % 5U);
        const double z = static_cast<double>(index / 15U);
        const double z_distance = z >= 3.0 ? z - 3.0 : 3.0 - z;
        result.velocity.push_back({
            -0.22 * (y - 2.0) + 0.018 * (z - 3.0),
            0.20 * (x - 1.0) - 0.014 * (z - 3.0),
            0.62 - 0.035 * z_distance + 0.008 * y,
        });
    }
    al_r20_v5_finish_fixture(result, dimensions);
    return result;
}

SmokeFixture al_r20_v5_radial_expansion_drift() {
    constexpr std::array<int, 3U> dimensions{{6, 6, 3}};
    SmokeFixture result;
    result.name = "v5-filled-radial-expansion-drift-6x6x3";
    result.position = make_lattice_fluid(dimensions);
    const Vec3 center = average_values(result.position);
    result.velocity.reserve(result.position.size());
    for (Vec3 position : result.position) {
        const Vec3 radial = position - center;
        result.velocity.push_back({
            0.12 + 1.6 * radial.x - 0.4 * radial.y,
            -0.09 + 1.4 * radial.y + 0.35 * radial.z,
            0.07 + 1.2 * radial.z - 0.3 * radial.x,
        });
    }
    al_r20_v5_finish_fixture(result, dimensions);
    return result;
}

SmokeFixture al_r20_v5_three_face_coupling() {
    constexpr std::array<int, 3U> dimensions{{5, 4, 6}};
    SmokeFixture result;
    result.name = "v5-filled-three-face-coupling-5x4x6";
    result.position = make_lattice_fluid(dimensions);
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x = static_cast<double>(index % 5U);
        const double y = static_cast<double>((index / 5U) % 4U);
        const double z = static_cast<double>(index / 20U);
        result.velocity.push_back({
            0.55 + 0.04 * (y - 1.5) - 0.03 * (z - 2.5),
            -0.48 - 0.025 * x + 0.02 * z,
            0.44 + 0.03 * x - 0.035 * y,
        });
    }
    al_r20_v5_finish_fixture(result, dimensions);
    return result;
}

std::vector<ALR20ManifestCase> al_r20_v5_manifest_cases() {
    std::vector<ALR20ManifestCase> cases;
    const char* role = "BLIND_HOLDOUT_V5";
    const char* provenance = "r20r57-source-only-after-v4-composition";
    cases.push_back({role, provenance, "filled-triaxial-saddle", 0U,
        al_r20_v5_triaxial_saddle(), {}});
    cases.push_back({role, provenance, "filled-layered-xz-shear", 0U,
        al_r20_v5_layered_xz_shear(), {}});
    cases.push_back({role, provenance, "filled-checkerboard-compression", 0U,
        al_r20_v5_checkerboard_compression(), {}});
    cases.push_back({role, provenance, "filled-oblique-jet-twist", 0U,
        al_r20_v5_oblique_jet_twist(), {}});
    cases.push_back({role, provenance, "filled-radial-expansion-drift", 0U,
        al_r20_v5_radial_expansion_drift(), {}});
    cases.push_back({role, provenance, "filled-three-face-coupling", 0U,
        al_r20_v5_three_face_coupling(), {}});
    for (ALR20ManifestCase& value : cases)
        value.source_root = al_r20_source_root(value);
    return cases;
}

std::string al_r20_v5_manifest_root(
    const std::vector<ALR20ManifestCase>& cases) {
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r57-v5-manifest.v1|"
        << AL_R20_R56_SEMANTIC_SHA256 << ':' << cases.size() << '|';
    for (const ALR20ManifestCase& value : cases)
        semantic << value.source_root << ';';
    semantic << "operator-executed=0|projection-executed=0|"
        "preflight-executed=0|solver-executed=0|timing-admitted=0";
    return sha256_hex(semantic.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_manifest_controls() {
    const std::vector<ALR20ManifestCase> cases = al_r20_v5_manifest_cases();
    constexpr std::array<std::size_t, 6U> expected_counts{{
        100U, 108U, 96U, 105U, 108U, 120U}};
    bool exact = cases.size() == expected_counts.size();
    std::size_t total_particles = 0U;
    std::vector<std::string> source_roots;
    std::vector<std::string> geometry_roots;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        const ALR20ManifestCase& value = cases[index];
        exact = exact && value.role == "BLIND_HOLDOUT_V5"
            && value.provenance
                == "r20r57-source-only-after-v4-composition"
            && al_r20_case_finite_and_owned(value)
            && index < expected_counts.size()
            && value.fixture.position.size() == expected_counts[index];
        total_particles += value.fixture.position.size();
        source_roots.push_back(value.source_root);
        geometry_roots.push_back(value.fixture.geometry_sha256);
    }
    std::sort(source_roots.begin(), source_roots.end());
    std::sort(geometry_roots.begin(), geometry_roots.end());
    exact = exact && total_particles == 637U
        && std::adjacent_find(source_roots.begin(), source_roots.end())
            == source_roots.end()
        && std::adjacent_find(geometry_roots.begin(), geometry_roots.end())
            == geometry_roots.end();
    const std::string result_sha = al_r20_v5_manifest_root(cases);
    const std::string route = exact
        ? "V5_HOLDOUT_MANIFEST_FROZEN"
        : "V5_HOLDOUT_MANIFEST_REJECTED";
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r57_v5_manifest.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r56_sha256\":\"" << AL_R20_R56_SEMANTIC_SHA256 << "\""
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
            << ",\"geometry_sha256\":\"" << value.fixture.geometry_sha256
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
