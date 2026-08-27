#define main nextengine_r63ze_embedded_main
#include "formula_denominator_residual_main.cpp"
#undef main

#include <cstdint>
#include <cstring>
#include <iomanip>
#include <utility>

namespace {

constexpr const char* R63ZE_RESULT_SHA256 =
    "2a291006153c6f91032c692c7d635c281e42cd81c4c73f5240881eb46b4b731c";
constexpr const char* R63ZE_STDOUT_SHA256 =
    "4fcb94cd94c176e515aedcdb7d0d724e17bac77ec0a94c215ced2dbf4d8ef4a4";
constexpr const char* R63ZE_TANGENT_PRODUCT_ROOT =
    "c24382a3f6e0dbb0a0566012e80dda2c3fb746d3577127e468030d3f39d29ff7";
constexpr const char* R63ZE_COMMON_PRODUCT_ROOT =
    "0edf282e6599b663d276c1e926a1fc04089e8d838cff38a6690776609e22112b";
constexpr const char* R63ZE_TANGENT_DENOMINATOR_ROOT =
    "13ddc46e643243b5b36ff2b48a15b397d216164110ce04d285409883e6e7347e";
constexpr const char* R63ZE_COMMON_DENOMINATOR_ROOT =
    "b2b6d74966dd1d5aeb2852932f4a62f46ce3b3113fc243f61da921ffdcc7a730";
constexpr const char* R63ZE_TANGENT_CERTIFICATE_ROOT =
    "d4ba011a2f5db87a970d4d0e93c922bd9ac4e0440d8412965b7fb657281b8ecd";
constexpr const char* R63ZE_COMMON_CERTIFICATE_ROOT =
    "591701263e1dc157acc68c5408d25a0da8c9375ee42b4d232adc3a0567bce5d4";
constexpr const char* R63ZE_TANGENT_RESIDUAL_ROOT =
    "42f88ce78aedcee893f42066e5f2b71d6ed335ea0ab61349030efdd446f570c5";
constexpr const char* R63ZE_COMMON_RESIDUAL_ROOT =
    "4e599be993d6508e062d50b41890aeded16cc61c983ed35fa4823b69611b1301";

FormulaProbeBinary128 positive_infinity() {
    return strtoflt128("inf", nullptr);
}

FormulaProbeBinary128 up_add(
    FormulaProbeBinary128 left, FormulaProbeBinary128 right) {
    return nextafterq(left + right, positive_infinity());
}

FormulaProbeBinary128 up_multiply(
    FormulaProbeBinary128 left, FormulaProbeBinary128 right) {
    return nextafterq(left * right, positive_infinity());
}

std::string binary64_bits_hex(double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    std::ostringstream stream;
    stream << "0x" << std::hex << std::setw(16) << std::setfill('0') << bits;
    return stream.str();
}

struct Interval {
    bool exact = false;
    FormulaProbeBinary128 center =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 bound =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 lower =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 upper =
        static_cast<FormulaProbeBinary128>(0.0);
    std::string producer_root;
    std::string root;
};

Interval make_interval(FormulaProbeBinary128 center,
    FormulaProbeBinary128 bound, const std::string& producer_root,
    bool producer_exact) {
    Interval result;
    result.center = center;
    result.bound = bound;
    result.producer_root = producer_root;
    result.exact = producer_exact && finiteq(center) != 0
        && finiteq(bound) != 0
        && bound >= static_cast<FormulaProbeBinary128>(0.0);
    if (result.exact) {
        result.lower = nextafterq(center - bound, -positive_infinity());
        result.upper = nextafterq(center + bound, positive_infinity());
        result.exact = finiteq(result.lower) != 0
            && finiteq(result.upper) != 0 && result.lower <= result.upper;
    }
    std::ostringstream material;
    material << result.exact << ':' << binary128_hex(result.center) << ':'
        << binary128_hex(result.bound) << ':' << binary128_hex(result.lower)
        << ':' << binary128_hex(result.upper) << ':' << producer_root;
    result.root = nextengine::nonlocal::sha256_hex(material.str());
    return result;
}

Interval scalar_interval(const FormulaProbeScalar& scalar,
    FormulaProbeBinary128 extra_bound =
        static_cast<FormulaProbeBinary128>(0.0)) {
    return make_interval(scalar.value, up_add(scalar.bound, extra_bound),
        scalar.root, scalar.exact);
}

bool interval_positive(const Interval& value) {
    return value.exact
        && value.lower > static_cast<FormulaProbeBinary128>(0.0);
}

bool interval_negative(const Interval& value) {
    return value.exact
        && value.upper < static_cast<FormulaProbeBinary128>(0.0);
}

bool intervals_overlap(const Interval& left, const Interval& right) {
    return left.exact && right.exact
        && std::max(left.lower, right.lower)
            <= std::min(left.upper, right.upper);
}

struct BoundedUpdate {
    bool exact = false;
    FormulaProbeBinary128 value =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 bound =
        static_cast<FormulaProbeBinary128>(0.0);
    std::string root;
};

BoundedUpdate bounded_update(FormulaProbeBinary128 left0,
    FormulaProbeBinary128 left1, FormulaProbeBinary128 right0,
    FormulaProbeBinary128 right1) {
    const FormulaProbeScalar scalar =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(
            {left0, left1}, {right0, right1});
    BoundedUpdate result;
    result.exact = scalar.exact && normal_or_zero(scalar.value)
        && finiteq(scalar.bound) != 0
        && scalar.bound >= static_cast<FormulaProbeBinary128>(0.0);
    result.value = scalar.value;
    result.bound = scalar.bound;
    result.root = nextengine::nonlocal::sha256_hex(
        std::string("bounded_update:") + scalar.root);
    return result;
}

struct AuditWork {
    std::size_t quadratic_dots = 0U;
    std::size_t product_difference_updates = 0U;
    std::size_t solution_difference_updates = 0U;
    std::size_t predicted_displacement_products = 0U;
    std::size_t scalar_difference_updates = 0U;
    std::size_t raw_sign_comparisons = 0U;
};

std::string audit_work_root(const AuditWork& work) {
    std::ostringstream material;
    material << work.quadratic_dots << ':'
        << work.product_difference_updates << ':'
        << work.solution_difference_updates << ':'
        << work.predicted_displacement_products << ':'
        << work.scalar_difference_updates << ':'
        << work.raw_sign_comparisons;
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool audit_work_equal(const AuditWork& left, const AuditWork& right) {
    return audit_work_root(left) == audit_work_root(right);
}

struct Primary {
    bool exact = false;
    bool complete = false;
    std::string failure;
    std::string fixture_root;
    std::string input_root;
    Work work;
    AuditWork audit_work;
    std::vector<std::string> solve_roots;
    std::vector<std::string> product_roots;
    std::vector<std::string> scalar_roots;
    std::vector<std::string> update_roots;
    std::vector<std::string> audit_update_roots;
    std::vector<FormulaProbeBinary128> p1;
    std::vector<FormulaProbeBinary128> tangent_product;
    std::vector<FormulaProbeBinary128> common_product;
    FormulaProbeScalar tangent_denominator;
    FormulaProbeScalar common_denominator;
    FormulaProbeBinary128 alpha_t =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 alpha_c =
        static_cast<FormulaProbeBinary128>(0.0);
    std::vector<FormulaProbeBinary128> x_t;
    std::vector<FormulaProbeBinary128> x_c;
    std::vector<FormulaProbeBinary128> r_t;
    std::vector<FormulaProbeBinary128> r_c;
    std::vector<FormulaProbeBinary128> x_t_bounds;
    std::vector<FormulaProbeBinary128> x_c_bounds;
    std::vector<FormulaProbeCertificate> certificates;
    Interval denominator_difference;
    Interval direct_quadratic_difference;
    Interval alpha_difference;
    std::vector<Interval> displacement_left;
    std::vector<Interval> displacement_right;
    std::vector<int> tangent_signs;
    std::vector<int> common_signs;
    std::size_t changed_signs = 0U;
    std::size_t zero_signs = 0U;
    std::string tangent_sign_root;
    std::string common_sign_root;
    std::string changed_index_root;
    std::string root;
};

std::string sign_root(const std::vector<int>& signs) {
    std::ostringstream material;
    for (int sign : signs) material << sign << ',';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string primary_root(const Primary& value) {
    using nextengine::nonlocal::fcr::formula_probe_binary128_vector_root;
    std::ostringstream material;
    material << value.exact << ':' << value.complete << ':' << value.failure
        << ':' << value.fixture_root << ':' << value.input_root << ':'
        << work_root(value.work) << ':' << audit_work_root(value.audit_work)
        << ':' << value.denominator_difference.root << ':'
        << value.direct_quadratic_difference.root << ':'
        << value.alpha_difference.root << ':' << value.tangent_sign_root
        << ':' << value.common_sign_root << ':' << value.changed_index_root
        << ':' << value.changed_signs << ':' << value.zero_signs << ':'
        << binary128_hex(value.tangent_denominator.value) << ':'
        << binary128_hex(value.tangent_denominator.bound) << ':'
        << binary128_hex(value.common_denominator.value) << ':'
        << binary128_hex(value.common_denominator.bound) << ':'
        << binary128_hex(value.alpha_t) << ':' << binary128_hex(value.alpha_c)
        << ":p1:" << formula_probe_binary128_vector_root(value.p1)
        << ":tp:" << formula_probe_binary128_vector_root(value.tangent_product)
        << ":cp:" << formula_probe_binary128_vector_root(value.common_product)
        << ":xt:" << formula_probe_binary128_vector_root(value.x_t)
        << ":xc:" << formula_probe_binary128_vector_root(value.x_c)
        << ":rt:" << formula_probe_binary128_vector_root(value.r_t)
        << ":rc:" << formula_probe_binary128_vector_root(value.r_c)
        << ":xtb:" << formula_probe_binary128_vector_root(value.x_t_bounds)
        << ":xcb:" << formula_probe_binary128_vector_root(value.x_c_bounds)
        << '|';
    for (const auto& root : value.solve_roots) material << "s:" << root << ';';
    for (const auto& root : value.product_roots) material << "p:" << root << ';';
    for (const auto& root : value.scalar_roots) material << "d:" << root << ';';
    for (const auto& root : value.update_roots) material << "u:" << root << ';';
    for (const auto& root : value.audit_update_roots)
        material << "a:" << root << ';';
    for (const auto& certificate : value.certificates)
        material << "c:" << certificate.root << ':'
            << binary64_bits_hex(certificate.error_upper) << ';';
    for (const auto& interval : value.displacement_left)
        material << "l:" << interval.root << ';';
    for (const auto& interval : value.displacement_right)
        material << "r:" << interval.root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

enum class R63ZFFault {
    None,
    FiniteOverflowScalar,
    NonpositiveTangentDenominator
};

Primary run_primary(const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& rhs,
    R63ZFFault injection = R63ZFFault::None) {
    Primary result;
    result.fixture_root = fixture.root;
    result.input_root =
        nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(rhs);
    const auto fail = [&](const std::string& stage) {
        result.failure = stage;
        result.root = primary_root(result);
        return result;
    };
    if (fixture.dimension != DIMENSION || rhs.size() != DIMENSION
        || !normal_vector(rhs))
        return fail("identity");
    if (injection == R63ZFFault::FiniteOverflowScalar) {
        const FormulaProbeBinary128 finite_max = strtoflt128(
            "1.18973149535723176508575932662800702e4932", nullptr);
        const FormulaProbeScalar overflow =
            nextengine::nonlocal::fcr::formula_probe_scalar_dot(
                {finite_max}, {finite_max});
        ++result.work.scalar_dots;
        result.scalar_roots.push_back(overflow.root);
        if (!overflow.exact) return fail("scalar_overflow");
        return fail("overflow_control_not_triggered");
    }

    const FormulaProbeSolve start =
        nextengine::nonlocal::fcr::formula_probe_factor_solve(
            fixture, rhs, fixture.inverse_scale);
    count_solve(result.work, start);
    result.solve_roots.push_back(start.root);
    if (!start.exact || !finite_vector(start.solution))
        return fail("initial_solve");
    const ProductResult hx0 = apply_product(fixture, false, start.solution);
    add_work(result.work, hx0.work);
    result.product_roots.push_back(hx0.root);
    if (!hx0.exact) return fail("initial_product");

    std::vector<FormulaProbeBinary128> r0(DIMENSION);
    bool updates_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const BoundedUpdate update = bounded_update(rhs[index],
            -hx0.value[index], static_cast<FormulaProbeBinary128>(1.0),
            static_cast<FormulaProbeBinary128>(1.0));
        updates_exact = updates_exact && update.exact;
        r0[index] = update.value;
        result.update_roots.push_back(update.root);
    }
    if (!updates_exact || !normal_vector(r0))
        return fail("initial_residual");
    const FormulaProbeSolve z0 =
        nextengine::nonlocal::fcr::formula_probe_factor_solve(
            fixture, r0, fixture.inverse_scale);
    count_solve(result.work, z0);
    result.solve_roots.push_back(z0.root);
    if (!z0.exact || !finite_vector(z0.solution))
        return fail("initial_preconditioner");
    const FormulaProbeScalar rho0 =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(r0, z0.solution);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(rho0.root);
    if (!rho0.exact || !rho0.positive) return fail("initial_rho");
    const std::vector<FormulaProbeBinary128> p0 = z0.solution;

    const ProductResult hp0 = apply_product(fixture, false, p0);
    add_work(result.work, hp0.work);
    result.product_roots.push_back(hp0.root);
    if (!hp0.exact) return fail("iteration0_product");
    const FormulaProbeScalar denominator0 =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(p0, hp0.value);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(denominator0.root);
    if (!denominator0.exact || !denominator0.positive)
        return fail("iteration0_denominator");
    const FormulaProbeBinary128 alpha0 = rho0.value / denominator0.value;
    ++result.work.scalar_divisions;
    if (!normal_or_zero(alpha0)) return fail("iteration0_alpha");
    std::vector<FormulaProbeBinary128> x1(DIMENSION);
    std::vector<FormulaProbeBinary128> r1(DIMENSION);
    updates_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const BoundedUpdate solution = bounded_update(start.solution[index],
            alpha0, static_cast<FormulaProbeBinary128>(1.0), p0[index]);
        const BoundedUpdate residual = bounded_update(r0[index], -alpha0,
            static_cast<FormulaProbeBinary128>(1.0), hp0.value[index]);
        updates_exact = updates_exact && solution.exact && residual.exact;
        x1[index] = solution.value;
        r1[index] = residual.value;
        result.update_roots.push_back(solution.root);
        result.update_roots.push_back(residual.root);
        ++result.work.solution_updates;
        ++result.work.residual_updates;
    }
    if (!updates_exact || !normal_vector(x1) || !normal_vector(r1))
        return fail("iteration0_update");
    const FormulaProbeSolve z1 =
        nextengine::nonlocal::fcr::formula_probe_factor_solve(
            fixture, r1, fixture.inverse_scale);
    count_solve(result.work, z1);
    result.solve_roots.push_back(z1.root);
    if (!z1.exact || !finite_vector(z1.solution))
        return fail("iteration1_preconditioner");
    const FormulaProbeScalar rho1 =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(r1, z1.solution);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(rho1.root);
    if (!rho1.exact || !rho1.positive) return fail("iteration1_rho");
    const FormulaProbeBinary128 beta1 = rho1.value / rho0.value;
    ++result.work.scalar_divisions;
    if (!normal_or_zero(beta1)) return fail("iteration1_beta");
    result.p1.resize(DIMENSION);
    updates_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const BoundedUpdate direction = bounded_update(z1.solution[index],
            beta1, static_cast<FormulaProbeBinary128>(1.0), p0[index]);
        updates_exact = updates_exact && direction.exact;
        result.p1[index] = direction.value;
        result.update_roots.push_back(direction.root);
        ++result.work.direction_updates;
    }
    if (!updates_exact || !normal_vector(result.p1))
        return fail("iteration1_direction");

    const ProductResult tangent = apply_product(fixture, false, result.p1);
    add_work(result.work, tangent.work);
    result.product_roots.push_back(tangent.root);
    if (!tangent.exact) return fail("tangent_product");
    const ProductResult common = apply_product(fixture, true, result.p1);
    add_work(result.work, common.work);
    result.product_roots.push_back(common.root);
    if (!common.exact) return fail("common_product");
    result.tangent_product = tangent.value;
    result.common_product = common.value;

    result.tangent_denominator =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(
            result.p1, result.tangent_product);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(result.tangent_denominator.root);
    if (!result.tangent_denominator.exact
        || !result.tangent_denominator.positive)
        return fail("tangent_denominator");
    result.common_denominator =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(
            result.p1, result.common_product);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(result.common_denominator.root);
    if (!result.common_denominator.exact
        || !result.common_denominator.positive)
        return fail("common_denominator");
    if (injection == R63ZFFault::NonpositiveTangentDenominator) {
        result.tangent_denominator.positive = false;
        result.tangent_denominator.value =
            static_cast<FormulaProbeBinary128>(0.0);
        result.tangent_denominator.root = nextengine::nonlocal::sha256_hex(
            result.tangent_denominator.root + ":nonpositive_fault");
        result.scalar_roots[3U] = result.tangent_denominator.root;
        return fail("selected_denominator");
    }

    result.alpha_t = rho1.value / result.tangent_denominator.value;
    result.alpha_c = rho1.value / result.common_denominator.value;
    result.work.scalar_divisions += 2U;
    if (!normal_or_zero(result.alpha_t) || !normal_or_zero(result.alpha_c))
        return fail("final_alpha");
    result.x_t.resize(DIMENSION);
    result.x_c.resize(DIMENSION);
    result.r_t.resize(DIMENSION);
    result.r_c.resize(DIMENSION);
    result.x_t_bounds.resize(DIMENSION);
    result.x_c_bounds.resize(DIMENSION);
    updates_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const BoundedUpdate xt = bounded_update(x1[index], result.alpha_t,
            static_cast<FormulaProbeBinary128>(1.0), result.p1[index]);
        const BoundedUpdate rt = bounded_update(r1[index], -result.alpha_t,
            static_cast<FormulaProbeBinary128>(1.0),
            result.tangent_product[index]);
        const BoundedUpdate xc = bounded_update(x1[index], result.alpha_c,
            static_cast<FormulaProbeBinary128>(1.0), result.p1[index]);
        const BoundedUpdate rc = bounded_update(r1[index], -result.alpha_c,
            static_cast<FormulaProbeBinary128>(1.0),
            result.common_product[index]);
        updates_exact = updates_exact && xt.exact && rt.exact
            && xc.exact && rc.exact;
        result.x_t[index] = xt.value;
        result.r_t[index] = rt.value;
        result.x_c[index] = xc.value;
        result.r_c[index] = rc.value;
        result.x_t_bounds[index] = xt.bound;
        result.x_c_bounds[index] = xc.bound;
        result.update_roots.push_back(xt.root);
        result.update_roots.push_back(rt.root);
        result.update_roots.push_back(xc.root);
        result.update_roots.push_back(rc.root);
        result.work.solution_updates += 2U;
        result.work.residual_updates += 2U;
    }
    if (!updates_exact || !normal_vector(result.x_t)
        || !normal_vector(result.x_c) || !normal_vector(result.r_t)
        || !normal_vector(result.r_c))
        return fail("final_update");

