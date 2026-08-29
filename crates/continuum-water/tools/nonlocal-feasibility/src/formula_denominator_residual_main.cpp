#include "formula_probe_cache.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <exception>
#include <iostream>
#include <quadmath.h>
#include <sstream>
#include <string>
#include <vector>

namespace {

using nextengine::nonlocal::fcr::FormulaProbeBinary128;
using nextengine::nonlocal::fcr::FormulaProbeCertificate;
using nextengine::nonlocal::fcr::FormulaProbeParentFixture;
using nextengine::nonlocal::fcr::FormulaProbeProduct;
using nextengine::nonlocal::fcr::FormulaProbeScalar;
using nextengine::nonlocal::fcr::FormulaProbeSolve;

constexpr std::size_t DIMENSION = 102U;
constexpr const char* PARENT_RESULT_SHA256 =
    "1fec1e31c506e65761909a6af99e44ee30b52f0fbb50432f2b74cbaa555d149f";
constexpr const char* PARENT_STDOUT_SHA256 =
    "90c73a8a6396036f6855268fc892edcab6ec7bc40542934d8cf051964dd89461";
constexpr const char* TANGENT_STATE2_SOLUTION_ROOT =
    "38f8d0b2c99126ff046d2bd6471c3c0a018b50fba66f38b418a8e3eb0600baea";
constexpr const char* COMMON_STATE2_SOLUTION_ROOT =
    "f7e28b44d35ee1e0bd0581425bded4f784f507be7b5bcce3605cc8b659bf59e8";

struct Work {
    std::size_t certificates = 0U;
    std::size_t products = 0U;
    std::size_t solves = 0U;
    std::size_t factor_terms = 0U;
    std::size_t factor_divisions = 0U;
    std::size_t scalar_dots = 0U;
    std::size_t scalar_divisions = 0U;
    std::size_t solution_updates = 0U;
    std::size_t residual_updates = 0U;
    std::size_t direction_updates = 0U;
    std::size_t tangent_inner_dots = 0U;
    std::size_t tangent_outer_dots = 0U;
    std::size_t tangent_inner_terms = 0U;
    std::size_t tangent_outer_terms = 0U;
    std::size_t tangent_scale_products = 0U;
    std::size_t common_dots = 0U;
    std::size_t common_terms = 0U;
    std::size_t adaptive_stops = 0U;
};

void add_work(Work& target, const Work& source) {
    target.certificates += source.certificates;
    target.products += source.products;
    target.solves += source.solves;
    target.factor_terms += source.factor_terms;
    target.factor_divisions += source.factor_divisions;
    target.scalar_dots += source.scalar_dots;
    target.scalar_divisions += source.scalar_divisions;
    target.solution_updates += source.solution_updates;
    target.residual_updates += source.residual_updates;
    target.direction_updates += source.direction_updates;
    target.tangent_inner_dots += source.tangent_inner_dots;
    target.tangent_outer_dots += source.tangent_outer_dots;
    target.tangent_inner_terms += source.tangent_inner_terms;
    target.tangent_outer_terms += source.tangent_outer_terms;
    target.tangent_scale_products += source.tangent_scale_products;
    target.common_dots += source.common_dots;
    target.common_terms += source.common_terms;
    target.adaptive_stops += source.adaptive_stops;
}

std::string work_root(const Work& work) {
    std::ostringstream material;
    material << work.certificates << ':' << work.products << ':'
        << work.solves << ':' << work.factor_terms << ':'
        << work.factor_divisions << ':' << work.scalar_dots << ':'
        << work.scalar_divisions << ':' << work.solution_updates << ':'
        << work.residual_updates << ':' << work.direction_updates << ':'
        << work.tangent_inner_dots << ':' << work.tangent_outer_dots << ':'
        << work.tangent_inner_terms << ':' << work.tangent_outer_terms << ':'
        << work.tangent_scale_products << ':' << work.common_dots << ':'
        << work.common_terms << ':' << work.adaptive_stops;
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool work_equal(const Work& left, const Work& right) {
    return work_root(left) == work_root(right);
}

bool finite_vector(const std::vector<FormulaProbeBinary128>& values) {
    return !values.empty()
        && std::all_of(values.begin(), values.end(), [](auto value) {
            return finiteq(value) != 0;
        });
}

bool normal_or_zero(FormulaProbeBinary128 value) {
    return finiteq(value) != 0
        && (value == static_cast<FormulaProbeBinary128>(0.0)
            || fabsq(value) >= ldexpq(
                static_cast<FormulaProbeBinary128>(1.0), -16382));
}

bool normal_vector(const std::vector<FormulaProbeBinary128>& values) {
    return !values.empty()
        && std::all_of(values.begin(), values.end(), normal_or_zero);
}

bool two_term_update(FormulaProbeBinary128 left0,
    FormulaProbeBinary128 left1, FormulaProbeBinary128 right0,
    FormulaProbeBinary128 right1, FormulaProbeBinary128& value) {
    const FormulaProbeScalar update =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(
            {left0, left1}, {right0, right1});
    value = update.value;
    return update.exact && normal_or_zero(value);
}

std::string binary128_hex(FormulaProbeBinary128 value) {
    std::array<char, 160U> buffer{};
    const int written = quadmath_snprintf(
        buffer.data(), buffer.size(), "%.28Qa", value);
    if (written <= 0 || static_cast<std::size_t>(written) >= buffer.size())
        return "format_error";
    return std::string(buffer.data(), static_cast<std::size_t>(written));
}

struct ProductResult {
    bool exact = false;
    bool common = false;
    std::vector<FormulaProbeBinary128> value;
    Work work;
    std::string root;
};

ProductResult apply_product(const FormulaProbeParentFixture& fixture,
    bool common, const std::vector<FormulaProbeBinary128>& input) {
    ProductResult result;
    result.common = common;
    const FormulaProbeProduct product = common
        ? nextengine::nonlocal::fcr::formula_probe_common_product(
            fixture, input)
        : nextengine::nonlocal::fcr::formula_probe_tangent_product(
            fixture, input);
    result.exact = product.exact && finite_vector(product.value);
    result.value = product.value;
    result.work.products = 1U;
    if (common) {
        result.work.common_dots = product.inner_dots;
        result.work.common_terms = product.inner_terms;
    } else {
        result.work.tangent_inner_dots = product.inner_dots;
        result.work.tangent_outer_dots = product.outer_dots;
        result.work.tangent_inner_terms = product.inner_terms;
        result.work.tangent_outer_terms = product.outer_terms;
        result.work.tangent_scale_products = product.scale_products;
    }
    result.root = nextengine::nonlocal::sha256_hex(
        std::string(common ? "common:" : "tangent:") + product.root);
    return result;
}

void count_solve(Work& work, const FormulaProbeSolve& solve) {
    ++work.solves;
    work.factor_terms += solve.forward_terms + solve.backward_terms;
    work.factor_divisions += solve.divisions;
}

struct State {
    bool exact = false;
    std::size_t index = 0U;
    std::vector<FormulaProbeBinary128> solution;
    std::vector<FormulaProbeBinary128> residual;
    std::vector<FormulaProbeBinary128> direction;
    std::vector<FormulaProbeBinary128> preconditioned;
    std::string product_root;
    std::string solve_root;
    std::string scalar_root;
    std::string root;
};

State seal_state(std::size_t index,
    const std::vector<FormulaProbeBinary128>& solution,
    const std::vector<FormulaProbeBinary128>& residual,
    const std::vector<FormulaProbeBinary128>& direction,
    const std::vector<FormulaProbeBinary128>& preconditioned,
    const std::string& product_root, const std::string& solve_root,
    const std::string& scalar_root) {
    State result;
    result.index = index;
    result.solution = solution;
    result.residual = residual;
    result.direction = direction;
    result.preconditioned = preconditioned;
    result.product_root = product_root;
    result.solve_root = solve_root;
    result.scalar_root = scalar_root;
    result.exact = solution.size() == DIMENSION
        && residual.size() == DIMENSION && direction.size() == DIMENSION
        && preconditioned.size() == DIMENSION && normal_vector(solution)
        && normal_vector(residual) && normal_vector(direction)
        && normal_vector(preconditioned) && !product_root.empty()
        && !solve_root.empty() && !scalar_root.empty();
    std::ostringstream material;
    material << result.exact << ':' << result.index << ':'
        << nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(
            result.solution)
        << ':'
        << nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(
            result.residual)
        << ':'
        << nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(
            result.direction)
        << ':'
        << nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(
            result.preconditioned)
        << ':' << result.product_root << ':' << result.solve_root << ':'
        << result.scalar_root;
    result.root = nextengine::nonlocal::sha256_hex(material.str());
    return result;
}

enum class FaultInjection { None, NonpositiveFinalDenominator };

struct Transaction {
    bool exact = false;
    bool complete = false;
    std::size_t selector = 0U;
    bool denominator_common = false;
    bool residual_common = false;
    std::string failure;
    std::string fixture_root;
    std::string input_root;
    Work work;
    std::vector<State> states;
    std::vector<std::string> product_roots;
    std::vector<std::string> solve_roots;
    std::vector<std::string> scalar_roots;
    FormulaProbeBinary128 tangent_denominator =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 common_denominator =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 alpha1 =
        static_cast<FormulaProbeBinary128>(0.0);
    FaultInjection injection = FaultInjection::None;
    std::string root;
};

std::string transaction_root(const Transaction& transaction) {
    std::ostringstream material;
    material << transaction.exact << ':' << transaction.complete << ':'
        << transaction.selector << ':' << transaction.denominator_common
        << ':' << transaction.residual_common << ':' << transaction.failure
        << ':' << transaction.fixture_root << ':' << transaction.input_root
        << ':' << static_cast<int>(transaction.injection) << ':'
        << work_root(transaction.work) << '|';
    for (const auto& root : transaction.product_roots)
        material << "product:" << root << ';';
    for (const auto& root : transaction.solve_roots)
        material << "solve:" << root << ';';
    for (const auto& root : transaction.scalar_roots)
        material << "scalar:" << root << ';';
    for (const auto& state : transaction.states)
        material << "state:" << state.root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

Transaction run_transaction(const FormulaProbeParentFixture& fixture,
    std::size_t selector, const std::vector<FormulaProbeBinary128>& rhs,
    FaultInjection injection = FaultInjection::None) {
    Transaction result;
    result.selector = selector;
    result.denominator_common = (selector & 2U) != 0U;
    result.residual_common = (selector & 1U) != 0U;
    result.fixture_root = fixture.root;
    result.input_root =
        nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(rhs);
    result.injection = injection;
    const auto fail = [&](const std::string& stage) {
        result.failure = stage;
        result.root = transaction_root(result);
        return result;
    };
    if (selector > 3U || fixture.dimension != DIMENSION
        || rhs.size() != DIMENSION || !normal_vector(rhs))
        return fail("identity");

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
    bool update_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        update_exact = two_term_update(rhs[index], -hx0.value[index],
            static_cast<FormulaProbeBinary128>(1.0),
            static_cast<FormulaProbeBinary128>(1.0), r0[index])
            && update_exact;
    }
    if (!update_exact || !normal_vector(r0))
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
    const State state0 = seal_state(0U, start.solution, r0, p0,
        z0.solution, hx0.root, z0.root, rho0.root);
    if (!state0.exact) return fail("state0");

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
    update_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        update_exact = two_term_update(start.solution[index], alpha0,
            static_cast<FormulaProbeBinary128>(1.0), p0[index], x1[index])
            && update_exact;
        update_exact = two_term_update(r0[index], -alpha0,
            static_cast<FormulaProbeBinary128>(1.0), hp0.value[index],
            r1[index]) && update_exact;
        ++result.work.solution_updates;
        ++result.work.residual_updates;
    }
    if (!update_exact || !normal_vector(x1) || !normal_vector(r1))
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
    std::vector<FormulaProbeBinary128> p1(DIMENSION);
    update_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        update_exact = two_term_update(z1.solution[index], beta1,
            static_cast<FormulaProbeBinary128>(1.0), p0[index], p1[index])
            && update_exact;
        ++result.work.direction_updates;
    }
    if (!update_exact || !normal_vector(p1))
        return fail("iteration1_direction");
    const State state1 = seal_state(1U, x1, r1, p1, z1.solution,
        hp0.root, z1.root, rho1.root);
    if (!state1.exact) return fail("state1");

