namespace {

constexpr const char* B4E2D7R19R31_IDENTITY_SHA256 =
    "7cef89a34df72c81730c09243af64105602ccbffe4a73fb10f5935cfa595a60a";
constexpr const char* B4E2D7R19R31_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r31-scalar-feasibility-interval|v1|parent=b6e531a5:34cf7a56d5c449dca20bf1ab800e061f62bfa0e8145996a6c032e080519596ad:41c3e833a7c618281d94219e059bfbab85cc6631f2bc916478856484ece4808b|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;violated-bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70;rhs-7cb60183e7b7a40d077e409282730bacbdf4ab92eb449a82ef87986e89d70c55;preimage-ebc17b986e68f823c2088aa3fd7719374bc5265b14cc3b751658bd65962c1430;response-fd75ab535dc05ed712a7eae7f4cfa1fc882d94e93c85c0a15f22b58e39137d7f;particles6000;rows1420;columns18000|problem=alpha-in-[0,1];selected:c+alpha*r<=0;inactive:c+alpha*r<=0;lower=max-selected-c/(-r);upper=min-inactive-(-c)/r|arithmetic=binary64-state;binary128-ratio-order;binary64-separate-multiply-add;nextafter-repair<=64|controls=dense-feasible;dense-inactive-blocker;dense-selected-impossible;dense-zero-margin;parent;source;partition;ratio-order;direct-predicates;work;rollback|observations=lower;upper;gap;blocker-roots;repair-ulps;safe-selected-progress;required-inactive-leakage;scaled-preimage|routes=scalar-interval-parent-rejected;scalar-interval-source-rejected;scalar-interval-dense-rejected;scalar-interval-partition-rejected;scalar-interval-order-rejected;scalar-interval-predicate-rejected;scalar-interval-work-rejected;scalar-feasibility-interval-candidate;inequality-active-set-reformulation-required|precedence=parent,source,dense,partition,order,predicate,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r30-once;new-pair-passes0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-scalar-feasibility-interval-classification-only)IDENTITY";

constexpr std::size_t AL_R31_MAX_REPAIR_ULPS = 64U;

struct ALR31PredicateSummary {
    std::size_t selected_positive = 0U;
    std::size_t inactive_positive = 0U;
    double selected_positive_norm = 0.0;
    double inactive_positive_norm = 0.0;
    double all_positive_norm = 0.0;
};

struct ALR31Interval {
    bool valid = false;
    bool order_exact = false;
    bool predicates_exact = false;
    bool lower_defined = false;
    bool scalar_feasible = false;
    Binary128 lower_quad = static_cast<Binary128>(0.0);
    Binary128 upper_quad = static_cast<Binary128>(1.0);
    double lower = 0.0;
    double upper = 1.0;
    std::size_t lower_repairs = 0U;
    std::size_t upper_repairs = 0U;
    std::size_t selected_non_decreasing = 0U;
    std::size_t zero_margin_inactive = 0U;
    std::vector<std::uint8_t> lower_blocker;
    std::vector<std::uint8_t> upper_blocker;
    std::vector<std::uint8_t> selected_non_decreasing_mask;
    ALR31PredicateSummary at_lower;
    ALR31PredicateSummary at_upper;
    std::string lower_blocker_root;
    std::string upper_blocker_root;
    std::string selected_non_decreasing_root;
    std::string root;
};

struct ALR31DenseControl {
    std::string name;
    bool passed = false;
    bool valid = false;
    bool scalar_feasible = false;
    bool lower_defined = false;
    double lower = 0.0;
    double upper = 0.0;
    std::size_t selected_non_decreasing = 0U;
    std::size_t zero_margin_inactive = 0U;
    std::string root;
};