    result.certificates = {
        nextengine::nonlocal::fcr::formula_probe_certificate(
            fixture, result.x_t),
        nextengine::nonlocal::fcr::formula_probe_certificate(
            fixture, result.x_c)};
    result.work.certificates = 2U;
    if (!result.certificates[0U].exact || !result.certificates[1U].exact)
        return fail("certificate");

    const BoundedUpdate denominator_delta = bounded_update(
        result.common_denominator.value, -result.tangent_denominator.value,
        static_cast<FormulaProbeBinary128>(1.0),
        static_cast<FormulaProbeBinary128>(1.0));
    ++result.audit_work.scalar_difference_updates;
    result.audit_update_roots.push_back(denominator_delta.root);
    updates_exact = updates_exact && denominator_delta.exact;
    result.denominator_difference = make_interval(denominator_delta.value,
        up_add(denominator_delta.bound,
            up_add(result.common_denominator.bound,
                result.tangent_denominator.bound)),
        denominator_delta.root, denominator_delta.exact);

    std::vector<FormulaProbeBinary128> product_delta(DIMENSION);
    std::vector<FormulaProbeBinary128> product_delta_bound(DIMENSION);
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const BoundedUpdate delta = bounded_update(
            result.common_product[index], -result.tangent_product[index],
            static_cast<FormulaProbeBinary128>(1.0),
            static_cast<FormulaProbeBinary128>(1.0));
        updates_exact = updates_exact && delta.exact;
        product_delta[index] = delta.value;
        product_delta_bound[index] = delta.bound;
        ++result.audit_work.product_difference_updates;
        result.audit_update_roots.push_back(delta.root);
    }
    const FormulaProbeScalar direct =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(
            result.p1, product_delta);
    ++result.audit_work.quadratic_dots;
    result.scalar_roots.push_back(direct.root);
    FormulaProbeBinary128 propagated =
        static_cast<FormulaProbeBinary128>(0.0);
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        propagated = up_add(propagated, up_multiply(
            fabsq(result.p1[index]), product_delta_bound[index]));
    }
    result.direct_quadratic_difference = scalar_interval(direct, propagated);

    const BoundedUpdate alpha_delta = bounded_update(result.alpha_c,
        -result.alpha_t, static_cast<FormulaProbeBinary128>(1.0),
        static_cast<FormulaProbeBinary128>(1.0));
    ++result.audit_work.scalar_difference_updates;
    result.audit_update_roots.push_back(alpha_delta.root);
    updates_exact = updates_exact && alpha_delta.exact;
    result.alpha_difference = make_interval(alpha_delta.value,
        alpha_delta.bound, alpha_delta.root, alpha_delta.exact);
    result.displacement_left.reserve(DIMENSION);
    result.displacement_right.reserve(DIMENSION);
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const BoundedUpdate observed = bounded_update(result.x_c[index],
            -result.x_t[index], static_cast<FormulaProbeBinary128>(1.0),
            static_cast<FormulaProbeBinary128>(1.0));
        ++result.audit_work.solution_difference_updates;
        result.audit_update_roots.push_back(observed.root);
        updates_exact = updates_exact && observed.exact;
        result.displacement_left.push_back(make_interval(observed.value,
            up_add(observed.bound,
                up_add(result.x_c_bounds[index], result.x_t_bounds[index])),
            observed.root, observed.exact));
        const FormulaProbeScalar predicted =
            nextengine::nonlocal::fcr::formula_probe_scalar_dot(
                {alpha_delta.value}, {result.p1[index]});
        ++result.audit_work.predicted_displacement_products;
        result.displacement_right.push_back(scalar_interval(predicted,
            up_multiply(fabsq(result.p1[index]), alpha_delta.bound)));
    }

    result.tangent_signs.resize(DIMENSION);
    result.common_signs.resize(DIMENSION);
    std::ostringstream changed_material;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        result.tangent_signs[index] = result.x_t[index]
                > static_cast<FormulaProbeBinary128>(0.0)
            ? 1 : result.x_t[index] < static_cast<FormulaProbeBinary128>(0.0)
                ? -1 : 0;
        result.common_signs[index] = result.x_c[index]
                > static_cast<FormulaProbeBinary128>(0.0)
            ? 1 : result.x_c[index] < static_cast<FormulaProbeBinary128>(0.0)
                ? -1 : 0;
        result.zero_signs += result.tangent_signs[index] == 0 ? 1U : 0U;
        result.zero_signs += result.common_signs[index] == 0 ? 1U : 0U;
        if (result.tangent_signs[index] != result.common_signs[index]) {
            ++result.changed_signs;
            changed_material << index << ';';
        }
        result.audit_work.raw_sign_comparisons += 2U;
    }
    result.tangent_sign_root = sign_root(result.tangent_signs);
    result.common_sign_root = sign_root(result.common_signs);
    result.changed_index_root = nextengine::nonlocal::sha256_hex(
        changed_material.str());

    bool displacement = result.displacement_left.size() == DIMENSION
        && result.displacement_right.size() == DIMENSION;
    for (std::size_t index = 0U; index < DIMENSION && displacement; ++index)
        displacement = intervals_overlap(result.displacement_left[index],
            result.displacement_right[index]);
    result.complete = true;
    result.exact = updates_exact && result.denominator_difference.exact
        && result.direct_quadratic_difference.exact
        && result.alpha_difference.exact && displacement
        && result.product_roots.size() == 4U
        && result.solve_roots.size() == 3U
        && result.scalar_roots.size() == 6U
        && result.update_roots.size() == 816U
        && result.audit_update_roots.size() == 206U;
    result.root = primary_root(result);
    return result;
}

