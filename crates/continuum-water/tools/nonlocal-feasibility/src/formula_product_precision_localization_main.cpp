#include "formula_probe_cache.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <exception>
#include <iostream>
#include <limits>
#include <quadmath.h>
#include <sstream>
#include <string>
#include <vector>

namespace {

using nextengine::nonlocal::fcr::FormulaProbeBinary128;
using nextengine::nonlocal::fcr::FormulaProbeCertificate;
using nextengine::nonlocal::fcr::FormulaProbeHybridRecurrence;
using nextengine::nonlocal::fcr::FormulaProbeParentFixture;
using nextengine::nonlocal::fcr::FormulaProbeProduct;
using nextengine::nonlocal::fcr::FormulaProbeTwofoldOperator;
using nextengine::nonlocal::fcr::FormulaProbeTwofoldProductAudit;
using nextengine::nonlocal::fcr::FormulaProbeTwofoldRecurrence;

constexpr std::size_t DIMENSION = 102U;
constexpr const char* EXPECTED_SIGN_ROOT =
    "89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094";

struct BuilderWork {
    std::size_t products = 0U;
    std::size_t dots = 0U;
    std::size_t terms = 0U;
    std::size_t scale_products = 0U;
};

std::string builder_root(const BuilderWork& value) {
    std::ostringstream material;
    material << value.products << ':' << value.dots << ':' << value.terms
        << ':' << value.scale_products;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::vector<FormulaProbeBinary128> build_dense_parent(
    const FormulaProbeParentFixture& fixture, BuilderWork& work,
    bool& exact) {
    std::vector<FormulaProbeBinary128> matrix(
        fixture.dimension * fixture.dimension);
    exact = fixture.dimension == DIMENSION;
    for (std::size_t column = 0U;
         column < fixture.dimension && exact; ++column) {
        std::vector<FormulaProbeBinary128> basis(
            fixture.dimension, static_cast<FormulaProbeBinary128>(0.0));
        basis[column] = static_cast<FormulaProbeBinary128>(1.0);
        const FormulaProbeProduct product =
            nextengine::nonlocal::fcr::formula_probe_tangent_product(
                fixture, basis);
        ++work.products;
        work.dots += product.inner_dots + product.outer_dots;
        work.terms += product.inner_terms + product.outer_terms;
        work.scale_products += product.scale_products;
        exact = product.exact && product.value.size() == fixture.dimension;
        for (std::size_t row = 0U;
             row < fixture.dimension && exact; ++row)
            matrix[row * fixture.dimension + column] = product.value[row];
    }
    exact = exact && work.products == 102U && work.dots == 42534U
        && work.terms == 6554520U && work.scale_products == 10404U;
    return matrix;
}

bool ladder(const std::vector<FormulaProbeCertificate>& certificates) {
    return certificates.size() == 3U
        && certificates[0U].exact && !certificates[0U].passed
        && certificates[0U].unresolved > 0U
        && certificates[1U].exact && !certificates[1U].passed
        && certificates[1U].unresolved > 0U
        && certificates[2U].exact && certificates[2U].passed
        && certificates[2U].positive == 24U
        && certificates[2U].negative == 78U
        && certificates[2U].unresolved == 0U
        && certificates[2U].sign_root == EXPECTED_SIGN_ROOT;
}

std::vector<FormulaProbeCertificate> certificates(
    const FormulaProbeTwofoldRecurrence& recurrence) {
    std::vector<FormulaProbeCertificate> result;
    for (const auto& state : recurrence.states)
        result.push_back(state.certificate);
    return result;
}

bool rejecting_endpoint(const FormulaProbeTwofoldRecurrence& recurrence) {
    return nextengine::nonlocal::fcr::formula_probe_twofold_recurrence_valid(
            recurrence)
        && recurrence.states.size() == 3U
        && std::all_of(recurrence.states.begin(), recurrence.states.end(),
            [](const auto& state) {
                return state.certificate.exact && !state.certificate.passed
                    && state.certificate.positive == 12U
                    && state.certificate.negative == 24U
                    && state.certificate.unresolved == 66U;
            });
}

std::string classify(bool apparatus, bool identity, bool work,
    bool product_audit, bool wide_parent, bool dense_parent,
    bool common_parent, bool controls, bool hybrid_ladder) {
    if (!apparatus) return "R63ZJ_APPARATUS_REJECTED";
    if (!identity) return "R63ZJ_IDENTITY_REJECTED";
    if (!work) return "R63ZJ_WORK_REJECTED";
    if (!product_audit) return "R63ZJ_PRODUCT_AUDIT_REJECTED";
    if (!wide_parent) return "R63ZJ_WIDE_PARENT_REJECTED";
    if (!dense_parent) return "R63ZJ_DENSE_PARENT_REJECTED";
    if (!common_parent) return "R63ZJ_COMMON_PARENT_REJECTED";
    if (!controls) return "R63ZJ_CONTROLS_REJECTED";
    return hybrid_ladder
        ? "DENSE_TWOFOLD_OPERATOR_PRODUCT_PRECISION_INSUFFICIENT"
        : "TWOFOLD_PRODUCT_PROJECTION_OR_RECURRENCE_INSUFFICIENT";
}

bool classifier_control() {
    const std::array<std::string, 8U> failures{
        "R63ZJ_APPARATUS_REJECTED", "R63ZJ_IDENTITY_REJECTED",
        "R63ZJ_WORK_REJECTED", "R63ZJ_PRODUCT_AUDIT_REJECTED",
        "R63ZJ_WIDE_PARENT_REJECTED", "R63ZJ_DENSE_PARENT_REJECTED",
        "R63ZJ_COMMON_PARENT_REJECTED", "R63ZJ_CONTROLS_REJECTED"};
    for (std::size_t index = 0U; index < failures.size(); ++index) {
        std::array<bool, 8U> values{};
        values.fill(true);
        values[index] = false;
        if (classify(values[0U], values[1U], values[2U], values[3U],
                values[4U], values[5U], values[6U], values[7U], false)
            != failures[index])
            return false;
    }
    return classify(true, true, true, true, true, true, true, true, true)
            == "DENSE_TWOFOLD_OPERATOR_PRODUCT_PRECISION_INSUFFICIENT"
        && classify(true, true, true, true, true, true, true, true, false)
            == "TWOFOLD_PRODUCT_PROJECTION_OR_RECURRENCE_INSUFFICIENT";
}

std::string result_root(const std::string& route,
    const FormulaProbeParentFixture& fixture,
    const FormulaProbeHybridRecurrence& hybrid,
    const FormulaProbeTwofoldProductAudit& audit,
    const FormulaProbeTwofoldRecurrence& dense,
    const FormulaProbeTwofoldRecurrence& common,
    const std::string& build_root, const std::string& controls_root) {
    std::ostringstream material;
    material << route << ':' << fixture.root << ':' << hybrid.root << ':'
        << audit.root << ':' << dense.root << ':' << common.root << ':'
        << build_root << ':' << controls_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-product-precision-localization "
                         "<parent-cache>\n";
            return 2;
        }
        using namespace nextengine::nonlocal::fcr;
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const bool fixture_exact =
            formula_probe_parent_fixture_valid(fixture);

        const FormulaProbeHybridRecurrence hybrid =
            formula_probe_hybrid_twofold_recurrence(fixture);
        const bool hybrid_exact =
            formula_probe_hybrid_twofold_recurrence_valid(hybrid);
        const FormulaProbeTwofoldProductAudit audit =
            formula_probe_twofold_product_exact_audit(
                fixture, hybrid.recurrence);
        const bool audit_exact =
            formula_probe_twofold_product_exact_audit_valid(audit);
        const std::vector<FormulaProbeCertificate> hybrid_certificates =
            certificates(hybrid.recurrence);
        const bool hybrid_ladder = ladder(hybrid_certificates);

        BuilderWork build_work;
        bool build_exact = false;
        const std::vector<FormulaProbeBinary128> materialized =
            build_dense_parent(fixture, build_work, build_exact);
        const FormulaProbeTwofoldOperator dense_artifact =
            formula_probe_tangent_twofold_operator(fixture, materialized);
        const FormulaProbeTwofoldRecurrence dense =
            formula_probe_twofold_recurrence(fixture, dense_artifact);
        const bool dense_reject = rejecting_endpoint(dense);
        const FormulaProbeTwofoldOperator common_artifact =
            formula_probe_common_twofold_operator(fixture);
        const FormulaProbeTwofoldRecurrence common =
            formula_probe_twofold_recurrence(fixture, common_artifact);
        const bool common_reject = rejecting_endpoint(common);
        const bool wide_parent = ladder(fixture.baseline_certificates);

        FormulaProbeParentFixture tangent_mutation = fixture;
        tangent_mutation.tangent[0U] = nextafterq(
            tangent_mutation.tangent[0U], HUGE_VALQ);
        const bool tangent_control =
            !formula_probe_hybrid_twofold_recurrence(tangent_mutation).exact;
        FormulaProbeParentFixture nonfinite = fixture;
        nonfinite.tangent[0U] = HUGE_VALQ;
        const bool nonfinite_control =
            !formula_probe_hybrid_twofold_recurrence(nonfinite).exact;
        FormulaProbeHybridRecurrence work_mutation = hybrid;
        ++work_mutation.hybrid_work.output_projections;
        const bool work_control =
            !formula_probe_hybrid_twofold_recurrence_valid(work_mutation);
        FormulaProbeHybridRecurrence result_mutation = hybrid;
        result_mutation.root = nextengine::nonlocal::sha256_hex(
            result_mutation.root + ":bad");
        const bool result_control =
            !formula_probe_hybrid_twofold_recurrence_valid(result_mutation);
        FormulaProbeTwofoldRecurrence product_mutation = hybrid.recurrence;
        bool product_control = false;
        if (!product_mutation.products.empty()
            && !product_mutation.products[0U].value_components.empty()) {
            product_mutation.products[0U].value_components[0U] =
                std::nextafter(
                    product_mutation.products[0U].value_components[0U],
                    std::numeric_limits<double>::infinity());
            product_control = !formula_probe_twofold_product_exact_audit(
                fixture, product_mutation).exact;
        }
        const bool classifier = classifier_control();
        const bool controls = tangent_control && nonfinite_control
            && work_control && result_control && product_control && classifier;
        std::ostringstream controls_material;
        controls_material << tangent_control << ':' << nonfinite_control << ':'
            << work_control << ':' << result_control << ':' << product_control
            << ':' << classifier;
        const std::string controls_root =
            nextengine::nonlocal::sha256_hex(controls_material.str());

        const bool work = hybrid.hybrid_work.operator_products == 3U
            && hybrid.hybrid_work.tangent_kernel_calls == 6U
            && hybrid.hybrid_work.tangent_inner_dots == 1890U
            && hybrid.hybrid_work.tangent_outer_dots == 612U
            && hybrid.hybrid_work.tangent_inner_terms == 192780U
            && hybrid.hybrid_work.tangent_outer_terms == 192780U
            && hybrid.hybrid_work.tangent_scale_products == 612U
            && hybrid.hybrid_work.input_components == 612U
            && hybrid.hybrid_work.output_projections == 612U
            && hybrid.hybrid_work.output_additions == 306U;
        const bool apparatus = fixture_exact && hybrid_exact && build_exact;
        const bool identity = hybrid.recurrence.fixture_root == fixture.root
            && !hybrid.callback_root.empty()
            && audit.exact_operator_root == dense_artifact.source_root;
        const std::string route = classify(apparatus, identity, work,
            audit_exact, wide_parent, dense_reject, common_reject, controls,
            hybrid_ladder);
        const bool exact = route
                == "DENSE_TWOFOLD_OPERATOR_PRODUCT_PRECISION_INSUFFICIENT"
            || route
                == "TWOFOLD_PRODUCT_PROJECTION_OR_RECURRENCE_INSUFFICIENT";
        const std::string dense_builder_root = builder_root(build_work);
        const std::string final_root = result_root(route, fixture, hybrid,
            audit, dense, common, dense_builder_root, controls_root);

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zj_product_precision_localization.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"hybrid\":{\"root\":\"" << hybrid.root
            << "\",\"callback_root\":\"" << hybrid.callback_root
            << "\",\"recurrence_root\":\"" << hybrid.recurrence.root
            << "\",\"exact\":" << (hybrid.exact ? "true" : "false")
            << ",\"complete\":"
            << (hybrid.recurrence.complete ? "true" : "false")
            << ",\"positivity\":"
            << (hybrid.recurrence.positivity_exact ? "true" : "false")
            << ",\"failure_stage\":\""
            << hybrid.recurrence.failure_stage << "\""
            << ",\"products\":" << hybrid.recurrence.products.size()
            << ",\"ladder\":" << (hybrid_ladder ? "true" : "false")
            << ",\"certificates\":[";
        for (std::size_t index = 0U;
             index < hybrid_certificates.size(); ++index) {
            if (index != 0U) std::cout << ',';
            const FormulaProbeCertificate& certificate =
                hybrid_certificates[index];
            std::cout << "{\"state\":" << index << ",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"positive\":" << certificate.positive
                << ",\"negative\":" << certificate.negative
                << ",\"unresolved\":" << certificate.unresolved
                << ",\"sign_root\":\"" << certificate.sign_root
                << "\",\"root\":\"" << certificate.root << "\"}";
        }
        std::cout
            << "]}"
            << ",\"product_audit\":{\"containments\":"
            << audit.containments << ",\"rows\":" << audit.rows
            << ",\"root\":\"" << audit.root << "\",\"work_root\":\""
            << audit.work_root << "\"}"
            << ",\"endpoints\":{\"wide_tangent_pass\":"
            << (wide_parent ? "true" : "false")
            << ",\"dense_k2_reject\":"
            << (dense_reject ? "true" : "false")
            << ",\"dense_root\":\"" << dense.root
            << "\",\"common_k2_reject\":"
            << (common_reject ? "true" : "false")
            << ",\"common_root\":\"" << common.root << "\"}"
            << ",\"work\":{\"operator_products\":"
            << hybrid.hybrid_work.operator_products
            << ",\"tangent_kernel_calls\":"
            << hybrid.hybrid_work.tangent_kernel_calls
            << ",\"tangent_inner_dots\":"
            << hybrid.hybrid_work.tangent_inner_dots
            << ",\"tangent_outer_dots\":"
            << hybrid.hybrid_work.tangent_outer_dots
            << ",\"tangent_inner_terms\":"
            << hybrid.hybrid_work.tangent_inner_terms
            << ",\"tangent_outer_terms\":"
            << hybrid.hybrid_work.tangent_outer_terms
            << ",\"scale_products\":"
            << hybrid.hybrid_work.tangent_scale_products
            << ",\"input_components\":"
            << hybrid.hybrid_work.input_components
            << ",\"output_projections\":"
            << hybrid.hybrid_work.output_projections
            << ",\"output_additions\":"
            << hybrid.hybrid_work.output_additions
            << ",\"dense_builder_root\":\"" << dense_builder_root
            << "\"}"
            << ",\"controls\":{\"tangent\":"
            << (tangent_control ? "true" : "false")
            << ",\"nonfinite\":"
            << (nonfinite_control ? "true" : "false")
            << ",\"work\":" << (work_control ? "true" : "false")
            << ",\"result\":" << (result_control ? "true" : "false")
            << ",\"product\":" << (product_control ? "true" : "false")
            << ",\"classifier\":" << (classifier ? "true" : "false")
            << ",\"root\":\"" << controls_root << "\"}"
            << ",\"claim_status\":\"AUTHOR_PASS_REVIEW_NOT_TESTED\""
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << final_root << "\"}\n";
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-product-precision-localization: "
                  << error.what() << '\n';
        return 2;
    }
}
