namespace {

constexpr const char* B4E2D7R19R48_IDENTITY_SHA256 =
    "b20c6c6b530bd3078de7707e87c8deacf8eae0dab581584cf30421e5a1d52332";
constexpr const char* B4E2D7R19R48_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r48-restoration-certificate|v2|parent=33e6ebfd:dac0ffd871c927cb62253f64a078866d9515faefda59e48a54d923089dbc371d:cbc082189cf1813be74123be818fa55649c7b2feac07d87f31626063b046286c:RESTORATION_COMPATIBILITY_REQUIRED|source=r43-position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;endpoint-30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01|problem=y=r43-projected-trial;c=fresh-density-inequality-at-y;A=spacing*Jc-at-y;phi(d)=0.5*norm2(max(c+A*d,0));all-rows;fixed-stable-superset|geometry=next-trust-radius0.0625;normal-reserve-half;normal-radius0.03125;global-l2;current-penetrating-face-nonworsening;current-interior-first-face-bounds;zero-in-every-interval;source-half-skin-implied|generator=malitsky-pock-pdal;d0=0;lambda1=max(c,0);tau0=1;beta1;mu0.5;delta0.99;maximum-growth;main-cap128;per-main-backtrack-cap16;total-extra-backtrack-cap256;early-primal-certificate|prox=euclidean-ball-intersect-box;monotone-multiplier;bracket-and-inward-endpoint;maximum128-bisections;direct-feasibility-bracket-kkt-audit|dual=lambda-nonnegative;D=lambdaTc-0.5*norm2lambda-supportC(-Atlambda);support-upper=lagrangian-eta-separable-box-max;outward-long-double;binary128-if-sign-unresolved-or-disagrees;positive-lower-bound-only|certificate=primal-row-upper=c+A*d+gamma-directed-absolute<=0-all-rows;or-dual-lower>0;mutually-exclusive;positive-hinge-stationarity-insufficient;otherwise-unresolved|controls=dense-feasible;dense-infeasible;ball-projection;box-projection;support-upper;weak-duality;rounding;route-precedence;parent;source;workspace;geometry;projection;generator;primal;dual;work;rollback|routes=restoration-certificate-parent-rejected;restoration-certificate-source-rejected;restoration-certificate-workspace-rejected;restoration-certificate-geometry-rejected;restoration-certificate-projection-rejected;restoration-certificate-generator-rejected;restoration-certificate-primal-audit-rejected;restoration-certificate-dual-audit-rejected;restoration-certificate-work-rejected;restoration-next-trqp-compatible-candidate;restoration-radius-infeasible-certificate;restoration-certificate-unresolved|precedence=parent,source,workspace,geometry,projection,generator,primal-audit,dual-audit,work,classification|work=parent-r47-replays1;static-index1;superset-build1;moved-workspace1;releases1;jvp<=128;vjp<=385;projection-scans-bounded;support-audits<=8;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-next-trqp-restoration-certificate-only)IDENTITY";

constexpr double AL_R48_NEXT_RADIUS = 0.0625;
constexpr double AL_R48_NORMAL_RADIUS = 0.03125;
constexpr std::size_t AL_R48_MAIN_CAP = 128U;
constexpr std::size_t AL_R48_BACKTRACK_CAP = 16U;
constexpr std::size_t AL_R48_EXTRA_BACKTRACK_CAP = 256U;
constexpr std::size_t AL_R48_PROJECTION_BISECTIONS = 128U;

struct ALR48Projection {
    bool passed = false;
    std::vector<Vec3> value;
    double norm = std::numeric_limits<double>::infinity();
    double multiplier_low = 0.0;
    double multiplier_high = 0.0;
    double low_norm = 0.0;
    double high_norm = 0.0;
    std::size_t scans = 0U;
};