struct PrimaryAudit {
    bool apparatus = false;
    bool endpoint = false;
    bool quadratic = false;
    bool displacement = false;
    bool no_raw_zero = false;
    bool tangent_pass_common_reject = false;
    bool common_error_larger = false;
};

PrimaryAudit audit_primary(const Primary& value) {
    PrimaryAudit result;
    result.apparatus = value.exact && value.complete
        && value.p1.size() == DIMENSION
        && value.tangent_product.size() == DIMENSION
        && value.common_product.size() == DIMENSION
        && value.x_t.size() == DIMENSION && value.x_c.size() == DIMENSION
        && value.r_t.size() == DIMENSION && value.r_c.size() == DIMENSION
        && value.certificates.size() == 2U
        && value.solve_roots.size() == 3U
        && value.product_roots.size() == 4U
        && value.scalar_roots.size() == 6U
        && value.update_roots.size() == 816U
        && value.audit_update_roots.size() == 206U
        && value.displacement_left.size() == DIMENSION
        && value.displacement_right.size() == DIMENSION
        && value.tangent_signs.size() == DIMENSION
        && value.common_signs.size() == DIMENSION;
    if (!result.apparatus) return result;
    using nextengine::nonlocal::fcr::formula_probe_binary128_vector_root;
    result.endpoint = value.product_roots[2U] == R63ZE_TANGENT_PRODUCT_ROOT
        && value.product_roots[3U] == R63ZE_COMMON_PRODUCT_ROOT
        && value.tangent_denominator.root
            == R63ZE_TANGENT_DENOMINATOR_ROOT
        && value.common_denominator.root == R63ZE_COMMON_DENOMINATOR_ROOT
        && binary128_hex(value.tangent_denominator.value)
            == "0x1.0aa2a45ce0a757d6d8d6a216fb75p-2"
        && binary128_hex(value.common_denominator.value)
            == "0x1.0d0965b4944f15fb2a261d799f16p-2"
        && binary128_hex(value.alpha_t)
            == "0x1.12a78d1467ea7a19c272f3cdbfeep+0"
        && binary128_hex(value.alpha_c)
            == "0x1.1033f5858aac3dfb3b0ae944a492p+0"
        && formula_probe_binary128_vector_root(value.x_t)
            == TANGENT_STATE2_SOLUTION_ROOT
        && formula_probe_binary128_vector_root(value.x_c)
            == COMMON_STATE2_SOLUTION_ROOT
        && formula_probe_binary128_vector_root(value.r_t)
            == R63ZE_TANGENT_RESIDUAL_ROOT
        && formula_probe_binary128_vector_root(value.r_c)
            == R63ZE_COMMON_RESIDUAL_ROOT
        && value.certificates[0U].root == R63ZE_TANGENT_CERTIFICATE_ROOT
        && value.certificates[1U].root == R63ZE_COMMON_CERTIFICATE_ROOT
        && certificate_matches(
            value.certificates[0U], true, 24U, 78U, 0U)
        && certificate_matches(
            value.certificates[1U], false, 12U, 24U, 66U);
    result.quadratic = intervals_overlap(value.denominator_difference,
            value.direct_quadratic_difference)
        && interval_positive(value.denominator_difference)
        && interval_positive(value.direct_quadratic_difference)
        && interval_negative(value.alpha_difference);
    result.displacement = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index)
        result.displacement = result.displacement
            && intervals_overlap(value.displacement_left[index],
                value.displacement_right[index]);
    result.no_raw_zero = value.zero_signs == 0U;
    result.tangent_pass_common_reject = value.certificates[0U].passed
        && !value.certificates[1U].passed;
    result.common_error_larger = value.certificates[1U].error_upper
        > value.certificates[0U].error_upper;
    return result;
}