    const ProductResult tangent = apply_product(fixture, false, p1);
    add_work(result.work, tangent.work);
    result.product_roots.push_back(tangent.root);
    if (!tangent.exact) return fail("iteration1_tangent_product");
    const ProductResult common = apply_product(fixture, true, p1);
    add_work(result.work, common.work);
    result.product_roots.push_back(common.root);
    if (!common.exact) return fail("iteration1_common_product");

    FormulaProbeScalar tangent_denominator =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(p1, tangent.value);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(tangent_denominator.root);
    result.tangent_denominator = tangent_denominator.value;
    if (!tangent_denominator.exact || !tangent_denominator.positive)
        return fail("iteration1_tangent_denominator");
    FormulaProbeScalar common_denominator =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(p1, common.value);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(common_denominator.root);
    result.common_denominator = common_denominator.value;
    if (!common_denominator.exact || !common_denominator.positive)
        return fail("iteration1_common_denominator");

    FormulaProbeScalar selected_denominator = result.denominator_common
        ? common_denominator : tangent_denominator;
    if (injection == FaultInjection::NonpositiveFinalDenominator) {
        selected_denominator.value =
            static_cast<FormulaProbeBinary128>(0.0);
        selected_denominator.positive = false;
        selected_denominator.root = nextengine::nonlocal::sha256_hex(
            selected_denominator.root + ":nonpositive_fault");
        result.scalar_roots[result.denominator_common ? 4U : 3U] =
            selected_denominator.root;
    }
    if (!selected_denominator.exact || !selected_denominator.positive)
        return fail("iteration1_denominator");
    result.alpha1 = rho1.value / selected_denominator.value;
    ++result.work.scalar_divisions;
    if (!normal_or_zero(result.alpha1)) return fail("iteration1_alpha");

