#include "formula_probe_cache.hpp"

#include <exception>
#include <iostream>
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

        const bool solve_matches = solve.exact
            && same_values(solve.solution, fixture.baseline_solutions[0U]);
        const bool products_distinct = tangent.exact && common.exact
            && !same_values(tangent.value, common.value);
        const bool certificate_matches = certificate.exact
            && certificate.root == fixture.baseline_certificates[2U].root;
        const bool invalid_rejected = !invalid_product.exact
            && invalid_product.value.empty() && !invalid_solve.exact
            && invalid_solve.solution.empty();
        const bool exact = formula_probe_parent_fixture_valid(fixture)
            && solve_matches && products_distinct && scalar.exact
            && scalar.positive && certificate_matches && invalid_rejected;

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.formula_probe_kernel_smoke.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
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