std::string classify_r63zf(bool apparatus, bool identity, bool work,
    bool endpoint, bool quadratic, bool displacement, bool no_raw_zero,
    std::size_t changed_signs, bool tangent_pass_common_reject,
    bool common_error_larger) {
    if (!apparatus) return "QUADRATIC_STEP_SIGN_APPARATUS_REJECTED";
    if (!identity) return "QUADRATIC_STEP_SIGN_IDENTITY_REJECTED";
    if (!work) return "QUADRATIC_STEP_SIGN_WORK_REJECTED";
    if (!endpoint) return "QUADRATIC_STEP_SIGN_ENDPOINT_REJECTED";
    if (!quadratic) return "QUADRATIC_DISCREPANCY_CONTAINMENT_REJECTED";
    if (!displacement) return "STEP_DISPLACEMENT_CONTAINMENT_REJECTED";
    if (!no_raw_zero) return "RAW_SOLUTION_ZERO_BOUNDARY_REJECTED";
    if (changed_signs != 0U) return "RAW_SIGN_CHANGE_CONTRIBUTES";
    if (tangent_pass_common_reject && common_error_larger)
        return "CERTIFICATE_ENCLOSURE_AMPLIFICATION_CANDIDATE";
    return "CERTIFICATE_MARGIN_REPRESENTATION_AUDIT_REQUIRED";
}

