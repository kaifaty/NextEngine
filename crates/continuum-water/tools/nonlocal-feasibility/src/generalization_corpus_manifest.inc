namespace {

constexpr double AL_R20_TRUST_RADIUS = 0.0625;
constexpr std::size_t AL_R20_RELEASED_SNAPSHOT_STEPS = 10U;

struct ALR20ManifestCase {
    std::string role;
    std::string provenance;
    std::string contact_regime;
    std::size_t snapshot_steps = 0U;
    SmokeFixture fixture;
    std::string source_root;
};

Vec3 al_r20_maximum_components(const std::vector<Vec3>& values) {
    Vec3 result{
        -std::numeric_limits<double>::infinity(),
        -std::numeric_limits<double>::infinity(),
        -std::numeric_limits<double>::infinity(),
    };
    for (Vec3 value : values) {
        result.x = std::max(result.x, value.x);
        result.y = std::max(result.y, value.y);
        result.z = std::max(result.z, value.z);
    }
    return result;
}

void al_r20_translate(std::vector<Vec3>& values, Vec3 offset) {
    for (Vec3& value : values) value += offset;
}

SmokeFixture al_r20_make_oblique_edge_holdout() {
    SmokeFixture result;
    result.name = "r20-blind-oblique-upper-x-lower-y-edge";
    result.position = make_lattice_fluid({4, 3, 2});
    const Vec3 center = average_values(result.position);
    for (Vec3& value : result.position) {
        const Vec3 local = value - center;
        value = center + Vec3{
            0.991 * local.x + 0.035 * local.y,
            0.986 * local.y,
            0.994 * local.z + 0.020 * local.x,
        };
    }
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = {
        0.45 - RADIUS, 0.35 - RADIUS, 0.30 - RADIUS};
    const Vec3 minimum = minimum_components(result.position);
    const Vec3 maximum = al_r20_maximum_components(result.position);
    al_r20_translate(result.position, {
        result.contact_high.x - maximum.x,
        result.contact_low.y - minimum.y,
        0.11 - minimum.z,
    });
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x_lane = static_cast<double>(index % 4U);
        const double y_lane = static_cast<double>((index / 4U) % 3U);
        const double z_lane = static_cast<double>(index / 12U);
        result.velocity.push_back({
            0.55 + 0.015 * x_lane,
            -0.75 - 0.020 * y_lane,
            0.08 + 0.010 * z_lane,
        });
    }
    result.boundary = make_box_owned_shell({9, 7, 6}, 2);
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

SmokeFixture al_r20_make_opposed_corner_holdout() {
    SmokeFixture result;
    result.name = "r20-blind-upper-x-upper-y-lower-z-corner";
    result.position = make_lattice_fluid({3, 2, 4});
    const Vec3 center = average_values(result.position);
    for (Vec3& value : result.position) {
        const Vec3 local = value - center;
        value = center + Vec3{
            0.987 * local.x + 0.025 * local.z,
            0.993 * local.y - 0.030 * local.x,
            0.989 * local.z + 0.018 * local.y,
        };
    }
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = {
        0.35 - RADIUS, 0.35 - RADIUS, 0.40 - RADIUS};
    const Vec3 minimum = minimum_components(result.position);
    const Vec3 maximum = al_r20_maximum_components(result.position);
    al_r20_translate(result.position, {
        result.contact_high.x - maximum.x,
        result.contact_high.y - maximum.y,
        result.contact_low.z - minimum.z,
    });
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x_lane = static_cast<double>(index % 3U);
        const double y_lane = static_cast<double>((index / 3U) % 2U);
        const double z_lane = static_cast<double>(index / 6U);
        result.velocity.push_back({
            0.48 + 0.020 * z_lane,
            0.42 + 0.015 * x_lane,
            -0.62 - 0.018 * y_lane,
        });
    }
    result.boundary = make_box_owned_shell({7, 7, 8}, 2);
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

SmokeFixture al_r20_make_filled_edge_holdout() {
    SmokeFixture result;
    result.name = "r20-v2-blind-filled-upper-x-lower-y-edge";
    result.position = make_lattice_fluid({5, 4, 3});
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x_lane = static_cast<double>(index % 5U);
        const double y_lane = static_cast<double>((index / 5U) % 4U);
        const double z_lane = static_cast<double>(index / 20U);
        result.velocity.push_back({
            0.52 + 0.010 * y_lane,
            -0.71 - 0.008 * x_lane,
            0.06 + 0.006 * z_lane,
        });
    }
    result.boundary = make_box_owned_shell({5, 4, 3}, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = {
        0.25 - RADIUS, 0.20 - RADIUS, 0.15 - RADIUS};
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

SmokeFixture al_r20_make_filled_opposed_corner_holdout() {
    SmokeFixture result;
    result.name = "r20-v2-blind-filled-upper-x-upper-y-lower-z-corner";
    result.position = make_lattice_fluid({4, 4, 4});
    result.velocity.reserve(result.position.size());
    for (std::size_t index = 0U; index < result.position.size(); ++index) {
        const double x_lane = static_cast<double>(index % 4U);
        const double y_lane = static_cast<double>((index / 4U) % 4U);
        const double z_lane = static_cast<double>(index / 16U);
        result.velocity.push_back({
            0.46 + 0.007 * z_lane,
            0.39 + 0.009 * x_lane,
            -0.58 - 0.006 * y_lane,
        });
    }
    result.boundary = make_box_owned_shell({4, 4, 4}, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = {
        0.20 - RADIUS, 0.20 - RADIUS, 0.20 - RADIUS};
    result.maximum_participants =
        result.position.size() + result.boundary.size();
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

void al_r20_advance_free_flight(
    SmokeFixture& fixture, std::size_t steps) {
    for (std::size_t step = 0U; step < steps; ++step) {
        for (std::size_t particle = 0U;
             particle < fixture.position.size(); ++particle) {
            fixture.velocity[particle] +=
                SMOKE_FRAME_TIME * fixture.gravity;
            fixture.position[particle] +=
                SMOKE_FRAME_TIME * fixture.velocity[particle];
        }
    }
    fixture.geometry_sha256 = geometry_hash(fixture);
}

std::string al_r20_source_root(const ALR20ManifestCase& value) {
    std::ostringstream material;
    const SmokeFixture& fixture = value.fixture;
    material << "nextengine.nonlocal.r20-corpus-source.v1|"
             << value.role << '|' << value.provenance << '|'
             << value.contact_regime << '|' << value.snapshot_steps << '|'
             << fixture.name << '|' << fixture.geometry_sha256 << '|'
             << fixture.position.size() << ':' << fixture.boundary.size()
             << '|' << fixture.closed_box_contact << '|'
             << fixture.lower_contact[0] << fixture.lower_contact[1]
             << fixture.lower_contact[2] << '|'
             << binary64_bits(fixture.contact_low.x) << ':'
             << binary64_bits(fixture.contact_low.y) << ':'
             << binary64_bits(fixture.contact_low.z) << '|'
             << binary64_bits(fixture.contact_high.x) << ':'
             << binary64_bits(fixture.contact_high.y) << ':'
             << binary64_bits(fixture.contact_high.z) << '|'
             << binary64_bits(fixture.gravity.x) << ':'
             << binary64_bits(fixture.gravity.y) << ':'
             << binary64_bits(fixture.gravity.z) << '|'
             << binary64_bits(SMOKE_FRAME_TIME) << ':'
             << binary64_bits(SPACING) << ':'
             << binary64_bits(AL_R20_TRUST_RADIUS) << '|';
    for (Vec3 position : fixture.position)
        material << binary64_bits(position.x) << ':'
                 << binary64_bits(position.y) << ':'
                 << binary64_bits(position.z) << ';';
    material << '|';
    for (Vec3 velocity : fixture.velocity)
        material << binary64_bits(velocity.x) << ':'
                 << binary64_bits(velocity.y) << ':'
                 << binary64_bits(velocity.z) << ';';
    material << '|';
    for (Vec3 boundary : fixture.boundary)
        material << binary64_bits(boundary.x) << ':'
                 << binary64_bits(boundary.y) << ':'
                 << binary64_bits(boundary.z) << ';';
    return sha256_hex(material.str());
}

bool al_r20_case_finite_and_owned(const ALR20ManifestCase& value) {
    const SmokeFixture& fixture = value.fixture;
    if (fixture.position.empty()
        || fixture.velocity.size() != fixture.position.size()
        || fixture.boundary.empty() || fixture.geometry_sha256.empty())
        return false;
    const auto finite_vec = [](Vec3 item) { return finite(item); };
    if (!std::all_of(fixture.position.begin(), fixture.position.end(), finite_vec)
        || !std::all_of(fixture.velocity.begin(), fixture.velocity.end(), finite_vec)
        || !std::all_of(fixture.boundary.begin(), fixture.boundary.end(), finite_vec))
        return false;
    if (fixture.closed_box_contact) {
        for (Vec3 position : fixture.position)
            if (position.x < fixture.contact_low.x
                || position.x > fixture.contact_high.x
                || position.y < fixture.contact_low.y
                || position.y > fixture.contact_high.y
                || position.z < fixture.contact_low.z
                || position.z > fixture.contact_high.z)
                return false;
    }
    return true;
}

std::vector<ALR20ManifestCase> al_r20_manifest_cases() {
    std::vector<ALR20ManifestCase> cases;
    cases.push_back({
        "TRANSFER_REGRESSION",
        "pre-r65-b3-face-smoke",
        "lower-y-face",
        0U,
        make_face_smoke_fixture(),
        {},
    });
    cases.push_back({
        "TRANSFER_REGRESSION",
        "pre-r65-b3-corner-smoke",
        "lower-x-lower-y-edge",
        0U,
        make_corner_smoke_fixture(),
        {},
    });
    cases.push_back({
        "TRANSFER_REGRESSION",
        "pre-r65-b4b-supported-column",
        "closed-box-supported-start",
        0U,
        make_b4b_supported_column_fixture(),
        {},
    });
    SmokeFixture released = make_b4b_released_block_fixture();
    al_r20_advance_free_flight(released, AL_R20_RELEASED_SNAPSHOT_STEPS);
    cases.push_back({
        "TRANSFER_REGRESSION",
        "pre-r65-b4b-released-block-analytic-free-flight-snapshot",
        "closed-box-pre-impact",
        AL_R20_RELEASED_SNAPSHOT_STEPS,
        std::move(released),
        {},
    });
    cases.push_back({
        "KINEMATIC_NEGATIVE_CONTROL",
        "r20-v1-blind-reclassified-after-input-only-preflight",
        "upper-x-lower-y-oblique-edge",
        0U,
        al_r20_make_oblique_edge_holdout(),
        {},
    });
    cases.push_back({
        "KINEMATIC_NEGATIVE_CONTROL",
        "r20-v1-blind-reclassified-after-input-only-preflight",
        "upper-x-upper-y-lower-z-opposed-corner",
        0U,
        al_r20_make_opposed_corner_holdout(),
        {},
    });
    cases.push_back({
        "BLIND_HOLDOUT",
        "r20-v2-new-no-solver-or-oracle-observation",
        "filled-upper-x-lower-y-edge",
        0U,
        al_r20_make_filled_edge_holdout(),
        {},
    });
    cases.push_back({
        "BLIND_HOLDOUT",
        "r20-v2-new-no-solver-or-oracle-observation",
        "filled-upper-x-upper-y-lower-z-opposed-corner",
        0U,
        al_r20_make_filled_opposed_corner_holdout(),
        {},
    });
    for (ALR20ManifestCase& value : cases)
        value.source_root = al_r20_source_root(value);
    return cases;
}

std::string al_r20_manifest_semantic_root(
    const std::vector<ALR20ManifestCase>& cases) {
    std::size_t transfer = 0U;
    std::size_t holdout = 0U;
    std::size_t negative = 0U;
    for (const ALR20ManifestCase& value : cases) {
        transfer += value.role == "TRANSFER_REGRESSION" ? 1U : 0U;
        holdout += value.role == "BLIND_HOLDOUT" ? 1U : 0U;
        negative += value.role == "KINEMATIC_NEGATIVE_CONTROL" ? 1U : 0U;
    }
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20-corpus-manifest.v2|"
             << cases.size() << ':' << transfer << ':' << negative << ':'
             << holdout << '|';
    for (const ALR20ManifestCase& value : cases)
        semantic << value.source_root << ';';
    semantic << "target=semi-implicit-predictor-minus-source-over-spacing|"
                "trqp=r64-density-plus-contact-box-plus-global-ball|"
                "solver-executed=0";
    return sha256_hex(semantic.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_corpus_manifest_controls() {
    const std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    bool exact = cases.size() == 8U;
    std::size_t transfer = 0U;
    std::size_t holdout = 0U;
    std::size_t negative = 0U;
    std::vector<std::string> roots;
    for (const ALR20ManifestCase& value : cases) {
        exact = exact && al_r20_case_finite_and_owned(value);
        transfer += value.role == "TRANSFER_REGRESSION" ? 1U : 0U;
        holdout += value.role == "BLIND_HOLDOUT" ? 1U : 0U;
        negative += value.role == "KINEMATIC_NEGATIVE_CONTROL" ? 1U : 0U;
        roots.push_back(value.source_root);
    }
    std::sort(roots.begin(), roots.end());
    exact = exact && transfer == 4U && negative == 2U && holdout == 2U
        && std::adjacent_find(roots.begin(), roots.end()) == roots.end();
    const std::string result_sha = al_r20_manifest_semantic_root(cases);

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20_corpus_manifest.v2\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\""
           << (exact ? "GENERALIZATION_CORPUS_MANIFEST_CANDIDATE"
                     : "GENERALIZATION_CORPUS_MANIFEST_REJECTED")
           << "\",\"case_count\":" << cases.size()
           << ",\"transfer_regressions\":" << transfer
           << ",\"kinematic_negative_controls\":" << negative
           << ",\"blind_holdouts\":" << holdout
           << ",\"trust_radius\":" << AL_R20_TRUST_RADIUS
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20ManifestCase& value = cases[index];
        report << "{\"name\":\"" << value.fixture.name
               << "\",\"role\":\"" << value.role
               << "\",\"provenance\":\"" << value.provenance
               << "\",\"contact_regime\":\"" << value.contact_regime
               << "\",\"snapshot_steps\":" << value.snapshot_steps
               << ",\"fluid_samples\":" << value.fixture.position.size()
               << ",\"boundary_samples\":" << value.fixture.boundary.size()
               << ",\"geometry_sha256\":\""
               << value.fixture.geometry_sha256
               << "\",\"source_sha256\":\"" << value.source_root << "\"}";
    }
    report << "]"
              ",\"target_map\":\"semi-implicit-predictor-minus-source-over-spacing\""
              ",\"trqp_formula\":\"r64-density-plus-contact-box-plus-global-ball\""
              ",\"solver_executed\":false,\"oracle_executed\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