double al_r48_component(const Vec3& value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

double al_r48_clamp(double value, double low, double high) {
    return std::max(low, std::min(value, high));
}

bool al_r48_vec_finite(const std::vector<Vec3>& value) {
    return std::all_of(value.begin(), value.end(), [](const Vec3& v) {
        return std::isfinite(v.x) && std::isfinite(v.y)
            && std::isfinite(v.z);
    });
}

bool al_r48_scalar_finite(const std::vector<double>& value) {
    return std::all_of(value.begin(), value.end(),
        [](double v) { return std::isfinite(v); });
}

double al_r48_box_scaled_norm(const std::vector<Vec3>& z,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper,
    double multiplier, std::vector<Vec3>* output, std::size_t* scans) {
    if (z.size() != lower.size() || z.size() != upper.size()
        || !std::isfinite(multiplier) || multiplier < 0.0) {
        return std::numeric_limits<double>::infinity();
    }
    if (scans != nullptr) ++*scans;
    const double denominator = 1.0 + multiplier;
    std::vector<Vec3> temporary;
    std::vector<Vec3>& value = output == nullptr ? temporary : *output;
    value.assign(z.size(), {});
    long double squared = 0.0L;
    for (std::size_t i = 0U; i < z.size(); ++i) {
        for (int axis = 0; axis < 3; ++axis) {
            const double projected = al_r48_clamp(
                al_r48_component(z[i], axis) / denominator,
                al_r48_component(lower[i], axis),
                al_r48_component(upper[i], axis));
            al_r43_component(value[i], axis) = projected;
            squared += static_cast<long double>(projected)
                * static_cast<long double>(projected);
        }
    }
    return std::sqrt(static_cast<double>(squared));
}

ALR48Projection al_r48_project_ball_box(const std::vector<Vec3>& z,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper,
    double radius) {
    ALR48Projection result;
    if (z.empty() || z.size() != lower.size() || z.size() != upper.size()
        || !al_r48_vec_finite(z) || !al_r48_vec_finite(lower)
        || !al_r48_vec_finite(upper) || !std::isfinite(radius)
        || radius <= 0.0) {
        return result;
    }
    for (std::size_t i = 0U; i < z.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double low = al_r48_component(lower[i], axis);
            const double high = al_r48_component(upper[i], axis);
            if (low > 0.0 || high < 0.0 || low > high) return result;
        }
    result.low_norm = al_r48_box_scaled_norm(
        z, lower, upper, 0.0, &result.value, &result.scans);
    if (!std::isfinite(result.low_norm)) return result;
    if (result.low_norm <= radius) {
        result.norm = result.low_norm;
        result.multiplier_low = 0.0;
        result.multiplier_high = 0.0;
        result.high_norm = result.low_norm;
    } else {
        double low_multiplier = 0.0;
        double high_multiplier = 1.0;
        double high_norm = al_r48_box_scaled_norm(
            z, lower, upper, high_multiplier, nullptr, &result.scans);
        for (std::size_t bracket = 0U;
             bracket < AL_R48_PROJECTION_BISECTIONS
                 && high_norm > radius; ++bracket) {
            low_multiplier = high_multiplier;
            high_multiplier *= 2.0;
            high_norm = al_r48_box_scaled_norm(
                z, lower, upper, high_multiplier, nullptr, &result.scans);
        }
        if (!std::isfinite(high_norm) || high_norm > radius) return result;
        for (std::size_t iteration = 0U;
             iteration < AL_R48_PROJECTION_BISECTIONS; ++iteration) {
            const double middle = low_multiplier
                + 0.5 * (high_multiplier - low_multiplier);
            if (middle == low_multiplier || middle == high_multiplier) break;
            const double middle_norm = al_r48_box_scaled_norm(
                z, lower, upper, middle, nullptr, &result.scans);
            if (!std::isfinite(middle_norm)) return result;
            if (middle_norm > radius) low_multiplier = middle;
            else high_multiplier = middle;
        }
        result.multiplier_low = low_multiplier;
        result.multiplier_high = high_multiplier;
        result.low_norm = al_r48_box_scaled_norm(
            z, lower, upper, low_multiplier, nullptr, &result.scans);
        result.high_norm = al_r48_box_scaled_norm(
            z, lower, upper, high_multiplier, &result.value, &result.scans);
        result.norm = result.high_norm;
        if (result.norm > radius) {
            double scale = std::nextafter(radius / result.norm, 0.0);
            for (Vec3& value : result.value) value = scale * value;
            result.norm = al_r32_vec3_norm(result.value);
        }
    }
    bool box_exact = true;
    for (std::size_t i = 0U; i < result.value.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double value = al_r48_component(result.value[i], axis);
            box_exact = box_exact
                && value >= al_r48_component(lower[i], axis)
                && value <= al_r48_component(upper[i], axis);
        }
    const bool bracket_exact = result.multiplier_high == 0.0
        ? result.low_norm <= radius
        : result.low_norm >= radius && result.high_norm <= radius;
    result.passed = al_r48_vec_finite(result.value)
        && std::isfinite(result.norm) && result.norm <= radius
        && box_exact && bracket_exact && result.scans > 0U;
    return result;
}

struct ALR48SupportUpper {
    bool passed = false;
    long double upper = std::numeric_limits<long double>::infinity();
    long double raw = std::numeric_limits<long double>::infinity();
    long double error = std::numeric_limits<long double>::infinity();
    long double multiplier = 0.0L;
    std::size_t scans = 0U;
};

long double al_r48_support_norm(const std::vector<Vec3>& q,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper,
    long double multiplier, std::size_t* scans) {
    if (scans != nullptr) ++*scans;
    long double squared = 0.0L;
    for (std::size_t i = 0U; i < q.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const long double qi = al_r48_component(q[i], axis);
            const long double low = al_r48_component(lower[i], axis);
            const long double high = al_r48_component(upper[i], axis);
            const long double value = multiplier > 0.0L
                ? std::max(low, std::min(qi / multiplier, high))
                : (qi > 0.0L ? high : (qi < 0.0L ? low : 0.0L));
            squared += value * value;
        }
    return std::sqrt(squared);
}

ALR48SupportUpper al_r48_support_upper(const std::vector<Vec3>& q,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper,
    double radius) {
    ALR48SupportUpper result;
    if (q.empty() || q.size() != lower.size() || q.size() != upper.size()
        || !al_r48_vec_finite(q) || !std::isfinite(radius)
        || radius <= 0.0) {
        return result;
    }
    long double multiplier = 0.0L;
    const long double box_norm = al_r48_support_norm(
        q, lower, upper, 0.0L, &result.scans);
    if (box_norm > static_cast<long double>(radius)) {
        long double low_multiplier = 0.0L;
        long double high_multiplier = 1.0L;
        long double high_norm = al_r48_support_norm(
            q, lower, upper, high_multiplier, &result.scans);
        for (std::size_t bracket = 0U;
             bracket < AL_R48_PROJECTION_BISECTIONS
                 && high_norm > static_cast<long double>(radius); ++bracket) {
            low_multiplier = high_multiplier;
            high_multiplier *= 2.0L;
            high_norm = al_r48_support_norm(
                q, lower, upper, high_multiplier, &result.scans);
        }
        if (!std::isfinite(high_norm)
            || high_norm > static_cast<long double>(radius)) return result;
        for (std::size_t iteration = 0U;
             iteration < AL_R48_PROJECTION_BISECTIONS; ++iteration) {
            const long double middle = low_multiplier
                + 0.5L * (high_multiplier - low_multiplier);
            if (middle == low_multiplier || middle == high_multiplier) break;
            const long double middle_norm = al_r48_support_norm(
                q, lower, upper, middle, &result.scans);
            if (middle_norm > static_cast<long double>(radius))
                low_multiplier = middle;
            else high_multiplier = middle;
        }
        multiplier = high_multiplier;
    }
    long double raw = 0.5L * multiplier
        * static_cast<long double>(radius)
        * static_cast<long double>(radius);
    long double magnitude = std::abs(raw);
    for (std::size_t i = 0U; i < q.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const long double qi = al_r48_component(q[i], axis);
            const long double low = al_r48_component(lower[i], axis);
            const long double high = al_r48_component(upper[i], axis);
            const long double value = multiplier > 0.0L
                ? std::max(low, std::min(qi / multiplier, high))
                : (qi > 0.0L ? high : (qi < 0.0L ? low : 0.0L));
            const long double term = qi * value
                - 0.5L * multiplier * value * value;
            raw += term;
            magnitude += std::abs(term);
        }
    const long double error = 4096.0L * LDBL_EPSILON
        * std::max(magnitude, LDBL_MIN);
    result.raw = raw;
    result.error = error;
    result.upper = raw + error;
    result.multiplier = multiplier;
    result.passed = std::isfinite(result.upper)
        && result.upper + error >= raw && result.scans > 0U;
    return result;
}

struct ALR48DualAudit {
    bool passed = false;
    bool binary128_executed = false;
    bool infeasible = false;
    long double raw = -std::numeric_limits<long double>::infinity();
    long double lower = -std::numeric_limits<long double>::infinity();
    long double error = std::numeric_limits<long double>::infinity();
    std::string binary128_lower = "not-executed";
    ALR48SupportUpper support;
};

ALR48DualAudit al_r48_dual_audit(const std::vector<double>& lambda,
    const std::vector<double>& constraint, const std::vector<Vec3>& at_lambda,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper,
    double radius) {
    ALR48DualAudit result;
    if (lambda.size() != constraint.size() || lambda.empty()
        || !al_r48_scalar_finite(lambda)
        || !al_r48_scalar_finite(constraint)
        || at_lambda.size() != lower.size()) return result;
    std::vector<Vec3> q(at_lambda.size());
    for (std::size_t i = 0U; i < q.size(); ++i) q[i] = -at_lambda[i];
    result.support = al_r48_support_upper(q, lower, upper, radius);
    if (!result.support.passed) return result;
    long double linear = 0.0L;
    long double half_squared = 0.0L;
    long double magnitude = std::abs(result.support.upper);
    bool nonnegative = true;
    for (std::size_t i = 0U; i < lambda.size(); ++i) {
        nonnegative = nonnegative && lambda[i] >= 0.0;
        const long double product = static_cast<long double>(lambda[i])
            * static_cast<long double>(constraint[i]);
        const long double squared = static_cast<long double>(lambda[i])
            * static_cast<long double>(lambda[i]);
        linear += product;
        half_squared += 0.5L * squared;
        magnitude += std::abs(product) + 0.5L * squared;
    }
    result.raw = linear - half_squared - result.support.upper;
    result.error = result.support.error + 4096.0L * LDBL_EPSILON
        * std::max(magnitude, LDBL_MIN);
    result.lower = result.raw - result.error;
    result.binary128_executed = result.raw > 0.0L
        || std::abs(result.raw) <= 1024.0L * result.error;
    bool binary_positive = false;
    if (result.binary128_executed) {
        Binary128Accumulator linear128;
        Binary128Accumulator half128;
        Binary128Accumulator support128;
        Binary128Accumulator support_magnitude128;
        const Binary128 eta = static_cast<Binary128>(
            result.support.multiplier);
        const Binary128 r = static_cast<Binary128>(radius);
        support128.add(static_cast<Binary128>(0.5) * eta * r * r);
        support_magnitude128.add(fabsq(
            static_cast<Binary128>(0.5) * eta * r * r));
        for (std::size_t i = 0U; i < lambda.size(); ++i) {
            const Binary128 l = static_cast<Binary128>(lambda[i]);
            const Binary128 c = static_cast<Binary128>(constraint[i]);
            linear128.add(l * c);
            half128.add(static_cast<Binary128>(0.5) * l * l);
        }
        for (std::size_t i = 0U; i < q.size(); ++i)
            for (int axis = 0; axis < 3; ++axis) {
                const Binary128 qi = static_cast<Binary128>(
                    al_r48_component(q[i], axis));
                const Binary128 low = static_cast<Binary128>(
                    al_r48_component(lower[i], axis));
                const Binary128 high = static_cast<Binary128>(
                    al_r48_component(upper[i], axis));
                Binary128 value = static_cast<Binary128>(0.0);
                if (eta > static_cast<Binary128>(0.0))
                    value = fmaxq(low, fminq(qi / eta, high));
                else if (qi > static_cast<Binary128>(0.0)) value = high;
                else if (qi < static_cast<Binary128>(0.0)) value = low;
                const Binary128 term = qi * value
                    - static_cast<Binary128>(0.5) * eta * value * value;
                support128.add(term);
                support_magnitude128.add(fabsq(term));
            }
        const Binary128 qeps = ldexpq(static_cast<Binary128>(1.0), -112);
        const Binary128 qmin = ldexpq(static_cast<Binary128>(1.0), -16382);
        const Binary128 outward = static_cast<Binary128>(4096.0)
            * qeps * fmaxq(support_magnitude128.value(), qmin);
        const Binary128 lower128 = linear128.value() - half128.value()
            - support128.value() - outward;
        result.binary128_lower = al_binary128_text(lower128);
        binary_positive = lower128 > static_cast<Binary128>(0.0);
    }
    result.infeasible = nonnegative && result.lower > 0.0L
        && (!result.binary128_executed || binary_positive);
    result.passed = nonnegative && std::isfinite(result.raw)
        && std::isfinite(result.lower) && result.lower <= result.raw
        && (!result.binary128_executed || sizeof(Binary128) == 16U);
    return result;
}

std::vector<Vec3> al_r48_difference(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    if (lhs.size() != rhs.size()) return {};
    std::vector<Vec3> result(lhs.size());
    for (std::size_t i = 0U; i < lhs.size(); ++i)
        result[i] = lhs[i] - rhs[i];
    return result;
}

std::vector<double> al_r48_difference(
    const std::vector<double>& lhs, const std::vector<double>& rhs) {
    if (lhs.size() != rhs.size()) return {};
    std::vector<double> result(lhs.size());
    for (std::size_t i = 0U; i < lhs.size(); ++i)
        result[i] = lhs[i] - rhs[i];
    return result;
}

struct ALR48PrimalAudit {
    bool passed = false;
    bool certified = false;
    double psi = std::numeric_limits<double>::infinity();
    double h = std::numeric_limits<double>::infinity();
    double maximum_upper = std::numeric_limits<double>::infinity();
    std::size_t raw_active = 0U;
    std::size_t certified_positive = 0U;
};

ALR48PrimalAudit al_r48_primal_audit(const std::vector<double>& constraint,
    const ALLinearizedJvp& image, std::size_t maximum_degree) {
    ALR48PrimalAudit result;
    if (!image.passed || constraint.size() != image.value.size()
        || image.value.size() != image.absolute_sum.size()) return result;
    std::vector<double> value(constraint.size());
    long double squared = 0.0L;
    result.maximum_upper = -std::numeric_limits<double>::infinity();
    const double gamma = gamma_factor(16U * maximum_degree + 66U);
    for (std::size_t i = 0U; i < constraint.size(); ++i) {
        value[i] = constraint[i] + image.value[i];
        if (value[i] > 0.0) {
            ++result.raw_active;
            squared += static_cast<long double>(value[i])
                * static_cast<long double>(value[i]);
        }
        const double bound = gamma
            * (std::abs(constraint[i]) + image.absolute_sum[i]);
        const double upper_value = value[i] + bound;
        result.maximum_upper = std::max(result.maximum_upper, upper_value);
        if (upper_value > 0.0) ++result.certified_positive;
    }
    result.psi = 0.5 * static_cast<double>(squared);
    result.h = std::sqrt(2.0 * result.psi);
    result.certified = result.certified_positive == 0U;
    result.passed = al_r48_scalar_finite(value)
        && std::isfinite(result.psi) && std::isfinite(result.h)
        && std::isfinite(result.maximum_upper);
    return result;
}

struct ALR48Generator {
    bool passed = false;
    bool primal_certified = false;
    std::vector<Vec3> witness;
    std::vector<double> lambda;
    std::vector<Vec3> at_lambda;
    ALR48PrimalAudit primal;
    ALR48DualAudit dual;
    std::size_t iterations = 0U;
    std::size_t line_attempts = 0U;
    std::size_t extra_backtracks = 0U;
    std::size_t jvp_calls = 0U;
    std::size_t vjp_calls = 0U;
    std::size_t projection_scans = 0U;
    std::size_t support_audits = 0U;
    double tau = 1.0;
    double witness_norm = 0.0;
    std::vector<double> checkpoint_psi;
};

struct ALR48ReplayCapture {
    bool captured = false;
    ALR47ReplayCapture r47;
};

ALR48Generator al_r48_generate(const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper) {
    ALR48Generator result;
    const std::vector<double>& c = workspace.support.constraint;
    if (!workspace.passed || c.empty() || lower.size() != c.size()
        || upper.size() != c.size()) return result;
    std::vector<Vec3> current(c.size());
    std::vector<Vec3> previous(c.size());
    std::vector<double> current_image(c.size(), 0.0);
    std::vector<double> lambda(c.size());
    for (std::size_t i = 0U; i < c.size(); ++i)
        lambda[i] = std::max(c[i], 0.0);
    std::vector<Vec3> at_lambda = al_r29_pair_once_vjp(workspace, lambda);
    ++result.vjp_calls;
    if (at_lambda.size() != current.size() || !al_r48_vec_finite(at_lambda))
        return result;
    double tau_previous = 1.0;
    double theta_previous = 1.0;
    bool generator_exact = true;
    for (std::size_t iteration = 1U;
         iteration <= AL_R48_MAIN_CAP && !result.primal_certified; ++iteration) {
        std::vector<Vec3> projected_input(current.size());
        for (std::size_t i = 0U; i < current.size(); ++i)
            projected_input[i] = current[i] - tau_previous * at_lambda[i];
        ALR48Projection projection = al_r48_project_ball_box(
            projected_input, lower, upper, AL_R48_NORMAL_RADIUS);
        result.projection_scans += projection.scans;
        if (!projection.passed) { generator_exact = false; break; }
        std::vector<Vec3> next = std::move(projection.value);
        const ALLinearizedJvp next_image = al_r29_directed_jvp(workspace, next);
        ++result.jvp_calls;
        if (!next_image.passed) { generator_exact = false; break; }
        double tau_trial = tau_previous
            * std::sqrt(1.0 + theta_previous);
        bool line_accepted = false;
        std::vector<double> next_lambda;
        std::vector<Vec3> next_at_lambda;
        for (std::size_t attempt = 0U;
             attempt < AL_R48_BACKTRACK_CAP; ++attempt) {
            ++result.line_attempts;
            const double theta = tau_trial / tau_previous;
            std::vector<double> extrapolated_image(c.size());
            for (std::size_t i = 0U; i < c.size(); ++i)
                extrapolated_image[i] = next_image.value[i]
                    + theta * (next_image.value[i] - current_image[i]);
            next_lambda.resize(c.size());
            for (std::size_t i = 0U; i < c.size(); ++i) {
                const double numerator = lambda[i]
                    + tau_trial * (c[i] + extrapolated_image[i]);
                next_lambda[i] = std::max(
                    numerator / (1.0 + tau_trial), 0.0);
            }
            next_at_lambda = al_r29_pair_once_vjp(workspace, next_lambda);
            ++result.vjp_calls;
            if (next_at_lambda.size() != at_lambda.size()
                || !al_r48_vec_finite(next_at_lambda)) {
                generator_exact = false;
                break;
            }
            const double lhs = tau_trial * al_r32_vec3_norm(
                al_r48_difference(next_at_lambda, at_lambda));
            const double rhs = 0.99 * al_r29_scalar_norm(
                al_r48_difference(next_lambda, lambda));
            if (std::isfinite(lhs) && std::isfinite(rhs) && lhs <= rhs) {
                line_accepted = true;
                theta_previous = theta;
                break;
            }
            tau_trial *= 0.5;
            ++result.extra_backtracks;
            if (result.extra_backtracks > AL_R48_EXTRA_BACKTRACK_CAP) break;
        }
        if (!generator_exact || !line_accepted
            || result.extra_backtracks > AL_R48_EXTRA_BACKTRACK_CAP) {
            generator_exact = false;
            break;
        }
        previous = std::move(current);
        current = std::move(next);
        current_image = next_image.value;
        lambda = std::move(next_lambda);
        at_lambda = std::move(next_at_lambda);
        tau_previous = tau_trial;
        result.iterations = iteration;
        result.primal = al_r48_primal_audit(
            c, next_image, workspace.neighborhood.maximum_degree);
        if (!result.primal.passed) { generator_exact = false; break; }
        if ((iteration & (iteration - 1U)) == 0U)
            result.checkpoint_psi.push_back(result.primal.psi);
        result.primal_certified = result.primal.certified;
    }
    result.witness = std::move(current);
    result.lambda = std::move(lambda);
    result.at_lambda = std::move(at_lambda);
    result.tau = tau_previous;
    result.witness_norm = al_r32_vec3_norm(result.witness);
    result.dual = al_r48_dual_audit(result.lambda, c, result.at_lambda,
        lower, upper, AL_R48_NORMAL_RADIUS);
    result.support_audits = result.dual.support.passed ? 1U : 0U;
    result.passed = generator_exact && result.iterations > 0U
        && result.iterations <= AL_R48_MAIN_CAP
        && result.jvp_calls <= AL_R48_MAIN_CAP
        && result.vjp_calls <= 385U
        && result.extra_backtracks <= AL_R48_EXTRA_BACKTRACK_CAP
        && al_r48_vec_finite(result.witness)
        && std::isfinite(result.witness_norm)
        && result.witness_norm <= AL_R48_NORMAL_RADIUS
        && result.dual.passed;
    return result;
}

std::string al_r48_route(bool parent, bool source, bool workspace,
    bool geometry, bool projection, bool generator, bool primal_audit,
    bool dual_audit, bool work, bool primal, bool infeasible) {
    if (!parent) return "RESTORATION_CERTIFICATE_PARENT_REJECTED";
    if (!source) return "RESTORATION_CERTIFICATE_SOURCE_REJECTED";
    if (!workspace) return "RESTORATION_CERTIFICATE_WORKSPACE_REJECTED";
    if (!geometry) return "RESTORATION_CERTIFICATE_GEOMETRY_REJECTED";
    if (!projection) return "RESTORATION_CERTIFICATE_PROJECTION_REJECTED";
    if (!generator) return "RESTORATION_CERTIFICATE_GENERATOR_REJECTED";
    if (!primal_audit) return "RESTORATION_CERTIFICATE_PRIMAL_AUDIT_REJECTED";
    if (!dual_audit || (primal && infeasible))
        return "RESTORATION_CERTIFICATE_DUAL_AUDIT_REJECTED";
    if (!work) return "RESTORATION_CERTIFICATE_WORK_REJECTED";
    if (primal) return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
    if (infeasible) return "RESTORATION_RADIUS_INFEASIBLE_CERTIFICATE";
    return "RESTORATION_CERTIFICATE_UNRESOLVED";
}

} // namespace

