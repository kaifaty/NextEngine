#include "formula_probe_cache.hpp"

#include <cmath>
#include <exception>
#include <iostream>
#include <limits>
#include <quadmath.h>
#include <string>
#include <vector>

namespace {

using nextengine::nonlocal::fcr::FormulaProbeBinary128;

bool same_values(const std::vector<FormulaProbeBinary128>& left,
    const std::vector<FormulaProbeBinary128>& right) {
    // Exact equality is intentional: this smoke protects the frozen author
    // fixture boundary, not an approximate physical-quality metric.
    if (left.size() != right.size()) return false;
    for (std::size_t index = 0U; index < left.size(); ++index) {
        if (left[index] != right[index]) return false;
    }
    return true;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-probe-kernel-smoke "
                         "<parent-fixture-cache>\n";
            return 2;
        }
        using namespace nextengine::nonlocal::fcr;
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const FormulaProbeSolve solve = formula_probe_factor_solve(
            fixture, fixture.original_rhs, fixture.inverse_scale);
        const FormulaProbeProduct tangent = formula_probe_tangent_product(
            fixture, fixture.baseline_solutions[0U]);
        const FormulaProbeProduct common = formula_probe_common_product(
            fixture, fixture.baseline_solutions[0U]);
        const FormulaProbeScalar scalar = formula_probe_scalar_dot(
            fixture.original_rhs, fixture.original_rhs);
        const FormulaProbeCertificate certificate = formula_probe_certificate(
            fixture, fixture.baseline_solutions[2U]);

        std::vector<FormulaProbeBinary128> invalid_input =
            fixture.original_rhs;
        invalid_input.pop_back();
        const FormulaProbeProduct invalid_product =
            formula_probe_tangent_product(fixture, invalid_input);
        const FormulaProbeSolve invalid_solve = formula_probe_factor_solve(
            fixture, invalid_input, fixture.inverse_scale);

        FormulaProbeParentFixture tangent_mutation = fixture;
        tangent_mutation.tangent[0U] = nextafterq(
            tangent_mutation.tangent[0U], strtoflt128("inf", nullptr));
        tangent_mutation.tangent_root = formula_probe_binary128_vector_root(
            tangent_mutation.tangent);
        tangent_mutation.root =
            formula_probe_parent_fixture_root(tangent_mutation);
        const FormulaProbeProduct mutated_tangent_product =
            formula_probe_tangent_product(
                tangent_mutation, fixture.baseline_solutions[0U]);

        FormulaProbeParentFixture factor_mutation = fixture;
        factor_mutation.factor_upper[0U] = std::nextafter(
            factor_mutation.factor_upper[0U],
            std::numeric_limits<double>::infinity());
        factor_mutation.factor_root = formula_probe_binary64_vector_root(
            factor_mutation.factor_upper);
        factor_mutation.root =
            formula_probe_parent_fixture_root(factor_mutation);
        const FormulaProbeSolve mutated_factor_solve =
            formula_probe_factor_solve(factor_mutation,
                fixture.original_rhs, factor_mutation.inverse_scale);

        FormulaProbeParentFixture scale_mutation = fixture;
        scale_mutation.inverse_scale = fixture.projected_scale;
        scale_mutation.sigma =
            static_cast<FormulaProbeBinary128>(1.0)
            / scale_mutation.inverse_scale;
        scale_mutation.root =
            formula_probe_parent_fixture_root(scale_mutation);
        const FormulaProbeProduct mutated_scale_product =
            formula_probe_tangent_product(
                scale_mutation, fixture.baseline_solutions[0U]);
        const FormulaProbeSolve mutated_scale_solve =
            formula_probe_factor_solve(scale_mutation,
                fixture.original_rhs, scale_mutation.inverse_scale);

        FormulaProbeParentFixture profile_mutation = fixture;
        profile_mutation.verifier_profile.root += ":mutation";
        profile_mutation.verifier_profile_root =
            profile_mutation.verifier_profile.root;
        profile_mutation.root =
            formula_probe_parent_fixture_root(profile_mutation);
        const FormulaProbeCertificate mutated_profile_certificate =
            formula_probe_certificate(
                profile_mutation, fixture.baseline_solutions[2U]);

        FormulaProbeParentFixture rhs_mutation = fixture;
        rhs_mutation.original_rhs[0U] =
            rhs_mutation.original_rhs[0U]
                    != static_cast<FormulaProbeBinary128>(0.0)
                ? -rhs_mutation.original_rhs[0U]
                : static_cast<FormulaProbeBinary128>(1.0);
        rhs_mutation.original_rhs_root = formula_probe_binary128_vector_root(
            rhs_mutation.original_rhs);
        rhs_mutation.root = formula_probe_parent_fixture_root(rhs_mutation);