ALR31PredicateSummary al_r31_predicates(
    const std::vector<double>& constraint,
    const std::vector<double>& response,
    const std::vector<std::uint8_t>& selected,
    double alpha) {
    ALR31PredicateSummary result;
    if (constraint.size() != response.size()
        || selected.size() != constraint.size()
        || !std::isfinite(alpha) || alpha < 0.0) {
        result.selected_positive = std::numeric_limits<std::size_t>::max();
        result.inactive_positive = std::numeric_limits<std::size_t>::max();
        result.selected_positive_norm =
            std::numeric_limits<double>::infinity();
        result.inactive_positive_norm =
            std::numeric_limits<double>::infinity();
        result.all_positive_norm = std::numeric_limits<double>::infinity();
        return result;
    }
    long double selected_squared = 0.0L;
    long double inactive_squared = 0.0L;
    for (std::size_t row = 0U; row < constraint.size(); ++row) {
        const double predicted = constraint[row] + alpha * response[row];
        if (predicted <= 0.0) continue;
        const long double square = static_cast<long double>(predicted)
            * static_cast<long double>(predicted);
        if (selected[row]) {
            ++result.selected_positive;
            selected_squared += square;
        } else {
            ++result.inactive_positive;
            inactive_squared += square;
        }
    }
    result.selected_positive_norm = std::sqrt(
        static_cast<double>(selected_squared));
    result.inactive_positive_norm = std::sqrt(
        static_cast<double>(inactive_squared));
    result.all_positive_norm = std::sqrt(
        static_cast<double>(selected_squared + inactive_squared));
    return result;
}

ALR31Interval al_r31_interval(
    const std::vector<double>& constraint,
    const std::vector<double>& response,
    const std::vector<std::uint8_t>& selected) {
    ALR31Interval result;
    const std::size_t count = constraint.size();
    result.lower_blocker.assign(count, 0U);
    result.upper_blocker.assign(count, 0U);
    result.selected_non_decreasing_mask.assign(count, 0U);
    if (count == 0U || response.size() != count || selected.size() != count
        || !al_r30_finite(constraint) || !al_r30_finite(response)) {
        return result;
    }

    bool partition_valid = true;
    bool have_selected = false;
    bool have_lower = false;
    bool have_upper_blocker = false;
    result.lower_defined = true;
    for (std::size_t row = 0U; row < count; ++row) {
        if (selected[row]) {
            have_selected = true;
            partition_valid = partition_valid && constraint[row] > 0.0;
            if (response[row] >= 0.0) {
                result.lower_defined = false;
                result.selected_non_decreasing_mask[row] = 1U;
                ++result.selected_non_decreasing;
                continue;
            }
            const Binary128 ratio = static_cast<Binary128>(constraint[row])
                / -static_cast<Binary128>(response[row]);
            if (!finiteq(ratio) || ratio < static_cast<Binary128>(0.0)) {
                partition_valid = false;
                continue;
            }
            if (!have_lower || ratio > result.lower_quad) {
                result.lower_quad = ratio;
                have_lower = true;
            }
        } else {
            partition_valid = partition_valid && constraint[row] <= 0.0;
            if (constraint[row] == 0.0 && response[row] > 0.0) {
                ++result.zero_margin_inactive;
            }
            if (response[row] <= 0.0) continue;
            const Binary128 numerator = constraint[row] == 0.0
                ? static_cast<Binary128>(0.0)
                : -static_cast<Binary128>(constraint[row]);
            const Binary128 ratio = numerator
                / static_cast<Binary128>(response[row]);
            if (!finiteq(ratio) || ratio < static_cast<Binary128>(0.0)) {
                partition_valid = false;
                continue;
            }
            if (ratio < result.upper_quad) {
                result.upper_quad = ratio;
                have_upper_blocker = true;
            }
        }
    }
    result.lower_defined = result.lower_defined && have_selected && have_lower;
    if (!partition_valid || !have_selected) return result;

    bool ordered = true;
    for (std::size_t row = 0U; row < count; ++row) {
        if (selected[row] && response[row] < 0.0) {
            const Binary128 ratio = static_cast<Binary128>(constraint[row])
                / -static_cast<Binary128>(response[row]);
            ordered = ordered && ratio <= result.lower_quad;
            if (have_lower && ratio == result.lower_quad) {
                result.lower_blocker[row] = 1U;
            }
        } else if (!selected[row] && response[row] > 0.0) {
            const Binary128 numerator = constraint[row] == 0.0
                ? static_cast<Binary128>(0.0)
                : -static_cast<Binary128>(constraint[row]);
            const Binary128 ratio = numerator
                / static_cast<Binary128>(response[row]);
            ordered = ordered && ratio >= result.upper_quad;
            if (have_upper_blocker && ratio == result.upper_quad) {
                result.upper_blocker[row] = 1U;
            }
        }
    }

    if (result.lower_defined) {
        result.lower = static_cast<double>(result.lower_quad);
        if (!std::isfinite(result.lower) || result.lower < 0.0) return result;
        result.at_lower = al_r31_predicates(
            constraint, response, selected, result.lower);
        while (result.at_lower.selected_positive != 0U
            && result.lower_repairs < AL_R31_MAX_REPAIR_ULPS) {
            result.lower = std::nextafter(
                result.lower, std::numeric_limits<double>::infinity());
            ++result.lower_repairs;
            result.at_lower = al_r31_predicates(
                constraint, response, selected, result.lower);
        }
    }
    result.upper = static_cast<double>(result.upper_quad);
    if (!std::isfinite(result.upper) || result.upper < 0.0
        || result.upper > 1.0) {
        return result;
    }
    result.at_upper = al_r31_predicates(
        constraint, response, selected, result.upper);
    while (result.at_upper.inactive_positive != 0U
        && result.upper_repairs < AL_R31_MAX_REPAIR_ULPS) {
        result.upper = std::nextafter(result.upper, 0.0);
        ++result.upper_repairs;
        result.at_upper = al_r31_predicates(
            constraint, response, selected, result.upper);
    }
    if (result.lower_defined) {
        result.at_lower = al_r31_predicates(
            constraint, response, selected, result.lower);
    }

    result.order_exact = ordered
        && result.lower_repairs <= AL_R31_MAX_REPAIR_ULPS
        && result.upper_repairs <= AL_R31_MAX_REPAIR_ULPS;
    result.predicates_exact = result.order_exact
        && result.at_upper.inactive_positive == 0U
        && (!result.lower_defined
            || result.at_lower.selected_positive == 0U);
    result.scalar_feasible = result.predicates_exact
        && result.lower_defined && result.selected_non_decreasing == 0U
        && result.lower <= 1.0 && result.lower <= result.upper
        && result.at_lower.inactive_positive == 0U;
    result.lower_blocker_root = al_r29_mask_root(result.lower_blocker);
    result.upper_blocker_root = al_r29_mask_root(result.upper_blocker);
    result.selected_non_decreasing_root = al_r29_mask_root(
        result.selected_non_decreasing_mask);
    std::ostringstream projection;
    projection << result.lower_defined << ':' << result.scalar_feasible << ':'
               << al_binary128_hex(result.lower_quad) << ':'
               << al_binary128_hex(result.upper_quad) << ':'
               << binary64_bits(result.lower) << ':'
               << binary64_bits(result.upper) << ':'
               << result.lower_repairs << ':' << result.upper_repairs << ':'
               << result.selected_non_decreasing << ':'
               << result.zero_margin_inactive << ':'
               << result.lower_blocker_root << ':' << result.upper_blocker_root
               << ':' << result.selected_non_decreasing_root << ':'
               << result.at_lower.selected_positive << ':'
               << result.at_lower.inactive_positive << ':'
               << result.at_upper.selected_positive << ':'
               << result.at_upper.inactive_positive;
    result.root = sha256_hex(projection.str());
    result.valid = result.order_exact && result.predicates_exact;
    return result;
}