bool classifier_r63zf_control() {
    bool exact = true;
    for (unsigned mask = 0U; mask < 64U; ++mask) {
        const bool apparatus = (mask & 1U) != 0U;
        const bool identity = (mask & 2U) != 0U;
        const bool work = (mask & 4U) != 0U;
        const bool endpoint = (mask & 8U) != 0U;
        const bool quadratic = (mask & 16U) != 0U;
        const bool displacement = (mask & 32U) != 0U;
        const char* expected = !apparatus
            ? "QUADRATIC_STEP_SIGN_APPARATUS_REJECTED"
            : !identity ? "QUADRATIC_STEP_SIGN_IDENTITY_REJECTED"
            : !work ? "QUADRATIC_STEP_SIGN_WORK_REJECTED"
            : !endpoint ? "QUADRATIC_STEP_SIGN_ENDPOINT_REJECTED"
            : !quadratic ? "QUADRATIC_DISCREPANCY_CONTAINMENT_REJECTED"
            : !displacement ? "STEP_DISPLACEMENT_CONTAINMENT_REJECTED"
            : "CERTIFICATE_MARGIN_REPRESENTATION_AUDIT_REQUIRED";
        exact = exact && classify_r63zf(apparatus, identity, work, endpoint,
            quadratic, displacement, true, 0U, false, false) == expected;
    }
    for (unsigned mask = 0U; mask < 16U; ++mask) {
        const bool no_raw_zero = (mask & 1U) != 0U;
        const std::size_t changed = (mask & 2U) != 0U ? 1U : 0U;
        const bool ladder = (mask & 4U) != 0U;
        const bool larger = (mask & 8U) != 0U;
        const char* expected = !no_raw_zero
            ? "RAW_SOLUTION_ZERO_BOUNDARY_REJECTED"
            : changed != 0U ? "RAW_SIGN_CHANGE_CONTRIBUTES"
            : ladder && larger
                ? "CERTIFICATE_ENCLOSURE_AMPLIFICATION_CANDIDATE"
                : "CERTIFICATE_MARGIN_REPRESENTATION_AUDIT_REQUIRED";
        exact = exact && classify_r63zf(true, true, true, true, true, true,
            no_raw_zero, changed, ladder, larger) == expected;
    }
    return exact;
}

