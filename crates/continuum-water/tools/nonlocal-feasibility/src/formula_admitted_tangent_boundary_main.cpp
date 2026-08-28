#include "formula_probe_cache.hpp"
#include "sha256.hpp"

#include <cstddef>
#include <exception>
#include <iostream>
#include <sstream>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>

namespace {

using namespace nextengine::nonlocal::fcr;

constexpr std::size_t DIMENSION = 102U;
constexpr std::size_t TANGENT_COLUMNS = 315U;
constexpr std::size_t TANGENT_COMPONENTS = 32130U;
constexpr std::size_t INPUTS = 6U;
constexpr const char* EXPECTED_TANGENT_ROOT =
    "114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1";

struct ProductTotals {
    std::size_t payload_validations = 0U;
    std::size_t predicate_checks = 0U;
    std::size_t tangent_payload_hashes = 0U;
    std::size_t frozen_scalar_parses = 0U;
    std::size_t input_guard_checks = 0U;
    std::size_t kernel_guard_checks = 0U;
    std::size_t kernel_calls = 0U;
    std::size_t inner_dots = 0U;
    std::size_t outer_dots = 0U;
    std::size_t inner_terms = 0U;
    std::size_t outer_terms = 0U;
    std::size_t scale_products = 0U;
    std::size_t kernel_root_derivations = 0U;
    std::size_t dot_guard_checks = 0U;
    std::size_t dot_root_derivations = 0U;
    std::size_t receipt_root_derivations = 0U;
    std::size_t result_root_derivations = 0U;
    std::string root;
};

struct CheckerWork {
    std::size_t input_shape_comparisons = 0U;
    std::size_t admission_semantic_comparisons = 0U;
    std::size_t admission_work_comparisons = 0U;
    std::size_t product_guard_comparisons = 0U;
    std::size_t product_component_comparisons = 0U;
    std::size_t product_work_comparisons = 0U;
    std::size_t product_root_comparisons = 0U;
    std::size_t aggregate_work_comparisons = 0U;
    std::size_t root_derivation_paths = 0U;
    std::size_t result_seals = 0U;
    std::string root;
};

struct ControlWork {
    std::size_t controls = 0U;
    std::size_t comparisons = 0U;
    std::size_t admission_predicate_checks = 0U;
    std::size_t tangent_payload_hashes = 0U;
    std::size_t frozen_scalar_parses = 0U;
    std::size_t short_input_guards = 0U;
    std::size_t short_input_kernels = 0U;
    std::size_t compile_time_api_controls = 0U;
    std::size_t digest_controls = 0U;
    std::size_t classifier_cases = 0U;
    std::size_t root_derivation_paths = 0U;
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

bool compare(bool value, std::size_t& counter) {
    ++counter;
    return value;
}

std::string derive_root(const std::string& material,
    std::size_t& counter) {
    ++counter;
    return nextengine::nonlocal::sha256_hex(material);
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
    material << work.input_guard_checks << ':' << work.kernel_calls << ':'
        << work.inner_dots << ':' << work.outer_dots << ':'
        << work.inner_terms << ':' << work.outer_terms << ':'
        << work.scale_products << ':' << work.kernel_root_derivations << ':'
        << work.receipt_root_derivations << ':'
        << work.result_root_derivations;
    return derive_root(material.str(), counter);
}

std::string admitted_product_result_root(
    const FormulaProbeAdmittedProduct& value, std::size_t& counter) {
    std::ostringstream material;
    material << value.exact() << ':' << value.failure_stage() << ':'
        << value.context_root() << ':' << value.product().root << ':'
        << value.work().root;
    return derive_root(material.str(), counter);
}

std::string product_totals_root(
    const ProductTotals& value, std::size_t& counter) {
    std::ostringstream material;
    material << value.payload_validations << ':' << value.predicate_checks
        << ':' << value.tangent_payload_hashes << ':'
        << value.frozen_scalar_parses << ':' << value.input_guard_checks
        << ':' << value.kernel_guard_checks << ':' << value.kernel_calls
        << ':'
        << value.inner_dots << ':' << value.outer_dots << ':'
        << value.inner_terms << ':' << value.outer_terms << ':'
        << value.scale_products << ':' << value.kernel_root_derivations
        << ':' << value.dot_guard_checks << ':'
        << value.dot_root_derivations << ':'
        << value.receipt_root_derivations << ':'
        << value.result_root_derivations;
    return derive_root(material.str(), counter);
}

std::string checker_work_root(const CheckerWork& value) {
    std::ostringstream material;
    material << value.input_shape_comparisons << ':'
        << value.admission_semantic_comparisons << ':'
        << value.admission_work_comparisons << ':'
        << value.product_guard_comparisons << ':'
        << value.product_component_comparisons << ':'
        << value.product_work_comparisons << ':'
        << value.product_root_comparisons << ':'
        << value.aggregate_work_comparisons << ':'
        << value.root_derivation_paths << ':' << value.result_seals;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string control_work_root(const ControlWork& value) {
    std::ostringstream material;
    material << value.controls << ':' << value.comparisons << ':'
        << value.admission_predicate_checks << ':'
        << value.tangent_payload_hashes << ':'
        << value.frozen_scalar_parses << ':'
        << value.short_input_guards << ':' << value.short_input_kernels
        << ':' << value.compile_time_api_controls << ':'
        << value.digest_controls << ':' << value.classifier_cases << ':'
        << value.root_derivation_paths;
    return nextengine::nonlocal::sha256_hex(material.str());
}

void add_candidate(ProductTotals& totals,
    const FormulaProbeAdmittedProductWork& work) {
    totals.input_guard_checks += work.input_guard_checks;
    totals.kernel_calls += work.kernel_calls;
    totals.inner_dots += work.inner_dots;
    totals.outer_dots += work.outer_dots;
    totals.inner_terms += work.inner_terms;
    totals.outer_terms += work.outer_terms;
    totals.scale_products += work.scale_products;
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
    totals.kernel_root_derivations += 5U;
    totals.dot_guard_checks += 2U * (TANGENT_COLUMNS + DIMENSION);
    totals.dot_root_derivations += 3U * (TANGENT_COLUMNS + DIMENSION);
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
        && compare(work.kernel_root_derivations == 5U,
            checker.product_work_comparisons)
        && compare(work.receipt_root_derivations == 1U,
            checker.product_work_comparisons)
        && compare(work.result_root_derivations == 1U,
            checker.product_work_comparisons);
}

bool exact_candidate_totals(const ProductTotals& value,
    std::size_t& comparisons) {
    return compare(value.payload_validations == 0U, comparisons)
        && compare(value.predicate_checks == 0U, comparisons)
        && compare(value.tangent_payload_hashes == 0U, comparisons)
        && compare(value.frozen_scalar_parses == 0U, comparisons)
        && compare(value.input_guard_checks == 6U, comparisons)
        && compare(value.kernel_guard_checks == 0U, comparisons)
        && compare(value.kernel_calls == 6U, comparisons)
        && compare(value.inner_dots == 1890U, comparisons)
        && compare(value.outer_dots == 612U, comparisons)
        && compare(value.inner_terms == 192780U, comparisons)
        && compare(value.outer_terms == 192780U, comparisons)
        && compare(value.scale_products == 612U, comparisons)
        && compare(value.kernel_root_derivations == 30U, comparisons)
        && compare(value.dot_guard_checks == 0U, comparisons)
        && compare(value.dot_root_derivations == 0U, comparisons)
        && compare(value.receipt_root_derivations == 6U, comparisons)
        && compare(value.result_root_derivations == 6U, comparisons);
}

bool exact_reference_totals(const ProductTotals& value,
    std::size_t& comparisons) {
    return compare(value.payload_validations == 6U, comparisons)
        && compare(value.predicate_checks == 60U, comparisons)
        && compare(value.tangent_payload_hashes == 6U, comparisons)
        && compare(value.frozen_scalar_parses == 12U, comparisons)
        && compare(value.input_guard_checks == 6U, comparisons)
        && compare(value.kernel_guard_checks == 36U, comparisons)
        && compare(value.kernel_calls == 6U, comparisons)
        && compare(value.inner_dots == 1890U, comparisons)
        && compare(value.outer_dots == 612U, comparisons)
        && compare(value.inner_terms == 192780U, comparisons)
        && compare(value.outer_terms == 192780U, comparisons)
        && compare(value.scale_products == 612U, comparisons)
        && compare(value.kernel_root_derivations == 30U, comparisons)
        && compare(value.dot_guard_checks == 5004U, comparisons)
        && compare(value.dot_root_derivations == 7506U, comparisons)
        && compare(value.receipt_root_derivations == 0U, comparisons)
        && compare(value.result_root_derivations == 0U, comparisons);
}

bool control_admission(const FormulaProbeTangentAdmission& value,
    const std::string& stage, std::size_t predicates,
    std::size_t payload_hashes, ControlWork& work) {
    ++work.controls;
    work.admission_predicate_checks += value.work().predicate_checks;
    work.tangent_payload_hashes += value.work().tangent_payload_hashes;
    work.frozen_scalar_parses += value.work().frozen_scalar_parses;
    bool exact = compare(!value.exact(), work.comparisons)
        && compare(!value.has_context(), work.comparisons)
        && compare(value.failure_stage() == stage, work.comparisons)
        && compare(value.work().predicate_checks == predicates,
            work.comparisons)
        && compare(value.work().tangent_payload_hashes == payload_hashes,
            work.comparisons)
        && compare(value.work().frozen_scalar_parses
                == (stage == "sigma_positive" ? 2U : 0U),
            work.comparisons)
        && compare(value.work().tangent_components_copied == 0U,
            work.comparisons)
        && compare(value.work().context_root_derivations == 0U,
            work.comparisons)
        && compare(value.work().receipt_root_derivations == 1U,
            work.comparisons)
        && compare(value.work().result_root_derivations == 1U,
            work.comparisons);
    exact = compare(value.work().root
            == admission_work_root(value.work(), work.root_derivation_paths),
        work.comparisons) && exact;
    exact = compare(value.root()
            == admission_result_root(value, work.root_derivation_paths),
        work.comparisons) && exact;
    return exact;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-admitted-tangent-boundary <cache>\n";
            return 2;
        }
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        CheckerWork checker;
        ControlWork controls;
        bool apparatus = true;

        std::vector<std::vector<FormulaProbeBinary128>> inputs;
        inputs.push_back(fixture.projected_rhs);
        inputs.push_back(fixture.original_rhs);
        if (fixture.baseline_solutions.size() >= 3U) {
            inputs.push_back(fixture.baseline_solutions[0U]);
            inputs.push_back(fixture.baseline_solutions[1U]);
            inputs.push_back(fixture.baseline_solutions[2U]);
        }
        if (fixture.common_projected_solutions.size() >= 3U)
            inputs.push_back(fixture.common_projected_solutions[2U]);
        apparatus = compare(inputs.size() == INPUTS,
            checker.input_shape_comparisons) && apparatus;
        for (const auto& input : inputs)
            apparatus = compare(input.size() == DIMENSION,
                checker.input_shape_comparisons) && apparatus;

        const FormulaProbeTangentAdmission admission =
            formula_probe_admit_tangent(fixture);
        bool admission_exact = compare(admission.exact(),
                checker.admission_semantic_comparisons)
            && compare(admission.has_context(),
                checker.admission_semantic_comparisons)
            && compare(admission.failure_stage().empty(),
                checker.admission_semantic_comparisons)
            && compare(admission.context().dimension() == DIMENSION,
                checker.admission_semantic_comparisons)
            && compare(admission.context().tangent_columns()
                    == TANGENT_COLUMNS,
                checker.admission_semantic_comparisons)
            && compare(admission.context().tangent_root()
                    == EXPECTED_TANGENT_ROOT,
                checker.admission_semantic_comparisons)
            && valid_admission_work(admission.work(), checker);
        admission_exact = compare(admission.context().context_root()
                == context_root(admission.context(),
                    checker.root_derivation_paths),
            checker.admission_semantic_comparisons) && admission_exact;
        admission_exact = compare(admission.work().root
                == admission_work_root(admission.work(),
                    checker.root_derivation_paths),
            checker.admission_semantic_comparisons) && admission_exact;
        admission_exact = compare(admission.root()
                == admission_result_root(admission,
                    checker.root_derivation_paths),
            checker.admission_semantic_comparisons) && admission_exact;

        ProductTotals candidate;
        ProductTotals reference;
        bool products_exact = apparatus && admission_exact;
        std::vector<std::string> admitted_roots;
        std::vector<std::string> legacy_roots;
        for (const auto& input : inputs) {
            const FormulaProbeAdmittedProduct admitted =
                formula_probe_admitted_tangent_product(
                    admission.context(), input);
            const FormulaProbeProduct legacy =
                formula_probe_tangent_product(fixture, input);
            add_candidate(candidate, admitted.work());
            add_reference(reference, legacy);
            products_exact = compare(admitted.exact() == legacy.exact,
                checker.product_guard_comparisons) && products_exact;
            products_exact = compare(admitted.failure_stage().empty(),
                checker.product_guard_comparisons) && products_exact;
            products_exact = compare(admitted.context_root()
                    == admission.context().context_root(),
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

        FormulaProbeParentFixture mutated = fixture;
        ++mutated.dimension;
        bool controls_exact = control_admission(
            formula_probe_admit_tangent(mutated), "dimension", 1U, 0U,
            controls);
        mutated = fixture;
        mutated.tangent_root[0U] = mutated.tangent_root[0U] == '0' ? '1' : '0';
        controls_exact = control_admission(formula_probe_admit_tangent(mutated),
            "declared_tangent_root", 4U, 0U, controls) && controls_exact;
        mutated = fixture;
        mutated.tangent[0U] += static_cast<FormulaProbeBinary128>(1.0);
        controls_exact = control_admission(formula_probe_admit_tangent(mutated),
            "tangent_payload_root", 5U, 1U, controls) && controls_exact;
        mutated = fixture;
        mutated.sigma = static_cast<FormulaProbeBinary128>(0.0);
        controls_exact = control_admission(formula_probe_admit_tangent(mutated),
            "sigma_positive", 8U, 1U, controls) && controls_exact;

        ++controls.controls;
        std::vector<FormulaProbeBinary128> short_input(DIMENSION - 1U);
        const FormulaProbeAdmittedProduct short_product =
            formula_probe_admitted_tangent_product(
                admission.context(), short_input);
        controls.short_input_guards += short_product.work().input_guard_checks;
        controls.short_input_kernels += short_product.work().kernel_calls;
        controls_exact = compare(!short_product.exact(), controls.comparisons)
            && compare(short_product.failure_stage() == "input_size",
                controls.comparisons)
            && compare(short_product.product().value.empty(),
                controls.comparisons)
            && compare(short_product.work().input_guard_checks == 1U,
                controls.comparisons)
            && compare(short_product.work().kernel_calls == 0U,
                controls.comparisons)
            && compare(short_product.work().kernel_root_derivations == 0U,
                controls.comparisons)
            && compare(short_product.work().receipt_root_derivations == 1U,
                controls.comparisons)
            && compare(short_product.work().result_root_derivations == 1U,
                controls.comparisons)
            && compare(short_product.work().root
                    == admitted_product_work_root(short_product.work(),
                        controls.root_derivation_paths),
                controls.comparisons)
            && compare(short_product.root()
                    == admitted_product_result_root(short_product,
                        controls.root_derivation_paths),
                controls.comparisons)
            && controls_exact;

        ++controls.controls;
        ++controls.compile_time_api_controls;
        constexpr bool api_closed =
            !std::is_default_constructible_v<FormulaProbeAdmittedTangent>
            && !std::is_aggregate_v<FormulaProbeAdmittedTangent>
            && !std::is_move_constructible_v<FormulaProbeAdmittedTangent>
            && !std::is_copy_assignable_v<FormulaProbeAdmittedTangent>
            && !std::is_move_assignable_v<FormulaProbeAdmittedTangent>
            && !std::is_constructible_v<FormulaProbeAdmittedTangent,
                const FormulaProbeParentFixture&>;
        static_assert(api_closed);
        controls_exact = compare(api_closed, controls.comparisons)
            && controls_exact;

        ++controls.controls;
        ++controls.digest_controls;
        std::string mutated_candidate_root = candidate.root;
        mutated_candidate_root[0U] = mutated_candidate_root[0U] == '0'
            ? '1' : '0';
        controls_exact = compare(mutated_candidate_root != candidate.root,
            controls.comparisons) && controls_exact;

        const std::vector<std::pair<RouteInput, std::string>> classifier{
            {RouteInput::None, "ADMITTED_TANGENT_WORK_BOUNDARY_CANDIDATE"},
            {RouteInput::Admission, "ADMITTED_TANGENT_ADMISSION_REJECTED"},
            {RouteInput::Product, "ADMITTED_TANGENT_PRODUCT_REJECTED"},
            {RouteInput::Work, "ADMITTED_TANGENT_WORK_REJECTED"},
            {RouteInput::Control, "INCONCLUSIVE"},
            {RouteInput::Seal, "INCONCLUSIVE"},
            {RouteInput::Unknown, "INCONCLUSIVE"},
        };
        for (const auto& [input, expected] : classifier) {
            ++controls.classifier_cases;
            controls_exact = compare(route(input) == expected,
                controls.comparisons) && controls_exact;
        }
        ++controls.root_derivation_paths;
        controls.root = control_work_root(controls);

        const bool checker_schedule_exact =
            checker.input_shape_comparisons == 7U
            && checker.admission_semantic_comparisons == 9U
            && checker.admission_work_comparisons == 7U
            && checker.product_guard_comparisons == 24U
            && checker.product_component_comparisons == 612U
            && checker.product_work_comparisons == 90U
            && checker.product_root_comparisons == 18U
            && checker.aggregate_work_comparisons == 34U
            && checker.root_derivation_paths == 17U;
        ++checker.result_seals;
        checker.root_derivation_paths += 2U;
        checker.root = checker_work_root(checker);

        RouteInput route_input = RouteInput::None;
        if (!apparatus || !admission_exact) route_input = RouteInput::Admission;
        else if (!products_exact) route_input = RouteInput::Product;
        else if (!work_exact || !checker_schedule_exact)
            route_input = RouteInput::Work;
        else if (!controls_exact) route_input = RouteInput::Control;
        const std::string selected_route = route(route_input);
        const bool passed = route_input == RouteInput::None;

        std::ostringstream semantic;
        semantic << "nextengine.nonlocal.r63zl.v1|" << passed << ':'
            << selected_route << ':' << admission.root() << ':'
            << candidate.root << ':' << reference.root << ':'
            << checker.root << ':' << controls.root << '|';
        for (const std::string& root : admitted_roots)
            semantic << root << ';';
        for (const std::string& root : legacy_roots)
            semantic << root << ';';
        const std::string result_root =
            nextengine::nonlocal::sha256_hex(semantic.str());

        std::cout << "{\"schema\":\"nextengine.nonlocal.r63zl.v1\""
                  << ",\"status\":\"" << (passed ? "PASS" : "FAIL")
                  << "\",\"route\":\"" << selected_route << "\""
                  << ",\"admission\":{\"exact\":"
                  << (admission_exact ? "true" : "false")
                  << ",\"predicate_checks\":"
                  << admission.work().predicate_checks
                  << ",\"tangent_payload_hashes\":"
                  << admission.work().tangent_payload_hashes
                  << ",\"frozen_scalar_parses\":"
                  << admission.work().frozen_scalar_parses
                  << ",\"tangent_components_copied\":"
                  << admission.work().tangent_components_copied
                  << ",\"context_root\":\""
                  << admission.context().context_root()
                  << "\",\"work_root\":\"" << admission.work().root
                  << "\",\"root\":\"" << admission.root() << "\"}"
                  << ",\"candidate_work\":{\"payload_validations\":"
                  << candidate.payload_validations
                  << ",\"tangent_payload_hashes\":"
                  << candidate.tangent_payload_hashes
                  << ",\"kernel_calls\":" << candidate.kernel_calls
                  << ",\"kernel_root_derivations\":"
                  << candidate.kernel_root_derivations
                  << ",\"kernel_guard_checks\":"
                  << candidate.kernel_guard_checks
                  << ",\"dot_guard_checks\":"
                  << candidate.dot_guard_checks
                  << ",\"dot_root_derivations\":"
                  << candidate.dot_root_derivations
                  << ",\"root\":\"" << candidate.root << "\"}"
                  << ",\"reference_work\":{\"payload_validations\":"
                  << reference.payload_validations
                  << ",\"tangent_payload_hashes\":"
                  << reference.tangent_payload_hashes
                  << ",\"frozen_scalar_parses\":"
                  << reference.frozen_scalar_parses
                  << ",\"kernel_calls\":" << reference.kernel_calls
                  << ",\"kernel_guard_checks\":"
                  << reference.kernel_guard_checks
                  << ",\"kernel_root_derivations\":"
                  << reference.kernel_root_derivations
                  << ",\"dot_guard_checks\":"
                  << reference.dot_guard_checks
                  << ",\"dot_root_derivations\":"
                  << reference.dot_root_derivations
                  << ",\"root\":\"" << reference.root << "\"}"
                  << ",\"comparisons\":{\"components\":"
                  << checker.product_component_comparisons
                  << ",\"products_exact\":"
                  << (products_exact ? "true" : "false")
                  << ",\"work_exact\":"
                  << (work_exact && checker_schedule_exact ? "true" : "false")
                  << ",\"checker_root\":\"" << checker.root << "\"}"
                  << ",\"controls\":{\"passed\":"
                  << (controls_exact ? "true" : "false")
                  << ",\"count\":" << controls.controls
                  << ",\"classifier_cases\":"
                  << controls.classifier_cases << ",\"root\":\""
                  << controls.root << "\"}"
                  << ",\"physics_mutation\":false"
                  << ",\"timing_admitted\":false"
                  << ",\"correspondence_authority\":false"
                  << ",\"representation_authority\":false"
                  << ",\"runtime_authority\":false"
                  << ",\"production_authority\":false"
                  << ",\"result_sha256\":\"" << result_root << "\"}\n";
        return passed ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal formula admitted tangent boundary failed: "
                  << error.what() << '\n';
        return 1;
    }
}
