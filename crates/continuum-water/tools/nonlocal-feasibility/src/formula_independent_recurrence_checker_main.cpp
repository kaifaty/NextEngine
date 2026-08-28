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
constexpr std::size_t EXPECTED_CONTROL_COUNT = 64U;
constexpr std::size_t EXPECTED_CLASSIFIER_CASES = 96U;
constexpr const char* EXPECTED_FIXTURE_ROOT =
    "7780543a21d3b32e39a1fd18e5056f61c610d69929b4c6b69640075d1e7c4553";
constexpr const char* EXPECTED_SIGN_ROOT =
    "89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094";

struct ComparisonWork {
    std::size_t trace_calls = 0U;
    std::size_t apparatus_comparisons = 0U;
    std::size_t identity_comparisons = 0U;
    std::size_t product_guard_comparisons = 0U;
    std::size_t product_metadata_comparisons = 0U;
    std::size_t product_component_comparisons = 0U;
    std::size_t product_root_comparisons = 0U;
    std::size_t callback_comparisons = 0U;
    std::size_t recurrence_guard_comparisons = 0U;
    std::size_t state_guard_comparisons = 0U;
    std::size_t state_component_comparisons = 0U;
    std::size_t scalar_guard_comparisons = 0U;
    std::size_t scalar_component_comparisons = 0U;
    std::size_t scalar_root_comparisons = 0U;
    std::size_t certificate_field_comparisons = 0U;
    std::size_t state_root_comparisons = 0U;
    std::size_t recurrence_work_field_comparisons = 0U;
    std::size_t recurrence_root_comparisons = 0U;
    std::size_t hybrid_work_field_comparisons = 0U;
    std::size_t hybrid_root_comparisons = 0U;
    std::size_t producer_root_derivations = 0U;
};

struct CheckerWork {
    std::size_t fixture_validations = 0U;
    std::size_t identity_derivations = 0U;
    std::size_t projected_scalars = 0U;
    std::size_t exact_scalars = 0U;
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
    std::size_t factor_payload_validations = 0U;
    std::size_t factor_solves = 0U;
    std::size_t factor_terms = 0U;
    std::size_t factor_divisions = 0U;
    std::size_t rho_dots = 0U;
    std::size_t denominator_dots = 0U;
    std::size_t dot_terms = 0U;
    std::size_t positivity_checks = 0U;
    std::size_t scalar_divisions = 0U;
    std::size_t solution_updates = 0U;
    std::size_t residual_updates = 0U;
    std::size_t direction_updates = 0U;
    std::size_t profile_payload_validations = 0U;
    std::size_t certificate_checks = 0U;
    std::size_t certificate_dots = 0U;
    std::size_t certificate_dot_products = 0U;
    std::size_t certificate_radius_terms = 0U;
    std::size_t certificate_solution_dots = 0U;
    std::size_t certificate_sign_comparisons = 0U;
    std::size_t root_derivation_paths = 0U;
    std::size_t result_seals = 0U;
    ComparisonWork comparison;
};

struct ControlWork {
    std::size_t controls = 0U;
    std::size_t reseal_paths = 0U;
    std::size_t root_derivation_paths = 0U;
    std::size_t author_validations = 0U;
    std::size_t author_fixture_validations = 0U;
    std::size_t author_callback_replays = 0U;
    std::size_t author_tangent_kernel_calls = 0U;
    std::size_t author_tangent_inner_dots = 0U;
    std::size_t author_tangent_outer_dots = 0U;
    std::size_t author_tangent_inner_terms = 0U;
    std::size_t author_tangent_outer_terms = 0U;
    std::size_t author_tangent_scale_products = 0U;
    std::size_t author_input_components = 0U;
    std::size_t author_output_projections = 0U;
    std::size_t author_output_additions = 0U;
    std::size_t classifier_cases = 0U;
    std::size_t result_seal_checks = 0U;
    ComparisonWork comparison;
};

struct Replay {
    bool exact = false;
    std::string failure_stage;
    FormulaProbeHybridIdentity identity;
    FormulaProbeHybridRecurrence expected;
    std::vector<FormulaProbeK2CertificateCheck> certificate_checks;
    CheckerWork checker_work;
    std::string semantic_trace_root;
    std::string expected_recurrence_work_root;
    std::string expected_hybrid_work_root;
    std::string expected_producer_payload_root;
    std::string expected_producer_receipt_root;
    std::string checker_work_root;
};