ALR31DenseControl al_r31_dense_control(
    std::string name,
    std::vector<double> constraint,
    std::vector<double> response,
    std::vector<std::uint8_t> selected,
    bool expected_feasible,
    bool expected_lower_defined,
    double expected_lower,
    double expected_upper,
    std::size_t expected_non_decreasing,
    std::size_t expected_zero_margin) {
    ALR31DenseControl result;
    result.name = std::move(name);
    const ALR31Interval interval = al_r31_interval(
        constraint, response, selected);
    result.valid = interval.valid;
    result.scalar_feasible = interval.scalar_feasible;
    result.lower_defined = interval.lower_defined;
    result.lower = interval.lower;
    result.upper = interval.upper;
    result.selected_non_decreasing = interval.selected_non_decreasing;
    result.zero_margin_inactive = interval.zero_margin_inactive;
    result.passed = interval.valid
        && interval.scalar_feasible == expected_feasible
        && interval.lower_defined == expected_lower_defined
        && (!expected_lower_defined
            || binary64_bits(interval.lower) == binary64_bits(expected_lower))
        && binary64_bits(interval.upper) == binary64_bits(expected_upper)
        && interval.selected_non_decreasing == expected_non_decreasing
        && interval.zero_margin_inactive == expected_zero_margin;
    result.root = sha256_hex(result.name + ':'
        + (result.passed ? "1:" : "0:") + interval.root);
    return result;
}