    const ProductResult& residual_product =
        result.residual_common ? common : tangent;
    std::vector<FormulaProbeBinary128> x2(DIMENSION);
    std::vector<FormulaProbeBinary128> r2(DIMENSION);
    update_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        update_exact = two_term_update(x1[index], result.alpha1,
            static_cast<FormulaProbeBinary128>(1.0), p1[index], x2[index])
            && update_exact;
        update_exact = two_term_update(r1[index], -result.alpha1,
            static_cast<FormulaProbeBinary128>(1.0),
            residual_product.value[index], r2[index]) && update_exact;
        ++result.work.solution_updates;
        ++result.work.residual_updates;
    }
    if (!update_exact || !normal_vector(x2) || !normal_vector(r2))
        return fail("iteration1_update");
    const State state2 = seal_state(2U, x2, r2, p1, z1.solution,
        residual_product.root, z1.root, selected_denominator.root);
    if (!state2.exact) return fail("state2");

    result.states = {state0, state1, state2};
    result.complete = true;
    result.exact = result.product_roots.size() == 4U
        && result.solve_roots.size() == 3U
        && result.scalar_roots.size() == 5U
        && result.states.size() == 3U;
    result.root = transaction_root(result);
    return result;
}

Transaction run_finite_overflow_transaction() {
    Transaction result;
    result.fixture_root = "finite_overflow_control";
    const FormulaProbeBinary128 finite_max = strtoflt128(
        "1.18973149535723176508575932662800702e4932", nullptr);
    const std::vector<FormulaProbeBinary128> input{finite_max};
    const FormulaProbeProduct product =
        nextengine::nonlocal::fcr::formula_probe_control_dense_product(
            {finite_max}, input);
    result.input_root =
        nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(input);
    result.work.products = 1U;
    result.work.common_dots = product.inner_dots;
    result.work.common_terms = product.inner_terms;
    result.product_roots.push_back(nextengine::nonlocal::sha256_hex(
        std::string("control_dense:") + product.root));
    result.failure = product.exact
        ? "finite_overflow_not_rejected" : "final_product_overflow";
    result.exact = product.exact;
    result.root = transaction_root(result);
    return result;
}