        FormulaProbeParentFixture common_mutation = fixture;
        common_mutation.common_components[0U] = std::nextafter(
            common_mutation.common_components[0U],
            std::numeric_limits<double>::infinity());
        common_mutation.common_component_root =
            formula_probe_binary64_vector_root(
                common_mutation.common_components);
        common_mutation.root =
            formula_probe_parent_fixture_root(common_mutation);
        const FormulaProbeProduct mutated_common_product =
            formula_probe_common_product(
                common_mutation, fixture.baseline_solutions[0U]);

        FormulaProbeParentFixture endpoint_mutation = fixture;
        endpoint_mutation.common_projected_solutions[0U][0U] =
            endpoint_mutation.common_projected_solutions[0U][0U]
                    != static_cast<FormulaProbeBinary128>(0.0)
                ? -endpoint_mutation.common_projected_solutions[0U][0U]
                : static_cast<FormulaProbeBinary128>(1.0);
        endpoint_mutation.common_projected_certificates.clear();
        for (const auto& solution :
             endpoint_mutation.common_projected_solutions) {
            endpoint_mutation.common_projected_certificates.push_back(
                formula_probe_certificate(endpoint_mutation, solution));
        }
        endpoint_mutation.root =
            formula_probe_parent_fixture_root(endpoint_mutation);

        const bool solve_matches = solve.exact
            && same_values(solve.solution, fixture.baseline_solutions[0U]);
        const bool products_distinct = tangent.exact && common.exact
            && !same_values(tangent.value, common.value);
        const bool certificate_matches = certificate.exact
            && certificate.root == fixture.baseline_certificates[2U].root;
        const bool invalid_rejected = !invalid_product.exact
            && invalid_product.value.empty() && !invalid_solve.exact
            && invalid_solve.solution.empty();
        const bool resealed_mutations_rejected =
            !formula_probe_parent_fixture_valid(rhs_mutation)
            && !formula_probe_parent_fixture_valid(common_mutation)
            && !mutated_common_product.exact
            && !formula_probe_parent_fixture_valid(endpoint_mutation);
        const bool resealed_kernel_payloads_rejected =
            !formula_probe_parent_fixture_valid(tangent_mutation)
            && !mutated_tangent_product.exact
            && !formula_probe_parent_fixture_valid(factor_mutation)
            && !mutated_factor_solve.exact
            && !formula_probe_parent_fixture_valid(scale_mutation)
            && !mutated_scale_product.exact && !mutated_scale_solve.exact
            && !formula_probe_parent_fixture_valid(profile_mutation)
            && !mutated_profile_certificate.exact;
        const bool exact = formula_probe_parent_fixture_valid(fixture)
            && solve_matches && products_distinct && scalar.exact
            && scalar.positive && certificate_matches && invalid_rejected
            && resealed_mutations_rejected
            && resealed_kernel_payloads_rejected;

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.formula_probe_kernel_smoke.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"original_rhs_root\":\""
            << fixture.original_rhs_root << "\""
            << ",\"projected_rhs_root\":\""
            << fixture.projected_rhs_root << "\""
            << ",\"common_component_root\":\""
            << fixture.common_component_root << "\""
            << ",\"common_solution_set_size\":"
            << fixture.common_projected_solutions.size()
            << ",\"common_certificate_set_root\":\""
            << formula_probe_certificate_set_root(
                fixture.common_projected_certificates) << "\""
            << ",\"common_comparator_root\":\""
            << fixture.common_projected_comparator_root << "\""
            << ",\"solve_root\":\"" << solve.root << "\""
            << ",\"tangent_product_root\":\"" << tangent.root << "\""
            << ",\"common_product_root\":\"" << common.root << "\""
            << ",\"scalar_root\":\"" << scalar.root << "\""
            << ",\"certificate_root\":\"" << certificate.root << "\""
            << ",\"solve_matches_baseline\":"
            << (solve_matches ? "true" : "false")
            << ",\"products_distinct\":"
            << (products_distinct ? "true" : "false")
            << ",\"certificate_matches\":"
            << (certificate_matches ? "true" : "false")
            << ",\"invalid_dimensions_rejected\":"
            << (invalid_rejected ? "true" : "false")
            << ",\"resealed_mutations_rejected\":"
            << (resealed_mutations_rejected ? "true" : "false")
            << ",\"resealed_kernel_payloads_rejected\":"
            << (resealed_kernel_payloads_rejected ? "true" : "false")
            << ",\"runtime_authority\":false"
            << ",\"production_authority\":false}"
            << '\n';
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-probe-kernel-smoke: "
                  << error.what() << '\n';
        return 2;
    }
}
