#include "formula_probe_cache.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <exception>
#include <iostream>
#include <limits>
#include <quadmath.h>
#include <sstream>
#include <string>
#include <vector>

namespace {

using nextengine::nonlocal::fcr::FormulaProbeBinary128;
using nextengine::nonlocal::fcr::FormulaProbeParentFixture;
using nextengine::nonlocal::fcr::FormulaProbeProduct;
using nextengine::nonlocal::fcr::FormulaProbeTwofoldExactAudit;
using nextengine::nonlocal::fcr::FormulaProbeTwofoldOperator;
using nextengine::nonlocal::fcr::FormulaProbeTwofoldRecurrence;

constexpr std::size_t DIMENSION = 102U;
constexpr std::size_t TANGENT_COLUMNS = 315U;
constexpr const char* EXPECTED_SIGN_ROOT =
    "89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094";

std::uint64_t double_bits(double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

struct BuilderWork {
    std::size_t basis_products = 0U;
    std::size_t inner_dots = 0U;
    std::size_t outer_dots = 0U;
    std::size_t inner_terms = 0U;
    std::size_t outer_terms = 0U;
    std::size_t scale_products = 0U;
    std::size_t copied_coefficients = 0U;
};

std::string builder_work_root(const BuilderWork& value) {
    std::ostringstream material;
    material << value.basis_products << ':' << value.inner_dots << ':'
        << value.outer_dots << ':' << value.inner_terms << ':'
        << value.outer_terms << ':' << value.scale_products << ':'
        << value.copied_coefficients;
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool builder_work_valid(const BuilderWork& value) {
    return value.basis_products == 102U && value.inner_dots == 32130U
        && value.outer_dots == 10404U
        && value.inner_terms == 3277260U
        && value.outer_terms == 3277260U
        && value.scale_products == 10404U
        && value.copied_coefficients == 10404U;
}

std::vector<FormulaProbeBinary128> build_materialized_matrix(
    const FormulaProbeParentFixture& fixture, BuilderWork& work,
    bool& exact) {
    std::vector<FormulaProbeBinary128> matrix(
        fixture.dimension * fixture.dimension);
    exact = fixture.dimension == DIMENSION
        && fixture.tangent_columns == TANGENT_COLUMNS;
    for (std::size_t column = 0U;
         column < fixture.dimension && exact; ++column) {
        std::vector<FormulaProbeBinary128> basis(
            fixture.dimension, static_cast<FormulaProbeBinary128>(0.0));
        basis[column] = static_cast<FormulaProbeBinary128>(1.0);
        const FormulaProbeProduct product =
            nextengine::nonlocal::fcr::formula_probe_tangent_product(
                fixture, basis);
        ++work.basis_products;
        work.inner_dots += product.inner_dots;
        work.outer_dots += product.outer_dots;
        work.inner_terms += product.inner_terms;
        work.outer_terms += product.outer_terms;
        work.scale_products += product.scale_products;
        exact = product.exact && product.value.size() == fixture.dimension;
        for (std::size_t row = 0U;
             row < fixture.dimension && exact; ++row) {
            matrix[row * fixture.dimension + column] = product.value[row];
            ++work.copied_coefficients;
        }
    }
    exact = exact && builder_work_valid(work);
    return matrix;
}

bool certificate_ladder(const FormulaProbeTwofoldRecurrence& value) {
    if (value.states.size() != 3U) return false;
    const auto& state0 = value.states[0U].certificate;
    const auto& state1 = value.states[1U].certificate;
    const auto& state2 = value.states[2U].certificate;
    return state0.exact && !state0.passed && state0.unresolved > 0U
        && state1.exact && !state1.passed && state1.unresolved > 0U
        && state2.exact && state2.passed && state2.positive == 24U
        && state2.negative == 78U && state2.unresolved == 0U
        && state2.sign_root == EXPECTED_SIGN_ROOT;
}

bool common_negative(const FormulaProbeTwofoldRecurrence& value) {
    if (!nextengine::nonlocal::fcr::formula_probe_twofold_recurrence_valid(
            value)
        || value.states.size() != 3U)
        return false;
    return std::all_of(value.states.begin(), value.states.end(),
        [](const auto& state) {
            return state.certificate.exact && !state.certificate.passed
                && state.certificate.positive == 12U
                && state.certificate.negative == 24U
                && state.certificate.unresolved == 66U;
        });
}

bool same_state_values(const FormulaProbeTwofoldRecurrence& left,
    const FormulaProbeTwofoldRecurrence& right) {
    if (left.states.size() != right.states.size()) return false;
    for (std::size_t index = 0U; index < left.states.size(); ++index) {
        if (left.states[index].solution_components
                != right.states[index].solution_components
            || left.states[index].residual_components
                != right.states[index].residual_components
            || left.states[index].direction_components
                != right.states[index].direction_components
            || left.states[index].preconditioned_components
                != right.states[index].preconditioned_components
            || left.states[index].certificate.root
                != right.states[index].certificate.root)
            return false;
    }
    return true;
}

std::string classify(bool apparatus, bool identity, bool operator_audit,
    bool recurrence, bool positivity, bool verifier, bool ladder,
    bool common_control, bool controls) {
    if (!apparatus) return "R63ZI_APPARATUS_REJECTED";
    if (!identity) return "R63ZI_IDENTITY_REJECTED";
    if (!operator_audit) return "R63ZI_OPERATOR_AUDIT_REJECTED";
    if (!recurrence) return "R63ZI_RECURRENCE_REJECTED";
    if (!positivity) return "R63ZI_POSITIVITY_REJECTED";
    if (!verifier) return "R63ZI_VERIFIER_REJECTED";
    if (!ladder) return "R63ZI_STATE2_REJECTED";
    if (!common_control) return "R63ZI_COMMON_NEGATIVE_REJECTED";
    if (!controls) return "R63ZI_CONTROLS_REJECTED";
    return "TANGENT_GRAM_TWOFOLD_STATE2_CANDIDATE";
}

bool classifier_control() {
    const std::array<std::string, 10U> expected{
        "R63ZI_APPARATUS_REJECTED", "R63ZI_IDENTITY_REJECTED",
        "R63ZI_OPERATOR_AUDIT_REJECTED", "R63ZI_RECURRENCE_REJECTED",
        "R63ZI_POSITIVITY_REJECTED", "R63ZI_VERIFIER_REJECTED",
        "R63ZI_STATE2_REJECTED", "R63ZI_COMMON_NEGATIVE_REJECTED",
        "R63ZI_CONTROLS_REJECTED",
        "TANGENT_GRAM_TWOFOLD_STATE2_CANDIDATE"};
    for (std::size_t first_false = 0U;
         first_false < expected.size(); ++first_false) {
        std::array<bool, 9U> values{};
        values.fill(true);
        if (first_false < values.size()) values[first_false] = false;
        if (classify(values[0U], values[1U], values[2U], values[3U],
                values[4U], values[5U], values[6U], values[7U],
                values[8U]) != expected[first_false])
            return false;
    }
    return true;
}

std::string result_root(const std::string& route,
    const FormulaProbeParentFixture& fixture,
    const FormulaProbeTwofoldOperator& artifact,
    const FormulaProbeTwofoldRecurrence& recurrence,
    const FormulaProbeTwofoldExactAudit& audit,
    const FormulaProbeTwofoldRecurrence& common,
    const std::string& builder_root, const std::string& controls_root) {
    std::ostringstream material;
    material << route << ':' << fixture.root << ':' << artifact.root << ':'
        << recurrence.root << ':' << audit.root << ':' << common.root << ':'
        << builder_root << ':' << controls_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-tangent-twofold-recurrence "
                         "<parent-cache>\n";
            return 2;
        }
        using namespace nextengine::nonlocal::fcr;
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const bool fixture_exact =
            formula_probe_parent_fixture_valid(fixture);
        BuilderWork builder_work;
        bool materialization_exact = false;
        const std::vector<FormulaProbeBinary128> materialized =
            build_materialized_matrix(
                fixture, builder_work, materialization_exact);
        const FormulaProbeTwofoldOperator artifact =
            formula_probe_tangent_twofold_operator(fixture, materialized);
        const bool artifact_exact =
            formula_probe_twofold_operator_valid(artifact);
        const FormulaProbeTwofoldRecurrence recurrence =
            formula_probe_twofold_recurrence(fixture, artifact);
        const bool recurrence_exact =
            formula_probe_twofold_recurrence_valid(recurrence);

        // The exact oracle is deliberately invoked only after the candidate
        // transaction and certificates have been sealed.
        const FormulaProbeTwofoldExactAudit audit =
            formula_probe_twofold_exact_audit(
                fixture, artifact, recurrence);
        const bool audit_exact =
            formula_probe_twofold_exact_audit_valid(audit);
        const bool ladder = certificate_ladder(recurrence);

        const FormulaProbeTwofoldOperator common_artifact =
            formula_probe_common_twofold_operator(fixture);
        const FormulaProbeTwofoldRecurrence common =
            formula_probe_twofold_recurrence(fixture, common_artifact);
        const bool common_exact = common_negative(common);

        FormulaProbeTwofoldOperator stale = artifact;
        stale.components[0U] = std::nextafter(
            stale.components[0U], std::numeric_limits<double>::infinity());
        const bool stale_control = !formula_probe_twofold_operator_valid(stale)
            && !formula_probe_twofold_recurrence(fixture, stale).exact;

        std::vector<FormulaProbeBinary128> coefficient_mutation = materialized;
        coefficient_mutation[0U] = nextafterq(
            coefficient_mutation[0U], HUGE_VALQ);
        const FormulaProbeTwofoldOperator resealed_coefficient =
            formula_probe_tangent_twofold_operator(
                fixture, coefficient_mutation);
        const FormulaProbeTwofoldRecurrence coefficient_recurrence =
            formula_probe_twofold_recurrence(fixture, resealed_coefficient);
        const bool coefficient_control =
            formula_probe_twofold_operator_valid(resealed_coefficient)
            && formula_probe_twofold_recurrence_valid(coefficient_recurrence)
            && resealed_coefficient.root != artifact.root
            && coefficient_recurrence.root != recurrence.root;

        std::vector<FormulaProbeBinary128> dropped_low(materialized.size());
        for (std::size_t index = 0U; index < dropped_low.size(); ++index)
            dropped_low[index] = static_cast<FormulaProbeBinary128>(
                artifact.components[2U * index]);
        const FormulaProbeTwofoldOperator dropped_low_artifact =
            formula_probe_tangent_twofold_operator(fixture, dropped_low);
        const FormulaProbeTwofoldRecurrence dropped_low_recurrence =
            formula_probe_twofold_recurrence(fixture, dropped_low_artifact);
        const bool dropped_low_control =
            formula_probe_twofold_operator_valid(dropped_low_artifact)
            && dropped_low_artifact.root != artifact.root
            && dropped_low_recurrence.root != recurrence.root;

        const double wider = std::nextafter(artifact.maximum_radius,
            std::numeric_limits<double>::infinity());
        const FormulaProbeTwofoldOperator widened =
            formula_probe_widen_twofold_operator_radius(artifact, wider);
        const FormulaProbeTwofoldRecurrence widened_recurrence =
            formula_probe_twofold_recurrence(fixture, widened);
        const bool radius_control = formula_probe_twofold_operator_valid(widened)
            && formula_probe_twofold_recurrence_valid(widened_recurrence)
            && widened.root != artifact.root
            && widened_recurrence.root != recurrence.root
            && same_state_values(widened_recurrence, recurrence);

        std::vector<FormulaProbeBinary128> nonfinite = materialized;
        nonfinite[0U] = HUGE_VALQ;
        const bool nonfinite_control =
            !formula_probe_tangent_twofold_operator(fixture, nonfinite).exact;
        FormulaProbeTwofoldOperator bad_dimension = artifact;
        ++bad_dimension.dimension;
        FormulaProbeTwofoldOperator bad_source = artifact;
        bad_source.source_root =
            nextengine::nonlocal::sha256_hex(bad_source.source_root + ":bad");
        const bool identity_controls =
            !formula_probe_twofold_operator_valid(bad_dimension)
            && !formula_probe_twofold_operator_valid(bad_source);
        FormulaProbeTwofoldRecurrence bad_result = recurrence;
        bad_result.root =
            nextengine::nonlocal::sha256_hex(bad_result.root + ":bad");
        const bool result_sealing =
            !formula_probe_twofold_recurrence_valid(bad_result);
        const bool classifier = classifier_control();
        const bool controls = stale_control && coefficient_control
            && dropped_low_control && radius_control && nonfinite_control
            && identity_controls && result_sealing && classifier;
        std::ostringstream controls_material;
        controls_material << stale_control << ':' << coefficient_control << ':'
            << dropped_low_control << ':' << radius_control << ':'
            << nonfinite_control << ':' << identity_controls << ':'
            << result_sealing << ':' << classifier << ':'
            << resealed_coefficient.root << ':'
            << coefficient_recurrence.root << ':'
            << dropped_low_artifact.root << ':'
            << dropped_low_recurrence.root << ':' << widened.root << ':'
            << widened_recurrence.root;
        const std::string controls_root =
            nextengine::nonlocal::sha256_hex(controls_material.str());

        const bool apparatus = fixture_exact && materialization_exact
            && builder_work_valid(builder_work);
        const bool identity = artifact_exact
            && artifact.source_root == audit.exact_operator_root;
        const bool verifier = recurrence_exact
            && std::all_of(recurrence.states.begin(), recurrence.states.end(),
                [](const auto& state) { return state.certificate.exact; });
        const std::string route = classify(apparatus, identity, audit_exact,
            recurrence_exact, recurrence.positivity_exact, verifier, ladder,
            common_exact, controls);
        const bool candidate = route
            == "TANGENT_GRAM_TWOFOLD_STATE2_CANDIDATE";
        const bool exact = candidate || route == "R63ZI_STATE2_REJECTED";
        const std::string builder_root = builder_work_root(builder_work);
        const std::string final_root = result_root(route, fixture, artifact,
            recurrence, audit, common, builder_root, controls_root);

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zi_tangent_twofold_recurrence.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"candidate\":" << (candidate ? "true" : "false")
            << ",\"claim_status\":\""
            << (candidate ? "AUTHOR_PASS_REVIEW_NOT_TESTED"
                          : "AUTHOR_PASS_REVIEW_NOT_TESTED_BOUNDED_REJECTED")
            << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"operator\":{\"source_root\":\"" << artifact.source_root
            << "\",\"materialization_root\":\""
            << artifact.materialization_root << "\",\"component_root\":\""
            << artifact.component_root << "\",\"radius_root\":\""
            << artifact.radius_root << "\",\"root\":\"" << artifact.root
            << "\",\"containments\":" << artifact.containments
            << ",\"entries\":" << artifact.entries
            << ",\"maximum_radius_bits\":"
            << double_bits(artifact.maximum_radius) << "}"
            << ",\"recurrence\":{\"root\":\"" << recurrence.root
            << "\",\"transaction_root\":\""
            << recurrence.transaction_root << "\",\"certificates\":[";
        for (std::size_t index = 0U; index < recurrence.states.size(); ++index) {
            if (index != 0U) std::cout << ',';
            const auto& certificate = recurrence.states[index].certificate;
            std::cout << "{\"state\":" << index << ",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"positive\":" << certificate.positive
                << ",\"negative\":" << certificate.negative
                << ",\"unresolved\":" << certificate.unresolved
                << ",\"solution_component_root\":\""
                << formula_probe_binary64_vector_root(
                    recurrence.states[index].solution_components)
                << "\""
                << ",\"sign_root\":\"" << certificate.sign_root
                << "\",\"root\":\"" << certificate.root << "\"}";
        }
        std::cout
            << "]}"
            << ",\"exact_audit\":{\"root\":\"" << audit.root
            << "\",\"operator_containments\":"
            << audit.operator_containments << ",\"operator_entries\":"
            << audit.operator_entries << ",\"product_containments\":"
            << audit.product_containments << ",\"product_rows\":"
            << audit.product_rows << ",\"work_root\":\"" << audit.work_root
            << "\"}"
            << ",\"common_negative\":{\"exact\":"
            << (common_exact ? "true" : "false") << ",\"root\":\""
            << common.root << "\",\"state_values_equal_to_candidate\":"
            << (same_state_values(common, recurrence) ? "true" : "false")
            << "}"
            << ",\"work\":{\"builder_root\":\"" << builder_root
            << "\",\"basis_products\":" << builder_work.basis_products
            << ",\"builder_dots\":"
            << builder_work.inner_dots + builder_work.outer_dots
            << ",\"builder_terms\":"
            << builder_work.inner_terms + builder_work.outer_terms
            << ",\"builder_scale_products\":"
            << builder_work.scale_products << ",\"exact_upper_dots\":"
            << audit.exact_upper_dots << ",\"exact_gram_products\":"
            << audit.exact_gram_products << ",\"exact_mirrors\":"
            << audit.exact_mirrors << ",\"operator_products\":"
            << recurrence.work.operator_products << ",\"factor_solves\":"
            << recurrence.work.factor_solves << "}"
            << ",\"controls\":{\"stale\":"
            << (stale_control ? "true" : "false")
            << ",\"resealed_coefficient\":"
            << (coefficient_control ? "true" : "false")
            << ",\"dropped_low\":"
            << (dropped_low_control ? "true" : "false")
            << ",\"radius\":" << (radius_control ? "true" : "false")
            << ",\"nonfinite\":"
            << (nonfinite_control ? "true" : "false")
            << ",\"identity\":"
            << (identity_controls ? "true" : "false")
            << ",\"result_sealing\":"
            << (result_sealing ? "true" : "false")
            << ",\"classifier\":" << (classifier ? "true" : "false")
            << ",\"root\":\"" << controls_root << "\"}"
            << ",\"author_release_repeats_required\":2"
            << ",\"review_status\":\"NOT_TESTED\""
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << final_root << "\"}\n";
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-tangent-twofold-recurrence: "
                  << error.what() << '\n';
        return 2;
    }
}