struct Lane {
    bool exact = false;
    bool passed = false;
    std::size_t selector = 0U;
    Transaction transaction;
    std::vector<FormulaProbeCertificate> certificates;
    std::string root;
};

std::string lane_root(const Lane& lane) {
    std::ostringstream material;
    material << lane.exact << ':' << lane.passed << ':' << lane.selector
        << ':' << lane.transaction.root << ':';
    for (const auto& certificate : lane.certificates)
        material << certificate.root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

Lane run_lane(const FormulaProbeParentFixture& fixture, std::size_t selector) {
    Lane result;
    result.selector = selector;
    result.transaction = run_transaction(
        fixture, selector, fixture.original_rhs);
    if (result.transaction.complete) {
        for (const auto& state : result.transaction.states) {
            result.certificates.push_back(
                nextengine::nonlocal::fcr::formula_probe_certificate(
                    fixture, state.solution));
            ++result.transaction.work.certificates;
        }
        result.transaction.root = transaction_root(result.transaction);
    }
    result.passed = result.certificates.size() == 3U
        && !result.certificates[0U].passed
        && !result.certificates[1U].passed
        && result.certificates[2U].passed;
    result.exact = result.transaction.exact && result.transaction.complete
        && result.certificates.size() == 3U
        && std::all_of(result.certificates.begin(),
            result.certificates.end(), [](const auto& certificate) {
                return certificate.exact;
            });
    result.root = lane_root(result);
    return result;
}

std::string schedule_name(std::size_t selector) {
    std::string result;
    result.push_back((selector & 2U) != 0U ? 'C' : 'T');
    result.push_back((selector & 1U) != 0U ? 'C' : 'T');
    return result;
}

std::string pass_pattern(const std::array<bool, 4U>& passes) {
    std::string result;
    for (bool passed : passes) result.push_back(passed ? '1' : '0');
    return result;
}

bool selector_set_valid(const std::vector<std::size_t>& selectors) {
    if (selectors.size() != 4U) return false;
    for (std::size_t index = 0U; index < selectors.size(); ++index) {
        if (selectors[index] != index) return false;
    }
    return true;
}

std::string classify(bool apparatus, bool identity, bool work,
    bool tangent_endpoint, bool common_endpoint, const std::string& pattern,
    bool certificate_alias) {
    if (!apparatus) return "DENOMINATOR_RESIDUAL_APPARATUS_REJECTED";
    if (!identity) return "DENOMINATOR_RESIDUAL_IDENTITY_REJECTED";
    if (!work) return "DENOMINATOR_RESIDUAL_WORK_REJECTED";
    if (!tangent_endpoint) return "TANGENT_ENDPOINT_REJECTED";
    if (!common_endpoint) return "COMMON_ENDPOINT_REJECTED";
    if (pattern == "1100") {
        return certificate_alias ? "DENOMINATOR_STEP_SUFFICIENT"
            : "CERTIFICATE_ROOT_ISOLATION_REJECTED";
    }
    if (pattern == "1010")
        return "CERTIFICATE_RESIDUAL_ISOLATION_REJECTED";
    if (pattern == "1000")
        return "CERTIFICATE_INDEPENDENT_ISOLATION_REJECTED";
    if (pattern == "1110")
        return "CERTIFICATE_INTERACTION_ISOLATION_REJECTED";
    return "DENOMINATOR_RESIDUAL_PATTERN_REJECTED";
}

bool classifier_control() {
    for (std::size_t bits = 0U; bits < 16U; ++bits) {
        std::array<bool, 4U> passes{};
        for (std::size_t lane = 0U; lane < passes.size(); ++lane)
            passes[lane] = (bits & (1U << lane)) != 0U;
        const std::string pattern = pass_pattern(passes);
        const std::string expected = pattern == "1100"
            ? "DENOMINATOR_STEP_SUFFICIENT"
            : pattern == "1010"
                ? "CERTIFICATE_RESIDUAL_ISOLATION_REJECTED"
                : pattern == "1000"
                    ? "CERTIFICATE_INDEPENDENT_ISOLATION_REJECTED"
                    : pattern == "1110"
                        ? "CERTIFICATE_INTERACTION_ISOLATION_REJECTED"
                        : "DENOMINATOR_RESIDUAL_PATTERN_REJECTED";
        if (classify(true, true, true, true, true, pattern, true)
            != expected)
            return false;
    }
    return classify(false, true, true, true, true, "1100", true)
            == "DENOMINATOR_RESIDUAL_APPARATUS_REJECTED"
        && classify(true, false, true, true, true, "1100", true)
            == "DENOMINATOR_RESIDUAL_IDENTITY_REJECTED"
        && classify(true, true, false, true, true, "1100", true)
            == "DENOMINATOR_RESIDUAL_WORK_REJECTED"
        && classify(true, true, true, false, true, "1100", true)
            == "TANGENT_ENDPOINT_REJECTED"
        && classify(true, true, true, true, false, "1100", true)
            == "COMMON_ENDPOINT_REJECTED"
        && classify(true, true, true, true, true, "1100", false)
            == "CERTIFICATE_ROOT_ISOLATION_REJECTED";
}

bool certificate_matches(const FormulaProbeCertificate& certificate,
    bool passed, std::size_t positive, std::size_t negative,
    std::size_t unresolved) {
    return certificate.exact && certificate.passed == passed
        && certificate.positive == positive
        && certificate.negative == negative
        && certificate.unresolved == unresolved;
}

std::string lane_set_root(const std::array<Lane, 4U>& lanes) {
    std::ostringstream material;
    for (const auto& lane : lanes)
        material << lane.selector << ':' << lane.root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool same_vector_root(const std::vector<FormulaProbeBinary128>& left,
    const std::vector<FormulaProbeBinary128>& right) {
    using nextengine::nonlocal::fcr::formula_probe_binary128_vector_root;
    return formula_probe_binary128_vector_root(left)
        == formula_probe_binary128_vector_root(right);
}

std::string root_or_missing(
    const std::vector<std::string>& roots, std::size_t index) {
    return index < roots.size() ? roots[index] : "missing";
}

struct LaneAudit {
    bool apparatus = false;
    bool tangent_endpoint = false;
    bool common_endpoint = false;
    bool prefix_alias = false;
    bool solution_alias = false;
    bool residual_distinct = false;
    bool state_distinct = false;
    bool final_product_alias = false;
    bool certificate_alias = false;
};

LaneAudit audit_lanes(const std::array<Lane, 4U>& lanes) {
    using nextengine::nonlocal::fcr::formula_probe_binary128_vector_root;
    LaneAudit result;
    result.apparatus = true;
    for (std::size_t selector = 0U; selector < lanes.size(); ++selector) {
        const Lane& lane = lanes[selector];
        result.apparatus = result.apparatus && lane.exact
            && lane.selector == selector && lane.transaction.exact
            && lane.transaction.complete
            && lane.transaction.selector == selector
            && lane.transaction.denominator_common
                == ((selector & 2U) != 0U)
            && lane.transaction.residual_common == ((selector & 1U) != 0U)
            && lane.transaction.states.size() == 3U
            && lane.transaction.product_roots.size() == 4U
            && lane.transaction.solve_roots.size() == 3U
            && lane.transaction.scalar_roots.size() == 5U
            && lane.certificates.size() == 3U;
    }
    if (!result.apparatus) return result;

    result.tangent_endpoint = lanes[0U].passed
        && formula_probe_binary128_vector_root(
            lanes[0U].transaction.states[2U].solution)
            == TANGENT_STATE2_SOLUTION_ROOT
        && certificate_matches(
            lanes[0U].certificates[0U], false, 12U, 24U, 66U)
        && certificate_matches(
            lanes[0U].certificates[1U], false, 12U, 24U, 66U)
        && certificate_matches(
            lanes[0U].certificates[2U], true, 24U, 78U, 0U);
    result.common_endpoint = !lanes[3U].passed
        && formula_probe_binary128_vector_root(
            lanes[3U].transaction.states[2U].solution)
            == COMMON_STATE2_SOLUTION_ROOT
        && certificate_matches(
            lanes[3U].certificates[0U], false, 12U, 24U, 66U)
        && certificate_matches(
            lanes[3U].certificates[1U], false, 12U, 24U, 66U)
        && certificate_matches(
            lanes[3U].certificates[2U], false, 12U, 24U, 66U);

    result.prefix_alias = true;
    result.final_product_alias = true;
    for (std::size_t lane = 1U; lane < lanes.size(); ++lane) {
        for (std::size_t state = 0U; state < 2U; ++state) {
            result.prefix_alias = result.prefix_alias
                && lanes[lane].transaction.states[state].root
                    == lanes[0U].transaction.states[state].root
                && lanes[lane].certificates[state].root
                    == lanes[0U].certificates[state].root;
        }
        result.final_product_alias = result.final_product_alias
            && lanes[lane].transaction.product_roots[2U]
                == lanes[0U].transaction.product_roots[2U]
            && lanes[lane].transaction.product_roots[3U]
                == lanes[0U].transaction.product_roots[3U]
            && lanes[lane].transaction.scalar_roots[3U]
                == lanes[0U].transaction.scalar_roots[3U]
            && lanes[lane].transaction.scalar_roots[4U]
                == lanes[0U].transaction.scalar_roots[4U];
    }
    result.solution_alias = same_vector_root(
            lanes[0U].transaction.states[2U].solution,
            lanes[1U].transaction.states[2U].solution)
        && same_vector_root(lanes[2U].transaction.states[2U].solution,
            lanes[3U].transaction.states[2U].solution);
    result.residual_distinct = !same_vector_root(
            lanes[0U].transaction.states[2U].residual,
            lanes[1U].transaction.states[2U].residual)
        && !same_vector_root(lanes[2U].transaction.states[2U].residual,
            lanes[3U].transaction.states[2U].residual);
    result.state_distinct = lanes[0U].transaction.states[2U].root
            != lanes[1U].transaction.states[2U].root
        && lanes[2U].transaction.states[2U].root
            != lanes[3U].transaction.states[2U].root;
    result.certificate_alias = lanes[0U].certificates[2U].root
            == lanes[1U].certificates[2U].root
        && lanes[2U].certificates[2U].root
            == lanes[3U].certificates[2U].root;
    return result;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-denominator-residual "
                         "<parent-fixture-cache>\n";
            return 2;
        }
        using namespace nextengine::nonlocal::fcr;
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const bool identity = formula_probe_parent_fixture_valid(fixture);

        std::array<Lane, 4U> lanes;
        std::array<bool, 4U> passes{};
        Work aggregate;
        for (std::size_t selector = 0U; selector < lanes.size(); ++selector) {
            lanes[selector] = run_lane(fixture, selector);
            passes[selector] = lanes[selector].passed;
            add_work(aggregate, lanes[selector].transaction.work);
        }

        const Work lane_expected{3U, 4U, 3U, 30906U, 612U, 5U, 3U,
            204U, 204U, 102U, 945U, 306U, 96390U, 96390U, 306U,
            102U, 10404U, 0U};
        const Work aggregate_expected{12U, 16U, 12U, 123624U, 2448U,
            20U, 12U, 816U, 816U, 408U, 3780U, 1224U, 385560U,
            385560U, 1224U, 408U, 41616U, 0U};
        bool work = work_equal(aggregate, aggregate_expected);
        for (const auto& lane : lanes)
            work = work && work_equal(lane.transaction.work, lane_expected);

        const LaneAudit audit = audit_lanes(lanes);
        const bool solution_residual_aliases = audit.prefix_alias
            && audit.solution_alias && audit.residual_distinct
            && audit.state_distinct && audit.final_product_alias;

        const bool classifier = classifier_control();
        const bool selectors = selector_set_valid({0U, 1U, 2U, 3U})
            && !selector_set_valid({0U, 1U, 1U, 3U})
            && !selector_set_valid({0U, 1U, 2U})
            && !selector_set_valid({0U, 1U, 2U, 4U})
            && !selector_set_valid({1U, 0U, 2U, 3U});

        const std::string original_lane_set_root = lane_set_root(lanes);
        bool identity_binding = false;
        bool certificate_isolation = false;
        if (audit.apparatus) {
            std::array<Lane, 4U> product_mutation = lanes;
            product_mutation[0U].transaction.product_roots[3U] =
                nextengine::nonlocal::sha256_hex(
                    product_mutation[0U].transaction.product_roots[3U]
                        + ":mutation");
            product_mutation[0U].transaction.root =
                transaction_root(product_mutation[0U].transaction);
            product_mutation[0U].root = lane_root(product_mutation[0U]);
            std::array<Lane, 4U> denominator_mutation = lanes;
            denominator_mutation[0U].transaction.scalar_roots[4U] =
                nextengine::nonlocal::sha256_hex(
                    denominator_mutation[0U].transaction.scalar_roots[4U]
                        + ":mutation");
            denominator_mutation[0U].transaction.root =
                transaction_root(denominator_mutation[0U].transaction);
            denominator_mutation[0U].root = lane_root(
                denominator_mutation[0U]);
            identity_binding =
                product_mutation[0U].transaction.root
                    != lanes[0U].transaction.root
                && denominator_mutation[0U].transaction.root
                    != lanes[0U].transaction.root
                && lane_set_root(product_mutation) != original_lane_set_root
                && lane_set_root(denominator_mutation)
                    != original_lane_set_root
                && same_vector_root(
                    product_mutation[0U].transaction.states[2U].solution,
                    lanes[0U].transaction.states[2U].solution)
                && same_vector_root(
                    denominator_mutation[0U].transaction.states[2U].solution,
                    lanes[0U].transaction.states[2U].solution);

            std::array<Lane, 4U> certificate_mutation = lanes;
            certificate_mutation[0U].certificates[2U].root =
                nextengine::nonlocal::sha256_hex(
                    certificate_mutation[0U].certificates[2U].root
                        + ":mutation");
            certificate_mutation[0U].root = lane_root(
                certificate_mutation[0U]);
            certificate_isolation =
                certificate_mutation[0U].transaction.root
                    == lanes[0U].transaction.root
                && certificate_mutation[0U].root != lanes[0U].root
                && lane_set_root(certificate_mutation)
                    != original_lane_set_root;
        }

        std::array<Lane, 4U> failing_lanes = lanes;
        failing_lanes[1U].exact = false;
        failing_lanes[1U].transaction.exact = false;
        failing_lanes[1U].transaction.complete = false;
        failing_lanes[1U].transaction.failure = "injected_incomplete_lane";
        failing_lanes[1U].transaction.states.clear();
        failing_lanes[1U].transaction.product_roots.clear();
        failing_lanes[1U].transaction.solve_roots.clear();
        failing_lanes[1U].transaction.scalar_roots.clear();
        failing_lanes[1U].certificates.clear();
        failing_lanes[1U].transaction.root =
            transaction_root(failing_lanes[1U].transaction);
        failing_lanes[1U].root = lane_root(failing_lanes[1U]);
        const LaneAudit failing_audit = audit_lanes(failing_lanes);
        const std::string failing_route = classify(failing_audit.apparatus,
            true, true, true, true, "1100", true);
        const bool failing_lane_control = !failing_audit.apparatus
            && failing_route == "DENOMINATOR_RESIDUAL_APPARATUS_REJECTED"
            && failing_lanes[1U].transaction.states.empty()
            && failing_lanes[1U].certificates.empty();

        const Transaction invalid_selector = run_transaction(
            fixture, 4U, fixture.original_rhs);
        const Work no_work;
        const bool invalid_control = !invalid_selector.exact
            && !invalid_selector.complete
            && invalid_selector.failure == "identity"
            && invalid_selector.states.empty()
            && invalid_selector.product_roots.empty()
            && invalid_selector.solve_roots.empty()
            && invalid_selector.scalar_roots.empty()
            && work_equal(invalid_selector.work, no_work);

        const Transaction overflow = run_finite_overflow_transaction();
        Work overflow_work;
        overflow_work.products = 1U;
        overflow_work.common_dots = 1U;
        overflow_work.common_terms = 1U;
        const bool overflow_control = !overflow.exact && !overflow.complete
            && overflow.failure == "final_product_overflow"
            && overflow.states.empty()
            && work_equal(overflow.work, overflow_work);

        const Transaction nonpositive = run_transaction(fixture, 0U,
            fixture.original_rhs,
            FaultInjection::NonpositiveFinalDenominator);
        const Work nonpositive_work{0U, 4U, 3U, 30906U, 612U, 5U, 2U,
            102U, 102U, 102U, 945U, 306U, 96390U, 96390U, 306U,
            102U, 10404U, 0U};
        const bool nonpositive_control = !nonpositive.exact
            && !nonpositive.complete
            && nonpositive.failure == "iteration1_denominator"
            && nonpositive.states.empty()
            && work_equal(nonpositive.work, nonpositive_work);

        const bool controls = classifier && selectors && identity_binding
            && certificate_isolation && failing_lane_control
            && invalid_control && overflow_control && nonpositive_control;
        std::ostringstream control_material;
        control_material << classifier << ':' << selectors << ':'
            << identity_binding << ':' << certificate_isolation << ':'
            << failing_lane_control << ':' << failing_route << ':'
            << failing_lanes[1U].root << ':'
            << invalid_control << ':' << invalid_selector.root << ':'
            << overflow_control << ':' << overflow.root << ':'
            << nonpositive_control << ':' << nonpositive.root << ':'
            << original_lane_set_root;
        const std::string controls_root =
            nextengine::nonlocal::sha256_hex(control_material.str());

        const std::string pattern = pass_pattern(passes);
        const std::string route = classify(
            audit.apparatus && controls && solution_residual_aliases,
            identity, work, audit.tangent_endpoint, audit.common_endpoint,
            pattern, audit.certificate_alias);
        const bool exact = route == "DENOMINATOR_STEP_SUFFICIENT"
            && audit.apparatus && identity && work && audit.tangent_endpoint
            && audit.common_endpoint && solution_residual_aliases
            && audit.certificate_alias && controls;

        std::ostringstream semantic;
        semantic << route << ':' << fixture.root << ':'
            << PARENT_RESULT_SHA256 << ':' << PARENT_STDOUT_SHA256 << ':'
            << work_root(aggregate) << ':' << controls_root << ':'
            << pattern << ':' << audit.prefix_alias << ':'
            << audit.solution_alias << ':' << audit.residual_distinct << ':'
            << audit.state_distinct << ':' << audit.final_product_alias << ':'
            << audit.certificate_alias << ':'
            << original_lane_set_root;
        const std::string result_root =
            nextengine::nonlocal::sha256_hex(semantic.str());

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.r63ze_denominator_residual.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"parent_result_sha256\":\"" << PARENT_RESULT_SHA256
            << "\",\"parent_stdout_sha256\":\"" << PARENT_STDOUT_SHA256
            << "\",\"work_root\":\"" << work_root(aggregate) << "\""
            << ",\"pass_pattern\":\"" << pattern << "\""
            << ",\"lanes\":[";
        for (std::size_t selector = 0U; selector < lanes.size(); ++selector) {
            if (selector != 0U) std::cout << ',';
            const Lane& lane = lanes[selector];
            std::cout << "{\"selector\":" << selector
                << ",\"schedule\":\"" << schedule_name(selector) << "\""
                << ",\"denominator\":\""
                << (lane.transaction.denominator_common ? "common" : "tangent")
                << "\",\"residual\":\""
                << (lane.transaction.residual_common ? "common" : "tangent")
                << "\",\"exact\":" << (lane.exact ? "true" : "false")
                << ",\"passed\":" << (lane.passed ? "true" : "false")
                << ",\"transaction_root\":\""
                << lane.transaction.root << "\""
                << ",\"final_tangent_product_root\":\""
                << root_or_missing(lane.transaction.product_roots, 2U) << "\""
                << ",\"final_common_product_root\":\""
                << root_or_missing(lane.transaction.product_roots, 3U) << "\""
                << ",\"tangent_denominator_root\":\""
                << root_or_missing(lane.transaction.scalar_roots, 3U) << "\""
                << ",\"common_denominator_root\":\""
                << root_or_missing(lane.transaction.scalar_roots, 4U) << "\""
                << ",\"tangent_denominator_hex\":\""
                << binary128_hex(lane.transaction.tangent_denominator) << "\""
                << ",\"common_denominator_hex\":\""
                << binary128_hex(lane.transaction.common_denominator) << "\""
                << ",\"alpha1_hex\":\""
                << binary128_hex(lane.transaction.alpha1) << "\""
                << ",\"states\":[";
            const std::size_t state_count = std::min(
                lane.transaction.states.size(), lane.certificates.size());
            for (std::size_t state = 0U; state < state_count; ++state) {
                if (state != 0U) std::cout << ',';
                const State& value = lane.transaction.states[state];
                const FormulaProbeCertificate& certificate =
                    lane.certificates[state];
                std::cout << "{\"state\":" << state
                    << ",\"solution_root\":\""
                    << formula_probe_binary128_vector_root(value.solution)
                    << "\",\"residual_root\":\""
                    << formula_probe_binary128_vector_root(value.residual)
                    << "\",\"direction_root\":\""
                    << formula_probe_binary128_vector_root(value.direction)
                    << "\",\"state_root\":\"" << value.root << "\""
                    << ",\"passed\":"
                    << (certificate.passed ? "true" : "false")
                    << ",\"positive\":" << certificate.positive
                    << ",\"negative\":" << certificate.negative
                    << ",\"unresolved\":" << certificate.unresolved
                    << ",\"certificate_root\":\""
                    << certificate.root << "\"}";
            }
            std::cout << "],\"root\":\"" << lane.root << "\"}";
        }
        std::cout << "],\"aliases\":{\"prefix\":"
            << (audit.prefix_alias ? "true" : "false")
            << ",\"solution\":"
            << (audit.solution_alias ? "true" : "false")
            << ",\"residual_distinct\":"
            << (audit.residual_distinct ? "true" : "false")
            << ",\"state_distinct\":"
            << (audit.state_distinct ? "true" : "false")
            << ",\"final_products_and_denominators\":"
            << (audit.final_product_alias ? "true" : "false")
            << ",\"certificate\":"
            << (audit.certificate_alias ? "true" : "false") << "}"
            << ",\"controls\":{\"classifier\":"
            << (classifier ? "true" : "false")
            << ",\"selectors\":" << (selectors ? "true" : "false")
            << ",\"identity_binding\":"
            << (identity_binding ? "true" : "false")
            << ",\"certificate_isolation\":"
            << (certificate_isolation ? "true" : "false")
            << ",\"failing_lane\":{\"passed\":"
            << (failing_lane_control ? "true" : "false")
            << ",\"route\":\"" << failing_route
            << "\",\"root\":\"" << failing_lanes[1U].root << "\"}"
            << ",\"invalid_selector\":{\"passed\":"
            << (invalid_control ? "true" : "false")
            << ",\"failure\":\"" << invalid_selector.failure
            << "\",\"work_root\":\"" << work_root(invalid_selector.work)
            << "\",\"root\":\"" << invalid_selector.root << "\"}"
            << ",\"overflow\":{\"passed\":"
            << (overflow_control ? "true" : "false")
            << ",\"failure\":\"" << overflow.failure
            << "\",\"work_root\":\"" << work_root(overflow.work)
            << "\",\"root\":\"" << overflow.root << "\"}"
            << ",\"nonpositive_denominator\":{\"passed\":"
            << (nonpositive_control ? "true" : "false")
            << ",\"failure\":\"" << nonpositive.failure
            << "\",\"work_root\":\"" << work_root(nonpositive.work)
            << "\",\"root\":\"" << nonpositive.root << "\"}"
            << ",\"root\":\"" << controls_root << "\"}"
            << ",\"work\":{\"certificates\":" << aggregate.certificates
            << ",\"products\":" << aggregate.products
            << ",\"solves\":" << aggregate.solves
            << ",\"factor_terms\":" << aggregate.factor_terms
            << ",\"factor_divisions\":" << aggregate.factor_divisions
            << ",\"scalar_dots\":" << aggregate.scalar_dots
            << ",\"scalar_divisions\":" << aggregate.scalar_divisions
            << ",\"solution_updates\":" << aggregate.solution_updates
            << ",\"residual_updates\":" << aggregate.residual_updates
            << ",\"direction_updates\":" << aggregate.direction_updates
            << ",\"tangent_inner_dots\":"
            << aggregate.tangent_inner_dots
            << ",\"tangent_outer_dots\":"
            << aggregate.tangent_outer_dots
            << ",\"tangent_inner_terms\":"
            << aggregate.tangent_inner_terms
            << ",\"tangent_outer_terms\":"
            << aggregate.tangent_outer_terms
            << ",\"tangent_scale_products\":"
            << aggregate.tangent_scale_products
            << ",\"common_dots\":" << aggregate.common_dots
            << ",\"common_terms\":" << aggregate.common_terms
            << ",\"adaptive_stops\":" << aggregate.adaptive_stops << "}"
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << result_root << "\"}"
            << '\n';
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-denominator-residual: "
                  << error.what() << '\n';
        return 2;
    }
}
