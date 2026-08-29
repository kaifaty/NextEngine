#include "formula_probe_cache.hpp"
#include "sha256.hpp"

#include <array>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <filesystem>
#include <fstream>
#include <functional>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <unistd.h>
#include <utility>
#include <vector>

namespace {

using namespace nextengine::nonlocal::fcr;

constexpr std::size_t DIMENSION = 102U;
constexpr std::size_t TANGENT_COLUMNS = 315U;
constexpr std::size_t TANGENT_COMPONENTS = 32130U;
constexpr std::size_t SELECTED_COMPONENTS = 32742U;
constexpr std::size_t INPUTS = 6U;
constexpr const char* EXPECTED_TANGENT_ROOT =
    "114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1";
constexpr const char* EXPECTED_PROJECTED_RHS_ROOT =
    "0df32db6fb6c7b3a6fb5e3760c7ff810f92e31baa1544a0911a8e0b6a11274ef";
constexpr const char* EXPECTED_ORIGINAL_RHS_ROOT =
    "64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1";

struct ProductTotals {
    std::string role;
    std::size_t payload_validations = 0U;
    std::size_t predicate_checks = 0U;
    std::size_t tangent_payload_hashes = 0U;
    std::size_t frozen_scalar_parses = 0U;
    std::size_t input_guard_checks = 0U;
    std::size_t input_payload_hashes = 0U;
    std::size_t input_identity_checks = 0U;
    std::size_t kernel_guard_checks = 0U;
    std::size_t kernel_calls = 0U;
    std::size_t inner_dots = 0U;
    std::size_t outer_dots = 0U;
    std::size_t inner_terms = 0U;
    std::size_t outer_terms = 0U;
    std::size_t scale_products = 0U;
    std::size_t kernel_buffer_allocations = 0U;
    std::size_t temporary_operand_allocations = 0U;
    std::size_t operand_components_copied = 0U;
    std::size_t kernel_root_derivations = 0U;
    std::size_t dot_guard_checks = 0U;
    std::size_t dot_root_derivations = 0U;
    std::size_t receipt_root_derivations = 0U;
    std::size_t result_root_derivations = 0U;
    std::string root;
};

struct CheckerWork {
    std::size_t read_semantic_comparisons = 0U;
    std::size_t read_work_comparisons = 0U;
    std::size_t admission_semantic_comparisons = 0U;
    std::size_t admission_work_comparisons = 0U;
    std::size_t product_guard_comparisons = 0U;
    std::size_t product_component_comparisons = 0U;
    std::size_t product_work_comparisons = 0U;
    std::size_t product_root_comparisons = 0U;
    std::size_t aggregate_work_comparisons = 0U;
    std::size_t control_receipt_comparisons = 0U;
    std::size_t schedule_field_comparisons = 0U;
    std::size_t root_derivation_paths = 0U;
    std::size_t result_seals = 0U;
    std::string root;
};

struct ControlReceipt {
    std::string name;
    std::string expected_route;
    std::string observed_route;
    std::string expected_stage;
    std::string observed_stage;
    std::string mutation_digest;
    std::string child_root;
    bool passed = false;
    std::size_t comparisons = 0U;
    std::size_t admission_predicate_checks = 0U;
    std::size_t tangent_payload_hashes = 0U;
    std::size_t frozen_scalar_parses = 0U;
    std::size_t selected_scalar_fields_copied = 0U;
    std::size_t selected_string_bytes_copied = 0U;
    std::size_t selected_vector_allocations = 0U;
    std::size_t selected_components_copied = 0U;
    std::size_t temporary_components_allocated = 0U;
    std::size_t temporary_bytes_allocated = 0U;
    std::size_t serialized_bytes_read = 0U;
    std::size_t serialized_bytes_written = 0U;
    std::size_t root_derivation_paths = 0U;
    std::string root;
};

struct ControlWork {
    std::size_t controls = 0U;
    std::size_t passed_controls = 0U;
    std::size_t comparisons = 0U;
    std::size_t admission_predicate_checks = 0U;
    std::size_t tangent_payload_hashes = 0U;
    std::size_t frozen_scalar_parses = 0U;
    std::size_t selected_scalar_fields_copied = 0U;
    std::size_t selected_string_bytes_copied = 0U;
    std::size_t selected_vector_allocations = 0U;
    std::size_t selected_components_copied = 0U;
    std::size_t temporary_components_allocated = 0U;
    std::size_t temporary_bytes_allocated = 0U;
    std::size_t serialized_bytes_read = 0U;
    std::size_t serialized_bytes_written = 0U;
    std::size_t classifier_cases = 0U;
    std::size_t root_derivation_paths = 0U;
    std::vector<ControlReceipt> receipts;
    std::vector<std::string> receipt_roots;
    std::string root;
};

enum class RouteInput {
    None,
    Admission,
    Product,
    Work,
    Control,
    Seal,
    Unknown,
};

struct BoundaryResult {
    bool read_exact = false;
    bool admission_exact = false;
    bool products_exact = false;
    bool work_exact = false;
    bool checker_exact = false;
    bool controls_exact = false;
    bool child_seals_exact = false;
    RouteInput route_input = RouteInput::Unknown;
    std::string selected_route;
    std::string read_root;
    std::string admission_root;
    std::string candidate_root;
    std::string reference_root;
    std::string checker_root;
    std::string controls_root;
    std::vector<std::string> admitted_roots;
    std::vector<std::string> legacy_roots;
    std::string root;
};

struct InputView {
    FormulaProbeTangentInputRole role = FormulaProbeTangentInputRole::Unknown;
    std::reference_wrapper<const std::vector<FormulaProbeBinary128>> values;
};

std::string route(RouteInput input) {
    switch (input) {
        case RouteInput::None:
            return "ADMITTED_TANGENT_WORK_BOUNDARY_CANDIDATE";
        case RouteInput::Admission:
            return "ADMITTED_TANGENT_ADMISSION_REJECTED";
        case RouteInput::Product:
            return "ADMITTED_TANGENT_PRODUCT_REJECTED";
        case RouteInput::Work:
            return "ADMITTED_TANGENT_WORK_REJECTED";
        case RouteInput::Control:
        case RouteInput::Seal:
        case RouteInput::Unknown:
            return "INCONCLUSIVE";
    }
    return "INCONCLUSIVE";
}

RouteInput classify(bool read_exact, bool admission_exact,
    bool products_exact, bool work_exact, bool checker_exact,
    bool controls_exact, bool child_seals_exact) {
    if (!read_exact || !admission_exact) return RouteInput::Admission;
    if (!products_exact) return RouteInput::Product;
    if (!work_exact || !checker_exact) return RouteInput::Work;
    if (!controls_exact) return RouteInput::Control;
    if (!child_seals_exact) return RouteInput::Seal;
    return RouteInput::None;
}

const char* role_name(FormulaProbeTangentInputRole role) {
    switch (role) {
        case FormulaProbeTangentInputRole::ProjectedRhs:
            return "projected_rhs";
        case FormulaProbeTangentInputRole::OriginalRhs:
            return "original_rhs";
        case FormulaProbeTangentInputRole::Baseline0:
            return "baseline_0";
        case FormulaProbeTangentInputRole::Baseline1:
            return "baseline_1";
        case FormulaProbeTangentInputRole::Baseline2:
            return "baseline_2";
        case FormulaProbeTangentInputRole::CommonProjected2:
            return "common_projected_2";
        case FormulaProbeTangentInputRole::Unknown:
            return "unknown";
    }
    return "unknown";
}

bool compare(bool value, std::size_t& counter) {
    ++counter;
    return value;
}

std::string derive_root(const std::string& material, std::size_t& counter) {
    ++counter;
    return nextengine::nonlocal::sha256_hex(material);
}

std::string read_work_root(const FormulaProbeTangentBoundaryReadWork& work,
    std::size_t& counter) {
    std::ostringstream material;
    material << work.header_predicate_checks << ':'
        << work.projection_predicate_checks << ':'
        << work.bounded_length_reads << ':' << work.selected_size_fields << ':'
        << work.selected_scalar_fields << ':' << work.selected_string_fields
        << ':' << work.selected_string_bytes << ':'
        << work.selected_vector_fields << ':'
        << work.selected_vector_allocations << ':'
        << work.selected_vector_components << ':'
        << work.skipped_boolean_fields << ':' << work.skipped_scalar_fields
        << ':' << work.skipped_string_fields << ':'
        << work.skipped_vector_fields << ':'
        << work.skipped_collection_elements << ':'
        << work.skipped_payload_bytes << ':' << work.eof_checks << ':'
        << work.receipt_root_derivations << ':'
        << work.result_root_derivations;
    return derive_root(material.str(), counter);
}

std::string read_result_root(const FormulaProbeTangentBoundaryRead& value,
    std::size_t& counter) {
    std::ostringstream material;
    material << value.exact << ':' << value.failure_stage << ':';
    if (value.fixture.has_value()) {
        material << value.fixture->dimension << ':'
            << value.fixture->tangent_columns << ':'
            << value.fixture->tangent_root << ':'
            << value.fixture->projected_rhs_root << ':'
            << value.fixture->original_rhs_root;
    }
    material << ':' << value.work.root;
    return derive_root(material.str(), counter);
}

std::string admission_work_root(
    const FormulaProbeTangentAdmissionWork& work, std::size_t& counter) {
    std::ostringstream material;
    material << work.predicate_checks << ':'
        << work.tangent_payload_hashes << ':'
        << work.frozen_scalar_parses << ':'
        << work.tangent_components_copied << ':'
        << work.context_root_derivations << ':'
        << work.receipt_root_derivations << ':'
        << work.result_root_derivations;
    return derive_root(material.str(), counter);
}

std::string admission_result_root(const FormulaProbeTangentAdmission& value,
    std::size_t& counter) {
    std::ostringstream material;
    material << value.exact() << ':' << value.failure_stage() << ':';
    if (value.has_context()) material << value.context().context_root();
    material << ':' << value.work().root;
    return derive_root(material.str(), counter);
}

std::string context_root(const FormulaProbeAdmittedTangent& value,
    std::size_t& counter) {
    std::ostringstream material;
    material << "nextengine.nonlocal.r63zl.admitted_tangent.v1|"
        << value.dimension() << ':' << value.tangent_columns() << ':'
        << value.tangent_root();
    return derive_root(material.str(), counter);
}

std::string admitted_product_work_root(
    const FormulaProbeAdmittedProductWork& work, std::size_t& counter) {
    std::ostringstream material;
    material << work.input_guard_checks << ':' << work.input_payload_hashes
        << ':' << work.input_identity_checks << ':' << work.kernel_calls << ':'
        << work.inner_dots << ':' << work.outer_dots << ':'
        << work.inner_terms << ':' << work.outer_terms << ':'
        << work.scale_products << ':' << work.kernel_buffer_allocations << ':'
        << work.temporary_operand_allocations << ':'
        << work.operand_components_copied << ':'
        << work.kernel_root_derivations << ':'
        << work.receipt_root_derivations << ':'
        << work.result_root_derivations;
    return derive_root(material.str(), counter);
}

std::string admitted_product_result_root(
    const FormulaProbeAdmittedProduct& value, std::size_t& counter) {
    std::ostringstream material;
    material << value.exact() << ':' << value.failure_stage() << ':'
        << value.context_root() << ':' << role_name(value.input_role()) << ':'
        << value.input_root() << ':' << value.product().root << ':'
        << value.work().root;
    return derive_root(material.str(), counter);
}

std::string product_totals_root(const ProductTotals& value,
    std::size_t& counter) {
    std::ostringstream material;
    material << value.role << ':' << value.payload_validations << ':'
        << value.predicate_checks << ':' << value.tangent_payload_hashes << ':'
        << value.frozen_scalar_parses << ':' << value.input_guard_checks << ':'
        << value.input_payload_hashes << ':' << value.input_identity_checks
        << ':' << value.kernel_guard_checks << ':' << value.kernel_calls << ':'
        << value.inner_dots << ':' << value.outer_dots << ':'
        << value.inner_terms << ':' << value.outer_terms << ':'
        << value.scale_products << ':' << value.kernel_buffer_allocations << ':'
        << value.temporary_operand_allocations << ':'
        << value.operand_components_copied << ':'
        << value.kernel_root_derivations << ':' << value.dot_guard_checks << ':'
        << value.dot_root_derivations << ':'
        << value.receipt_root_derivations << ':'
        << value.result_root_derivations;
    return derive_root(material.str(), counter);
}

std::string checker_work_root(const CheckerWork& value) {
    std::ostringstream material;
    material << value.read_semantic_comparisons << ':'
        << value.read_work_comparisons << ':'
        << value.admission_semantic_comparisons << ':'
        << value.admission_work_comparisons << ':'
        << value.product_guard_comparisons << ':'
        << value.product_component_comparisons << ':'
        << value.product_work_comparisons << ':'
        << value.product_root_comparisons << ':'
        << value.aggregate_work_comparisons << ':'
        << value.control_receipt_comparisons << ':'
        << value.schedule_field_comparisons << ':'
        << value.root_derivation_paths << ':' << value.result_seals;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string control_receipt_root(const ControlReceipt& value) {
    std::ostringstream material;
    material << value.name << ':' << value.expected_route << ':'
        << value.observed_route << ':' << value.expected_stage << ':'
        << value.observed_stage << ':' << value.mutation_digest << ':'
        << value.child_root << ':' << value.passed << ':' << value.comparisons
        << ':' << value.admission_predicate_checks << ':'
        << value.tangent_payload_hashes << ':'
        << value.frozen_scalar_parses << ':'
        << value.selected_scalar_fields_copied << ':'
        << value.selected_string_bytes_copied << ':'
        << value.selected_vector_allocations << ':'
        << value.selected_components_copied << ':'
        << value.temporary_components_allocated << ':'
        << value.temporary_bytes_allocated << ':'
        << value.serialized_bytes_read << ':'
        << value.serialized_bytes_written << ':'
        << value.root_derivation_paths;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string control_work_root(const ControlWork& value) {
    std::ostringstream material;
    material << value.controls << ':' << value.passed_controls << ':'
        << value.comparisons << ':'
        << value.admission_predicate_checks << ':'
        << value.tangent_payload_hashes << ':'
        << value.frozen_scalar_parses << ':'
        << value.selected_scalar_fields_copied << ':'
        << value.selected_string_bytes_copied << ':'
        << value.selected_vector_allocations << ':'
        << value.selected_components_copied << ':'
        << value.temporary_components_allocated << ':'
        << value.temporary_bytes_allocated << ':'
        << value.serialized_bytes_read << ':'
        << value.serialized_bytes_written << ':'
        << value.classifier_cases << ':' << value.root_derivation_paths << '|';
    for (const std::string& root : value.receipt_roots)
        material << root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string boundary_result_root(const BoundaryResult& value) {
    std::ostringstream material;
    material << "nextengine.nonlocal.r63zl.v2|" << value.read_exact << ':'
        << value.admission_exact << ':' << value.products_exact << ':'
        << value.work_exact << ':' << value.checker_exact << ':'
        << value.controls_exact << ':' << value.child_seals_exact << ':'
        << static_cast<int>(value.route_input) << ':' << value.selected_route
        << ':' << value.read_root << ':' << value.admission_root << ':'
        << value.candidate_root << ':' << value.reference_root << ':'
        << value.checker_root << ':' << value.controls_root << '|';
    for (const std::string& root : value.admitted_roots)
        material << root << ';';
    material << '|';
    for (const std::string& root : value.legacy_roots)
        material << root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

BoundaryResult make_boundary_result(bool read_exact, bool admission_exact,
    bool products_exact, bool work_exact, bool checker_exact,
    bool controls_exact, bool child_seals_exact, const std::string& read_root,
    const std::string& admission_root, const std::string& candidate_root,
    const std::string& reference_root, const std::string& checker_root,
    const std::string& controls_root,
    const std::vector<std::string>& admitted_roots,
    const std::vector<std::string>& legacy_roots) {
    BoundaryResult result;
    result.read_exact = read_exact;
    result.admission_exact = admission_exact;
    result.products_exact = products_exact;
    result.work_exact = work_exact;
    result.checker_exact = checker_exact;
    result.controls_exact = controls_exact;
    result.child_seals_exact = child_seals_exact;
    result.route_input = classify(read_exact, admission_exact, products_exact,
        work_exact, checker_exact, controls_exact, child_seals_exact);
    result.selected_route = route(result.route_input);
    result.read_root = read_root;
    result.admission_root = admission_root;
    result.candidate_root = candidate_root;
    result.reference_root = reference_root;
    result.checker_root = checker_root;
    result.controls_root = controls_root;
    result.admitted_roots = admitted_roots;
    result.legacy_roots = legacy_roots;
    result.root = boundary_result_root(result);
    return result;
}

bool boundary_result_valid(const BoundaryResult& value,
    const std::string& expected_read_root,
    const std::string& expected_admission_root,
    const std::string& expected_candidate_root,
    const std::string& expected_reference_root,
    const std::string& expected_checker_root,
    const std::string& expected_controls_root,
    const std::vector<std::string>& expected_admitted_roots,
    const std::vector<std::string>& expected_legacy_roots) {
    const RouteInput expected = classify(value.read_exact,
        value.admission_exact, value.products_exact, value.work_exact,
        value.checker_exact, value.controls_exact, value.child_seals_exact);
    return value.route_input == expected
        && value.selected_route == route(expected)
        && value.read_root == expected_read_root
        && value.admission_root == expected_admission_root
        && value.candidate_root == expected_candidate_root
        && value.reference_root == expected_reference_root
        && value.checker_root == expected_checker_root
        && value.controls_root == expected_controls_root
        && value.admitted_roots == expected_admitted_roots
        && value.legacy_roots == expected_legacy_roots
        && value.admitted_roots.size() == value.legacy_roots.size()
        && value.root == boundary_result_root(value);
}

void add_candidate(ProductTotals& totals,
    const FormulaProbeAdmittedProductWork& work) {
    totals.input_guard_checks += work.input_guard_checks;
    totals.input_payload_hashes += work.input_payload_hashes;
    totals.input_identity_checks += work.input_identity_checks;
    totals.kernel_calls += work.kernel_calls;
    totals.inner_dots += work.inner_dots;
    totals.outer_dots += work.outer_dots;
    totals.inner_terms += work.inner_terms;
    totals.outer_terms += work.outer_terms;
    totals.scale_products += work.scale_products;
    totals.kernel_buffer_allocations += work.kernel_buffer_allocations;
    totals.temporary_operand_allocations +=
        work.temporary_operand_allocations;
    totals.operand_components_copied += work.operand_components_copied;
    totals.kernel_root_derivations += work.kernel_root_derivations;
    totals.receipt_root_derivations += work.receipt_root_derivations;
    totals.result_root_derivations += work.result_root_derivations;
}

void add_reference(ProductTotals& totals, const FormulaProbeProduct& product) {
    ++totals.payload_validations;
    totals.predicate_checks += 10U;
    ++totals.tangent_payload_hashes;
    totals.frozen_scalar_parses += 2U;
    ++totals.input_guard_checks;
    totals.kernel_guard_checks += 6U;
    ++totals.kernel_calls;
    totals.inner_dots += product.inner_dots;
    totals.outer_dots += product.outer_dots;
    totals.inner_terms += product.inner_terms;
    totals.outer_terms += product.outer_terms;
    totals.scale_products += product.scale_products;
    totals.kernel_buffer_allocations += product.kernel_buffer_allocations;
    totals.temporary_operand_allocations +=
        product.temporary_operand_allocations;
    totals.operand_components_copied += product.operand_components_copied;
    totals.kernel_root_derivations += 5U;
    totals.dot_guard_checks += 2U * (TANGENT_COLUMNS + DIMENSION);
    totals.dot_root_derivations += 3U * (TANGENT_COLUMNS + DIMENSION);
}

bool valid_read_work(const FormulaProbeTangentBoundaryReadWork& work,
    CheckerWork& checker) {
    return compare(work.header_predicate_checks == 12U,
               checker.read_work_comparisons)
        && compare(work.projection_predicate_checks == 2U,
            checker.read_work_comparisons)
        && compare(work.bounded_length_reads == 194U,
            checker.read_work_comparisons)
        && compare(work.selected_size_fields == 2U,
            checker.read_work_comparisons)
        && compare(work.selected_scalar_fields == 3U,
            checker.read_work_comparisons)
        && compare(work.selected_string_fields == 3U,
            checker.read_work_comparisons)
        && compare(work.selected_string_bytes == 192U,
            checker.read_work_comparisons)
        && compare(work.selected_vector_fields == 7U,
            checker.read_work_comparisons)
        && compare(work.selected_vector_allocations == 7U,
            checker.read_work_comparisons)
        && compare(work.selected_vector_components == SELECTED_COMPONENTS,
            checker.read_work_comparisons)
        && compare(work.skipped_boolean_fields == 16U,
            checker.read_work_comparisons)
        && compare(work.skipped_scalar_fields == 135U,
            checker.read_work_comparisons)
        && compare(work.skipped_string_fields == 43U,
            checker.read_work_comparisons)
        && compare(work.skipped_vector_fields == 9U,
            checker.read_work_comparisons)
        && compare(work.skipped_collection_elements == 8U,
            checker.read_work_comparisons)
        && compare(work.skipped_payload_bytes == 507849U,
            checker.read_work_comparisons)
        && compare(work.eof_checks == 1U, checker.read_work_comparisons)
        && compare(work.receipt_root_derivations == 1U,
            checker.read_work_comparisons)
        && compare(work.result_root_derivations == 1U,
            checker.read_work_comparisons);
}

bool valid_admission_work(const FormulaProbeTangentAdmissionWork& work,
    CheckerWork& checker) {
    return compare(work.predicate_checks == 10U,
               checker.admission_work_comparisons)
        && compare(work.tangent_payload_hashes == 1U,
            checker.admission_work_comparisons)
        && compare(work.frozen_scalar_parses == 2U,
            checker.admission_work_comparisons)
        && compare(work.tangent_components_copied == TANGENT_COMPONENTS,
            checker.admission_work_comparisons)
        && compare(work.context_root_derivations == 1U,
            checker.admission_work_comparisons)
        && compare(work.receipt_root_derivations == 1U,
            checker.admission_work_comparisons)
        && compare(work.result_root_derivations == 1U,
            checker.admission_work_comparisons);
}

bool valid_product_work(const FormulaProbeAdmittedProductWork& work,
    CheckerWork& checker) {
    return compare(work.input_guard_checks == 1U,
               checker.product_work_comparisons)
        && compare(work.input_payload_hashes == 1U,
            checker.product_work_comparisons)
        && compare(work.input_identity_checks == 1U,
            checker.product_work_comparisons)
        && compare(work.kernel_calls == 1U,
            checker.product_work_comparisons)
        && compare(work.inner_dots == TANGENT_COLUMNS,
            checker.product_work_comparisons)
        && compare(work.outer_dots == DIMENSION,
            checker.product_work_comparisons)
        && compare(work.inner_terms == TANGENT_COMPONENTS,
            checker.product_work_comparisons)
        && compare(work.outer_terms == TANGENT_COMPONENTS,
            checker.product_work_comparisons)
        && compare(work.scale_products == DIMENSION,
            checker.product_work_comparisons)
        && compare(work.kernel_buffer_allocations == 4U,
            checker.product_work_comparisons)
        && compare(work.temporary_operand_allocations == 0U,
            checker.product_work_comparisons)
        && compare(work.operand_components_copied == 0U,
            checker.product_work_comparisons)
        && compare(work.kernel_root_derivations == 5U,
            checker.product_work_comparisons)
        && compare(work.receipt_root_derivations == 1U,
            checker.product_work_comparisons)
        && compare(work.result_root_derivations == 1U,
            checker.product_work_comparisons);
}

bool exact_candidate_totals(const ProductTotals& value,
    std::size_t& comparisons) {
    return compare(value.role == "candidate", comparisons)
        && compare(value.payload_validations == 0U, comparisons)
        && compare(value.predicate_checks == 0U, comparisons)
        && compare(value.tangent_payload_hashes == 0U, comparisons)
        && compare(value.frozen_scalar_parses == 0U, comparisons)
        && compare(value.input_guard_checks == 6U, comparisons)
        && compare(value.input_payload_hashes == 6U, comparisons)
        && compare(value.input_identity_checks == 6U, comparisons)
        && compare(value.kernel_guard_checks == 0U, comparisons)
        && compare(value.kernel_calls == 6U, comparisons)
        && compare(value.inner_dots == 1890U, comparisons)
        && compare(value.outer_dots == 612U, comparisons)
        && compare(value.inner_terms == 192780U, comparisons)
        && compare(value.outer_terms == 192780U, comparisons)
        && compare(value.scale_products == 612U, comparisons)
        && compare(value.kernel_buffer_allocations == 24U, comparisons)
        && compare(value.temporary_operand_allocations == 0U, comparisons)
        && compare(value.operand_components_copied == 0U, comparisons)
        && compare(value.kernel_root_derivations == 30U, comparisons)
        && compare(value.dot_guard_checks == 0U, comparisons)
        && compare(value.dot_root_derivations == 0U, comparisons)
        && compare(value.receipt_root_derivations == 6U, comparisons)
        && compare(value.result_root_derivations == 6U, comparisons);
}

bool exact_reference_totals(const ProductTotals& value,
    std::size_t& comparisons) {
    return compare(value.role == "reference", comparisons)
        && compare(value.payload_validations == 6U, comparisons)
        && compare(value.predicate_checks == 60U, comparisons)
        && compare(value.tangent_payload_hashes == 6U, comparisons)
        && compare(value.frozen_scalar_parses == 12U, comparisons)
        && compare(value.input_guard_checks == 6U, comparisons)
        && compare(value.input_payload_hashes == 0U, comparisons)
        && compare(value.input_identity_checks == 0U, comparisons)
        && compare(value.kernel_guard_checks == 36U, comparisons)
        && compare(value.kernel_calls == 6U, comparisons)
        && compare(value.inner_dots == 1890U, comparisons)
        && compare(value.outer_dots == 612U, comparisons)
        && compare(value.inner_terms == 192780U, comparisons)
        && compare(value.outer_terms == 192780U, comparisons)
        && compare(value.scale_products == 612U, comparisons)
        && compare(value.kernel_buffer_allocations == 24U, comparisons)
        && compare(value.temporary_operand_allocations == 2502U, comparisons)
        && compare(value.operand_components_copied == 385560U, comparisons)
        && compare(value.kernel_root_derivations == 30U, comparisons)
        && compare(value.dot_guard_checks == 5004U, comparisons)
        && compare(value.dot_root_derivations == 7506U, comparisons)
        && compare(value.receipt_root_derivations == 0U, comparisons)
        && compare(value.result_root_derivations == 0U, comparisons);
}

bool control_check(bool value, ControlReceipt& receipt, ControlWork& work) {
    ++receipt.comparisons;
    ++work.comparisons;
    return value;
}

void finish_control(ControlReceipt receipt, ControlWork& work) {
    const bool route_exact = control_check(
        receipt.expected_route == receipt.observed_route, receipt, work);
    const bool stage_exact = control_check(
        receipt.expected_stage == receipt.observed_stage, receipt, work);
    receipt.passed = receipt.passed && route_exact && stage_exact;
    ++receipt.root_derivation_paths;
    receipt.root = control_receipt_root(receipt);
    ++work.controls;
    if (receipt.passed) ++work.passed_controls;
    work.admission_predicate_checks += receipt.admission_predicate_checks;
    work.tangent_payload_hashes += receipt.tangent_payload_hashes;
    work.frozen_scalar_parses += receipt.frozen_scalar_parses;
    work.selected_scalar_fields_copied +=
        receipt.selected_scalar_fields_copied;
    work.selected_string_bytes_copied +=
        receipt.selected_string_bytes_copied;
    work.selected_vector_allocations +=
        receipt.selected_vector_allocations;
    work.selected_components_copied += receipt.selected_components_copied;
    work.temporary_components_allocated +=
        receipt.temporary_components_allocated;
    work.temporary_bytes_allocated += receipt.temporary_bytes_allocated;
    work.serialized_bytes_read += receipt.serialized_bytes_read;
    work.serialized_bytes_written += receipt.serialized_bytes_written;
    work.root_derivation_paths += receipt.root_derivation_paths;
    work.receipts.push_back(receipt);
    work.receipt_roots.push_back(receipt.root);
}

void admission_control(const std::string& name,
    const FormulaProbeTangentAdmission& value, const std::string& stage,
    std::size_t predicates, std::size_t payload_hashes,
    const std::string& mutation_material,
    std::size_t mutation_payload_hashes, ControlWork& work) {
    ControlReceipt receipt;
    receipt.name = name;
    receipt.expected_route = route(RouteInput::Admission);
    receipt.observed_route = value.exact()
        ? route(RouteInput::None) : route(RouteInput::Admission);
    receipt.expected_stage = stage;
    receipt.observed_stage = value.failure_stage();
    receipt.child_root = value.root();
    receipt.mutation_digest = nextengine::nonlocal::sha256_hex(
        name + ":" + stage + ":" + mutation_material);
    ++receipt.root_derivation_paths;
    receipt.root_derivation_paths += mutation_payload_hashes;
    receipt.admission_predicate_checks = value.work().predicate_checks;
    receipt.tangent_payload_hashes = value.work().tangent_payload_hashes
        + mutation_payload_hashes;
    receipt.frozen_scalar_parses = value.work().frozen_scalar_parses;
    bool exact = true;
    exact = control_check(!value.exact(), receipt, work) && exact;
    exact = control_check(!value.has_context(), receipt, work) && exact;
    exact = control_check(value.failure_stage() == stage, receipt, work)
        && exact;
    exact = control_check(value.work().predicate_checks == predicates,
        receipt, work) && exact;
    exact = control_check(value.work().tangent_payload_hashes == payload_hashes,
        receipt, work) && exact;
    exact = control_check(value.work().tangent_components_copied == 0U,
        receipt, work) && exact;
    exact = control_check(value.work().context_root_derivations == 0U,
        receipt, work) && exact;
    std::size_t roots = 0U;
    exact = control_check(value.work().root
            == admission_work_root(value.work(), roots), receipt, work)
        && exact;
    exact = control_check(value.root()
            == admission_result_root(value, roots), receipt, work)
        && exact;
    receipt.root_derivation_paths += roots;
    receipt.passed = exact;
    finish_control(std::move(receipt), work);
}

bool controls_valid(const ControlWork& work, CheckerWork& checker) {
    bool exact = compare(work.controls == work.receipt_roots.size(),
        checker.control_receipt_comparisons);
    exact = compare(work.controls == work.receipts.size(),
        checker.control_receipt_comparisons) && exact;
    exact = compare(work.passed_controls == work.controls,
        checker.control_receipt_comparisons) && exact;
    exact = compare(work.root_derivation_paths == 45U,
        checker.control_receipt_comparisons) && exact;
    for (std::size_t index = 0U; index < work.receipts.size(); ++index) {
        const ControlReceipt& receipt = work.receipts[index];
        exact = compare(receipt.passed,
            checker.control_receipt_comparisons) && exact;
        ++checker.root_derivation_paths;
        exact = compare(receipt.root == control_receipt_root(receipt),
            checker.control_receipt_comparisons) && exact;
        exact = compare(receipt.root == work.receipt_roots[index],
            checker.control_receipt_comparisons) && exact;
    }
    ++checker.root_derivation_paths;
    exact = compare(work.root == control_work_root(work),
        checker.control_receipt_comparisons) && exact;
    return exact;
}

bool checker_schedule_valid(const CheckerWork& work) {
    return work.read_semantic_comparisons == 10U
        && work.read_work_comparisons == 19U
        && work.admission_semantic_comparisons == 9U
        && work.admission_work_comparisons == 7U
        && work.product_guard_comparisons == 36U
        && work.product_component_comparisons == 612U
        && work.product_work_comparisons == 120U
        && work.product_root_comparisons == 18U
        && work.aggregate_work_comparisons == 48U
        && work.control_receipt_comparisons == 44U
        && work.schedule_field_comparisons == 13U
        && work.root_derivation_paths == 41U
        && work.result_seals == 2U;
}

FormulaProbeTangentBoundaryRead serialized_cache_control(
    const std::string& source, ControlReceipt& receipt) {
    std::ifstream input(source, std::ios::binary | std::ios::ate);
    if (!input) throw std::runtime_error("cannot open serialized control source");
    const std::streamsize size = input.tellg();
    if (size <= 0) throw std::runtime_error("empty serialized control source");
    input.seekg(0);
    std::vector<char> bytes(static_cast<std::size_t>(size));
    input.read(bytes.data(), size);
    if (!input) throw std::runtime_error("cannot read serialized control source");
    receipt.serialized_bytes_read = bytes.size();
    receipt.temporary_bytes_allocated = 2U * bytes.size();
    bytes[0U] = bytes[0U] == 'N' ? 'X' : 'N';
    receipt.mutation_digest = nextengine::nonlocal::sha256_hex(
        std::string(bytes.data(), bytes.size()));
    ++receipt.root_derivation_paths;
    const std::filesystem::path path = std::filesystem::temp_directory_path()
        / ("nextengine-r63zl-cache-control-"
            + std::to_string(static_cast<long long>(getpid())) + ".bin");
    {
        std::ofstream output(path, std::ios::binary | std::ios::trunc);
        if (!output) throw std::runtime_error(
            "cannot open serialized control destination");
        output.write(bytes.data(), static_cast<std::streamsize>(bytes.size()));
        if (!output) throw std::runtime_error(
            "cannot write serialized control destination");
    }
    receipt.serialized_bytes_written = bytes.size();
    FormulaProbeTangentBoundaryRead read =
        read_formula_probe_tangent_boundary_fixture(path.string());
    std::error_code error;
    std::filesystem::remove(path, error);
    if (error) throw std::runtime_error(
        "cannot remove serialized control destination");
    return read;
}

void print_read_rejection(const FormulaProbeTangentBoundaryRead& read) {
    std::size_t roots = 0U;
    const bool sealed = read.work.root == read_work_root(read.work, roots)
        && read.root == read_result_root(read, roots);
    BoundaryResult result = make_boundary_result(read.exact, false, false,
        false, true, true, sealed, read.root, "", "", "", "", "", {}, {});
    const bool result_exact = boundary_result_valid(result, read.root, "", "",
        "", "", "", {}, {});
    std::cout << "{\"schema\":\"nextengine.nonlocal.r63zl.v2\""
              << ",\"status\":\"FAIL\""
              << ",\"route\":\"" << result.selected_route << "\""
              << ",\"cache_failure_stage\":\"" << read.failure_stage << "\""
              << ",\"read_sealed\":" << (sealed ? "true" : "false")
              << ",\"result_sealed\":"
              << (result_exact ? "true" : "false")
              << ",\"result_sha256\":\"" << result.root << "\"}\n";
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cout << "{\"schema\":\"nextengine.nonlocal.r63zl.v2\""
                      << ",\"status\":\"FAIL\",\"route\":\"INCONCLUSIVE\""
                      << ",\"failure_stage\":\"arguments\"}\n";
            return 2;
        }

        const FormulaProbeTangentBoundaryRead read =
            read_formula_probe_tangent_boundary_fixture(argv[1]);
        if (!read.exact || !read.fixture.has_value()) {
            print_read_rejection(read);
            return 1;
        }
        const FormulaProbeTangentBoundaryFixture& fixture = *read.fixture;
        CheckerWork checker;

        bool read_exact = compare(read.exact,
            checker.read_semantic_comparisons);
        read_exact = compare(read.failure_stage.empty(),
            checker.read_semantic_comparisons) && read_exact;
        read_exact = compare(read.fixture.has_value(),
            checker.read_semantic_comparisons) && read_exact;
        read_exact = compare(fixture.dimension == DIMENSION,
            checker.read_semantic_comparisons) && read_exact;
        read_exact = compare(fixture.tangent_columns == TANGENT_COLUMNS,
            checker.read_semantic_comparisons) && read_exact;
        read_exact = compare(fixture.tangent_root == EXPECTED_TANGENT_ROOT,
            checker.read_semantic_comparisons) && read_exact;
        read_exact = compare(
            fixture.projected_rhs_root == EXPECTED_PROJECTED_RHS_ROOT,
            checker.read_semantic_comparisons) && read_exact;
        read_exact = compare(
            fixture.original_rhs_root == EXPECTED_ORIGINAL_RHS_ROOT,
            checker.read_semantic_comparisons) && read_exact;
        read_exact = valid_read_work(read.work, checker) && read_exact;
        read_exact = compare(read.work.root
                == read_work_root(read.work, checker.root_derivation_paths),
            checker.read_semantic_comparisons) && read_exact;
        read_exact = compare(read.root
                == read_result_root(read, checker.root_derivation_paths),
            checker.read_semantic_comparisons) && read_exact;

        const std::array<InputView, INPUTS> inputs{{
            {FormulaProbeTangentInputRole::ProjectedRhs,
                std::cref(fixture.projected_rhs)},
            {FormulaProbeTangentInputRole::OriginalRhs,
                std::cref(fixture.original_rhs)},
            {FormulaProbeTangentInputRole::Baseline0,
                std::cref(fixture.baseline_solution_0)},
            {FormulaProbeTangentInputRole::Baseline1,
                std::cref(fixture.baseline_solution_1)},
            {FormulaProbeTangentInputRole::Baseline2,
                std::cref(fixture.baseline_solution_2)},
            {FormulaProbeTangentInputRole::CommonProjected2,
                std::cref(fixture.common_projected_solution_2)},
        }};

        const FormulaProbeTangentAdmission admission =
            formula_probe_admit_tangent(fixture);
        bool admission_exact = compare(admission.exact(),
            checker.admission_semantic_comparisons);
        admission_exact = compare(admission.has_context(),
            checker.admission_semantic_comparisons) && admission_exact;
        admission_exact = compare(admission.failure_stage().empty(),
            checker.admission_semantic_comparisons) && admission_exact;
        admission_exact = valid_admission_work(admission.work(), checker)
            && admission_exact;
        if (admission.has_context()) {
            admission_exact = compare(
                admission.context().dimension() == DIMENSION,
                checker.admission_semantic_comparisons) && admission_exact;
            admission_exact = compare(
                admission.context().tangent_columns() == TANGENT_COLUMNS,
                checker.admission_semantic_comparisons) && admission_exact;
            admission_exact = compare(
                admission.context().tangent_root() == EXPECTED_TANGENT_ROOT,
                checker.admission_semantic_comparisons) && admission_exact;
            admission_exact = compare(admission.context().context_root()
                    == context_root(admission.context(),
                        checker.root_derivation_paths),
                checker.admission_semantic_comparisons) && admission_exact;
        }
        admission_exact = compare(admission.work().root
                == admission_work_root(admission.work(),
                    checker.root_derivation_paths),
            checker.admission_semantic_comparisons) && admission_exact;
        admission_exact = compare(admission.root()
                == admission_result_root(admission,
                    checker.root_derivation_paths),
            checker.admission_semantic_comparisons) && admission_exact;

        if (!admission_exact || !admission.has_context()) {
            BoundaryResult rejected = make_boundary_result(read_exact, false,
                false, false, true, true, true, read.root, admission.root(),
                "", "", "", "", {}, {});
            const bool rejected_sealed = boundary_result_valid(rejected,
                read.root, admission.root(), "", "", "", "", {}, {});
            std::cout << "{\"schema\":\"nextengine.nonlocal.r63zl.v2\""
                      << ",\"status\":\"FAIL\",\"route\":\""
                      << rejected.selected_route << "\""
                      << ",\"admission_failure_stage\":\""
                      << admission.failure_stage() << "\""
                      << ",\"result_sealed\":"
                      << (rejected_sealed ? "true" : "false")
                      << ",\"result_sha256\":\"" << rejected.root
                      << "\"}\n";
            return 1;
        }

        ProductTotals candidate;
        candidate.role = "candidate";
        ProductTotals reference;
        reference.role = "reference";
        std::vector<FormulaProbeAdmittedProduct> admitted_products;
        admitted_products.reserve(INPUTS);
        bool candidate_complete = true;
        for (const InputView& input : inputs) {
            admitted_products.push_back(formula_probe_admitted_tangent_product(
                admission.context(), input.role, input.values.get()));
            add_candidate(candidate, admitted_products.back().work());
            candidate_complete = admitted_products.back().exact()
                && candidate_complete;
        }

        std::vector<FormulaProbeProduct> legacy_products;
        legacy_products.reserve(INPUTS);
        if (candidate_complete) {
            for (const InputView& input : inputs) {
                legacy_products.push_back(formula_probe_tangent_product(
                    fixture, input.values.get()));
                add_reference(reference, legacy_products.back());
            }
        }

        bool products_exact = candidate_complete
            && legacy_products.size() == INPUTS;
        std::vector<std::string> admitted_roots;
        std::vector<std::string> legacy_roots;
        for (std::size_t input = 0U;
             input < admitted_products.size() && input < legacy_products.size();
             ++input) {
            const FormulaProbeAdmittedProduct& admitted =
                admitted_products[input];
            const FormulaProbeProduct& legacy = legacy_products[input];
            products_exact = compare(admitted.exact(),
                checker.product_guard_comparisons) && products_exact;
            products_exact = compare(legacy.exact,
                checker.product_guard_comparisons) && products_exact;
            products_exact = compare(admitted.failure_stage().empty(),
                checker.product_guard_comparisons) && products_exact;
            products_exact = compare(admitted.context_root()
                    == admission.context().context_root(),
                checker.product_guard_comparisons) && products_exact;
            products_exact = compare(admitted.input_role() == inputs[input].role,
                checker.product_guard_comparisons) && products_exact;
            products_exact = compare(admitted.product().value.size()
                    == legacy.value.size(),
                checker.product_guard_comparisons) && products_exact;
            for (std::size_t index = 0U;
                 index < admitted.product().value.size()
                    && index < legacy.value.size(); ++index) {
                products_exact = compare(admitted.product().value[index]
                        == legacy.value[index],
                    checker.product_component_comparisons) && products_exact;
            }
            products_exact = valid_product_work(admitted.work(), checker)
                && products_exact;
            products_exact = compare(admitted.product().inner_dots
                    == legacy.inner_dots,
                checker.product_work_comparisons) && products_exact;
            products_exact = compare(admitted.product().outer_dots
                    == legacy.outer_dots,
                checker.product_work_comparisons) && products_exact;
            products_exact = compare(admitted.product().inner_terms
                    == legacy.inner_terms,
                checker.product_work_comparisons) && products_exact;
            products_exact = compare(admitted.product().outer_terms
                    == legacy.outer_terms,
                checker.product_work_comparisons) && products_exact;
            products_exact = compare(admitted.product().scale_products
                    == legacy.scale_products,
                checker.product_work_comparisons) && products_exact;
            products_exact = compare(admitted.product().root == legacy.root,
                checker.product_root_comparisons) && products_exact;
            products_exact = compare(admitted.work().root
                    == admitted_product_work_root(admitted.work(),
                        checker.root_derivation_paths),
                checker.product_root_comparisons) && products_exact;
            products_exact = compare(admitted.root()
                    == admitted_product_result_root(admitted,
                        checker.root_derivation_paths),
                checker.product_root_comparisons) && products_exact;
            admitted_roots.push_back(admitted.root());
            legacy_roots.push_back(legacy.root);
        }

        bool work_exact = exact_candidate_totals(
            candidate, checker.aggregate_work_comparisons)
            && exact_reference_totals(
                reference, checker.aggregate_work_comparisons);
        candidate.root = product_totals_root(
            candidate, checker.root_derivation_paths);
        reference.root = product_totals_root(
            reference, checker.root_derivation_paths);
        work_exact = compare(candidate.root
                == product_totals_root(candidate,
                    checker.root_derivation_paths),
            checker.aggregate_work_comparisons) && work_exact;
        work_exact = compare(reference.root
                == product_totals_root(reference,
                    checker.root_derivation_paths),
            checker.aggregate_work_comparisons) && work_exact;

        ControlWork controls;
        FormulaProbeTangentBoundaryFixture mutated = fixture;
        {
            ControlReceipt receipt;
            receipt.name = "control_fixture_copy";
            receipt.expected_route = route(RouteInput::None);
            receipt.observed_route = route(RouteInput::None);
            receipt.expected_stage = "copy";
            receipt.observed_stage = "copy";
            receipt.selected_components_copied = SELECTED_COMPONENTS;
            receipt.selected_scalar_fields_copied = 5U;
            receipt.selected_string_bytes_copied = 192U;
            receipt.selected_vector_allocations = 7U;
            receipt.mutation_digest = nextengine::nonlocal::sha256_hex(
                "r63zl-control-fixture-copy-v1");
            ++receipt.root_derivation_paths;
            receipt.child_root = read.root;
            receipt.passed = control_check(
                mutated.tangent.size() == fixture.tangent.size(),
                receipt, controls);
            finish_control(std::move(receipt), controls);
        }

        ++mutated.dimension;
        admission_control("dimension", formula_probe_admit_tangent(mutated),
            "dimension", 1U, 0U, "103", 0U, controls);
        --mutated.dimension;
        mutated.tangent_root[0U] = mutated.tangent_root[0U] == '0' ? '1' : '0';
        admission_control("declared_tangent_root",
            formula_probe_admit_tangent(mutated), "declared_tangent_root",
            4U, 0U, mutated.tangent_root, 0U, controls);
        mutated.tangent_root[0U] = mutated.tangent_root[0U] == '0' ? '1' : '0';
        const FormulaProbeBinary128 saved_tangent = mutated.tangent[0U];
        mutated.tangent[0U] += static_cast<FormulaProbeBinary128>(1.0);
        const std::string tangent_mutation_root =
            formula_probe_binary128_vector_root(mutated.tangent);
        admission_control("tangent_payload_root",
            formula_probe_admit_tangent(mutated), "tangent_payload_root",
            5U, 1U, tangent_mutation_root, 1U, controls);
        mutated.tangent[0U] = saved_tangent;
        const FormulaProbeBinary128 saved_sigma = mutated.sigma;
        mutated.sigma = static_cast<FormulaProbeBinary128>(0.0);
        admission_control("sigma_positive", formula_probe_admit_tangent(mutated),
            "sigma_positive", 8U, 1U, "0x0p+0", 0U, controls);
        mutated.sigma = saved_sigma;

        {
            ControlReceipt receipt;
            receipt.name = "short_input";
            receipt.expected_route = route(RouteInput::Product);
            receipt.expected_stage = "input_size";
            std::vector<FormulaProbeBinary128> short_input(DIMENSION - 1U);
            receipt.temporary_components_allocated = short_input.size();
            const FormulaProbeAdmittedProduct product =
                formula_probe_admitted_tangent_product(admission.context(),
                    FormulaProbeTangentInputRole::ProjectedRhs, short_input);
            receipt.observed_route = product.exact()
                ? route(RouteInput::None) : route(RouteInput::Product);
            receipt.observed_stage = product.failure_stage();
            receipt.child_root = product.root();
            receipt.mutation_digest = nextengine::nonlocal::sha256_hex(
                "r63zl-short-input-v1:length=101");
            ++receipt.root_derivation_paths;
            bool exact = true;
            exact = control_check(!product.exact(), receipt, controls) && exact;
            exact = control_check(product.product().value.empty(),
                receipt, controls) && exact;
            exact = control_check(product.work().input_guard_checks == 1U,
                receipt, controls) && exact;
            exact = control_check(product.work().input_payload_hashes == 0U,
                receipt, controls) && exact;
            exact = control_check(product.work().kernel_calls == 0U,
                receipt, controls) && exact;
            std::size_t roots = 0U;
            exact = control_check(product.work().root
                    == admitted_product_work_root(product.work(), roots),
                receipt, controls) && exact;
            exact = control_check(product.root()
                    == admitted_product_result_root(product, roots),
                receipt, controls) && exact;
            receipt.root_derivation_paths += roots;
            receipt.passed = exact;
            finish_control(std::move(receipt), controls);
        }

        {
            ControlReceipt receipt;
            receipt.name = "closed_api";
            receipt.expected_route = route(RouteInput::None);
            receipt.observed_route = route(RouteInput::None);
            receipt.expected_stage = "compile_time";
            receipt.observed_stage = "compile_time";
            constexpr bool api_closed =
                !std::is_default_constructible_v<FormulaProbeAdmittedTangent>
                && !std::is_aggregate_v<FormulaProbeAdmittedTangent>
                && !std::is_copy_assignable_v<FormulaProbeAdmittedTangent>
                && !std::is_move_assignable_v<FormulaProbeAdmittedTangent>
                && !std::is_constructible_v<FormulaProbeAdmittedTangent,
                    const FormulaProbeTangentBoundaryFixture&>;
            static_assert(api_closed);
            receipt.mutation_digest = nextengine::nonlocal::sha256_hex(
                "r63zl-closed-api-v2");
            ++receipt.root_derivation_paths;
            receipt.child_root = admission.context().context_root();
            receipt.passed = control_check(api_closed, receipt, controls);
            finish_control(std::move(receipt), controls);
        }

        {
            ControlReceipt receipt;
            receipt.name = "mutated_frozen_input";
            receipt.expected_route = route(RouteInput::Product);
            receipt.expected_stage = "input_identity";
            std::vector<FormulaProbeBinary128> mutated_input =
                fixture.projected_rhs;
            receipt.selected_vector_allocations = 1U;
            receipt.selected_components_copied = mutated_input.size();
            mutated_input[0U] += static_cast<FormulaProbeBinary128>(1.0);
            const FormulaProbeAdmittedProduct product =
                formula_probe_admitted_tangent_product(admission.context(),
                    FormulaProbeTangentInputRole::ProjectedRhs, mutated_input);
            receipt.observed_route = product.exact()
                ? route(RouteInput::None) : route(RouteInput::Product);
            receipt.observed_stage = product.failure_stage();
            receipt.child_root = product.root();
            receipt.mutation_digest = product.input_root();
            bool exact = true;
            exact = control_check(!product.exact(), receipt, controls) && exact;
            exact = control_check(product.input_root()
                    != EXPECTED_PROJECTED_RHS_ROOT,
                receipt, controls) && exact;
            exact = control_check(product.work().input_guard_checks == 1U,
                receipt, controls) && exact;
            exact = control_check(product.work().input_payload_hashes == 1U,
                receipt, controls) && exact;
            exact = control_check(product.work().input_identity_checks == 1U,
                receipt, controls) && exact;
            exact = control_check(product.work().kernel_calls == 0U,
                receipt, controls) && exact;
            std::size_t roots = 0U;
            exact = control_check(product.work().root
                    == admitted_product_work_root(product.work(), roots),
                receipt, controls) && exact;
            exact = control_check(product.root()
                    == admitted_product_result_root(product, roots),
                receipt, controls) && exact;
            receipt.root_derivation_paths += roots;
            receipt.passed = exact;
            finish_control(std::move(receipt), controls);
        }

        {
            ControlReceipt receipt;
            receipt.name = "serialized_cache_rejection";
            receipt.expected_route = route(RouteInput::Admission);
            receipt.expected_stage = "cache_magic";
            const FormulaProbeTangentBoundaryRead rejected =
                serialized_cache_control(argv[1], receipt);
            receipt.observed_route = rejected.exact
                ? route(RouteInput::None) : route(RouteInput::Admission);
            receipt.observed_stage = rejected.failure_stage;
            receipt.child_root = rejected.root;
            bool exact = true;
            exact = control_check(!rejected.exact, receipt, controls) && exact;
            exact = control_check(!rejected.fixture.has_value(),
                receipt, controls) && exact;
            exact = control_check(rejected.work.header_predicate_checks == 1U,
                receipt, controls) && exact;
            std::size_t roots = 0U;
            exact = control_check(rejected.work.root
                    == read_work_root(rejected.work, roots),
                receipt, controls) && exact;
            exact = control_check(rejected.root
                    == read_result_root(rejected, roots),
                receipt, controls) && exact;
            receipt.root_derivation_paths += roots;
            receipt.passed = exact;
            finish_control(std::move(receipt), controls);
        }

        auto receipt_seal_control = [&](const std::string& name,
                                        const ProductTotals& original) {
            ControlReceipt receipt;
            receipt.name = name;
            receipt.expected_route = route(RouteInput::Seal);
            receipt.expected_stage = "resealed_receipt";
            ProductTotals changed = original;
            ++changed.kernel_calls;
            std::size_t roots = 0U;
            changed.root = product_totals_root(changed, roots);
            const bool internally_sealed = changed.root
                == product_totals_root(changed, roots);
            std::size_t semantic_comparisons = 0U;
            const bool semantic_exact = changed.role == "candidate"
                ? exact_candidate_totals(changed, semantic_comparisons)
                : exact_reference_totals(changed, semantic_comparisons);
            receipt.comparisons += semantic_comparisons;
            controls.comparisons += semantic_comparisons;
            const bool seal_exact = internally_sealed && semantic_exact
                && changed.root == original.root;
            const RouteInput observed = classify(true, true, true, true, true,
                true, seal_exact);
            receipt.observed_route = route(observed);
            receipt.observed_stage = seal_exact ? "accepted" : "resealed_receipt";
            receipt.mutation_digest = changed.root;
            receipt.child_root = original.root;
            receipt.root_derivation_paths += roots;
            receipt.passed = control_check(internally_sealed,
                    receipt, controls)
                && control_check(!semantic_exact, receipt, controls)
                && control_check(!seal_exact, receipt, controls)
                && control_check(observed == RouteInput::Seal,
                    receipt, controls);
            finish_control(std::move(receipt), controls);
        };
        receipt_seal_control("candidate_receipt_seal", candidate);
        receipt_seal_control("reference_receipt_seal", reference);

        {
            ControlReceipt receipt;
            receipt.name = "final_result_seal";
            receipt.expected_route = route(RouteInput::Seal);
            receipt.expected_stage = "result_root";
            BoundaryResult synthetic = make_boundary_result(true, true, true,
                true, true, true, true, read.root, admission.root(),
                candidate.root, reference.root, "checker-sentinel",
                "controls-sentinel", admitted_roots, legacy_roots);
            synthetic.candidate_root[0U] =
                synthetic.candidate_root[0U] == '0' ? '1' : '0';
            synthetic.root = boundary_result_root(synthetic);
            const bool valid = boundary_result_valid(synthetic, read.root,
                admission.root(), candidate.root, reference.root,
                "checker-sentinel", "controls-sentinel", admitted_roots,
                legacy_roots);
            const RouteInput observed = valid
                ? RouteInput::None : RouteInput::Seal;
            receipt.observed_route = route(observed);
            receipt.observed_stage = valid ? "accepted" : "result_root";
            receipt.mutation_digest = synthetic.root;
            receipt.child_root = candidate.root;
            receipt.root_derivation_paths += 3U;
            receipt.passed = control_check(!valid, receipt, controls)
                && control_check(observed == RouteInput::Seal,
                    receipt, controls);
            finish_control(std::move(receipt), controls);
        }

        {
            ControlReceipt receipt;
            receipt.name = "classifier";
            receipt.expected_route = route(RouteInput::None);
            receipt.observed_route = route(RouteInput::None);
            receipt.expected_stage = "all_cases";
            receipt.observed_stage = "all_cases";
            const std::array<std::pair<RouteInput, std::string>, 7U> cases{{
                {RouteInput::None,
                    "ADMITTED_TANGENT_WORK_BOUNDARY_CANDIDATE"},
                {RouteInput::Admission,
                    "ADMITTED_TANGENT_ADMISSION_REJECTED"},
                {RouteInput::Product,
                    "ADMITTED_TANGENT_PRODUCT_REJECTED"},
                {RouteInput::Work, "ADMITTED_TANGENT_WORK_REJECTED"},
                {RouteInput::Control, "INCONCLUSIVE"},
                {RouteInput::Seal, "INCONCLUSIVE"},
                {RouteInput::Unknown, "INCONCLUSIVE"},
            }};
            bool exact = true;
            std::ostringstream digest;
            for (const auto& [input, expected] : cases) {
                ++controls.classifier_cases;
                exact = control_check(route(input) == expected,
                    receipt, controls) && exact;
                digest << static_cast<int>(input) << ':' << expected << ';';
            }
            receipt.mutation_digest =
                nextengine::nonlocal::sha256_hex(digest.str());
            ++receipt.root_derivation_paths;
            receipt.child_root = "classifier";
            receipt.passed = exact;
            finish_control(std::move(receipt), controls);
        }

        ++controls.root_derivation_paths;
        controls.root = control_work_root(controls);
        bool controls_exact = controls_valid(controls, checker);

        checker.result_seals += 2U;
        checker.root_derivation_paths += 6U;
        checker.schedule_field_comparisons = 13U;
        const bool checker_exact = checker_schedule_valid(checker);
        checker.root = checker_work_root(checker);
        const bool checker_seal_exact =
            checker.root == checker_work_root(checker);

        BoundaryResult result = make_boundary_result(read_exact,
            admission_exact, products_exact, work_exact, checker_exact,
            controls_exact, checker_seal_exact, read.root, admission.root(),
            candidate.root, reference.root, checker.root, controls.root,
            admitted_roots, legacy_roots);
        const bool first_result_seal = boundary_result_valid(result, read.root,
            admission.root(), candidate.root, reference.root, checker.root,
            controls.root, admitted_roots, legacy_roots);
        const BoundaryResult reconstructed = make_boundary_result(read_exact,
            admission_exact, products_exact, work_exact, checker_exact,
            controls_exact, checker_seal_exact, read.root, admission.root(),
            candidate.root, reference.root, checker.root, controls.root,
            admitted_roots, legacy_roots);
        const bool second_result_seal = boundary_result_valid(reconstructed,
            read.root, admission.root(), candidate.root, reference.root,
            checker.root, controls.root, admitted_roots, legacy_roots)
            && reconstructed.root == result.root;
        const bool post_final_checker_exact = checker_schedule_valid(checker);
        const bool passed = result.route_input == RouteInput::None
            && first_result_seal && second_result_seal
            && post_final_checker_exact;

        std::cout << "{\"schema\":\"nextengine.nonlocal.r63zl.v2\""
                  << ",\"status\":\"" << (passed ? "PASS" : "FAIL")
                  << "\",\"route\":\"" << result.selected_route << "\""
                  << ",\"selective_read\":{\"exact\":"
                  << (read_exact ? "true" : "false")
                  << ",\"selected_vectors\":"
                  << read.work.selected_vector_fields
                  << ",\"selected_components\":"
                  << read.work.selected_vector_components
                  << ",\"skipped_bytes\":"
                  << read.work.skipped_payload_bytes
                  << ",\"root\":\"" << read.root << "\"}"
                  << ",\"admission\":{\"exact\":"
                  << (admission_exact ? "true" : "false")
                  << ",\"predicate_checks\":"
                  << admission.work().predicate_checks
                  << ",\"tangent_payload_hashes\":"
                  << admission.work().tangent_payload_hashes
                  << ",\"tangent_components_copied\":"
                  << admission.work().tangent_components_copied
                  << ",\"root\":\"" << admission.root() << "\"}"
                  << ",\"candidate\":{\"input_hashes\":"
                  << candidate.input_payload_hashes
                  << ",\"kernel_calls\":" << candidate.kernel_calls
                  << ",\"temporary_operand_allocations\":"
                  << candidate.temporary_operand_allocations
                  << ",\"operand_components_copied\":"
                  << candidate.operand_components_copied
                  << ",\"root\":\"" << candidate.root << "\"}"
                  << ",\"reference\":{\"payload_validations\":"
                  << reference.payload_validations
                  << ",\"tangent_payload_hashes\":"
                  << reference.tangent_payload_hashes
                  << ",\"kernel_calls\":" << reference.kernel_calls
                  << ",\"temporary_operand_allocations\":"
                  << reference.temporary_operand_allocations
                  << ",\"operand_components_copied\":"
                  << reference.operand_components_copied
                  << ",\"root\":\"" << reference.root << "\"}"
                  << ",\"comparisons\":{\"components\":"
                  << checker.product_component_comparisons
                  << ",\"products_exact\":"
                  << (products_exact ? "true" : "false")
                  << ",\"work_exact\":"
                  << (work_exact && checker_exact ? "true" : "false")
                  << ",\"checker_exact\":"
                  << (checker_exact ? "true" : "false")
                  << ",\"checker_counts\":["
                  << checker.read_semantic_comparisons << ','
                  << checker.read_work_comparisons << ','
                  << checker.admission_semantic_comparisons << ','
                  << checker.admission_work_comparisons << ','
                  << checker.product_guard_comparisons << ','
                  << checker.product_component_comparisons << ','
                  << checker.product_work_comparisons << ','
                  << checker.product_root_comparisons << ','
                  << checker.aggregate_work_comparisons << ','
                  << checker.control_receipt_comparisons << ','
                  << checker.schedule_field_comparisons << ','
                  << checker.root_derivation_paths << ','
                  << checker.result_seals << ']'
                  << ",\"checker_root\":\"" << checker.root << "\"}"
                  << ",\"controls\":{\"passed\":"
                  << (controls_exact ? "true" : "false")
                  << ",\"count\":" << controls.controls
                  << ",\"passed_count\":" << controls.passed_controls
                  << ",\"comparisons\":" << controls.comparisons
                  << ",\"root_paths\":" << controls.root_derivation_paths
                  << ",\"classifier_cases\":"
                  << controls.classifier_cases << ",\"root\":\""
                  << controls.root << "\"}"
                  << ",\"result_sealed\":"
                  << (first_result_seal && second_result_seal
                          ? "true" : "false")
                  << ",\"physics_mutation\":false"
                  << ",\"timing_admitted\":false"
                  << ",\"correspondence_authority\":false"
                  << ",\"representation_authority\":false"
                  << ",\"runtime_authority\":false"
                  << ",\"production_authority\":false"
                  << ",\"result_sha256\":\"" << result.root << "\"}\n";
        return passed ? 0 : 1;
    } catch (const std::exception& error) {
        std::cout << "{\"schema\":\"nextengine.nonlocal.r63zl.v2\""
                  << ",\"status\":\"FAIL\",\"route\":\"INCONCLUSIVE\""
                  << ",\"failure_stage\":\"unexpected_exception\""
                  << ",\"diagnostic_sha256\":\""
                  << nextengine::nonlocal::sha256_hex(error.what())
                  << "\"}\n";
        return 1;
    }
}
