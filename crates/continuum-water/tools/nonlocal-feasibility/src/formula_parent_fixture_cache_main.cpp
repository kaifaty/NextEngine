#include "formula_probe_cache.hpp"

#include <exception>
#include <filesystem>
#include <iostream>
#include <string>

int main(int argc, char** argv) {
    try {
        if (argc != 3
            || (std::string(argv[1]) != "--write"
                && std::string(argv[1]) != "--read")) {
            std::cerr << "usage: nonlocal-formula-parent-fixture-cache "
                         "--write|--read <path>\n";
            return 2;
        }
        using nextengine::nonlocal::fcr::FormulaProbeParentFixture;
        const std::string mode = argv[1];
        const std::string path = argv[2];
        FormulaProbeParentFixture fixture;
        if (mode == "--write") {
            fixture = nextengine::nonlocal::fcr::
                capture_formula_probe_parent_fixture();
            nextengine::nonlocal::fcr::write_formula_probe_parent_fixture(
                path, fixture);
            fixture = nextengine::nonlocal::fcr::
                read_formula_probe_parent_fixture(path);
        } else {
            fixture = nextengine::nonlocal::fcr::
                read_formula_probe_parent_fixture(path);
        }
        const bool exact = fixture.exact
            && nextengine::nonlocal::fcr::
                formula_probe_parent_fixture_valid(fixture);
        std::cout
            << "{\"schema\":\"nextengine.nonlocal.formula_probe_parent_fixture_cache.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"mode\":\"" << mode.substr(2U) << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"bytes\":" << std::filesystem::file_size(path)
            << ",\"external_cache_only\":true"
            << ",\"runtime_authority\":false"
            << ",\"production_authority\":false}"
            << '\n';
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-parent-fixture-cache: "
                  << error.what() << '\n';
        return 2;
    }
}