enum class Mismatch {
    None,
    Apparatus,
    CallbackIdentity,
    Product,
    CallbackRoot,
    RecurrenceGuard,
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

bool compare_field(bool equal, std::size_t* counter) {
    if (counter != nullptr) ++*counter;
    return equal;
}

bool compare_components(const std::vector<double>& left,
    const std::vector<double>& right, std::size_t* counter) {
    if (left.size() != right.size()) return false;
    bool equal = true;
    for (std::size_t index = 0U; index < left.size(); ++index) {
        if (counter != nullptr) ++*counter;
        equal &= bits(left[index]) == bits(right[index]);
    }
    return equal;
}

void append_scalar(
    std::ostringstream& material, const FormulaProbeK2Scalar& value) {
    material << value.exact << ':' << bits(value.high) << ':'
        << bits(value.low) << ':' << value.root;
}

std::string legacy_state_root(const FormulaProbeTwofoldState& value) {
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

std::string transaction_state_root(const FormulaProbeTwofoldState& value) {
    std::ostringstream material;
    material << value.index << ':'
        << formula_probe_k2_vector_root(value.solution_components) << ':'
        << formula_probe_k2_vector_root(value.residual_components) << ':'
        << formula_probe_k2_vector_root(value.direction_components) << ':'
        << formula_probe_k2_vector_root(value.preconditioned_components)
        << ':' << value.operator_root << ':' << value.solve_root << ':'
        << value.rho.root << ':' << value.denominator.root << ':'
        << value.alpha.root << ':' << value.beta.root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string recurrence_work_root(const FormulaProbeTwofoldWork& value) {
    std::ostringstream material;
    material << value.certificates << ':' << value.operator_products << ':'
        << value.operator_entries << ':' << value.factor_solves << ':'
        << value.factor_terms << ':' << value.factor_divisions << ':'
        << value.rho_dots << ':' << value.denominator_dots << ':'
        << value.dot_terms << ':' << value.scalar_divisions << ':'
        << value.solution_updates << ':' << value.residual_updates << ':'
        << value.direction_updates << ':' << value.adaptive_stops;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string hybrid_work_root(const FormulaProbeHybridWork& value) {
    std::ostringstream material;
    material << value.fixture_validations << ':'
        << value.identity_derivations << ':' << value.operator_products << ':'
        << value.tangent_kernel_calls << ':' << value.tangent_inner_dots << ':'
        << value.tangent_outer_dots << ':' << value.tangent_inner_terms << ':'
        << value.tangent_outer_terms << ':' << value.tangent_scale_products
        << ':' << value.input_components << ':' << value.output_projections
        << ':' << value.output_additions << ':' << value.certificate_dots << ':'
        << value.certificate_dot_products << ':'
        << value.certificate_radius_terms << ':'
        << value.certificate_solution_dots << ':'
        << value.certificate_sign_comparisons;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string producer_payload_root(const FormulaProbeTwofoldWork& recurrence,
    const FormulaProbeHybridWork& hybrid) {
    return nextengine::nonlocal::sha256_hex(
        recurrence_work_root(recurrence) + ":" + hybrid_work_root(hybrid));
}

std::string producer_receipt_root(
    const std::string& role, const std::string& payload_root) {
    return nextengine::nonlocal::sha256_hex(
        "r63zk-producer-receipt-v2:" + role + ":" + payload_root);
}

std::string comparison_work_root(const ComparisonWork& value) {
    std::ostringstream material;
    material << value.trace_calls << ':' << value.apparatus_comparisons << ':'
        << value.identity_comparisons << ':'
        << value.product_guard_comparisons << ':'
        << value.product_metadata_comparisons << ':'
        << value.product_component_comparisons << ':'
        << value.product_root_comparisons << ':'
        << value.callback_comparisons << ':'
        << value.recurrence_guard_comparisons << ':'
        << value.state_guard_comparisons << ':'
        << value.state_component_comparisons << ':'
        << value.scalar_guard_comparisons << ':'
        << value.scalar_component_comparisons << ':'
        << value.scalar_root_comparisons << ':'
        << value.certificate_field_comparisons << ':'
        << value.state_root_comparisons << ':'
        << value.recurrence_work_field_comparisons << ':'
        << value.recurrence_root_comparisons << ':'
        << value.hybrid_work_field_comparisons << ':'
        << value.hybrid_root_comparisons << ':'
        << value.producer_root_derivations;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string checker_work_root(
    const CheckerWork& value, const std::string& comparison_root) {
    std::ostringstream material;
    material << value.fixture_validations << ':'
        << value.identity_derivations << ':' << value.projected_scalars << ':'
        << value.exact_scalars << ':' << value.operator_products << ':'
        << value.tangent_kernel_calls << ':' << value.tangent_inner_dots << ':'
        << value.tangent_outer_dots << ':' << value.tangent_inner_terms << ':'
        << value.tangent_outer_terms << ':' << value.tangent_scale_products
        << ':' << value.input_components << ':' << value.output_projections
        << ':' << value.output_additions << ':'
        << value.factor_payload_validations << ':' << value.factor_solves << ':'
        << value.factor_terms << ':' << value.factor_divisions << ':'
        << value.rho_dots << ':' << value.denominator_dots << ':'
        << value.dot_terms << ':' << value.positivity_checks << ':'
        << value.scalar_divisions << ':' << value.solution_updates << ':'
        << value.residual_updates << ':' << value.direction_updates << ':'
        << value.profile_payload_validations << ':'
        << value.certificate_checks << ':' << value.certificate_dots << ':'
        << value.certificate_dot_products << ':'
        << value.certificate_radius_terms << ':'
        << value.certificate_solution_dots << ':'
        << value.certificate_sign_comparisons << ':'
        << value.root_derivation_paths << ':' << value.result_seals << ':'
        << comparison_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string control_work_root(
    const ControlWork& value, const std::vector<bool>& controls,
    const std::string& comparison_root) {
    std::ostringstream material;
    material << value.controls << ':' << value.reseal_paths << ':'
        << value.root_derivation_paths << ':' << value.author_validations << ':'
        << value.author_fixture_validations << ':'
        << value.author_callback_replays << ':'
        << value.author_tangent_kernel_calls << ':'
        << value.author_tangent_inner_dots << ':'
        << value.author_tangent_outer_dots << ':'
        << value.author_tangent_inner_terms << ':'
        << value.author_tangent_outer_terms << ':'
        << value.author_tangent_scale_products << ':'
        << value.author_input_components << ':'
        << value.author_output_projections << ':'
        << value.author_output_additions << ':' << value.classifier_cases << ':'
        << value.result_seal_checks << ':' << comparison_root << '|';
    for (bool control : controls) material << control << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

void add_solve_work(Replay& replay, const FormulaProbeK2Solve& solve) {
    ++replay.expected.recurrence.work.factor_solves;
    replay.expected.recurrence.work.factor_terms +=
        solve.forward_terms + solve.backward_terms;
    replay.expected.recurrence.work.factor_divisions += solve.divisions;
    ++replay.checker_work.factor_solves;
    replay.checker_work.factor_terms +=
        solve.forward_terms + solve.backward_terms;
    replay.checker_work.factor_divisions += solve.divisions;
}

void add_dot_work(Replay& replay, const FormulaProbeK2Dot& dot, bool rho) {
    if (rho) {
        ++replay.expected.recurrence.work.rho_dots;
        ++replay.checker_work.rho_dots;
    } else {
        ++replay.expected.recurrence.work.denominator_dots;
        ++replay.checker_work.denominator_dots;
    }
    replay.expected.recurrence.work.dot_terms += dot.terms;
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
        replay.expected.hybrid_work.input_components += 2U;
        replay.checker_work.input_components += 2U;
    }
    const FormulaProbeProduct high_product = exact
        ? formula_probe_tangent_product(fixture, high)
        : FormulaProbeProduct{};
    const FormulaProbeProduct low_product = exact
        ? formula_probe_tangent_product(fixture, low)
        : FormulaProbeProduct{};
    ++replay.expected.hybrid_work.operator_products;
    replay.expected.hybrid_work.tangent_kernel_calls += 2U;
    replay.expected.hybrid_work.tangent_inner_dots +=
        high_product.inner_dots + low_product.inner_dots;
    replay.expected.hybrid_work.tangent_outer_dots +=
        high_product.outer_dots + low_product.outer_dots;
    replay.expected.hybrid_work.tangent_inner_terms +=
        high_product.inner_terms + low_product.inner_terms;
    replay.expected.hybrid_work.tangent_outer_terms +=
        high_product.outer_terms + low_product.outer_terms;
    replay.expected.hybrid_work.tangent_scale_products +=
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
        replay.expected.hybrid_work.output_projections += 2U;
        ++replay.expected.hybrid_work.output_additions;
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
    replay.checker_work.root_derivation_paths += 5U;
    ++replay.expected.recurrence.work.operator_products;
    replay.expected.recurrence.work.operator_entries += result.entries;
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
    const FormulaProbeK2Scalar& beta, Replay& replay) {
    FormulaProbeTwofoldState result;
    result.exact = certificate.exact;
    result.index = index;
    result.solution_components = solution;
    result.residual_components = residual;
    result.direction_components = direction;
    result.preconditioned_components = preconditioned;
    result.certificate = certificate.certificate;
    result.rho = rho;
    result.denominator = denominator;
    result.alpha = alpha;
    result.beta = beta;
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
    result.root = legacy_state_root(result);
    ++replay.checker_work.root_derivation_paths;
    return result;
}

void add_certificate_work(
    Replay& replay, const FormulaProbeK2CertificateCheck& check) {
    ++replay.expected.recurrence.work.certificates;
    ++replay.checker_work.certificate_checks;
    replay.checker_work.certificate_dots += check.dots;
    replay.checker_work.certificate_dot_products += check.dot_products;
    replay.checker_work.certificate_radius_terms += check.radius_terms;
    replay.checker_work.certificate_solution_dots += check.solution_dots;
    replay.checker_work.certificate_sign_comparisons +=
        check.sign_comparisons;
    replay.expected.hybrid_work.certificate_dots += check.dots;
    replay.expected.hybrid_work.certificate_dot_products += check.dot_products;
    replay.expected.hybrid_work.certificate_radius_terms += check.radius_terms;
    replay.expected.hybrid_work.certificate_solution_dots += check.solution_dots;
    replay.expected.hybrid_work.certificate_sign_comparisons +=
        check.sign_comparisons;
}

std::string expected_transaction_root(const FormulaProbeParentFixture& fixture,
    const std::string& artifact_root,
    const FormulaProbeK2Scalar& inverse_scale,
    const std::vector<double>& rhs,
    const FormulaProbeTwofoldWork& work,
    const std::vector<FormulaProbeTwofoldState>& states) {
    std::ostringstream material;
    material << true << ':' << true << ':' << true << ':' << true << ':'
        << states.size() << ':' << "" << ':' << artifact_root << ':'
        << fixture.factor_root << ':' << fixture.permutation_root << ':'
        << inverse_scale.root << ':' << formula_probe_k2_vector_root(rhs) << ':'
        << work.operator_products << ':' << work.factor_solves << ':'
        << work.rho_dots << ':' << work.denominator_dots << ':'
        << work.solution_updates << ':' << work.residual_updates << ':'
        << work.direction_updates << '|';
    for (const FormulaProbeTwofoldState& state : states)
        material << transaction_state_root(state) << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string semantic_trace_root(const Replay& replay) {
    const FormulaProbeHybridRecurrence& value = replay.expected;
    std::ostringstream material;
    material << value.exact << ':' << value.recurrence.exact << ':'
        << value.recurrence.complete << ':'
        << value.recurrence.positivity_exact << ':'
        << value.recurrence.sealed_states << ':'
        << value.recurrence.failure_stage << ':'
        << value.recurrence.fixture_root << ':'
        << value.recurrence.artifact_root << ':'
        << replay.identity.root << ':' << value.callback_identity_root << ':'
        << value.callback_root << '|';
    for (const FormulaProbeTwofoldProduct& product : value.recurrence.products) {
        material << product.exact << ':' << product.site << ':'
            << product.dimension << ':' << product.entries << ':'
            << product.artifact_root << ':' << product.input_root << ':'
            << product.value_root << ':' << product.high_product_root << ':'
            << product.low_product_root << ':' << product.execution_root << ':'
            << product.callback_root << ':' << product.root << ':'
            << formula_probe_binary64_vector_root(product.input_components)
            << ':'
            << formula_probe_binary64_vector_root(product.value_components)
            << ';';
    }
    material << '|';
    for (const FormulaProbeTwofoldState& state : value.recurrence.states) {
        material << state.exact << ':' << state.index << ':'
            << formula_probe_binary64_vector_root(state.solution_components)
            << ':'
            << formula_probe_binary64_vector_root(state.residual_components)
            << ':'
            << formula_probe_binary64_vector_root(state.direction_components)
            << ':' << formula_probe_binary64_vector_root(
                state.preconditioned_components) << ':';
        append_scalar(material, state.rho);
        material << ':';
        append_scalar(material, state.denominator);
        material << ':';
        append_scalar(material, state.alpha);
        material << ':';
        append_scalar(material, state.beta);
        material << ':' << state.operator_root << ':' << state.solve_root << ':'
            << state.certificate.exact << ':' << state.certificate.passed << ':'
            << state.certificate.positive << ':' << state.certificate.negative
            << ':' << state.certificate.unresolved << ':'
            << bits(state.certificate.error_upper) << ':'
            << state.certificate.solution_root << ':'
            << state.certificate.sign_root << ':' << state.certificate.root
            << ':' << state.root << ';';
    }
    material << '|' << replay.expected_producer_payload_root << ':'
        << value.recurrence.certificate_root << ':'
        << value.recurrence.transaction_root << ':' << value.recurrence.root
        << ':' << value.root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

Replay independent_replay(const FormulaProbeParentFixture& fixture,
    const FormulaProbeHybridIdentity& identity) {
    Replay result;
    result.identity = identity;
    result.checker_work.fixture_validations = 1U;
    result.checker_work.identity_derivations = 1U;
    result.expected.hybrid_work.fixture_validations = 1U;
    result.expected.hybrid_work.identity_derivations = 1U;
    result.expected.recurrence.fixture_root = fixture.root;
    result.expected.recurrence.artifact_root = identity.carrier_root;
    result.expected.callback_identity_root = identity.callback_identity_root;
    if (!identity.exact || fixture.root != EXPECTED_FIXTURE_ROOT
        || fixture.dimension != DIMENSION || identity.carrier_root.empty()
        || identity.callback_identity_root.empty()) {
        result.failure_stage = "fixture_identity";
        return result;
    }

    std::vector<double> rhs;
    rhs.reserve(2U * fixture.dimension);
    bool arithmetic = true;
    for (FormulaProbeBinary128 source : fixture.projected_rhs) {
        const FormulaProbeK2Scalar value = formula_probe_k2_project(source);
        arithmetic = arithmetic && value.exact;
        rhs.push_back(value.high);
        rhs.push_back(value.low);
        ++result.checker_work.projected_scalars;
    }
    const FormulaProbeK2Scalar inverse_scale =
        formula_probe_k2_project(fixture.projected_scale);
    ++result.checker_work.projected_scalars;
    const FormulaProbeK2Scalar zero = formula_probe_k2_exact_double(0.0);
    const FormulaProbeK2Scalar one = formula_probe_k2_exact_double(1.0);
    result.checker_work.exact_scalars += 2U;
    arithmetic = arithmetic && inverse_scale.exact && zero.exact && one.exact;

    ++result.checker_work.factor_payload_validations;
    const FormulaProbeK2Solve start = formula_probe_k2_factor_solve(
        fixture, "start", inverse_scale, rhs);
    add_solve_work(result, start);
    const FormulaProbeTwofoldProduct kx0 = replay_product(fixture, "Kx0",
        identity.carrier_root, identity.callback_identity_root,
        start.solution_components, result);
    const std::vector<double> r0 = formula_probe_k2_update(
        rhs, one, kx0.value_components, -1.0);
    ++result.checker_work.factor_payload_validations;
    const FormulaProbeK2Solve z0 = formula_probe_k2_factor_solve(
        fixture, "initial_residual", inverse_scale, r0);
    add_solve_work(result, z0);
    const FormulaProbeK2Dot rho0 =
        formula_probe_k2_dot(r0, z0.solution_components);
    add_dot_work(result, rho0, true);
    ++result.checker_work.positivity_checks;
    if (!arithmetic || !start.exact || !kx0.exact || !z0.exact
        || !rho0.exact || r0.size() != 2U * fixture.dimension) {
        result.failure_stage = "initial_arithmetic";
        return result;
    }
    if (!formula_probe_k2_positive(rho0.value)) {
        result.failure_stage = "initial_rho";
        return result;
    }

    const std::vector<double> x0 = start.solution_components;
    const std::vector<double> p0 = z0.solution_components;
    const FormulaProbeTwofoldProduct kp0 = replay_product(fixture, "Kp0",
        identity.carrier_root, identity.callback_identity_root, p0, result);
    const FormulaProbeK2Dot denominator0 =
        formula_probe_k2_dot(p0, kp0.value_components);
    add_dot_work(result, denominator0, false);
    ++result.checker_work.positivity_checks;
    if (!kp0.exact || !denominator0.exact) {
        result.failure_stage = "iteration1_arithmetic";
        return result;
    }
    if (!formula_probe_k2_positive(denominator0.value)) {
        result.failure_stage = "iteration1_denominator";
        return result;
    }

    const FormulaProbeK2Scalar alpha0 =
        formula_probe_k2_divide(rho0.value, denominator0.value);
    ++result.expected.recurrence.work.scalar_divisions;
    ++result.checker_work.scalar_divisions;
    const std::vector<double> x1 =
        formula_probe_k2_update(x0, alpha0, p0, 1.0);
    const std::vector<double> r1 = formula_probe_k2_update(
        r0, alpha0, kp0.value_components, -1.0);
    result.expected.recurrence.work.solution_updates += fixture.dimension;
    result.expected.recurrence.work.residual_updates += fixture.dimension;
    result.checker_work.solution_updates += fixture.dimension;
    result.checker_work.residual_updates += fixture.dimension;
    if (!alpha0.exact || x1.size() != 2U * fixture.dimension
        || r1.size() != 2U * fixture.dimension) {
        result.failure_stage = "iteration1_update";
        return result;
    }

    ++result.checker_work.factor_payload_validations;
    const FormulaProbeK2Solve z1 = formula_probe_k2_factor_solve(
        fixture, "iteration1_residual", inverse_scale, r1);
    add_solve_work(result, z1);
    const FormulaProbeK2Dot rho1 =
        formula_probe_k2_dot(r1, z1.solution_components);
    add_dot_work(result, rho1, true);
    ++result.checker_work.positivity_checks;
    if (!z1.exact || !rho1.exact) {
        result.failure_stage = "iteration1_preconditioner";
        return result;
    }
    if (!formula_probe_k2_positive(rho1.value)) {
        result.failure_stage = "iteration1_rho";
        return result;
    }

    const FormulaProbeK2Scalar beta1 =
        formula_probe_k2_divide(rho1.value, rho0.value);
    ++result.expected.recurrence.work.scalar_divisions;
    ++result.checker_work.scalar_divisions;
    const std::vector<double> p1 = formula_probe_k2_update(
        z1.solution_components, beta1, p0, 1.0);
    result.expected.recurrence.work.direction_updates += fixture.dimension;
    result.checker_work.direction_updates += fixture.dimension;
    if (!beta1.exact || p1.size() != 2U * fixture.dimension) {
        result.failure_stage = "direction_update";
        return result;
    }

    const FormulaProbeTwofoldProduct kp1 = replay_product(fixture, "Kp1",
        identity.carrier_root, identity.callback_identity_root, p1, result);
    const FormulaProbeK2Dot denominator1 =
        formula_probe_k2_dot(p1, kp1.value_components);
    add_dot_work(result, denominator1, false);
    ++result.checker_work.positivity_checks;
    if (!kp1.exact || !denominator1.exact) {
        result.failure_stage = "iteration2_arithmetic";
        return result;
    }
    if (!formula_probe_k2_positive(denominator1.value)) {
        result.failure_stage = "iteration2_denominator";
        return result;
    }

    const FormulaProbeK2Scalar alpha1 =
        formula_probe_k2_divide(rho1.value, denominator1.value);
    ++result.expected.recurrence.work.scalar_divisions;
    ++result.checker_work.scalar_divisions;
    const std::vector<double> x2 =
        formula_probe_k2_update(x1, alpha1, p1, 1.0);
    const std::vector<double> r2 = formula_probe_k2_update(
        r1, alpha1, kp1.value_components, -1.0);
    result.expected.recurrence.work.solution_updates += fixture.dimension;
    result.expected.recurrence.work.residual_updates += fixture.dimension;
    result.checker_work.solution_updates += fixture.dimension;
    result.checker_work.residual_updates += fixture.dimension;
    if (!alpha1.exact || x2.size() != 2U * fixture.dimension
        || r2.size() != 2U * fixture.dimension) {
        result.failure_stage = "iteration2_update";
        return result;
    }

    result.expected.recurrence.products = {kx0, kp0, kp1};
    const std::array<std::vector<double>, 3U> solutions{x0, x1, x2};
    for (const std::vector<double>& solution : solutions) {
        ++result.checker_work.profile_payload_validations;
        FormulaProbeK2CertificateCheck check =
            formula_probe_k2_certificate_check(fixture, solution);
        arithmetic = arithmetic && check.exact;
        add_certificate_work(result, check);
        result.certificate_checks.push_back(std::move(check));
    }
    result.expected.recurrence.states.push_back(replay_state(0U, x0, r0, p0,
        z0.solution_components, result.certificate_checks[0U],
        kx0.execution_root, z0.root, rho0.value, zero, zero, zero, result));
    result.expected.recurrence.states.push_back(replay_state(1U, x1, r1, p1,
        z1.solution_components, result.certificate_checks[1U],
        kp0.execution_root, z1.root, rho1.value, denominator0.value, alpha0,
        beta1, result));
    result.expected.recurrence.states.push_back(replay_state(2U, x2, r2, p1,
        z1.solution_components, result.certificate_checks[2U],
        kp1.execution_root, z1.root, rho1.value, denominator1.value, alpha1,
        beta1, result));

    result.expected.recurrence.complete = true;
    result.expected.recurrence.positivity_exact = true;
    result.expected.recurrence.sealed_states =
        result.expected.recurrence.states.size();
    std::vector<FormulaProbeCertificate> certificates;
    for (const FormulaProbeTwofoldState& state : result.expected.recurrence.states)
        certificates.push_back(state.certificate);
    result.expected.recurrence.certificate_root =
        formula_probe_certificate_set_root(certificates);
    result.expected.recurrence.transaction_root = expected_transaction_root(
        fixture, identity.carrier_root, inverse_scale, rhs,
        result.expected.recurrence.work, result.expected.recurrence.states);
    result.expected.recurrence.exact = arithmetic
        && result.expected.recurrence.states.size() == 3U
        && result.expected.recurrence.products.size() == 3U;
    result.expected.recurrence.root =
        formula_probe_twofold_recurrence_root(result.expected.recurrence);
    result.expected.callback_root =
        formula_probe_hybrid_callback_root(result.expected);

    result.exact = result.expected.recurrence.exact
        && result.expected.recurrence.work.certificates == 3U
        && result.expected.recurrence.work.operator_products == 3U
        && result.expected.recurrence.work.operator_entries == 31212U
        && result.expected.recurrence.work.factor_solves == 3U
        && result.expected.recurrence.work.factor_terms == 30906U
        && result.expected.recurrence.work.factor_divisions == 612U
        && result.expected.recurrence.work.rho_dots == 2U
        && result.expected.recurrence.work.denominator_dots == 2U
        && result.expected.recurrence.work.dot_terms == 408U
        && result.expected.recurrence.work.scalar_divisions == 3U
        && result.expected.recurrence.work.solution_updates == 204U
        && result.expected.recurrence.work.residual_updates == 204U
        && result.expected.recurrence.work.direction_updates == 102U
        && result.expected.recurrence.work.adaptive_stops == 0U
        && result.expected.hybrid_work.fixture_validations == 1U
        && result.expected.hybrid_work.identity_derivations == 1U
        && result.expected.hybrid_work.operator_products == 3U
        && result.expected.hybrid_work.tangent_kernel_calls == 6U
        && result.expected.hybrid_work.tangent_inner_dots == 1890U
        && result.expected.hybrid_work.tangent_outer_dots == 612U
        && result.expected.hybrid_work.tangent_inner_terms == 192780U
        && result.expected.hybrid_work.tangent_outer_terms == 192780U
        && result.expected.hybrid_work.tangent_scale_products == 612U
        && result.expected.hybrid_work.input_components == 612U
        && result.expected.hybrid_work.output_projections == 612U
        && result.expected.hybrid_work.output_additions == 306U
        && result.expected.hybrid_work.certificate_dots == 306U
        && result.expected.hybrid_work.certificate_dot_products == 125460U
        && result.expected.hybrid_work.certificate_radius_terms == 93636U
        && result.expected.hybrid_work.certificate_solution_dots == 306U
        && result.expected.hybrid_work.certificate_sign_comparisons == 306U;
    result.expected.exact = result.exact;
    result.expected.root =
        formula_probe_hybrid_twofold_recurrence_root(result.expected);
    result.expected_recurrence_work_root =
        recurrence_work_root(result.expected.recurrence.work);
    result.expected_hybrid_work_root =
        hybrid_work_root(result.expected.hybrid_work);
    result.expected_producer_payload_root = producer_payload_root(
        result.expected.recurrence.work, result.expected.hybrid_work);
    result.expected_producer_receipt_root = producer_receipt_root(
        "expected", result.expected_producer_payload_root);
    result.semantic_trace_root = semantic_trace_root(result);
    result.checker_work.root_derivation_paths += 9U;
    return result;
}

bool compare_certificate(const FormulaProbeCertificate& left,
    const FormulaProbeCertificate& right, ComparisonWork* work) {
    std::size_t* counter = work == nullptr
        ? nullptr : &work->certificate_field_comparisons;
    bool equal = true;
    equal &= compare_field(left.exact == right.exact, counter);
    equal &= compare_field(left.passed == right.passed, counter);
    equal &= compare_field(left.positive == right.positive, counter);
    equal &= compare_field(left.negative == right.negative, counter);
    equal &= compare_field(left.unresolved == right.unresolved, counter);
    equal &= compare_field(
        bits(left.error_upper) == bits(right.error_upper), counter);
    equal &= compare_field(left.solution_root == right.solution_root, counter);
    equal &= compare_field(left.sign_root == right.sign_root, counter);
    equal &= compare_field(left.root == right.root, counter);
    return equal;
}

bool compare_scalar(const FormulaProbeK2Scalar& left,
    const FormulaProbeK2Scalar& right, ComparisonWork* work) {
    std::size_t* guard = work == nullptr
        ? nullptr : &work->scalar_guard_comparisons;
    std::size_t* components = work == nullptr
        ? nullptr : &work->scalar_component_comparisons;
    std::size_t* roots = work == nullptr
        ? nullptr : &work->scalar_root_comparisons;
    bool equal = true;
    equal &= compare_field(left.exact == right.exact, guard);
    equal &= compare_field(bits(left.high) == bits(right.high), components);
    equal &= compare_field(bits(left.low) == bits(right.low), components);
    equal &= compare_field(left.root == right.root, roots);
    return equal;
}

bool compare_recurrence_work(const FormulaProbeTwofoldWork& left,
    const FormulaProbeTwofoldWork& right, ComparisonWork* work) {
    std::size_t* counter = work == nullptr
        ? nullptr : &work->recurrence_work_field_comparisons;
    bool equal = true;
    equal &= compare_field(left.certificates == right.certificates, counter);
    equal &= compare_field(
        left.operator_products == right.operator_products, counter);
    equal &= compare_field(left.operator_entries == right.operator_entries, counter);
    equal &= compare_field(left.factor_solves == right.factor_solves, counter);
    equal &= compare_field(left.factor_terms == right.factor_terms, counter);
    equal &= compare_field(
        left.factor_divisions == right.factor_divisions, counter);
    equal &= compare_field(left.rho_dots == right.rho_dots, counter);
    equal &= compare_field(
        left.denominator_dots == right.denominator_dots, counter);
    equal &= compare_field(left.dot_terms == right.dot_terms, counter);
    equal &= compare_field(
        left.scalar_divisions == right.scalar_divisions, counter);
    equal &= compare_field(
        left.solution_updates == right.solution_updates, counter);
    equal &= compare_field(
        left.residual_updates == right.residual_updates, counter);
    equal &= compare_field(
        left.direction_updates == right.direction_updates, counter);
    equal &= compare_field(left.adaptive_stops == right.adaptive_stops, counter);
    return equal;
}

bool compare_hybrid_work(const FormulaProbeHybridWork& left,
    const FormulaProbeHybridWork& right, ComparisonWork* work) {
    std::size_t* counter = work == nullptr
        ? nullptr : &work->hybrid_work_field_comparisons;
    bool equal = true;
    equal &= compare_field(
        left.fixture_validations == right.fixture_validations, counter);
    equal &= compare_field(
        left.identity_derivations == right.identity_derivations, counter);
    equal &= compare_field(
        left.operator_products == right.operator_products, counter);
    equal &= compare_field(
        left.tangent_kernel_calls == right.tangent_kernel_calls, counter);
    equal &= compare_field(
        left.tangent_inner_dots == right.tangent_inner_dots, counter);
    equal &= compare_field(
        left.tangent_outer_dots == right.tangent_outer_dots, counter);
    equal &= compare_field(
        left.tangent_inner_terms == right.tangent_inner_terms, counter);
    equal &= compare_field(
        left.tangent_outer_terms == right.tangent_outer_terms, counter);
    equal &= compare_field(
        left.tangent_scale_products == right.tangent_scale_products, counter);
    equal &= compare_field(
        left.input_components == right.input_components, counter);
    equal &= compare_field(
        left.output_projections == right.output_projections, counter);
    equal &= compare_field(
        left.output_additions == right.output_additions, counter);
    equal &= compare_field(
        left.certificate_dots == right.certificate_dots, counter);
    equal &= compare_field(left.certificate_dot_products
        == right.certificate_dot_products, counter);
    equal &= compare_field(left.certificate_radius_terms
        == right.certificate_radius_terms, counter);
    equal &= compare_field(left.certificate_solution_dots
        == right.certificate_solution_dots, counter);
    equal &= compare_field(left.certificate_sign_comparisons
        == right.certificate_sign_comparisons, counter);
    return equal;
}

Mismatch compare_trace(const FormulaProbeHybridRecurrence& trace,
    const Replay& replay, ComparisonWork* work) {
    if (work != nullptr) ++work->trace_calls;
    const FormulaProbeHybridRecurrence& expected = replay.expected;
    std::size_t* apparatus = work == nullptr
        ? nullptr : &work->apparatus_comparisons;
    bool apparatus_exact = true;
    apparatus_exact &= compare_field(replay.exact, apparatus);
    apparatus_exact &= compare_field(trace.recurrence.states.size()
        == expected.recurrence.states.size(), apparatus);
    apparatus_exact &= compare_field(trace.recurrence.products.size()
        == expected.recurrence.products.size(), apparatus);
    if (!apparatus_exact) return Mismatch::Apparatus;

    std::size_t* identity = work == nullptr
        ? nullptr : &work->identity_comparisons;
    bool identity_exact = true;
    identity_exact &= compare_field(trace.callback_identity_root
        == expected.callback_identity_root, identity);
    identity_exact &= compare_field(trace.recurrence.fixture_root
        == expected.recurrence.fixture_root, identity);
    identity_exact &= compare_field(trace.recurrence.artifact_root
        == expected.recurrence.artifact_root, identity);
    if (!identity_exact) return Mismatch::CallbackIdentity;

    for (std::size_t index = 0U;
         index < expected.recurrence.products.size(); ++index) {
        const FormulaProbeTwofoldProduct& actual =
            trace.recurrence.products[index];
        const FormulaProbeTwofoldProduct& wanted =
            expected.recurrence.products[index];
        std::size_t* guard = work == nullptr
            ? nullptr : &work->product_guard_comparisons;
        std::size_t* metadata = work == nullptr
            ? nullptr : &work->product_metadata_comparisons;
        std::size_t* components = work == nullptr
            ? nullptr : &work->product_component_comparisons;
        std::size_t* roots = work == nullptr
            ? nullptr : &work->product_root_comparisons;
        bool product_exact = true;
        product_exact &= compare_field(actual.exact == wanted.exact, guard);
        product_exact &= compare_field(actual.site == wanted.site, metadata);
        product_exact &= compare_field(
            actual.dimension == wanted.dimension, metadata);
        product_exact &= compare_field(actual.entries == wanted.entries, metadata);
        product_exact &= compare_field(actual.input_components.size()
            == wanted.input_components.size(), guard);
        product_exact &= compare_field(actual.value_components.size()
            == wanted.value_components.size(), guard);
        product_exact &= compare_components(
            actual.input_components, wanted.input_components, components);
        product_exact &= compare_components(
            actual.value_components, wanted.value_components, components);
        product_exact &= compare_field(
            actual.artifact_root == wanted.artifact_root, roots);
        product_exact &= compare_field(
            actual.input_root == wanted.input_root, roots);
        product_exact &= compare_field(
            actual.value_root == wanted.value_root, roots);
        product_exact &= compare_field(
            actual.high_product_root == wanted.high_product_root, roots);
        product_exact &= compare_field(
            actual.low_product_root == wanted.low_product_root, roots);
        product_exact &= compare_field(
            actual.execution_root == wanted.execution_root, roots);
        product_exact &= compare_field(
            actual.callback_root == wanted.callback_root, roots);
        product_exact &= compare_field(actual.root == wanted.root, roots);
        if (!product_exact) return Mismatch::Product;
    }

    std::size_t* callback = work == nullptr
        ? nullptr : &work->callback_comparisons;
    if (!compare_field(trace.callback_root == expected.callback_root, callback))
        return Mismatch::CallbackRoot;

    std::size_t* recurrence_guard = work == nullptr
        ? nullptr : &work->recurrence_guard_comparisons;
    bool recurrence_guard_exact = true;
    recurrence_guard_exact &= compare_field(trace.exact == expected.exact,
        recurrence_guard);
    recurrence_guard_exact &= compare_field(trace.recurrence.exact
        == expected.recurrence.exact, recurrence_guard);
    recurrence_guard_exact &= compare_field(trace.recurrence.complete
        == expected.recurrence.complete, recurrence_guard);
    recurrence_guard_exact &= compare_field(trace.recurrence.positivity_exact
        == expected.recurrence.positivity_exact, recurrence_guard);
    recurrence_guard_exact &= compare_field(trace.recurrence.sealed_states
        == expected.recurrence.sealed_states, recurrence_guard);
    recurrence_guard_exact &= compare_field(trace.recurrence.failure_stage
        == expected.recurrence.failure_stage, recurrence_guard);
    recurrence_guard_exact &= compare_field(trace.recurrence.transaction_root
        == expected.recurrence.transaction_root, recurrence_guard);
    if (!recurrence_guard_exact) return Mismatch::RecurrenceGuard;

    for (std::size_t index = 0U;
         index < expected.recurrence.states.size(); ++index) {
        const FormulaProbeTwofoldState& actual = trace.recurrence.states[index];
        const FormulaProbeTwofoldState& wanted =
            expected.recurrence.states[index];
        std::size_t* guard = work == nullptr
            ? nullptr : &work->state_guard_comparisons;
        std::size_t* components = work == nullptr
            ? nullptr : &work->state_component_comparisons;
        bool state_exact = true;
        state_exact &= compare_field(actual.exact == wanted.exact, guard);
        state_exact &= compare_field(actual.index == wanted.index, guard);
        state_exact &= compare_field(actual.solution_components.size()
            == wanted.solution_components.size(), guard);
        state_exact &= compare_field(actual.residual_components.size()
            == wanted.residual_components.size(), guard);
        state_exact &= compare_field(actual.direction_components.size()
            == wanted.direction_components.size(), guard);
        state_exact &= compare_field(actual.preconditioned_components.size()
            == wanted.preconditioned_components.size(), guard);
        state_exact &= compare_components(
            actual.solution_components, wanted.solution_components, components);
        state_exact &= compare_components(
            actual.residual_components, wanted.residual_components, components);
        state_exact &= compare_components(actual.direction_components,
            wanted.direction_components, components);
        state_exact &= compare_components(actual.preconditioned_components,
            wanted.preconditioned_components, components);
        if (!state_exact) return Mismatch::State;

        bool scalar_exact = true;
        scalar_exact &= compare_scalar(actual.rho, wanted.rho, work);
        scalar_exact &= compare_scalar(
            actual.denominator, wanted.denominator, work);
        scalar_exact &= compare_scalar(actual.alpha, wanted.alpha, work);
        scalar_exact &= compare_scalar(actual.beta, wanted.beta, work);
        std::size_t* scalar_roots = work == nullptr
            ? nullptr : &work->scalar_root_comparisons;
        scalar_exact &= compare_field(
            actual.operator_root == wanted.operator_root, scalar_roots);
        scalar_exact &= compare_field(
            actual.solve_root == wanted.solve_root, scalar_roots);
        scalar_exact &= compare_field(
            actual.rho_root == wanted.rho_root, scalar_roots);
        scalar_exact &= compare_field(actual.denominator_root
            == wanted.denominator_root, scalar_roots);
        scalar_exact &= compare_field(
            actual.alpha_root == wanted.alpha_root, scalar_roots);
        scalar_exact &= compare_field(
            actual.beta_root == wanted.beta_root, scalar_roots);
        if (!scalar_exact) return Mismatch::Scalar;

        if (!compare_certificate(actual.certificate, wanted.certificate, work))
            return Mismatch::Certificate;
        std::size_t* state_roots = work == nullptr
            ? nullptr : &work->state_root_comparisons;
        if (!compare_field(actual.root == wanted.root, state_roots))
            return Mismatch::State;
    }

    std::size_t* certificate_fields = work == nullptr
        ? nullptr : &work->certificate_field_comparisons;
    if (!compare_field(trace.recurrence.certificate_root
            == expected.recurrence.certificate_root, certificate_fields))
        return Mismatch::Certificate;

    bool recurrence_work_exact = compare_recurrence_work(
        trace.recurrence.work, expected.recurrence.work, work);
    std::size_t* recurrence_roots = work == nullptr
        ? nullptr : &work->recurrence_root_comparisons;
    const std::string actual_recurrence_work_root =
        recurrence_work_root(trace.recurrence.work);
    if (work != nullptr) ++work->producer_root_derivations;
    recurrence_work_exact &= compare_field(actual_recurrence_work_root
        == replay.expected_recurrence_work_root, recurrence_roots);
    recurrence_work_exact &= compare_field(trace.recurrence.root
        == expected.recurrence.root, recurrence_roots);
    if (!recurrence_work_exact) return Mismatch::RecurrenceWork;

    bool hybrid_work_exact = compare_hybrid_work(
        trace.hybrid_work, expected.hybrid_work, work);
    std::size_t* hybrid_roots = work == nullptr
        ? nullptr : &work->hybrid_root_comparisons;
    const std::string actual_hybrid_work_root =
        hybrid_work_root(trace.hybrid_work);
    const std::string actual_payload_root = producer_payload_root(
        trace.recurrence.work, trace.hybrid_work);
    if (work != nullptr) work->producer_root_derivations += 3U;
    hybrid_work_exact &= compare_field(actual_hybrid_work_root
        == replay.expected_hybrid_work_root, hybrid_roots);
    hybrid_work_exact &= compare_field(actual_payload_root
        == replay.expected_producer_payload_root, hybrid_roots);
    hybrid_work_exact &= compare_field(trace.root == expected.root, hybrid_roots);
    if (!hybrid_work_exact) return Mismatch::HybridWork;
    return Mismatch::None;
}

void reseal_outer(FormulaProbeHybridRecurrence& value, ControlWork* work) {
    std::vector<FormulaProbeCertificate> certificates;
    for (const FormulaProbeTwofoldState& state : value.recurrence.states)
        certificates.push_back(state.certificate);
    value.recurrence.certificate_root =
        formula_probe_certificate_set_root(certificates);
    value.recurrence.root =
        formula_probe_twofold_recurrence_root(value.recurrence);
    value.callback_root = formula_probe_hybrid_callback_root(value);
    value.root = formula_probe_hybrid_twofold_recurrence_root(value);
    if (work != nullptr) {
        ++work->reseal_paths;
        work->root_derivation_paths += 4U;
    }
}

std::string mismatch_route(Mismatch mismatch) {
    switch (mismatch) {
    case Mismatch::None:
        return {};
    case Mismatch::Apparatus:
        return "INDEPENDENT_CHECKER_APPARATUS_REJECTED";
    case Mismatch::CallbackIdentity:
        return "HYBRID_TRACE_IDENTITY_REJECTED";
    case Mismatch::Product:
        return "HYBRID_TRACE_PRODUCT_REJECTED";
    case Mismatch::CallbackRoot:
        return "HYBRID_TRACE_CALLBACK_REJECTED";
    case Mismatch::RecurrenceGuard:
        return "HYBRID_TRACE_RECURRENCE_GUARD_REJECTED";
    case Mismatch::State:
        return "HYBRID_TRACE_STATE_REJECTED";
    case Mismatch::Scalar:
        return "HYBRID_TRACE_SCALAR_REJECTED";
    case Mismatch::Certificate:
        return "HYBRID_TRACE_CERTIFICATE_REJECTED";
    case Mismatch::RecurrenceWork:
        return "HYBRID_TRACE_RECURRENCE_WORK_REJECTED";
    case Mismatch::HybridWork:
        return "HYBRID_TRACE_HYBRID_WORK_REJECTED";
    }
    return "INDEPENDENT_CHECKER_CLASSIFIER_REJECTED";
}

std::string classify(Mismatch mismatch, bool work, bool controls, bool ladder) {
    const std::string mismatch_class = mismatch_route(mismatch);
    if (!mismatch_class.empty()) return mismatch_class;
    if (mismatch != Mismatch::None)
        return "INDEPENDENT_CHECKER_CLASSIFIER_REJECTED";
    if (!work) return "INDEPENDENT_CHECKER_WORK_REJECTED";
    if (!controls) return "INDEPENDENT_CHECKER_CONTROLS_REJECTED";
    if (!ladder) return "HYBRID_TRACE_LADDER_REJECTED";
    return "HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE";
}

bool classifier_control() {
    const std::array<Mismatch, 12U> mismatches{
        Mismatch::None,
        Mismatch::Apparatus,
        Mismatch::CallbackIdentity,
        Mismatch::Product,
        Mismatch::CallbackRoot,
        Mismatch::RecurrenceGuard,
        Mismatch::State,
        Mismatch::Scalar,
        Mismatch::Certificate,
        Mismatch::RecurrenceWork,
        Mismatch::HybridWork,
        static_cast<Mismatch>(255),
    };
    for (Mismatch mismatch : mismatches) {
        for (unsigned mask = 0U; mask < 8U; ++mask) {
            const bool work = (mask & 1U) != 0U;
            const bool controls = (mask & 2U) != 0U;
            const bool ladder = (mask & 4U) != 0U;
            std::string expected = mismatch_route(mismatch);
            if (expected.empty()) {
                if (mismatch != Mismatch::None)
                    expected = "INDEPENDENT_CHECKER_CLASSIFIER_REJECTED";
                else if (!work)
                    expected = "INDEPENDENT_CHECKER_WORK_REJECTED";
                else if (!controls)
                    expected = "INDEPENDENT_CHECKER_CONTROLS_REJECTED";
                else if (!ladder)
                    expected = "HYBRID_TRACE_LADDER_REJECTED";
                else
                    expected =
                        "HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE";
            }
            if (classify(mismatch, work, controls, ladder) != expected)
                return false;
        }
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

bool baseline_comparison_work_exact(const ComparisonWork& value) {
    return value.trace_calls == 1U
        && value.apparatus_comparisons == 3U
        && value.identity_comparisons == 3U
        && value.product_guard_comparisons == 9U
        && value.product_metadata_comparisons == 9U
        && value.product_component_comparisons == 1224U
        && value.product_root_comparisons == 24U
        && value.callback_comparisons == 1U
        && value.recurrence_guard_comparisons == 7U
        && value.state_guard_comparisons == 18U
        && value.state_component_comparisons == 2448U
        && value.scalar_guard_comparisons == 12U
        && value.scalar_component_comparisons == 24U
        && value.scalar_root_comparisons == 30U
        && value.certificate_field_comparisons == 28U
        && value.state_root_comparisons == 3U
        && value.recurrence_work_field_comparisons == 14U
        && value.recurrence_root_comparisons == 2U
        && value.hybrid_work_field_comparisons == 17U
        && value.hybrid_root_comparisons == 3U
        && value.producer_root_derivations == 4U;
}

bool checker_schedule_work_exact(const CheckerWork& value) {
    return value.fixture_validations == 1U
        && value.identity_derivations == 1U
        && value.projected_scalars == 103U
        && value.exact_scalars == 2U
        && value.operator_products == 3U
        && value.tangent_kernel_calls == 6U
        && value.tangent_inner_dots == 1890U
        && value.tangent_outer_dots == 612U
        && value.tangent_inner_terms == 192780U
        && value.tangent_outer_terms == 192780U
        && value.tangent_scale_products == 612U
        && value.input_components == 612U
        && value.output_projections == 612U
        && value.output_additions == 306U
        && value.factor_payload_validations == 3U
        && value.factor_solves == 3U
        && value.factor_terms == 30906U
        && value.factor_divisions == 612U
        && value.rho_dots == 2U
        && value.denominator_dots == 2U
        && value.dot_terms == 408U
        && value.positivity_checks == 4U
        && value.scalar_divisions == 3U
        && value.solution_updates == 204U
        && value.residual_updates == 204U
        && value.direction_updates == 102U
        && value.profile_payload_validations == 3U
        && value.certificate_checks == 3U
        && value.certificate_dots == 306U
        && value.certificate_dot_products == 125460U
        && value.certificate_radius_terms == 93636U
        && value.certificate_solution_dots == 306U
        && value.certificate_sign_comparisons == 306U
        && value.root_derivation_paths == 33U
        && value.result_seals == 1U
        && baseline_comparison_work_exact(value.comparison);
}

void record_author_validation_work(ControlWork& work, bool completed) {
    ++work.author_validations;
    if (!completed) return;
    ++work.author_fixture_validations;
    ++work.author_callback_replays;
    work.author_tangent_kernel_calls += 6U;
    work.author_tangent_inner_dots += 1890U;
    work.author_tangent_outer_dots += 612U;
    work.author_tangent_inner_terms += 192780U;
    work.author_tangent_outer_terms += 192780U;
    work.author_tangent_scale_products += 612U;
    work.author_input_components += 612U;
    work.author_output_projections += 612U;
    work.author_output_additions += 306U;
    ++work.root_derivation_paths;
}

bool control_comparison_work_exact(const ComparisonWork& value) {
    return value.trace_calls == 61U
        && value.apparatus_comparisons == 183U
        && value.identity_comparisons == 177U
        && value.product_guard_comparisons == 414U
        && value.product_metadata_comparisons == 414U
        && value.product_component_comparisons == 56100U
        && value.product_root_comparisons == 1104U
        && value.callback_comparisons == 41U
        && value.recurrence_guard_comparisons == 280U
        && value.state_guard_comparisons == 486U
        && value.state_component_comparisons == 65892U
        && value.scalar_guard_comparisons == 292U
        && value.scalar_component_comparisons == 584U
        && value.scalar_root_comparisons == 730U
        && value.certificate_field_comparisons == 582U
        && value.state_root_comparisons == 55U
        && value.recurrence_work_field_comparisons == 70U
        && value.recurrence_root_comparisons == 10U
        && value.hybrid_work_field_comparisons == 51U
        && value.hybrid_root_comparisons == 9U
        && value.producer_root_derivations == 14U;
}

bool control_work_exact(const ControlWork& value) {
    return value.controls == EXPECTED_CONTROL_COUNT
        && value.reseal_paths == 12U
        && value.root_derivation_paths == 78U
        && value.author_validations == 19U
        && value.author_fixture_validations == 19U
        && value.author_callback_replays == 19U
        && value.author_tangent_kernel_calls == 114U
        && value.author_tangent_inner_dots == 35910U
        && value.author_tangent_outer_dots == 11628U
        && value.author_tangent_inner_terms == 3662820U
        && value.author_tangent_outer_terms == 3662820U
        && value.author_tangent_scale_products == 11628U
        && value.author_input_components == 11628U
        && value.author_output_projections == 11628U
        && value.author_output_additions == 5814U
        && value.classifier_cases == EXPECTED_CLASSIFIER_CASES
        && value.result_seal_checks == 5U
        && control_comparison_work_exact(value.comparison);
}

std::string recurrence_work_json(const FormulaProbeTwofoldWork& value,
    const std::string& root) {
    std::ostringstream json;
    json << "{\"root\":\"" << root << "\",\"certificates\":"
        << value.certificates << ",\"operator_products\":"
        << value.operator_products << ",\"operator_entries\":"
        << value.operator_entries << ",\"factor_solves\":"
        << value.factor_solves << ",\"factor_terms\":"
        << value.factor_terms << ",\"factor_divisions\":"
        << value.factor_divisions << ",\"rho_dots\":" << value.rho_dots
        << ",\"denominator_dots\":" << value.denominator_dots
        << ",\"dot_terms\":" << value.dot_terms
        << ",\"scalar_divisions\":" << value.scalar_divisions
        << ",\"solution_updates\":" << value.solution_updates
        << ",\"residual_updates\":" << value.residual_updates
        << ",\"direction_updates\":" << value.direction_updates
        << ",\"adaptive_stops\":" << value.adaptive_stops << '}';
    return json.str();
}

std::string hybrid_work_json(const FormulaProbeHybridWork& value,
    const std::string& root) {
    std::ostringstream json;
    json << "{\"root\":\"" << root << "\",\"fixture_validations\":"
        << value.fixture_validations << ",\"identity_derivations\":"
        << value.identity_derivations << ",\"operator_products\":"
        << value.operator_products << ",\"tangent_kernel_calls\":"
        << value.tangent_kernel_calls << ",\"tangent_inner_dots\":"
        << value.tangent_inner_dots << ",\"tangent_outer_dots\":"
        << value.tangent_outer_dots << ",\"tangent_inner_terms\":"
        << value.tangent_inner_terms << ",\"tangent_outer_terms\":"
        << value.tangent_outer_terms << ",\"tangent_scale_products\":"
        << value.tangent_scale_products << ",\"input_components\":"
        << value.input_components << ",\"output_projections\":"
        << value.output_projections << ",\"output_additions\":"
        << value.output_additions << ",\"certificate_dots\":"
        << value.certificate_dots << ",\"certificate_dot_products\":"
        << value.certificate_dot_products << ",\"certificate_radius_terms\":"
        << value.certificate_radius_terms
        << ",\"certificate_solution_dots\":"
        << value.certificate_solution_dots
        << ",\"certificate_sign_comparisons\":"
        << value.certificate_sign_comparisons << '}';
    return json.str();
}

std::string comparison_work_json(
    const ComparisonWork& value, const std::string& root) {
    std::ostringstream json;
    json << "{\"root\":\"" << root
        << "\",\"trace_calls\":" << value.trace_calls
        << ",\"apparatus_comparisons\":" << value.apparatus_comparisons
        << ",\"identity_comparisons\":" << value.identity_comparisons
        << ",\"product_guard_comparisons\":"
        << value.product_guard_comparisons
        << ",\"product_metadata_comparisons\":"
        << value.product_metadata_comparisons
        << ",\"product_component_comparisons\":"
        << value.product_component_comparisons
        << ",\"product_root_comparisons\":"
        << value.product_root_comparisons
        << ",\"callback_comparisons\":" << value.callback_comparisons
        << ",\"recurrence_guard_comparisons\":"
        << value.recurrence_guard_comparisons
        << ",\"state_guard_comparisons\":"
        << value.state_guard_comparisons
        << ",\"state_component_comparisons\":"
        << value.state_component_comparisons
        << ",\"scalar_guard_comparisons\":"
        << value.scalar_guard_comparisons
        << ",\"scalar_component_comparisons\":"
        << value.scalar_component_comparisons
        << ",\"scalar_root_comparisons\":"
        << value.scalar_root_comparisons
        << ",\"certificate_field_comparisons\":"
        << value.certificate_field_comparisons
        << ",\"state_root_comparisons\":"
        << value.state_root_comparisons
        << ",\"recurrence_work_field_comparisons\":"
        << value.recurrence_work_field_comparisons
        << ",\"recurrence_root_comparisons\":"
        << value.recurrence_root_comparisons
        << ",\"hybrid_work_field_comparisons\":"
        << value.hybrid_work_field_comparisons
        << ",\"hybrid_root_comparisons\":"
        << value.hybrid_root_comparisons
        << ",\"producer_root_derivations\":"
        << value.producer_root_derivations << '}';
    return json.str();
}

std::string checker_work_json(const CheckerWork& value,
    const std::string& root, const std::string& comparison_root, bool exact) {
    std::ostringstream json;
    json << "{\"root\":\"" << root << "\",\"fixture_validations\":"
        << value.fixture_validations << ",\"identity_derivations\":"
        << value.identity_derivations << ",\"projected_scalars\":"
        << value.projected_scalars << ",\"exact_scalars\":"
        << value.exact_scalars << ",\"operator_products\":"
        << value.operator_products << ",\"tangent_kernel_calls\":"
        << value.tangent_kernel_calls << ",\"tangent_inner_dots\":"
        << value.tangent_inner_dots << ",\"tangent_outer_dots\":"
        << value.tangent_outer_dots << ",\"tangent_inner_terms\":"
        << value.tangent_inner_terms << ",\"tangent_outer_terms\":"
        << value.tangent_outer_terms << ",\"tangent_scale_products\":"
        << value.tangent_scale_products << ",\"input_components\":"
        << value.input_components << ",\"output_projections\":"
        << value.output_projections << ",\"output_additions\":"
        << value.output_additions << ",\"factor_payload_validations\":"
        << value.factor_payload_validations << ",\"factor_solves\":"
        << value.factor_solves << ",\"factor_terms\":"
        << value.factor_terms << ",\"factor_divisions\":"
        << value.factor_divisions << ",\"rho_dots\":" << value.rho_dots
        << ",\"denominator_dots\":" << value.denominator_dots
        << ",\"dot_terms\":" << value.dot_terms
        << ",\"positivity_checks\":" << value.positivity_checks
        << ",\"scalar_divisions\":" << value.scalar_divisions
        << ",\"solution_updates\":" << value.solution_updates
        << ",\"residual_updates\":" << value.residual_updates
        << ",\"direction_updates\":" << value.direction_updates
        << ",\"profile_payload_validations\":"
        << value.profile_payload_validations << ",\"certificate_checks\":"
        << value.certificate_checks << ",\"certificate_dots\":"
        << value.certificate_dots << ",\"certificate_dot_products\":"
        << value.certificate_dot_products << ",\"certificate_radius_terms\":"
        << value.certificate_radius_terms
        << ",\"certificate_solution_dots\":"
        << value.certificate_solution_dots
        << ",\"certificate_sign_comparisons\":"
        << value.certificate_sign_comparisons
        << ",\"root_derivation_paths\":" << value.root_derivation_paths
        << ",\"result_seals\":" << value.result_seals
        << ",\"comparison\":"
        << comparison_work_json(value.comparison, comparison_root)
        << ",\"exact\":" << (exact ? "true" : "false") << '}';
    return json.str();
}

std::string control_work_json(const ControlWork& value,
    const std::string& root, const std::string& comparison_root,
    const std::vector<bool>& controls, bool exact) {
    std::ostringstream json;
    json << "{\"root\":\"" << root << "\",\"controls\":"
        << value.controls << ",\"reseal_paths\":" << value.reseal_paths
        << ",\"root_derivation_paths\":" << value.root_derivation_paths
        << ",\"author_validations\":" << value.author_validations
        << ",\"author_fixture_validations\":"
        << value.author_fixture_validations
        << ",\"author_callback_replays\":"
        << value.author_callback_replays
        << ",\"author_tangent_kernel_calls\":"
        << value.author_tangent_kernel_calls
        << ",\"author_tangent_inner_dots\":"
        << value.author_tangent_inner_dots
        << ",\"author_tangent_outer_dots\":"
        << value.author_tangent_outer_dots
        << ",\"author_tangent_inner_terms\":"
        << value.author_tangent_inner_terms
        << ",\"author_tangent_outer_terms\":"
        << value.author_tangent_outer_terms
        << ",\"author_tangent_scale_products\":"
        << value.author_tangent_scale_products
        << ",\"author_input_components\":"
        << value.author_input_components
        << ",\"author_output_projections\":"
        << value.author_output_projections
        << ",\"author_output_additions\":"
        << value.author_output_additions
        << ",\"classifier_cases\":" << value.classifier_cases
        << ",\"result_seal_checks\":" << value.result_seal_checks
        << ",\"comparison\":"
        << comparison_work_json(value.comparison, comparison_root)
        << ",\"outcomes\":[";
    for (std::size_t index = 0U; index < controls.size(); ++index) {
        if (index != 0U) json << ',';
        json << (controls[index] ? "true" : "false");
    }
    json << "],\"exact\":" << (exact ? "true" : "false") << '}';
    return json.str();
}

std::string result_root(const std::string& route,
    const FormulaProbeParentFixture& fixture,
    const FormulaProbeHybridRecurrence& trace, const Replay& replay,
    const std::string& actual_producer_receipt_root,
    const std::string& checker_root,
    const std::string& controls_root) {
    std::ostringstream material;
    material << route << ':' << fixture.root << ':' << trace.root << ':'
        << replay.semantic_trace_root << ':'
        << replay.expected_producer_receipt_root << ':'
        << actual_producer_receipt_root << ':' << checker_root << ':'
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
        const FormulaProbeHybridIdentity identity =
            formula_probe_hybrid_identity(fixture);
        const bool fixture_exact = identity.exact
            && fixture.root == EXPECTED_FIXTURE_ROOT;
        const FormulaProbeHybridRecurrence untrusted =
            formula_probe_hybrid_twofold_recurrence(fixture);
        Replay replay = independent_replay(fixture, identity);
        if (!fixture_exact || !replay.exact
            || untrusted.recurrence.states.size() != 3U
            || untrusted.recurrence.products.size() != 3U
            || replay.expected.recurrence.states.size() != 3U
            || replay.expected.recurrence.products.size() != 3U) {
            const std::string route =
                "INDEPENDENT_CHECKER_APPARATUS_REJECTED";
            const std::string semantic = nextengine::nonlocal::sha256_hex(
                route + ":" + fixture.root + ":" + untrusted.root + ":"
                + replay.failure_stage);
            std::cout
                << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zk_independent_recurrence_checker.v2\""
                << ",\"status\":\"FAIL\",\"route\":\"" << route
                << "\",\"fixture_root\":\"" << fixture.root
                << "\",\"failure_stage\":\"" << replay.failure_stage
                << "\",\"timing_admitted\":false"
                << ",\"runtime_authority\":false"
                << ",\"gpu_authority\":false"
                << ",\"production_authority\":false"
                << ",\"result_sha256\":\"" << semantic << "\"}\n";
            return 1;
        }

        const std::string actual_recurrence_work_root =
            recurrence_work_root(untrusted.recurrence.work);
        const std::string actual_hybrid_work_root =
            hybrid_work_root(untrusted.hybrid_work);
        const std::string actual_producer_payload_root = producer_payload_root(
            untrusted.recurrence.work, untrusted.hybrid_work);
        const std::string actual_producer_receipt_root = producer_receipt_root(
            "actual", actual_producer_payload_root);
        replay.checker_work.root_derivation_paths += 4U;
        const Mismatch baseline = compare_trace(
            untrusted, replay, &replay.checker_work.comparison);

        ControlWork control_work;
        std::vector<bool> controls;
        controls.reserve(EXPECTED_CONTROL_COUNT);
        const auto compare_control = [&](const FormulaProbeHybridRecurrence& value,
                                         Mismatch expected) {
            controls.push_back(compare_trace(
                value, replay, &control_work.comparison) == expected);
        };
        const auto author_validate = [&](const FormulaProbeHybridRecurrence& value) {
            const bool valid =
                formula_probe_hybrid_twofold_recurrence_valid(fixture, value);
            record_author_validation_work(control_work, valid);
            return valid;
        };
        FormulaProbeHybridRecurrence mutation = untrusted;

        mutation.recurrence.states[2U].solution_components[0U] =
            std::nextafter(mutation.recurrence.states[2U]
                .solution_components[0U],
                std::numeric_limits<double>::infinity());
        mutation.recurrence.states[2U].root =
            legacy_state_root(mutation.recurrence.states[2U]);
        ++control_work.root_derivation_paths;
        reseal_outer(mutation, &control_work);
        controls.push_back(author_validate(mutation)
            && compare_trace(mutation, replay, &control_work.comparison)
                == Mismatch::State);

        mutation = untrusted;
        mutation.recurrence.states[2U].residual_components[0U] =
            std::nextafter(mutation.recurrence.states[2U]
                .residual_components[0U],
                std::numeric_limits<double>::infinity());
        compare_control(mutation, Mismatch::State);
        mutation = untrusted;
        mutation.recurrence.states[1U].direction_components[0U] =
            std::nextafter(mutation.recurrence.states[1U]
                .direction_components[0U],
                std::numeric_limits<double>::infinity());
        compare_control(mutation, Mismatch::State);
        mutation = untrusted;
        mutation.recurrence.states[1U].preconditioned_components[0U] =
            std::nextafter(mutation.recurrence.states[1U]
                .preconditioned_components[0U],
                std::numeric_limits<double>::infinity());
        compare_control(mutation, Mismatch::State);
        mutation = untrusted;
        mutation.recurrence.states[1U].exact = false;
        compare_control(mutation, Mismatch::State);
        mutation = untrusted;
        ++mutation.recurrence.states[1U].index;
        compare_control(mutation, Mismatch::State);
        mutation = untrusted;
        mutation.recurrence.states[1U].operator_root =
            nextengine::nonlocal::sha256_hex("mutated-operator");
        compare_control(mutation, Mismatch::Scalar);
        mutation = untrusted;
        mutation.recurrence.states[1U].solve_root =
            nextengine::nonlocal::sha256_hex("mutated-solve");
        compare_control(mutation, Mismatch::Scalar);

        const auto scalar_payload_control = [&](std::size_t selector) {
            FormulaProbeHybridRecurrence value = untrusted;
            FormulaProbeK2Scalar* scalar = nullptr;
            if (selector == 0U) scalar = &value.recurrence.states[1U].rho;
            else if (selector == 1U)
                scalar = &value.recurrence.states[1U].denominator;
            else if (selector == 2U)
                scalar = &value.recurrence.states[1U].alpha;
            else
                scalar = &value.recurrence.states[1U].beta;
            scalar->high = std::nextafter(
                scalar->high, std::numeric_limits<double>::infinity());
            controls.push_back(author_validate(value)
                && compare_trace(value, replay, &control_work.comparison)
                    == Mismatch::Scalar);
        };
        scalar_payload_control(0U);
        scalar_payload_control(1U);
        scalar_payload_control(2U);
        scalar_payload_control(3U);
        mutation = untrusted;
        mutation.recurrence.states[1U].rho.low = std::nextafter(
            mutation.recurrence.states[1U].rho.low,
            std::numeric_limits<double>::infinity());
        controls.push_back(author_validate(mutation)
            && compare_trace(mutation, replay, &control_work.comparison)
                == Mismatch::Scalar);
        mutation = untrusted;
        mutation.recurrence.states[1U].rho.exact = false;
        controls.push_back(author_validate(mutation)
            && compare_trace(mutation, replay, &control_work.comparison)
                == Mismatch::Scalar);

        mutation = untrusted;
        mutation.recurrence.states[1U].rho_root =
            nextengine::nonlocal::sha256_hex("mutated-rho-root");
        mutation.recurrence.states[1U].root =
            legacy_state_root(mutation.recurrence.states[1U]);
        ++control_work.root_derivation_paths;
        reseal_outer(mutation, &control_work);
        controls.push_back(author_validate(mutation)
            && compare_trace(mutation, replay, &control_work.comparison)
                == Mismatch::Scalar);
        mutation = untrusted;
        mutation.recurrence.states[1U].root =
            nextengine::nonlocal::sha256_hex("mutated-state-root");
        compare_control(mutation, Mismatch::State);

        mutation = untrusted;
        mutation.recurrence.products[0U].input_components[0U] =
            std::nextafter(mutation.recurrence.products[0U]
                .input_components[0U],
                std::numeric_limits<double>::infinity());
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].value_components[0U] =
            std::nextafter(mutation.recurrence.products[0U]
                .value_components[0U],
                std::numeric_limits<double>::infinity());
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].exact = false;
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].site = "mutated-site";
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        ++mutation.recurrence.products[0U].dimension;
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        ++mutation.recurrence.products[0U].entries;
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].artifact_root =
            nextengine::nonlocal::sha256_hex("mutated-artifact");
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].input_root =
            nextengine::nonlocal::sha256_hex("mutated-input-root");
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].value_root =
            nextengine::nonlocal::sha256_hex("mutated-value-root");
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].high_product_root =
            nextengine::nonlocal::sha256_hex("mutated-high-root");
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].low_product_root =
            nextengine::nonlocal::sha256_hex("mutated-low-root");
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].execution_root =
            nextengine::nonlocal::sha256_hex("mutated-execution-root");
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].callback_root =
            nextengine::nonlocal::sha256_hex("mutated-product-callback");
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.recurrence.products[0U].root =
            nextengine::nonlocal::sha256_hex("mutated-product-root");
        compare_control(mutation, Mismatch::Product);

        mutation = untrusted;
        mutation.exact = false;
        compare_control(mutation, Mismatch::RecurrenceGuard);
        mutation = untrusted;
        mutation.recurrence.exact = false;
        compare_control(mutation, Mismatch::RecurrenceGuard);
        mutation = untrusted;
        mutation.recurrence.complete = false;
        compare_control(mutation, Mismatch::RecurrenceGuard);
        mutation = untrusted;
        mutation.recurrence.positivity_exact = false;
        compare_control(mutation, Mismatch::RecurrenceGuard);
        mutation = untrusted;
        --mutation.recurrence.sealed_states;
        compare_control(mutation, Mismatch::RecurrenceGuard);
        mutation = untrusted;
        mutation.recurrence.failure_stage = "mutated-stage";
        compare_control(mutation, Mismatch::RecurrenceGuard);
        mutation = untrusted;
        mutation.recurrence.fixture_root =
            nextengine::nonlocal::sha256_hex("mutated-fixture");
        compare_control(mutation, Mismatch::CallbackIdentity);
        mutation = untrusted;
        mutation.recurrence.artifact_root =
            nextengine::nonlocal::sha256_hex("mutated-carrier");
        compare_control(mutation, Mismatch::CallbackIdentity);
        mutation = untrusted;
        mutation.recurrence.certificate_root =
            nextengine::nonlocal::sha256_hex("mutated-certificate-set");
        compare_control(mutation, Mismatch::Certificate);
        mutation = untrusted;
        mutation.recurrence.transaction_root =
            nextengine::nonlocal::sha256_hex("mutated-transaction");
        compare_control(mutation, Mismatch::RecurrenceGuard);
        mutation = untrusted;
        mutation.recurrence.root =
            nextengine::nonlocal::sha256_hex("mutated-recurrence-root");
        compare_control(mutation, Mismatch::RecurrenceWork);

        const auto certificate_control = [&](auto mutate_certificate) {
            FormulaProbeHybridRecurrence value = untrusted;
            const std::string original_root =
                value.recurrence.states[2U].certificate.root;
            mutate_certificate(value.recurrence.states[2U].certificate);
            if (value.recurrence.states[2U].certificate.root != original_root) {
                value.recurrence.states[2U].root =
                    legacy_state_root(value.recurrence.states[2U]);
                ++control_work.root_derivation_paths;
            }
            reseal_outer(value, &control_work);
            controls.push_back(author_validate(value)
                && compare_trace(value, replay, &control_work.comparison)
                    == Mismatch::Certificate);
        };
        certificate_control([](FormulaProbeCertificate& certificate) {
            certificate.exact = false;
        });
        certificate_control([](FormulaProbeCertificate& certificate) {
            certificate.passed = false;
        });
        certificate_control([](FormulaProbeCertificate& certificate) {
            ++certificate.positive;
        });
        certificate_control([](FormulaProbeCertificate& certificate) {
            ++certificate.negative;
        });
        certificate_control([](FormulaProbeCertificate& certificate) {
            ++certificate.unresolved;
        });
        certificate_control([](FormulaProbeCertificate& certificate) {
            certificate.error_upper = std::nextafter(
                certificate.error_upper,
                std::numeric_limits<double>::infinity());
        });
        certificate_control([](FormulaProbeCertificate& certificate) {
            certificate.solution_root =
                nextengine::nonlocal::sha256_hex("mutated-solution");
        });
        certificate_control([](FormulaProbeCertificate& certificate) {
            certificate.sign_root =
                nextengine::nonlocal::sha256_hex("mutated-sign");
        });
        certificate_control([](FormulaProbeCertificate& certificate) {
            certificate.root =
                nextengine::nonlocal::sha256_hex("mutated-certificate");
        });

        mutation = untrusted;
        ++mutation.recurrence.work.factor_solves;
        reseal_outer(mutation, &control_work);
        controls.push_back(author_validate(mutation)
            && compare_trace(mutation, replay, &control_work.comparison)
                == Mismatch::RecurrenceWork);
        mutation = untrusted;
        ++mutation.hybrid_work.output_projections;
        compare_control(mutation, Mismatch::HybridWork);
        mutation = untrusted;
        ++mutation.hybrid_work.certificate_dots;
        controls.push_back(author_validate(mutation)
            && compare_trace(mutation, replay, &control_work.comparison)
                == Mismatch::HybridWork);
        mutation = untrusted;
        mutation.callback_identity_root =
            nextengine::nonlocal::sha256_hex("mutated-callback");
        compare_control(mutation, Mismatch::CallbackIdentity);
        mutation = untrusted;
        mutation.callback_root =
            nextengine::nonlocal::sha256_hex("mutated-callback-root");
        compare_control(mutation, Mismatch::CallbackRoot);
        mutation = untrusted;
        mutation.recurrence.states.pop_back();
        compare_control(mutation, Mismatch::Apparatus);
        mutation = untrusted;
        mutation.recurrence.products.pop_back();
        compare_control(mutation, Mismatch::Apparatus);
        mutation = untrusted;
        mutation.recurrence.states[0U].solution_components[0U] =
            std::numeric_limits<double>::quiet_NaN();
        compare_control(mutation, Mismatch::State);
        mutation = untrusted;
        mutation.recurrence.states[0U].solution_components.pop_back();
        compare_control(mutation, Mismatch::State);
        mutation = untrusted;
        mutation.recurrence.products[0U].input_components.pop_back();
        compare_control(mutation, Mismatch::Product);
        mutation = untrusted;
        mutation.root = nextengine::nonlocal::sha256_hex("mutated-top-root");
        compare_control(mutation, Mismatch::HybridWork);

        const std::string seal_route =
            "HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE";
        control_work.root_derivation_paths += 6U;
        const std::string sealed_result = result_root(seal_route, fixture,
            untrusted, replay, actual_producer_receipt_root,
            "checker-seal", "control-seal");
        const bool result_sealed = sealed_result != result_root(
                "HYBRID_TRACE_STATE_REJECTED", fixture, untrusted, replay,
                actual_producer_receipt_root, "checker-seal", "control-seal")
            && sealed_result != result_root(seal_route, fixture, untrusted,
                replay, "mutated-actual-producer", "checker-seal",
                "control-seal")
            && sealed_result != result_root(seal_route, fixture, untrusted,
                replay, actual_producer_receipt_root, "mutated-checker",
                "control-seal")
            && sealed_result != result_root(seal_route, fixture, untrusted,
                replay, actual_producer_receipt_root, "checker-seal",
                "mutated-control")
            && sealed_result != nextengine::nonlocal::sha256_hex(
                "mutated-result");
        control_work.result_seal_checks = 5U;
        controls.push_back(result_sealed);

        const FormulaProbeK2Scalar positive = formula_probe_k2_exact_double(1.0);
        const FormulaProbeK2Scalar negative = formula_probe_k2_exact_double(-1.0);
        const FormulaProbeK2Scalar zero = formula_probe_k2_exact_double(0.0);
        controls.push_back(formula_probe_k2_positive(positive)
            && !formula_probe_k2_positive(negative)
            && !formula_probe_k2_positive(zero));
        const bool classifier_exact = classifier_control();
        control_work.classifier_cases = EXPECTED_CLASSIFIER_CASES;
        controls.push_back(classifier_exact);

        control_work.controls = controls.size();
        control_work.root_derivation_paths += 2U;
        bool controls_exact = controls.size() == EXPECTED_CONTROL_COUNT;
        for (bool passed : controls) controls_exact &= passed;
        controls_exact &= control_work_exact(control_work);
        const std::string control_comparison_root =
            comparison_work_root(control_work.comparison);
        const std::string controls_root =
            control_work_root(
                control_work, controls, control_comparison_root);

        replay.checker_work.result_seals = 1U;
        replay.checker_work.root_derivation_paths += 2U;
        const std::string checker_comparison_root =
            comparison_work_root(replay.checker_work.comparison);
        replay.checker_work_root = checker_work_root(
            replay.checker_work, checker_comparison_root);
        const bool replay_exact = baseline == Mismatch::None;
        const bool producer_work_exact = actual_recurrence_work_root
                == replay.expected_recurrence_work_root
            && actual_hybrid_work_root == replay.expected_hybrid_work_root
            && actual_producer_payload_root
                == replay.expected_producer_payload_root;
        const bool checker_ledger_exact =
            checker_schedule_work_exact(replay.checker_work);
        const bool control_ledger_exact = control_work_exact(control_work);
        const bool work_exact = replay.exact && producer_work_exact
            && checker_ledger_exact && control_ledger_exact
            && !replay.expected_producer_receipt_root.empty()
            && !actual_producer_receipt_root.empty()
            && !replay.checker_work_root.empty() && !controls_root.empty();
        const bool ladder_exact = ladder(replay);
        const std::string route = classify(
            baseline, work_exact, controls_exact, ladder_exact);
        const bool exact = route
            == "HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE";
        const std::string semantic = result_root(route, fixture, untrusted,
            replay, actual_producer_receipt_root,
            replay.checker_work_root, controls_root);

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zk_independent_recurrence_checker.v2\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"untrusted_trace_root\":\"" << untrusted.root << "\""
            << ",\"independent_trace_root\":\""
            << replay.semantic_trace_root << "\""
            << ",\"callback_identity_root\":\""
            << replay.expected.callback_identity_root << "\""
            << ",\"checker_identity_root\":\"" << replay.identity.root
            << "\""
            << ",\"callback_root\":\""
            << replay.expected.callback_root << "\""
            << ",\"trace_match\":" << (replay_exact ? "true" : "false")
            << ",\"positivity_checks\":"
            << replay.checker_work.positivity_checks
            << ",\"scalar_components_compared\":"
            << replay.checker_work.comparison.scalar_component_comparisons
            << ",\"ladder\":" << (ladder_exact ? "true" : "false")
            << ",\"state2\":{\"positive\":"
            << replay.expected.recurrence.states[2U].certificate.positive
            << ",\"negative\":"
            << replay.expected.recurrence.states[2U].certificate.negative
            << ",\"unresolved\":"
            << replay.expected.recurrence.states[2U].certificate.unresolved
            << ",\"sign_root\":\""
            << replay.expected.recurrence.states[2U].certificate.sign_root
            << "\"}"
            << ",\"producer_work\":{\"expected\":{\"recurrence\":"
            << recurrence_work_json(replay.expected.recurrence.work,
                replay.expected_recurrence_work_root)
            << ",\"hybrid\":"
            << hybrid_work_json(replay.expected.hybrid_work,
                replay.expected_hybrid_work_root)
            << ",\"payload_root\":\""
            << replay.expected_producer_payload_root
            << "\",\"receipt_root\":\""
            << replay.expected_producer_receipt_root
            << "\"},\"actual\":{\"recurrence\":"
            << recurrence_work_json(untrusted.recurrence.work,
                actual_recurrence_work_root)
            << ",\"hybrid\":"
            << hybrid_work_json(untrusted.hybrid_work,
                actual_hybrid_work_root)
            << ",\"payload_root\":\"" << actual_producer_payload_root
            << "\",\"receipt_root\":\"" << actual_producer_receipt_root
            << "\"},\"exact\":"
            << (producer_work_exact ? "true" : "false") << "}"
            << ",\"checker_work\":"
            << checker_work_json(replay.checker_work,
                replay.checker_work_root, checker_comparison_root,
                checker_ledger_exact)
            << ",\"control_work\":"
            << control_work_json(control_work, controls_root,
                control_comparison_root, controls,
                controls_exact && control_ledger_exact)
            << ",\"claim_status\":\"AUTHOR_REPAIR_PASS_REVIEW_NEXT\""
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << semantic << "\"}\n";
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "independent recurrence checker failed: "
            << error.what() << '\n';
        const std::string route =
            "INDEPENDENT_CHECKER_APPARATUS_REJECTED";
        const std::string semantic = nextengine::nonlocal::sha256_hex(
            route + ":cache_read");
        std::cout
            << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zk_independent_recurrence_checker.v2\""
            << ",\"status\":\"FAIL\",\"route\":\"" << route << "\""
            << ",\"failure_stage\":\"cache_read\""
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << semantic << "\"}\n";
        return 1;
    }
}