std::array<ALR31DenseControl, 4U> al_r31_dense_controls() {
    return {{
        al_r31_dense_control(
            "feasible", {1.0,-1.0}, {-2.0,0.25}, {1U,0U},
            true, true, 0.5, 1.0, 0U, 0U),
        al_r31_dense_control(
            "inactive-blocker", {1.0,-0.1}, {-1.0,1.0}, {1U,0U},
            false, true, 1.0, 0.1, 0U, 0U),
        al_r31_dense_control(
            "selected-impossible", {1.0,-1.0}, {0.0,0.0}, {1U,0U},
            false, false, 0.0, 1.0, 1U, 0U),
        al_r31_dense_control(
            "zero-margin", {1.0,0.0}, {-1.0,1.0}, {1U,0U},
            false, true, 1.0, 0.0, 0U, 1U),
    }};
}

std::string al_r31_json_double(double value) {
    if (!std::isfinite(value)) return "null";
    std::ostringstream text;
    text << std::setprecision(17) << value;
    return text.str();
}

std::string al_r31_route(
    bool parent, bool source, bool dense, bool partition,
    bool order, bool predicate, bool work, bool scalar_feasible) {
    if (!parent) return "SCALAR_INTERVAL_PARENT_REJECTED";
    if (!source) return "SCALAR_INTERVAL_SOURCE_REJECTED";
    if (!dense) return "SCALAR_INTERVAL_DENSE_REJECTED";
    if (!partition) return "SCALAR_INTERVAL_PARTITION_REJECTED";
    if (!order) return "SCALAR_INTERVAL_ORDER_REJECTED";
    if (!predicate) return "SCALAR_INTERVAL_PREDICATE_REJECTED";
    if (!work) return "SCALAR_INTERVAL_WORK_REJECTED";
    return scalar_feasible
        ? "SCALAR_FEASIBILITY_INTERVAL_CANDIDATE"
        : "INEQUALITY_ACTIVE_SET_REFORMULATION_REQUIRED";
}

struct ALR31ScalarIntervalCapture {
    bool captured = false;
    ALR30RangeProjectionCapture r30;
    ALR31Interval interval;
};

} // namespace

