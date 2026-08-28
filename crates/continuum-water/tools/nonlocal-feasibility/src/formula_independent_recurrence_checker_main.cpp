#include "formula_probe_cache.hpp"
#include "sha256.hpp"

#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <exception>
#include <iostream>
#include <limits>
#include <sstream>
#include <string>
#include <vector>

namespace {

using namespace nextengine::nonlocal::fcr;

constexpr std::size_t DIMENSION = 102U;
constexpr const char* EXPECTED_FIXTURE_ROOT =
    "7780543a21d3b32e39a1fd18e5056f61c610d69929b4c6b69640075d1e7c4553";
constexpr const char* EXPECTED_SIGN_ROOT =
    "89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094";

struct CheckerWork {
    std::size_t operator_products = 0U;
    std::size_t tangent_kernel_calls = 0U;
    std::size_t tangent_inner_dots = 0U;
    std::size_t tangent_outer_dots = 0U;
    std::size_t tangent_inner_terms = 0U;
    std::size_t tangent_outer_terms = 0U;
    std::size_t tangent_scale_products = 0U;
    std::size_t input_components = 0U;
    std::size_t output_projections = 0U;
    std::size_t output_additions = 0U;
    std::size_t factor_solves = 0U;
    std::size_t factor_terms = 0U;
    std::size_t factor_divisions = 0U;
    std::size_t rho_dots = 0U;
    std::size_t denominator_dots = 0U;
    std::size_t dot_terms = 0U;
    std::size_t scalar_divisions = 0U;
    std::size_t solution_updates = 0U;
    std::size_t residual_updates = 0U;
    std::size_t direction_updates = 0U;
    std::size_t certificate_checks = 0U;
    std::size_t certificate_dots = 0U;
    std::size_t certificate_dot_products = 0U;
    std::size_t certificate_radius_terms = 0U;
    std::size_t certificate_solution_dots = 0U;
    std::size_t certificate_sign_comparisons = 0U;
    std::size_t state_component_comparisons = 0U;
    std::size_t product_component_comparisons = 0U;
    std::size_t scalar_root_comparisons = 0U;
    std::size_t certificate_field_comparisons = 0U;
    std::size_t work_field_comparisons = 0U;
    std::size_t controls = 0U;
    std::size_t classifier_cases = 0U;
};

struct Replay {
    bool exact = false;
    FormulaProbeTwofoldWork recurrence_work;
    FormulaProbeHybridWork hybrid_work;
    std::vector<FormulaProbeTwofoldState> states;
    std::vector<FormulaProbeTwofoldProduct> products;
    std::vector<FormulaProbeK2CertificateCheck> certificate_checks;
    CheckerWork checker_work;
    std::string artifact_root;
    std::string callback_identity_root;
    std::string callback_root;
    std::string trace_root;
    std::string work_root;
};

enum class Mismatch {
    None,
    Apparatus,
    CallbackIdentity,
    CallbackRoot,
    Product,
    State,
    Scalar,
    Certificate,
    RecurrenceWork,
    HybridWork,
};

std::uint64_t bits(double value) {
    std::uint64_t result = 0U;
    static_assert(sizeof(result) == sizeof(value));
    std::memcpy(&result, &value, sizeof(result));
    return result;
}

bool same_components(
    const std::vector<double>& left, const std::vector<double>& right) {
    if (left.size() != right.size()) return false;
    for (std::size_t index = 0U; index < left.size(); ++index)
        if (bits(left[index]) != bits(right[index])) return false;
    return true;
}

std::string state_root(const FormulaProbeTwofoldState& value) {
    std::ostringstream material;
    material << value.exact << ':' << value.index << ':'
        << formula_probe_binary64_vector_root(value.solution_components) << ':'
        << formula_probe_binary64_vector_root(value.residual_components) << ':'
        << formula_probe_binary64_vector_root(value.direction_components) << ':'
        << formula_probe_binary64_vector_root(value.preconditioned_components)
        << ':' << value.certificate.root << ':' << value.operator_root << ':'
        << value.solve_root << ':' << value.rho_root << ':'
        << value.denominator_root << ':' << value.alpha_root << ':'
        << value.beta_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string checker_work_root(const CheckerWork& value) {
    std::ostringstream material;
    material << value.operator_products << ':'
        << value.tangent_kernel_calls << ':' << value.tangent_inner_dots << ':'
        << value.tangent_outer_dots << ':' << value.tangent_inner_terms << ':'
        << value.tangent_outer_terms << ':' << value.tangent_scale_products
        << ':' << value.input_components << ':' << value.output_projections
        << ':' << value.output_additions << ':' << value.factor_solves << ':'
        << value.factor_terms << ':' << value.factor_divisions << ':'
        << value.rho_dots << ':' << value.denominator_dots << ':'
        << value.dot_terms << ':' << value.scalar_divisions << ':'
        << value.solution_updates << ':' << value.residual_updates << ':'
        << value.direction_updates << ':' << value.certificate_checks << ':'
        << value.certificate_dots << ':' << value.certificate_dot_products
        << ':' << value.certificate_radius_terms << ':'
        << value.certificate_solution_dots << ':'
        << value.certificate_sign_comparisons << ':'
        << value.state_component_comparisons << ':'
        << value.product_component_comparisons << ':'
        << value.scalar_root_comparisons << ':'
        << value.certificate_field_comparisons << ':'
        << value.work_field_comparisons << ':' << value.controls << ':'
        << value.classifier_cases;
    return nextengine::nonlocal::sha256_hex(material.str());
}

void add_solve_work(Replay& replay, const FormulaProbeK2Solve& solve) {
    ++replay.recurrence_work.factor_solves;
    replay.recurrence_work.factor_terms +=
        solve.forward_terms + solve.backward_terms;
    replay.recurrence_work.factor_divisions += solve.divisions;
    ++replay.checker_work.factor_solves;
    replay.checker_work.factor_terms +=
        solve.forward_terms + solve.backward_terms;
    replay.checker_work.factor_divisions += solve.divisions;
}

void add_dot_work(Replay& replay, const FormulaProbeK2Dot& dot, bool rho) {
    if (rho) {
        ++replay.recurrence_work.rho_dots;
        ++replay.checker_work.rho_dots;
    } else {
        ++replay.recurrence_work.denominator_dots;
        ++replay.checker_work.denominator_dots;
    }
    replay.recurrence_work.dot_terms += dot.terms;
    replay.checker_work.dot_terms += dot.terms;
}

FormulaProbeTwofoldProduct replay_product(
    const FormulaProbeParentFixture& fixture, const std::string& site,
    const std::string& artifact_root,
    const std::string& callback_identity_root,
    const std::vector<double>& input, Replay& replay) {
    FormulaProbeTwofoldProduct result;
    result.site = site;
    result.dimension = fixture.dimension;
    result.entries = fixture.dimension * fixture.dimension;
    result.input_components = input;
    result.artifact_root = artifact_root;
    result.input_root = formula_probe_binary64_vector_root(input);
    bool exact = input.size() == 2U * fixture.dimension;
    std::vector<FormulaProbeBinary128> high(fixture.dimension);
    std::vector<FormulaProbeBinary128> low(fixture.dimension);
    for (std::size_t index = 0U; index < fixture.dimension && exact; ++index) {
        high[index] = static_cast<FormulaProbeBinary128>(input[2U * index]);
        low[index] = static_cast<FormulaProbeBinary128>(input[2U * index + 1U]);
        replay.hybrid_work.input_components += 2U;
        replay.checker_work.input_components += 2U;
    }
    const FormulaProbeProduct high_product = exact
        ? formula_probe_tangent_product(fixture, high)
        : FormulaProbeProduct{};
    const FormulaProbeProduct low_product = exact
        ? formula_probe_tangent_product(fixture, low)
        : FormulaProbeProduct{};
    ++replay.hybrid_work.operator_products;
    replay.hybrid_work.tangent_kernel_calls += 2U;
    replay.hybrid_work.tangent_inner_dots +=
        high_product.inner_dots + low_product.inner_dots;
    replay.hybrid_work.tangent_outer_dots +=
        high_product.outer_dots + low_product.outer_dots;
    replay.hybrid_work.tangent_inner_terms +=
        high_product.inner_terms + low_product.inner_terms;
    replay.hybrid_work.tangent_outer_terms +=
        high_product.outer_terms + low_product.outer_terms;
    replay.hybrid_work.tangent_scale_products +=
        high_product.scale_products + low_product.scale_products;
    ++replay.checker_work.operator_products;
    replay.checker_work.tangent_kernel_calls += 2U;
    replay.checker_work.tangent_inner_dots +=
        high_product.inner_dots + low_product.inner_dots;
    replay.checker_work.tangent_outer_dots +=
        high_product.outer_dots + low_product.outer_dots;
    replay.checker_work.tangent_inner_terms +=
        high_product.inner_terms + low_product.inner_terms;
    replay.checker_work.tangent_outer_terms +=
        high_product.outer_terms + low_product.outer_terms;
    replay.checker_work.tangent_scale_products +=
        high_product.scale_products + low_product.scale_products;
    exact = exact && high_product.exact && low_product.exact
        && high_product.value.size() == fixture.dimension
        && low_product.value.size() == fixture.dimension;
    result.value_components.reserve(2U * fixture.dimension);
    for (std::size_t row = 0U; row < fixture.dimension && exact; ++row) {
        const FormulaProbeK2Scalar high_projected =
            formula_probe_k2_project(high_product.value[row]);
        const FormulaProbeK2Scalar low_projected =
            formula_probe_k2_project(low_product.value[row]);
        const FormulaProbeK2Scalar combined =
            formula_probe_k2_add(high_projected, low_projected, 1.0);
        exact = high_projected.exact && low_projected.exact && combined.exact;
        result.value_components.push_back(combined.high);
        result.value_components.push_back(combined.low);
        replay.hybrid_work.output_projections += 2U;
        ++replay.hybrid_work.output_additions;
        replay.checker_work.output_projections += 2U;
        ++replay.checker_work.output_additions;
    }
    result.exact = exact
        && result.value_components.size() == 2U * fixture.dimension;
    result.value_root =
        formula_probe_binary64_vector_root(result.value_components);
    result.high_product_root = high_product.root;
    result.low_product_root = low_product.root;
    std::ostringstream execution;
    execution << result.exact << ':' << result.dimension << ':'
        << result.entries << ':' << result.artifact_root << ':'
        << formula_probe_k2_vector_root(result.input_components) << ':'
        << formula_probe_k2_vector_root(result.value_components);
    result.execution_root =
        nextengine::nonlocal::sha256_hex(execution.str());
    result.callback_root = formula_probe_hybrid_product_callback_root(
        callback_identity_root, result);
    result.root = formula_probe_twofold_product_root(result);
    ++replay.recurrence_work.operator_products;
    replay.recurrence_work.operator_entries += result.entries;
    return result;
}

FormulaProbeTwofoldState replay_state(std::size_t index,
    const std::vector<double>& solution,
    const std::vector<double>& residual,
    const std::vector<double>& direction,
    const std::vector<double>& preconditioned,
    const FormulaProbeK2CertificateCheck& certificate,
    const std::string& operator_root, const std::string& solve_root,
    const FormulaProbeK2Scalar& rho,
    const FormulaProbeK2Scalar& denominator,
    const FormulaProbeK2Scalar& alpha,
    const FormulaProbeK2Scalar& beta) {
    FormulaProbeTwofoldState result;
    result.exact = certificate.exact;
    result.index = index;
    result.solution_components = solution;
    result.residual_components = residual;
    result.direction_components = direction;
    result.preconditioned_components = preconditioned;
    result.certificate = certificate.certificate;
    result.operator_root = operator_root;
    result.solve_root = solve_root;
    result.rho_root = rho.root;
    result.denominator_root = denominator.root;
    result.alpha_root = alpha.root;
    result.beta_root = beta.root;
    result.exact = result.exact && solution.size() == 2U * DIMENSION
        && residual.size() == 2U * DIMENSION
        && direction.size() == 2U * DIMENSION
        && preconditioned.size() == 2U * DIMENSION
        && rho.exact && denominator.exact && alpha.exact && beta.exact;
    result.root = state_root(result);
    return result;
}

void add_certificate_work(
    Replay& replay, const FormulaProbeK2CertificateCheck& check) {
    ++replay.recurrence_work.certificates;
    ++replay.checker_work.certificate_checks;
    replay.checker_work.certificate_dots += check.dots;
    replay.checker_work.certificate_dot_products += check.dot_products;
    replay.checker_work.certificate_radius_terms += check.radius_terms;
    replay.checker_work.certificate_solution_dots += check.solution_dots;
    replay.checker_work.certificate_sign_comparisons +=
        check.sign_comparisons;
}

Replay independent_replay(const FormulaProbeParentFixture& fixture) {
    Replay result;
    result.artifact_root = formula_probe_hybrid_carrier_root(fixture);
    result.callback_identity_root =
        formula_probe_hybrid_callback_identity_root(fixture);
    if (!formula_probe_parent_fixture_valid(fixture)
        || fixture.root != EXPECTED_FIXTURE_ROOT
        || fixture.dimension != DIMENSION || result.artifact_root.empty()
        || result.callback_identity_root.empty())
        return result;

    std::vector<double> rhs;
    rhs.reserve(2U * fixture.dimension);
    bool arithmetic = true;
    for (FormulaProbeBinary128 source : fixture.projected_rhs) {
        const FormulaProbeK2Scalar value = formula_probe_k2_project(source);
        arithmetic = arithmetic && value.exact;
        rhs.push_back(value.high);
        rhs.push_back(value.low);
    }
    const FormulaProbeK2Scalar inverse_scale =
        formula_probe_k2_project(fixture.projected_scale);
    const FormulaProbeK2Scalar zero = formula_probe_k2_exact_double(0.0);
    const FormulaProbeK2Scalar one = formula_probe_k2_exact_double(1.0);
    arithmetic = arithmetic && inverse_scale.exact && zero.exact && one.exact;

    const FormulaProbeK2Solve start = formula_probe_k2_factor_solve(
        fixture, "start", inverse_scale, rhs);
    add_solve_work(result, start);
    const FormulaProbeTwofoldProduct kx0 = replay_product(fixture, "Kx0",
        result.artifact_root, result.callback_identity_root,
        start.solution_components, result);
    const std::vector<double> r0 = formula_probe_k2_update(
        rhs, one, kx0.value_components, -1.0);
    const FormulaProbeK2Solve z0 = formula_probe_k2_factor_solve(
        fixture, "initial_residual", inverse_scale, r0);
    add_solve_work(result, z0);
    const FormulaProbeK2Dot rho0 =
        formula_probe_k2_dot(r0, z0.solution_components);
    add_dot_work(result, rho0, true);
    const std::vector<double> x0 = start.solution_components;
    const std::vector<double> p0 = z0.solution_components;
    const FormulaProbeTwofoldProduct kp0 = replay_product(fixture, "Kp0",
        result.artifact_root, result.callback_identity_root, p0, result);
    const FormulaProbeK2Dot denominator0 =
        formula_probe_k2_dot(p0, kp0.value_components);
    add_dot_work(result, denominator0, false);
    const FormulaProbeK2Scalar alpha0 =
        formula_probe_k2_divide(rho0.value, denominator0.value);
    ++result.recurrence_work.scalar_divisions;
    ++result.checker_work.scalar_divisions;
    const std::vector<double> x1 =
        formula_probe_k2_update(x0, alpha0, p0, 1.0);
    const std::vector<double> r1 = formula_probe_k2_update(
        r0, alpha0, kp0.value_components, -1.0);
    result.recurrence_work.solution_updates += fixture.dimension;
    result.recurrence_work.residual_updates += fixture.dimension;
    result.checker_work.solution_updates += fixture.dimension;
    result.checker_work.residual_updates += fixture.dimension;
    const FormulaProbeK2Solve z1 = formula_probe_k2_factor_solve(
        fixture, "iteration1_residual", inverse_scale, r1);
    add_solve_work(result, z1);
    const FormulaProbeK2Dot rho1 =
        formula_probe_k2_dot(r1, z1.solution_components);
    add_dot_work(result, rho1, true);
    const FormulaProbeK2Scalar beta1 =
        formula_probe_k2_divide(rho1.value, rho0.value);
    ++result.recurrence_work.scalar_divisions;
    ++result.checker_work.scalar_divisions;
    const std::vector<double> p1 = formula_probe_k2_update(
        z1.solution_components, beta1, p0, 1.0);
    result.recurrence_work.direction_updates += fixture.dimension;
    result.checker_work.direction_updates += fixture.dimension;
    const FormulaProbeTwofoldProduct kp1 = replay_product(fixture, "Kp1",
        result.artifact_root, result.callback_identity_root, p1, result);
    const FormulaProbeK2Dot denominator1 =
        formula_probe_k2_dot(p1, kp1.value_components);
    add_dot_work(result, denominator1, false);
    const FormulaProbeK2Scalar alpha1 =
        formula_probe_k2_divide(rho1.value, denominator1.value);
    ++result.recurrence_work.scalar_divisions;
    ++result.checker_work.scalar_divisions;
    const std::vector<double> x2 =
        formula_probe_k2_update(x1, alpha1, p1, 1.0);
    const std::vector<double> r2 = formula_probe_k2_update(
        r1, alpha1, kp1.value_components, -1.0);
    result.recurrence_work.solution_updates += fixture.dimension;
    result.recurrence_work.residual_updates += fixture.dimension;
    result.checker_work.solution_updates += fixture.dimension;
    result.checker_work.residual_updates += fixture.dimension;

    result.products = {kx0, kp0, kp1};
    const std::array<std::vector<double>, 3U> solutions{x0, x1, x2};
    for (const std::vector<double>& solution : solutions) {
        FormulaProbeK2CertificateCheck check =
            formula_probe_k2_certificate_check(fixture, solution);
        arithmetic = arithmetic && check.exact;
        add_certificate_work(result, check);
        result.certificate_checks.push_back(std::move(check));
    }
    result.states.push_back(replay_state(0U, x0, r0, p0,
        z0.solution_components, result.certificate_checks[0U], kx0.execution_root,
        z0.root, rho0.value, zero, zero, zero));
    result.states.push_back(replay_state(1U, x1, r1, p1,
        z1.solution_components, result.certificate_checks[1U], kp0.execution_root,
        z1.root, rho1.value, denominator0.value, alpha0, beta1));
    result.states.push_back(replay_state(2U, x2, r2, p1,
        z1.solution_components, result.certificate_checks[2U], kp1.execution_root,
        z1.root, rho1.value, denominator1.value, alpha1, beta1));

    FormulaProbeHybridRecurrence callback_material;
    callback_material.callback_identity_root = result.callback_identity_root;
    callback_material.recurrence.fixture_root = fixture.root;
    callback_material.recurrence.artifact_root = result.artifact_root;
    callback_material.recurrence.products = result.products;
    result.callback_root =
        formula_probe_hybrid_callback_root(callback_material);

    arithmetic = arithmetic && start.exact && z0.exact && z1.exact
        && kx0.exact && kp0.exact && kp1.exact
        && rho0.exact && rho1.exact && denominator0.exact
        && denominator1.exact && alpha0.exact && alpha1.exact && beta1.exact
        && x0.size() == 2U * fixture.dimension
        && x1.size() == 2U * fixture.dimension
        && x2.size() == 2U * fixture.dimension
        && r0.size() == 2U * fixture.dimension
        && r1.size() == 2U * fixture.dimension
        && r2.size() == 2U * fixture.dimension;
    result.exact = arithmetic && result.states.size() == 3U
        && result.products.size() == 3U
        && result.recurrence_work.certificates == 3U
        && result.recurrence_work.operator_products == 3U
        && result.recurrence_work.operator_entries == 31212U
        && result.recurrence_work.factor_solves == 3U
        && result.recurrence_work.factor_terms == 30906U
        && result.recurrence_work.factor_divisions == 612U
        && result.recurrence_work.rho_dots == 2U
        && result.recurrence_work.denominator_dots == 2U
        && result.recurrence_work.dot_terms == 408U
        && result.recurrence_work.scalar_divisions == 3U
        && result.recurrence_work.solution_updates == 204U
        && result.recurrence_work.residual_updates == 204U
        && result.recurrence_work.direction_updates == 102U
        && result.hybrid_work.operator_products == 3U
        && result.hybrid_work.tangent_kernel_calls == 6U
        && result.hybrid_work.tangent_inner_dots == 1890U
        && result.hybrid_work.tangent_outer_dots == 612U
        && result.hybrid_work.tangent_inner_terms == 192780U
        && result.hybrid_work.tangent_outer_terms == 192780U
        && result.hybrid_work.tangent_scale_products == 612U
        && result.hybrid_work.input_components == 612U
        && result.hybrid_work.output_projections == 612U
        && result.hybrid_work.output_additions == 306U
        && result.checker_work.certificate_checks == 3U
        && result.checker_work.certificate_dots == 306U
        && result.checker_work.certificate_dot_products == 125460U
        && result.checker_work.certificate_radius_terms == 93636U
        && result.checker_work.certificate_solution_dots == 306U
        && result.checker_work.certificate_sign_comparisons == 306U;

    std::ostringstream trace;
    trace << result.exact << ':' << fixture.root << ':'
        << result.artifact_root << ':' << result.callback_identity_root << ':'
        << result.callback_root << '|';
    for (const FormulaProbeTwofoldProduct& product : result.products)
        trace << product.site << ':' << product.input_root << ':'
            << product.value_root << ':' << product.high_product_root << ':'
            << product.low_product_root << ':' << product.execution_root << ';';
    trace << '|';
    for (const FormulaProbeTwofoldState& state : result.states)
        trace << state.root << ':' << state.certificate.root << ';';
    result.trace_root = nextengine::nonlocal::sha256_hex(trace.str());
    result.work_root = checker_work_root(result.checker_work);
    return result;
}

bool same_certificate(
    const FormulaProbeCertificate& left,
    const FormulaProbeCertificate& right) {
    return left.exact == right.exact && left.passed == right.passed
        && left.positive == right.positive && left.negative == right.negative
        && left.unresolved == right.unresolved
        && bits(left.error_upper) == bits(right.error_upper)
        && left.solution_root == right.solution_root
        && left.sign_root == right.sign_root && left.root == right.root;
}

bool same_recurrence_work(
    const FormulaProbeTwofoldWork& left,
    const FormulaProbeTwofoldWork& right) {
    return left.certificates == right.certificates
        && left.operator_products == right.operator_products
        && left.operator_entries == right.operator_entries
        && left.factor_solves == right.factor_solves
        && left.factor_terms == right.factor_terms
        && left.factor_divisions == right.factor_divisions
        && left.rho_dots == right.rho_dots
        && left.denominator_dots == right.denominator_dots
        && left.dot_terms == right.dot_terms
        && left.scalar_divisions == right.scalar_divisions
        && left.solution_updates == right.solution_updates
        && left.residual_updates == right.residual_updates
        && left.direction_updates == right.direction_updates
        && left.adaptive_stops == right.adaptive_stops;
}

bool same_hybrid_work(
    const FormulaProbeHybridWork& left,
    const FormulaProbeHybridWork& right) {
    return left.operator_products == right.operator_products
        && left.tangent_kernel_calls == right.tangent_kernel_calls
        && left.tangent_inner_dots == right.tangent_inner_dots
        && left.tangent_outer_dots == right.tangent_outer_dots
        && left.tangent_inner_terms == right.tangent_inner_terms
        && left.tangent_outer_terms == right.tangent_outer_terms
        && left.tangent_scale_products == right.tangent_scale_products
        && left.input_components == right.input_components
        && left.output_projections == right.output_projections
        && left.output_additions == right.output_additions;
}

Mismatch compare_trace(const FormulaProbeHybridRecurrence& trace,
    const Replay& replay, CheckerWork* work) {
    if (!replay.exact || trace.recurrence.states.size() != 3U
        || trace.recurrence.products.size() != 3U)
        return Mismatch::Apparatus;
    if (trace.callback_identity_root != replay.callback_identity_root
        || trace.recurrence.fixture_root != EXPECTED_FIXTURE_ROOT
        || trace.recurrence.artifact_root != replay.artifact_root)
        return Mismatch::CallbackIdentity;
    for (std::size_t index = 0U; index < replay.products.size(); ++index) {
        const FormulaProbeTwofoldProduct& actual =
            trace.recurrence.products[index];
        const FormulaProbeTwofoldProduct& expected = replay.products[index];
        if (work != nullptr) work->product_component_comparisons += 408U;
        if (actual.site != expected.site
            || actual.dimension != expected.dimension
            || actual.entries != expected.entries
            || !same_components(
                actual.input_components, expected.input_components)
            || !same_components(
                actual.value_components, expected.value_components)
            || actual.high_product_root != expected.high_product_root
            || actual.low_product_root != expected.low_product_root
            || actual.execution_root != expected.execution_root
            || actual.callback_root != expected.callback_root)
            return Mismatch::Product;
    }
    if (trace.callback_root != replay.callback_root)
        return Mismatch::CallbackRoot;
    for (std::size_t index = 0U; index < replay.states.size(); ++index) {
        const FormulaProbeTwofoldState& actual = trace.recurrence.states[index];
        const FormulaProbeTwofoldState& expected = replay.states[index];
        if (work != nullptr) work->state_component_comparisons += 816U;
        if (actual.index != expected.index
            || !same_components(
                actual.solution_components, expected.solution_components)
            || !same_components(
                actual.residual_components, expected.residual_components)
            || !same_components(
                actual.direction_components, expected.direction_components)
            || !same_components(actual.preconditioned_components,
                expected.preconditioned_components))
            return Mismatch::State;
        if (work != nullptr) work->scalar_root_comparisons += 6U;
        if (actual.operator_root != expected.operator_root
            || actual.solve_root != expected.solve_root
            || actual.rho_root != expected.rho_root
            || actual.denominator_root != expected.denominator_root
            || actual.alpha_root != expected.alpha_root
            || actual.beta_root != expected.beta_root)
            return Mismatch::Scalar;
        if (work != nullptr) work->certificate_field_comparisons += 9U;
        if (!same_certificate(actual.certificate, expected.certificate))
            return Mismatch::Certificate;
    }
    if (work != nullptr) work->work_field_comparisons += 14U;
    if (!same_recurrence_work(
            trace.recurrence.work, replay.recurrence_work))
        return Mismatch::RecurrenceWork;
    if (work != nullptr) work->work_field_comparisons += 10U;
    if (!same_hybrid_work(trace.hybrid_work, replay.hybrid_work))
        return Mismatch::HybridWork;
    return Mismatch::None;
}

void reseal_outer(FormulaProbeHybridRecurrence& value) {
    std::vector<FormulaProbeCertificate> certificates;
    for (const FormulaProbeTwofoldState& state : value.recurrence.states)
        certificates.push_back(state.certificate);
    value.recurrence.certificate_root =
        formula_probe_certificate_set_root(certificates);
    value.recurrence.root =
        formula_probe_twofold_recurrence_root(value.recurrence);
    value.callback_root = formula_probe_hybrid_callback_root(value);
    value.root = formula_probe_hybrid_twofold_recurrence_root(value);
}

std::string classify(bool apparatus, bool replay, bool work,
    bool controls, bool ladder) {
    if (!apparatus) return "INDEPENDENT_CHECKER_APPARATUS_REJECTED";
    if (!replay) return "HYBRID_TRACE_CORRESPONDENCE_REJECTED";
    if (!work) return "INDEPENDENT_CHECKER_WORK_REJECTED";
    if (!controls) return "INDEPENDENT_CHECKER_CONTROLS_REJECTED";
    if (!ladder) return "HYBRID_TRACE_LADDER_REJECTED";
    return "HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE";
}

bool classifier_control() {
    for (unsigned mask = 0U; mask < 32U; ++mask) {
        const bool apparatus = (mask & 1U) != 0U;
        const bool replay = (mask & 2U) != 0U;
        const bool work = (mask & 4U) != 0U;
        const bool controls = (mask & 8U) != 0U;
        const bool ladder = (mask & 16U) != 0U;
        std::string expected;
        if (!apparatus) expected = "INDEPENDENT_CHECKER_APPARATUS_REJECTED";
        else if (!replay) expected = "HYBRID_TRACE_CORRESPONDENCE_REJECTED";
        else if (!work) expected = "INDEPENDENT_CHECKER_WORK_REJECTED";
        else if (!controls) expected = "INDEPENDENT_CHECKER_CONTROLS_REJECTED";
        else if (!ladder) expected = "HYBRID_TRACE_LADDER_REJECTED";
        else expected = "HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE";
        if (classify(apparatus, replay, work, controls, ladder) != expected)
            return false;
    }
    return true;
}

bool ladder(const Replay& replay) {
    if (replay.certificate_checks.size() != 3U) return false;
    const FormulaProbeCertificate& c0 =
        replay.certificate_checks[0U].certificate;
    const FormulaProbeCertificate& c1 =
        replay.certificate_checks[1U].certificate;
    const FormulaProbeCertificate& c2 =
        replay.certificate_checks[2U].certificate;
    return c0.exact && !c0.passed && c0.unresolved > 0U
        && c1.exact && !c1.passed && c1.unresolved > 0U
        && c2.exact && c2.passed && c2.positive == 24U
        && c2.negative == 78U && c2.unresolved == 0U
        && c2.sign_root == EXPECTED_SIGN_ROOT;
}

std::string result_root(const std::string& route,
    const FormulaProbeParentFixture& fixture,
    const FormulaProbeHybridRecurrence& trace, const Replay& replay,
    const std::string& controls_root) {
    std::ostringstream material;
    material << route << ':' << fixture.root << ':' << trace.root << ':'
        << replay.trace_root << ':' << replay.work_root << ':'
        << controls_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-independent-recurrence-checker "
                         "<parent-cache>\n";
            return 2;
        }
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const bool fixture_exact = formula_probe_parent_fixture_valid(fixture)
            && fixture.root == EXPECTED_FIXTURE_ROOT;
        const FormulaProbeHybridRecurrence untrusted =
            formula_probe_hybrid_twofold_recurrence(fixture);
        Replay replay = independent_replay(fixture);
        if (!fixture_exact || !replay.exact
            || untrusted.recurrence.states.size() != 3U
            || untrusted.recurrence.products.size() != 3U
            || replay.states.size() != 3U || replay.products.size() != 3U) {
            const std::string route =
                "INDEPENDENT_CHECKER_APPARATUS_REJECTED";
            const std::string semantic = nextengine::nonlocal::sha256_hex(
                route + ":" + fixture.root + ":" + untrusted.root);
            std::cout
                << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zk_independent_recurrence_checker.v1\""
                << ",\"status\":\"FAIL\",\"route\":\"" << route
                << "\",\"fixture_root\":\"" << fixture.root
                << "\",\"timing_admitted\":false"
                << ",\"runtime_authority\":false"
                << ",\"gpu_authority\":false"
                << ",\"production_authority\":false"
                << ",\"result_sha256\":\"" << semantic << "\"}\n";
            return 1;
        }
        CheckerWork comparison_work = replay.checker_work;
        const Mismatch baseline =
            compare_trace(untrusted, replay, &comparison_work);

        std::array<bool, 18U> controls{};
        std::size_t control = 0U;
        FormulaProbeHybridRecurrence mutation = untrusted;
        mutation.recurrence.states[2U].solution_components[0U] =
            std::nextafter(mutation.recurrence.states[2U]
                .solution_components[0U],
                std::numeric_limits<double>::infinity());
        mutation.recurrence.states[2U].root =
            state_root(mutation.recurrence.states[2U]);
        reseal_outer(mutation);
        const bool resealed_state_author_valid =
            formula_probe_hybrid_twofold_recurrence_valid(fixture, mutation);
        controls[control++] = resealed_state_author_valid
            && compare_trace(mutation, replay, nullptr) == Mismatch::State;

        mutation = untrusted;
        mutation.recurrence.states[2U].residual_components[0U] =
            std::nextafter(mutation.recurrence.states[2U]
                .residual_components[0U],
                std::numeric_limits<double>::infinity());
        controls[control++] =
            compare_trace(mutation, replay, nullptr) == Mismatch::State;
        mutation = untrusted;
        mutation.recurrence.states[1U].direction_components[0U] =
            std::nextafter(mutation.recurrence.states[1U]
                .direction_components[0U],
                std::numeric_limits<double>::infinity());
        controls[control++] =
            compare_trace(mutation, replay, nullptr) == Mismatch::State;
        mutation = untrusted;
        mutation.recurrence.states[1U].preconditioned_components[0U] =
            std::nextafter(mutation.recurrence.states[1U]
                .preconditioned_components[0U],
                std::numeric_limits<double>::infinity());
        controls[control++] =
            compare_trace(mutation, replay, nullptr) == Mismatch::State;
        mutation = untrusted;
        mutation.recurrence.states[1U].alpha_root =
            nextengine::nonlocal::sha256_hex("mutated-alpha");
        controls[control++] =
            compare_trace(mutation, replay, nullptr) == Mismatch::Scalar;
        mutation = untrusted;
        mutation.recurrence.products[0U].input_components[0U] =
            std::nextafter(mutation.recurrence.products[0U]
                .input_components[0U],
                std::numeric_limits<double>::infinity());
        controls[control++] =
            compare_trace(mutation, replay, nullptr) == Mismatch::Product;
        mutation = untrusted;
        mutation.recurrence.products[0U].value_components[0U] =
            std::nextafter(mutation.recurrence.products[0U]
                .value_components[0U],
                std::numeric_limits<double>::infinity());
        controls[control++] =
            compare_trace(mutation, replay, nullptr) == Mismatch::Product;

        mutation = untrusted;
        mutation.recurrence.states[2U].certificate.passed = false;
        reseal_outer(mutation);
        const bool resealed_certificate_author_valid =
            formula_probe_hybrid_twofold_recurrence_valid(fixture, mutation);
        controls[control++] = resealed_certificate_author_valid
            && compare_trace(mutation, replay, nullptr)
                == Mismatch::Certificate;
        mutation = untrusted;
        ++mutation.recurrence.states[2U].certificate.unresolved;
        controls[control++] = compare_trace(mutation, replay, nullptr)
            == Mismatch::Certificate;
        mutation = untrusted;
        mutation.recurrence.states[2U].certificate.sign_root =
            nextengine::nonlocal::sha256_hex("mutated-sign");
        controls[control++] = compare_trace(mutation, replay, nullptr)
            == Mismatch::Certificate;
        mutation = untrusted;
        mutation.recurrence.states[2U].certificate.root =
            nextengine::nonlocal::sha256_hex("mutated-certificate");
        controls[control++] = compare_trace(mutation, replay, nullptr)
            == Mismatch::Certificate;

        mutation = untrusted;
        ++mutation.recurrence.work.factor_solves;
        reseal_outer(mutation);
        const bool resealed_work_author_valid =
            formula_probe_hybrid_twofold_recurrence_valid(fixture, mutation);
        controls[control++] = resealed_work_author_valid
            && compare_trace(mutation, replay, nullptr)
                == Mismatch::RecurrenceWork;
        mutation = untrusted;
        ++mutation.hybrid_work.output_projections;
        controls[control++] = compare_trace(mutation, replay, nullptr)
            == Mismatch::HybridWork;
        mutation = untrusted;
        mutation.callback_identity_root =
            nextengine::nonlocal::sha256_hex("mutated-callback");
        controls[control++] = compare_trace(mutation, replay, nullptr)
            == Mismatch::CallbackIdentity;
        mutation = untrusted;
        mutation.callback_root =
            nextengine::nonlocal::sha256_hex("mutated-callback-root");
        controls[control++] = compare_trace(mutation, replay, nullptr)
            == Mismatch::CallbackRoot;
        mutation = untrusted;
        mutation.recurrence.states.pop_back();
        controls[control++] = compare_trace(mutation, replay, nullptr)
            == Mismatch::Apparatus;
        mutation = untrusted;
        mutation.recurrence.states[0U].solution_components[0U] =
            std::numeric_limits<double>::quiet_NaN();
        controls[control++] = compare_trace(mutation, replay, nullptr)
            == Mismatch::State;
        const std::string sealed_result = result_root(
            "HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE",
            fixture, untrusted, replay, "control-seal");
        const std::string mutated_result = result_root(
            "HYBRID_TRACE_CORRESPONDENCE_REJECTED",
            fixture, untrusted, replay, "control-seal");
        controls[control++] = sealed_result != mutated_result;

        bool controls_exact = control == controls.size();
        std::ostringstream controls_material;
        controls_material << controls.size() << '|';
        for (bool passed : controls) {
            controls_exact = controls_exact && passed;
            controls_material << passed << ';';
        }
        const bool classifier_exact = classifier_control();
        controls_exact = controls_exact && classifier_exact;
        controls_material << classifier_exact << ':'
            << resealed_state_author_valid << ':'
            << resealed_certificate_author_valid << ':'
            << resealed_work_author_valid;
        const std::string controls_root =
            nextengine::nonlocal::sha256_hex(controls_material.str());

        comparison_work.controls = controls.size();
        comparison_work.classifier_cases = 32U;
        replay.checker_work = comparison_work;
        replay.work_root = checker_work_root(replay.checker_work);
        const bool replay_exact = baseline == Mismatch::None;
        const bool work_exact = replay.exact
            && replay.checker_work.state_component_comparisons == 2448U
            && replay.checker_work.product_component_comparisons == 1224U
            && replay.checker_work.scalar_root_comparisons == 18U
            && replay.checker_work.certificate_field_comparisons == 27U
            && replay.checker_work.work_field_comparisons == 24U
            && replay.checker_work.controls == 18U
            && replay.checker_work.classifier_cases == 32U;
        const bool ladder_exact = ladder(replay);
        const std::string route = classify(
            fixture_exact && replay.exact, replay_exact, work_exact,
            controls_exact, ladder_exact);
        const bool exact = route
            == "HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE";
        const std::string semantic = result_root(
            route, fixture, untrusted, replay, controls_root);

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zk_independent_recurrence_checker.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"untrusted_trace_root\":\"" << untrusted.root << "\""
            << ",\"independent_trace_root\":\"" << replay.trace_root << "\""
            << ",\"callback_identity_root\":\""
            << replay.callback_identity_root << "\""
            << ",\"callback_root\":\"" << replay.callback_root << "\""
            << ",\"trace_match\":" << (replay_exact ? "true" : "false")
            << ",\"ladder\":" << (ladder_exact ? "true" : "false")
            << ",\"state2\":{\"positive\":"
            << replay.states[2U].certificate.positive
            << ",\"negative\":" << replay.states[2U].certificate.negative
            << ",\"unresolved\":"
            << replay.states[2U].certificate.unresolved
            << ",\"sign_root\":\""
            << replay.states[2U].certificate.sign_root << "\"}"
            << ",\"producer_work\":{\"operator_products\":"
            << replay.recurrence_work.operator_products
            << ",\"factor_solves\":" << replay.recurrence_work.factor_solves
            << ",\"rho_dots\":" << replay.recurrence_work.rho_dots
            << ",\"denominator_dots\":"
            << replay.recurrence_work.denominator_dots
            << ",\"scalar_divisions\":"
            << replay.recurrence_work.scalar_divisions << "}"
            << ",\"checker_work\":{\"root\":\"" << replay.work_root
            << "\",\"certificate_checks\":"
            << replay.checker_work.certificate_checks
            << ",\"state_component_comparisons\":"
            << replay.checker_work.state_component_comparisons
            << ",\"product_component_comparisons\":"
            << replay.checker_work.product_component_comparisons
            << ",\"scalar_root_comparisons\":"
            << replay.checker_work.scalar_root_comparisons
            << ",\"certificate_field_comparisons\":"
            << replay.checker_work.certificate_field_comparisons
            << ",\"work_field_comparisons\":"
            << replay.checker_work.work_field_comparisons << "}"
            << ",\"controls\":{\"count\":" << controls.size()
            << ",\"exact\":" << (controls_exact ? "true" : "false")
            << ",\"resealed_state_author_valid\":"
            << (resealed_state_author_valid ? "true" : "false")
            << ",\"resealed_certificate_author_valid\":"
            << (resealed_certificate_author_valid ? "true" : "false")
            << ",\"resealed_work_author_valid\":"
            << (resealed_work_author_valid ? "true" : "false")
            << ",\"root\":\"" << controls_root << "\"}"
            << ",\"claim_status\":\"AUTHOR_PASS_REVIEW_NOT_TESTED\""
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << semantic << "\"}\n";
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "independent recurrence checker failed: "
            << error.what() << '\n';
        return 1;
    }
}