std::string result_identity(const std::string& route,
    const std::string& fixture_root, const std::string& primary_root_value,
    const std::string& core_work_root, const std::string& audit_root,
    const std::string& controls_root) {
    std::ostringstream semantic;
    semantic << route << ':' << fixture_root << ':' << R63ZE_RESULT_SHA256
        << ':' << R63ZE_STDOUT_SHA256 << ':' << primary_root_value << ':'
        << core_work_root << ':' << audit_root << ':' << controls_root;
    return nextengine::nonlocal::sha256_hex(semantic.str());
}

} // namespace

#ifndef NEXTENGINE_R63ZF_EMBEDDED
int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-quadratic-step-sign "
                         "<parent-fixture-cache>\n";
            return 2;
        }
        using namespace nextengine::nonlocal::fcr;
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const bool identity = formula_probe_parent_fixture_valid(fixture);
        const Primary primary = run_primary(fixture, fixture.original_rhs);
        const PrimaryAudit audit = audit_primary(primary);

        const Work expected{2U, 4U, 3U, 30906U, 612U, 5U, 4U,
            306U, 306U, 102U, 945U, 306U, 96390U, 96390U, 306U,
            102U, 10404U, 0U};
        const AuditWork audit_expected{1U, 102U, 102U, 102U, 2U, 204U};
        const bool work = work_equal(primary.work, expected)
            && audit_work_equal(primary.audit_work, audit_expected);

        const bool classifier = classifier_r63zf_control();
        Interval bound_mutation = primary.denominator_difference;
        bound_mutation.bound = nextafterq(
            bound_mutation.bound, positive_infinity());
        bound_mutation = make_interval(bound_mutation.center,
            bound_mutation.bound, bound_mutation.producer_root,
            primary.denominator_difference.exact);
        Primary bound_primary = primary;
        bound_primary.denominator_difference = bound_mutation;
        bound_primary.root = primary_root(bound_primary);
        const bool bound_control = bound_mutation.center
                == primary.denominator_difference.center
            && bound_mutation.root != primary.denominator_difference.root
            && bound_primary.root != primary.root;
        const Interval reversed = make_interval(
            -primary.denominator_difference.center,
            primary.denominator_difference.bound,
            nextengine::nonlocal::sha256_hex(
                primary.denominator_difference.producer_root + ":reversed"),
            primary.denominator_difference.exact);
        const bool reverse_control = interval_negative(reversed);
        Primary sign_primary = primary;
        if (!sign_primary.tangent_signs.empty())
            sign_primary.tangent_signs[0U] =
                -sign_primary.tangent_signs[0U];
        sign_primary.tangent_sign_root = sign_root(sign_primary.tangent_signs);
        sign_primary.changed_signs = 1U;
        sign_primary.changed_index_root =
            nextengine::nonlocal::sha256_hex("0;");
        sign_primary.root = primary_root(sign_primary);
        const bool sign_control = !sign_primary.tangent_signs.empty()
            && sign_primary.tangent_sign_root != primary.tangent_sign_root
            && sign_primary.root != primary.root
            && classify_r63zf(true, true, true, true, true, true, true, 1U,
                true, true) == "RAW_SIGN_CHANGE_CONTRIBUTES";
        const bool zero_control = classify_r63zf(true, true, true, true, true,
            true, false, 0U, true, true)
            == "RAW_SOLUTION_ZERO_BOUNDARY_REJECTED";

        std::vector<FormulaProbeBinary128> invalid(DIMENSION - 1U,
            static_cast<FormulaProbeBinary128>(1.0));
        const Primary invalid_primary = run_primary(fixture, invalid);
        const Work no_work;
        const AuditWork no_audit_work;
        const bool invalid_control = !invalid_primary.exact
            && !invalid_primary.complete && invalid_primary.failure == "identity"
            && work_equal(invalid_primary.work, no_work)
            && audit_work_equal(invalid_primary.audit_work, no_audit_work);
        const Primary overflow = run_primary(fixture, fixture.original_rhs,
            R63ZFFault::FiniteOverflowScalar);
        Work overflow_expected;
        overflow_expected.scalar_dots = 1U;
        const PrimaryAudit overflow_audit = audit_primary(overflow);
        const std::string overflow_route = classify_r63zf(
            overflow_audit.apparatus, true, true, true, true, true, true,
            0U, true, true);
        const bool overflow_control = !overflow.exact && !overflow.complete
            && overflow.failure == "scalar_overflow"
            && overflow.solve_roots.empty()
            && overflow.product_roots.empty()
            && overflow.scalar_roots.size() == 1U
            && overflow.x_t.empty() && overflow.x_c.empty()
            && overflow.r_t.empty() && overflow.r_c.empty()
            && overflow.certificates.empty()
            && work_equal(overflow.work, overflow_expected)
            && audit_work_equal(overflow.audit_work, no_audit_work)
            && !overflow_audit.apparatus
            && overflow_route == "QUADRATIC_STEP_SIGN_APPARATUS_REJECTED";
        const Primary nonpositive = run_primary(fixture,
            fixture.original_rhs, R63ZFFault::NonpositiveTangentDenominator);
        const Work nonpositive_expected{0U, 4U, 3U, 30906U, 612U, 5U, 2U,
            102U, 102U, 102U, 945U, 306U, 96390U, 96390U, 306U,
            102U, 10404U, 0U};
        const bool nonpositive_control = !nonpositive.exact
            && !nonpositive.complete
            && nonpositive.failure == "selected_denominator"
            && nonpositive.x_t.empty() && nonpositive.certificates.empty()
            && work_equal(nonpositive.work, nonpositive_expected);

        Primary incomplete = primary;
        incomplete.exact = false;
        incomplete.complete = false;
        incomplete.x_c.clear();
        incomplete.certificates.clear();
        incomplete.root = primary_root(incomplete);
        const PrimaryAudit incomplete_audit = audit_primary(incomplete);
        const std::string incomplete_route = classify_r63zf(
            incomplete_audit.apparatus, true, true, true, true, true, true,
            0U, true, true);
        const bool incomplete_control = !incomplete_audit.apparatus
            && incomplete_route == "QUADRATIC_STEP_SIGN_APPARATUS_REJECTED"
            && incomplete.x_c.empty() && incomplete.certificates.empty();

        const bool local_controls = classifier && bound_control && reverse_control
            && sign_control && zero_control && invalid_control
            && overflow_control && nonpositive_control && incomplete_control;
        std::ostringstream control_material;
        control_material << classifier << ':' << bound_control << ':'
            << bound_mutation.root << ':' << bound_primary.root << ':'
            << reverse_control << ':' << reversed.root << ':' << sign_control
            << ':' << sign_primary.tangent_sign_root << ':'
            << sign_primary.root << ':' << zero_control
            << ':' << invalid_control << ':' << invalid_primary.root << ':'
            << overflow_control << ':' << overflow.root << ':'
            << work_root(overflow.work) << ':' << overflow_route << ':'
            << nonpositive_control << ':' << nonpositive.root << ':'
            << incomplete_control << ':' << incomplete.root << ':'
            << incomplete_route;
        const std::string preliminary_controls_root =
            nextengine::nonlocal::sha256_hex(control_material.str());

        const std::string preliminary_route = classify_r63zf(
            audit.apparatus && local_controls, identity, work, audit.endpoint,
            audit.quadratic, audit.displacement, audit.no_raw_zero,
            primary.changed_signs, audit.tangent_pass_common_reject,
            audit.common_error_larger);
        const std::string preliminary_result = result_identity(
            preliminary_route, fixture.root, primary.root,
            work_root(primary.work), audit_work_root(primary.audit_work),
            preliminary_controls_root);
        const std::string preliminary_bound_result = result_identity(
            preliminary_route, fixture.root, bound_primary.root,
            work_root(primary.work), audit_work_root(primary.audit_work),
            preliminary_controls_root);
        const std::string preliminary_sign_result = result_identity(
            "RAW_SIGN_CHANGE_CONTRIBUTES", fixture.root, sign_primary.root,
            work_root(primary.work), audit_work_root(primary.audit_work),
            preliminary_controls_root);
        const bool result_sealing_control =
            preliminary_bound_result != preliminary_result
            && preliminary_sign_result != preliminary_result;
        control_material << ':' << result_sealing_control << ':'
            << preliminary_result << ':' << preliminary_bound_result << ':'
            << preliminary_sign_result;
        const std::string controls_root =
            nextengine::nonlocal::sha256_hex(control_material.str());
        const bool controls = local_controls && result_sealing_control;
        const std::string route = classify_r63zf(
            audit.apparatus && controls, identity, work, audit.endpoint,
            audit.quadratic, audit.displacement, audit.no_raw_zero,
            primary.changed_signs, audit.tangent_pass_common_reject,
            audit.common_error_larger);
        const std::string result_root = result_identity(route, fixture.root,
            primary.root, work_root(primary.work),
            audit_work_root(primary.audit_work), controls_root);
        const std::string bound_result_root = result_identity(route,
            fixture.root, bound_primary.root, work_root(primary.work),
            audit_work_root(primary.audit_work), controls_root);
        const std::string sign_result_root = result_identity(
            "RAW_SIGN_CHANGE_CONTRIBUTES", fixture.root, sign_primary.root,
            work_root(primary.work), audit_work_root(primary.audit_work),
            controls_root);
        const bool final_result_sealing = bound_result_root != result_root
            && sign_result_root != result_root;
        const bool exact = audit.apparatus && identity && work && controls
            && final_result_sealing
            && audit.endpoint && audit.quadratic && audit.displacement
            && audit.no_raw_zero
            && (route == "RAW_SIGN_CHANGE_CONTRIBUTES"
                || route == "CERTIFICATE_ENCLOSURE_AMPLIFICATION_CANDIDATE"
                || route == "CERTIFICATE_MARGIN_REPRESENTATION_AUDIT_REQUIRED");
        std::cout
            << "{\"schema\":\"nextengine.nonlocal.r63zf_quadratic_step_sign.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"parent_result_sha256\":\"" << R63ZE_RESULT_SHA256
            << "\",\"parent_stdout_sha256\":\"" << R63ZE_STDOUT_SHA256
            << "\",\"quadratic\":{\"tangent\":{\"center_hex\":\""
            << binary128_hex(primary.tangent_denominator.value)
            << "\",\"bound_hex\":\""
            << binary128_hex(primary.tangent_denominator.bound)
            << "\",\"root\":\"" << primary.tangent_denominator.root
            << "\"},\"common\":{\"center_hex\":\""
            << binary128_hex(primary.common_denominator.value)
            << "\",\"bound_hex\":\""
            << binary128_hex(primary.common_denominator.bound)
            << "\",\"root\":\"" << primary.common_denominator.root
            << "\"},\"difference\":{\"center_hex\":\""
            << binary128_hex(primary.denominator_difference.center)
            << "\",\"bound_hex\":\""
            << binary128_hex(primary.denominator_difference.bound)
            << "\",\"root\":\"" << primary.denominator_difference.root
            << "\"},\"direct\":{\"center_hex\":\""
            << binary128_hex(primary.direct_quadratic_difference.center)
            << "\",\"bound_hex\":\""
            << binary128_hex(primary.direct_quadratic_difference.bound)
            << "\",\"root\":\""
            << primary.direct_quadratic_difference.root << "\"}}"
            << ",\"step\":{\"alpha_t_hex\":\""
            << binary128_hex(primary.alpha_t)
            << "\",\"alpha_c_hex\":\"" << binary128_hex(primary.alpha_c)
            << "\",\"difference_center_hex\":\""
            << binary128_hex(primary.alpha_difference.center)
            << "\",\"difference_bound_hex\":\""
            << binary128_hex(primary.alpha_difference.bound)
            << "\",\"contained_components\":" << DIMENSION
            << ",\"tangent_solution_root\":\""
            << formula_probe_binary128_vector_root(primary.x_t)
            << "\",\"common_solution_root\":\""
            << formula_probe_binary128_vector_root(primary.x_c)
            << "\",\"tangent_residual_root\":\""
            << formula_probe_binary128_vector_root(primary.r_t)
            << "\",\"common_residual_root\":\""
            << formula_probe_binary128_vector_root(primary.r_c) << "\"}"
            << ",\"raw_signs\":{\"tangent_root\":\""
            << primary.tangent_sign_root << "\",\"common_root\":\""
            << primary.common_sign_root << "\",\"changed\":"
            << primary.changed_signs << ",\"zeros\":" << primary.zero_signs
            << ",\"changed_index_root\":\"" << primary.changed_index_root
            << "\"},\"certificates\":[";
        for (std::size_t index = 0U; index < primary.certificates.size();
             ++index) {
            if (index != 0U) std::cout << ',';
            const FormulaProbeCertificate& certificate =
                primary.certificates[index];
            std::cout << "{\"endpoint\":\""
                << (index == 0U ? "tangent" : "common")
                << "\",\"solution_root\":\""
                << certificate.solution_root << "\",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"positive\":" << certificate.positive
                << ",\"negative\":" << certificate.negative
                << ",\"unresolved\":" << certificate.unresolved
                << ",\"error_upper_bits\":\""
                << binary64_bits_hex(certificate.error_upper)
                << "\",\"root\":\"" << certificate.root << "\"}";
        }
        std::cout << "],\"controls\":{\"classifier\":"
            << (classifier ? "true" : "false")
            << ",\"bound_mutation\":{\"passed\":"
            << (bound_control ? "true" : "false")
            << ",\"primary_root\":\"" << bound_primary.root
            << "\",\"result_root\":\"" << bound_result_root << "\"}"
            << ",\"reversed_discrepancy\":"
            << (reverse_control ? "true" : "false")
            << ",\"sign_mutation\":{\"passed\":"
            << (sign_control ? "true" : "false")
            << ",\"sign_root\":\"" << sign_primary.tangent_sign_root
            << "\",\"primary_root\":\"" << sign_primary.root
            << "\",\"result_root\":\"" << sign_result_root << "\"}"
            << ",\"raw_zero\":" << (zero_control ? "true" : "false")
            << ",\"invalid\":" << (invalid_control ? "true" : "false")
            << ",\"overflow\":{\"passed\":"
            << (overflow_control ? "true" : "false")
            << ",\"work_root\":\"" << work_root(overflow.work)
            << "\",\"primary_root\":\"" << overflow.root
            << "\",\"route\":\"" << overflow_route << "\"}"
            << ",\"nonpositive\":"
            << (nonpositive_control ? "true" : "false")
            << ",\"incomplete_endpoint\":{\"passed\":"
            << (incomplete_control ? "true" : "false")
            << ",\"route\":\"" << incomplete_route << "\"}"
            << ",\"result_sealing\":"
            << (final_result_sealing ? "true" : "false")
            << ",\"root\":\"" << controls_root << "\"}"
            << ",\"work\":{\"core_root\":\"" << work_root(primary.work)
            << "\",\"audit_root\":\""
            << audit_work_root(primary.audit_work)
            << "\",\"certificates\":" << primary.work.certificates
            << ",\"products\":" << primary.work.products
            << ",\"solves\":" << primary.work.solves
            << ",\"scalar_dots\":" << primary.work.scalar_dots
            << ",\"scalar_divisions\":" << primary.work.scalar_divisions
            << ",\"solution_updates\":" << primary.work.solution_updates
            << ",\"residual_updates\":" << primary.work.residual_updates
            << ",\"direction_updates\":" << primary.work.direction_updates
            << ",\"quadratic_dots\":"
            << primary.audit_work.quadratic_dots
            << ",\"raw_sign_comparisons\":"
            << primary.audit_work.raw_sign_comparisons << "}"
            << ",\"primary_root\":\"" << primary.root << "\""
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << result_root << "\"}"
            << '\n';
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-quadratic-step-sign: "
                  << error.what() << '\n';
        return 2;
    }
}
#endif