SplitBoundaryReport run_al_scalar_feasibility_interval_controls_impl(
    ALR31ScalarIntervalCapture* output_capture) {
    const bool identity_exact = sha256_hex(B4E2D7R19R31_IDENTITY_PROJECTION)
        == B4E2D7R19R31_IDENTITY_SHA256;
    ALR30RangeProjectionCapture capture;
    const SplitBoundaryReport parent =
        run_al_linearized_range_projection_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "41c3e833a7c618281d94219e059bfbab85cc6631f2bc916478856484ece4808b"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "34cf7a56d5c449dca20bf1ab800e061f62bfa0e8145996a6c032e080519596ad";

    const std::string constraint_before = al_r10_dual_root(capture.constraint);
    const std::string preimage_before = al_binary64_vec3_root(capture.preimage);
    const std::string response_before = al_r10_dual_root(capture.full_response);
    const bool source_exact = parent_exact
        && capture.state_root
            == "851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7"
        && capture.topology_root
            == "070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5"
        && capture.preimage_root
            == "ebc17b986e68f823c2088aa3fd7719374bc5265b14cc3b751658bd65962c1430"
        && capture.response_root
            == "fd75ab535dc05ed712a7eae7f4cfa1fc882d94e93c85c0a15f22b58e39137d7f"
        && preimage_before == capture.preimage_root
        && response_before == capture.response_root
        && capture.constraint.size() == 6000U
        && capture.violated.size() == capture.constraint.size()
        && capture.preimage.size() == capture.constraint.size()
        && capture.full_response.size() == capture.constraint.size();

    std::vector<double> rhs;
    std::size_t selected_count = 0U;
    if (source_exact) {
        for (std::size_t row = 0U; row < capture.constraint.size(); ++row) {
            if (capture.violated[row]) {
                ++selected_count;
                rhs.push_back(-capture.constraint[row]);
            }
        }
    }
    const std::string rhs_root = al_r10_dual_root(rhs);
    const bool partition_exact = source_exact && selected_count == 1420U
        && capture.violated_root
            == "bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70"
        && al_r29_mask_root(capture.violated) == capture.violated_root
        && capture.rhs_root
            == "7cb60183e7b7a40d077e409282730bacbdf4ab92eb449a82ef87986e89d70c55"
        && rhs_root == capture.rhs_root;

    const std::array<ALR31DenseControl, 4U> dense_controls =
        al_r31_dense_controls();
    const bool dense_exact = std::all_of(
        dense_controls.begin(), dense_controls.end(),
        [](const ALR31DenseControl& value) { return value.passed; });
    std::ostringstream dense_projection;
    for (const ALR31DenseControl& value : dense_controls) {
        dense_projection << value.name << ':' << value.passed << ':'
                         << value.root << '|';
    }
    const std::string dense_root = sha256_hex(dense_projection.str());

    ALR31Interval interval;
    if (partition_exact && dense_exact) {
        interval = al_r31_interval(
            capture.constraint, capture.full_response, capture.violated);
    }
    const bool order_exact = interval.valid && interval.order_exact;
    const bool predicate_exact = order_exact && interval.predicates_exact;
    const bool work_exact = true;
    const bool rollback_exact = constraint_before
            == al_r10_dual_root(capture.constraint)
        && preimage_before == al_binary64_vec3_root(capture.preimage)
        && response_before == al_r10_dual_root(capture.full_response)
        && capture.violated_root == al_r29_mask_root(capture.violated);

    struct RouteCase {
        const char* expected;
        std::string observed;
    };
    const std::array<RouteCase, 9U> routes{{
        {"SCALAR_FEASIBILITY_INTERVAL_CANDIDATE", al_r31_route(1,1,1,1,1,1,1,1)},
        {"INEQUALITY_ACTIVE_SET_REFORMULATION_REQUIRED", al_r31_route(1,1,1,1,1,1,1,0)},
        {"SCALAR_INTERVAL_PARENT_REJECTED", al_r31_route(0,1,1,1,1,1,1,1)},
        {"SCALAR_INTERVAL_SOURCE_REJECTED", al_r31_route(1,0,1,1,1,1,1,1)},
        {"SCALAR_INTERVAL_DENSE_REJECTED", al_r31_route(1,1,0,1,1,1,1,1)},
        {"SCALAR_INTERVAL_PARTITION_REJECTED", al_r31_route(1,1,1,0,1,1,1,1)},
        {"SCALAR_INTERVAL_ORDER_REJECTED", al_r31_route(1,1,1,1,0,1,1,1)},
        {"SCALAR_INTERVAL_PREDICATE_REJECTED", al_r31_route(1,1,1,1,1,0,1,1)},
        {"SCALAR_INTERVAL_WORK_REJECTED", al_r31_route(1,1,1,1,1,1,0,1)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const RouteCase& value : routes) {
        routes_exact = routes_exact && value.expected == value.observed;
        route_projection << value.expected << ':' << value.observed << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && partition_exact && order_exact && predicate_exact
        && work_exact && rollback_exact && routes_exact;
    const std::string route = passed ? al_r31_route(
        true, true, true, true, true, true, true,
        interval.scalar_feasible) : std::string{};
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!partition_exact) first_failure = "PARTITION";
    else if (!order_exact) first_failure = "ORDER";
    else if (!predicate_exact) first_failure = "PREDICATE";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    const double initial_selected_norm = al_r30_norm(rhs);
    const double safe_progress = initial_selected_norm > 0.0
        ? 1.0 - interval.at_upper.selected_positive_norm
            / initial_selected_norm
        : 0.0;
    const double gap = interval.lower_defined
        ? interval.upper - interval.lower
        : -std::numeric_limits<double>::infinity();
    const double lower_preimage_rms = interval.lower_defined
        ? interval.lower * capture.preimage_rms : 0.0;
    const double lower_preimage_maximum = interval.lower_defined
        ? interval.lower * capture.preimage_maximum : 0.0;
    const double upper_preimage_rms =
        interval.upper * capture.preimage_rms;
    const double upper_preimage_maximum =
        interval.upper * capture.preimage_maximum;

    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R31_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << constraint_before << ':' << capture.violated_root << ':'
             << preimage_before << ':' << response_before << '|'
             << dense_root << ':' << interval.root << '|'
             << binary64_bits(interval.lower) << ':'
             << binary64_bits(interval.upper) << ':'
             << interval.scalar_feasible << ':'
             << interval.at_lower.inactive_positive << ':'
             << interval.at_upper.selected_positive << '|'
             << route_root << ':' << routes_exact << '|' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r31_scalar_feasibility_interval.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R31_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"constraint_root\":\"" << constraint_before
           << "\",\"violated_root\":\"" << capture.violated_root
           << "\",\"preimage_root\":\"" << preimage_before
           << "\",\"response_root\":\"" << response_before
           << "\",\"particles\":" << capture.constraint.size()
           << ",\"selected\":" << selected_count << "}"
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"cases\":" << dense_controls.size()
           << ",\"root\":\"" << dense_root << "\",\"controls\":[";
    for (std::size_t index = 0U; index < dense_controls.size(); ++index) {
        const ALR31DenseControl& value = dense_controls[index];
        if (index != 0U) report << ',';
        report << "{\"name\":\"" << value.name << "\",\"passed\":"
               << (value.passed ? "true" : "false")
               << ",\"valid\":" << (value.valid ? "true" : "false")
               << ",\"scalar_feasible\":"
               << (value.scalar_feasible ? "true" : "false")
               << ",\"lower_defined\":"
               << (value.lower_defined ? "true" : "false")
               << ",\"lower\":" << value.lower
               << ",\"upper\":" << value.upper
               << ",\"selected_non_decreasing\":"
               << value.selected_non_decreasing
               << ",\"zero_margin_inactive\":"
               << value.zero_margin_inactive << '}';
    }
    report << "]}"
           << ",\"interval\":{\"lower_defined\":"
           << (interval.lower_defined ? "true" : "false")
           << ",\"scalar_feasible\":"
           << (interval.scalar_feasible ? "true" : "false")
           << ",\"lower\":" << al_r31_json_double(interval.lower)
           << ",\"upper\":" << al_r31_json_double(interval.upper)
           << ",\"gap\":" << al_r31_json_double(gap)
           << ",\"lower_binary128\":\""
           << al_binary128_hex(interval.lower_quad)
           << "\",\"upper_binary128\":\""
           << al_binary128_hex(interval.upper_quad)
           << "\",\"lower_repairs\":" << interval.lower_repairs
           << ",\"upper_repairs\":" << interval.upper_repairs
           << ",\"selected_non_decreasing\":"
           << interval.selected_non_decreasing
           << ",\"zero_margin_inactive\":"
           << interval.zero_margin_inactive
           << ",\"lower_blocker_root\":\""
           << interval.lower_blocker_root
           << "\",\"upper_blocker_root\":\""
           << interval.upper_blocker_root
           << "\",\"selected_non_decreasing_root\":\""
           << interval.selected_non_decreasing_root
           << "\",\"root\":\"" << interval.root << "\"}"
           << ",\"at_required_lower\":{\"selected_positive\":"
           << interval.at_lower.selected_positive
           << ",\"inactive_positive\":"
           << interval.at_lower.inactive_positive
           << ",\"selected_positive_norm\":"
           << interval.at_lower.selected_positive_norm
           << ",\"inactive_positive_norm\":"
           << interval.at_lower.inactive_positive_norm
           << ",\"all_positive_norm\":"
           << interval.at_lower.all_positive_norm
           << ",\"preimage_rms\":" << lower_preimage_rms
           << ",\"preimage_maximum\":" << lower_preimage_maximum << "}"
           << ",\"at_safe_upper\":{\"selected_positive\":"
           << interval.at_upper.selected_positive
           << ",\"inactive_positive\":"
           << interval.at_upper.inactive_positive
           << ",\"selected_positive_norm\":"
           << interval.at_upper.selected_positive_norm
           << ",\"inactive_positive_norm\":"
           << interval.at_upper.inactive_positive_norm
           << ",\"all_positive_norm\":"
           << interval.at_upper.all_positive_norm
           << ",\"selected_violation_reduction\":" << safe_progress
           << ",\"preimage_rms\":" << upper_preimage_rms
           << ",\"preimage_maximum\":" << upper_preimage_maximum << "}"
           << ",\"work\":{\"parent_r30_replays\":1,\"new_pair_passes\":0"
              ",\"new_hvp\":0,\"new_model\":0,\"new_trials\":0"
              ",\"new_precision\":0,\"new_outer\":0}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"correction_applied\":false,\"nonlinear_evaluation\":false"
              ",\"floor_classification\":false,\"state_mutation\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (output_capture != nullptr) {
        output_capture->captured = passed;
        output_capture->r30 = std::move(capture);
        output_capture->interval = std::move(interval);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_scalar_feasibility_interval_controls() {
    return run_al_scalar_feasibility_interval_controls_impl(nullptr);
}
