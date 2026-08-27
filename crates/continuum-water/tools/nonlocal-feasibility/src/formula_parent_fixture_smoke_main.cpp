#include "formula_probe_api.hpp"

#include <cmath>
#include <exception>
#include <iomanip>
#include <iostream>
#include <limits>

int main() {
    try {
        using nextengine::nonlocal::fcr::FormulaProbeParentFixture;
        using nextengine::nonlocal::fcr::capture_formula_probe_parent_fixture;
        using nextengine::nonlocal::fcr::formula_probe_parent_fixture_valid;
        const FormulaProbeParentFixture fixture =
            capture_formula_probe_parent_fixture();
        const bool fixture_valid =
            formula_probe_parent_fixture_valid(fixture);
        FormulaProbeParentFixture mutation = fixture;
        if (!mutation.factor_upper.empty())
            mutation.factor_upper[0U] = std::nextafter(
                mutation.factor_upper[0U],
                std::numeric_limits<double>::infinity());
        const bool exact = fixture.exact && fixture_valid
            && !mutation.factor_upper.empty()
            && !formula_probe_parent_fixture_valid(mutation);
        std::cout
            << "{\"schema\":\"nextengine.nonlocal.formula_probe_parent_fixture_smoke.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"capture_exact\":"
            << (fixture.exact ? "true" : "false")
            << ",\"validator_exact\":"
            << (fixture_valid ? "true" : "false")
            << ",\"dimension\":" << fixture.dimension
            << ",\"tangent_values\":" << fixture.tangent.size()
            << ",\"common_components\":"
            << fixture.common_components.size()
            << ",\"profile_components\":"
            << fixture.verifier_profile.vector_components.size()
                + fixture.verifier_profile.matrix_components.size()
            << ",\"baseline_states\":"
            << fixture.baseline_solutions.size()
            << ",\"combined_states\":"
            << fixture.common_projected_solutions.size()
            << ",\"mutation_rejected\":"
            << (!formula_probe_parent_fixture_valid(mutation)
                    ? "true" : "false")
            << ",\"runtime_authority\":false"
            << ",\"production_authority\":false}"
            << '\n';
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-parent-fixture-smoke: "
                  << error.what() << '\n';
        return 2;
    }
}