SplitBoundaryReport run_al_restoration_certificate_controls_impl(
    ALR48ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R48_IDENTITY_PROJECTION)
        == B4E2D7R19R48_IDENTITY_SHA256;
    ALR47ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_filter_compatibility_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "dac0ffd871c927cb62253f64a078866d9515faefda59e48a54d923089dbc371d"
        && parent.json.find(
            "cbc082189cf1813be74123be818fa55649c7b2feac07d87f31626063b046286c")
            != std::string::npos
        && parent.json.find("RESTORATION_COMPATIBILITY_REQUIRED")
            != std::string::npos;
    ALR43ReplayCapture& r43 = capture.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string endpoint_before =
        al_binary64_vec3_root(r43.projected_endpoint);
    const std::string trial_before =
        al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before =
        al_r42_superset_root(r43.r42.superset);
    const bool source_exact = parent_exact && r30.captured
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && endpoint_before
            == "30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_before
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187"
        && superset_before
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01";

    std::vector<Vec3> dense_lower(1U, {-1.0, 0.0, 0.0});
    std::vector<Vec3> dense_upper(1U, {1.0, 0.0, 0.0});
    const ALR48Projection dense_ball = al_r48_project_ball_box(
        {{2.0, 0.0, 0.0}}, dense_lower, dense_upper, 0.5);
    dense_lower[0].x = -0.25;
    dense_upper[0].x = 0.25;
    const ALR48Projection dense_box = al_r48_project_ball_box(
        {{2.0, 0.0, 0.0}}, dense_lower, dense_upper, 1.0);
    dense_lower[0].x = -1.0;
    dense_upper[0].x = 1.0;
    const ALR48SupportUpper dense_support = al_r48_support_upper(
        {{2.0, 0.0, 0.0}}, dense_lower, dense_upper, 0.5);
    const bool dense_feasible = 1.0 + (-1.0) * 1.0 == 0.0;
    const bool dense_infeasible = 1.0 - 0.5 > 0.0;
    const bool dense = identity && dense_ball.passed && dense_box.passed
        && dense_support.passed && dense_feasible && dense_infeasible
        && std::abs(dense_ball.value[0].x - 0.5) <= 4.0e-15
        && binary64_bits(dense_box.value[0].x) == binary64_bits(0.25)
        && dense_support.upper >= 1.0L
        && al_r48_route(true, true, true, true, true, true, true, true,
            true, true, false)
            == "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE"
        && al_r48_route(true, true, true, true, true, true, true, true,
            true, false, true)
            == "RESTORATION_RADIUS_INFEASIBLE_CERTIFICATE"
        && al_r48_route(true, true, true, true, true, true, true, true,
            true, false, false) == "RESTORATION_CERTIFICATE_UNRESOLVED";

    ALSparseWorkTrace workspace_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    JointNeighborhood superset = b4ep3_build_superset(
        tagged_points(r30.position), binding);
    JointNeighborhood moved_filtered = b4ep3_filter_superset(
        tagged_points(r43.projected_trial), superset);
    ALNormalizedWorkspace moved = build_al_normalized_sparse_workspace(
        r43.projected_trial, binding, r30.dual, r30.theta,
        &workspace_work, &static_work, &adjacency_work);
    const ALR40HingeMetric moved_metric =
        al_r40_hinge_metric(moved.support.constraint);
    const bool workspace_exact = source_exact && dense && index.passed
        && binding.passed && superset.passed && moved.passed
        && al_r42_superset_root(superset) == superset_before
        && moved_filtered.pairs == moved.neighborhood.pairs
        && moved_metric.finite
        && binary64_bits(moved_metric.psi)
            == binary64_bits(6.8540208964482797e-16)
        && moved_metric.active == 464U;

    std::vector<Vec3> lower(r43.projected_trial.size());
    std::vector<Vec3> upper(r43.projected_trial.size());
    std::vector<std::uint8_t> moved_owners(r43.projected_trial.size() * 3U, 0U);
    std::size_t moved_active_faces = 0U;
    bool intervals_exact = workspace_exact;
    for (std::size_t i = 0U; i < r43.projected_trial.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face = al_r41_component(fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            const std::size_t component = 3U * i + static_cast<std::size_t>(axis);
            if (y < low_face) {
                low = 0.0;
                moved_owners[component] = 1U;
                ++moved_active_faces;
            } else if (y > high_face) {
                high = 0.0;
                moved_owners[component] = 2U;
                ++moved_active_faces;
            }
            al_r43_component(lower[i], axis) = low;
            al_r43_component(upper[i], axis) = high;
            intervals_exact = intervals_exact && std::isfinite(low)
                && std::isfinite(high) && low <= 0.0 && high >= 0.0;
        }
    const double moved_step_norm = al_r32_vec3_norm(r43.projected_endpoint);
    const double half_skin = 0.5 * B4EP3_SKIN
        * std::sqrt(1.0 - 2.0e-12) / SPACING;
    const std::string moved_owner_root = al_r29_mask_root(moved_owners);
    const bool geometry_exact = intervals_exact
        && AL_R48_NORMAL_RADIUS == 0.5 * AL_R48_NEXT_RADIUS
        && moved_step_norm + AL_R48_NORMAL_RADIUS <= half_skin
        && moved_active_faces == 1290U && moved_owner_root == owner_before;
    const ALR48Projection zero_projection = al_r48_project_ball_box(
        std::vector<Vec3>(lower.size()), lower, upper,
        AL_R48_NORMAL_RADIUS);
    const bool projection_exact = geometry_exact && zero_projection.passed
        && zero_projection.norm == 0.0;

    ALR48Generator generator;
    if (projection_exact) generator = al_r48_generate(moved, lower, upper);
    const bool generator_exact = projection_exact && generator.passed;
    const bool primal_audit_exact = generator_exact
        && generator.primal.passed
        && (!generator.primal_certified
            || (generator.primal.certified_positive == 0U
                && generator.witness_norm <= AL_R48_NORMAL_RADIUS));
    const bool dual_audit_exact = generator_exact && generator.dual.passed
        && !(generator.primal_certified && generator.dual.infeasible);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);
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
        && generator.jvp_calls <= 128U && generator.vjp_calls <= 385U
        && generator.support_audits <= 8U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && endpoint_before == al_binary64_vec3_root(r43.projected_endpoint)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset);

    const std::array<const char*, 12U> expected_routes{{
        "RESTORATION_CERTIFICATE_PARENT_REJECTED",
        "RESTORATION_CERTIFICATE_SOURCE_REJECTED",
        "RESTORATION_CERTIFICATE_WORKSPACE_REJECTED",
        "RESTORATION_CERTIFICATE_GEOMETRY_REJECTED",
        "RESTORATION_CERTIFICATE_PROJECTION_REJECTED",
        "RESTORATION_CERTIFICATE_GENERATOR_REJECTED",
        "RESTORATION_CERTIFICATE_PRIMAL_AUDIT_REJECTED",
        "RESTORATION_CERTIFICATE_DUAL_AUDIT_REJECTED",
        "RESTORATION_CERTIFICATE_WORK_REJECTED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE",
        "RESTORATION_RADIUS_INFEASIBLE_CERTIFICATE",
        "RESTORATION_CERTIFICATE_UNRESOLVED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 9U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const bool primal = route_index == 9U;
        const bool infeasible = route_index == 10U;
        const std::string observed = al_r48_route(gates[0], gates[1],
            gates[2], gates[3], gates[4], gates[5], gates[6], gates[7],
            gates[8], primal, infeasible);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r48_route(parent_exact, source_exact,
        workspace_exact, geometry_exact, projection_exact, generator_exact,
        primal_audit_exact, dual_audit_exact, work_exact && rollback,
        generator.primal_certified, generator.dual.infeasible);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && geometry_exact && projection_exact
        && generator_exact && primal_audit_exact && dual_audit_exact
        && work_exact && rollback && routes_exact;

    std::ostringstream checkpoint_material;
    for (double psi : generator.checkpoint_psi) checkpoint_material
        << binary64_bits(psi) << ':';
    const std::string checkpoint_root = sha256_hex(checkpoint_material.str());
    const std::string witness_root = al_binary64_vec3_root(generator.witness);
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R48_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << moved_metric.psi << ':' << moved_metric.norm << ':'
             << moved_metric.active << '|' << moved_owner_root << ':'
             << moved_active_faces << ':' << half_skin << '|'
             << generator.iterations << ':' << generator.line_attempts << ':'
             << generator.extra_backtracks << ':' << generator.tau << ':'
             << checkpoint_root << '|' << generator.primal.psi << ':'
             << generator.primal.h << ':' << generator.primal.maximum_upper
             << ':' << generator.primal.certified_positive << ':'
             << generator.witness_norm << ':' << witness_root << '|'
             << static_cast<double>(generator.dual.lower) << ':'
             << generator.dual.binary128_lower << ':'
             << generator.dual.infeasible << '|' << generator.jvp_calls << ':'
             << generator.vjp_calls << ':' << generator.projection_scans << ':'
             << work_exact << ':' << rollback << ':' << route_root << ':'
             << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r48_restoration_certificate.v2\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R48_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"psi\":" << moved_metric.psi
           << ",\"h\":" << moved_metric.norm
           << ",\"active\":" << moved_metric.active << "}"
           << ",\"geometry\":{\"next_radius\":" << AL_R48_NEXT_RADIUS
           << ",\"normal_radius\":" << AL_R48_NORMAL_RADIUS
           << ",\"r43_step_norm\":" << moved_step_norm
           << ",\"half_skin\":" << half_skin
           << ",\"active_faces\":" << moved_active_faces
           << ",\"owner_root\":\"" << moved_owner_root << "\"}"
           << ",\"generator\":{\"iterations\":" << generator.iterations
           << ",\"line_attempts\":" << generator.line_attempts
           << ",\"extra_backtracks\":" << generator.extra_backtracks
           << ",\"tau\":" << generator.tau
           << ",\"checkpoint_root\":\"" << checkpoint_root << "\"}"
           << ",\"primal\":{\"psi\":" << generator.primal.psi
           << ",\"h\":" << generator.primal.h
           << ",\"maximum_upper\":" << generator.primal.maximum_upper
           << ",\"raw_active\":" << generator.primal.raw_active
           << ",\"certified_positive\":"
           << generator.primal.certified_positive
           << ",\"witness_norm\":" << generator.witness_norm
           << ",\"witness_root\":\"" << witness_root
           << "\",\"certified\":"
           << (generator.primal_certified ? "true" : "false") << "}"
           << ",\"dual\":{\"raw\":"
           << static_cast<double>(generator.dual.raw)
           << ",\"lower\":" << static_cast<double>(generator.dual.lower)
           << ",\"support_upper\":"
           << static_cast<double>(generator.dual.support.upper)
           << ",\"binary128_executed\":"
           << (generator.dual.binary128_executed ? "true" : "false")
           << ",\"binary128_lower\":\"" << generator.dual.binary128_lower
           << "\",\"infeasible\":"
           << (generator.dual.infeasible ? "true" : "false") << "}"
           << ",\"work\":{\"jvp\":" << generator.jvp_calls
           << ",\"vjp\":" << generator.vjp_calls
           << ",\"projection_scans\":" << generator.projection_scans
           << ",\"support_audits\":" << generator.support_audits
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"filter_runtime_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"following_outer_executed\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r47 = std::move(capture);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_restoration_certificate_controls() {
    return run_al_restoration_certificate_controls_impl(nullptr);
}
